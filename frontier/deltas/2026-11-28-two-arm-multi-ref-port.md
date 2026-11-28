claim: two-arm-multi-ref-port
by: claude-code
outcome: done
evidence: bench/two_arm.py and bench/test_two_arm.py from origin/bench/open-port a1dc5aeb; examples/ab.rs `ab run --unpinned`; test_two_arm 159/159 (157 open-port + 2 new); master's former 52 checks 52/52 on the ported scorer; ab example tests 30/30, clippy -D warnings clean

**What was asked:** put the multi-reference scorer that recalibration recipes invoke on master, and give `ab run` a real `--unpinned`. The goal is that the next §5c run doesn't return an empty verdict, as recalibrate-0518 did.
**What I found / did:**
- **Master's `two_arm.py` had not changed since the open-port branch point (3182b904).** So open-port's scorer is a strict superset: I took it and its tests verbatim. Master's old 52-check suite also passes on it, 52/52.
- **New check in `test_two_arm.py`:** every `bench/two_arm.py` call in `.just/*.just` must parse on this checkout (argparse exit 2 fails; a VOID on absent records is fine). On master that covers `bench-cores-less-one` and `bench-w12`. An unknown flag gives exit 2, so a recipe that outruns the scorer now fails CI rather than an i9 hour.
- **`ab run --unpinned`:** every run, 1 thread included, goes under `taskset -c <every online CPU>`, and the record's `cpus` holds that explicit list rather than `all`. That is what the open-port scorer's reviewed unpinned check demands: it treats `all` as inherited affinity, not proof. The flag is refused beside `--spread`, `--pin` or `--hold-cpu` (exit 2). It reuses the existing `taskset` path: no new `unsafe`. Smoke test here: `cpus` 0,1,2,3 at 1 and 2 threads on a 4-CPU box.
- **Frozen item (4):** the pinned re-score in `docs/notebook/2026-11-28-recalibrate-0518-readback/pinned-verdict.txt` is reproduced byte for byte with the merged scorer.
- **Not done:** `counters.py --events imc`. Memory-controller counts need system-wide `perf -a`, which `bench/imc.py` already does. A recipe that wants DRAM traffic should call `imc.py`. No master recipe uses `--events imc`.
**What changed:** `bench/two_arm.py`, `bench/test_two_arm.py`, `examples/ab.rs`, this delta; claim marked done.
**What's next:** recalibrate-0518's unpinned records still say `all`, so under this scorer they stay VOID, as the readback found. For release-0-5-18's holder:
- either accept the pinned verdict and the unpinned diagnostic (my readback's recommendation);
- or rerun the unpinned half with this `ab` (the recipe's `--unpinned` now means what it meant to say) and judge it with the recipe's own invocation.
**Caveats:** `taskset` sets the affinity but `ab` doesn't read it back. A cpuset narrower than the online CPUs would make `taskset` fail, and the run would then not succeed, rather than shrink silently.
