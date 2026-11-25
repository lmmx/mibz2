# Critique of the fresh Neon chat (`research/neon-movement`, c71d0a22) against ESSENCE and the referee

*An independent reviewer, read-only. It recomputed block-0 from Neon's JSON and ran one simulation (Cole–Vishkin against random labels).*


Read-only review. Recomputed figures: S1B block-0 Q and D from the JSON (the sums match); M-21 against S1A; S5's probe and stream D from its own JSON; S4's "2.94·m law" with an independent simulation (`nm-critique/cv.py`). No worktree was needed.

## Sense × verdict

| Sense | New sense? | Q (acq. incl.) | D | P | Verdict |
|---|---|---|---|---|---|
| S1A raw-LF landmarks | No. This is M-21 / record 114 Sense E. | 1.08 n (11–31·h₁) | 15–17k | 2 (probe) | It reproduces our M-21 exactly. |
| S1B Open + landmarks on R | No. Row-axis Open, then a sparse ruling set on R. That is list ranking (record 109). For k≥2 the "text" landmark key is a DAG-id hash (`W[i][0]&0xF`), not a text byte. | 5.0–10.3·h₁ (0.34–0.60 n) | **1,093–6,013** | 4–5 | A real D cut, the lesser find. Q never falls below m₁₆ plus Open's acquisition. |
| S2A period-p affine | Partly. It is RV with slope μ, and the image is contiguous only when μ=1. | ≥ walk; saves ≤2.8% h₁ (UB) | — | 2 | Exact but negligible. |
| S2C S_Σ shapes | Relabelling is new framing, but transport fails | — | — | — | The collapse is pigeonhole: 57k words into ≤B₈ = 4,140 shape classes. |
| S3 sort-merge doubling | No. Wyllie pointer jumping in external memory (Chiang et al. 1995), disclosed. | "0" (doubling). On m₁₆ it is Q_acq 2.4m plus 0.70–0.81M DAG expansion, ≈ n. | "0" | 117–145 | Record 108's trap: the cost moved into P≫K, and Neon says so. |
| S4 independent set | No. Miller–Reif / Anderson–Miller / CV list ranking, which is CGGTVV's EM algorithm. The algorithms are cited but the recursion is claimed as new. | "0"; on m₁₆, Q = Q_acq ≥ m (1.14·h₁ on hex) | "0" | ≈600 sweeps | Classic. The "law" is not a property of CV (below). |
| S5 ∂_thick wave | No. M-21 stream wave with Y = thick exits (records 104 and 113 §3, M-17 sweeps), disclosed. | stream 1.09–4.24·h₁; probe 225–374k | **15–44k** (own JSON), not the headline 48–86 | 51–89 / 3 | Uncharged scatter (below). |
| S6 IFS residual | No. FM-index sampled rank / PGM residuals, disclosed. | n | n | 2 | State compression only (20→3.4 bits). |

**Enrichment:** none. The only quantity carried along a step is S3/S4's `dist`/`pos` weight, which is the text position. ESSENCE names that as what "restates the inversion". S6's ε is stored state, not something that rides along a step.

## Strongest finding

S1B, Open's thick-strip contraction followed by first-return sections on the retained map, is the one real result. It is also new as a measurement: our M-21 measured raw LF only. On block-0, D = 10 (strip rounds) + 399 + 77 + 35 (level gap maxima) + 572 (|Y₃|) = 1,093. Q = 210,406 (Open's acquisition) + 88,290 + 5,414 + 866 + 572 = 305,548 = 8.70·h₁.

Against Open's walk (D = m₁₆ = 88–202k), D falls 37–81×. Against our raw M-21 at k=3 (15–17k), it falls 2.5–15×. Q falls from M-21's 1.08 n to 0.34–0.60 n, still 5–10·h₁. It is a D find at P=4–5. It is not a Q find: record 109 applies, because every row of M is stepped once at level 1.

## Discrepancies

