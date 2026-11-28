# Re-pricing `low-thread-open-unification` (`t = 1..3` `OpenCells`) on `#1312` Bounded Slabs and Composed `v0.5.19` Stack

*28 November 2026. Smooth-zone re-pricing note & pre-build verification for `frontier/claims/low-thread-open-unification.toml`. Local pre-build committed strictly locally on `build/low-thread-open-on-1312` (`34930ba7` on `701366e1`, not pushed while `#1282` holds `src/parallel.rs`). Every measured number below comes from fresh-process 3-arm alternating paired runs (`n = 9` rounds) on the cloud Xeon VM (`2.30 GHz` Haswell-EP, 2 vCPU) under both portable `base` and `JCC-32B` (`-x86-align-branch-boundary=32 -x86-align-branch=fused+jcc+jmp`), with `126/126` `AB_SHA256=1` C `libbzip2` parity.*

## 1. Executive Summary

Routing `t = 1, 2, 3` (`LAZY_FROM_THREADS = 1`, `UNPINNED_LAZY_FROM = 1`, `SMT_CAP_MIN_CORES = 8` preserved for `plan_with`) and calling-thread entry points (`BzReader::new`, `decode`, `decode_blocks`) from `Cells`/`EagerCells` onto unified `OpenCells` on top of `#1312`'s bounded 80 KiB four-lane forward walk delivers **large, immediate CPU and memory wins on `wikidata` across every low thread count (`1T, 2T, 3T`)** — and composing with the `v0.5.19` worker stack (`#1328` zero-RSS `A6b` + `default-a5a6a-fast` + `#1311` fused `Segmented Runs`) eliminates the only minor `text/3T` watch item and turns guard corpora into strict wins:

- **Arm A (`#1312` bounded slabs alone, shippable right after `#1324`):**
  - `wikidata` `1T / 2T / 3T` (`JCC-32B` CPU): **`0.8125×` (`−18.8%`) / `0.7411×` (`−25.9%`) / `0.7114×` (`−28.9%`)** (`base` CPU: `0.8274× / 0.7812× / 0.7619×`).
  - `wikidata` `1T / 2T / 3T` Peak RSS (`VmHWM`): **`27.1 → 15.1 MiB` (`0.556×`) / `51.7 → 28.9 MiB` (`0.559×`) / `68.8 → 32.9 MiB` (`0.476×`, `−52.4%`)**.
  - Guards: `text/1T` (`0.9907×` JCC), `text/2T` (`1.0183×` JCC), `runs/1T..3T` (`0.9412–1.0209×` JCC), `random`, `records`, `pattern`, `onebyte` all clean; **one minor watch item on `text/3T`** (`1.0431×` base / `1.0513×` JCC CPU, 95% CI crossing `1.000`: `[0.995, 1.111]`).
- **Arm B (composed `v0.5.19` stack: `#1312` + `#1328` `0e044745` + `default-a5a6a-fast` `f451359d` + `#1311` `df4f0bbd`):**
  - `wikidata` `1T / 2T / 3T` (`JCC-32B` CPU): **`0.7113×` (`−28.9%`) / `0.6731×` (`−32.7%`) / `0.6577×` (`−34.2%`)** (`base` CPU: `0.7255× / 0.7071× / 0.6985×`), at **`0.489–0.563×`** peak RSS.
  - **Every guard watch eliminated + strict wins across guard corpora (`JCC-32B` CPU):** `text/1T` **`0.9490×`** `[0.920, 0.979]` (`−5.1%`, RSS `0.956×`), `text/2T` **`0.9326×`** `[0.896, 0.971]` (`−6.7%`, RSS `0.969×`), `text/3T` **`0.9881×`** (`base` `0.9631×`), `runs/2T` **`0.8264×`** (`−17.4%`), `runs/3T` **`0.8735×`** (`−12.7%`), `onebyte/2T` **`0.8566×`** (`−14.3%`), `onebyte/3T` **`0.8775×`** (`−12.3%`), `pattern/1T` **`0.9354×`** (`−6.5%`), `random/1T` **`0.9635×`** (`−3.7%`), and `runs/1T` neutral (`0.9756×` base / `1.0059×` JCC). **0/21 regressions.**

