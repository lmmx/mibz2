# Bounding the ready-block L3 working set: the gates price out

*Claim [`delivery-l3-working-set`](../../../frontier/claims/delivery-l3-working-set.toml), smooth probe, 28 November. No new runs and no `src/` edits. Its frozen stop test admits a striated tuning claim only if a tighter `(READY_BYTES, SLOTS_PER_THREAD)` cuts 8-10-worker wall by at least 2.0% on shipped `none` or 4.0% on a fast-worker arm (`S`/`SC`), with k = 6..8 regressing by no more than 0.5%. Before spending a workstation run, this note bounds what any tighter setting could gain, using the two traced i9 bundles the premises rest on. Both bundles' SHA-256 were checked: `c71c4a56` (pooled-scan, unpinned, `97e397ad…8867`) and `cc8ba137` (zero-copy-floor, scanner pinned to CPU 6 alone, `10df0d89…5e34`). The script is [`delivery-l3-working-set.py`](delivery-l3-working-set.py) (`python3 -I delivery-l3-working-set.py <c71c runs.jsonl> <cc8 runs.jsonl>`). It uses traced decodes only: n = 3 a cell in c71c and 4 in cc8. `ready avg` is the `ready_bytes` occupancy integral over the whole decode.*

## Verdict

**Neither gate can be met, so no striated tuning claim is admitted** (C2 intact: the gates are unchanged, this is an upper bound against them).

- **The shipped `none` arm can't clear 2.0%. Its bound is about 0.** At 8-10 workers the wall is the scanner's. With the scanner pinned (cc8), its A2+A3 holds at 169.4-176.7 ms across k = 8..10 (a single 176.7 at k = 8; 170.8-171.3 at k = 10), while ready avg rises from 0.36 to 1.55 MiB and hwm from 6.0 to 16.3 MiB. Ready-block L3 residency does not reach the critical station. The workers are not critical: they wait for the scanner (`job_wait` 5-25 ms a worker pinned, 6-45 ms unpinned), so cutting their A9 or `copy_out` (the reader, also off the critical path) moves no wall. Besides the scan, the only wall terms a cap acts on are the scanner's `slot_wait` (0.8-9.7 ms) and drain, and a tighter cap or window can only *raise* `slot_wait`. Within a cell, the high-wall runs are the high-`slot_wait` runs (cc8 k = 9: 199.0 ms at `slot_wait` 9.2 and ready avg 0.95, against 187.7-189.0 at `slot_wait` ~1). The unpinned scanner's 187-192 ms at k = 10 is scheduling: pinned, the same arm's scanner is 171 ms.
- **The fast-worker `S` arm's bound is about 2.9% against the 4.0% gate.** In `S` the workers are critical (`job_wait` 0.1-2.5 ms), so worker inflation is wall. It splits in two:
  - **Non-uniform:** the slowest one to five workers in every k = 9..10 run share a physical core with another busy thread (SMT siblings `c` and `c ± 10`; §2). They take 1.3-1.7× the per-block time, and their late head-of-line blocks are what pile up the ready queue: the high-ready runs are the runs with the worst collisions (S k = 10: ready avg 2.04 MiB, slowest workers 2,727 and 2,695 µs a block, against 1.14 MiB with 1,964 and 1,861). A cap changes no placement, so this part is out of its reach.
  - **Uniform** (the faster half of the workers): +0.5..2.0% over k = 6 at 8, +3.1..3.6% at 9 and +6.7..9.9% at 10. This includes the k × per-worker live set (`worker-l3-footprint`) and memory bandwidth, as well as ready bytes. Within the k = 10 cell its slope on ready avg is about 52 µs a block per MiB (1,596 → 1,643 µs over 1.14 → 2.04 MiB), 3.3% per MiB. If all of that slope were caused by ready bytes, and a cap could pull ready avg from the cell median 1.40 MiB to the k = 9 level of 0.52-0.66 MiB without head-of-line stalls, the saving is at most 0.88 MiB × 3.3% ≈ 2.9% of `S`'s 10-worker wall. That is under 4.0%. At 8 workers the cap never binds (hwm 6.0-8.6 MiB < 12 MiB), and a cap low enough to bind is overridden by its own floor of k + 2 blocks.
