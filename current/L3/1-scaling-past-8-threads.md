# 1. Scaling beyond eight threads on the i9

- **Doing:** validating a release after the policy passed confirmation and merged.
- **In order to:** 0, faster correct production decoding. Recoverable headroom and its mechanism remain questions.
- **Status:** policy admitted and merged; release validation underway. Mechanism and other performance investigations have no new recorded dispatch.
- **Known:** the [team-size screen](https://github.com/lmmx/mibz2/blob/3421fe600f544724425db665caf06fa89dd25f06/docs/journal/2026-11-06-team-size.md) completed 320 runs with matching output, a complete grid and an unflagged control. Its registered primaries returned 1.017 (95% interval 1.008–1.026) and 1.003 (0.993–1.014), below the 1.03 minimum. The run was done as intended; neither primary qualified. Both point estimates fell within their broad predicted ranges. No confirmation was due for either failed primary.
- **Exploration, not qualification:** three teams at 10 and 12 requested threads returned 1.075 and 1.097, with about 18% less peak memory. These cells were selected after seeing the data. The [screen and corrected interpretation](https://github.com/lmmx/mibz2/blob/838d765c7de4956ada5a29c61ee30a436b1cf605/docs/journal/2026-11-06-team-size.md) separates table-count association from causation: concurrency and window change too; the last-level cache explanation is a hypothesis. Throughput divided by tables is not measured block service rate.

## 1.1 Three-team confirmation (#220)

- **Doing:** completed two registered runs comparing three groups of four at 12 threads and three groups of three at 10 requested threads (nine decoding threads) with four groups of two at eight.
- **In order to:** 1, identify production-policy candidates.
- **Status:** done as intended, as recorded in the [result journal](https://github.com/lmmx/mibz2/blob/544462881497fc957fe17c35ae7508ca1bac23d5/docs/journal/2026-11-06-three-tables.md#result); both arrangements qualify under the unchanged rule. This records the Coordinator's analysis, not a fresh Observer adjudication.

| Arrangement | Run 1 throughput ratio (95% interval) | Run 2 throughput ratio (95% interval) | Peak memory, runs 1 / 2 | Processor time, runs 1 / 2 |
|---|---|---|---|---|
| Three groups of four | 1.090 (1.072–1.107) | 1.086 (1.062–1.111) | 0.82 / 0.83 | 1.04 / 1.04 |
| Three groups of three | 1.076 (1.058–1.095) | 1.087 (1.074–1.100) | 0.82 / 0.82 | 0.90 / 0.89 |

- **Prediction separate from qualification:** groups of four were predicted at 1.04–1.09 and both point estimates lie in that range; groups of three were predicted at 1.03–1.07 and both exceeded its upper end. Memory and processor-time directions agree with predictions. No mechanism follows from that agreement.
- **Rule and outcome:** the same cell passed all predicates and resource limits twice; no switching, pooling or replacement. Both qualify, but the rule neither ranks them nor adopts either. Scope: i9-10900K and the level-9 Wikidata corpus. The nominal fixed-run error bound is 0.05 for this experiment under its model, not all future work ([rule and outcome](https://github.com/lmmx/mibz2/blob/544462881497fc957fe17c35ae7508ca1bac23d5/docs/journal/2026-11-06-three-tables.md)).
- **Provenance:** Louis handed over confirmation bundle 20261106T190811Z, collected at a3880ce8 with executable inputs unchanged from freeze 656d2794. The Coordinator records report reproduction and independent recomputation from fresh raw runs. Hardware counters were conditionally unavailable in both runs. This account does not independently recalculate the data.
- **Diagnostics:** placement effects remain whole-placement comparisons. The journal has corrected its tracing and sibling-sharing wording. Neither run establishes the table-count mechanism.

## 1.4 Production-policy comparison: completed

- **Doing:** completed the registered same-requested-count test and confirmation; policy merged.
- **In order to:** 1 and 0, turn candidate gains into validated production throughput.
- **Status:** both runs done as intended and VALID PASS, as recorded by the Coordinator; admitted, no replacement. [Result](https://github.com/lmmx/mibz2/pull/145#issuecomment-6698087098); [journal at open documentation PR's pin](https://github.com/lmmx/mibz2/blob/83cf36d304ebf02adbb452cd011a334210191881/docs/journal/2026-11-06-three-team-policy.md).
- **Policy:** ten physical cores, 20 MiB last-level cache and ≥10 requested threads select three groups of three decoding workers, window twelve. Other selections retain old rule. Both windows and grouping change as intended, not an isolated cache experiment.
- **Identity:** frozen candidate 3b780fb3d70efc8573bed1485254a5579e3a82c7, baseline 8b111867ff1caaca727a0d35e160786c499c4849. First run at 912baf00; confirmation head 422db167ea1b86b8ff45623c2c03984243807418, with matching frozen inputs and identical executables/corpus by the Coordinator's digest check.
- **Ten-thread findings:** run 1 speed 1.089 (1.075–1.103), CPU 0.88, memory 0.82; confirmation 1.088 (1.076–1.101), CPU 0.89, memory 0.81. Approximate predictions were speed 1.08, CPU 0.9 and memory 0.82; directions and scale reproduced.
- **Full rule satisfied:** both controls and every higher-count no-harm row passed in both runs under the unchanged rule. No pooling, replacement or switched primary. Confirmation no-harm upper wall ratios at 11/12/16/20 were 0.953/0.936/0.941/0.937, each below 1.02. This is recorded qualification, not Observer adjudication.
- **Control nuance:** first eight-thread 0.986 (0.977–0.995) passed the drift screen despite excluding 1; confirmation 1.005 (0.992–1.019) did not repeat that offset. Nine-thread confirmation 1.007 (0.997–1.017). No equivalence or identified build/layout cause follows.
- **Provenance:** run 1 three-team-policy/20261106T220546Z at evidence 820f4a87ee8f2ce43744350ef15462bed448c3bc; confirmation 20261106T225836Z at [0b340a53ce9b11cf1e4db9d1746dcaf34afb2b74](https://github.com/lmmx/mibz2/blob/0b340a53ce9b11cf1e4db9d1746dcaf34afb2b74/three-team-policy/20261106T225836Z.tar.gz). Confirmation SHA-256 7275bb95fe8328630fc856607544ea76ecc692c880518dcf1e5d9c49d6248c62; 280 rows; Coordinator reports digest match and identical frozen-judge verdict. Observer read the archived checksum and reports, not the binary bundle or a new calculation.
- **Integration:** [#234](https://github.com/lmmx/mibz2/pull/234) merged tested head 422db167 as 16646a9bb6de6c9af34689bc78788b1e79883864. Admission is complete; package release is not.
- **Limits retained:** same-topology other processors, unmeasured counts, other workloads, custom windows and contention remain unestablished. Census overhead unseparated; mechanism not isolated. Review fixture scope is not general build certification.
- **Source reconciliation:** master journal includes run 1 but still says confirmation not run; merged PR body still waits for two runs. [#243](https://github.com/lmmx/mibz2/pull/243), open at 83cf36d304ebf02adbb452cd011a334210191881, adds confirmation and admission. Use the newer result LOG plus verified merge for current status.

## 1.5 Release

- **Doing:** running release validation against py-0.2.4, in order to serve 0 by packaging 1.4's merged gain.
- **Status:** [dispatched paired Benchmark](https://github.com/lmmx/mibz2/actions/runs/42310037840) in progress on master 16646a9b; master Test [42309987808](https://github.com/lmmx/mibz2/actions/runs/42309987808) also in progress at inspection. No completed release claim.
- **Known:** Coordinator identifies #234 as the only src change since py-0.2.4 and names comparison base 1e644f9a ([plan](https://github.com/lmmx/mibz2/pull/145#issuecomment-6698087098)). Latest published release inspected: v0.2.4. No new version number invented.
- **Next:** completed automated evidence, then Coordinator's release under standing rules. No new workstation run requested and no implicit restart of paused investigations.

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

Both registered runs saved about 17–18% peak memory while pursuing throughput. That accepted side effect accompanies the qualified candidates; it is not independent approval for production adoption ([outcome](https://github.com/lmmx/mibz2/blob/544462881497fc957fe17c35ae7508ca1bac23d5/docs/journal/2026-11-06-three-tables.md#outcome)).
