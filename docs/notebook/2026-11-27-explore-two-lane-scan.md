# Two-lane scan: interleaving two blocks' Huffman decodes in the one scanner thread

*Explore, 27 Nov 2026. Origin/master `3d8fc666`. VM numbers only: a 4-vCPU Xeon, model 207 (Emerald Rapids class, 48 KiB L1d, no JCC erratum), shared host, unpinned. None of this is i9 evidence.*

## The direction

The scanner's symbol decode (A3, about 81% of S) is one serial dependency chain per block. Each lookup is shift, then L1d load, then shift: 7 cycles on Comet Lake by the A3 dissection's count, about 13 cycles a lookup on the i9 once refills and slow entries are included. Nothing on the map uses the core's other issue slots during that chain:
- **Within-block parallelism** (speculative self-sync) is a dead end.
- **Pooled scan** moves the scan onto other threads, and **slack-hybrid** gives it to idle workers. Both spend more CPU and need more threads.

A **two-lane scanner** decodes block i and block i+1 in the same thread, one lookup from each in turn. Their chains are independent, so out-of-order execution overlaps them. It adds no thread and no CPU-seconds, and it doesn't move work to another station. The second lane needs block i+1's start bit before block i ends. A magic finder gives it: the AVX2 `Finder` (9.8 GB/s on the i9) ran on the 91%-idle feeder thread in the pooled-scan branch. The finder's guess is checked when lane A reaches its EOB (A's end bit must equal B's start), and a false candidate is discarded.

## Probe

`docs/journal/2026-11-27-explore/two-lane-scan/ilp_probe.rs.txt`, a `#[cfg(test)]` module appended to `src/block.rs` in a scratch worktree. Three items in `src/huff.rs` were made `pub(crate)` (`huff-visibility.diff`); production code was not otherwise changed.
- **Inputs.** Each block's header, selectors and tables are prepared once, outside the timing, along with its reference symbols from production `decode_runs`. The probe measures A3 only; A2 (table build) is out of scope.
- **Arms.** *single* is production `decode_runs`, block by block. *static* pairs blocks (2i, 2i+1); the shorter one's tail runs on one lane. *rolling* uses two lanes, and a lane takes the next block as soon as it reaches EOB, so one lane runs alone only at the end of the stream. A lane's per-run tails (fewer than 10 symbols left in a coalesced run) go through production `Huff::decode_run`. The lockstep loop is production's 5-step batch, interleaved A, B, A, B.
- **Build and method.** All arms are built with `-C target-feature=+bmi2`, the same codegen as production's BMI2 dispatch. Repeats are 33, rotating arm order; the ratio is the paired per-repeat median against *single*.
- **Exactness.** Every lane's symbols equal production's for every block: 431/431 wikidata and 17/17 text, static and rolling.

| corpus | blocks | rolling / single (paired median) | repeats faster | min-of-repeats ratio | single ns/sym | rolling ns/sym |
|---|---:|---:|---:|---:|---:|---:|
| wikidata.bz2 (SHA-256 checked) | 431 | **0.844** | 33/33 | 0.842 | 2.148 | 1.804 |
| wikidata, full blocks only (>100k syms) | 92 | 0.841 | 30/33 | 0.819 | 2.037 | 1.716 |
| text (repo `docs/` + `src/`, 14.8 MB, `bzip2 -9`) | 17 | **0.888** | 30/33 | 0.886 | 1.887 | 1.687 |

Static pairing gains only 0.966 on all wikidata blocks: unequal partners leave long one-lane tails, so the rolling schedule is the design.

**The codegen matters.** The first version kept each lane's bit buffer in a struct the compiler spilled to the stack. Its hot chain went `mov 0x100(%rsp)`, then `shr`, load, `shlx`, then a store back to the stack, putting store-forwarding on the critical path. That version gained only 0.923. Copying both lanes' `Reg`, `ck` and `rem` into locals for the lockstep loop gave the 0.844 above. Six stack references remain in the hot span. A third lane would raise register pressure further; it has not been tried.

## What it would be worth (derived, not measured)

- **i9 Wikidata.** With A3 at about 81% of S, a 0.84 A3 ratio is about 0.87 S, or roughly -18 ms of the 143.5 ms post-#1034 serial S on wikidata x4 if the VM ratio carried over. Wikidata is worker-bound at the i9's shipped 8T/10T plans, so this is masked there and pays at 1-2T and at k = 9-10.
- **Text.** The text knee is 4-5 on the i9 and the wall at k = 8 follows S, so a 0.89 A3 ratio is roughly -9% wall. Here the scanner sets the wall and nothing masks the cut.
- **Composition.**
  - *K = 3 A3 redesign* (`scanner-a3-k3-pipe-u8`): it cuts lookups, and interleaving hides each lookup's latency. They stack, but not multiplicatively: the fewer and wider K = 3 lookups leave less latency to hide.
  - *A2 fast build and the adaptive seam window*: orthogonal.
  - *Slack-hybrid*: complementary. Two lanes raise the scanner's own rate; helpers take blocks only when workers idle.
  - *Pooled scan*: a different trade. It spends CPU on other cores, which is what failed at the knee.

## Risks a build must price

1. **L1d.** The i9 has 32 KiB of L1d against this VM's 48 KiB. Two lanes keep two blocks' tables live, about 2 x 4 KiB root LUTs hot plus group switches, beside the two 2 KiB chunks.
2. **Finder cost and seams.** A finder on the feeder thread costs the scanner nothing, by the slack-hybrid note's measurements. Blocks that straddle 4 MiB input chunks need the seam window, or need to fall back to one lane.
3. **Ordering and admission.** Blocks finish out of order by at most one. The scanner can hold a finished block until its predecessor issues, which costs one extra symbols buffer (about 1.8 MB at MAX_SYMS u16); the alternative is issuing by place. The `READY_BYTES` and slot admission must count two blocks in flight.
4. **Errors.** A corrupt block i must still be reported as libbzip2 reports it, and lane B's speculative work discarded. Truncation checks per lane mirror `decode_runs_inner`'s `take` logic. The probe padded the input instead.
5. **JCC.** The VM lacks the erratum. An i9 run needs the JCC-aligned control beside the default build.

Filed as idea claim `scanner-two-lane-interleave`.
