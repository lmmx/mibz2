claim: scanner-a2-fast-i9-run
by: claude-code
outcome: review
evidence: branch bench/scanner-a2-fast; head 689e55f7 (origin build/scanner-a2-fast-head) = #1117 merged with master 2af67a67; local smoke run of both arms (not timed)

**What was asked:** the i9 recipe for gate 2 of `scanner-a2-fast-build`, run before #1117 merges.
**What I found:** #1117 merges into current master cleanly except `frontier/surface.tsv` (master's row S taken, the new claim linked); on the merged tree `cargo test --release --lib` 208 passed and `tests/lazy_admission.rs` 3 passed. A local smoke run builds both arms with the recipe's own `build()`, decodes all six corpora byte-exact against libbz2 at 1 and 10 workers in both, and runs the pinned phases sample on every corpus (wikidata x4 S' 0.255 -> 0.241 s, one sample, direction only). `runs` and `onebyte` are single blocks scanned in 0.03-0.3 ms with 8-50% spread, so a point-estimate 2% guard would stop on noise: the guard is "ratio > 1.020 with lower bound > 1", fixed before any i9 data and justified in the registration.
**What changed:** `bench/scanner_a2_fast.py` (reuses #1034's decode, hash and placement checks), `bench/test_scanner_a2_fast.py` (7 tests), `just bench-scanner-a2-fast`, registration `docs/journal/2026-11-27-scanner-a2-fast/registration.md` (15 cells, 10 paired rounds, 300 processes; S' upper bound <= 0.970 with disjoint ranges, guards, RSS <= 1.02, no Wikidata wall loss beyond noise).
**What's next:** one review of the registration by an agent other than claude-code (C6); then the ready tag on the merged recipe commit and a queue entry (branch master), and the bundle read. On PASS an agent other than claude-code integrates #1117 (`take-review` refuses claude-code, its producer's kind).
**Caveats:** local toolchain is not 1.99.1; the i9 run is the first timed data.
