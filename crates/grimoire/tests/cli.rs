use std::fs;
use std::path::PathBuf;
use std::process::Command;

use grimoire::{
    Address, ResourceBundle, ResourceCharge, ResourceFlow, ResourceKind, ResourceModel,
    ResourceReport, ResourceScenario, evaluate_layer, parse_description, prototype_schemas,
    validate_description,
};

fn example() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/reference.grimoire")
}

fn scry_example() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/scry.grimoire")
}

fn scry_events() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/scry-resources.tsv")
}

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_grimoire"))
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("failed to start CLI: {error}"))
}

#[test]
fn cli_validates_canonicalizes_evaluates_and_cuts_the_public_example() {
    let path = example();
    let path = path
        .to_str()
        .unwrap_or_else(|| panic!("example path is not UTF-8"));

    let validation = run(&["validate", path]);
    assert!(
        validation.status.success(),
        "{}",
        String::from_utf8_lossy(&validation.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&validation.stdout).trim(), "valid");

    let canonical = run(&["canonicalize", path]);
    assert!(
        canonical.status.success(),
        "{}",
        String::from_utf8_lossy(&canonical.stderr)
    );
    let canonical_text =
        String::from_utf8(canonical.stdout).unwrap_or_else(|error| panic!("{error}"));
    let description = parse_description(&canonical_text).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(description.layers.len(), 3);

    let evaluation = run(&["evaluate", path, "architecture"]);
    assert!(
        evaluation.status.success(),
        "{}",
        String::from_utf8_lossy(&evaluation.stderr)
    );
    let evaluation_text = String::from_utf8_lossy(&evaluation.stdout);
    assert!(evaluation_text.contains("layer=architecture"));
    assert!(evaluation_text.contains("elements=6"));

    let cut = run(&["cut", path, "deployment"]);
    assert!(
        cut.status.success(),
        "{}",
        String::from_utf8_lossy(&cut.stderr)
    );
    parse_description(&String::from_utf8(cut.stdout).unwrap_or_else(|error| panic!("{error}")))
        .unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn cli_reports_typed_resource_scenarios_from_explicit_events() {
    let description = example();
    let description = description
        .to_str()
        .unwrap_or_else(|| panic!("example path is not UTF-8"));
    let events_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/reference-resources.tsv");
    let events_path = events_path
        .to_str()
        .unwrap_or_else(|| panic!("events path is not UTF-8"));

    let output = run(&["resources", description, "cost", events_path]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("scenario\tindexed-hit\tprobability=0.75"));
    assert!(text.contains("flow-total\tbytes\t3584"));
    assert!(text.contains("charge-total\tflop-work\t1250000"));
    assert!(text.contains("charge-total\tmemory-bytes\t6144"));
    assert!(!text.contains("charge-total\tbytes\t"));
}

#[test]
fn cli_reports_missing_layers_as_visible_failures() {
    let description = example();
    let description = description
        .to_str()
        .unwrap_or_else(|| panic!("example path is not UTF-8"));
    let output = run(&["evaluate", description, "missing"]);
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("layer does not exist"));
}

