# 1. Scaling beyond eight threads on the i9

- **Doing:** testing a different team count before changing production policy.
- **In order to:** 0, faster correct production decoding. Recoverable headroom and its mechanism remain questions.
- **Status:** waiting on confirmation results from Louis; run 1 passed both arrangements. Alternative-performance dispatch was last paused; no new dispatch is recorded.
- **Known:** the [team-size screen](https://github.com/lmmx/mibz2/blob/3421fe600f544724425db665caf06fa89dd25f06/docs/journal/2026-11-06-team-size.md) completed 320 runs with matching output, a complete grid and an unflagged control. Its registered primaries returned 1.017 (95% interval 1.008–1.026) and 1.003 (0.993–1.014), below the 1.03 minimum. The run was done as intended; neither primary qualified. Both point estimates fell within their broad predicted ranges. No confirmation was due for either failed primary.
- **Exploration, not qualification:** three teams at 10 and 12 requested threads returned 1.075 and 1.097, with about 18% less peak memory. These cells were selected after seeing the data. The [screen and corrected interpretation](https://github.com/lmmx/mibz2/blob/838d765c7de4956ada5a29c61ee30a436b1cf605/docs/journal/2026-11-06-team-size.md) separates table-count association from causation: concurrency and window change too; the last-level cache explanation is a hypothesis. Throughput divided by tables is not measured block service rate.

## 1.1 Three-team confirmation (#220)

- **Doing:** testing three teams of four at 12 threads and three teams of three at 10 requested threads (nine decoding threads), against four teams of two at eight.
- **In order to:** 1, identify a policy candidate; production adoption would need its own continuous-integration (CI), parity, safety and regression evidence.
- **Status:** run 1 done as intended, as recorded by the Coordinator; both primaries pass. The conditional counter collection was unavailable, not a failure of the registered run. Confirmation execution is unconfirmed, and its results are outstanding from Louis. No qualification or production change yet ([handoff](https://github.com/lmmx/mibz2/pull/145#issuecomment-6693085860); [journal](https://github.com/lmmx/mibz2/blob/a3880ce89c6e40e6efa19a3f6ac352f8cc7d6f83/docs/journal/2026-11-06-three-tables.md#result)).
- **Result:** three groups of four: 1.090× throughput, 95% interval 1.072–1.107, peak memory 0.82×, processor time 1.04× reference. Three groups of three: 1.076×, interval 1.058–1.095, memory 0.82×, processor time 0.90×. The journal records all byte/grid/plan/reference/control predicates passing.
- **Prediction separate from verdict:** 1.04–1.09 and 1.03–1.07 were predicted. The first is at the upper end, the second just above its range. The expected regression from the exploratory screen did not clearly appear; one run does not explain why. Both memory predictions held.
- **Rule unchanged:** [registration](https://github.com/lmmx/mibz2/blob/a3880ce89c6e40e6efa19a3f6ac352f8cc7d6f83/docs/journal/2026-11-06-three-tables.md#the-rule). Mean ≥1.03, lower bound >1.00, all predicates, peak memory ≤1.15× and processor time ≤1.5× reference; the same cell must pass one predeclared second run. No pooling, switching or third run; one invalidation replacement budget across the experiment. Qualification would only produce a policy candidate, still requiring separate production checks.
- **Pause versus dependency:** the prior deliberate hold for Oracle verification ended with the verified first account and Louis's run-1 handoff. The second result is a genuine qualification dependency, not a new Oracle gate.
- **Evidence scope:** Louis identified bundle 20261106T105317Z as run 1; the journal records code at 639c3df1, unchanged from freeze 656d2794, and report reproduction from raw runs. This account reads the recorded evidence; it does not independently recalculate the benchmark or adjudicate its gate.
- **Diagnostics:** packing versus spreading workers reduced throughput by 12–21% across the plans. This is a whole-placement effect, not an isolated measurement of sibling sharing. An interval including zero does not prove tracing had no effect; worker activity does not establish every helper's productive occupancy. Table count, group size and window remain confounded.


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

Run 1 also saved about 18% peak memory for both three-team cells while pursuing throughput. This matches the prediction and the earlier screen, but remains subject to confirmation and is not an independent production qualification ([frozen registration](https://github.com/lmmx/mibz2/blob/838d765c7de4956ada5a29c61ee30a436b1cf605/docs/journal/2026-11-06-three-tables.md)).
