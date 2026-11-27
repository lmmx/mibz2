# Pre-data registration: 8-worker knee on a pinned text panel vs wikidata (`text-knee-i9`)

- **Claim:** `frontier/claims/text-knee-i9.toml` (P2, `probe`, `striated`)
- **Date:** 27 November 2026
- **Premises:**
  - `docs/journal/2026-11-27-explore/station-balance-off-wikidata.md` (`#1075`)
  - `docs/journal/2026-11-27-explore/route-census.md`
  - `docs/journal/2026-11-27-synthesis/twelfth-pass.md` (`#1109`)
  - `docs/journal/2026-11-27-synthesis/thirteenth-pass.md` (`#1133`)
- **Recipe:** `just bench-text-knee` (`.just/text-knee.just` → `bench/text-knee.sh` → `bench/text_knee.py`)
- **Status:** Frozen before any i9 measurement (C2 / C6).

## 1. Frozen stop test (verbatim from `frontier/claims/text-knee-i9.toml`)

> Frozen before data: on the i9, for the panel's median corpus, the measured knee (the smallest k whose median wall is within 2% of the best wall over k = 4..10, 9 interleaved rounds) is <= 7 while wikidata's is >= 8: CONFIRMED, and DECODER.md gains a text row. If the panel's knee is >= 8, the VM ratio does not transfer: REFUTED, recorded as a dead-end row for "VM station ratio as an i9 knee predictor". Otherwise INCONCLUSIVE, with the measured k* (W/S) on the i9 reported.

## 2. Question and motivation

On a 4-vCPU VM at one thread (`#1075`), the worker:scanner demand ratio $k^* = W/S$ off wikidata was $0.58\text{--}0.85\times$ wikidata's (median $r = 0.64$). Scaling from the shipped i9 wikidata knee of $\approx 8$ predicts typical text balances at $k^* \approx 4.6\text{--}6.8$, making text scanner-bound at 8 workers. That VM transfer assumes L3 contention across contending i9 workers inflates $W$ similarly on text and on wikidata. This probe measures the shipped decoder's actual multi-worker wall knee across $k \in \{4, 5, 6, 7, 8, 9, 10\}$ on the i9-10900K alongside single-thread `examples/station_balance.rs` ($S$, $W$, $k^* = W/S$) on the same machine.

## 3. Pinned corpora (SHA-256 verified before measurement)

### 3.1 Text panel from pinned commit `2fed5f184bf66ddaa50c72779a126dafe7811eef`

The three text corpora are built deterministically from repository commit `2fed5f184bf66ddaa50c72779a126dafe7811eef` (shallow-fetched into a private temporary repository via `git init` + `git fetch --depth=1 --no-tags <origin> <commit>` when not present in a shallow checkout). For each corpus, tracked paths matching the glob are sorted by UTF-8 path order and concatenated into a unit byte stream.