#[test]
fn cli_runs_the_scry_workflow_and_matches_direct_resource_evaluation() {
    let description_path = scry_example();
    let description_path = description_path
        .to_str()
        .unwrap_or_else(|| panic!("Scry example path is not UTF-8"));
    let events_path = scry_events();
    let events_path = events_path
        .to_str()
        .unwrap_or_else(|| panic!("Scry events path is not UTF-8"));

    let validation = run(&["validate", description_path]);
    assert!(
        validation.status.success(),
        "{}",
        String::from_utf8_lossy(&validation.stderr)
    );

    let canonical = run(&["canonicalize", description_path]);
    assert!(
        canonical.status.success(),
        "{}",
        String::from_utf8_lossy(&canonical.stderr)
    );
    parse_description(
        &String::from_utf8(canonical.stdout).unwrap_or_else(|error| panic!("{error}")),
    )
    .unwrap_or_else(|error| panic!("{error}"));

    let evaluation = run(&["evaluate", description_path, "architecture"]);
    assert!(
        evaluation.status.success(),
        "{}",
        String::from_utf8_lossy(&evaluation.stderr)
    );
    assert!(String::from_utf8_lossy(&evaluation.stdout).contains("elements=31"));

    let cut = run(&["cut", description_path, "deployment"]);
    assert!(
        cut.status.success(),
        "{}",
        String::from_utf8_lossy(&cut.stderr)
    );
    parse_description(&String::from_utf8(cut.stdout).unwrap_or_else(|error| panic!("{error}")))
        .unwrap_or_else(|error| panic!("{error}"));

    let output = run(&["resources", description_path, "cost", events_path]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cli_text = String::from_utf8_lossy(&output.stdout);
    let direct = direct_scry_report();
    for kind in ResourceKind::ALL {
        if let Some(quantity) = direct.total_flow(kind) {
            assert!(cli_text.contains(&format!("flow-total\t{kind}\t{quantity}")));
        }
        if let Some(quantity) = direct.total_charge(kind) {
            assert!(cli_text.contains(&format!("charge-total\t{kind}\t{quantity}")));
        }
    }
    assert!(cli_text.contains("scenario\tcold-ingest\tprobability=0.2"));
    assert!(cli_text.contains("scenario\tindexed-search\tprobability=0.45"));
}

fn direct_scry_report() -> ResourceReport {
    let source = fs::read_to_string(scry_example()).unwrap_or_else(|error| panic!("{error}"));
    let description = parse_description(&source).unwrap_or_else(|error| panic!("{error}"));
    let schemas = prototype_schemas().unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas).unwrap_or_else(|errors| panic!("{errors:?}"));
    let reprojection = evaluate_layer(&description, "cost")
        .unwrap_or_else(|error| panic!("{error}"))
        .structural;
    ResourceModel::new(vec![
        scenario(
            "cold-ingest",
            0.20,
            vec![
                flow(
                    "model-cache-to-passage-embed",
                    "model-cache/output",
                    "passage-embed/model",
                    ResourceKind::Bytes,
                    133806060,
                ),
                flow(
                    "origin-to-fetch",
                    "origin/output",
                    "fetch/input",
                    ResourceKind::Bytes,
                    8192,
                ),
                flow(
                    "fetch-to-slice",
                    "fetch/output",
                    "slice/input",
                    ResourceKind::Bytes,
                    8192,
                ),
                flow(
                    "slice-to-passage-embed",
                    "slice/output",
                    "passage-embed/input",
                    ResourceKind::Bytes,
                    8192,
                ),
                flow(
                    "passage-embed-to-corpus",
                    "passage-embed/output",
                    "corpus/input",
                    ResourceKind::Bytes,
                    1536,
                ),
            ],
            vec![
                charge("passage-embed", ResourceKind::FlopWork, 1000000),
                charge("passage-embed", ResourceKind::MemoryBytes, 16384),
                charge(
                    "passage-embed",
                    ResourceKind::BandwidthBytesPerSecond,
                    1000000000,
                ),
                charge("fetch", ResourceKind::LatencyNanoseconds, 5000000),
            ],
        ),
        scenario(
            "warm-ingest",
            0.20,
            vec![
                flow(
                    "origin-to-fetch",
                    "origin/output",
                    "fetch/input",
                    ResourceKind::Bytes,
                    8192,
                ),
                flow(
                    "fetch-to-slice",
                    "fetch/output",
                    "slice/input",
                    ResourceKind::Bytes,
                    8192,
                ),
                flow(
                    "slice-to-passage-embed",
                    "slice/output",
                    "passage-embed/input",
                    ResourceKind::Bytes,
                    8192,
                ),
                flow(
                    "passage-embed-to-corpus",
                    "passage-embed/output",
                    "corpus/input",
                    ResourceKind::Bytes,
                    1536,
                ),
            ],
            vec![
                charge("passage-embed", ResourceKind::FlopWork, 1000000),
                charge("passage-embed", ResourceKind::MemoryBytes, 16384),
                charge(
                    "passage-embed",
                    ResourceKind::BandwidthBytesPerSecond,
                    1000000000,
                ),
                charge("fetch", ResourceKind::LatencyNanoseconds, 5000000),
            ],
        ),
        scenario(
            "indexed-search",
            0.45,
            vec![
                flow(
                    "query-to-query-embed",
                    "query/output",
                    "query-embed/input",
                    ResourceKind::Bytes,
                    512,
                ),
                flow(
                    "model-cache-to-query-embed",
                    "model-cache/output",
                    "query-embed/model",
                    ResourceKind::Bytes,
                    133806060,
                ),
                flow(
                    "query-embed-to-search",
                    "query-embed/output",
                    "search/query",
                    ResourceKind::Bytes,
                    1536,
                ),
                flow(
                    "corpus-to-search",
                    "corpus/output",
                    "search/corpus",
                    ResourceKind::Bytes,
                    1048576,
                ),
                flow(
                    "search-to-handle",
                    "search/output",
                    "handle/input",
                    ResourceKind::Bytes,
                    1536,
                ),
            ],
            vec![
                charge("query-embed", ResourceKind::FlopWork, 500000),
                charge("search", ResourceKind::MemoryBytes, 1048576),
                charge("search", ResourceKind::BandwidthBytesPerSecond, 1000000000),
                charge("search", ResourceKind::LatencyNanoseconds, 10000000),
            ],
        ),
        scenario(
            "context-lookup",
            0.15,
            vec![
                flow(
                    "handle-to-neighbours",
                    "handle/output",
                    "neighbours/input",
                    ResourceKind::Bytes,
                    3072,
                ),
                flow(
                    "handle-to-provenance",
                    "handle/output",
                    "provenance/input",
                    ResourceKind::Bytes,
                    256,
                ),
                flow(
                    "neighbours-to-mcp",
                    "neighbours/output",
                    "mcp/response",
                    ResourceKind::Bytes,
                    4096,
                ),
                flow(
                    "provenance-to-mcp",
                    "provenance/output",
                    "mcp/response",
                    ResourceKind::Bytes,
                    256,
                ),
            ],
            vec![
                charge("neighbours", ResourceKind::MemoryBytes, 4096),
                charge("provenance", ResourceKind::MemoryBytes, 256),
                charge(
                    "neighbours",
                    ResourceKind::BandwidthBytesPerSecond,
                    100000000,
                ),
                charge("provenance", ResourceKind::LatencyNanoseconds, 1000000),
            ],
        ),
    ])
    .unwrap_or_else(|error| panic!("{error}"))
    .evaluate(&reprojection)
    .unwrap_or_else(|error| panic!("{error}"))
}

fn scenario(
    name: &str,
    probability: f64,
    flows: Vec<ResourceFlow>,
    charges: Vec<ResourceCharge>,
) -> ResourceScenario {
    ResourceScenario::new(
        name,
        probability,
        "fixture source and protocol",
        flows,
        charges,
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

fn flow(
    relation: &str,
    source: &str,
    destination: &str,
    kind: ResourceKind,
    quantity: u64,
) -> ResourceFlow {
    ResourceFlow::new(
        address(relation),
        address(source),
        address(destination),
        ResourceBundle::new(vec![(kind, quantity)]).unwrap_or_else(|error| panic!("{error}")),
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

fn charge(target: &str, kind: ResourceKind, quantity: u64) -> ResourceCharge {
    ResourceCharge::new(
        address(target),
        ResourceBundle::new(vec![(kind, quantity)]).unwrap_or_else(|error| panic!("{error}")),
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

fn address(value: &str) -> Address {
    Address::parse(&format!("@scry/{value}"))
        .unwrap_or_else(|error| panic!("invalid fixture address {value}: {error}"))
}
