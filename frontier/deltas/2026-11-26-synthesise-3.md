claim: synthesise
by: claude-code
outcome: done
evidence: docs/journal/2026-11-26-synthesis/third-pass.md; origin/evidence c71c4a56 (pooled-scan runs.jsonl, SHA-256 97e397ad…8867, checked; 96 traces re-aggregated); pooled-scan/results/report.txt; floor/README.md (cc8ba137); delivery/README.md (#966); 2026-11-26-explore/cold-store-and-rle1-llc.md (#952); src/rle.rs:103-170; src/python.rs:121-139

**What was asked:** Connect the deltas since #965 into what none says alone; check dead-ends against the newest results; open the implied claims at idea. No hold.
**Deltas read:** delivery-copyout (#966, missed by both earlier passes), floor-and-pooled-scan-publish (#972), bottlenecks-floor-correction-outline/-publish (#969, #975), smooth-striated-trajectories (#970), lexicon-and-cap-clarification (#971), tooling-review-discoverability (#973), event-stigmergy-shadow (#976), harden-frontier-tooling (03994d6), frontier-first-day-feedback (#967), blog-publishing-authority (#964); synthesise and synthesise-2 before them.
**What I found:**
- **The fast-worker R + D cliff is the C oracle's cold store, the #952 mechanism again.** At k = 10 the S arm (real C, every worker busy) and SC share a 16.3 MiB ready high-water mark, but A9 is 127.7 against 213.4 ms and copy_out 47.8 against 76.7. S keeps scaling (184.9, 174.5, 169.6 ms at 8, 9, 10); only SC scales backwards. #954's relation 1 ("faster workers give back gains at 8-10") was read off SC/PC.
- **Real R inflation is small:** none's A9 rises 88.4 → 102.8 → 131.3 ms (k 6, 8, 10). All of it removed would be 0.9% of the wall at 8 and 2.2% at 10, partly oversubscription (the scanner's own scan rises 7% from 8 to 10). `delivery-l3-working-set`'s gates (2.0% shipped, 4.0% fast) sit at or above that whole bound (2.5% in S at 10).
- **The ready queue is mostly runts:** time-average 0.3-4.3 MiB, 4-17x below its high-water mark; 4.4-16.8 blocks on average, so 70-260 KB a block; full blocks drain in 0.1-0.8 ms. The per-worker live set (≈1.0-1.6 MiB, from oracle store sizes, derived) is the larger L3 term. That makes the fold, forward emission and a fused RLE1 the L3 levers, not READY_BYTES.
- **The pooled scan conserves work:** P(k) = S(k) + scan/k and PC = SC + scan/k, within 1.5% at k = 6-9. K1's capture is then 1 - (scan/k)/(C - SC) = 0.54 at 8 (measured 0.52). The built scan revives only if S(8) ≤ ≈158 ms; the hybrid's ceiling (k·S + scan)/(k+1) is 184.1 ms at 8 today (4.6% under none), ≈155-163 ms after the fold and forward emission (second pass's 152-161 ms). mtf-fold-build's S arm will give the number without a pooled run.
- **RLE1's four-pass route runs on 13.7% of blocks** (#966: 59/431 with shift > 0). Dropping the shift-up pass alone is < 9% of A9; `single-pass-rle1-crc`'s 35% gate needs find_runs, compaction and CRC fused into one tiled pass.
- **Orphaned step:** #966's Python double copy (src/python.rs:121-139, zero + copy + copy) had no claim.
- **Dead ends:** row 7 (no third station at 8-10) is reinforced for real code, not revived; row 6 (pooled scan KILL) stands, with its revival condition now a number. C2 intact everywhere.
**What changed:**
- Added `docs/journal/2026-11-26-synthesis/third-pass.md` (tables and derivations).
- New claims at idea: `worker-l3-footprint` (probe, smooth), `python-read-single-copy` (build, striated, `src/python.rs`).
- Premise notes (questions only; stop tests untouched) on `delivery-l3-working-set`, `warm-paired-oracle-k1`, `single-pass-rle1-crc`, `pooled-scan-hybrid`; third-pass added to their premises.
- `surface.tsv` rows R and D: the 10W figures split into real code and oracle arms.
**What's next:** take `worker-l3-footprint` before `delivery-l3-working-set` (it can propose dropping it). Read `mtf-fold-build`'s S arm into P′ = S′ + scan/8 before anyone registers a pooled re-test or the hybrid. `warm-paired-oracle-k1` should cite S, not SC.
**Caveats:** Traced cells are n = 3, steady-phase sums, within 1-2 ms of #952's figures. The per-worker live set is derived from store sizes, not measured. Oversubscription and L3 are not separated cleanly at 9-10 workers. Nothing here is a gate result.
