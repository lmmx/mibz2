# Carbon Cycle 40 (`2/20`) — Push-Streaming `StreamDecoder::parscan` Prototype, 1,500-Case Chunked Fuzz Parity, & Cumulative-`Arc` / `notify_all` Kill

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Base Reference:** `origin/master` (`53f7ecf935d62ac7896eafaf55c5d249789614ca`)  
**Artifact:** [c40_par_scan_stream.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle40/c40_par_scan_stream.rs)

---

## 1. Streaming Contract Verification (`PASS 1..5`) & One-Line Performance Kill

1. **Streaming Contract Exactness (`PASS 1..5` — Retained Architecture Invariants):**
   - Implemented `StreamDecoder::parscan` and `BzReader::parallel_parscan` (`#![forbid(unsafe_code)]`) satisfying all streaming semantics of `docs/journal/2026-11-10-parallel-scan.md` §6:
     - `PASS 1`: Full `11.85 MB` `wikidata.bz2` decoded across multiple `4 MiB` window compactions (`209.70 MB` byte-exact).
     - `PASS 2`: Prime and sub-header streaming write/read chunkings (`1 B`, `7 B`, `63 B`, `4 KiB`, `100,003 B`) byte-exact across `3..20` workers (`wait-on-overrun` before EOF verified).
     - `PASS 3`: Mid-stream `cancel()` and blocked-writer wakeup verified.
     - `PASS 4`: Panicking `Read` source unwind through `BzReader::parallel_parscan` verified.
     - `PASS 5`: `1,500 / 1,500` mutated multi-stream corrupt/truncated inputs under random write/read chunk sizes matched `origin/master` on both partial output bytes before the error and `Error { kind, block, bit }`.
2. **One-Line Performance Kill (`C40` Cumulative `Arc::from` + Shared-`Condvar` `notify_all`):**
   - **KILLED `C40` (`Arc::from(&buf[..])` cumulative snapshots + single-`Condvar` `notify_all`):** Cumulative `Arc::from(&buf[..])` re-copies the window prefix on each growth (`21.6 MB` copied on `9.59 MB` `records-l9`), dispatches boundary-straddling candidates before the next chunk is pulled (causing wasted `overran` double-decodes), and broadcasts `shared.cv.notify_all()` to all $W$ workers on every tile and block completion (`1.042×` wall on `wikidata-l9` at `8W`, `1.304×` wall / `275.9 ms` vs `206.4 ms` CPU on `records-l9` at `20W`).

> [!NOTE]
> Diagnostic note on `C40_Serial` in §2: `LAST_COORD_WORK_NS` in `C40` subtracted `cv.wait` from `Instant::now()` (wall clock rather than `CLOCK_THREAD_CPUTIME_ID`), so under `8W–20W` oversubscription on 2 vCPUs it included OS runqueue wait time while runnable. Cycle 41 switches coordinator telemetry to `CLOCK_THREAD_CPUTIME_ID` and replaces `Arc::from` + `notify_all` with a zero-copy `Arc<Vec<u8>>` chunk rope + split `coord_cv` / `worker_cv` `notify_one`.

---

## 2. Push-Streaming `1T | 2T | 4W | 8W | 10W | 16W | 20W` A/B + A/A Results (`C40` — Killed)

### 2.1 `wikidata-l9` (`11.85 MB` $\to$ `209.70 MB`)

| Workers | `MasterA` Wall (ms) | `MasterB` Wall (ms) | `C40` Wall (ms) | `A/A` Wall | `C40 / Master` Wall | `Master` CPU (ms) | `C40` CPU (ms) | `Master` Serial (`%CPU` / `%Crit`) | `C40` CoordWall+Read (`%CPU` / `%Crit`) | `Master` `CritW` (ms) | `C40` `CritW` (ms) | `C40 / Master` `CritW` |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `1T` | `483.28` | `481.98` | `467.96` | `0.9973×` | **`0.9683×`** | `482.89` | `467.80` | `76.14 ms` (`15.8%` / `15.8%`) | `61.70 ms` (`13.2%` / `13.2%`) | `482.89` | `467.80` | `0.9688×` |
| `2T` | `259.65` | `257.08` | `257.46` | `0.9901×` | **`0.9916×`** | `506.93` | `501.50` | `88.14 ms` (`17.4%` / `42.1%`) | `36.68 ms` (`7.3%` / `15.7%`) | `209.40` | `233.65` | `1.1158×` |
| `4W` | `269.11` | `268.22` | `266.70` | `0.9967×` | **`0.9910×`** | `522.73` | `528.73` | `88.19 ms` (`16.9%` / `81.1%`) | `46.59 ms` (`8.8%` / `38.6%`) | `108.74` | `120.69` | `1.1099×` |
| `8W` | `277.96` | `278.32` | `289.68` | `1.0013×` | `1.0422×` | `537.36` | `559.56` | `88.51 ms` (`16.5%` / `100.0%`) | `104.45 ms` (`18.7%` / `100.0%`) | `88.51` | `104.45` | `1.1800×` |
| `10W` | `286.89` | `286.33` | `305.00` | `0.9981×` | `1.0631×` | `555.08` | `590.05` | `90.13 ms` (`16.2%` / `100.0%`) | `124.94 ms` (`21.2%` / `100.0%`) | `90.13` | `124.94` | `1.3862×` |
| `16W` | `292.15` | `293.02` | `300.92` | `1.0030×` | `1.0300×` | `568.58` | `588.67` | `90.05 ms` (`15.8%` / `100.0%`) | `129.65 ms` (`22.0%` / `100.0%`) | `90.05` | `129.65` | `1.4397×` |
| `20W` | `297.53` | `296.00` | `310.59` | `0.9949×` | `1.0439×` | `576.75` | `607.80` | `90.51 ms` (`15.7%` / `100.0%`) | `130.58 ms` (`21.5%` / `100.0%`) | `90.51` | `130.58` | `1.4428×` |

