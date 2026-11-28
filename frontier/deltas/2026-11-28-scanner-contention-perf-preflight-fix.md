claim: scanner-contention-residual
by: antigravity
outcome: blocked
evidence: origin/evidence aecb640d (scanner-contention/run-20261128T050613Z.tar.gz, sha256 9f79c432...): VOID in 2 s at perf preflight (rc=234, zero build, zero timing data collected); d5658cfb env.json confirms perf_event_paranoid = 0 on i9-10900K

**What was asked:** fix the `perf stat --per-thread` rc=234 preflight failure on `bench-scanner-contention` and re-pin `ready/bench-scanner-contention` without altering the frozen registration or decision rules.
**What I found:** Linux `perf stat` rejects `--per-thread` in workload mode (`-- <cmd>`), requiring `-p`/`-t`/`-a` instead. Because `scanner_coalesced_run` already pins `mibz2-scan` exclusively to physical core 0 (`scanner_cpu = 0`) and `taskset -c 1..9,11..19` excludes both core 0 and its SMT sibling from all other threads, CPU-scoped `perf stat -C <scanner_cpu> -x, -o <csv> -e <events> -- <cmd>` isolates `mibz2-scan` cleanly without `sudo` under the i9's `perf_event_paranoid = 0`.
**What changed:** updated `bench/scanner_contention.py` to use `build_perf_stat_cmd` (`perf stat -C <scanner_cpu> -x, -o <csv> -e <events> -- ...`) identically in `preflight_perf` (on a pinned 3 ms user-space workload on `scanner_cpu`, trying `:u` events first with clean fallback to plain event names) and `measure_sample`, generalized `parse_perf_scanner_csv` for standard `-C -x,` and `cpu/<ev>/u` formats, and added regression tests in `bench/test_scanner_contention.py` (15 tests, 13 subtests pass). Frozen registration thresholds, grid, and verdict logic are unchanged.
**What's next:** merge this fix PR, move `ready/bench-scanner-contention` via `run-signal.yml` to the merged SHA, and await the physical i9 evidence bundle.
**Caveats:** none; zero timing data existed in `aecb640d`.
