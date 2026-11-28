# Synthesis, twenty-fourth pass: `0.5.18` composition, the post-`#1217` worker/scanner balance, the 32 KiB L1d trap, and `0390e19f` forensics

*28 Nov 2026, master `3d98e71a`, `origin/evidence 0390e19f`. Claim `synthesise` (continuing from `synth-23` at `757a91b7`). No new runs and no edits to claims held by other sessions. Every figure carries its evidence scope (physical i9-10900K vs VM vs derived); C2 holds throughout.*

Merged or returned since `synth-23` (`757a91b7`): `#1299` (`synth-23`), `#1300` (`result-to-release` marked `done`), `#1301` (`scanner-contention-residual` admitted; recipe now open in `#1304`), `#1294` (`a6c-wire-redesign` merged on `src/open.rs` + `src/open/outq.rs`), `#1305` (`explore-composed-worker-stack` 4-way `B0..B4` VM ablation + `B4` stage dissection), `#1306` (`scanner-two-lane-interleave-step1` `B = 10` and `B = 9` Step 1 probe on 32 KiB L1d Haswell-EP VM), and two physical i9-10900K evidence bundles: `origin/evidence 95edea2a` (`runs-front-i9/run-20261128T030132Z`, **`PASS: rules 1-4 hold`**) and `origin/evidence 0390e19f` (`crc32-ring-rescue/run-20261128T040117Z`, **`INCOMPLETE (exit 1)`**).

## 1. Coupled station ledger & `0.5.18` candidate composition

Four `src/` commits have already landed on `master` since `v0.5.17`, and a fifth (`#1274`) now has a passing physical i9 adoption bundle (`95edea2a`, currently held for formal verdict readback and merge):

| Change | Scope / status | Station & mechanism | Evidence (with scope) |
|---|---|---|---|
| `#1234` (`faa976e5`) | Merged on `master` (`src/open/outq.rs`) | `C-A6a`: clears native `push_slow` `vzeroupper` (`11 → 0`) | VM + `objdump` hygiene (`0` UB Miri) |
| `#1246` (`91ab4c8d`) | Merged on `master` (`src/open.rs`) | `C-A6a`: skips redundant `within_cap` pre-scan on `[0.10, 0.15)` lazy-admitted blocks | VM recovery `-4.12%` 1T / `-2.93%` 8T CPU (`489/489` route-identical); directly removes `v0.5.17`'s `5182d546` i9 `+2.63%` 8T / `+1.52%` 10T regression |
| `#1117` (`15c289b5`) | Merged on `master` (`src/block.rs`, `src/huff.rs`) | `S` (`A2`): direct selector runs, multi-length code-length LUT, single-write table build | **Physical i9 PASS (`9dacc085`):** `phases/wikidata/1 -4.52%` (`142.8 → 136.3 ms`, `-6.5 ms`), 5/5 guard scans clean, `RSS <= 1.0068×` |
| `#1294` (`0e9b59ff`) | Merged on `master` (`src/open.rs`, `src/open/outq.rs`) | `C-A6c`: `#![forbid(unsafe_code)]` `Cell` 4× `wire_xor_edges` + register-cursor overlapping `wire_fwd_edges` (`0 B` extra alloc) | VM `mibz2-workers-lazy`: `wikidata` `-3.46%` 1W / `-1.66%` 8W CPU, `wikidata x4` `-4.10%` / `-5.03%` CPU, `text` `-1.79%` 1W CPU, `RSS 1.001–1.002×`; CI bench neutral (21/21 exact) |
| `#1274` (`1bb555c3`) | Open PR (`runs-front-i9-rescue`, currently held) | `M`: zero-`vpermq` 64-sym `u64 vptest` Pass 1, hoisted 2-cycle XMM `STEPS`, `[u32; 256]` stack counts | **Physical i9 PASS (`95edea2a`, SHA-256 `51dd2d6d...`, 16 rounds, 2s settle):** `wikidata/6` **`0.9892 [0.9853, 0.9931]`** (disjoint faster, satisfies Rule 1), `wikidata/8` `0.9908 [0.9712, 1.0108]`, `wikidata/1` `0.9939 [0.9825, 1.0053]`, `wikidata/10` `1.0015 [0.9880, 1.0152]`, `text/1` `0.9822 [0.9593, 1.0056]`, `text/8` `1.0002 [0.9839, 1.0169]`, 10/10 secondary cells clean, **16/16 RSS `0.9919–1.0012×`** |