## 2. Paired 3-Arm Benchmark Tables (`n = 9`, All 7 Corpora × `t = 1, 2, 3`)

All three arms (`ctrl` = `701366e1` + `--features forward-anchor-lanes` running shipped `Cells`/`EagerCells` at `t = 1..3`; `Arm A` = `34930ba7` + `--features forward-anchor-lanes`; `Arm B` = `34930ba7` + `#1328` `0e044745` + `default-a5a6a-fast` `f451359d` + `#1311` `df4f0bbd` + `--features forward-anchor-lanes`) were measured in the same 3-arm alternating `ab run` rounds (`--rounds 9 --settle 0`, distinct verified binary SHA-256s).

- **Parity (`AB_SHA256=1`):** **126/126 PASS** (`7` corpora × `t = 1, 2, 3` × `6` binaries, 100% C `libbzip2` SHA-256 match).
- **Binary identity (`nm -p`/`nm -pC` + `objdump -d`):** `<mibz2::open::lanes::Walk>::run::<4>` verified non-empty (`490` instructions) in both `ctrl` and `Arm A`.

### 2a. Portable `base` x86-64 (`cand / ctrl`, `n = 9` paired rounds)

| Cell | `ctrl` CPU | `ctrl` RSS | Arm A CPU `[95% CI]` | Arm A Wall | Arm A RSS (ratio) | Arm B CPU `[95% CI]` | Arm B Wall | Arm B RSS (ratio) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **`wikidata/1T`** | `1.125s` | `27.1 MiB` | **`0.8274`** `[0.782, 0.875]` | `0.9142` | `15.1 MiB` (`0.556`) | **`0.7255`** `[0.703, 0.748]` | `0.7395` | `15.1 MiB` (`0.558`) |
| **`wikidata/2T`** | `1.187s` | `51.7 MiB` | **`0.7812`** `[0.749, 0.815]` | `0.7915` | `28.9 MiB` (`0.559`) | **`0.7071`** `[0.693, 0.722]` | `0.7080` | `29.1 MiB` (`0.563`) |
| **`wikidata/3T`** | `1.227s` | `68.8 MiB` | **`0.7619`** `[0.742, 0.782]` | `0.7551` | `32.9 MiB` (`0.476`) | **`0.6985`** `[0.675, 0.722]` | `0.7119` | `33.9 MiB` (`0.489`) |
| `text/1T` | `0.277s` | `23.5 MiB` | `1.0113` `[0.957, 1.069]` | `0.9854` | `23.4 MiB` (`0.996`) | `0.9824` `[0.938, 1.029]` | `1.0402` | `22.5 MiB` (`0.958`) |
| `text/2T` | `0.309s` | `40.4 MiB` | `0.9850` `[0.915, 1.061]` | `0.9397` | `39.7 MiB` (`0.991`) | `0.9449` `[0.852, 1.048]` | `0.9474` | `38.6 MiB` (`0.970`) |
| `text/3T` | `0.333s` | `52.5 MiB` | `1.0431` `[0.978, 1.113]` | `1.0560` | `53.7 MiB` (`0.999`) | `0.9631` `[0.912, 1.017]` | `0.9842` | `53.1 MiB` (`1.035`) |
| `records/1T` | `0.514s` | `23.0 MiB` | `0.9995` `[0.960, 1.040]` | `0.9895` | `22.9 MiB` (`0.995`) | **`0.9693`** `[0.948, 0.991]` | `0.9510` | `22.8 MiB` (`0.993`) |
| `records/2T` | `0.562s` | `49.9 MiB` | `0.9796` `[0.929, 1.033]` | `1.0084` | `49.8 MiB` (`0.996`) | `0.9753` `[0.937, 1.015]` | `0.9653` | `49.6 MiB` (`1.006`) |
| `records/3T` | `0.561s` | `55.1 MiB` | `0.9912` `[0.965, 1.018]` | `0.9874` | `55.3 MiB` (`1.002`) | `0.9959` `[0.959, 1.034]` | `0.9634` | `55.2 MiB` (`1.001`) |
| `random/1T` | `0.459s` | `22.7 MiB` | **`0.9574`** `[0.923, 0.993]` | `0.8420` | `22.6 MiB` (`0.992`) | `0.9614` `[0.911, 1.014]` | `0.8713` | `22.6 MiB` (`0.997`) |
| `random/2T` | `0.497s` | `47.2 MiB` | `0.9888` `[0.913, 1.071]` | `0.9504` | `47.2 MiB` (`1.000`) | **`0.9606`** `[0.934, 0.988]` | `0.9262` | `47.3 MiB` (`1.001`) |
| `random/3T` | `0.492s` | `53.2 MiB` | **`0.9708`** `[0.945, 0.997]` | `0.9633` | `53.4 MiB` (`0.991`) | **`0.9723`** `[0.949, 0.996]` | `0.9499` | `53.3 MiB` (`0.995`) |
| `runs/1T` | `0.051s` | `33.7 MiB` | `1.0132` `[0.958, 1.072]` | `1.0282` | `33.7 MiB` (`1.000`) | `0.9756` `[0.932, 1.022]` | `1.0289` | `33.3 MiB` (`0.987`) |
| `runs/2T` | `0.055s` | `33.2 MiB` | `0.9814` `[0.914, 1.054]` | `0.9536` | `30.9 MiB` (`0.941`) | **`0.8285`** `[0.759, 0.904]` | `0.7951` | `30.2 MiB` (`0.911`) |
| `runs/3T` | `0.057s` | `31.1 MiB` | `1.0167` `[0.960, 1.077]` | `1.1203` | `32.3 MiB` (`0.996`) | **`0.8361`** `[0.782, 0.894]` | `0.8811` | `30.3 MiB` (`0.973`) |
| `pattern/1T` | `0.211s` | `15.2 MiB` | `1.0175` `[0.932, 1.110]` | `1.0811` | `15.4 MiB` (`1.008`) | `0.9785` `[0.891, 1.075]` | `0.9316` | `14.4 MiB` (`0.948`) |
| `pattern/2T` | `0.228s` | `20.5 MiB` | `0.9618` `[0.906, 1.021]` | `0.9698` | `19.8 MiB` (`0.964`) | `0.9727` `[0.900, 1.052]` | `0.9745` | `20.5 MiB` (`1.002`) |
| `pattern/3T` | `0.230s` | `28.4 MiB` | `1.0513` `[1.008, 1.097]` | `1.0881` | `28.4 MiB` (`1.001`) | `1.0220` `[0.989, 1.056]` | `1.0558` | `28.6 MiB` (`1.000`) |
| `onebyte/1T` | `0.080s` | `54.3 MiB` | `1.0276` `[0.964, 1.095]` | `1.0569` | `54.3 MiB` (`1.001`) | `0.9859` `[0.920, 1.056]` | `0.9472` | `53.6 MiB` (`0.987`) |
| `onebyte/2T` | `0.086s` | `51.1 MiB` | `0.9883` `[0.925, 1.056]` | `1.0249` | `51.2 MiB` (`1.002`) | **`0.8789`** `[0.806, 0.959]` | `0.9641` | `50.9 MiB` (`0.995`) |
| `onebyte/3T` | `0.090s` | `53.9 MiB` | **`0.9459`** `[0.906, 0.988]` | `1.0293` | `51.1 MiB` (`0.974`) | **`0.8508`** `[0.767, 0.944]` | `0.8710` | `50.9 MiB` (`0.944`) |