Because `repo_json` (1.90 MB) and `src_rs` (1.53 MB) each exceed one 900 KB bzip2 block but contain only 2–3 raw blocks at 1×, running $k = 4\text{--}10$ workers on 2–3 blocks would measure block-count starvation rather than station balance. Therefore:
1. Each unit stream is repeated `raw_rep` times before `bzip2 -9` (Python `bz2.compress(..., compresslevel=9)`, verified bit-identical to `bzip2 -9 -c`) so every base stream contains at least 10 full 900 KB blocks (`unit > 900 KB`, so no repeated slice ever falls inside the same 900 KB BWT block).
2. Each compressed base stream is then multistream-concatenated `stream_rep` times (the same multistream construction used by `wikidata x4`) so every wall corpus contains 192–200 full 900 KB blocks (~183–190 MB uncompressed, ~35–47 MB compressed, matching `wikidata x4`'s steady-state timing scale so the 2% knee threshold sits cleanly above process-startup and scheduling jitter).

| Corpus | Glob (`2fed5f18`) | Files | Unit raw (B) | `raw_rep` | Base raw (B) | Base `.bz2` (B) | Ratio | `stream_rep` | Wall raw (B) | Wall `.bz2` (B) | Full blocks (wall) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `docs_md` | `docs/**/*.md` | 656 | 11,480,438 | 1 | 11,480,438 | 2,952,993 | 3.89× | 16 | 183,687,008 | 47,247,888 | 208 |
| `repo_json` | `**/*.json` | 181 | 1,897,118 | 5 | 9,485,590 | 1,933,754 | 4.91× | 20 | 189,711,800 | 38,675,080 | 220 |
| `src_rs` | `src/**/*.rs` | 52 | 1,525,862 | 6 | 9,155,172 | 1,757,562 | 5.21× | 20 | 183,103,440 | 35,151,240 | 180 |

Pinned SHA-256 digests verified by `bench/text_knee.py`:
- `docs_md`:
  - `base_bz2`: `ae6b9f59478b92f306e394eb1a8b36c45a20cd15dfbdf24fcd5ee0eebc5b2c8a`
  - `wall_bz2`: `857095a4bd46e580c876ae20c95192176ba6a9ec7e32c3a5c68bbed5d8ee188d`
  - `wall_raw`: `ef330c7a80dc148c75abdb0ce8c8ce110f6c38d4ee02ee092c1c04483902f8ac`
- `repo_json`:
  - `base_bz2`: `d87064ec66834ff4733fe0f62644111a5259c3e64f07b1a459957804e357f657`
  - `wall_bz2`: `83fd2eb4c81fbb6f5cda058c51a8a5f954f3eeaf39d23e5e6b6855e4bfa7b523`
  - `wall_raw`: `2778e69b4d0b5f65b3f49cc4c668c41b5e60950d52b9ce7ed17e5ce19398fea6`
- `src_rs`:
  - `base_bz2`: `dd3329e28b2e2a1c4abfbac743a574ae039bfe9f13988312e13ecd761dcbc05b`
  - `wall_bz2`: `6e7e6bb4ede8d1662976cbf364524db691d311ff1885c3634b520cc96d558ea1`
  - `wall_raw`: `a0486cee34ba6c74823c336024a41189ccf41ba4fb257db593c60e5e2c7285b8`

### 3.2 Control corpus (`wikidata`)

`corpora/wikidata.bz2` (built from `bench/wikidata.json` / `origin/bench-data` via `examples/windows.rs`, verified SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 431 total blocks / 233 full blocks, 209,700,196 B uncompressed, 11,854,933 B compressed, 17.69×) is used as the base stream for `station_balance`, and concatenated 4× (`wikidata x4`, 932 full blocks, 838,800,784 B uncompressed, 47,419,732 B compressed, SHA-256 `0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e`) for wall timing so the 2% knee threshold (~4 ms at ~200 ms wall) is not swamped by process-startup overhead.

## 4. Machine, build, and runtime protocol

- **Host:** Physical Intel Core i9-10900K (`10` physical cores, `20` logical CPUs, full unrestricted affinity, unlimited cgroup CPU quota).
- **Toolchain and codegen:** Pinned `rustc 1.99.1`, `cargo build --release --locked` directly on clean `HEAD` into a private `CARGO_TARGET_DIR`, with **default baseline x86-64 codegen** (`RUSTFLAGS`, `CARGO_BUILD_RUSTFLAGS`, and `CARGO_ENCODED_RUSTFLAGS` must be unset/empty, per `#1109`). No `-C target-cpu` or branch-alignment flags are added.
- **Shipped decoder arm:** `ab`'s `mibz2` arm (`BzReader::parallel(file, k)`), unpinned across all 20 i9 CPUs, at $k \in \{4, 5, 6, 7, 8, 9, 10\}$. Note that on the unpinned i9-10900K, `StreamDecoder::new`'s `unpinned_cell` runs $k$ lazy workers for $k \in \{4, 5, 6, 7, 8, 9\}$ and trims $k = 10$ to `UNPINNED_FROM_TEN = 8` lazy workers, matching the shipped decoder behaviour.
- **Correctness parity (untimed, pre-sweep):** For each of the 4 wall corpora, verify byte count and pinned decompressed SHA-256 via independent Python `bz2.open` (libbzip2), run `ab child libbzip2 1 <wall.bz2>` to obtain C `libbzip2 1.0.8`'s exact `(out, hash)`, and verify `ab child mibz2 8 <wall.bz2>` matches C `libbzip2`'s exact `(out, hash)` with `err == ""` and `termination == "success"`.
- **Wall sweep:** `ROUNDS = 9` interleaved rounds. Each round shuffles all $4 \times 7 = 28$ `(corpus, k)` cells using deterministic seed `1135` and executes `ab run <cell.jsonl> --arms mibz2 --threads <k> --rounds 1 --prespin-ms 100 --settle 0.2 --max-busy 1.5 --sysfs-state <wall.bz2>`, validating `err == ""`, `termination == "success"`, exact `out`, exact `hash`, positive finite `wall`, and positive `rss_kb`.
- **Single-thread station demand (`examples/station_balance.rs`):** `STATION_ROUNDS = 3` interleaved rounds (shuffled across the 4 base streams), pinned to physical core 0's first CPU (`taskset -c <cpu0> station_balance <base.bz2> 5`), recording per-corpus median `S_ms`, `W_ms`, `k* = W/S`, `r = k* / k*_wikidata`, `ns/sym`, and `ns/out_byte`.
- **Estimated workstation runtime:** ~4–5 minutes (well under the 15-minute workstation budget).

## 5. Frozen scoring and verdict rules

1. For each corpus $c \in \{\texttt{docs\_md}, \texttt{repo\_json}, \texttt{src\_rs}, \texttt{wikidata}\}$ and each $k \in \{4, 5, 6, 7, 8, 9, 10\}$, compute $\text{med}(c, k) = \text{median}(\text{wall}_{c,k,0..8})$ over the 9 rounds.
2. Let $\text{best}(c) = \min_{k \in 4..10} \text{med}(c, k)$. The measured knee is
   $$\text{knee}(c) = \min \{ k \in \{4,\dots,10\} : \text{med}(c, k) \le 1.02 \times \text{best}(c) \}.$$
3. Let $\text{knee}_{\text{panel}} = \text{median}(\text{knee}(\texttt{docs\_md}), \text{knee}(\texttt{repo\_json}), \text{knee}(\texttt{src\_rs}))$ (exact integer, middle of three sorted values) and $\text{knee}_{\text{wiki}} = \text{knee}(\texttt{wikidata})$.
4. **Verdict:**
   - **VOID** if any prerequisite fails (non-i9-10900K host, restricted affinity or CPU quota, wrong `rustc`, dirty tracked checkout, non-empty `RUSTFLAGS`, panel/wikidata SHA-256 mismatch, `AB_SHA256=1` parity failure, any failed/errored/missing/duplicate timed wall row, or missing/invalid `station_balance` row).
   - **CONFIRMED** if $\text{knee}_{\text{panel}} \le 7$ and $\text{knee}_{\text{wiki}} \ge 8$.
   - **REFUTED** if $\text{knee}_{\text{panel}} \ge 8$.
   - **INCONCLUSIVE** otherwise ($\text{knee}_{\text{panel}} \le 7$ and $\text{knee}_{\text{wiki}} \le 7$), reporting the measured i9 $k^* = W/S$ for every corpus.

## 6. Evidence bundle

`bench/text-knee.sh` writes one timestamped run directory under `target/text-knee/run-<UTC>` containing `console.log`, `registration.md`, `provenance.json`, `lscpu.txt`, `lscpu-e.txt`, `env.json`, `binaries.json`, `inputs.json`, `parity.jsonl`, `station_balance.jsonl`, `runs.jsonl`, `summary.json`, `outcome.txt`, and `toolchain.txt`, packed into one tarball via `bash bench/bundle.sh`.
