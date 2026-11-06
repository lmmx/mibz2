# Published-model assessment: traffic, latency and work per byte

Trigger: [6643809123](https://github.com/lmmx/mibz2/pull/145#issuecomment-6643809123).
Researcher assessment, 2026-11-04. Read-only research; no implementation, new experiment, merge or authority change.

**Prioritise the already-described P87 buffer-reuse comparison for a direct interference intervention. Keep scalar tiling as a distinct work-per-byte hypothesis. A smaller table needs a production walk-and-build price before it outranks either. The published record does not establish that instructions stop binding at two or more workers.**

## What the sources actually establish

Read the [B2 analysis at 9723f06847b88a10ba93d07c3f35680e58766015](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-b2-gathered-walk.md), its portfolio/register, and the following master posts at bbce58b1a4d129e3dbe11746efbefddd1dd1a14b:

- [Room-walk overlap](https://github.com/lmmx/mibz2/blob/bbce58b1a4d129e3dbe11746efbefddd1dd1a14b/docs/site/blog/2026-10-31-room-walk-overlap.md).
- [Memory interference](https://github.com/lmmx/mibz2/blob/bbce58b1a4d129e3dbe11746efbefddd1dd1a14b/docs/site/blog/2026-11-01-inverse-bwt-memory-interference.md).
- [MLP](https://github.com/lmmx/mibz2/blob/bbce58b1a4d129e3dbe11746efbefddd1dd1a14b/docs/site/blog/2026-11-02-memory-level-parallelism.md) and [technical notes](https://github.com/lmmx/mibz2/blob/bbce58b1a4d129e3dbe11746efbefddd1dd1a14b/docs/site/blog/2026-11-02-memory-level-parallelism-notes.md).

The overlap post calls Path B a hypothesis, not a verified machine-code finding. Its 38.55/4.49 = 8.59 is a latency-hiding factor, **not measured occupancy**; serial-chain latency need not equal the loaded/prefetched walk's latency. The illustrative eight-chain floor is 4.819 c/B versus 1.205 for 32 chains. Preserve exposed independence; do not promote either number to a measured production bound.

The interference post supplies stronger evidence for a concurrency problem: traversal+assembly 1.234→1.806 ns/row, versus almost unchanged instructions per byte for the whole decoder. Its off-core 39→58-cycle duration includes prefetches and is not dependent-demand latency. It narrows the suspicion to shared-resource interference; it does not identify capacity, request congestion or write interference separately. These older measurements are not new #157 counters.

## The interval model does not imply the proposed dichotomy

In B2, U is macro-ops per output byte, D dispatch width, R retire-window capacity and ell a latency parameter. **R is not a cache-line budget; ell is not a count of lines.**

The simple form is w = U/D + ell*U/R. The actual prediction form is:

    w = issue(U, ports, gather) + ell*max(U/R, 1/m, 1/K) + replay

Here m is the assumed request-buffer cap and K exposed chains; gather adds its latency penalties and tail work remains incompletely priced. With published Zen-3 assumptions, U=14.8, D=6, R=256, ell=38.55:

- issue = 2.467 c/B;
- window penalty = 2.229;
- sum = 4.695 versus observed 4.49.

Calibrating to 4.49 instead gives ell≈34.998 and penalty 2.023. B2 explicitly reports that a step simulation of documented mechanics instead predicts 2.47. Thus the additive fit is useful, but its mechanism is unconfirmed.

Within the uncapped form, reducing U reduces **both** terms: dw/dU = 1/D + ell/R. Under the published SMT-partition assumption, halving R and D doubles each thread's cost while preserving a U reduction's ratio. B2 §6 consequently predicts similar relative gains at four copies, conditional on no new shared bottleneck. “At >=2 workers it is ell,R rather than u” therefore is not a deduction from this model. Two workers also need not mean two SMT siblings; placement matters.

#157 attacks Path-B control work, but is not B2's full eight-step tiled-output kernel. Its Callgrind retired-instruction ratio is not U, and its Rust 1.97 sample is not the Rust 1.99.1 timing binary/corpus. First-run null-compatible multi-worker timing consequently cannot be substituted for B2's matched B/A falsifier. Conversely, the fit cannot rescue a failed production effect: a transferable mechanism must predict the actual binary, placement and whole-decoder result. The source's proposed B/A≥.95 closure should itself be scoped to that tested kernel and conditions, not all future fewer-op mechanisms.

## Which proposal actually changes cache-line demand?

| Intervention | Published benefit mechanism | What it does not establish |
|---|---|---|
| Scalar tiling, B | Lower issue/AGU work and window occupancy; amortised output bookkeeping | Still one random T load per byte. No demonstrated reduction in shared T-line fills |
| Gather, C | Less scalar state/output work; more exposed elements | Same random T elements; gathering is not evidence of coalescing unrelated lines. Extra latency, shared execution resources and tail costs remain |
| Smaller table | Fewer distinct resident table lines; potentially better reuse and lower traffic/latency | Size alone does not specify misses/byte. Additional dependent accesses, extraction and construction can consume the gain |
| P87 reuse of dead ll | About 0.9 MB fewer distinct written addresses per worker/block; potentially less ownership/eviction/write-back interference | Same table, logical assembly bytes and copy work. Saved footprint is not measured saved traffic |

For the consistent 7763 run, B2 predicts scalar B at 3.30–3.42 c/B, or **9.9–11.0%** of the 10.78 c/B decoder baseline. The summary's broader 10–13% is not the precise range for this row. C at 2.53–3.17 gives **12.2–18.2%**, with an assumed floor 1 + 35/24 + .05≈2.51. These are old-model projections before new tail/shared-load penalties, not achieved gains and not additive to #157. Reprice the remaining work against #157 if it becomes the baseline.

P87's 0.9 MB is approximately 14,063 distinct 64-byte line equivalents per worker (3.6 MB across four workers), not that many avoided requests. Assembly follows its own traversal: any traversal improvement must operate through other workers, later blocks or altered overlap. Both arms should retain equal allocation/pretouch and identical build/walk code as the existing proposal specifies. Dropping an allocation is a different comparison.

For a smaller representation, let H denote non-walk cost per output byte and h the extra walk work not represented by U. The production break-even condition is:

    H_new + issue_new + ell_new*max(U_new/R, 1/m_new, 1/K_new) + h_new
      < H_old + issue_old + ell_old*max(U_old/R, 1/m_old, 1/K_old) + h_old

This is a comparison framework, not a validated predictor. Extra dependent accesses need their own recurrence/service terms; they cannot be hidden by calling the table smaller. In a separately established bandwidth-limited regime, reducing bytes transferred per output byte b helps the b/B term; reducing allocation size without reducing b need not help. Published #42 already supplies the caution: its smaller 0.6 MB representation improved a single chain yet lost heavily with 32 lanes. Construction and full-block tails must remain charged.

## Decision and discriminating evidence

1. **Reuse first as the smallest existing production question, not the largest promised payoff.** The published contrast is D=(C_A4−C_B4)−(C_A1−C_B1), with .10 CPU-s/GB proposed as material. Require absolute four-worker improvement separately; D>0 can coexist with a slower decoder. Separate traversal, assembly, build and RLE1.
2. **If the objective is to identify the shared cost, use the already-described fixed-decoder/controlled-neighbour design.** Its real-decoder neighbours must first reproduce at least 70% of the real slowdown. A failed realism gate invalidates that setup, not the interference hypothesis. Footprint/read-request/write/stream arms measure sensitivities, not clean percentages of causality.
3. **Retain tiling as work reduction, and compact tables as a priced locality hypothesis.** Tiling/gathers cannot currently be sold as fewer shared-cache lines. A compact layout becomes preferable when an attainable reduction in loaded latency or traffic pays for its actual production recurrence, instructions, build and tail. No existing number in this checkpoint supplies that price.

This ranking uses the published questions and controls. It grants no execution authority and reports no new performance result. [Arithmetic](audit.json) is independently recomputed from the cited numbers.
