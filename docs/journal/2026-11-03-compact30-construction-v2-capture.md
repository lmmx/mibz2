# Compact30: constructor v2's construction-cost capture, INVALID, and the engineering STOP (reports 217–218)

**Formal result: INVALID.** G1 failed: production's warm W4 stability. **Engineering decision
(report 217): STOP further measurement investment in constructor v2.** No new experiment is
authorised.

The capture was the one authorised by report 216:
- run on the i9 on 2026-11-03 at 22:29 UTC;
- the frozen harness at `fa21f33`, protocol `c30-construction-v2`;
- 20 rounds, verified against the dry-run archive `65689a58…`.

INVALID is kept as it is. It is not relabelled FAIL, not rerun, and the gates aren't
weakened.

**What is preserved:**
- v2's independent correctness PASS (reports 209–211);
- the 69 targeted Miri cases (branch `research/compact30-ctor2-miri`, `e57a225`);
- the architectural result, with the frozen source at `8beffe2`;
- v1's separate STOP;
- compact30 traversal's VALID ADVANCE.

**The archive:** `c30-construction-v2-20261103T222928Z.tar.gz`, 5,334,709 bytes, sha256
`ebdb74549b39f83f07ae1f7d1c943916bac2ecb0be85868799aa30fe5e3d8af9`. It holds 69 files and its
manifest, including the dry run's evidence. It is kept as received in
[2026-11-03-compact30-construction-v2-capture/](2026-11-03-compact30-construction-v2-capture/).

**Checked here:**
- The checksum matches the console.
- All 69 manifest entries match their files, and the source has no tracked changes.
- The pinned analyser (`bench/c30_construction.py` at `fa21f33`) was replayed on the 75,087
  archived records, with the archive's RSS records and dry-run evidence, under `python3 -I`.
  It reproduces `c30-analysis.json` exactly, and its console matches the archive's analysis
  output (`replay.console`).
- The guardian independently recomputed the six ratios from the raw records (report 217).

## What the registered analysis says

- **Schedule:** S0–S10 and S1_protocol all pass.
- **Gates:** G3–G10 pass, including G5 (clock overhead) and all three G2 conditions.
- **G1 fails**, on production's warm W4 stability:

| G1 (production, warm W4) | max ÷ min | limit | rounds (min → max) |
|---|--:|--:|---|
| CPU ns per row | 1.0743 | 1.05 | 4 → 5 |
| completion | 1.2147 | 1.05 | 14 → 5 |

- **G2 passed by a wide margin.** No warm W4 step exceeded 2% in either arm. The largest step
  skew was 0.50% for P and 0.14% for C, and the row-weighted aggregates were 0.023% (P) and
  0.019% (C).
- **Decision:** `INVALID: a validity gate failed; no decision`.

**The ratios below are descriptive only; the capture is INVALID.**

| warm, C ÷ P | point | 95% CI | registered limit |
|---|--:|---|--:|
| b4 (CPU per row) | 1.0862 | [1.0773, 1.0952] | ≤ 1.06714 |
| e4 (completion) | 1.1144 | [1.0923, 1.1369] | ≤ 1.02 |
| b1 | 1.2877 | [1.2492, 1.3274] | — |
| full blocks only: b4 / e4 | 1.0836 / 1.1118 | [1.0747, 1.0925] / [1.0900, 1.1340] | — |
| short blocks only: b4 / e4 | 2.5186 / 2.0901 | [2.3546, 2.6940] / [1.9790, 2.2076] | — |
| cold b4 / e4 | 1.1057 / 1.1430 | [1.1007, 1.1107] / [1.1375, 1.1485] | — |

Both deciding lower bounds lie above their limits.

**Memory and faults:**
- Peak VmHWM was 1,772,572,672 B, 0.955 of the estimate.
- No major faults.
- Minor faults: warm P 0, warm C 100, cold P 1,658, cold C 317.

## Per round, warm W4 (post hoc diagnostic; nothing reclassified)

| round | P W4 CPU ns/row | C W4 CPU ns/row | P W4 completion ms | C W4 completion ms |
|--:|--:|--:|--:|--:|
| 0 | 0.5863 | 0.6455 | 31.62 | 35.86 |
| 1 | 0.5954 | 0.6434 | 32.16 | 35.76 |
| 2 | 0.5860 | 0.6376 | 31.78 | 35.47 |
| 3 | 0.5949 | 0.6420 | 32.05 | 35.75 |
| 4 | 0.5854 | 0.6410 | 31.75 | 35.62 |
| 5 | 0.6289 | 0.6408 | 38.32 | 35.73 |
| 6 | 0.5912 | 0.6462 | 31.81 | 35.90 |
| 7 | 0.5936 | 0.6500 | 31.89 | 36.18 |
| 8 | 0.5860 | 0.6407 | 31.60 | 35.66 |
| 9 | 0.5961 | 0.6443 | 32.04 | 35.82 |
| 10 | 0.5862 | 0.6459 | 31.55 | 35.93 |
| 11 | 0.6004 | 0.6546 | 32.33 | 36.45 |
| 12 | 0.5869 | 0.6504 | 31.60 | 36.14 |
| 13 | 0.5962 | 0.6506 | 32.12 | 36.18 |
| 14 | 0.5857 | 0.6444 | 31.55 | 35.90 |
| 15 | 0.5992 | 0.6424 | 32.24 | 35.80 |
| 16 | 0.5868 | 0.6441 | 31.79 | 35.81 |
| 17 | 0.6019 | 0.6480 | 32.40 | 36.11 |
| 18 | 0.5981 | 0.6425 | 32.16 | 35.73 |
| 19 | 0.5925 | 0.6456 | 31.87 | 35.93 |

