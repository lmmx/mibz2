### (a) Dependent transitions a byte

Slice: P01 to P16 of the 30 October register, plus one added row, N1, for the k-step
composition of Ψ or LF (the companion's section 3.1, which has no P ID). Read only. Nothing was
built or run. Every figure is copied from the file named beside it. "(inferred)" marks my own
arithmetic.

**Source keys.** All under `docs/journal/` unless a path is given.

| key | file |
|---|---|
| REG | `scratchpad/portfolio.md` (the 30 October register) |
| COMP | `scratchpad/portfolio-main.md` (its companion), sections 2.2 and 3.1 |
| PWK | `2026-10-27-pair-walk.md` |
| PFC | `2026-10-27-precopy-forward-chains.md` |
| BLM | `2026-10-27-bloom-inverse-bwt.md` |
| LR | `2026-10-27-literature-reading.md` |
| MLP | `2026-10-27-mlp-scheduling-and-run-structures.md` |
| MSW | `2026-10-27-move-structure-walk.md` |
| WSW | `2026-10-27-walk-sweep.md` |
| RS | `2026-10-27-research-synthesis.md` |
| WS | `2026-10-28-workstation-scaling.md` |
| XR | `2026-10-29-expert-review.md` |
| LS | `2026-10-29-literature-study.md` |
| SR | `2026-10-29-succinct-review.md`, read from `origin/research/psi-census` by `git show` |
| CPF | `2026-10-30-compact-psi-feasibility.md` |
| WR | `2026-11-05-walk-round.md` |
| BC | `2026-11-05-binding-census.md` and `2026-11-05-binding-census/nogo.md` |
| PW, SC | `2026-11-05-partitioned-walk.md`, `2026-11-05-segment-cost.md` |
| KM | `2026-11-07-knee-mechanism.md` |
| LFA | `2026-11-07-left-field-archive-check.md` |
| RF8 | `2026-11-08-run-fed-tvector-and-psi2-census.md` |
| PFR | `2026-11-09-pair-from-runs-result.md` |
| PKT | `2026-11-09-packet-census.md` |
| PHR | `2026-11-09-phrase-structure.md` ("Correction and close" stands over the body) |
| RFS, RFD | `2026-11-09-run-fed-stage-result.md`, `2026-11-09-run-fed-direct-result.md` |
| BWT | `2026-11-10-backward-table.md` |
| BDC | `2026-11-10-by-data-class.md` |
| SN | `2026-11-10-share-nothing.md` |
| DEG, DER | `scratchpad/reg2/design-e-registration.md`, `scratchpad/reg2/design-e-result.md` |

**Yardsticks used below.**
- Production's inversion on Wikidata: build 4.92 + room walk 10.41 = 15.33 instructions a row
  (DEG section 4, callgrind, first 3 blocks). SN section 3a rounds it to 15.3. SN section 1
  gives 6.2 + 10.1 = 16.3 a decoded byte from a whole-corpus run. The two differ by 6%.
- The break-even: "Break-even with the cooperative default (about 1.5 GB/s) is near g = 2.5"
  (SN section 3b), from the model 3.0 GB/s ÷ (0.3 + 0.7 g). It "assumes the table is small
  enough to behave as level 1's does" (0.4 MB a table). 2.5 × 15.33 = 38.3 instructions a row,
  and 2 × 15.33 = 30.7 (inferred).
- The knee: 14.4 MB of live table on the i9 (WS section 9; SN section 3).

| ID | idea, one line | the advantage it claims, as a quantity that could be false | status 2026-11-10 | closed by: construction / advantage / n.a., with the figure | whole-loop price: counted figure, or "derived: …", or "unknown" | cheapest falsifier now: kind and what | bearing on share-nothing | sources | premise the verdict rests on | premise still true? (yes / no / moved: how) |
|---|---|---|---|---|---|---|---|---|---|---|
| P01 | Ψ² pair walk: a dense 4 B/row table of ψ², two bytes a step, libsais's bigram build. | Half the dependent loads a byte makes build plus walk cheaper than production's (LR H1: walk "from ~12 toward ~6–7 c/B"). | rejected | Construction, twice. #34: 17.4–17.6 against 13.1 c/B (stage, VM, 1 thread); CI +40.6% at 1 thread, +24.3% at 4. #272, run-built table with #34's consumer: R/P 1.895 (1.885–1.905), CI EPYC 7763, one pinned copy. The advantage held past the knee only: 1.107 at 8 and 1.117 at 10 threads (i9, full decoder, against v0.1.4). | No instruction count of the complete loop. Timed: build 5.668 (run-built) or 8.246 (row-built) against 2.213 nominal c/row; entry point 11.705 against 6.184 c/B (PFR). REG keeps "27.8 against libsais's 23.2 instr/B". | None needed for the dense table. Its table is 3.6 MB, so the 2.5 break-even does not apply. Derived i9 bound: 631 MB/s × 1.117 = 0.70 GB/s at ten against about 1.5 (inferred, SN 3b and XR). | none: the same 4 B a row. It cuts reads past L2 a byte (libsais 0.897 against 1.366, LFA), which a small table removes anyway. | REG; COMP 2.2; PWK; XR section 7; BC nogo; LFA; RF8; PFR; PKT; BDC 4c; SN | "our walk is bound by instructions per step, not by memory latency" (PWK). "Beyond the knee is where no decoder should run" (XR). | moved. WR read CI's walk as bound by misses in flight; BC nogo ruled that "does not rescue". The second premise is reversed: SN makes ten independent workers the objective. LFA's condition "6 or more independent tables live" is now the plan's shape. |
| P02 | LF² as `LF[LF[i]]`: a gather pass and an 8 B/row pair table (#29). | Two bytes a load pays for a table twice the size. | rejected | Construction: "#29 tested constructions their papers do not use" (LR). CI Xeon 8573C, full decoder: +9.3% at 1 thread, +20.7% at 4; RSS 77 → 105 MB. | unknown. Nearest: a dense pair fill from runs costs 3.8× the build on Wikidata and 5.4× on text (BDC 4c, local). | None needed. A table of 7.2 MB a block is the wrong direction. | none: it doubles the table (BWT B: 4.5 MB at k = 2, 7.2 MB at k = 4). | REG; LR; MLP section 4; WS section 7; BWT section 3; BDC 4c, 4e | Class II by arithmetic: "the 7.2 MB table exceeds Skylake's STLB reach" (REG). | no for the STLB reading: page walks are 1–2.5% of cycles on the i9 (WS section 7). The size objection is stronger now. Never run on the i9 at any thread count. |
| P03 | E6: libsais's walk parameters in our ψ² port (24 lanes, a round of prefetch lead, 256-row chains). | The port's pair walk falls from about 10 c/B to libsais's about 6.7 (PWK, VM). | conditional | n.a.: never run. Its precondition E1 neither passed nor failed (XR). Nothing about E6 itself was measured. | unknown. Proxy: the pair consumer's residual is 6.072 against production's 3.984 nominal c/B; C37 would need "a reduction of more than 34.4%" (PFR). | Counting probe: a pair consumer in the room walk's skeleton (C37), by callgrind, against 10.41 a row. | none: the same table as P01. | REG; XR sections 4, 7; PWK; LFA; PFR | "only if E1 passes" (XR). "each costs about twice as much" (PWK), measured in the 0.1.4 flagged skeleton. | moved. E1 stands unresolved. The step premise is skeleton-specific: no pair consumer has run in the room skeleton. LFA lists E6 as "still untried". |
| P04 | Precopy: copy repeated text from bytes already decoded instead of walking it. | Copies replace "about half the walk's misses with sequential copies" (LR H2). | rejected | Construction three times, then the advantage on this corpus by count. #30: +118.2% (CI). #36: +18.0% / +6.0% with 31.5% copied (CI). Phrase walk, no per-step test: 4.72 against 2.71 ns a byte (VM, 1 copy). Laned: 15.3 instructions and 0.24 mispredicted branches a row against 10.4–10.65 (PHR close 2). | Counted: laned copy walk 15.3 instructions and 0.24 mispredictions a row (callgrind, 3 blocks, 8 lanes). Table build 1.35–1.40 against 0.64–0.69 ns a byte (PHR section 4, local). | Counting probe: "a copy event at 40 instructions and one misprediction or fewer" (PHR). It is about 150 and two now. | none: its state stream is 3.6 MB a block, "as large as the T-vector" (PHR section 5). | REG; PFC; LR; BC nogo; PKT sections 4, 6a; PHR | "The steps a forward chain skips are the cheap ones" (PFC). "the walk is not waiting on memory" (PHR close 2). | yes at 1 to 4 threads, on CI and the VM. Never read at 8–10 on the i9: coop section 5c's run has no record, and `bench-phrase-knee` was "withdrawn before any run" (PHR close 5). |
| P05 | A scan before the walk that finds repeat candidates (Bloom's reading). | Finding candidates before the walk removes the per-step neighbour test (+22–27% with 0% copied, PFC). | superseded | Advantage, through its realisation. `phrase_walk` takes candidates from the cells of a run scan, pays no per-step test, and still costs 15.3 instructions a row against 10.4–10.65 (PHR). | As P04. | none needed | none | REG; BLM; PFC; PHR | "if the forward-chain experiment shows the per-step compare is the cost" (BLM). | moved. The compare was removed and the loss stayed. The cost is the event: "about 150 instructions and two mispredictions" (PHR close 2). |
| P06 | Tunnels (Baier): blocks of rows that LF maps in parallel, used as copy opportunities. | Parallel paths found from the BWT predict where a walk can copy. | superseded | Advantage for copying, by census. Certified repeats cover 738 bytes of 209.7 M at k ≤ 8, 3.5 × 10⁻⁶ of the output (PKT). Tube packets leave 0.27–0.58 events a row (PHR section 3). As k-step cells the same structure is open: row N1. | n.a.: census only. | none needed for copying. For cells, see N1. | shrinks the table, through N1 only. | REG; LR; PKT; PHR sections 3, 8; COMP 3.1 T4 | "the same structure as KKP's s-runs and KP's chains" (LR). | yes. COMP T4: the record partition is KKP's s-runs and Baier's blocks. "A long cell certifies nothing unless the walk returns to it at a constant stride" (PKT section 4). |
| P07 | "Wikidata is in Precopy's regime." | A 17.5× bzip2 ratio puts Wikidata where Precopy wins. | rejected | Advantage. n/r 13.85 against KKP's n/r ≥ 50 (LR); copy-based inversion overtakes "only from n/r of about 80" (PHR close 3). The exact copy walk is 1.75 times production locally. | n.a. | Census: n/r of any new input class. PHR reopens at "n/r of 80 or more". | none | REG; MLP section 3; LS; PHR | Mean run length n/r decides the regime. | yes, refined. Phrases are heavy-tailed: 69.2% of bytes lie in phrases over 128 bytes, size-biased mean 484 (PHR section 3). Events still cost more than steps. |
| P08 | LZP pre-pass. | Long recurrences are removed before the BWT. | rejected | n.a.: class I. It is encoder-side and "can't apply to decoding standard bzip2 streams" (BLM). | n.a. | none needed | none | REG; BLM; PFC | The information is not in a standard stream. | yes |
| P09 | KKP's interleaved parts with stored end positions. | Stored ends give each lane a known start and length. | superseded | n.a.: class I, the ends are not in the format. Flagged pseudo-random starts replaced it (#4, #12). | The substitute's cost is now counted: 457 instructions a segment steady, 262–338 cycles a segment net (SC). Segment assembly is 15.2% of the default plan's cold writes (SN section 3). | none needed | none directly. For k-step tables the lane-end problem returns and is met by capped records (COMP 3.1 g2). | REG; LR; SC; SN; COMP 3.1 | "our version needs no metadata" (LR). | yes. But "no metadata" is not "no cost": see the price. |
| P10 | Auxiliary sampled indices (libsais's `I[]`). | Samples of text positions give independent chains. | rejected | n.a.: class I. "A bzip2 stream carries only `orig_ptr`, so they can't exist for our input" (RS section 7). | n.a. | none needed | none | REG; RS section 7 | The format carries no samples. | yes |
| P11 | Length-capped move structure for ψ: 8-byte interval records, one byte a step. | A table a sixth of the T-vector's working set makes build plus walk cheaper once the walk is memory-bound. | rejected | Construction, at one thread: +90% to +108% (g 1.90–2.08) against 0.1.5's build plus room walk, VM kernel, criterion ≤ −10% (MSW). The advantage under four or ten copies was never measured: E8 "never run [coord]" (REG). | derived: 41 instructions a step by listing (select variant, MSW) against 16 then and 10.41 now. Build 7.8–9.2 against 2.7 c/B, untuned, not counted in instructions. | Counting probe: the move room step in today's skeleton by callgrind. 41 alone is over 38.3 (inferred), so it closes on paper unless the step shrinks. | shrinks the table: 0.638–0.824 B/row, 574–742 kB a mean block, 1,584–2,005 kB the largest (MSW). | REG; MSW; MLP section 3; WSW; XR E8; LS; BC nogo; LFA; RF8; PHR close 2; BWT section 4; SN | "The room walk is bound by instructions per step" (MSW). Criterion: −10% at one thread. | moved. The gate is now g ≤ about 2.5 at ten for a small table (SN 3b). WSW called the four-copy condition "measured" on 27 October; the run was never made. PHR timed a move-style laned walk at 9.3–9.8 ns a byte against 1.7–1.9 (VM). |
| P12 | Run-aware LF traversal (Gagie, Navarro, Prezza). | Run boundaries expose predictable LF transitions. | superseded | n.a.: realised, not failed. Run lanes shipped: 82% of Wikidata's rows go in by 16-row run stores (RF8). Run-fed fills paused: #269 R_stage 1.1592, #277 R_direct 1.0525 (CI, STOP). | Backward 8-wide fill: 0.293 against `bwt::build`'s 0.458 ns a row (BWT section 2, local). No instruction count. | CI A/B on the stage harness: "front plus build must be at most 0.75 of today's on Wikidata" (BWT section 2). | shrinks other live memory: no last column, 12.8% of cold writes (SN section 3). | REG; LR; RF8; RFS; RFD; BWT; BDC section 2 | "they add nothing past the run lemma" (LR). | yes. The run lemma had more to give: in the LF direction "a run of L is a run of consecutive entries" (BWT section 1). |
| P13 | Two-tier translation directory: an 8-byte descriptor a 32-row group, payloads for non-run groups. | 5.70 bits a row with one load on about 78% of steps; "can win only past the knee" (REG). | open | n.a.: "Never built or timed" (LFA). | unknown. Model: chain about 53 cycles against 43, latencies assumed. Sibling, counted: FOR-14's ordinary step is 38 instructions against 16 (LFA). | Counting probe: RUN and non-RUN steps as a walk-only arm, instructions and mispredictions by callgrind (LFA steps 1–2), judged at 38.3 a row with its build, not at 12. | shrinks the table: 0.641 MB a block, 6.4 MB for ten (inferred). | REG; SR sections 9, 14; CPF; LS; LFA | Kill thresholds "bounded by #195's loss at 12 instructions" (CI, 4 threads). "residency has little left to buy at the operating point" (LFA). | no. SN makes residency at ten independent workers the objective and sets break-even near g = 2.5, not parity. Never read on the i9. |
| P14 | Rank/select and wavelet structures inside the walk. | A succinct structure gives a small table at an acceptable step. | rejected | Construction, for rank: design E's one-thread inversion is 3.665× (3.591–3.740), CI EPYC 9V74, Wikidata. Select and wavelet forms: analysis only. | Counted, for a rank step: 22–26 instructions a step, 48.2 a row with lane bookkeeping, against 10.41 (DEG section 4, DER). | Counting probe: reopen only at "no more than 1.44× the room walk's instructions a row (about 15)" (DER). | shrinks the table, if the step is paid for. | REG; SR section 6; DEG; DER; SN 3a | "Each step becomes several dependent operations (access class 4)" (REG). | no, for rank over L: E serves 98.18% of Wikidata's steps from one access (DEG section 3). The cost was instructions, which the class model does not count. |
| P15 | Blocked rank directory over L. | L plus per-block counts is a table well under 4 B/row. | rejected | Construction. The register's class I reason is void (next column). Built as design E, 2.039 B/row: FALSIFIED at 3.665×; build 1.827× the scatter's time at 0.82× its instructions (DER). | Counted: E build 4.06 + walk 48.21 instructions a row against 4.92 + 10.41 (DEG section 4). | Counting probe: E's step in the room skeleton, modelled at "about 26 + 6 = 32 instructions a row" (DEG section 4). | shrinks the table to 1.8 MB; ten are "about 18 MB", past the 14 MB knee (DEG section 6). | REG; CPF section 3; SR section 6; BWT section 1; BDC section 3; DEG; DER | "the walk needs select, not rank" (REG, from SR). | no. The walk can run in the LF direction: "the order the walk emits in costs nothing", checked on 431 blocks (BWT). |
| P16 | Compressed Ψ near nH₀ with class 1–2 access. | A table under 24 bits a row with one random load a step keeps the step's cost. | conditional | n.a.: theory. Nearest measured: `u` (24-bit) u1/o1 0.681 on the i9 (LFA); E (16.3 bits a row, 98% one access) 3.67× on CI. | unknown for Ψ. E's 48.2 a row for the LF analogue. | Counting probe on any candidate: whole loop with build at or under 38.3 a row (inferred), before timing. | shrinks the table | REG; SR sections 6, 10; LFA; DEG; DER | Classes rank by dependent-load depth; "instructions are cheap next to misses" (SR section 6). | no on CI at one thread. E met classes 1–2 below 24 bits and lost: "The walk is instruction-bound" (DER). Not read at ten on the i9. |
| N1 | k-step composition (COMP 3.1; BWT design D): records of a displacement and k bytes on the record partition, a rank directory, built from L's runs by doubling. | k bytes a lookup from a table of 0.90 MB (k = 4) or 1.54 MB (k = 8) on the median block, at an inversion within 2.5× production's. | open | n.a.: census and brute-force checks only. No walker and no build has been timed or counted. | derived: walk about 12 instructions a row at k = 4 and 6 at k = 8; build 6 to 10; inversion about 18 and 16 against 15.3 (SN 3a, from E's as-built counts). | Counting probe: the doubling pass by callgrind on three blocks, then the k-byte step in the room skeleton. | shrinks the table and removes cooperation. | COMP 3.1; RF8; PKT; PHR section 3; BWT sections 3, 4a; BDC 4c; SN 3, 3a, 3b | COMP's stop rule: "more than 0.67 × today's inversion instructions". SN 3a: "Two things are open". | moved. The gate is g ≤ about 2.5 (SN 3b). "Lanes" is already answered in COMP 3.1 g2 and RF8. The 1.44 MB size is the 12-byte record's; COMP counts 0.90 MB. |

#### The k-step composition: every counted size and build cost in the record

Sizes and counts are censuses (untimed, exact for the input). Costs are models unless marked
measured.

| quantity | value | where |
|---|---|---|
| Cells of Ψ^k over r, Wikidata, 233 full blocks, pooled | 1.22, 1.51, 1.97, 2.71, 3.85, 5.48 at k = 2, 4, 8, 16, 32, 64 | PHR section 3 |
| The same as records a block: m₁ (= r), min / p10 / median / p90 / max | 47,936 / 51,108 / 56,032 / 99,380 / 179,972 | COMP 3.1 h |
| m₂ | 57,676 / 62,125 / 68,029 / 121,772 / 203,914 | COMP 3.1 h |
| m₄ | 70,271 / 76,737 / 84,287 / 149,854 / 227,058 | COMP 3.1 h |
| m₈ | 89,048 / 100,416 / 109,726 / 195,995 / 268,255 | COMP 3.1 h |
| m₈ / (8\|B\|) | 0.18 / 0.24 / 0.25 / 0.26 / 0.30 | COMP 3.1 h |
| Cells a row, pooled | r/n 0.0722; r₂/n 0.0881; 0.1088 n at k = 4; 0.1422 n at k = 8 | RF8 section 3.2; BWT section 4a |
| Pair cells also cut by L's boundaries | 1.372 r (per block 1.207 to 1.526) | PKT section 3 |
| P⁴ resident, 8-byte records + 225 KB directory, MB | 0.79 / 0.84 / 0.90 / 1.42 / 2.04 | COMP 3.1 h |
| P⁸ resident, 12-byte records + 225 KB directory, MB | 1.29 / 1.43 / 1.54 / 2.58 / 3.44 | COMP 3.1 h |
| P¹⁶ resident, 20-byte records, MB | 2.62 / 2.97 / 3.28 / 5.50 / 7.49 | COMP 3.1 h |
| Design D as sized on 10 November | 1.6 B/row at k = 4 (12-byte records), 2.5 at k = 8 (13 bytes held in 16); 1.44 MB and 2.25 MB a block | BWT section 4a; SN section 3a |
| Two-hop move-style table, capped at 2n/r, with markers, 8-byte records | 0.96 B/row pooled (0.83 median, 1.44 p90, 2.44 max); 862 kB a mean block, 2,196 kB the largest; 0.53 fast-forwards a two-hop step | RF8 section 3.2 |
| Markers at k = 2 | 1,417 records a block for 512 starts, 2,823 for 1,024 | RF8 section 3.2 |
| Segment caps at k = 8 | at most 2(k − 1) = 14 extra records a start, at most 5 observed; "512 starts add about 5% pieces" | COMP 3.1 g2; BWT section 4a |
| Lookups a block under caps | H = Σ⌈ℓ_s/k⌉ < n/k + 512; n/H > 7.96 at k = 8 | COMP 3.1 g2 |
| Gap distribution along the traversal | mean gap 13.8; 84.8% of gaps ≤ 2 steps; 93.4% of rows in gaps longer than 2 | COMP 3.1 h |
| Parents passing to the next level unsplit (40 blocks, median) | 0.906 (1 → 2), 0.939 (2 → 4), 0.942 (4 → 8); most children of one parent 3,161 / 3,439 / 2,093 | COMP 3.1 h |
| Build work, as groups or cell-steps | 190–210k groups a block, "about 0.2–0.25 group moves an output byte" (parents in); 0.197 n to k = 4 and 0.34 n to k = 8 (pieces out) | COMP 3.1 c′; SN 3a; BWT 4a |
| Build traffic, P⁸, bytes moved an output byte | layout A 19.5 / 21.0 / 23.1 / 40.6 / 66.3; layout C 14.7 / 15.7 / 17.0 / 28.1 / 44.1; T-vector build 5.0 | COMP 3.1 h, i |
| Peak live MB during the P⁸ build | A 3.9 / 4.3 / 4.7 / 8.2 / 12.2; B 3.4 / 3.7 / 4.0 / 7.0 / 9.9; C 3.6 / 3.9 / 4.2 / 7.2 / 10.1; T-vector 4.5 | COMP 3.1 i |
| Per-block choice at a 4.5 MB peak budget | 161 P⁸, 40 P⁴, 16 P², 16 T-vector, of 233 | COMP 3.1 i |
| Build cost, models | P⁴ "≈7 a byte when overlap is heavy" to "≈15 at the bound" against 6.65 (COMP e); 0.34 n at about 15 instructions "is 5 a row" (BWT 4a); 0.197 n and 0.34 n "at about 30 instructions each: 6 to 10 a row against the build's 4.9" (SN 3a) | as named |
| Build cost, measured analogues | #42's three O(r) passes: "about 30–40 cycles a run each", 7.8–9.2 against 2.7 c/B (MSW, VM). Run-intersection pair build, dense: 5.668 against 2.213 nominal c/row, 2.563× (PFR, CI 7763) | as named |
| Walk cost, models | "≈10–14 instructions a lookup", 2.5–3.5 a byte at k = 4 (COMP e); "about 20 instructions for 8 bytes" (BWT 4a); about 23, 12 and 6 a row at k = 2, 4, 8 with E's step and bookkeeping (SN 3a) | as named |
| Negative controls, one block each | records m₈ 899,001, P⁸ 11.0 MB; random the same; text (the crate's sources) r 135,801, m₂ / m₄ / m₈ 200,487 / 277,841 / 352,169, P⁸ 4.45 MB | COMP 3.1 h |
| Text, other counts | r₂/n 0.245, r₂/r 1.48, table 2.20 B/row (RF8, 13 blocks); r₂ 0.449 n, r₄ 0.626 n, r₈ 0.797 n, D "2.5 bytes a row at k = 2 and 4.6 at k = 4" (BDC 4c) | as named |
| Path cache, k = 4 pair cells without repetition | 6.79 M distinct 4-paths, 0.45 r, about 29,000 a block; 0.1252 loads a byte | PKT section 5 |
| Checks | C_k = M_k in 22,940 random and 58,212 exhaustive cases; doubling reproduces the records in 21,970 cases; counting gate 8,160 levels; 10,188 capped segments, 0 failures; two-hop walk byte-identical on 464 blocks | COMP 3.1 a, c′, k, g2; RF8 section 3.1 |

The two censuses agree: the median block's m₈/m₁ is 109,726 / 56,032 = 1.96 (inferred) against
PHR's pooled 1.97.

#### Mined

1. **N1, the k-step table's size.** SN section 3a: "At k = 4 the table is 1.44 MB and ten of
   them sit exactly at the knee; at k = 8 it is 2.25 MB and six fit." That is BWT section 4a's
   record: "its first row, its LF^k base, its k bytes", 12 bytes at k = 4. COMP 3.1 d and h
   counted the same partition with a displacement record, "(D_I: u32, k bytes): 8 bytes for
   k ≤ 4, 12 for k ≤ 8", on all 233 blocks: P⁴ 0.79 / 0.84 / 0.90 / 1.42 / 2.04 MB and P⁸
   1.29 / 1.43 / 1.54 / 2.58 / 3.44 MB, directory included. The step
   `i = p.base + (i − p.first)` needs only base − first (inferred). So ten P⁴ tables are
   9.0 MB at the median block and 14.2 MB at the ninth decile, not 14.4 MB (inferred), and
   nine P⁸ tables fit at the median (13.9 MB, inferred). So k = 4 is at or under 1.42 MB on nine
   blocks in ten, against a target of about 1.2 to 1.4 MB; the worst block is 2.04 MB and needs
   the per-block fallback. Cheapest falsifier: a census, recounting the
   table's bytes with displacement records and caps in the LF direction on the 233 blocks.
   Worth: on SN 3b's model, g = 18 / 15.3 = 1.18 gives 3.0 ÷ (0.3 + 0.7 × 1.18) = 2.7 GB/s
   against about 1.5 (inferred). The size finding decides whether that model's "small enough"
   assumption can hold at ten workers.

2. **N1, the build's budget and its trip count.** SN 3a: "The build. Doubling is a loop a cell
   with a data-dependent trip count, the shape of E's build, which ran 1.83× the scatter's
   time", priced at "6 to 10 a row against the build's 4.9". Two sentences in the record weaken
   this. First, COMP 3.1 h: "About 94% of Ψ⁴ parents pass to Ψ⁸ whole ... a fast unsplit path
   is justified, and the split loop's cost is dominated by a few large parents." The trip count
   is one on 0.906 to 0.942 of parents. Second, SN 3b, written after 3a: break-even "near
   g = 2.5". With the walk at 12 a row at k = 4, the build may cost 38.3 − 12 = 26.3
   instructions a row before the tie, and 30.7 − 12 = 18.7 at the brief's "twice" (inferred).
   Even at E's worse time an instruction (1.827 ÷ 0.82 = 2.2×, inferred), 6 to 10 counted
   instructions stand for 13 to 22. Cheapest falsifier: a counting probe, callgrind on the
   doubling pass for three blocks; stop above 26 a row at k = 4. Worth: the build term moves g
   between 1.18 (build 6) and 1.44 (build 10), 2.7 to 2.3 GB/s on the model (inferred).

3. **N1, "open problem 1: lanes" is closed in the record.** SN 3a: "A lane that steps k rows
   of text at a time cannot land on a marker that lies inside its step, and
   `2026-11-10-backward-table.md` §4a did not say how lanes end." COMP 3.1 g2 did: "A capped
   record stores its own length, t ≤ k", "Checked by brute force: 10,188 segments ...,
   0 failures", "at most 5 per start were observed", and H < n/k + 512. RF8 section 3.1 built
   capped, marked two-hop records and walked them byte for byte on all 464 corpus blocks. BWT
   section 4a itself gives the rule ("stop after j bytes") and its cost, "about 5% pieces at
   k = 8". SN's proposed fix is the same construction. So one of D's two blockers needs no new
   design. Cheapest falsifier: none needed in the Ψ direction; a census of capped records at
   k = 4 in the LF direction. Worth: cannot be bounded apart from finding 1.

4. **P13, the two-tier directory, was shelved for the regime that is now the target.** REG:
   "Predicted to lose inside the L3, can win only past the knee." LFA's verdict rests on "the
   3×3 policy keeps three 3.6 MB tables in the L3 ..., so residency has little left to buy at
   the operating point", and its kill thresholds (RUN step over 10, weighted step over 12) are
   "bounded by #195's loss at 12 instructions", a CI result at four threads. SN reverses the
   operating point: cooperation costs "38.0 instructions a decoded byte against one thread's
   26.7", and break-even for a small table is "near g = 2.5". The two-tier table is the
   smallest exact-sized table in the slice with a one-load common step and no rank: 0.641 MB a
   block, 5.70 bits a row. It was "Never built or timed". Cheapest falsifier: a counting probe
   of its RUN and non-RUN steps in the room skeleton, judged at 38.3 instructions a row with
   its build, not at 12. A sketch at LFA's own threshold: 0.774 × 10 + 0.226 × 38 = 16.3
   instructions a step (inferred, RUN at the kill threshold, the rest at FOR-14's counted
   step). Worth: 3.0 ÷ (0.3 + 0.7 g); 1.8 GB/s at g = 2. Its build is unpriced, so the bound
   cannot be tightened from the record.

5. **Four gates in the record for one object, and the verdict tables use the old ones.** COMP
   3.1's stop rule is 0.67× today's inversion. RF8 section 3.3 stopped line B because "the walk
   alone would be about three times the line's own gate", a gate of parity with production.
   SN 3a judged C, D and E at "a one-thread walk within 1.44× of the plain walk, about 15
   instructions a row". SN 3b then wrote "The gate, from these figures and no longer from §2's
   1.44×". Re-read at 2.5: D at k = 2 is about 23 + 0.088 × 30 = 25.6 against 15.3, g = 1.67,
   2.0 GB/s (inferred), where 3a's table says "fails". RF8's line B is 13 + 8 = 21 against
   12 c/B by its own model, g = 1.75 (inferred), on a table of 0.96 B/row. C stays out:
   about 45 against 15.3 is g near 2.9 (inferred). RF8 left this door open itself: "a host or
   schedule where the walk is memory-bound rather than instruction-bound would change line B's
   arithmetic." With E's as-built step, COMP's own 0.67 rule would stop N1 at the k it
   names (at k = 4, 18 / 15.3 = 1.18, inferred); the 2.5 gate is what keeps it open. Cheapest
   falsifier: none for the re-reading; it is arithmetic on recorded figures. Worth: it restores
   k = 2 as the cheapest-to-build member, 0.088 n cell-steps against k = 4's 0.197 n.

6. **P01, the premise that discounted the pair walk's only win has been reversed.** XR section
   7: "Beyond the knee is where no decoder should run. pairwalk's gain is at 8–10 workers ...
   Cooperation keeps the tables under the L3 instead." SN: "ten workers each run the one-thread
   decoder and share nothing". LFA's reopening condition, "a target machine and plan that keep
   6 or more independent tables live", is now the plan's shape. What that makes possible for
   the dense table is small and bounded: ten plain workers on level 9 give 619–631 MB/s (SN
   3b), and 631 × 1.117 = 0.70 GB/s (inferred, the ratio is 0.1.4-era) against about 1.5. So
   the condition is met in letter and the dense form still loses. The mechanism E1 validated,
   fewer reads past L2 a byte, is what N1 carries into a small table. Cheapest falsifier: none
   needed. Worth: nil for the 3.6 MB table; it settles that P01's survivor is N1.

7. **P01 and P03, "half the steps, each twice the work" is a fact about one skeleton.** PWK:
   "The walk takes half the steps, but each costs about twice as much", measured in the 0.1.4
   flagged walk. The same entry: "libsais walks a ψ² table in ≈6.7 c/B, against our port's ≈10
   ... Those mechanics may transfer to our ψ walk without the pair table". They did transfer,
   to the single table, as the room walk. No pair consumer was ever put in that skeleton: #272
   used "#34's consumer", and PFR names C37 "staged pair rooms" as a target that "would need a
   reduction of more than 34.4%". libsais's 6.7 against 10 is a 33% reduction (inferred), on
   another machine and date. Cheapest falsifier: a counting probe of a pair step in the room
   skeleton. Worth at one thread, by PFR's joint frontier: with a matched consumer the builder
   "would have to fall to 1.90 or below" against 5.668, so no win. At ten: bounded by finding
   6. It matters only as the step price for N1 at k = 2.

8. **P11 and E8, a reopening condition declared met and never run.** WSW: "The doubling above
   is that condition, measured ... otherwise it closes for good." REG: E8 "never run [coord]",
   and "measure it at 8–10 workers, not only 1–4". SN 3b now gives the regime's prize and its
   gate. Read against it, #42's own figure passes: +90% to +108% is g 1.90–2.08, with a table
   of 574–742 kB. But the baseline moved: the room walk was 16 instructions a step then and is
   10.41 a row now, and the move step is 41 by listing, over 38.3 before its build (inferred).
   PHR close 2 agrees: the same lanes with no copies "take 9.3–9.8 ns a byte" against
   production's 1.7–1.9. The two entries contradict on the ratio (1.46–1.85× in MSW, about 5×
   in PHR); the baselines and skeletons differ. Cheapest falsifier: a counting probe of the
   move step in today's skeleton. Worth: likely nil as a one-byte table; the count would close
   E8 without a workstation run.

9. **P15, an impossibility that was not one.** REG and CPF: blocked rank is "ruled out for the
   ψ walk (the walk needs select, not rank)", class I. BWT section 1: "mibz2 buffers a whole
   block before RLE1, so the order the walk emits in costs nothing", checked on 431 blocks. The
   structure was then built as design E and failed as a construction: 3.665× (DER). DER's
   reopening line, "no more than 1.44× the room walk's instructions a row (about 15)", itself
   quotes the gate SN 3b replaced. At 2.5, E in the room skeleton is (32 + 4.06) ÷ 15.33 =
   2.35 by instruction count (inferred), a tie at best, and its ten tables are "about 18 MB",
   past the knee. Cheapest falsifier: none worth running. Worth: about nil; the gain is that
   every other class I entry resting on "the walk needs select" should be re-read.

10. **P16 and P14, the condition was met and the cost model failed.** REG's condition for P16:
    "a representation in classes 1–2 … below 24 bits". SR section 6 ranks designs by
    dependent-load depth and holds that "instructions are cheap next to misses". E is 2.039
    bytes a row, 16.3 bits (inferred), with "98.18%" of Wikidata's steps from one access, and
    its inversion costs 3.67×; "The walk is instruction-bound" (DER). So access class does not
    predict cost on CI at one thread. Under share-nothing the regime at ten is the regime at
    one: level 1's CPU-seconds go 0.66 → 0.84 from 1 to 10 workers (SN 3b). Cheapest
    falsifier: for any candidate, a counting probe of the whole loop first. Worth: a screening
    rule, not a speed-up: cannot be bounded from the record.

11. **"Text" means three corpora, and the k-step verdict on text depends on which.** BDC 4c:
    "r₂ 0.449 n, r₄ 0.626 n, r₈ 0.797 n ... D is a lever for data with long runs. On text it is
    not", on a text with 0.300 runs a row. COMP 3.1 h's text control has r/n 0.151 (inferred
    from r 135,801 of 899,000) and m₈ 352,169, 0.392 n (inferred). RF8's text has r/n 0.166 and
    r₂/n 0.245; PKT's has 0.2337. All are synthetic or "a property of the checkout" (PKT). At
    COMP's count a text P⁴ is 277,841 × 8 B + 225 KB = 2.4 MB (inferred), under the T-vector's
    3.6; at BDC's it is 4.6 B/row, over it. Cheapest falsifier: a census (`phrase_census`) on a
    pinned real text input. Worth: it decides whether text blocks take the k-step table or the
    fallback; cannot be bounded from the record.

12. **P04, the copy family has no i9 reading, and its recorded reopening bar is a count.** PFC:
    "The steps a forward chain skips are the cheap ones ... which are cached", from CI at 1 and
    4 threads. REG: coop section 5c asked for `perf/precopy-chains` "at 8–10 workers on the
    workstation"; "No record of that run." PHR close 5 withdrew `bench-phrase-knee` "before any
    run", on local evidence from a VM whose "L3 is 260 MiB. So it says nothing about the i9's
    knee" (PHR section 4). At ten 3.6 MB tables the L3 hit share is 0.539 (KM E2), so "cached"
    does not hold there. But the copy walk keeps a 3.6 MB state stream, so it does not serve a
    small-table plan. Cheapest falsifier: PHR's own, by count: "a copy event at 40 instructions
    and one misprediction or fewer". Worth: cannot be bounded from the record.

#### Changed since 30 October

- **P01:** BC nogo (premise reversed, rejection kept); LFA (verdict "Closed", two reopening
  conditions; libsais equal alone, 0.989 at one block); RF8 (r₂/r 1.22); PFR (#272: R/P 1.895,
  NEGATIVE STOP; run build 0.6865 of the row build, 2.563× the one-hop build); PKT; BDC 4c;
  SN (the regime).
- **P02:** BWT section 3 and BDC 4c, 4e (dense tables sized and counted out).
- **P03:** status rejected → conditional, by the rule that a design closes only when its
  advantage is disproved. LFA lists it as untried; PFR names C37 and C38.
- **P04:** BC nogo; PKT (tandem ceilings 6.1% and 10.0%, "Not a cap on Precopy-style
  copying"); PHR (phrase census, exact copy walk, closed 10 November).
- **P05:** PHR (realised without the per-step test, and still slower).
- **P06:** PKT (certified packets 3.5 × 10⁻⁶); PHR section 3 (tube packets); COMP T4.
- **P07:** PHR (crossover "n/r of about 80"; heavy-tailed phrases).
- **P09:** SC and SN (the substitute's cost counted); COMP 3.1 g2 and RF8 (capped records).
- **P11:** BC nogo; LFA; RF8 line B; PHR close 2; BWT section 4 (design C); SN 3a (C
  withdrawn) and 3b (the gate).
- **P12:** RF8 line A; RFS; RFD; BWT design A; BDC section 2.
- **P13:** LFA (re-derived, with an untimed falsifier and thresholds); SN (the regime).
- **P14, P15:** BWT section 1; DEG; DER (rank in the walk built as design E and falsified).
- **P16:** DEG and DER (its condition met in class and size, lost on instructions).
- **N1:** RF8; PKT; PHR; BWT sections 3, 4a; BDC 4c; SN.
- **No change:** P08, P10.

#### Reopen candidates

Rejected or superseded IDs whose rejection was of a construction only.

- **P01 (dense pair table).** At one thread: a build "at no more than today's one-pass build
  plus 1 c/B" (LFA), or, with a consumer matching production's residual, a builder at "1.90 or
  below" nominal c/B against 5.668 (PFR). At ten on the i9 it is bounded near 0.70 GB/s
  (finding 6). Reopen only as N1.
- **P03 (E6 and C37).** A pair consumer in the room skeleton with "a reduction of more than
  34.4%" of the pair residual (PFR).
- **P04, P05 (copying).** "a copy event at 40 instructions and one misprediction or fewer",
  or "an input that matters with n/r of 80 or more" (PHR). For share-nothing it also needs a
  state stream well under 3.6 MB a block.
- **P11 (move structure).** Step plus build at or under 38.3 instructions a row, or 30.7 at
  "twice" (inferred), counted in today's skeleton. As listed it is 41 before the build.
- **P14, P15 (rank over L, design E).** DER: a step and skeleton "at no more than 1.44× the
  room walk's instructions a row (about 15)". By SN 3b the bar is the whole inversion at or
  under 38.3 with ten tables under 14.4 MB; E's 1.8 MB table misses the second part.
- **Line B and D at k = 2 (inside N1).** Stopped on a parity gate and a 1.44× gate. At 2.5
  they need a walk plus build at or under 38.3 a row; the models give about 25.6 (inferred).
- **Not candidates:** P02 (its table grows), P08, P09, P10 (format), P07 (advantage disproved
  on this corpus), P06 and P12 (superseded by realisations that are open as N1 and design A).

#### Notes on the reading

- **Not located as primary text:** #6's feasibility figures, the H1 cheap test, #9's offline
  census and coop section 5c (`2026-10-28-cooperative-inversion.md` is not in `docs/journal/`
  on this checkout). Their figures are quoted from REG only.
- **A contradiction in the yardstick:** production's build is 4.92 instructions a row in DEG
  section 4 and 6.2 a decoded byte in SN section 1. Rows and decoded bytes are both 209.7 M.
- **Regimes never run, by ID:** P02, P04, P11, P13 and P16 have no i9 reading at any thread
  count. P01 has no reading in the room skeleton. N1 has no timing or count on any machine.
