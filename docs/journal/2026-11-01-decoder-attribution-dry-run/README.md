# The resource attribution's dry run on the preparation VM (untimed; no figure interpreted)

- **Source:** `ATTR_DRY_RUN=1 bash bench/attribution_run.sh preparation-vm`, at
  `research/decoder-attribution` `1b915e5` (pin `68956ca`).
- **What it checks:** the whole path at 2 rounds and 1 repetition.
- **The VM:** 4 CPUs, no SMT, no hardware PMU. So H0 and H1 are unresolved here by
  construction, and the hypotheses' statuses in `result.json` mean nothing about any
  decoder.
- **What it shows:**
  - the runner's checks pass (sources, C6, corpora, correctness, tests);
  - the event probe records unavailable events, and refuses `perf record`'s fallback to
    `cpu-clock`;
  - the raw-marks analysis agrees with the instrument (worst 0.011 of worker time);
  - the self-test on synthetic counts runs the rules end to end.
- **Paths:** written as `<repo>` and `<scratch>`. The binaries are not kept; their
  SHA-256 is in `binaries.sha256`.
- **See** `docs/journal/2026-11-01-decoder-attribution/freeze.md` for the step table.
