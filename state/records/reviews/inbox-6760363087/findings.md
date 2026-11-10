# Width draft: interim findings

Inbox6760363087; draft f918f783aa6b9f683d867c057a4c4937c99392ca, blob d16f1c6bc8e1f645616df9d8f62cc947e25c6c20.
REVISE before freeze. No implementation or timing.

- Physical u32 index k*r (k=1/4) preserves load/shift recurrence; not a proof of equal complete walker or builder. Charge starts, orig_ptr, unmark, bounds, tail, prefetch and logical-n conversions. One-row constructor different from production invalidates direct transplantation of f and occupancy.
- Nominal 200k-row page count at 4KiB is196/782 with page alignment,197/783 worst 64B alignment, not exact200/800. Distinguish logical4N stores in both arms from unique cache lines and repeated memory traffic.
- 1831*maximum block size is not observed row sum. Successful RLE1 gives N<=1.25*209700196=262125245; even unique-line byte upper budgets are approximately1.049/4.195GB, not1.5/5.9GB (DRAM traffic may be higher and is not determined).
- B4/A4 includes cache effects and does not identify pages/traffic separately; four tables nominally inside L3 do not establish fitting total active working set. Invariant side-effect cancellation requires explicit scale/model; whole-process additive components do not generically cancel in log interaction.
- exp(I)<1.15 is at most a small interaction on this exact layout/input/counts, not a kill of all compact tables. Equal factor2 width penalties at both W give exp(I)=1 despite large table effects.
- Historical slot-indexed round0-excluded replay: SD log W10/W4 CPU=.105273(-2),.057699(-9); SD paired difference=.111984. Existing judge is moving-block bootstrap5 with1/10 sensitivities; ±5% iid arithmetic is not its coverage.
- 210 slots remains arithmetic only; now5clocks/block,9155 reads/-2decode,228.875microseconds at assumed25ns. Untimed exact matched counts and source safety review before workstation recipe; no count performed by this role.
