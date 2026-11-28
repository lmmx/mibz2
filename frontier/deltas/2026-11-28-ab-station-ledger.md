claim: ab-station-ledger
by: antigravity
outcome: done
evidence: origin/evidence f8b21e4c (recalibrate-0518/run-20261128T100510Z, sha256 51fb951b...97f2), 6e5d4cbb (crossover/run-20261128T130104Z, sha256 f05627ad...0c16), 16d5e409 (release-scaling/run-20261128T125005Z, sha256 c6c46332...7778)

**What was asked:** add a standalone read-only CLI (`bench/station_ledger.py BUNDLE_JSONL...`) and unit tests (`bench/test_station_ledger.py`) printing per `(corpus, threads, arm)` median wall, mean `S` (`mibz2-scan`), mean `W` (sum of `mibz2-N`), `k`, `W/k`, `W/k - S`, residual `wall - max(S, W/k)`, binding station, and a `> 10%` `W` growth flag across an arm family, skipping and counting empty `thread_cpu` rows.
**What changed:**
- Added `bench/station_ledger.py` (stdlib-only, reads `.jsonl`, directories, and `.tar.gz`/`.tar` bundles while skipping `parity.jsonl` and duplicate `recorded/*.jsonl` copies; supports `--corpus`, `--arm`, `--growth-threshold`, `--json`).
- Added `bench/test_station_ledger.py` (4/4 pytest passing, including the frozen 6-row synthetic JSONL test with 1 empty `thread_cpu` row skipped and counted, arm-family `> 10%` `W` growth flagging, binding edge cases, and tar/CLI checks).
- Zero changes to `examples/ab.rs` or any frozen benchmark recipe.
**Stop test verification:**
- Verified no existing `bench/*.py` prints these columns (`bench/report.py` only splits total CPU shares).
- Ran `bench/station_ledger.py` on `f8b21e4c` (`recalibrate-0518/run-20261128T100510Z.tar.gz`): all 9 Wikidata cells in `docs/notebook/2026-11-28-explore-shipped-knee-worker-bound.md` match to within `0.050 ms` (`<= 0.1 ms` gate); `unpinned.jsonl` skipped and counted all `384` empty `thread_cpu` rows; flagged `W` growth on `wikidata-x4/mibz2-occ` unpinned (`+23.4%`, `k=3->10`) and `text/mibz2-occ` (`+168.4%` pinned, `+211.0%` unpinned).
**Readout on the two newest `thread_cpu` bundles:**
- `6e5d4cbb` (`crossover/run-20261128T130104Z`, single-copy Wikidata 209.7 MB, 400/400 rows used): shipped `mibz2` is worker-bound at every shipped even/unpinned count (`T=4`: `+77.7 ms`, `T=6`: `+29.6 ms`, `T=8` `k=6`: `+18.2 ms`, `T=10` `k=7`: `+11.4 ms`, `T=12..20` `k=8`: `+4.0..+7.9 ms`, `resid +14.8..+23.5 ms`, flat `W` `-17.8%`), reaching a `tie` (`-0.2 ms`) only at untrimmed odd `T=9` (`k=9`). Untrimmed `mibz2-workers-w` crosses to scanner-bound at `T=10..20` (`-2.5..-20.6 ms`) and flags `W_GROWTH(+24.1%)` (`k=4->20`, `345.0->428.0 ms`). Cooperative `mibz2-team2` (`+657.6%`, `311.0->2356.0 ms`) and `mibz2-team3` (`+270.0%`, `246.5->912.0 ms`) flag severe contention/waiting `W` growth.
- `16d5e409` (`release-scaling/run-20261128T125005Z`, v0.5.18 vs v0.5.17 on single-copy Wikidata, 240/240 rows used): both builds are worker-bound across every parallel count `T=2..20` (`k=2..8`): `W/k - S = +135.0 -> +6.6 ms` (v0.5.18) and `+133.0 -> +2.6..+5.4 ms` (v0.5.17), residuals `+14.8..+24.4 ms` at `T=8..20`, zero `W` growth (`-8.1%` / `-8.5%`). `T=1` classifies as `single` and `lbzip2` as `other` (`k=0`).
**What's next:** use `python3 bench/station_ledger.py <bundle.tar.gz>` during future bundle readbacks to check binding station and `W` inflation before pricing scanner vs worker levers.
**Caveats:** `thread_cpu` is quantised to 10 ms ticks and counts CPU time (including spin/syscall waiting), not traced busy time; `W` growth flagging distinguishes waiting/contention from work demand only across multiple `k` within an arm family.