### 2.2 `wikidata-l1` (`3.22 MB` $\to$ `36.00 MB`)

| Workers | `MasterA` Wall (ms) | `MasterB` Wall (ms) | `C40` Wall (ms) | `A/A` Wall | `C40 / Master` Wall | `Master` CPU (ms) | `C40` CPU (ms) | `Master` Serial (`%CPU` / `%Crit`) | `C40` CoordWall+Read (`%CPU` / `%Crit`) | `Master` `CritW` (ms) | `C40` `CritW` (ms) | `C40 / Master` `CritW` |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `1T` | `116.46` | `116.76` | `106.48` | `1.0026×` | **`0.9143×`** | `116.40` | `106.48` | `29.23 ms` (`25.1%` / `25.1%`) | `21.13 ms` (`19.8%` / `19.8%`) | `116.40` | `106.48` | `0.9148×` |
| `2T` | `73.93` | `75.60` | `65.50` | `1.0225×` | **`0.8860×`** | `132.34` | `127.19` | `31.93 ms` (`24.1%` / `63.6%`) | `8.80 ms` (`6.9%` / `14.9%`) | `50.21` | `59.20` | `1.1791×` |
| `4W` | `73.66` | `73.18` | `65.74` | `0.9935×` | **`0.8924×`** | `133.39` | `129.41` | `31.93 ms` (`23.9%` / `100.0%`) | `6.48 ms` (`5.0%` / `21.1%`) | `31.93` | `30.71` | `0.9618×` |
| `8W` | `71.59` | `73.09` | `69.85` | `1.0210×` | **`0.9757×`** | `134.52` | `131.25` | `31.95 ms` (`23.7%` / `100.0%`) | `14.58 ms` (`11.1%` / `96.8%`) | `31.95` | `15.06` | `0.4715×` |
| `10W` | `73.14` | `72.53` | `73.64` | `0.9917×` | `1.0069×` | `136.28` | `136.80` | `31.88 ms` (`23.4%` / `100.0%`) | `19.80 ms` (`14.5%` / `100.0%`) | `31.88` | `19.80` | `0.6211×` |
| `16W` | `73.62` | `73.31` | `74.69` | `0.9958×` | `1.0145×` | `138.76` | `139.61` | `32.25 ms` (`23.2%` / `100.0%`) | `19.93 ms` (`14.3%` / `100.0%`) | `32.25` | `19.93` | `0.6180×` |
| `20W` | `74.37` | `73.98` | `76.72` | `0.9947×` | `1.0316×` | `141.65` | `142.17` | `32.16 ms` (`22.7%` / `100.0%`) | `20.29 ms` (`14.3%` / `100.0%`) | `32.16` | `20.29` | `0.6309×` |

### 2.3 `records-l9` (`9.59 MB` $\to$ `9.60 MB`)

| Workers | `MasterA` Wall (ms) | `MasterB` Wall (ms) | `C40` Wall (ms) | `A/A` Wall | `C40 / Master` Wall | `Master` CPU (ms) | `C40` CPU (ms) | `Master` Serial (`%CPU` / `%Crit`) | `C40` CoordWall+Read (`%CPU` / `%Crit`) | `Master` `CritW` (ms) | `C40` `CritW` (ms) | `C40 / Master` `CritW` |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `1T` | `156.03` | `156.52` | `150.33` | `1.0031×` | **`0.9635×`** | `155.97` | `150.32` | `38.27 ms` (`24.5%` / `24.5%`) | `32.21 ms` (`21.4%` / `21.4%`) | `155.97` | `150.32` | `0.9637×` |
| `2T` | `89.45` | `88.56` | `90.38` | `0.9901×` | `1.0105×` | `167.19` | `169.40` | `38.87 ms` (`23.2%` / `60.5%`) | `18.07 ms` (`10.7%` / `24.0%`) | `64.21` | `75.44` | `1.1750×` |
| `4W` | `91.60` | `93.43` | `95.21` | `1.0199×` | `1.0394×` | `172.58` | `185.03` | `38.96 ms` (`22.6%` / `100.0%`) | `27.94 ms` (`15.1%` / `71.8%`) | `38.96` | `38.91` | `0.9987×` |
| `8W` | `107.27` | `109.16` | `112.39` | `1.0176×` | `1.0477×` | `203.85` | `218.06` | `38.91 ms` (`19.1%` / `100.0%`) | `31.97 ms` (`14.7%` / `100.0%`) | `38.91` | `31.97` | `0.8216×` |
| `10W` | `110.20` | `109.92` | `122.74` | `0.9974×` | `1.1138×` | `208.55` | `237.51` | `38.93 ms` (`18.7%` / `100.0%`) | `42.25 ms` (`17.8%` / `100.0%`) | `38.93` | `42.25` | `1.0854×` |
| `16W` | `105.21` | `111.66` | `132.73` | `1.0613×` | `1.2615×` | `200.51` | `255.61` | `38.90 ms` (`19.4%` / `100.0%`) | `47.19 ms` (`18.5%` / `100.0%`) | `38.90` | `47.19` | `1.2131×` |
| `20W` | `109.45` | `110.15` | `142.67` | `1.0064×` | `1.3035×` | `206.39` | `275.94` | `39.00 ms` (`18.9%` / `100.0%`) | `75.75 ms` (`27.5%` / `100.0%`) | `39.00` | `75.75` | `1.9422×` |
