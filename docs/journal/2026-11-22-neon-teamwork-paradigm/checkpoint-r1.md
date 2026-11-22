# Round 1 Checkpoint — `research/neon-teamwork-paradigm`

*Date:* 2026-11-22 (UTC)  
*Base SHA:* `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`origin/master`, `mibz2` 0.5.11)  
*Status:* WIP checkpoint after Round 1 independent exploration; Round 2 adversarial audit, cross-pollination, and native safe-Rust A/B + A/A benchmarks against `OpenCells` are in progress.

## Summary of Round 1 Independent Formulations (Pre-Audit)

1. **Investigator Alpha — `Lazard-BWT` (Lazard Free-Monoid Quotient Contraction):**
   - BWT–Lazard Commutation Theorem (`BWT(Phi_x(T)) = Splice_x(BWT(T))` at `min(Sigma)` / `max(Sigma)`), monotone self-loop queue resolution, strict `r_t <= r` run non-increase, exact primary-mark transport, and bidirectional independent-set wavefronts.
   - Verified on `26,439` small cases + all 6 authenticated Wikidata blocks (`D = 450–883` rounds, `tot_syms = 0.76r–1.04r`, `94–96%` in-place single-run overwrites).
   - Round 2 active: native safe-Rust A/B + A/A (1T & 4W) of flat-slab `Lazard-BWT` vs `OpenCells` (0.5.11), `Q = 0` radix-streaming / external-memory formulation (total bytes streamed & passes), and prior-art check.

2. **Investigator Beta — `BDMR` (BWT-Direct Monotone Recompression):**
   - Bucket-Split Bipartite Pair Contraction (any 2-coloring `Sigma = Sigma_L U Sigma_R` with zero row reordering via contiguous preimage bucket splitting) + 1-round exact monotone self-run contraction (closed-form spiral division).
   - Provable `n_{t+1} <= 3/4 n_t` (`25–26` greedy rounds / `42–44` parallel `O(1)`-depth rounds), balanced SLP of height `25–26` and size `0.72r–0.88r`, `\sum r_t = 4.06–4.34 r_0`, `max_r / r_0 = 1.00x` on all 6 Wikidata blocks (`5,227` exhaustive small cases + 6 torture families + 6 blocks verified).
   - Round 2 active: native safe-Rust A/B + A/A (1T & 4W) vs `OpenCells` (0.5.11), `Q = 0` external-memory streaming accounting, `check_manifest.py` manifest, and prior-art check.

3. **Investigator Gamma — `Phi_BWT` Model-Independent Universality Suite + `MF-BSE`:**
   - **Part I:** Exact `Phi_BWT` / `Phi_BWT_bin` and Hairpin-Closed 2-Party Split `Phi_BWT^{(2)}` constructions (`2,830` verified cases). Proves (Thm 1) `L`-completeness under uniform `TC^0` reductions (even binary, any `r/n`), (Thm 2) unconditional read-only multi-pass streaming lower bound `S >= Omega(r / (L * p^4))` bits for arbitrary bit-level/algebraic decoders (breaking `A_sem`), and (Thm 3) exact Krylov `x^m - 1` obstruction.
   - **Part II:** Multi-Frontal Bucket-Order Schur Elimination (`MF-BSE`) — zero-fill `K`-bucket block Gaussian elimination (`<= m - 1` shortcuts, `Q = 0` random misses outside fast memory `Z` when `m <= c Z^2/B`; `24,568` exhaustive small cases + 6 blocks verified).
   - Round 2 active: native safe-Rust A/B + A/A (1T & 4W) vs `OpenCells` (0.5.11) replacing C, explicit model-boundary classification (read-only streaming vs read-write/external-memory streaming) against Alpha & Beta, and `Phi_BWT` stress test of `BDMR`.

4. **Adversarial Mathematical Referee:** Independently auditing all three proposals, proof steps (including Gamma Thm 1–2 assumptions & coupling, Alpha Thm 2 non-BWT edge cases, Beta Thm 1–2 edge cases & worst-case `r_t`), acquisition manifests, and prior-art boundaries.
