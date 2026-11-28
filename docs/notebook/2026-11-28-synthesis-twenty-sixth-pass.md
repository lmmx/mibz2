# Synthesis, twenty-sixth pass: the 8W L3 slope holds, the open-route CI cell pays off, `huff-bmi2-build`'s premise is already on master, and the review queue is the bottleneck

*28 Nov 2026, master `4f3e9533`. Claim `synthesise`, continuing from the twenty-fifth pass (`#1321`, `e3dc64b1`). No new runs. Every measured figure is quoted from the delta, PR, or bundle named beside it; anything marked *derived* is arithmetic on those figures, not a measurement.*

Landed since `#1321`: `#1322`/`#1323`/`#1325`/`#1326` (`scanner-contention-bounded-recheck` admitted, queued at `8346e392`, and scored **EXPLAINED** from physical i9 bundle `1950f77f`), `#1245` (`ci-bench-open-route-arm`, non-gating 1T `mibz2-workers-lazy` cell in CI), `#1327`/`#1329` (`abi3-py310` Python delivery dissection and admission of `python-abi3-delivery-build`), `#1332` (`bench-crc32-ring-rescue` requeued at `30886324` after `0390e19f` VOID), `#1331` (pre-i9 CI primary-loss record on `#1295`), and `#1330` (outside-reader blog post on the L3 slab-contention result).

## 1. Working-set coupling confirmed (`1950f77f`, `EXPLAINED`) — re-pricing the coupled 8W system

The twenty-fifth pass (§1) predicted from `a7ac01d8`'s `~4.9M` cycles/MiB slope that `#1312`'s bounded `80 KiB/worker` lanes Arena would pull `lanes` 8W/4W scanner cycles from `1.0647×` down to `~1.025–1.03×`. Rerunning `bench-scanner-contention` unchanged on the i9-10900K at `8346e392` (`origin/evidence` `1950f77f`, `#1326`) confirmed the prediction with room to spare:

| Metric (`wikidata-x4`, core-0 pinned scanner, `n = 12`) | Pre-`#1312` (`a7ac01d8`) | Post-`#1312` (`1950f77f`) | Shift |
|---|---:|---:|---:|
| `base` 8W/4W scanner cycles (control) | `1.0230 [1.0196, 1.0265]` | `1.0231 [1.0199, 1.0263]` | `+0.0001` |
| `lanes` 8W/4W scanner cycles | `1.0647 [1.0619, 1.0675]` | **`1.0235 [1.0202, 1.0268]`** | **`-4.12 pp`** |
| Excess `E = lanes - base` (`EXPLAINED <= 0.010`) | `+0.0417` | **`+0.000386`** | **99.1% removed** |
| 4W→8W L3-miss-stall increment ratio (`<= 1.5×`) | `2.44×` (`+37.64M` vs `+15.40M`) | **`1.161×`** (`+17.67M` vs `+15.22M`) | `-1.28×` |
| 8W peak RSS (`lanes` vs `base`) | `+6.53 MiB` (`1.061×`) | **`-1.32 MiB`** (`103.29` vs `104.61 MiB`, `0.9900×`) | **`-7.85 MiB`** |
| Pinned 8W wall (`lanes` vs `base`) | `0.1787 s` (`4.69 GB/s`) | **`0.1691 s` (`4.961 GB/s`, `-11.10%` vs `base/8`)** | **`-9.6 ms` (`-5.4%`)** |

*Derived coupled-system re-pricing:* at `1950f77f`'s `lanes/8` clock (`clk_r = 1.3260` on a `3.70 GHz` base clock = `4.906 GHz`), `775.59M` core-0 scanner cycles equal **`~158.1 ms`** of active serial scan against **`169.1 ms`** measured wall — an **`~11.0 ms`** (`6.5%`) pipeline/drain margin, while 1W/4W `lanes` wall (`1,090.5 ms` / `301.0 ms`, `-17.10%` / `-16.83%` vs `base`) leaves worker demand well below the scanner at 8 workers. Consequently:

