claim: scanner-a2-fast-i9-run
by: claude-code
outcome: done
evidence: origin/evidence 9dacc085 (scanner-a2-fast/run-20261127T233936Z.tar.gz, sha256 c42ecb37705312cdc7715b9eff3b8872dc3bbe2dd955936b4d4420c61aed212b); registration docs/journal/2026-11-27-scanner-a2-fast/registration.md at 40a16165; PR #1252

**What was asked:** independently verify #1252's PASS verdict for the #1117 adoption run against the bundle and the frozen rule, then integrate it.
**What I found (PASS confirmed, independently recomputed):**
- **Integrity:** the tarball from `9dacc085` hashes to `c42ecb37…d212b`, matching its sidecar. The bundle's `registration.md` is byte-identical to the one frozen at `40a16165` (its only commit), and `bench/scanner_a2_fast.py` is unchanged since then. `provenance.json` names recipe `40a16165`, head `689e55f7`, base `2af67a67`, rustc 1.99.1, seed 1117.
- **Byte parity:** `measure(..., check=True)` (bench/scanner_coalesced.py) raises VOID unless each decode's streamed SHA-256 equals the independent libbz2 reference; all 28 parity rows ran (7 inputs x k=1,10 x 2 arms, bmi2 true) and the outcome is PASS, not VOID. All 300 timed samples have the registered output byte count. Both arms' disassembly: 22 shlx, 8 shrx, 0 vzeroupper.
- **Decision rule §4, from `runs.jsonl` with my own paired log-ratio t-interval (n = 10):** every figure in #1252 reproduces to four places.
  - Rule 1: `phases/wikidata/1` 0.9548 [0.9514, 0.9582], upper <= 0.970, ranges disjoint (head max 137.24 < base min 142.42 ms). PASS.
  - Rule 2: guard scans 0.9560..1.0044, none above 1.020 with lower bound above 1. PASS.
  - Rule 3: median peak RSS ratio 1.0006..1.0068 at all nine real cells. PASS.
  - Rule 4: `real/wikidata` k=1/8/9/10 0.9930..1.0058, none above 1.020. PASS.
**What changed:** added this delta to #1252 and merged master into it (surface row S keeps master's two-lane interleave and #1246 notes beside the A2 verdict; no other row touched), then merged #1252.
**What's next:** #1117 (`scanner-a2-fast-build`) is cleared on its i9 gate; its integration is the next step.
**Caveats:** two diagnostic, ungated cells lean slower beyond noise: `real/records/8` 1.0076 [1.0016, 1.0137] and `real/onebyte/8` 1.0288 [1.0038, 1.0544] (a ~12 ms single-block decode). Neither is a frozen gate (rule 4 covers Wikidata only) and C2 is not relaxed either way; they are worth watching in the release's i9 gate. At k = 8 Wikidata stays worker-bound (0.9999), so the -6.5 ms scanner cut is headroom, not wall, until worker-side cuts land.