1. **S5 headline vs JSON.** The README says D collapses "from 900,000 to 48–86". That figure is the thin-wave depth alone. Gluing the excursions is a chain of K_exc lookups, and Neon's own `cost_probe_wave`/`cost_stream_wave` give D = 14,865–44,055.
2. **S5 "Q_thin = 0" is not free.** `exc_words[eid].append(b)` scatters every thin row's byte to its excursion: m addressed stores, uncharged. Doing it pointer-free needs a sort of the (eid, t, b) triples, which adds passes. P_bytes also leaves out the n-entry `is_thin` bitmap probed every round (≈0.125 n bytes × 51–89 rounds).
3. **S4's "universal 2.94–2.97·m law".** Σ|V_k| ≈ 1/(1−ρ), and local minima of random labels have density 1/3, which gives 3.00·m on any input. My simulation:

   | Rule | Random cycle | Monotone cycle |
   |---|---|---|
   | Random labels | 3.00·m | 3.01·m |
   | CV | 2.94·m | 2.03·m |

   So 2.94 reflects that CV colours on these permutations behave like random labels. It is neither universal nor a worst-case guarantee: one-round CV with 2·log m colours bounds the independent-set density only by about 1/(4 log m).
4. **Inconsistent accounting.** On hex, m₁₆ = n with no thick runs, yet S1B/S4 charge Q_acq = 65,536, while the raw-LF path for the same R charges 0. PRIOR-ART says S4 is "~1.5 m" where S4.md says 2.94 m. PRIOR-ART says "m ≈ h₁" where m₁₆ = 2.5–5.8·h₁.
5. **S3/S4 "Q=0" headlines** leave out acquisition (2.4·m₁₆ on block-0) and S3's DAG expansion (0.70–0.81M). Literal Q=0 holds only on raw LF, at P = 145–600.

## Checks

- **M-21 vs S1A agree exactly** on block-0: |Y₃| = 15,844, D = 16,135, Q = 1,082,072 (30.81·h₁), pair `":` 41,219 (gap 21.8/129/286). The other blocks match too. Neon's synth-tmpl row puts a k=2 value in the k=3 column.
- **"Byte-exact 8/8".** The synthetic blocks are checked against the true T, so that check is independent. The admitted blocks are checked against a textbook LF walk in `common.py`, which shares only `compute_counts_and_runs` with the methods under test, and it asserts a single n-cycle. That is adequate, but there is no external text hash: the bzip2 block CRC is not used.
- **Hidden LF walks.** `build_retained_system_genuine` composes strips by endpoint only, with each lookup charged. The `lf[]` walks in s1/s2/s6 are probe-mode and audit measurements, charged ≥ n. In s3/s4, `build_lf_array` is one streaming pass with 256 cursors. I found no hidden walk. The uncharged items are the S5 scatter and the bitmap.

## Theorems

- **2A** (LF(s+r+tp) = LF(s+r) + tμ_r): correct and trivial, since it is rank counting over a periodic segment.
- **2C(1)** (Bell-number bound): trivial, a count of equality patterns.
- **2C(2) "Equivariance Obstruction":** an assertion with a heuristic Θ(√(n/σ)) argument, not a proof. Its content, that LF is not S_Σ-equivariant, is obvious.
- **6.1(a)** is a product of slopes, and **6.1(b)** is heuristic.
- **1.7–2.8% of h₁** is an upper bound on one-step savings, against a target of ≥50%. It is not meaningful.

## Most promising thread

S1B as a D result. Three things would close or test its gap:

1. **Charge glue consistently.** The retained-row glue can be streamed, which leaves Q_acq ≈ Q_strips, about 1.4m on block-0.
2. **Hybrid stream-plus-probe for level 1.** Stream level 1 for the first few rounds (90% of gaps are short; p99 ≪ max) and probe the tail. That trades P for Q at P ≤ K + 2.
3. **Report it against Open, not n.** Being honest, Q < h₁ is out of reach in this family (record 109), so this is a latency (D) result for MONODROMY 114 §5's referee question, not a breakthrough. No sense here meets ESSENCE's breakthrough bar, and none applies enrichment.
