claim: forward-emission-adopt
by: antigravity
outcome: dropped
evidence: origin/evidence ccfa473f (forward-adopt/run-20261128T020039Z.tar.gz, SHA-256 2514bb60d89ca634483486824d0124e69e336023a4f73b1783f5ec04e030617d, commit 9875c712, rustc 1.99.1); bench/forward_adopt.py score; docs/notebook/2026-11-28-forward-emission-adopt-i9-verdict.md; frontier/dead-ends.tsv; frontier/claims/forward-emission-bounded-slab-rescue.toml

**What was asked:** verify and score the archived physical i9-10900K C6 adoption bundle for `forward-emission-adopt` (`#1110` + `#1217` disjoint-slab four-lane forward walk vs default bidirectional `OpenCells` walk), record the unrelaxed C2 verdict, remove the completed `bench-forward-adopt` queue entry, and file the bounded-slab rescue successor.

**What I found (unrelaxed C2 verdict: `DO NOT ADOPT`):**
- **Exactness & guards:** `exactness_ok: PASS` (100% parity across 34 untimed `AB_SHA256=1` runs vs C `libbzip2` and all 544 timed repeats), `distinct_binaries: PASS`, `guards_no_regression: PASS` (12/12 guard cells clean: `text/1 0.9906x`, `records/8 0.9945x`, `pattern/8 0.9899x`, `runs/1 0.9942x`, `onebyte/1 0.9959x`), `guards_rss_ok: PASS` (`0.9965-1.0022x`).
- **Wikidata wall:** `wikidata/1` `0.8462 [0.8450, 0.8474]` (`-15.38%`, 16/16 faster, PASS `<= 0.900`), `wikidata/6` `0.8648 [0.8594, 0.8702]` (`-13.52%`, 16/16 faster), `wikidata/10` `0.9649 [0.9541, 0.9758]` (`-3.51%`, 15/16 faster, PASS `<= 0.985`), `wikidata/12` `0.9681 [0.9320, 1.0055]` (paired med `0.9537`, 12/16 faster). **`wikidata/8` FAIL:** `0.9638 [0.9422, 0.9859]` (`t_ci95_hi = 0.9859 > 0.965`) despite `13/16` faster wall rounds (paired median `0.9553`, `-4.47%`) and `16/16` faster CPU rounds (`1.792 -> 1.653 s`, `-7.8%` CPU); 3 unpinned rounds (`r00 1.0254`, `r08 1.0777`, `r12 1.0182`) ran at full `4.88-4.89 GHz` (`cpr = 1.326`, `runq <= 0.1 ms`, `cpu < base`) with an unpinned scanner SMT-placement tail.
- **`wikidata_rss_ok` FAIL (smoking gun):** `wikidata/1` `32.12 -> 33.84 MiB` (`+1.72 MiB`, `1.0541x > 1.020x`), `wikidata/6` `54.83 -> 59.73 MiB` (`+4.89 MiB`, `1.0890x > 1.080x`), `wikidata/8` `60.90 -> 67.61 MiB` (`+6.71 MiB`, `1.1081x > 1.080x`), `wikidata/10` `60.79 -> 67.73 MiB` (`+6.94 MiB`, `1.1067x > 1.080x`), `wikidata/12` `61.14 -> 67.66 MiB` (`+6.52 MiB`, `1.1049x > 1.080x`). Root cause: `#1217`'s `Arena` retains up to `~32` `Box<[u8; 32768]>` slabs (`~0.84-1.0 MiB/worker`) in `Arena::pool` across the entire block on top of `out` (`+6.71 MiB` across 8 workers, exceeding both `1.080x` and `PERFORMANCE-POLICY.md` §2's `10%` bound).

**What changed:**
- Recorded complete 17-cell verdict and root-cause analysis in `docs/notebook/2026-11-28-forward-emission-adopt-i9-verdict.md`.
- Removed completed `bench-forward-adopt` entry from `bench/workstation-queue.txt` (`python3 bench/test_workstation_queue.py` passes).
- Closed `frontier/claims/forward-emission-adopt.toml` (`status = "dropped"`), appended scoped `DO NOT ADOPT` row to `frontier/dead-ends.tsv`, and updated `frontier/surface.tsv`.
- Filed P1 rescue successor `frontier/claims/forward-emission-bounded-slab-rescue.toml` (`status = "idea"`, `priority = 1`, `#![forbid(unsafe_code)]`) targeting the `+1.72 MiB` 1W / `+6.71 MiB` 8W slab-pool RSS cost via bounded-slab / streaming-RLE1+CRC32 emission (`<= 1.020x` 1W, `<= 1.030x` 6–12W on i9).

**What's next:** prototype and verify the `#![forbid(unsafe_code)]` bounded-slab RSS fix locally, then admit & run C6 physical i9 adoption for `forward-emission-bounded-slab-rescue` once `src/open.rs` is free.
