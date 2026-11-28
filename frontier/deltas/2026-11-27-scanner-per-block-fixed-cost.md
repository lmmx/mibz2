claim: scanner-per-block-fixed-cost
by: antigravity
outcome: done
evidence: examples/scanner_per_block_fixed_cost.rs, docs/journal/2026-11-27-explore/scanner-pattern-regression.md, CI runs 45947633032, 45921211004, 45949114795, 45933661519, 45947374790, 45944904862, 45928216036

**Verdict: `REFUTED` (disjoint in the opposite direction).**

1. **Per-block fixed scanner cost (`A2`) and first-block latency (`examples/scanner_per_block_fixed_cost.rs`, `#![forbid(unsafe_code)]`, 455/455 blocks bit-exact across `pattern`, `runs`, `onebyte`, `wikidata`, 1T best-of-9 interleaved, baseline x86-64 and `-C target-cpu=native`):**
   - On `pattern` (`22` blocks, `n_sel = 3 < PAIR_GATE_SEL = 64` on 22/22 blocks, `2,661` syms total, `0.1%` of 1T decode time): `#1034` cuts `A2` setup **−88.9%** (`3.934 → 0.437 µs/blk`, `86.56 → 9.62 µs` total, disjoint) and cuts total `S = A2 + A3` **−77.1%** (`96.03 → 22.03 µs`, `4.36×` faster, disjoint; native `−77.0%`). Cold first-block scan latency (`open → read → blk0 → worker recv`) drops `30.44 → 17.06 µs` (`−43.9%`; warm `25.04 µs` inside parent's `[18.73..45.60] µs` range).
   - `onebyte` `S` drops **−85.4%** (`4.31 → 0.63 µs`); `runs` `S` drops **−19.7%** (`305.6 → 245.4 µs`, `A2` flat `+0.06%`); `wikidata` `S` drops **−19.3%** (`96.66 → 78.01 ms`).
   - Implication: `scanner-a2-fast-build` will not move 4T `pattern` wall (`A2` on `pattern` is already `0.006–0.010 ms` across 22 blocks) and should be judged on `wikidata`/`text`.

2. **Root cause of CI `45947633032` `+7.3%` 4T `pattern` wall (`−2.2%` CPU, `+7.9%` peak RSS, `+62%` `runq`):**
   - Pre-existing bimodal 6-on-4 thread oversubscription (`main` + `mibz2-scan` + 4 `Segmented` workers, `slots = 16`): OS preemption of `main`/head-of-line worker stalls strict-order `ready` draining, causing workers to allocate fresh `956 KiB` `Vec<u8>` output buffers (`+3..8 MB` RSS, lower SMT concurrency → lower total CPU, `+8..14%` wall). Confirmed by `occupancy --features trace` (`ready→taken` up to `24.9 ms`; `slots = 4` caps RSS at `33.3 MB`) and across 10 paired CI runs (`r(rss,wall) = +0.33..+0.84` in 10/10 runs; `r(rss,cpu) = −0.19..−0.67` in 9/10).
   - Across 4 true null-control CI runs (zero Rust decoder changes, `80` executions), the high-RSS (`>38.5 MB`) spike rate is `30/80` (`37.5%`); **Python-only PR #1033 (`45933661519`, `src/python.rs` only) independently fired `+6.0% [+0.2%, +12.2%]` slower wall and `+5.9%` RSS on `pattern t=4`**.
   - Across all 3 independent `#1034` CI runs (`45921211004` `2/10`, `45947633032` `6/10`, master `45949114795` `3/10`), `#1034`'s high-RSS rate is `11/30` (`36.7%`) — identical to null controls. Local paired `N = 30` quiet VM runs (`base` and JCC-32B) show zero regression (`−5.6%` / `+0.8%` wall).
   - Structural mitigation aligns with `ready-cap-worker-gate` (`#1094`). Evidence appended to `frontier/flags/2026-11-27-scanner-coalesced-ci-acceptance.md` (kept `status: open` for independent policy disposition).
