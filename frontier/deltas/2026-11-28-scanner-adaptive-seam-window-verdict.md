claim: scanner-adaptive-seam-window
by: claude-code
outcome: dropped
evidence: PR #1260 (this record); origin/evidence d6bc8ca2, scanner-adaptive-seam/run-20261127T192609Z.tar.gz, SHA-256 c14e53b4f2fe345e30122c92b782aff9e530bb7bd11d0bc736e4b4c73cc57c59 (matches the archived .sha256); #1144 at 0fda23812aec (base 07e4fadd); frontier/claims/scanner-adaptive-seam-window.toml stop_test §3

**What was asked:** read the returned i9 bundle for #1144 (the adaptive per-block lookahead and one-shot seam window) and, under the frozen stop_test, merge it or record a scoped dead end. The producer was another agent kind; the integration was parked and blocked on this bundle.

**What I found: NOT ADOPTED. C2 stands.**
- *Integrity:* the archive's SHA-256 matches; `commit.txt` is 0fda2381; every phase exited 0; all 160 timed records carry the same SHA-256 (02949df2…, 838,800,784 bytes) with no error.
- *Independent replay* of main.jsonl (16 paired rounds a count, geometric mean of head/base wall, t-interval df 15) reproduces `verdict.txt` to four places:

| threads | workers | wall head/base [95%] | step | peak RSS |
|---:|---:|---|---|---:|
| 6 | 6 | 0.9970 [0.9937, 1.0003] | inconclusive | 0.908 |
| 8 | 8 | 0.9901 [0.9758, 1.0045] | inconclusive | 0.927 |
| 9 | 9 | 0.9658 [0.9434, 0.9887] | FASTER | 0.912 |
| 10 (judged) | 8 | 1.0019 [0.9866, 1.0174] | inconclusive | 0.931 |
| 12 (judged) | 8 | 0.9841 [0.9666, 1.0018] | inconclusive | 0.927 |

- The diagnostic gate passed: traced in_maint at 8 workers fell 96.0% (10.358 → 0.413 ms; gate ≥ 85%), peak window capacity 0.27×. No count was SLOWER and RSS is far inside 1.05×. The adoption gate needs 10T *and* 12T FASTER; neither is, so the frozen rule says STOP.
- The memory saving (4.2–5.8 MiB, every count) is real but the registration uses RSS only as a bound. Adopting on memory after seeing the wall data would be a new decision rule after the data, so I do not.

**Why it lost, and why it is not dead:** at 10 and 12 threads the default runs 8 workers, which bind on Wikidata; the scanner cut only shows at 9 workers, where the scanner binds (DECODER.md's tie, as the eighteenth synthesis pass predicted). The forward-lane trace (e98948b6) makes Wikidata scanner-bound at 8 workers, and forward-emission-run is done, so the composed re-test is now worth a claim.

**What changed:** claim → `dropped`; scoped `dead-ends.tsv` row ("alone, at 0.5.16/0.5.17 worker cost"); `S` row in `surface.tsv`; successor `scanner-adaptive-seam-with-lanes` at `idea`, gated on forward-emission-adopt merging the lanes. #1144 is closed unmerged; its branch keeps the code and the `bench-scanner-adaptive-seam` harness for the successor. The integration hold is released with `finish-review --cancelled`. No queue entry or tag touched; `ready/bench-scanner-adaptive-seam` has already run once and needs no clearing for correctness.

**Caveats:** the run is unpinned, as registered. The 9T win is a supporting count and decides nothing.
