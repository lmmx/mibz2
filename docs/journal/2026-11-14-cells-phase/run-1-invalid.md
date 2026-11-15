# Cells phase attribution: run 1 INVALID, one unchanged re-run

- **Run 1:** `cells-phase/run-20261114T232758Z` (sha256 e6b802dd…52eb), at 6bcf2053.
- **Validity failed** on one reading: idle rate 10.2% of the decode's, pass B1, 1 thread,
  repetition 9. The registered bound is below 10%. No verdict was computed, and none of the
  run's figures are used (#145 LOG 6886081482).
- **The re-run** is the registered study unchanged. This commit adds only this note, so that
  the workstation queue (which runs a recipe at most once per commit) can run it. Nothing the
  recipe builds, runs or analyses differs from 6bcf2053.
- **If the re-run is INVALID too,** the study stops, and the idle pairing is redesigned before
  any new data (candidates: a logged retake of an over-bound pair; a gate on the median idle
  share per pass and thread count; gating only the passes the verdict uses).
