# segmented-fused-postwalk: pricing the peak-RSS blocker (#1183)

*28 Nov 2026, local 4-vCPU cloud VM, no CI runs triggered. #1183 head `3c80d181` (src identical to the blocked exact head `56049aeb`) against its merge base `3d8fc666`, both built `--release --example ab` here; corpora `ab gen corpora 4` (CI's scale). Diagnostic only: under C2 the BLOCKED verdict on `56049aeb` stands, and nothing here is evidence for the claim's gate.*

## The question

The B5 review (`frontier/deltas/2026-11-27-segmented-fused-postwalk-b5-review.md`, on the PR branch) passed timing and blocked the merge on the frozen literal `peak RSS <= 1.02x` point bound: `pattern` 4T +4.3% ("more"), `wikidata` 2T +3.0% (n.s.). Its next step was "a revision that removes the extra peak RSS", with three candidate mechanisms to price first. Do any of them exist on `pattern`?

## The three candidates, read against the code

| candidate | applies to `pattern`? | why |
|---|---|---|
| `grow_out`'s `start == 0` first-growth estimate and the `raw/100 + 16 KiB` headroom | no | `pattern` is `b"0123456"` repeated: no run of 4 equal bytes, so RLE1's output length equals its input `n`; `out.reserve(n + 64)` covers the block and `grow_out` is never entered |
| `decode_block`'s `s.ll.clear(); s.ll.reserve(n)` beside a grown `out` | no | the 2T/4T runs go through `parallel.rs` -> `decode_owned`, which has no such reserve; and a reserved, untouched range is not resident |
| `out.reserve(n + 64)` against pooled buffers in the workers | no | workers' buffers are `Vec::with_capacity(n + n/16 + 64)` (`parallel.rs:1398`), so the reserve is a no-op |

## Local paired measurement (20 rounds, `ab run --spread --rounds 20`, 1/2/4 threads)

Peak RSS, paired geometric mean head/base with 95% t-interval, and each arm's per-run range:

| cell | head/base | head range (MB) | base range (MB) |
|---|---|---|---|
| `pattern` 1T | 0.956 [0.954, 0.958] | 15.8-16.0 | 16.3-16.8 |
| `pattern` 2T | 1.022 [0.984, 1.061] | 20.0-25.6 | 20.3-23.2 |
| `pattern` 4T | 1.002 [0.963, 1.044] | 36.0-43.4 | 37.0-44.4 |
| `onebyte` 1T | 0.992 [0.990, 0.994] | 33.9-34.1 | 34.1-34.4 |
| `onebyte` 2T | 1.014 [1.012, 1.016] | 30.7-31.0 | 30.2-30.6 |
| `onebyte` 4T | 1.014 [1.012, 1.016] | 30.6-31.1 | 30.2-30.6 |

A/A control (the base binary as both arms, 20 rounds): `pattern` 2T 0.982 [0.951, 1.015], 4T 0.991 [0.960, 1.023]; per-run ranges 20.4-24.9 / 36.0-43.4 MB.

## Reading

- **`pattern` 2T/4T carries no removable RSS.** Head against base is as wide as base against itself, and both arms span the same 36-44 MB on 4T. That is the bimodal queued-output regime of `2026-11-28-rss-bound-calibration.md` §2.4 (a 39 MiB valley; a single cross-mode pair moves the ratio by +-12%). The CI cell that blocked #1183 is one draw from it; the next run of the same src gave -0.6% (`3c80d18`, bench comment on #1183). At 1T, where RSS is deterministic, `pattern` is 4.4% *lower* on the head.
- **The only deterministic increase is `onebyte` 2T/4T, +1.4% (about +0.4 MB), inside the 1.02 bar.** `onebyte` is one block of n ~ 392 KB expanding 51x, so `decode_owned`'s pooled `n + n/16 + 64` buffer is regrown once to ~20 MB by `grow_out`; a copying realloc of the first ~417 KB is the likely source (not instrumented). Removing it would not change the gate's outcome.
- **So a revision aimed at RSS has nothing to remove on the cell that failed.** Under the frozen literal 1.02 point bound on all 14 multi-thread cells, a src change with no RSS effect fails a single CI run by chance in 17/27 strict-null runs (calibration §2.3). A "distinct improvement under the same frozen gate" is therefore a lottery on this gate, not an optimisation.

## What follows (no verdict changed)

- Don't spend a CI cycle on an RSS revision of #1183 for `pattern`. The priced levers are empty.
- The eligible route the frontier already uses for this failure mode is the target-machine rescue, as with `crc32-avx2-ring-rescue` and `runs-front-i9-rescue`: a paired i9-10900K run of #1183's code against its merge base, under a memory rule calibrated before its data (calibration §5), with CI for no-regression only. #1183's gain is real on CI's own machines (`text` -4..-9% wall; `runs`/`onebyte` -15..-21% at 2/4T, on three runs and two CPU families), so the i9 run tests transfer to the target and the eight-worker knee, not visibility.
- **Hold mechanics: keep #1183 open.** The parked hold records `pr_head = e12e0aa1`, but the PR head is now `3c80d181`. `finish_review(cancelled=True)` and reconciliation both require the two to match, so closing #1183 would strand `src/bwt.rs`, `src/rle.rs` and `src/lib.rs` the same way `src/crc.rs` is stranded now (claim `parked-hold-moved-head`). A successor continues on #1183: `take-review` checks only the claim card at the PR head (status `open`/`review`), the PR number and base ancestry, not `pr_head`.