**Round 5 alone breaks G1.** Production's CPU (0.6289 ns/row) and completion (38.32 ms) are
both far outside its other 19 rounds:
- CPU 0.585–0.602 ns/row;
- completion 31.55–32.40 ms.

Without round 5, production's spread would be 1.028 (CPU) and 1.027 (completion). C's
round 5 is ordinary (35.73 ms). Round 5 is also the **only** round in which v2 appears
within budget, at C ÷ P b4 1.019 and e4 0.933. In every other round, b4 ≥ 1.072 and
e4 ≥ 1.110.

So the anomaly flattered v2. Report 217's diagnostic, which excludes round 5, gives about
1.090 and 1.125. Its cause is unknown, because the records don't establish one.

This analysis neither reclassifies the capture nor amounts to a revised experiment. The
registered result stays INVALID.

**Directional comparison with v1.** Both captures are formally invalid and were made
separately. Their descriptive warm b4 went from 1.316 (v1) to 1.086 (v2), and e4 from 1.364
to 1.114. That shows a large reduction in construction overhead, but it isn't a registered
v1-against-v2 result. Completion time remains the larger gap.

## Report 217: the guardian's disposition, as relayed

Kept as relayed. A chart that didn't survive the relay is marked as such, and the heading
levels are adjusted; the words are unchanged.

I understand the frustration. We've now had two completed 20-round captures invalidated, despite extensive preflight work.

But this time the evidence contains a major positive finding: constructor v2 has substantially reduced the construction overhead seen with v1. The problem is that it still appears too expensive, particularly in four-worker completion time.

I've independently checked your uploaded capture archive.

Archive integrity verified: 69/69 files

SHA-256: `ebdb74549b39f83f07ae1f7d1c943916bac2ecb0be85868799aa30fe5e3d8af9`

I also recomputed all six construction ratios directly from the raw 75,087 JSONL records. They match the archived analysis exactly.

### Why it was invalidated

This was not another synchronization failure. All three G2 checks passed, with zero start-skew outliers.

Only the production baseline's warm four-worker stability failed:

| Stability gate                    | Maximum allowed | Observed |
| --------------------------------- | --------------- | -------- |
| Production W4 CPU cost, max/min   | 1.05×           | 1.0743×  |
| Production W4 completion, max/min | 1.05×           | 1.2147×  |

The biggest disturbance was production round 5. Its completion time jumped to 38.32 ms, compared with approximately 31.5–32.4 ms in the other rounds. V2's completion time in that round was an ordinary 35.73 ms.

That's a real baseline anomaly, not evidence of a v2 correctness failure. Its underlying cause is unknown.

The formal result therefore remains INVALID, not FAIL.

### What v2 achieved

The following ratios are descriptive only, because the capture was INVALID.

| Four-worker measure          | V1 capture | V2 capture | Required |
| ---------------------------- | ---------- | ---------- | -------- |
| Construction CPU, C/P        | 1.316×     | 1.086×     | ≤ 1.067× |
| Construction completion, C/P | 1.364×     | 1.114×     | ≤ 1.020× |

