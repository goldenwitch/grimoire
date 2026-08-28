# Shannon information flow and channel semantics

Status: user-directed prototype contract; finite implementation checkpoint with open claim gaps.

This proposal realigns the information-flow work around a stronger goal:
follow Shannon information from an addressed input variable to one or more
explicit terminal variables through a composed channel model.

The core specification remains the authority for addresses, elements, layers,
and projections. This proposal does not add a new core element kind. It defines
a channel interpretation over selected addressed structure and records the
uncertainty of claims that cannot be computed exactly.

## Evidence

The local Scry corpus contains the following indexed sources for this contract:

- [The Information Bottleneck Method](https://arxiv.org/abs/physics/0004057)
  frames relevant representation as compression of one variable for prediction
  of another.
- [Opening the Black Box of Deep Neural Networks via Information](https://arxiv.org/abs/1703.00810)
  uses layerwise mutual-information quantities and the data-processing view of
  representations, while exposing estimator and quantization choices.
- [Mutual Information Neural Estimation](https://arxiv.org/abs/1801.04062)
  provides a scalable neural estimator for high-dimensional continuous variables
  and makes estimator tightness and consistency part of the evidence.
- [Contrastive Multiview Coding](https://arxiv.org/abs/1906.05849) connects
  shared-view representation learning to mutual information while warning that
  information maximization alone is not a complete account of representation
  quality.
- [A Bayesian Framework for Information-Theoretic Probing](https://arxiv.org/abs/2109.03853)
  distinguishes finite-data Bayesian information from classical mutual
  information computed as if the true distribution were known.

The sources support a confidence-aware contract, but they do not make one
estimator or one notion of relevance universally correct.

## Goal

Given a source input port `X`, a selected structural reprojection, and explicit
terminal ports `Y`, the information-flow account should be able to state:

- which channel interpretation connects `X` to `Y`;
- how much Shannon information about `X` is present at `Y`;
- what fraction of source information is retained at `Y`, when that fraction is
  defined; and
- how certain the estimate is when the channel or distribution is empirical.

The graph alone answers only a structural question: which addressed routes are
available. It does not determine a probability distribution or a Shannon
quantity.

## Vocabulary

- **source variable**: the random variable carried by one addressed input port.
- **terminal variable**: the random variable carried by one addressed terminal
  port.
- **channel**: a conditional distribution from a block's input variables to its
  output variables.
- **source distribution**: the distribution supplied for the source variable
  and any other exogenous inputs required by the selected graph.
- **information retention**: the normalized quantity
  `I(X;Y) / H(X)`, defined only when `H(X) > 0`.
- **route allocation claim**: an attribution statement assigning approximate
  shares to a declared partition of routes or terminal outcomes.
- **confidence**: the statistical or epistemic qualification attached to an
  estimated claim. It is not probability mass in the channel and is not a
  replacement for an uncertainty interval.

The source and terminal are addressed ports, not inferred from block names.
The first contract names terminal addresses explicitly. A graph-leaf query may
be a later convenience, but a leaf is not automatically a semantic output.

## Shannon quantities

For a discrete source and terminal, mutual information is:

`I(X;Y) = sum p(x,y) log_2(p(x,y) / (p(x)p(y)))`

Source entropy is:

`H(X) = -sum p(x) log_2 p(x)`

When `H(X) > 0`, the percentage of source entropy retained at a terminal is:

`retention_pct(X,Y) = 100 * I(X;Y) / H(X)`

This is a terminal-retention statement. It is not automatically a statement
that a percentage of information "went down" one edge or route.

A source may branch to several terminals. The joint terminal quantity is:

`I(X;Y_1,...,Y_n)`

It is not the sum of the marginal quantities. Marginal sums can double-count
redundant information and miss information available only through a joint
observation. When a terminal has side information `Z`, the relevant quantity
may instead be `I(X;Y | Z)`.

## Approximate percentages and confidence

The phrase "about X percent goes this way" is meaningful only after its
reference quantity and decomposition are named.

A terminal-retention claim may report an estimate and interval:

- estimate: `r_hat`;
- uncertainty interval: `[r_low, r_high]`;
- confidence or credibility level: `gamma`; and
- estimator, sample source, and protocol.

A route allocation claim additionally names an attribution method and a route
partition. The shares are constrained to sum to one only when the method
explicitly defines a partition of the information budget. Mutual information
does not supply a unique route decomposition for arbitrary branching networks.

The first contract therefore treats route shares as claims, not as an implicit
property of every channel. A claim without a declared partition or method is
incomplete. A claim that cannot be computed receives an explicit unknown or
interval, never an invented exact bit count.

Confidence must be reported with its meaning. A frequentist confidence level,
a Bayesian credible level, a bootstrap interval, and an expert assessment are
not interchangeable. The claim records the method and level rather than
collapsing them into one unexplained scalar.

For downstream decisions, the prototype defaults to a Bayesian posterior over
the uncertain source distribution and channel parameters when finite data are
used. It reports a posterior estimate, a credible interval, and decision
probabilities such as:

`P(retention_pct(X,Y) >= threshold | data)`

This is a probability over the claim under the stated model and prior. It is
not the channel probability and it is not classical frequentist confidence.

The name Bayesian mutual information is reserved for an agent-relative
information quantity whose estimand differs from classical `I(X;Y)`. A
posterior credible interval around classical mutual information must be labeled
as uncertainty about classical mutual information, not silently renamed.

Exact finite channel calculations have exact results under their supplied
model and do not need an empirical confidence interval. An empirical estimate
of a real neural channel does.

## Channel semantics

For a finite discrete block, a channel is a stochastic kernel:

`K(y | x) >= 0`

and:

`sum_y K(y | x) = 1`

A block with multiple inputs and outputs has a joint kernel:

`K(y_1,...,y_m | x_1,...,x_n)`

A deterministic block is represented by a kernel concentrated on its output
function. Noise, quantization, dropout, or a stochastic decoder have ordinary
non-deterministic kernels.

For a chain `X -> Z -> Y`, composition is:

`K_XY(y | x) = sum_z K_ZY(y | z) K_XZ(z | x)`

For an acyclic graph, block kernels compose with the wiring induced by the
addressed connections. Connections carry or identify variables; blocks supply
conditional channel behavior. Multiple inputs remain joint inputs. No
independence assumption is introduced merely because two ports have different
addresses.

A channel interpretation may be exact, supplied by a finite reference kernel,
or empirical, supplied by an external artifact or estimator. The interpretation
must identify which one it is.

## Branches, merges, and data processing

A branch preserves one source variable as a joint input to its downstream
terminals. It does not make the information values on the branches additive.

A merge retains the joint input variables of the receiving block. If one source
is being measured in the presence of another, the query must state whether it
uses marginal or conditional mutual information.

For a valid Markov chain `X -> Z -> Y`, the data-processing inequality applies:

`I(X;Y) <= I(X;Z)`

The inequality does not apply merely because three addresses appear in a path.
The channel factorization and conditioning assumptions must support it.

## Cycles and continuous variables

The core graph may contain cycles, but a bare cyclic graph does not define a
unique static joint distribution. Channel evaluation therefore requires one of:

- a finite time-unrolled acyclic graph;
- an explicit fixed-point or equilibrium contract; or
- an external execution artifact that supplies the joint process.

The first executable prototype accepts finite acyclic channel graphs and finite
horizon unrollings. It reports a bare cyclic channel query as unresolved rather
than silently choosing a fixed point.

Continuous neural representations require density or estimator assumptions.
Differential entropy is not interchangeable with discrete Shannon entropy and
can be coordinate-dependent or undefined. Continuous channel observations are
a later empirical boundary; the finite discrete calculus is the reference
semantics, not a claim that real tensors are discrete.

## Causality and gradient flow

Mutual information measures statistical dependence. It does not establish that
intervening on `X` changes `Y`.

Gradient flow and Shannon information are also different relations. The existing
forward/backward fixture may use `invert` to express a backward structural view,
but `invert` does not reverse a Shannon channel and does not define an
information quantity.

A future causal-intervention layer would need its own treatment. It is outside
this channel contract.

## Mapping to Grimoire

- The core graph remains addressed structural topology.
- Source and terminal variables are addressed ports.
- A channel interpretation is layered over selected structure; it is not a new
  block, port, or connection kind.
- The projection language selects and composes structure. It does not infer
  source distributions, channel kernels, or confidence intervals.
- Real model kernels should normally be external artifacts or empirical
  estimators. Small finite kernels can serve as executable reference fixtures.
- `measurement/1` may record auxiliary sourced numbers, but it is not sufficient
  to represent a source-terminal information relation or its uncertainty.
- A future information-claim schema must name source, terminal or terminal set,
  quantity, unit, estimate, uncertainty, method, and evidence context.

The exact claim serialization and registry boundary remain a design task. No
new open-ended schema constructor is introduced to solve it.

## Executable prototype regime

The current reference implementation provides:

- finite discrete distributions with validated nonnegative probabilities;
- finite stochastic channels and deterministic channels;
- Bayesian posterior uncertainty for finite categorical source and channel
  parameters, with credible intervals and threshold probabilities;
- channel composition over acyclic chains;
- mutual information and source entropy in bits;
- normalized terminal retention when the source entropy is nonzero;
- joint terminal queries for branches;
- finite-horizon unrolled cycles; and
- visible rejection of bare cyclic queries.

The mathematical channel core remains independent from the text grammar. The
graph adapter maps addressed ports and connections to the finite channel model
after the laws are tested.

## Conformance laws

The current fixture family includes:

- an identity channel with retention `1`;
- a constant channel with mutual information `0`;
- a noisy channel whose mutual information is below source entropy;
- composition of two channels with the same result as their matrix product;
- associativity of channel composition;
- a data-processing inequality fixture;
- a branch where joint information differs from the sum of marginal values;
- a merge requiring a conditional query;
- an exact result versus an empirical interval claim;
- a Bayesian posterior claim whose decision probability changes with evidence;
- a finite-horizon recurrent unrolling; and
- a bare cycle reported as unresolved.

Claims should also validate their own epistemic shape: intervals are ordered,
estimates lie within their intervals, percentages have an explicit denominator,
and route shares name the partition and attribution method that makes their
sum meaningful.

## Decisions and gaps

The current prototype decisions are:

- Shannon information is queried from an addressed source variable to explicit
  terminal variables.
- Mutual information is the default transfer quantity; source entropy is
  context, and normalized retention is a derived percentage when defined.
- Route percentages are attribution claims with an explicit partition and
  method; they are not inferred by summing edge mutual informations.
- Exact finite kernels and empirical channel estimates are separate claim
  regimes.
- For finite empirical channels, Bayesian posterior uncertainty is the default
  decision-support representation; classical mutual information and
  agent-relative Bayesian mutual information remain distinct estimands.
- The first executable regime is finite, discrete, acyclic, and static, with
  finite-horizon unrolling for recurrent examples.
- Confidence qualifies estimates and records their method; it does not alter
  channel probabilities.

Remaining gaps are continuous estimators, fixed-point channel semantics,
standard route-attribution methods, prior sensitivity, and the
serialization/registry boundary for source-terminal information claims. Each
binds only when a fixture needs it.
