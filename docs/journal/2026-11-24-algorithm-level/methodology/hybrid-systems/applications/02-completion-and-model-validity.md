# Finite completion and model validity

*Original decoder interpretation, 25 November 2026. Repository context: `9c38ad6c18237a1b923f9b762a1f634864d3b8a4`; stage-1 bundle: `character/run-20261125T001135Z`. This is our reasoning, not a précis of Goebel, Sanfelice and Teel. Chapters 4–6 have not been read in full here. Connections to graphical convergence, well-posedness and robustness are chapter leads awaiting that reading; no theorem from those chapters is invoked.*

## The useful goal is a completed finite decode

**Argued.** Specify four obligations separately: correct output and error behaviour; eventual completion; completion within a physical-time budget; bounded retained memory. Bounded queues do not establish eventual completion: a reader that stops consuming can leave a bounded, permanently stalled pipeline. Eventual completion does not establish a useful time bound either. Fair scheduling can permit arbitrarily long finite delays.

Define successful completion as the last required output byte delivered, with the prescribed end/error checks complete. Record resource shutdown separately if the benchmark includes it. Keep invalid-input termination and cancellation as distinct outcomes, preserving the decoder's error semantics. `taken == issued` is insufficient: the reader can still hold an unconsumed buffer, and the scanner can be processing its next block. The implementation increments `taken` when it removes an ordered result, before copying its bytes to the caller. [Source: `advance`, `read`, `scanner`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs).

## Physical time, discrete progress and completion

**Proved, by the definitions.** Hybrid time `(t, j)` contains physical time `t` and an event count `j`. Increasing `j` is not evidence that wall time advances. Several publish, take and admission events may share a timestamp. Conversely, one delayed input read can consume considerable wall time without completing an event.

**Argued.** A useful finite-job proof needs explicit assumptions: finite input; terminating per-block computations; eventual input availability and consumption; progress by runnable roles; and no infinite sequence of internal retries at one physical instant. Bounds on service, input, sink and scheduling delays strengthen termination into a time bound. A finite event budget must count semantic progress, not condition-variable wakeups: spurious wakeups need not decrease remaining work. If every counted event retires an obligation from a finite ledger, that ledger proves a bound on those events; it still needs a time bound between progress events.

**Proved for a restricted model.** With a fixed finite acyclic precedence/resource-order graph, nonnegative task durations, and tasks starting as soon as predecessors permit, completion time is the maximum path sum. It varies continuously with those durations. This elementary result does not cover changing worker assignment, admission decisions, deadlock, or service rates that depend on concurrent activity.

**Argued.** Closeness of a progress variable alone does not imply closeness of completion time. For example, `max(1-t, ε)` approaches `max(1-t, 0)` uniformly as `ε → 0`, while the former never reaches zero. This is an illustration about a completion observer, not a claimed hybrid-system counterexample. Any application of a trajectory-convergence theorem must separately connect its topology and assumptions to the decoder's terminal event.

## Preserve the program's event semantics

**Argued.** Start from ordered jobs, routes, ownership, scanner-current work, worker residual work, the next missing result, reader-current bytes and terminal state. Equal counts can conceal different missing-block identities and residual work; occupancy is an observable, not a sufficient replacement. Admission is

```text
issued - taken >= slots
  OR (ready_bytes >= byte_threshold AND issued - taken >= floor).
```

The scanner checks it before scanning; `issued` advances afterward. The byte threshold gates new scanning, not workers' completion, so it is not a hard bound on all retained output. The model must preserve these release events. [Source: `Shared::full`, `scanner`, `advance`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs).

Stage 1 reports valid causal linear order and conservation in all 840 grid decodes. Use that ledger order to reconstruct trajectories, resolving equal timestamps by causal sequence. The legacy `trace` marks in `parallel.rs` are a different source: `Enqueued` occurs after sending and `Ready` after unlocking. Timestamp sorting those marks can put receipt before its recorded enqueue or taking before its recorded readiness. [Stage-1 record](../../../leads/structure-and-function.md#the-dynamical-layer-25-november-after-externalchatgpt-dynamicsmd-and-stage-1s-data).

**Argued.** A set-valued abstract transition relation can represent uncertain scheduling, but must not silently introduce indefinite postponement of enabled completion. State what execution fairness or urgency is assumed. Likewise, represent different worker counts in separate models or a common maximum-worker state with inactive modes; changing the dimension is not an ordinary small continuous perturbation.

## A faithful replay does not identify service mechanisms

**Proved as an observational ambiguity.** For a fixed amount of work, an observed service interval constrains an integrated rate, `work = ∫ rate(t) dt`. Infinitely many rate histories have that integral. Endpoint timings alone cannot determine which history occurred, much less uniquely distinguish frequency, cache interference, SMT sharing or memory latency. If work itself is unknown, the ambiguity is larger.

**Argued from stage 1.** Scanner busy time rises from 0.1850 s at eight workers to 0.2275 s at twenty: 23.0%. Mean admitted blocks changes from 19.47 to 18.73 while steady worker starvation rises from 3.7% to 56.1%. Reusing one scanner demand across counts is unsupported. Similar mean occupancy does not itself establish whether instantaneous occupancy is predictive; that requires conditional trajectory comparisons. Nor do these aggregates identify a contention law. At ten workers the reader now has measured copy, sink and wait times of 0.0465, 0.0042 and 0.1517 s; absence of reader measurements is no longer a valid objection. The diagnostic itself is classified `PERTURBED` at ten and twenty workers. [Stage-1 report, sections “Where the time goes”, “The reader”, “The ledgers' cost”; provenance in the source notes](../source-notes.md).

**Conjectured.** A compact state augmented with a measured contention variable will predict held-out pulse responses. Replaying each block with its observed service duration tests event plumbing; it does not test this conjecture. Predict unseen durations and event times without feeding their realised values back into the predictor.

## Exact additions to the programme

Add the following acceptance contract to the finite-pipeline analysis in [the experiment programme](../../../briefs/experiment-programme.md#the-analysis-and-where-mathematics-comes-in):

| Requirement | Cheapest check |
|---|---|
| Separate output completion, error, cancellation and shutdown | Hand-check tiny valid, invalid and empty cases against API behaviour. |
| Preserve actual guards, ownership and causal order | Replay existing stage-1 ledgers; reproduce conservation and residence identities before fitting rates. |
| State which demands are fixed and which depend on state | Hold out whole rounds or interventions; report service-time and completion-time errors separately. |
| Distinguish termination from a wall-time bound | List input, sink, fairness and delay assumptions; use a small simulator case with a deliberate sink stall. |
| Give robustness an operational meaning | Perturb valid-input service budgets and event timing; check completion, peak retention and error ordering. |

Use already adopted block-range pulses and placement/frequency contrasts when stage 2 is registered; these notes authorize no additional workstation run. Any default-policy change still belongs to the programme's adoption registration. The immediate deliverable is a model whose claims have explicit domains and falsifiers, not an unqualified stability certificate.