*[A chart, "Constructor overhead versus production", plotting the table above against the required ceiling, didn't survive the relay. Its caption: "Lower is better. Descriptive ratios from two formally invalid captures, not a registered comparison between v1 and v2."]*

Across the two captures, the observed overhead decreased substantially. Because both captures were invalid and conducted separately, that improvement is directional rather than a registered v1-versus-v2 result.

However, v2's four-worker completion disadvantage remains substantial.

I checked the effect of the anomalous production round 5 as a post hoc diagnostic, not as a revised experiment. Excluding it would actually make v2 look worse: approximately 1.090× construction CPU and 1.125× completion.

That matters because the production anomaly was helping v2's reported ratios, not hiding a potential win.

The full-block-only results also show the same disadvantage, so short blocks aren't the explanation.

### My engineering decision

Do not rerun this capture.

The protocol requires INVALID, and we should respect that. But a third lengthy measurement merely to get a formal FAIL is not a good use of your machine or your time.

I recommend closing the v2 construction-cost measurement investment, preserving the prototype as a successful algorithmic result, and leaving compact30's validated traversal ADVANCE intact.

This is not an instruction to abandon the compact30 representation forever. It means we should not fund another constructor experiment without a genuinely different performance argument.

#### Message for Claude

Guardian disposition on the i9 constructor-v2 capture:

**Formal result: INVALID. Engineering decision: STOP further measurement investment in constructor v2.**

The Guardian independently verified archive SHA-256 `ebdb74549b39f83f07ae1f7d1c943916bac2ecb0be85868799aa30fe5e3d8af9`, all 69 manifest files and all six ratios against the raw records.

G1 failed for production's warm W4 CPU stability (1.0743 versus 1.05) and completion stability (1.2147 versus 1.05). All synchronization checks passed.

The descriptive warm W4 ratios were b4 = 1.08619 and e4 = 1.11437. Production round 5 was anomalously slow, which actually reduced the apparent candidate disadvantage. Excluding it diagnostically, without reclassifying the capture, makes both ratios worse.

Do not relabel INVALID as FAIL. Do not weaken the gates, rerun, tune or integrate v2.

Archive the complete capture and this decision. Preserve v2's independent correctness PASS, targeted Miri results, architectural improvement and frozen source. Preserve v1's separate STOP and the compact30 traversal VALID ADVANCE.

No new experiment is authorized.

The distinction worth preserving is that Astra's second design worked technically: it removed a substantial amount of constructor overhead, passed independent correctness review and survived Miri. What it did not establish is an affordable way to build compact30 tables under the current performance requirements.

You have enough evidence now to make an engineering decision without putting the i9 through another capture.

## Report 218: the researcher's assessment, as relayed

Kept as relayed. The stat blocks are put in a table and the heading levels are adjusted; the
words are unchanged.

The researcher wrote this from the console output, before seeing the archive. That is why it
says the per-round pattern is unknown, and why it recommends not to stop "from this console
alone". Report 217's decision came after the guardian's own inspection of the archive. The
per-round table above answers the researcher's open questions: which rounds broke G1 (round
5 alone), and whether the cost difference holds in the other rounds (it does, in all 19).

That's frustrating, particularly after how much work went into making v2 correct and reducing the scalar loop from 49 instructions to 12.

But there's a significant difference from v1: v2 appears to have recovered most of the construction overhead. The experiment is still formally INVALID, and the economic target has not been met, but this is much closer than the first attempt.

### What the results actually say

| | v2 | target | |
|---|--:|--:|---|
| four-worker CPU cost | **1.086×** | ≤ 1.067× | above budget |
| completion time | **1.114×** | ≤ 1.020× | |

Compare that with the previous capture:

| Warm four-worker metric | V1     | V2     | Change in observed ratio |
| ----------------------- | ------ | ------ | ------------------------ |
| CPU cost, C/P           | 1.316× | 1.086× | \~17.5% lower            |
| Completion time, C/P    | 1.364× | 1.114× | \~18.3% lower            |

These are comparisons between separate, formally invalid captures, not a controlled paired v1-versus-v2 experiment. They suggest a substantial improvement, but cannot establish its exact magnitude.

And there is still a real economic problem: v2's reported confidence intervals remain entirely above both PASS thresholds.

### Why this run is INVALID

Interestingly, G2 passed completely this time. There were no synchronization outliers, and the largest warm start skew was just 0.50% for production and 0.14% for compact30.

The only failures are:

- `G1_warm_w4_cpu_ns_row: false`
- `G1_warm_w4_completion_ns: false`

Those are the stability checks on production's four-worker control. Unlike v1, where the stability failures were confined to one-worker results, the variation now affects the metrics that actually decide the experiment.

The console doesn't show production's per-round minimum, maximum or pattern, so we cannot yet identify the reason. Possible causes include CPU frequency variation or external contention, but the log doesn't establish either.

### How close is v2?

Using the observed point estimates:

- CPU cost: approximately another 1.75% reduction from v2's current cost would reach the 1.06714 ceiling.
- Completion time: approximately another 8.47% reduction would reach 1.02.

Those calculations concern the point estimates only. A registered PASS requires the upper confidence bounds to clear both limits, so slightly more improvement—or less uncertainty—would be needed.

This is why I wouldn't declare the v2 architecture exhausted. Astra's approach has apparently achieved a major reduction in the overhead it was designed to address. Completion time is now the larger remaining obstacle.

### What I'd do next

Do not automatically rerun or change G1. Preserve the archive as INVALID, and have the guardian examine the raw records for the exact production-control instability before making another investment decision.

The archive is:

`c30-construction-v2-20261103T222928Z.tar.gz`

SHA-256:

`ebdb74549b39f83f07ae1f7d1c943916bac2ecb0be85868799aa30fe5e3d8af9`

If you upload that archive, I can independently recompute the per-round data, identify which rounds invalidate G1, and examine whether the construction-cost difference is consistent despite the unstable control. That would be analysis of existing evidence, not another benchmark.

My assessment is formal INVALID, but materially promising engineering evidence—and considerably more encouraging than v1. I would not make a STOP decision on v2 from this console alone.
