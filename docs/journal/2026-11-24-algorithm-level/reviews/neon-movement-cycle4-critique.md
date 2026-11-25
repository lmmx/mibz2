# Critique of Neon (movement) cycle 4 (36e9dbca..f6f5d863)

*An independent reviewer, read-only. It recomputed the totals from `data/t1_t2_depth2_schedule_costed.tsv`, read the code that builds and charges R² and R⁴, and checked block-0's figures. Attribution is clean.*


*Read-only review. I recomputed the totals from `data/t1_t2_depth2_schedule_costed.tsv` with awk and read the R²/R⁴ build and cost code. I did not rerun the scripts. Attribution on all 7 commits is clean: author and committer are both "Louis Maddox <louismmx@gmail.com>", and no message carries a trailer, model name or session link.*

## Bottom line
No route gets below h₁ once the referee's own columns are added up. The TSV has two columns, "+IET" and "+payload", and each figure adds one of them to links+build, never both. Summed, the totals at k=0 are:

| route | links+build | +IET | +payload (headline) | **all-in** |
|---|---|---|---|---|
| Open walk_R_flat | 2.105–3.271 | 2.205–3.610 | 4.210–6.542 | **4.31–6.88** |
| M17_1sweep_flat | 0.799–1.289 | 0.899–1.628 | 1.561–2.486 | **1.66–2.83** |
| CL1_2sweep_flat | 0.709–1.172 | 0.810–1.511 | 1.382–2.253 | **1.48–2.59** |
| R4_pair_flat (L1 cursor table) | 0.690–1.173 | 0.790–1.512 | 1.216–1.991 | **1.32–2.33** |

The best all-in figure is R⁴ on block-210, at 1.32 h₁. Nothing is at or below h₁/2.

## Item table
| Claim | Verdict in referee terms | New? | Errors |
|---|---|---|---|
| T1: Open's right-end IET replaces CL4's binary-search acquisition, 0.10–0.34 h₁ | **Correct, and the right fix.** It answers cycle 3's main objection. | No. Record 57 and record 120 §1 already say Open's constructor runs at strip scale. Neon cites Open. | IET is charged as steps but adds no pass to P, and it is missing from the headline "with payload" total. |
| T1/T2: link+build 0.69–1.29 h₁ against an Open baseline of 2.1–3.3 h₁ | The arithmetic checks: walk = m₁₆/h₁, and M17/R⁴ build = B_P (+ pair segments), the same convention as cycle 3. The baseline is the right comparator for "composition against no composition": it counts m₁₆ dependent link loads and no dict reads. | No. These are the figures in record 120 §5's table (0.88 / 1.00 / 2.56 / 3.26 on block-0), now measured on 6 blocks. | The headline "1.216–1.991 with payload" leaves out IET; all-in is 1.32–2.33. "−69 to −71% against Open" charges Open one addressed read per link for a 16–34 KB L1 dictionary, which doubles its Q. R⁴'s payload reads hit a 3.5 MB buffer instead. Link against link, the cut is about 63–67%. |
| R⁴ L1-cursor model | 1,908–2,928 concurrent streams, each counted as free between restarts. The R⁴ cursor table holds 4 B per stream, so a stream misses at least once every 16 steps, and no hardware prefetcher tracks about 2k streams. This model is optimistic. Under strict Π, R⁴ loses (2.67–4.18), which Neon reports. | Measured version of record 120 §5 | The 4n = 3.5 MB payload build is charged neither to Q nor as a pass. Neon says so qualitatively. |
| DAG emit 4.2–6.6 h₁ | Correct, and it accepts the cycle-3 critique. | — | — |
| NEEDS M-27 closed (m(S*₁)/m₁₆ = 0.891–0.925 > 0.85 on 6/6) | **Correctly applied** under record 120's pre-registered rule ("above 0.85 on ≥4 blocks closes §3"). I checked block-0: 89,874 − 8,192 = 81,682 = 0.909 m₁₆. Record 120 §4's guess that "4 in 5 of interior rows fall in S*" is refuted: the measured share is 10.8–20.1%. | Yes: new data, and a new mechanism | B_P fragmentation of 4.1–8.7× is a real finding beyond the rule. It explains why the 9% link cut turns into a 15–81% loss. The k=∞ runs (239–5,347 rounds, 0.60–0.64 h₁ links only) break P = O(1). |
| T5: ΔQ = 0; Theorems T5.1 and T5.2 | **ΔQ = 0 is true by construction.** Every rider is streamed, so Q cannot move; the relevant metrics are P and D. The theorems are **narrower than they are presented**. T5.1: flipping L[n−1] shifts C by a per-label **additive constant**, and ranks within a label are untouched. T5.2: relabelling seq_to_unseq is a **bijection** of labels. Record 121's Proposition M already carries exactly such riders: placeholder labels, a count histogram and (label, rank), resolved after the fact by an O(256) prefix sum and one table lookup per record. The theorems therefore rule out *final* LF values before EOB or MTF. They do not rule out cheap equivariant or deferred-offset riders. | Formalises a triviality. The inventory repeats record 122 §4: h₁ from adjacent non-zero indices, r, widths, selectors. | "No … output-order code is determined" is true, but it is record 121's last row, not a consequence of T5.1/T5.2. "Hence the 26% is T4 × T1 masking" is a non sequitur: the theorems say nothing about the four-arm decomposition. |
| T5 inventory: RLE2-only geometry exact 8/8; CAP=127 splits 3.5–10.2% of r; queue −30 to −46%; selector NMI 0.19–0.34 | Measured and useful as P/D leads. `CAP=127` and `open.rs::maximal()` exist on master. Fusing `runs.rs` into Open's Pass 1 removes one worker pass, and that is a P cut. | The CAP-split count and the selector NMI are new. The rest is record 122. | Unmeasured in time. The NMI comes from 4 bit-exact blocks plus 2 recompressed ones. |
| #31: c + max(b, a/k) is misspecified (+13.9% C, +23.6% D at k=10) | The fit arithmetic checks (a = 1.2816, c = 0.029). | Standard (USL, tandem queues) | **Inconsistent:** the doc calls the k=10 miss "before SMT", while its own point 2 says k=10 is the 11th thread. A 2-point fit validated at a third point leaves no room for diagnosis. The USL fit has α < 0, which is unphysical. k* = 6.51 = a/b uses b = Arm A's whole wall at k=10, c included. With b − c the crossover is 7.6. |

