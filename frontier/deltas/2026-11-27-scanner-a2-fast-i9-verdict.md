claim: scanner-a2-fast-i9-run
by: antigravity
outcome: done
evidence: origin/evidence 9dacc085 (scanner-a2-fast/run-20261127T233936Z.tar.gz, sha256 c42ecb37705312cdc7715b9eff3b8872dc3bbe2dd955936b4d4420c61aed212b); docs/notebook/2026-11-27-scanner-a2-fast-i9-verdict.md; docs/journal/2026-11-27-scanner-a2-fast/registration.md

**What was asked:** Read the physical i9-10900K adoption bundle for PR #1117 (`scanner-a2-fast-build`: direct selector runs, multi-length code-length LUT, single-write table build in `src/block.rs` and `src/huff.rs`) against its frozen registration (`docs/journal/2026-11-27-scanner-a2-fast/registration.md`), check its SHA-256 and correctness invariants, and record the verdict.
**What I found (`PASS`):**
- **Integrity & codegen (§2):** `origin/evidence` commit `9dacc085` (`scanner-a2-fast/run-20261127T233936Z.tar.gz`) matches SHA-256 `c42ecb37705312cdc7715b9eff3b8872dc3bbe2dd955936b4d4420c61aed212b` bit-for-bit. All 28 parity runs (`parity.jsonl`, 7 corpora × 2 arms × `k=1, 10`, `bmi2=true`) match `libbz2` SHA-256; all 300 timed samples (`runs.jsonl`) pass byte/block integrity; both `bmi2-base.asm` and `bmi2-head.asm` preserve exact `22 shlx`, `8 shrx`, `0 vzeroupper`.
- **Gate 1 (`S'` cut ≥ 3.0% on `wikidata ×4`):** **PASS** — `phases/wikidata/1` drops `142.774 -> 136.306 ms` (`-6.47 ms`), ratio **`0.9548` (`-4.52%`)**, 95% CI **`[0.9514, 0.9582]`** (`<= 0.970`), **disjoint** 10-round ranges (`head [135.63, 137.24] ms < base [142.42, 143.42] ms`).
- **Gate 2 (Guard serial scan, 5 corpora):** **PASS** — `text 0.9968 [0.9736, 1.0206]`, `records 1.0044 [0.9675, 1.0428]`, `random 0.9996 [0.9656, 1.0348]`, `runs 0.9560 [0.8975, 1.0183]`, `onebyte 0.9924 [0.9252, 1.0645]`.
- **Gate 3 (Peak RSS ≤ 1.02×):** **PASS** — `1.0006x..1.0068x` across all 9 real cells.
- **Gate 4 (No whole-decoder loss on `wikidata ×4`):** **PASS** — `k=1` `0.9930 [0.9825, 1.0036]`, `k=8` `0.9999 [0.9952, 1.0047]`, `k=9` `1.0041 [0.9823, 1.0264]`, `k=10` `1.0058 [0.9848, 1.0273]`. Diagnostic off-wikidata 8W walls: `text 0.9967`, `records 1.0076`, `random 1.0041`, `runs 1.0077`, `onebyte 1.0288` (overlapping 12 ms single-block ranges).
**What changed:** Added `docs/notebook/2026-11-27-scanner-a2-fast-i9-verdict.md` and this delta, updated `frontier/surface.tsv` row `S` with the physical i9 `PASS` figures, removed the read `bench-scanner-a2-fast` line from `bench/workstation-queue.txt`, and closed `frontier/claims/scanner-a2-fast-i9-run.toml` (`status = "done"`).
**What's next:** Independently review and integrate PR #1117 (`scanner-a2-fast-build`) on exact-head CI.