1. **8W `lanes` is cleanly serial-scanner-bound at zero extra L3 tax.** Serial-scanner cuts now convert near-1:1 into 8W wall until workers re-bind near `~140–150 ms` (*derived*).
2. **Concrete path to `5.5–6.0 GB/s` (`152.5–139.8 ms` on `838.80 MB`):** `scanner-a2-fast-build` is `status = "done"` and its stale reservation lapsed at `08:27:13Z`, unblocking `src/block.rs` + `src/huff.rs` for `scanner-a3-k3-pipe-u8` (`-5.64..-7.65%` of paid A2d+A3 on VM; `~-7..11 ms` *derived* on i9). Composing `scanner-a3-k3-pipe-u8` with `scanner-adaptive-seam-composed` (`-9.95 ms` measured on i9 in `d6bc8ca2` for 8W window maintenance; `#1235` verified clean 3-layer composition) and `u8` `syms` handover targets **`~149–154 ms` (`~5.45–5.63 GB/s`, *derived*)** on 8W `lanes` before worker levers beyond `#1312` (`L6a` stitch removal, `A6b`, `A6a`) become the next binding limit toward `140 ms` (`6.0 GB/s`).
3. **Adoption de-risking:** `1950f77f` removes the single biggest RSS/contention risk on `bench-forward-adopt-rescue` (`#1324`), making `#1324`'s C6 review the highest-value i9-queue unlock.

## 2. Open-route CI diagnostic cell (`#1245`) pays off immediately on `#1328`

`#1245` (`9dd08e0a`, `ci-bench-open-route-arm`) landed a non-gating 1-thread `mibz2-workers-lazy` cell (`wikidata` + `text`) in `.github/workflows/bench.yml` (`+43 s` on `#1245`, `+19 s` on `#1328`), leaving `PERFORMANCE-POLICY.md` §2 untouched.

Its first live test on PR `#1328` (`a6b-zero-rss-endpoint-build`, `37a2a37a`, `#![forbid(unsafe_code)]`, 32/32 CI green, AMD EPYC 9V74) immediately exposed what the default `Cells`-route 1T table hid:
- Default 1T `wikidata` (`Cells` route): `-0.9% [-4.8%, +3.2%]` (noise, route not exercised).
- **Forced open-route 1T `wikidata` (`mibz2-workers-lazy`):** **`0.969 [0.949, 0.990]` (`-3.1%` wall)** and **`0.969 [0.950, 0.989]` (`-3.1%` CPU)** at **`26 -> 26 MB` (`0 B` extra RSS)**; `text` 1T flat (`1.006 [0.985, 1.028]`, `23 -> 23 MB`).

*Update to twenty-fifth pass §3:* `#1328`'s EPYC open-route gain (`-3.1%`) is almost exactly half its VM `workers-lazy` 1W gain (`-6.23%`), consistent with `#1274`'s `~0.33–0.5×` VM-to-target transfer ratio while proving `0 B` RSS overhead. Because `1950f77f` proved RSS-neutral worker cuts avoid any 8W scanner L3 tax, `#1328`'s value is clean on the Open route (`wikidata` at `t >= LAZY_FROM_THREADS`, or `t = 1..4` once `plan-schedule-and-lazy-threshold` / low-thread open unification ships). Any i9 registration for `#1328` must gate on an Open-exercising cell rather than a `Cells`-routed 1T cell.

## 3. Premise correction for `huff-bmi2-build`: the `decode_runs` BMI2 win is already on `master`

Direct inspection of `origin/master` (`4f3e9533`, `src/block.rs:131–169`) confirms that block-level `#[target_feature(enable = "bmi2")] unsafe fn decode_runs_bmi2` already shipped in `#1034` (`5ed20c1d`) and was carried forward by `#1117` (`15c289b5`), emitting `shlx`/`shrx` with zero `vzeroupper` at one dispatch per block.

The only remaining gaps relative to `huff-bmi2-build`'s original scope are:
1. **Dispatch hygiene:** `src/block.rs:163` calls `std::is_x86_feature_detected!("bmi2")` directly, so `Kernels::scalar()` does not disable BMI2 in `decode_runs`.
2. **Residual non-decode work:** `read_selectors` / code-length decode and AVX2+BMI2 `build_into` (~`0.5–1.5%` on post-`#1117` `master`, *derived*, compared with the pre-`#1034` `>= 4.0%` frozen scan-phase gate).

**Consequence (save an i9 run):** Do **not** spend an i9 adoption slot testing `huff-bmi2-build` against current `master` under its pre-`#1034` `>= 4.0%` gate — that `4%+` gain is already realized on `master`, so a re-run would predictably fail the frozen gate on already-collected gains. Handle `Kernels.bmi2` wiring as a tiny correctness/hygiene follow-up (zero i9 slot needed), and defer any BMI2 table-build specialization until *after* `scanner-a3-k3-pipe-u8` replaces the `K=2` table builder with `K=3`. Dated note appended to `frontier/claims/huff-bmi2-build.toml`'s `why` field (status and frozen `stop_test` untouched per C2).