### 2b. Branch-Aligned `JCC-32B` VM Control (`cand / ctrl`, `n = 9` paired rounds)

| Cell | `ctrl` CPU | `ctrl` RSS | Arm A CPU `[95% CI]` | Arm A Wall | Arm A RSS (ratio) | Arm B CPU `[95% CI]` | Arm B Wall | Arm B RSS (ratio) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **`wikidata/1T`** | `1.142s` | `27.1 MiB` | **`0.8125`** `[0.741, 0.891]` | `0.8411` | `15.0 MiB` (`0.554`) | **`0.7113`** `[0.685, 0.739]` | `0.7136` | `15.2 MiB` (`0.560`) |
| **`wikidata/2T`** | `1.207s` | `51.6 MiB` | **`0.7411`** `[0.713, 0.770]` | `0.7133` | `28.9 MiB` (`0.559`) | **`0.6731`** `[0.656, 0.691]` | `0.6508` | `29.0 MiB` (`0.563`) |
| **`wikidata/3T`** | `1.217s` | `68.7 MiB` | **`0.7114`** `[0.667, 0.759]` | `0.7585` | `33.2 MiB` (`0.483`) | **`0.6577`** `[0.640, 0.676]` | `0.6706` | `34.0 MiB` (`0.495`) |
| `text/1T` | `0.280s` | `23.4 MiB` | `0.9907` `[0.943, 1.041]` | `0.9589` | `23.3 MiB` (`0.994`) | **`0.9490`** `[0.920, 0.979]` | `0.8546` | `22.4 MiB` (`0.956`) |
| `text/2T` | `0.311s` | `41.2 MiB` | `1.0183` `[0.986, 1.052]` | `0.9903` | `40.6 MiB` (`0.991`) | **`0.9326`** `[0.896, 0.971]` | `0.8659` | `39.2 MiB` (`0.969`) |
| `text/3T` | `0.324s` | `54.1 MiB` | `1.0513` `[0.995, 1.111]` | `1.0542` | `54.1 MiB` (`1.012`) | `0.9881` `[0.940, 1.038]` | `1.0176` | `54.8 MiB` (`1.001`) |
| `records/1T` | `0.502s` | `23.0 MiB` | `1.0109` `[0.983, 1.040]` | `1.0561` | `22.7 MiB` (`0.991`) | `1.0087` `[0.984, 1.035]` | `1.0069` | `22.7 MiB` (`0.991`) |
| `records/2T` | `0.548s` | `50.1 MiB` | `0.9980` `[0.967, 1.030]` | `1.0066` | `49.8 MiB` (`0.994`) | `1.0072` `[0.960, 1.056]` | `0.9919` | `50.0 MiB` (`0.993`) |
| `records/3T` | `0.549s` | `55.0 MiB` | `1.0247` `[0.995, 1.056]` | `1.0309` | `55.1 MiB` (`1.009`) | `1.0127` `[0.998, 1.027]` | `1.0463` | `55.2 MiB` (`1.001`) |
| `random/1T` | `0.445s` | `22.7 MiB` | `0.9891` `[0.953, 1.027]` | `1.0344` | `22.5 MiB` (`0.990`) | **`0.9635`** `[0.937, 0.991]` | `0.9839` | `22.5 MiB` (`0.990`) |
| `random/2T` | `0.475s` | `47.1 MiB` | `0.9523` `[0.875, 1.036]` | `0.9170` | `47.3 MiB` (`1.005`) | `0.9661` `[0.919, 1.016]` | `0.9448` | `47.2 MiB` (`1.003`) |
| `random/3T` | `0.475s` | `52.5 MiB` | `1.0148` `[0.967, 1.065]` | `1.0601` | `53.3 MiB` (`1.016`) | `1.0383` `[0.948, 1.137]` | `1.1114` | `53.2 MiB` (`1.005`) |
| `runs/1T` | `0.052s` | `33.6 MiB` | `1.0209` `[0.946, 1.102]` | `0.9457` | `33.6 MiB` (`0.999`) | `1.0059` `[0.950, 1.066]` | `0.9371` | `33.2 MiB` (`0.986`) |
| `runs/2T` | `0.057s` | `32.4 MiB` | **`0.9412`** `[0.895, 0.990]` | `0.9639` | `30.8 MiB` (`0.998`) | **`0.8264`** `[0.777, 0.879]` | `0.7224` | `30.3 MiB` (`0.937`) |
| `runs/3T` | `0.056s` | `31.0 MiB` | `0.9788` `[0.922, 1.039]` | `1.1098` | `30.9 MiB` (`0.996`) | **`0.8735`** `[0.828, 0.922]` | `0.9527` | `30.2 MiB` (`0.973`) |
| `pattern/1T` | `0.225s` | `15.2 MiB` | `0.9702` `[0.930, 1.013]` | `0.9702` | `15.1 MiB` (`0.994`) | **`0.9354`** `[0.906, 0.966]` | `0.8427` | `14.4 MiB` (`0.946`) |
| `pattern/2T` | `0.226s` | `20.5 MiB` | `0.9930` `[0.956, 1.032]` | `1.0119` | `20.6 MiB` (`1.003`) | `0.9914` `[0.934, 1.052]` | `0.9931` | `20.5 MiB` (`1.006`) |
| `pattern/3T` | `0.238s` | `28.4 MiB` | `1.0070` `[0.959, 1.058]` | `0.9849` | `28.4 MiB` (`1.005`) | `1.0080` `[0.961, 1.058]` | `1.0012` | `28.6 MiB` (`1.030`) |
| `onebyte/1T` | `0.083s` | `54.2 MiB` | `1.0166` `[0.907, 1.139]` | `0.9618` | `54.1 MiB` (`0.998`) | `0.9736` `[0.897, 1.057]` | `0.9666` | `53.5 MiB` (`0.986`) |
| `onebyte/2T` | `0.088s` | `52.7 MiB` | `1.0006` `[0.949, 1.055]` | `1.0110` | `53.3 MiB` (`1.000`) | **`0.8566`** `[0.811, 0.904]` | `0.8108` | `50.8 MiB` (`0.967`) |
| `onebyte/3T` | `0.087s` | `51.1 MiB` | `1.0301` `[0.948, 1.120]` | `1.0722` | `51.1 MiB` (`1.000`) | **`0.8775`** `[0.848, 0.908]` | `1.0327` | `50.8 MiB` (`0.995`) |

