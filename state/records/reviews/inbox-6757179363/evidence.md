# Pool-wait evidence checkpoint — inbox 6757179363

Source #321: 02cc5f89988abb9ab436eae1af4b89576c19d590.
Run 42740378038; job 129579655245; artifact 13728908238.
ZIP SHA-256 verified: 7ef2c489e715bbd591412aa18221e73c27169b9db133b27f4384f5e2d730f1c9.

Independent archive arithmetic: all 1512 timing rows, 504 complete paired-round groups, 28 corpus/thread cells, three arms and 18 rounds. No errors; each corpus has one output/hash. Eighteen separately instrumented count rows also have one output/hash. Ordinary paired-log t17 replay in replay.json agrees with the reported screen: both arms have beneficial intervals, none of 112 arm/cell/metric intervals wholly exceeds 1. This validates reported arithmetic, not dependence assumptions or safety suites.

Important limits already established:
- Forced pipelined plans: requested 1/2/4 all use 1x3; requested 9 uses 3x3. env.json reports four logical CPUs on TWO physical cores (core_cpus 0,1 and 2,3). Four threads is not four independent physical cores; additional reader/scanner roles exist.
- Count runs omit timed ab's --spread affinity and use a separately instrumented executable. Counts cannot give exact per-cell causal attribution.
- wakes counts unpark calls, not kernel wakeups; parks counts calls, not verified descheduling.
- spin_ns_estimate uses minimum wall elapsed time of three idle atomic-load/spin_loop probes. It is not measured waiting-thread CPU cost under contention.
- 1x3 baseline pool spins 1080245; 3x3 1182232.33. 1x3 pool0 caller parks 340.67 versus254; pool64 340. 64 has almost no observed caller-park advantage.
- Both screen passes remain candidates only. Current target route's service draft requires >2% wall speed superiority plus CPU no-harm, so CPU-only spin reduction cannot discharge its wall condition.

Routing audit at the pinned coord.py reports five historical resolved misroutes and LOG6757065549 in #162. Coordinator LOG6757075751 already records its correction; no inbox edit/re-dispatch performed. No implementation, compilation, timing, safety certification or automation changes.
