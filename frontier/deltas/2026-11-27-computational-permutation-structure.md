claim: computational-permutation-structure
by: claude-code
outcome: done
evidence: registration deb5401 (before data); examples/permutation_operator_probe.rs; docs/journal/2026-11-24-algorithm-level/computational-permutation-structure/{registration,results}.md and data/

**What was asked:** work backwards from the decoder's computation to a permutation operator that pays (>= 39.18 ms C+E against the strongest matched forward control), or a scoped obstruction and a sharper missing primitive.

**What I found:** two operators, frozen before data, and both STOP. Every arm was byte-identical to `bwt::walk` on 233/233 admitted wikidata blocks (1 and 4 lanes, both builds).
- **O1, labelled powers `J_k = (p^k, W_k)` as interval maps.** Pieces are the runs of the preceding-k-mer column (`<= min(n, k r)`, checked by recurrence).
  - On raw LF they grow far below the bound (`P_k/n` 0.064, 0.078, 0.096, 0.126, 0.175, 0.250 for k = 1, 2, 4, 8, 16, 32). At visit parity with Open (`k >= 9`) the piece table is about 5x Open's bytes: 0/117 train blocks fit.
  - On Open's `T`, `P_k(T)/m` is 0.552 to 0.736 for k = 1 to 8: pieces average 1.8 rows. Structurally excluded, so not timed.
- **O2, word-capped labelled doubling of `T` by independent gathers.** This is the post-Open pass that `mat_f3b_post_open`'s dependent chase did not test.
  - Visits fall to 0.555x, 0.342x and 0.246x of `m` for r = 1, 2, 3.
  - Each round costs 4.9-5.8 ns per row, about one visit of the matched 4-lane walk (4.9 ns). Held-out `C+E4` is 152.6 to 325.9 ms against `ctrl4` 66.5 ms (baseline), and 147.8 to 298.5 against 62.2 ms (native), disjoint at every r.
  - Even J_3's 24 ms consumer gain is 12x below its construction cost.
- **Obstruction (scoped to wikidata, LIMIT=16):** composition costs about the walk it would shorten. A contraction constructor pays only if it is sublinear in `m`, or fused into a pass that already touches every row group (A6a, A6b), and only if it yields shared words.
- **Missing primitive:** frequent code digrams `(w(x), w(Tx))` with their row sets as O(1) intervals, at `O(B + #digrams)` cost. Policriti-Prezza 2018, Kempa 2019 and the lex-parse were checked; none supplies it.

**What changed:** the probe example, the registration, results and raw outputs; two `dead-ends.tsv` rows; the surface row `E-walk`; the claim marked done. No new claim.

**What's next:** `bwt-digram-return-map` also STOPped while this was open (contraction 108-113 ms; 76.5% of thin runs are w=1), consistent with the obstruction. The one narrower question left: does A6b's band trace see `(band, image band)` pairs contiguous enough to share a word at O(B) cost? No new claim. The obstruction is falsified by `P_1(T)/m <= 0.25` on another corpus or limit, or by a gather round costing <= 0.15 of a 4-lane visit. `census` and `bench` measure both in under a minute.

**Caveats:**
- Measured on a 2 MiB-L2 VM, not #992's machine or the i9, and warm only; the i9 numbers are ratio projections.
- This harness's `ctrl4` is weaker than #992's, which favours O2.
- The reference `T` is built offline from LF; Open's own construction cost is not re-measured.