## 3. Calibrated Physical i9-10900K Projection (**DERIVED**)

Applying the calibrated transfer ledger (`docs/notebook/2026-11-28-vm-i9-transfer-ledger.md`) to the `JCC-32B` VM deltas:

- **Dominant mechanism (`wikidata` `1T..3T`):** replacing `Cells` (including its 8.6 MiB/worker `Segmented` fallback) with `OpenCells` + `#1312` bounded 80 KiB four-lane forward emission combines **MLP latency hiding** (`1.37–1.85×` transfer prior) with **−44%..−52% resident memory** (already measured on physical i9 at `1W`: `#1312` beat shipped `OpenCells` by `−17.10%` in `1950f77f`, overturning `v0.5.14`'s `+12.3%` `1T` gap cleanly).
- **Derived i9 `wikidata` `1T / 2T / 3T` expectation:**
  - **Arm A (post-`#1324`):** `~−10% to −25%` wall reduction across `1T..3T`, with peak RSS `~0.48–0.56×` (well under the `<= 0.900×` stop-test gate).
  - **Arm B (composed `v0.5.19` stack):** `~−15% to −30%` wall reduction across `1T..3T` (adding `#1328` + `default-a5a6a-fast` compute savings discounted at `0.45×` plus `#1311`'s `Segmented` wins on `runs`/`onebyte`/`text`), with zero guard risk.
- **Sequencing takeaway:** Arm A is already ready to queue immediately after `#1324` promotes `#1312` to default (updating `BASE` in `bench/low_thread_open_i9.py`), while landing `#1328`, `default-a5a6a-fast`, and `#1311` alongside/before it turns `text/1T..3T` from a neutral/slight watch into a clean win (`−5.1%` / `−6.7%` / `−1.2%` VM `JCC-32B`).

## 4. `EagerCells` / `Seq(Cells)` Retirement Audit

Once `t = 1..3` and calling-thread defaults move onto `OpenCells`, **zero default production paths** (`decode`, `decode_blocks`, `decode_parallel`, `BzReader::new`, `StreamDecoder::new`, `StreamDecoder::planned`) instantiate `bwt::Cells` (`CellsKernel`) or `bwt::LazyCells` (`CellsVariant`):

- **Production dependency of `OpenCells` on `src/cells*.rs`:** **only `crate::cells::lazy_counts_admit(records, n)`** (~20 lines in `src/cells_lazy.rs`), easily relocated into `src/open.rs`.
- **Total `src/cells*.rs` code becoming unreachable from default paths (`5,154` LOC across 11 files):**
  - `src/cells.rs`: `1,732` LOC (`Cells` / `CellsKernel`, eager slot table + writer + tests)
  - `src/cells_lazy.rs` + `src/cells_lazy_induce.rs`: `793` LOC (`LazyCells` / `CellsVariant`)
  - `src/cells_lever.rs`, `src/cells_w12.rs`, `src/cells_owned.rs`, `src/cells_replay.rs`, and `src/cells_*_test.rs`: `2,629` LOC (research variants & unit tests)
- **Routing simplification in `src/parallel.rs`:** `LAZY_FROM_THREADS` / `UNPINNED_LAZY_FROM` / `StreamDecoder::workers(..., lazy: bool)` duality and ~80–120 lines of `Cells`-vs-`OpenCells` routing/test scaffolding become obsolete once `Cells` is retired.
- **Semver constraint:** Because `bwt::Cells`, `bwt::LazyCells`, `bwt::CellsKernel`, `bwt::CellsVariant`, `Scratch::cells_counts`, and `StreamDecoder::workers(..., lazy)` are `pub` items in `0.5.x`, `0.5.19` keeps them compiling (zero breaking changes, immediately dropping `Cells` from every default monomorphization and runtime path), and full removal of `src/cells*.rs` (~5,130 LOC) is queued cleanly for `v0.6.0`.

## 5. Pre-Build & C6 Recipe Verification (`build/low-thread-open-on-1312` at `34930ba7`)

- **Clippy:** `cargo clippy --all-targets -- -D warnings` (default and `--features forward-anchor-lanes`) — **0 warnings**.
- **Tests (`--features forward-anchor-lanes`):** unit tests PASS (`parallel::plan_tests::*`, `read::tests::*`, `open::lanes::*`, `runs::tests::miri_runs_synthetic_blocks`), plus all **10 integration suites (`59/59` PASS, `1` ignored):** `cells` (5), `cells_default` (1+1 ign), `in_place` (7), `corpus` (2), `lazy_admission` (3), `lazy_workers` (1), `service` (10), `service_default` (4), `decode` (14), `pipelined` (12).
- **Freeze-ready C6 i9 recipe:** `bench/low_thread_open_i9.py`, `bench/test_low_thread_open_i9.py` (**12/12 PASS**, `ruff check`/`format` clean), `.just/low-thread-open-i9.just`, and `docs/journal/2026-11-28-low-thread-open-unification/registration.md`. Verified live `--smoke` + `score` (`VOID: not the registered run; diagnostic only: VOID: host is not unrestricted i9-10900K`) + `bash bench/bundle.sh` (`16K` `.tar.gz`), `check_source(require_clean=True)` (`dirty: false`, exact 3-file decoder diff), and `frontier.py check` (`0 errors, 0 warnings`).
