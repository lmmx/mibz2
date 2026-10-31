# Exploratory host exception, recorded before timing

On 31 October 2026 the maintainer explicitly authorized one exploratory single-core
run of the prepared P74-C experiment on the existing Codex Cloud EPYC 9V74 VM.
This overrides the host exclusion only for this run. The strict qualification
protocol remains unchanged; neither its original assembly gate nor host gate is
retroactively declared passed. No shipping qualification is possible from this run.

Reference HEAD: 1d07d84015b5c96005c670a1d3ce20427a51305f.
Implementation remains 1bf554db61a82ff86f832b8697b90675e6a6ba46.
The sole harness change recognizes the explicit named exception, verifies model
EPYC 9V74 and quota 200000/100000, and prints the exception on stderr. Without
that named exception the strict quota guard still applies. Timing loop, arm order,
compiler flags, kernels, policies, analysis and dataset bytes are unchanged.

Pin CPU 0 (one of the three exposed cores, no exposed SMT siblings), run the five
frozen corpora in their existing order with the original two warm-ups per arm,
12 balanced paired rounds and two passes per observation, plain then checkpoints.
No rows may be dropped or selectively repeated. No further timing run is authorized
by this record. No multicore, i9, PMU, CI dispatch or optimization experiments.

Before/after: snapshot cpu.stat, cpu.max, /proc/stat, CPU pressure, exposed topology,
affinity and process names. Record wall duration, compiler/source/binary/data hashes.
Predeclared conservative noise flags: throttled_usec increase >1% of run wall time,
nr_throttled increase >1% of nr_periods increase, or pinned CPU steal-time share
>1% (where exposed) marks economic interpretation inconclusive. These are quality
flags, not exclusions or permission to rerun. Unreported hypervisor interference
remains possible even when these flags are absent. Report every nominal interval
and unfavorable result regardless. Compare inherited economic criteria descriptively,
not as final qualification on this host.

Command (after build and checks, no other workloads launched concurrently):

```sh
P74_APPROVED_RUNNER=codex-epyc9v74-cpu0-exploratory \
P74_EXPLORATORY_HOST_EXCEPTION=codex-epyc9v74-single-core-2026-10-31 \
taskset -c 0 target/release/examples/p74_experiment --measure-approved \
  /workspace/work/p74/experiment/corpora/wikidata-fitted.bz2 \
  /workspace/work/p74/experiment/corpora/random.bz2 \
  /workspace/work/p74/experiment/corpora/records.bz2 \
  /workspace/work/p74/experiment/corpora/mixed-independent.bz2 \
  /workspace/work/p74/experiment/corpora/text.bz2
```

All stage throughput denominators are decoded MTF L bytes, not final RLE1 output.
Fitted Wikidata remains fitted; the independently seeded mixed control is not
independent Wikidata. No numerical whole-decoder speedup will be inferred as measured.