**Cross-interaction for `0.5.18` (derived, not yet i9-measured as a combined candidate):**

- Because `0.5.18` ships the default bidirectional `OpenCells` walk (`forward-anchor-lanes` remains opt-in behind `--features` until `forward-emission-bounded-slab-rescue` clears C6), **`#1294`'s `wire_xor_edges` speedup IS live on all 233/233 Wikidata Open blocks in `0.5.18`**, even though `#1305` shows `#1294` becomes near-zero once `forward-anchor-lanes` later retires `wire_xor_edges`.
- Under the shipped i9 `trim_cell` schedule (6 workers at 8T, 7 workers at 10T — worker-bound on Wikidata, scanner-bound on `text`/`records`/`random`), `#1246` + `#1294` (+ `#1274` once merged) cut worker demand directly in the shipped 8T/10T Wikidata cells while `#1117` cuts serial scan `-4.52%` (`-6.5 ms`) at 1–2T and off-Wikidata 8–10T. Against `v0.5.16` (the last known good tag enforced by `#1256`'s `pre-release-i9-gate` corrective-recovery clause) and `v0.5.17`, the combined `0.5.18` candidate is projected to recover the `v0.5.17` 8T/10T loss cleanly and finish net-faster once `#1256` clears independent O2 review and the final candidate passes `bench-release-scaling` on the i9.
- **Note for `forward-emission-bounded-slab-rescue` (updating `synth-23` §3):** `#1294` is now on `master` (and `#1274` is ready to follow). Both speed up the bidirectional base more than or equally to the forward candidate (`#1294` primarily helps `wire_xor_edges`, trimming the 8W margin by ~1 pp as `synth-23` §3 projected), so the bounded-slab rescue's C6 registration MUST record the exact post-`#1294` (and post-`#1274` if merged) base SHA.

## 2. Post-`#1217` worker budget & sequencing (`#1305`)

`docs/notebook/2026-11-28-explore-composed-worker-stack.md` (`#1305`, VM `base` + `jcc`, 38/38 SHA-256 exact) is the first end-to-end 4-layer ablation (`B0` shipped → `B1` `+#1217` → `B2` `+#1294` → `B3` `+L6a` → `B4` `+mtf-fold` wired directly into `L6a`'s `lanes::walk_fused_same` via `fold_runs_with_rle`) and settles three sequencing constraints:

1. **Cumulative post-`#1217` worker gain (VM, `jcc` alignment-controlled):** `B4/B0` reaches **`-20.68%` (1W) / `-18.27%` (4W)** (`-25.49%` / `-26.76%` `base`), and `B4/B1` (on top of `#1217`) reaches **`-11.00%` (1W) / `-7.80%` (4W) / `-10.0%` (8W CPU)**. Inside `B4/B1`, `L6a` contributes `-1.88%` disjoint at `4W jcc` / `-2.9%` at `8W jcc` while *lowering* 8W RSS (`-0.3..-0.9 MiB`), and `mtf-fold` contributes **`-7.80%` (1W) / `-5.85%` (4W) `jcc`**.
2. **Two pre-ship blockers caught before any i9 run:**
   - **Additive 8W RSS stack:** `B1` (`#1217` unbounded `Arena::pool`, `+6.71 MiB` / `1.108×` in i9 `ccfa473f`, `+4.9..+10.3 MiB` VM) and `B4` (`mtf-fold` retaining a dual `~956 KiB/worker` `s.ll` buffer alongside `out`, adding another `+4.2..+5.0 MiB` at 8W) stack to **`1.206–1.270× B0`**. `forward-emission-bounded-slab-rescue` (`P1`, `src/open/lanes.rs` alone, now unblocked for later default-flip too since `#1294` released `src/open.rs`) MUST land first; `mtf-fold` must stop retaining the dual `s.ll` buffer before C6.
   - **`runs.bz2` sub-bound admission catch (`B3 → B4`):** `mtf-fold` regresses `runs.bz2` by `+6.7..+12.7%` (4W) / `+4.3..+5.1%` RSS because blocks with `syms >= 4096` and `n << max_len` have `lits < bound / 10` (`in_dense = false`), bypassing the dense-band bitmap prescan and allocating a `956 KiB` `buf` before failing `cap` (`Outcome::NotOpen`). Extending the pre-allocation bitmap check to sub-bound blocks fixes all 6/6 guards before `mtf-fold` queues.
   - **Mandatory composition wiring:** out of the box, `OpenCellsOf::fold` called `self.open.walk` + `rle::unrle_in_place`, which silently bypassed `L6a`; `fold_runs_with_rle` must wire `lanes::walk_fused_same` directly into `fold_inner`.
3. **Residual `B4` worker stage shares (VM `jcc`, `592.6 ms` total `W`) & cross-link with `#1274` (`95edea2a`):**
   - **`A7` (`lanes::walk_into_arena`, `33.1%` `jcc` / `31.9%` `base`)** + **`A5+A6a` (`fold::front_lazy` + `FoldSink`, `30.0%` `jcc` / `31.1%` `base`)** together are **`63.1%`** of remaining worker CPU (`A6b` `20.3%`, `A8+A9` `10.3%`, `A6c` shrunk to `5.2%`).
   - **Concrete new synergy:** `src/runs/fold.rs` still runs the pre-`#1238` 32-symbol `vpermq` Pass 1. Now that `#1274` (`95edea2a`) has a physical i9 **PASS** on the exact zero-`vpermq` 64-sym `u64 vptest` Pass 1 + 2-cycle XMM `STEPS` kernel in `src/runs.rs`, porting that i9-proven front-end into `src/runs/fold.rs` attacks `B4`'s `#2` stage (`30.0%`) with zero new microarchitectural risk, alongside `forward-lanes-mlp-and-prefetch` (`idea`) on `#1` (`A7`, `33.1%`).

**Recommended post-`0.5.18` worker sequence:** (1) `forward-emission-bounded-slab-rescue` (`P1`, `src/open/lanes.rs`) → (2) `L6a` (`forward-rle1-crc-vectored-build`, streaming RLE1+CRC directly off bounded slabs, net RSS reduction) → (3) `mtf-fold-streaming-admission` (with the `runs.bz2` sub-bound prescan fix, non-retained `s.ll`, and `fold_runs_with_rle` wiring into `L6a`) → (4) `A7` (`forward-lanes-mlp-and-prefetch`) and `A5+A6a` (`fold.rs` Pass 1 port from `#1274`).

## 3. Serial scanner ceiling on 32 KiB L1d (`#1306` + `#1117` + `scanner-a3-k3-pipe-u8` + `scanner-contention-residual`)

`#1305` §6 derives that once bounded `B4` ships, 8-worker Wikidata drops to `~151–156 ms/w`, putting `k = 8` deeply scanner-bound (`k* ≈ 6.55–6.78`) against the shipped scanner (`S ≈ 184.4 ms`). Reaching balanced 8-worker throughput (`~5.36–5.49 GB/s` derived, `k* ≈ 7.91–8.19`) requires cutting the contended 8-worker scanner toward `~152 ms`.

`docs/notebook/2026-11-28-scanner-two-lane-interleave-step1.md` (`#1306`, local prototype `82ddf946` on a 32 KiB L1d / 2-load-port Haswell-EP VM, matching Comet Lake's L1d width) sharpens the scanner roadmap considerably:

- **Inline magic finder is ruled out:** even with a 100%-safe 2 KiB branchless `M0 & M1` 4-byte-stride finder at 3.2–5.1 GB/s, inline scanning adds `+3.0–6.6%` CPU (`wikidata` `0.949× → 0.995×`, `records` `1.027× → 1.077×`). Any speculative two-lane design requires an off-thread finder.
- **The 32 KiB L1d Pareto trap (`B = 10` vs `B = 9`, `HelperThread` CPU, default / JCC-32B, 0 critical-chain spills in `objdump`):**
  - At **`B = 10`** (`12 × 4 KiB = 48 KiB > 32 KiB L1d`): `wikidata` reaches `0.9492× / 0.9504×` (misses `<= 0.92×` by ~3 pp), `text` sits at `1.0127× / 1.0252×` (misses `<= 0.94×` by ~7–8 pp), `records` `1.0270× / 0.9888×`, `random` `0.9291× / 0.9386×`.
  - At **`B = 9`** (`12 × 2 KiB + 4 KiB chunks = 28 KiB < 32 KiB L1d`): on uniform-code corpora with `sym/lk = 1.000` (zero lost `K = 2` pairs), `records` plunges **`-4.7` to `-8.4 pp`** (`0.9434× / 0.9416×`) and `random` plunges **`-4.4` to `-5.9 pp`** (`0.8847× / 0.8801×`), decisively proving Notebook Risk #1 (32 KiB L1d thrashing). Yet on skewed corpora, `B = 9` loses more in extra lookups and slow exits than L1d recovers: `wikidata` `+7.8%` lookups (`0.9880× / 0.9903×`), `text` `+4.6%` lookups and `1.83% → 3.30%` slow exits (`1.0632× / 1.0706×`), `runs` `24×` slow exits (`0.32% → 7.67%`, `1.4820× / 1.4709×`).
  - `scanner-two-lane-interleave` stays at `status = "idea"` (not dropped yet, as `#1117` raised A3's share of S and Comet Lake's 224-entry ROB exceeds Haswell's 192), but it is **strictly behind** single-lane levers that stay inside `6 × 4 KiB = 24 KiB < 32 KiB L1d`.

**Clear highest-ROI scanner sequence (all staying `<= 24 KiB` active LUTs in 32 KiB L1d):**

1. **`#1117` (`scanner-a2-fast-build`)** — already merged on `master` (`-6.5 ms` / `-4.52%` measured i9 `phases/wikidata/1`).
2. **`scanner-a3-k3-pipe-u8`** — single-lane `B = 10, K = 3` single-`u32` `u8`-chunk decode + pipelined 4-step refill (`+488 B` `ScanScratch` = `1.008×`, `-5.6%..-7.6%` paid A3 / `~6.5–8.8 ms` derived off i9 `S`, `-7.9%..-11.6%` on `text`). Immediately claimable once PR `#1303` releases `#1117`'s stale merged hold on `src/block.rs` + `src/huff.rs`.
3. **`scanner-adaptive-seam-composed`** — `#1144`'s `-9.95 ms` traced `in_maint` (`10.36 → 0.41 ms`) and `-4.2..-5.8 MiB` RSS, composed on top of bounded-slab `forward-anchor-lanes` where 8W Wikidata is scanner-bound.
4. **`scanner-contention-residual` (PR `#1304`)** — i9 per-thread PMU counters on the scanner thread at `k = 1, 4, 8` to settle whether the `+6.6%` (`+11.1 ms`) post-lanes clock-adjusted contention residual is L3-miss stalling (justifying a memory-side scanner lever) or `n = 1` noise.

## 4. Read-only forensics on `0390e19f` (`crc32-ring-rescue`) & critical-path review queue

### Root cause of `origin/evidence 0390e19f` (`crc32-ring-rescue/run-20261128T040117Z`, `INCOMPLETE (exit 1)`)

Read-only inspection of `0390e19f` (SHA-256 `44fa7413...`, PR `#1295` at `19902717`, currently held):

- **Kernel build & codegen audit passed:** both `base` (`0179ca9d`, 28.02 s) and `head` (`19902717`, 27.68 s) compiled cleanly on the i9, and `codegen_audit.json` confirmed `mibz2::crc::avx2_ring8_update` in `head` only (`"base": []`, `"head": ["0000000000107600 t mibz2::crc::avx2_ring8_update"]`). Zero defect in the CRC kernel itself.
- **Exact crash site:** `bench/crc32_ring_rescue.py:191` ran `subprocess.run([bins["base", "ab"], "gen", str(gen), "4"], check=True)`, which panicked at `tests/common/mod.rs:416` (`Os { code: 2, kind: NotFound, message: "No such file or directory" }`, exit 101).
- **Exact cause:** `build()`'s `finally:` block (`bench/crc32_ring_rescue.py:147`) ran `git worktree remove --force trees["base"]` immediately after copying `base-ab` out of `tmp/base-src`. `examples/ab.rs gen` calls `tests/common/mod.rs:412-416`, which reads `env!("CARGO_MANIFEST_DIR")/{src,tests,examples}` — baked into `base-ab` at compile time as the just-deleted `.../tmpvf7t_87n/base-src` path.
- **Fix for `#1295`'s holder:** either keep `trees["base"]` alive until `inputs()` finishes (move `git worktree remove --force trees["base"]` into `run()`'s `finally:` block, preserving exact base-tree `text.bz2` bytes) or invoke `bins["head", "ab"]` on line 191 (built in the live repo checkout), plus add a unit test exercising `inputs()` after worktree cleanup before re-queuing.
- **Systemic C6 recipe invariant:** every i9 recipe that invokes `ab gen` must ensure the compiled binary's `CARGO_MANIFEST_DIR` still exists when `ab gen` executes.

### Critical-path review & queue replenishment bottlenecks

`frontier.py brief` reports the workstation repository queue is currently **starved** (`0` unrun `ready/*` entries). The seven highest-leverage open PRs for independent reviewers or holders arriving next are:

| Priority | PR (head, state) | Unblocks |
|---:|---|---|
| 1 | **`#1256`** (`653048a0`, CLEAN) — `pre-release-i9-gate` | **P1 O2 review** (`.github/workflows/deploy.yml`): unblocks `release-0-5-18` and enforces the i9 pre-release regression gate |
| 2 | **`#1303`** (`5fefd6ef`, CLEAN) — `fix/merged-hold-moved-head` | Releases `#1117`'s stale merged integration hold on `src/block.rs` + `src/huff.rs`, immediately unblocking **`scanner-a3-k3-pipe-u8`** |
| 3 | **`#1274`** (`1bb555c3`, CLEAN, currently held) — `runs-front-i9-rescue` | Already **i9 PASS (`95edea2a`)**; holder verdict readback + merge lands `-1.08%` `wikidata/6` / `-1.78%` `text/1` for `0.5.18` and unblocks porting Pass 1 into `src/runs/fold.rs` |
| 4 | **`#1282`** (`90280dc9`, CLEAN) — `plan-schedule-and-lazy-threshold` | Independent C6 review immediately replenishes the starved i9 queue (`LAZY_FROM_THREADS = 4`, 5T/7T/9T pinned trims, capped window scaling) |
| 5 | **`#1304`** (`2b97c37a`) — `scanner-contention-residual` | Independent C6 review replenishes the starved i9 queue with the per-thread scanner PMU contention diagnosis |
| 6 | **`#1295`** (`19902717`, currently held) — `crc32-avx2-ring-rescue` | 1-line `bench/crc32_ring_rescue.py:147,191` fix + re-queue on i9 (kernel + codegen audit already green in `0390e19f`) |
| 7 | **`#1245`** (`d03b8565`, CLEAN) — `ci-bench-open-route-arm` | Independent review lands the 1T forced-open-route CI bench cell (`mibz2-workers-lazy`), closing the CI bench blind spot noted in `synth-23` §3 |
