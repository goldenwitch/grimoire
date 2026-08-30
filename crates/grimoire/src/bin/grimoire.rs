use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

use grimoire::{
    Address, ResourceBundle, ResourceCharge, ResourceFlow, ResourceKind, ResourceModel,
    ResourceReport, ResourceScenario, evaluate_layer, extract_cut, parse_description,
    prototype_schemas, serialize_description, validate_description,
};

const USAGE: &str = "usage:
  grimoire validate <description|->
  grimoire canonicalize <description|->
  grimoire evaluate <description|-> <layer>
  grimoire cut <description|-> <layer> [<layer> ...]
  grimoire resources <description|-> <layer> <events.tsv>

The events file is tab-delimited:
  scenario\tNAME\tPROBABILITY\tASSUMPTION
  flow\tSCENARIO\tRELATION\tSOURCE\tDESTINATION\tRESOURCE\tQUANTITY
  charge\tSCENARIO\tTARGET\tRESOURCE\tQUANTITY";

#[derive(Debug)]
struct CliError(String);

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for CliError {}

fn main() {
    if let Err(error) = run() {
        eprintln!("grimoire: {error}");
        eprintln!("{USAGE}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let mut arguments = env::args().skip(1);
    let Some(command) = arguments.next() else {
        return Err(CliError("a command is required".to_owned()));
    };
    match command.as_str() {
        "validate" => {
            validated_description(&read_text(&required(&mut arguments, "description")?)?)?;
            ensure_no_extra(&mut arguments)?;
            println!("valid");
        }
        "canonicalize" => {
            let source = read_text(&required(&mut arguments, "description")?)?;
            ensure_no_extra(&mut arguments)?;
            let description = validated_description(&source)?;
            let canonical = serialize_description(&description).map_err(error)?;
            io::stdout()
                .write_all(canonical.as_bytes())
                .map_err(error)?;
        }
        "evaluate" => {
            let source = read_text(&required(&mut arguments, "description")?)?;
            let layer = required(&mut arguments, "layer")?;
            ensure_no_extra(&mut arguments)?;
            let description = validated_description(&source)?;
            let result = evaluate_layer(&description, &layer).map_err(error)?;
            println!("layer={layer}");
            println!("elements={}", result.structural.elements.len());
            println!("decorations={}", result.decorations.len());
            println!("checks={}", result.checks.len());
            for check in result.checks {
                println!(
                    "check\tname={}\tpassed={}\tobserved={}",
                    check.name, check.passed, check.observed
                );
            }
        }
        "cut" => {
            let source = read_text(&required(&mut arguments, "description")?)?;
            let layers: Vec<String> = arguments.collect();
            if layers.is_empty() {
                return Err(CliError("at least one layer is required".to_owned()));
            }
            let description = validated_description(&source)?;
            let layer_names: Vec<&str> = layers.iter().map(String::as_str).collect();
            let schemas = prototype_schemas().map_err(error)?;
            let cut = extract_cut(&description, &layer_names, &schemas).map_err(error)?;
            let canonical = serialize_description(&cut).map_err(error)?;
            io::stdout()
                .write_all(canonical.as_bytes())
                .map_err(error)?;
        }
        "resources" => {
            let source = read_text(&required(&mut arguments, "description")?)?;
            let layer = required(&mut arguments, "layer")?;
            let events_path = required(&mut arguments, "events")?;
            ensure_no_extra(&mut arguments)?;
            let description = validated_description(&source)?;
            let reprojection = evaluate_layer(&description, &layer)
                .map_err(error)?
                .structural;
            let events = read_text(&events_path)?;
            let model = parse_events(&events)?;
            let report = model.evaluate(&reprojection).map_err(error)?;
            print_resource_report(&report);
        }
        "help" | "--help" | "-h" => println!("{USAGE}"),
        unknown => return Err(CliError(format!("unknown command `{unknown}`"))),
    }
    Ok(())
}

fn required(arguments: &mut impl Iterator<Item = String>, name: &str) -> Result<String, CliError> {
    arguments
        .next()
        .ok_or_else(|| CliError(format!("{name} is required")))
}

fn ensure_no_extra(arguments: &mut impl Iterator<Item = String>) -> Result<(), CliError> {
    if let Some(extra) = arguments.next() {
        return Err(CliError(format!("unexpected argument `{extra}`")));
    }
    Ok(())
}

fn read_text(path: &str) -> Result<String, CliError> {
    if path == "-" {
        let mut source = String::new();
        io::stdin().read_to_string(&mut source).map_err(error)?;
        return Ok(source);
    }
    fs::read_to_string(Path::new(path)).map_err(error)
}

fn validated_description(source: &str) -> Result<grimoire::Description, CliError> {
    let description = parse_description(source).map_err(error)?;
    let schemas = prototype_schemas().map_err(error)?;
    validate_description(&description, &schemas)
        .map_err(|errors| CliError(format_validation_errors(&errors)))?;
    Ok(description)
}

fn format_validation_errors(errors: &[grimoire::ValidationError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Default)]
struct ScenarioInput {
    probability: f64,
    assumption: String,
    flows: Vec<ResourceFlow>,
    charges: Vec<ResourceCharge>,
}

type FlowKey = (String, String, String, String);
type ResourceEntries = Vec<(ResourceKind, u64)>;

fn parse_events(source: &str) -> Result<ResourceModel, CliError> {
    let mut scenarios: BTreeMap<String, ScenarioInput> = BTreeMap::new();
    let mut flow_entries: BTreeMap<FlowKey, ResourceEntries> = BTreeMap::new();
    let mut charge_entries: BTreeMap<(String, String), ResourceEntries> = BTreeMap::new();

    for (line_index, line) in source.lines().enumerate() {
        let line_number = line_index + 1;
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        match fields.first().copied() {
            Some("scenario") if fields.len() == 4 => {
                let name = fields[1].to_owned();
                if scenarios.contains_key(&name) {
                    return Err(line_error(line_number, "scenario is repeated"));
                }
                scenarios.insert(
                    name,
                    ScenarioInput {
                        probability: parse_f64(fields[2], line_number, "probability")?,
                        assumption: fields[3].to_owned(),
                        ..ScenarioInput::default()
                    },
                );
            }
            Some("flow") if fields.len() == 7 => {
                let kind = parse_kind(fields[5], line_number)?;
                let quantity = parse_u64(fields[6], line_number, "quantity")?;
                flow_entries
                    .entry((
                        fields[1].to_owned(),
                        fields[2].to_owned(),
                        fields[3].to_owned(),
                        fields[4].to_owned(),
                    ))
                    .or_default()
                    .push((kind, quantity));
            }
            Some("charge") if fields.len() == 5 => {
                let kind = parse_kind(fields[3], line_number)?;
                let quantity = parse_u64(fields[4], line_number, "quantity")?;
                charge_entries
                    .entry((fields[1].to_owned(), fields[2].to_owned()))
                    .or_default()
                    .push((kind, quantity));
            }
            Some(kind) => return Err(line_error(line_number, &format!("invalid record `{kind}`"))),
            None => return Err(line_error(line_number, "record is empty")),
        }
    }

    for ((scenario, relation, source, destination), entries) in flow_entries {
        let input = scenarios
            .get_mut(&scenario)
            .ok_or_else(|| CliError(format!("flow references unknown scenario `{scenario}`")))?;
        let resources = ResourceBundle::new(entries).map_err(error)?;
        input.flows.push(
            ResourceFlow::new(
                parse_address(&relation)?,
                parse_address(&source)?,
                parse_address(&destination)?,
                resources,
            )
            .map_err(error)?,
        );
    }
    for ((scenario, target), entries) in charge_entries {
        let input = scenarios
            .get_mut(&scenario)
            .ok_or_else(|| CliError(format!("charge references unknown scenario `{scenario}`")))?;
        let resources = ResourceBundle::new(entries).map_err(error)?;
        input
            .charges
            .push(ResourceCharge::new(parse_address(&target)?, resources).map_err(error)?);
    }

    let scenarios = scenarios
        .into_iter()
        .map(|(name, input)| {
            ResourceScenario::new(
                name,
                input.probability,
                input.assumption,
                input.flows,
                input.charges,
            )
            .map_err(error)
        })
        .collect::<Result<Vec<_>, _>>()?;
    ResourceModel::new(scenarios).map_err(error)
}

fn parse_address(value: &str) -> Result<Address, CliError> {
    Address::parse(value).map_err(error)
}

fn parse_kind(value: &str, line: usize) -> Result<ResourceKind, CliError> {
    ResourceKind::parse(value)
        .ok_or_else(|| line_error(line, &format!("unknown resource `{value}`")))
}

fn parse_f64(value: &str, line: usize, name: &str) -> Result<f64, CliError> {
    value
        .parse()
        .map_err(|_| line_error(line, &format!("{name} is not a number")))
}

fn parse_u64(value: &str, line: usize, name: &str) -> Result<u64, CliError> {
    value
        .parse()
        .map_err(|_| line_error(line, &format!("{name} is not a nonnegative integer")))
}

fn line_error(line: usize, message: &str) -> CliError {
    CliError(format!("events line {line}: {message}"))
}

fn print_resource_report(report: &ResourceReport) {
    println!("resource-report");
    for (name, probability, assumption) in report.scenarios() {
        println!("scenario\t{name}\tprobability={probability}\tassumption={assumption}");
    }
    for (relation, estimate) in report.flow_estimates() {
        for (kind, quantity) in estimate.quantities() {
            println!("flow\t{relation}\t{kind}\t{quantity}");
        }
    }
    for (target, estimate) in report.charge_estimates() {
        for (kind, quantity) in estimate.quantities() {
            println!("charge\t{target}\t{kind}\t{quantity}");
        }
    }
    for kind in ResourceKind::ALL {
        if let Some(quantity) = report.total_flow(kind) {
            println!("flow-total\t{kind}\t{quantity}");
        }
        if let Some(quantity) = report.total_charge(kind) {
            println!("charge-total\t{kind}\t{quantity}");
        }
    }
}

fn error(error: impl fmt::Display) -> CliError {
    CliError(error.to_string())
}
