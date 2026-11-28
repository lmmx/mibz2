claim: scanner-contention-residual
by: antigravity
outcome: done
evidence: origin/evidence a7ac01d8 (scanner-contention/run-20261128T052545Z.tar.gz, sha256 ce86eaf2b812e3e0beae93684941daf98b72caf9864eba8d37873cd029550d09); docs/notebook/2026-11-28-scanner-contention-residual-verdict.md; frontier/DECODER.md; frontier/surface.tsv

Verified and recorded the physical i9-10900K PMU **`CONFIRMED`** verdict for
`scanner-contention-residual` (`c7fa3284`, pinned core `0` scanner, `12` rounds,
`2 s` settle, `wikidata-x4` `838,800,784 B`, `4/4` SHA-256 parity exact,
independent `bench/scanner_contention.py score` re-score byte-identical).

- **Frozen C1a verdict (`CONFIRMED`):** `lanes 8W/4W` scanner cycles =
  `1.0647 [1.0619, 1.0675]` (`> 1.000`, `+49.28M` cyc), and
  `cycle_activity.stalls_l3_miss:u` explains **`76.4%`** (`+37.64M` cyc, `>= 50%`).
- **Six-cell PMU split:** scanner instructions are invariant (`1,427.0M`,
  `0.99997..1.00001x`) and `cycles/ref-cycles` cleanly separates the `1W -> 4W`
  turbo bin (`1.43 -> 1.33`, flat `4W -> 8W`). At `4W`, `base` cycles are flat
  vs `1W` (`0.9997 [0.9971, 1.0024]`). From `4W` to `8W` at constant clock:
  - `base/8`: `774.91M` cyc (`1.0230 [1.0196, 1.0265]`, `+17.43M` cyc, **`88.4%`**
    = `+15.40M` L3-miss stalls, `5.1%` L3 stall share, `0.501M` LLC misses, wall
    `0.1926 s` = `4.36 GB/s`).
  - `lanes/8`: `811.14M` cyc (`1.0647 [1.0619, 1.0675]`, `+49.28M` cyc, **`76.4%`**
    = `+37.64M` L3-miss stalls, `8.1%` L3 stall share, `0.591M` LLC misses, IPC
    `1.872 -> 1.759`, wall `0.1787 s` = `4.69 GB/s`).
- **Causal link & disposition:** At `8W`, `lanes` carries `+6.53 MiB` higher peak
  RSS (`110.88` vs `104.35 MiB`) from `#1217`'s unbounded `Arena::pool`, pushing
  the 8W working set past the i9's 20 MiB L3 and more than doubling scanner
  L3-miss stalls (`2.44x`). P1 `forward-emission-bounded-slab-rescue` (PR `#1312`,
  `80 KiB` bounded-slab Arena) already targets this exact lever, so no duplicate
  idea claim is filed. Dequeued `bench-scanner-contention` from
  `bench/workstation-queue.txt` and updated `frontier/DECODER.md` and `surface.tsv`.