- **`SC`** is excluded as a fast-worker reference for real code. Its 8-10-worker A9 and `copy_out` blow-up needs the C oracle's 120.6 MiB cold store streamed while every worker is busy (third-pass §1); `S` has the same 16.3 MiB hwm without it.
- **The k = 6..8 guard pushes the same way.** The claim's sixth-pass note and `eight-worker-coupling` found `SLOTS_PER_THREAD < 4` worsens k = 8 head-of-line `slot_wait`, because 15-16 of the 32 slots hold runts.

## What the data point at instead (outside this claim)

- **Placement at 9-10 workers is the larger term.** In `S` at k = 10 the mean worker is 13-30% slower per block than at k = 6, against 7-10% for the faster half; the difference is sibling sharing. `i9-clock-modes` (pinned vs unpinned, per-thread cycles and placement) owns that question; §2 is evidence for it.
- **The window's real cost runs the other way** (looser, not tighter): runt-exempt slots, about 1.21 ms at k = 8, already priced in the `8-worker coupling gap` dead-end row.

## 1. Stations (k = 8..10, per traced decode)

| bundle | arm | k | wall | ready avg / hwm MiB | scan A2+A3 | scanner `slot_wait` | worker `job_wait` | `copy_out` |
|---|---|---:|---:|---|---:|---:|---:|---:|
| cc8 pinned | none | 8 | 193.5-200.0 | 0.36-0.38 / 6.0-7.7 | 169.9-176.7 | 5.4-7.1 | 4.9-10.8 | 31.2-33.1 |
| cc8 pinned | none | 9 | 187.7-199.0 | 0.47-0.95 / 6.0-10.3 | 169.4-171.3 | 0.9-9.2 | 12.3-17.7 | 34.1-38.4 |
| cc8 pinned | none | 10 | 198.0-200.1 | 1.41-1.55 / 12.9-16.3 | 170.8-171.3 | 7.1-9.7 | 19.8-25.2 | 44.9-47.8 |
| c71c | none | 10 | 211.0-214.5 | 1.13-1.30 / 13.7-16.3 | 187.2-191.6 | 3.5-5.6 | 38.0-45.0 | 42.1-43.1 |
| c71c | S | 9 | 165.7-168.5 | 0.52-0.66 / 8.6-9.4 | (oracle) | — | 0.1-0.2 | 35.5-37.7 |
| c71c | S | 10 | 161.4-179.1 | 1.14-2.04 / 14.6-17.2 | (oracle) | — | 1.3-2.5 | 44.2-48.7 |

## 2. Worker busy per block (A5..A10, µs per A7 visit), c71c

| arm | k | ready avg | faster half (vs k = 6) | mean (vs k = 6) | two slowest | on a shared physical core |
|---|---:|---:|---|---|---|---|
| S | 9 | 0.52-0.66 | 1,542-1,549 (+3.1..3.6%) | 1,565-1,591 (+4.6..6.4%) | ≤ 1,762 | the slowest 1-2 |
| S | 10 | 1.14 | 1,596 (+6.7%) | 1,691 (+13.1%) | 1,964, 1,861 | 5 workers |
| S | 10 | 1.40 | 1,599 (+6.9%) | 1,752 (+17.2%) | 2,182, 2,158 | 4 workers |
| S | 10 | 2.04 | 1,643 (+9.9%) | 1,938 (+29.6%) | 2,727, 2,695 | 5 workers |
| none | 9 | 0.46 | 1,564 (+5.5%) | 1,599 (+7.9%) | 1,739, 1,617 | 2 |
| none | 9 | 1.22 | 1,553-1,574 (+4.7..6.2%) | 1,783-1,796 (+20..21%) | 2,381-2,494 | 3 |

In every run the slowest workers are those sharing a core. The script prints the full table, with k = 6..8.

## Caveats

- **This is observational.** Only an intervention (a capped run on the i9) measures what a cap does. But the `none` bound rests on the scanner being invariant to ready occupancy when pinned, and on the workers' slack; a cap can't turn those into a gain. The `S` bound assumes the whole within-cell uniform slope is caused by ready bytes, which overstates it.
- **These are three or four traces a cell,** with tracing overhead (ledgers on). The timed walls (report.txt) agree with the cell medians to within 1-2%.
- **The C2 stop test is unchanged;** this note does not relax a frozen verdict. Any future fast-worker arm (after the fold, forward emission and the A2/A3 builds) can re-price the `S` bound by rerunning the script on its own traced bundle.
