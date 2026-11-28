claim: scanner-contention-bounded-recheck
by: antigravity
outcome: done
evidence: origin/evidence 1950f77ff56bbed8bde9510276472f53b7aace64 (`scanner-contention/run-20261128T065932Z.tar.gz`, 15393 B, sha256 `3857bbc196c499e28c8103736d4eda22a2ad27e55f779c78c1bc14efe2c29c82`, verified against `.sha256` and #145); commit `8346e392a6f6b73660107308812570b1e897df09` (post-#1312 `0762c7ea`); physical i9-10900K (`strict_i9=true`, `cpu_quota=unlimited`, `distinct_binaries=true`, 4/4 C libbzip2 SHA-256 parity); verdict EXPLAINED (`E=+0.0004 <= 0.010`, L3-stall increment `1.161x <= 1.5x`)

**What was asked:** rerun the frozen `bench-scanner-contention` PMU probe unchanged at a commit containing `#1312` (`80 KiB/worker` bounded slab pool) and score strictly against the frozen `EXPLAINED` / `PARTIAL` / `NOT EXPLAINED` stop test.
**What I found:** **VERDICT = `EXPLAINED`.** Bounding `forward-anchor-lanes`'s slab pool collapsed `99.1%` of the 8W scanner-cycle excess over `base` and lowered 8W peak RSS below `base`:
- **`base` 8W/4W (control):** `1.0231 [1.0199, 1.0263]` (`+17.49M` cyc, `+15.22M` L3-miss stalls, `87.0%` L3 share) — reproduces `a7ac01d8`'s `1.0230 [1.0196, 1.0265]` to `0.0001`.
- **`lanes` 8W/4W (post-`#1312`):** **`1.0235 [1.0202, 1.0268]`** (`+17.77M` cyc, `+17.67M` L3-miss stalls, `99.4%` L3 share) — collapsed from `a7ac01d8`'s `1.0647 [1.0619, 1.0675]` (`+49.28M` cyc, `+37.64M` L3-miss stalls).
- **Frozen stop-test clauses:** `E = 1.023457 - 1.023071 = +0.000386 <= 0.010` (PASS, down from `+0.0417`) and `lanes / base` 4W->8W L3-miss-stall increment `+17.67M / +15.22M = 1.161x <= 1.5x` (PASS, down from `2.44x`).
- **Cross-arm paired `lanes / base` (`n = 12`):**
  - `k = 1`: wall `1.0905s` (`0.769 GB/s`) vs `1.3092s` (`0.641 GB/s`) = `0.8290 [0.8160, 0.8421]` (`-17.10%`); peak RSS `78.95` vs `78.93 MiB` (`1.0003x`).
  - `k = 4`: wall `0.3010s` (`2.787 GB/s`) vs `0.3626s` (`2.313 GB/s`) = `0.8317 [0.8266, 0.8368]` (`-16.83%`); peak RSS `91.25` vs `91.93 MiB` (`0.9929x`).
  - `k = 8`: wall **`0.1691s` (`4.961 GB/s`)** vs `0.1910s` (`4.392 GB/s`) = **`0.8890 [0.8816, 0.8964]` (`-11.10%`, `-9.6 ms` faster than `a7ac01d8`'s `0.1787s` / `4.69 GB/s`)**; scanner cycles `775.59M` vs `774.99M` = **`0.9998 [0.9977, 1.0020]`** (IPC `1.840` vs `1.841`, LLC misses `1.0022x`); peak RSS **`103.29` vs `104.61 MiB` (`0.9900 [0.9834, 0.9966]`, `-1.32 MiB` below `base`, `-7.85 MiB` vs `a7ac01d8`'s `+6.53 MiB`)**.
**What changed:** dequeued `bench-scanner-contention` from `bench/workstation-queue.txt` (`bench/test_workstation_queue.py` 4/4 PASS), recorded the verdict in `docs/notebook/2026-11-28-scanner-contention-bounded-recheck-verdict.md`, updated `frontier/surface.tsv` row `S` and `frontier/DECODER.md`, and closed `scanner-contention-bounded-recheck` (`status = "done"`, frozen `stop_test` untouched).
**What's next:** proceed with `forward-emission-bounded-slab-rescue`'s full C6 adoption run (`bench-forward-adopt` post-`#1312`), which now ships both `-1.0%` lower 8W RSS and ~`4.96 GB/s` pinned-scanner 8W throughput without scanner L3 contention.
**Caveats:** `bench-scanner-contention` pins `mibz2-scan` to core 0 with workers on cores `1..9,11..19`; unpinned 10T/8W adoption still relies on the registered C6 `bench-forward-adopt` suite.