## 4. Python `abi3-py310` delivery breakthrough (`#1327`, `#1329`) & whole-stream RSS map

`#1327` (`31c35662`) dissected every `abi3-py310` Python API path with zero `unsafe` and zero packaging/wheel-matrix changes (`#1329` admitted `python-abi3-delivery-build` to `open`, ordinary-CI acceptance route, zero i9 queue cost):
- **Streaming & line iteration (VM `n = 7`, 0/25 regressions):** `25.4×` faster raw `BZ2Reader` line iteration (`-96.1%` CPU), `-27.7..-29.3%` CPU (`-34.8..-41.4%` wall) on `open().readlines()`, `-13.5..-15.3%` CPU on `for line in open()`, and `-7.4..-9.7%` CPU (`-12.8%` HWM at `t = 1`) on `wikidata` `open().read(64 KiB)`.
- **Whole-stream `2×` RSS floor mapped for `python-zero-extra-copy` (`needs_louis`):** PyO3 `0.29.3`'s `PyBytes::new_with` unconditionally zeroes the buffer via `core::ptr::write_bytes(buffer, 0u8, len)` (`bytes.rs:123`), forcing `2×` RSS (`437 MiB` on `210 MiB` `wikidata`). Direct `libc` probing showed chunks `> 32 MiB` (`> DEFAULT_MMAP_THRESHOLD_MAX`) bypass `brk` retention unconditionally and cut peak RSS `-41.3%` (`-45.3%` at `16 MiB`). This sharpens the owner decision on `python-zero-extra-copy`: dropping `abi3-py310` for per-version safe `PyBytesWriter` saves both the copy and `2×` RSS, whereas ~12 lines of Limited-API FFI + `> 32 MiB` `mmap` chunks cuts `2×` RSS without changing wheel packaging.

## 5. Bottleneck snapshot: review queue & workstation queue (honest ledger)

- **Workstation queue (replenished, 1 runnable entry):** `bench-crc32-ring-rescue` (`#1295` fixed and pinned at `30886324` via `#1332` after `0390e19f`'s `CARGO_MANIFEST_DIR` `VOID`). **Important pre-data constraint (`#1331`, `67182753`):** on AMD EPYC 9V74 against `31c35662`, `#1295` at `30886324` showed primary losses (`wikidata/1 +1.0%`, `text/1 +0.9%`, `text/2 +3.7%`), failing registration §4's CI merge condition as frozen. Even if the i9 run passes §3, `#1295` cannot merge as all-x86 default — an Intel-gated successor would need its own registration.
- **Six `CLEAN`, CI-green PRs awaiting independent O2/B5/C6 scrutiny (ordered by leverage):**
  1. **`#1324` (P1, `forward-emission-bounded-slab-rescue`, `9b438827`, 24/24 pass):** frozen Clause 3 C6 i9 registration & recipe — **top priority** to keep the i9 queue fed with the highest-impact multi-corpus adoption run (`1950f77f` already confirmed its 8W RSS/contention fix).
  2. **`#1256` (P1, `pre-release-i9-gate`, `653048a0`, 24/24 pass):** enforces physical i9 pre-release regression gate — unblocks P1 `release-0-5-18`.
  3. **`#1328` (P2, `a6b-zero-rss-endpoint-build`, `37a2a37a`, 32/32 pass):** zero `unsafe`, `-3.1%` open-route CI win at `0 B` extra RSS.
  4. **`#1282` (P2, `plan-schedule-and-lazy-threshold`, `d8d0c8b5`, 31/31 pass):** lowers `LAZY_FROM_THREADS` to 4, fills 5T/7T/9T trims, adds C6 i9 recipe.
  5. **`#1311` (P2, `segmented-fused-postwalk-rescue`, `df4f0bbd`, 31/31 pass):** ordinary-CI acceptance route, fuses `Segmented` `Runs` post-walk while preserving unfused `Column` for `random` 1T.
  6. **`#1303` (`Claim: none`, `fix/merged-hold-moved-head`, `5fefd6ef`, 23/23 pass):** frontier tooling fix preventing stale integration holds when descendant PR heads move.

## Claims and docs touched

- `frontier/claims/huff-bmi2-build.toml`: appended dated synthesis-26 premise note to `why` (status and frozen `stop_test` untouched).
- `frontier/DECODER.md`: added concise synthesis-26 coupled-system and BMI2 premise note (all existing entries intact).
- `synthesise` stays `open` (recurring).
