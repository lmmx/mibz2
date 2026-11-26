claim: warm-paired-oracle-k1
by: antigravity
outcome: done
evidence: docs/journal/2026-11-24-algorithm-level/ideal-parts/warm-paired-oracle-protocol.md; origin/evidence c71c4a56 (SHA-256 97e397ad586a6700c04a874e50dbc5e26ee9fc91912b115111cbe0550ddc8867); cc8ba137 (SHA-256 10df0d895755494ec6f06369071d649674d855ab953201a2b2888845ab265e34); docs/journal/2026-11-26-synthesis/third-pass.md; docs/journal/2026-11-26-explore/cold-store-and-rle1-llc.md

**What was asked:** Write a frozen, reusable paired-oracle K1 registration protocol covering (a) cold-store-free `C`-paired scanner tests, (b) judged vs. diagnostic counts and mandatory trace diagnostics for `S`-paired worker tests anchored to `S` (not `SC`), and (c) strict C2 non-relaxation of prior verdicts.

**What I found:** Direct re-verification of `c71c4a56`'s 96 traced decodes (`runs.jsonl`, all-phase and steady-phase reconciled to 0.1 ms) and store sizes (`oracle_store.C = 115.03 MiB`, 5.75× L3; `S = 47.77 MiB`, 2.39× L3) sharpens both paired-oracle cases:
- **`C`-paired scanner tests:** Whenever `C` is scanner-bound (`job_wait = 50.6–91.8 ms/w` at `k = 8, 9`) and `XC`/`SC` are worker-bound (`job_wait = 0.2–5.0 ms/w`), any worker-side oracle cost `Δ > 0` (`+24.5..+32.7 ms/w` cold `A7` under zero-copy `C`, or `~3–6 ms/w` `hand_C` under copying `C`) adds 0 ms to `C` and 100% to `XC` and `SC`, strictly compressing both linear and log-scale capture (proved for all `C_obs / SC_obs <= e`; in `c71c4a56`, `C/SC ~ 1.33`). Tier 1 eliminates oracle `C` completely: screen `pooled-scan-hybrid` from `S`-paired worker runs via `H_ceil(k) = (k·S'(k) + scan)/(k + 1)` and run only once real worker wins bring `S'(8) <= 165 ms`, testing `H / none_fast` and `H / S_fast` on real warm construction. Tier 2 fallback bans unadjusted `XC/C` vs `sqrt(SC/C)` in favor of regime-symmetric worker-bound gates (`X/S` or `XC/SC`) plus mandatory `job_wait`/`ΔA7` flags.
- **`S`-paired worker tests (anchored to `S`, not `SC`):** `S` keeps construction real and warm (`A7 = 583.9 ms` vs `none` `581.0 ms` at `k = 8`), adds only a symmetric `+0.8–0.9 ms/w` `A5` cost from `S`'s 47.8 MiB store, and scales through `k = 10` (`184.9 -> 174.5 -> 169.6 ms`, `A9 = 129.8/127.7 ms`, `copy_out = 48.7/47.8 ms`, `read_wait = 115.1 ms`). The `215.6 ms` `A9` and `77.9 ms` `copy_out` cliff belongs solely to `SC`. Even so, `k = 1` (`X/none`) and `k = 8` (`XS/S`) stay the primary judged counts because `S` enters 10-core SMT oversubscription at `k = 10` (`11 > 10` cores, `+7–8%` `A6a..A7`, `+22.9 ms` `A9`) and `ready_bytes hwm = 16.31 MiB`. Crucially, `S`-paired primary gates MUST use direct `XS/S <= tau` and NEVER `SC`-anchored capture `ln(S/XS)/ln(S/SC)`, because cold-`C` bloat in `SC` reverses the bias and artificially inflates apparent capture.
- **C2 intact:** `pooled-scan`'s KILL (`c71c4a56`, #942) stands permanently; `mtf-fold-build`'s frozen registration (#960) already complies (`FS/S <= 0.975` at `k = 8`, `SC` reported-only) and is untouched.

**What changed:**
- Added `docs/journal/2026-11-24-algorithm-level/ideal-parts/warm-paired-oracle-protocol.md` (with registration pre-flight checklist).
- Marked `frontier/claims/warm-paired-oracle-k1.toml` `status = "done"`. No `surface.tsv` or `dead-ends.tsv` rows changed (methodology spec only).

**What's next:**
- Per `warm-paired-oracle-k1`'s stop test, a **different agent** must cold-review `warm-paired-oracle-protocol.md` (leaving a review delta) before `forward-emission` or `pooled-scan-hybrid` cites it in a striated adoption registration.
- `mtf-fold-build` results reporting should cite `S` (not `SC`) for `k = 9..10` and include `ready_bytes` time-average alongside hwm plus the `H_ceil(8)` readout.

**Caveats:** Analytical and methodological only — no new workstation runs, no `src/` edits, no production gates changed; thresholds (`165 ms`, `5 ms/w`, `10 ms`) are calibrated to `wikidata x4` on the 10-core / 20 MiB L3 i9-10900K.