## Anything below h₁ total with emission?
Not as charged: the all-in minimum is 1.32 h₁. **One case to scrutinise:** the payload could be inlined in the R⁴ record. 86.8% of 4-word payloads are ≤63 B (TSV note), so the payload would sit in the same line as R4[cur], and emission would add no addressed read. Total Q is then the "+IET" column: **0.79–1.51 h₁, below h₁ on blocks 210, 207 and 85 (0.79, 0.98, 0.98)**. This is Q-only and below h₁ but not near h₁/2. It hides a roughly 3.5 MB record array per block with no locality, a 13% line-crossing tail, the L1-cursor assumption and the uncharged 4n build. With the same inlining, Open is 2.2–3.6 h₁. It is a metric artefact to rule on, not a breakthrough. The referee should decide whether an inlined payload in the record counts as one read.

## Strongest result
NEEDS M-27 closed, together with the B_P-fragmentation mechanism. It is pre-registered, applied correctly, and refutes a stated conjecture with a mechanism. Second: replacing CL4's searches with the IET acquisition charge brings acquisition back to its true scale.

## Overclaims
1. The headline "total, acquisition + emission included" omits IET from the payload figure.
2. "−69 to −71% against Open" charges L1 dictionary reads as addressed reads.
3. "Theorems" T5.1 and T5.2 are presented as an impossibility result; they show only that final values depend globally on the input.
4. ΔQ = 0 is presented as a finding.
5. The four-arm decomposition is presented as following from T5.
6. "Before SMT" contradicts the doc's own k+1 thread count.
7. P remains understated: IET, the payload build and emission are not counted as passes (cycle-3 item 7 recurs).

## Recommendations extracted (#31/#33)
- Don't fit c + max(b, a/k) over k = 1..20. Use the per-round starvation gap Δ_k = S_busy − mean W_busy and the idle fraction directly. Take k̂* = min{k : median Δ_k > 0} with a paired-bootstrap 95% CI. Fit USL piecewise on k = 1..9 only. About 13 interleaved rounds pin k* to ±1 (σ 5.6% at k=10), so the 20 rounds suffice.
- **Measure:** the worker phase split (MTF/RLE2, construction, emission, RLE1/CRC) for the T2/T3/T6 rows. Q-to-seconds exchange rates over the 232 census blocks: emission time against links, n and dictionary bytes; construction time against strips, IET steps and m₁₆. Then screen R⁴'s −65% links against its +3.5 MB build. Busy/idle and LLC/IPC at k = 8..12 to separate SMT from DRAM saturation. Per-block variance.
- **Reason, don't run:** exactness through the independent oracle, the full Q_total/h₁ screen with exchange rates, pipeline min-combination, and the obstructions.
- **From T5:** fuse `runs.rs` front into Open's Pass 1 inside the worker, which removes `maximal()` and the CAP splits at zero scanner cost. Pass the selectors on as a regime hint. Both are P/D measurements still to make.
