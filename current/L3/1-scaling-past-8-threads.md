# 1. Scaling beyond eight threads on the i9

- **Doing:** testing a different team count before changing production policy.
- **In order to:** 0, faster correct production decoding. Recoverable headroom and its mechanism remain questions.
- **Status:** waiting on Louis's results; new performance dispatch paused.
- **Known:** the [team-size screen](https://github.com/lmmx/mibz2/blob/3421fe600f544724425db665caf06fa89dd25f06/docs/journal/2026-11-06-team-size.md) completed 320 runs with matching output, a complete grid and an unflagged control. Its registered primaries returned 1.017 (95% interval 1.008–1.026) and 1.003 (0.993–1.014), below the 1.03 minimum. The run was done as intended; neither primary qualified. Both point estimates fell within their broad predicted ranges. No confirmation was due for either failed primary.
- **Exploration, not qualification:** three teams at 10 and 12 requested threads returned 1.075 and 1.097, with about 18% less peak memory. These cells were selected after seeing the data. The [screen and corrected interpretation](https://github.com/lmmx/mibz2/blob/838d765c7de4956ada5a29c61ee30a436b1cf605/docs/journal/2026-11-06-team-size.md) separates table-count association from causation: concurrency and window change too; the last-level cache explanation is a hypothesis. Throughput divided by tables is not measured block service rate.

## 1.1 Three-team confirmation (#220)

- **Doing:** testing three teams of four at 12 threads and three teams of three at 10 requested threads (nine decoding threads), against four teams of two at eight.
- **In order to:** 1, identify a policy candidate; production adoption would need its own continuous-integration (CI), parity, safety and regression evidence.
- **Status:** waiting on Louis's held results ([Louis's recorded decision](https://github.com/lmmx/mibz2/blob/3421fe600f544724425db665caf06fa89dd25f06/docs/journal/decisions.md)). The repository does not establish execution status or a verdict for the fresh run. Do not call it unstarted or passed.
- **Prediction:** 1.04–1.09 and 1.03–1.07. Outcome and qualification: unreported.
- **Rule:** [frozen registration](https://github.com/lmmx/mibz2/blob/838d765c7de4956ada5a29c61ee30a436b1cf605/docs/journal/2026-11-06-three-tables.md). All byte/grid/plan/reference/control predicates; mean ≥1.03 and lower interval bound >1.00; peak resident memory ≤1.15× and processor-seconds ≤1.5× reference; the same cell must pass one predeclared confirmation. One replacement budget for the whole experiment. No pooling or third run.
- **Pause versus dependency:** Louis chose to hold the results pending Oracle configuration and verification. The experiment has no scientific dependency on this account. This rebuild does not release that hold.

## 1.2 Mechanism

- **Doing:** no newly dispatched experiment; purpose is to explain the slowdown so any intervention targets its cause.
- **In order to:** 1 directly, whatever 1.1 finds.
- **Status:** paused by the Coordinator ([6683240055](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683240055), qualified by [6683965024](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683965024)).
- **Known:** the Optimiser's active-set analysis identifies roughly 3.6 MB of randomly read T-vector per full block plus written segment prefixes, rather than treating all retained buffers as active cache use ([6683232182](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683232182)).
- **Premise retracted:** a 25% footprint reduction equalises four teams with three; it is not necessary for a benefit. The Coordinator withdrew it as a reopening threshold after the Researcher's counterexample. Only the dispatch pause survives. C15's measured failure remains; no i9 gain is implied ([6683965024](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683965024)).

## 1.3 Placement and pipelining

- **Doing:** no new dispatch.
- **In order to:** 1, use cores or overlap differently.
- **Status:** paused by the same Coordinator decision, not blocked by 1.1.
- **Premise:** placement has an observed packed/spread whole-process ratio of 0.778; physical-core count changes too, so it is not isolated sibling-sharing cost. Stage pipelining still needs a quantified recoverable-time premise. Neither is qualified for adoption ([screen and corrected interpretation](https://github.com/lmmx/mibz2/blob/838d765c7de4956ada5a29c61ee30a436b1cf605/docs/journal/2026-11-06-team-size.md)).

## Accepted, not aimed at

The exploratory three-team cells saved about 18% peak memory while pursuing throughput. That accepted side effect is not an independent production qualification, nor a confirmed result of #220 ([frozen registration](https://github.com/lmmx/mibz2/blob/838d765c7de4956ada5a29c61ee30a436b1cf605/docs/journal/2026-11-06-three-tables.md)).
