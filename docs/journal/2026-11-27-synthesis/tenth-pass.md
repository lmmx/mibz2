# Synthesis, tenth pass: the ninth pass's worker-bound knee holds only for unaligned wikidata

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml), 27 November 2026. It follows the ninth pass (#1074, `b7e2187`), which was recorded only as a delta ([`2026-11-27-synthesise-9.md`](../../../frontier/deltas/2026-11-27-synthesise-9.md)) and one `DECODER.md` bullet. This pass is read-only: no `src/` edit, no hardware run, no frozen gate reopened (C2). **(derived)** marks arithmetic across sources. Every derived i9 number here rests on VM diagnostics (C5).*

## What landed since the ninth pass (`b7e2187..f90e924`)

| PR | What it says | Station |
|---|---|---|
| #1075 (`534bc65`) | Station balance off wikidata. The 1-thread VM demand ratio k\* = W/S on seven text/binary corpora is 0.58–0.85× wikidata's (median 0.64). Derived i9 text knee ≈ 4.6–6.8 | both |
| #1077 (`c1239ce`) | Whole-decoder JCC 32B alignment audit. `OpenCells` `decode_block` −13.7% (native) to −15.6% (base), disjoint. `S` −1.2..−3.1%. Off-wikidata cells −0.6..−9.1%, 12 of 12 faster | codegen |
| #1076 (`8e04854`) | W4\_v3 MLP/layout probe STOP. Coupled K = 4, S = 16, 32 KiB is the x86-64 optimum on unchanged m. K = 6/8/10 spill (15 GPRs). JCC alignment alone cuts the control's cold A7+A9 by −17.9% and its total by −14.8% (646.98 → 551.46 ms) | W: A6c+A7+A9 |
| #1079 (`f90e924`) | A6b second pass, scoped STOP (sub-15% on unaligned builds). The remaining safe A6b room after #1046 is −12..−14% of A6b, about 2.7–3.0 ms/w at 8W | W: A6b |
| #1053 (`07cf47a`) | The A6b endpoint redesign, merged behind `a6b-endpoint-redesign`. Its i9 recipe is #1085 (open) | W: A6b |
| #1034 (`5ed20c1`) | Scanner coalescing + u8 LUT + BMI2, merged. Flag: the exact-head CI run shows `pattern` −7.3% at 4T (95% CI +2.6..+12.3%) | S |
| #1080 (open) | Segmented fallback build: sequential LF, trusted `fill_avx2`, AVX2 reverse assembly. −15..−16% inverse BWT on text/source, with zero effect on wikidata | W: Segmented |

## 1. Two independent results undo the ninth pass's flip, for different reasons

The ninth pass derived that at k = 8 the scanner (S′ ≈ 134–139 ms) and the composed workers (W′/8 ≈ 134.7 ms warm, 140–145.5 cold) tie, or that the workers bind. Both of its inputs were measured on wikidata with default codegen. Each of the two new results relaxes one of those conditions, and each one alone puts the scanner back on the critical path:

- **Corpus (#1075).** Scan cost per symbol is flat across corpora (2.3–2.8 ns on text). Symbols per output byte rise 1.9–4.6× off wikidata, while worker cost per byte rises less. On text, then, S′ − W′/8 ≈ (1 − k\*/8)·S′ ≈ 0.15–0.43 S′ **(derived)**. That gap is tens of milliseconds, not 0–4.
- **Codegen (#1076 + #1077).** Alignment helps the two stations unevenly: the workers by about −14..−15% and the scanner by about −1..−3%. #1077's station attribution puts 75–82% of the worker gain in `CE_open`, the most branch-dense module (397–418 of 1,712–1,746 violations). Applying #1076's cold ratio (0.852) to the ninth pass's composed stack gives aligned **W′/8 ≈ 115 ms warm and 119–124 ms cold**. Applying #1077's scanner ratio gives **S′ ≈ 130–137 ms (derived)**. Wikidata at k = 8 is then **scanner-bound by about 6–22 ms**.

These are separate mechanisms that agree in direction. The ninth pass's statement "workers bind at k = 8" is true only of wikidata with unaligned codegen, which is exactly the configuration every adoption figure so far was measured in.

**The premise that decides it is not settled.** `DECODER.md`'s JCC bullet says the i9-10900K carries Intel's JCC-erratum microcode mitigation. #1077's delta says Comet Lake-S stepping 5 "has the silicon JCC fix". They can't both be true. If the i9 pays the mitigation, the i9's measured baselines already contain the penalty, and alignment flips the knee as above. If it doesn't, the VM gains don't transfer, and the ninth pass's tie stands for wikidata. Only an i9 measurement resolves this, and it is the cheapest measurement in the frontier: two builds of shipped master and no code. That is why `jcc-aligned-release-build` rises from P3 to P2 in this pass.

**A trap in that claim's stop test (noted, not changed).** Its k = 8 bar is ≥ 1.5% wall. If alignment moves the workers below the scanner, k = 8 wall improves only by the scanner's own −1..−3%, even though the worker gain is about 14%. A near miss at k = 8 would then be the knee masking the gain, not the absence of a gain. The registration should report the S/W station split (`station_balance` on the i9) and k = 4..6, where the workers still bind. The bar itself is left for the registrant; C2 isn't engaged, since there is no data yet.

## 2. What this does to the worker stack at the eight-worker knee

The three worker lines touch disjoint routes and stations, so their CPU gains add by route mix. Their *wall* gains at k = 8 are all capped by S′:

| Lever | Wikidata (Open, 233 of 431 blocks) | Text (Open ≈ 16% of full blocks; Segmented 90.7%) | Pays at k = 8 when |
|---|---|---|---|
| Forward emission + vectored RLE1/CRC (A6c+A7+A9) | 46% of the composed worker | ≈ 0 (Open is a minority route) | wikidata, unaligned |
| A6b (#1053 + #1079's leftover) | 17% of the composed worker; leftover ≈ 2.7–3.0 ms/w | ≈ 0 | wikidata, unaligned; the leftover is below the tie margin |
| Segmented (#1080) | 0 | −15..−16% of inverse BWT, which is 52–66% of W, so ≈ −8..−10% of W | never at k = 8; text is scanner-bound. Pays at k ≲ 5 and in CPU |
| JCC alignment | −14% of W, −1..−3% of S | −0.6..−9.1% total | wherever the workers bind |

Two consequences follow:

- **A7 has no algorithmic lever left on x86-64.** #1068 killed the branchy literal paths. #1076 killed lane count (the 15-GPR wall), decoupled chase, pipelining and arena hoisting. The only remaining A7+A9 gain measured larger than 5% is codegen alignment (−17.9%). `forward-lane-count-probe`, opened by the ninth pass, is answered by #1076's K = 6/8/10 arms and is dropped here. A further worker gain on Open now needs fewer visits (m), not faster visits.
- **Worker builds keep their value, but the value moves.** At k = 8 on aligned wikidata and on text, the worker stack buys slack (CPU, energy, SMT headroom) and wall at k = 9–10, where the ninth pass found the workers' L3 burst. `forward-rle1-crc-vectored-build` and `segmented-fallback-build` stay at P2 because they ship correct work that CI rewards at 1T/4T. Neither should be sold as the 6 GB/s path at k = 8 until the i9 says which station binds.

## 3. The hybrid scan, demoted twice, is live on both axes

The ninth pass moved `pooled-scan-hybrid` to k = 9–10, because its condition S′ − W′/8 > ~5.5 ms failed at k = 8. On text the gap is 0.15–0.43 S′ (#1075). On aligned wikidata it is about 6–22 ms (§1). The condition holds in both cases. The pooled scan as built stays killed, and its verdict stands. The slack-only design (#1056: zero k = 6 regression, feeder-offloaded `Finder`) is the one that gains. It returns from P4 to P3. Its registration should state the corpus and codegen it is priced on.

Wall at k = 8 is then S′. The scanner levers are the 6 GB/s path again: #1034 (merged), `scanner-a2-fast-build` (−4.4..−7.4% S′; its dependency #1034 has merged, so it is admitted here) and #1031 (−11.2 ms `in_maint`). With S′ ≈ 130–137 ms aligned and A2 at −4.4..−7.4%, **S′ ≈ 121–131 ms (derived)**, under the 140 ms contour without the hybrid.

## 4. The #1034 flag, read with the station model

`pattern` is `b"0123456"` repeated: it compresses enormously and is almost all runs. It decodes in a handful of blocks, its k\* is very large, and its scan time is negligible. A scanner-only change can't move such a corpus's throughput at 4T. A 7.3% loss there therefore points away from the scan kernel's per-symbol speed and towards a **per-block fixed cost on the scanner's path**: LUT/table construction, BMI2 dispatch, `syms` reuse. With few blocks and four workers, every worker's start waits on that fixed cost, so pipeline-fill latency, not throughput, sets the wall. Two things fit this reading. The flag sees the loss only at 4T, and A2 (the per-block table and selector stages) is exactly what `scanner-a2-fast-build` cuts by −32..−38% standalone. That build may therefore also be the "materially corrected implementation" the flag asks for. The smooth probe `scanner-per-block-fixed-cost` (idea, below) tests this before anyone spends a CI rerun. This is a hypothesis about the flag, not its resolution; a different agent from the flagged work's author resolves the flag.

A second reading: CI judges at 1T and 4T. On a 4-vCPU runner, text's VM k\* is 3.7–5.4, so 4T is near text's tie. A worker lever like #1080 should show a muted 4T gain beside its 1T gain. That pattern is the knee, not a regression, and #1080's bench report should be read with this in mind.

## 5. Changes made in this pass

- `jcc-aligned-release-build` P3 → P2. Its `why` says it decides the binding station, and carries the k = 8 masking caution.
- `text-knee-i9` P3 → P2. Every knee statement now needs a corpus label (#1075), and this claim measures it.
- `pooled-scan-hybrid` P4 → P3, annotated with the corpus/codegen condition.
- `scanner-a2-fast-build`: idea → open (P2). Its dependency #1034 is merged, and 9 open claims sit under the limit of 12.
- `forward-lane-count-probe` dropped. #1076 answers it, and its dead-end row covers K = 6/8/10.
- New ideas: `station-balance-aligned` (smooth, P2: rerun `examples/station_balance.rs` under the JCC-aligned build on wikidata and the text panel, the VM test of §1's derivation), `scanner-per-block-fixed-cost` (smooth, P2, §4) and `a6b-v2-fold` (striated, P4: fold #1079's V2\_u128 ≈ 80 LOC into `src/open/endpoint.rs` when it is next touched).
- `DECODER.md` gains one bullet. The knee is conditional on corpus and codegen. State both.

## What would change these conclusions

- **The i9 doesn't pay the JCC penalty.** Then §1's codegen leg vanishes, and the wikidata tie stands. The corpus leg still holds for text.
- **#1075's ratio doesn't transfer to eight contending workers** (`text-knee-i9` REFUTED). Then the corpus leg vanishes.
- **Composition isn't multiplicative.** #1076's 0.852 is for the composed control, measured, and it is the one ratio used here for W′. The scanner ratio comes from #1077's shipped scanner, not the post-#1034 scanner.
