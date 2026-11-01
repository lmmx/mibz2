# The resource attribution: the freeze record

**What is frozen:**
- the protocol (`../2026-11-01-decoder-attribution-protocol.md`) and amendment 1;
- `frozen.json`: 23 sources' SHA-256, the corpora, the schedule, the thresholds, the
  allocation rule and the toolchain;
- `bench/attribution-pin`: implementation `68956ca`.

The runner refuses to run if any source differs from its hash, or if HEAD differs from the
pin outside the pin file and this directory.

**The i9 command:** `just bench-attribution i9-10900K`, run from `research/decoder-attribution`.

## The dry run on the preparation VM (untimed; no figure interpreted)

**The run:**
- `ATTR_DRY_RUN=1`: the runner's whole path at 2 rounds and 1 repetition.
- **The VM:** an Intel Xeon @ 2.80 GHz, 4 CPUs, no SMT. It exposes no hardware PMU.
- **perf:** the installed 6.8.12 binary, not the kernel-matched wrapper, which is
  unavailable here.
- **Records:** `docs/decoder-attribution-records`.

| Step | Result |
|---|---|
| Sources | 23 match the frozen hashes |
| Builds | `occupancy` (trace) and `attribution_decode` (default features), rustc 1.97.0 |
| Tests | `trace_phases` (C1, C2) and the instrument's own tests: pass |
| C6, production unchanged | **same code:** 104 `mibz2` functions, 32,298 instructions identical to the base `45b366d`'s. Sensitivity, shown once: against a trace-feature build of the same example, 5 functions differ and 1 is added (exit 1) |
| Corpora | 3 match the frozen SHA-256 (the walk trial's) |
| Correctness | every corpus decodes to its frozen hash at w = 1 and 4 |
| Allocation | CPUs 0–3 on this VM: cpu0's core included, as there are only four. On the i9 the rule gives CPUs 1–4, siblings 11–14 |
| Events (this VM) | `task-clock` alone countable. 18 events recorded unavailable with perf's reasons. `perf record -e cycles` fell back to `cpu-clock`, was detected by `evlist`, and was refused, so no samples were taken |
| Analysis self-test | the counter, sample and rule path on synthetic perf output: pass |
| M1 timeline | complete traces at both w on all three corpora. No mode perturbs at this round count |
| Agreement with the instrument | within 0.011 on every timeline run (tolerance 0.02) |
| Siblings | none on this VM; the check runs |
| Rules | each evaluated or reported unresolved with its reason. H0 and H1 are unresolved here: no cycles, no samples |

**Elapsed on the VM:** 17 s of timeline and 2 s of counters (task clock only), plus the
builds.
