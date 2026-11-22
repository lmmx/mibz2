# Wormhole Cycle 70 (Neon): Final Round 6 Handoff (`Cycles 51–70`)

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole` (strictly docs-only diff vs `origin/master`)  
**Baselines Verified:** `origin/master` `3296195e3` (`#739` Wormhole PR 1–3) **and** live `origin/master` **`a3f26aa01`** (`#739` + `#745` Mechanisms' L1-resident Huffman block reader)  
**Mechanisms Coordination:** `origin/research/carbon-word-transport` `300a29976` (`C38a`/`C38b`)  
**Manifest:** [`c70_final_r6_handoff.toml`](2026-11-22-wormhole-cycle70/c70_final_r6_handoff.toml) (`check_manifest.py`: `PASS`)

---

## Executive Summary — Complete Answers to All Four Round 6 Charges

| Round 6 Charge | Verdict | Headline Evidence & Deliverable |
|---|---|---|
| **1. Re-diagnose on `#739` (`3296195e3` → `a3f26aa01`)** | **CLOSED** (`C51, C55, C64, C68`) | `#739` broke `10W` DRAM (**`10.82 → 4.6 B/B`**, `23.1 ms` DDR4 floor), `minflt -40.7%`, `sys -41.8%`, `0 B s.ll`, and closed **W3/W4/W5**. `#745` cut `S1 read_item` **`-18.6%` (`-27.4%` L1)**. **W0 (`trimmed_here` `7W/28` cap)** and **W1 (serial scan)** are the last two runtime walls; once removed, **`99.15%` of CPU is parallel** and **`S3 OpenCells` (`64.3%`)** dominates. |
| **2. i9-10900K `trimmed_here` Grid (`10/12/16/20T`)** | **CLOSED** (`C56, C60`) | Delivered **`28`-arm experimental grid** (`16` cells at `10T` + `12` unique SMT cells for `12/16/20T`, `< 4 min` on i9 — **grid only, no hardcoded table**). Anchored to all 4 real i9 points (`0.0–2.6 ms` error). Predicts **`9W / 27`** (`53.2 ms`, `-14.7%` vs `7W/28 = 62.4 ms`) wins on `a3f26aa01` (`9W + 1` dedicated scan core = `10/10` phys cores), while **`10W / 30`** (`~48.5–50.0 ms`) wins once PR 4 lands. |
| **3. PR 2 Residual (`L1_AtomicFast` & locks)** | **CLOSED — PORT NOTHING** (`C53, C55`) | **`0` hot-path `notify_all()` calls remain** in `#739` (`nA = 1` at EOF only). Same-binary A/B across `4W L9`, `4W L1`, `10W`, `16W`, and `ParScan` **killed `L1_AtomicFast` (`0.990–1.006×`, `~8.6 µs`)** and **`Str_TrimLock` (`0.988–1.009×`)**. Wall W5 is finished. |
| **4. PR 4 `BlockSpan` Streaming Interface Spec** | **CLOSED & HARDENED** (`C57–C63, C65–C67, C69`) | Complete written spec (`SpanCand`, `SpanFinder`, `WindowBuf`, `SpecOutcome`, `ChainValidator`), Pareto default **`W2M_L3` (`2 MiB × 3 = 6 MiB`)** (+ `W4M_L2` co-champion), **6 Mandatory Invariants (#1–#6)** verified across `9/9` adversarial categories, **zero-scanner-thread drop-in `Pr4StreamDecoder`**, zero-copy **`SliceWindow<I>`** for `decode_parallel`, strict **FIFO** dispatch, and **activation gate `W >= phys_cores`**. Compiled with **zero rework** against live `a3f26aa01` (`#739 + #745`), qualifying at **`973/973` (`100.0%`, `fb=0`)** fast hits and **`0.728×–0.976×` peak RSS** (`55.3 vs 67.0 MiB` at `10W`, vs `#741`'s `1.28×`!). |

---

## 1. Prioritized Production Action Plan (Ready for Coordinator & Mechanisms)

### Action 1 — Queue the Cycle 56 `28`-Arm i9 `W×S` Grid on `a3f26aa01` (`< 4 min`, highest immediate ROI)
- **Do not hardcode a `trimmed_here` table before measuring.** Queue the `16` `10T` arms (`W ∈ {7,8,9,10} × {2W,3W,4W}` + `W ∈ {11,12} × {2W,3W}`) plus the `12` unique `12/16/20T` SMT arms from [Cycle 56 §2](2026-11-22-wormhole-cycle56-i9-worker-slots-grid.md) (with a `hetero`/`level-1` HOL guard on the top-3 finalists).
- **Predicted i9 outcome on `a3f26aa01` (`#739 + #745`):** **`9W / 27` (`53.2 ms`)** or **`9W / 18..36` (`52.9–53.9 ms`)** beats current `7W/28` (`62.4 ms`) by **`~-9 ms` (`-14.7%`)** with `0%` SMT clash (`9` workers + `1` dedicated serial `#745` scanner = `10/10` physical cores), and resolves `#739`'s inconclusive `20T` cell by clamping `12/16/20T` to the `10T` winner.

### Action 2 — Port Theorem 52.1 `ZeroShift52` In-Place `RLE1` (`~45` safe-Rust lines in `src/rle.rs`)
- Drop-in replacement for `src/rle.rs:121-170` behind the existing `find_runs_avx2` (no signature change, `#![forbid(unsafe_code)]`, verified on `5,000` adversarial randomized blocks + `973/973` corpus blocks).
- Exploits bzip2 RLE1's skip-free downward step ($\Delta_k = \text{extra}_k - 1 \ge -1$) to decompose runs at $g = 0$ crossings into disjoint valleys ($g \le 0$, L→R) and hills ($g > 0$, R→L): **`0` pre-shift copy, `0` `s.ll` staging fallback**, cuts `copy_within` **`-46.6%` (`350.8 → 187.4 MB`, `-1.56 B/B` R+W)**, cuts RLE1+CRC **`0.8944× [0.8850, 0.9039]`**, and adds **`-1.57%` wall at `4W` / `-3.0..-6.6 ms` user CPU** on live `a3f26aa01` ([Cycle 52](2026-11-22-wormhole-cycle52-rle-zero-shift-and-feeder-audit.md), [Cycle 69](2026-11-22-wormhole-cycle69-final-ablation-a3f26.md)).

### Action 3 — Joint PR 4 `BlockSpan` Port (`Mechanisms C38b` + `Wormhole W2M_L3 / Pr4StreamDecoder`)
- **Mechanisms (`src/scan_span.rs`):** Export `SpanCand { rel_bit, guessed_level }` and `SpanFinder::find_cands` backed by `C38b`'s `find_magics_anchor_range` (indexing `b"BZ"`/`b"Zh"` alongside the 48-bit magic bitmap so mid-window multi-stream level transitions refresh `guessed_level` immediately).
- **`StreamDecoder` (`src/parallel.rs`):** Plug `SpanFinder` into `Pr4StreamDecoder` ([Cycle 58](2026-11-22-wormhole-cycle58-pr4-blockspan-streaming-interface-spec.md), [Cycle 65](2026-11-22-wormhole-cycle65-pr4-push-stream-decoder-adapter.md)) with:
  - **Window Pareto default:** **`W2M_L3`** (`win_cap = 2 MiB`, `MAX_LIVE_WINS = 3`, `6 MiB` total input RAM — or `W4M_L2` `8 MiB` co-champion), **`0` dedicated scanner thread** (`W` worker threads total), **strict FIFO** dispatch ([Cycle 67](2026-11-22-wormhole-cycle67-pr4-lpt-straggler-kill.md)), and **`64 KiB` L2-hot feeder chunk** for `StreamDecoder::write` callers.
  - **Activation Gate (fixes `#741`'s `1.11×` `6T` loss):** Engage PR 4 when `workers >= phys_cores` (e.g., `>= 10` on i9-10900K) or explicitly requested; below `phys_cores`, serial `StreamDecoder` gets a free dedicated core (`W + 1 <= phys_cores`).
  - **All 6 Mandatory Production Invariants (hardened in Cycles 59, 61, 62):**
    1. **`published_seq` EOF Guard (`C59`):** `take_next_spec` returns `None` only when `st.win_closed && st.taken_seq >= st.published_seq` (never `tq.q.is_empty()` alone — prevents last-block worker race / `fb=1`).
    2. **Zero-`memset` Window Refill (`C59`):** Keep recycled buffers at `win_cap`, read directly into `&mut buf[filled..win_cap]`, truncate after loop.
    3. **Livelock-Free Oversized-Block Guard (`C61`):** Cap `tail_carry <= (win_cap / 2).max(16)` so `abs_base` advances strictly every window, and accumulate across released windows in `fb_buf` on fallback (`904 KiB` blocks pass in `256 KiB` windows at `peak_lw <= 2`).
    4. **`magic_over` Before `BadBlockMagic` (`C61`):** Check `br.overran()` immediately after reading the 48-bit magic so `4..9 B` truncations report `Truncated`.
    5. **`br.overran()` Precedes `ErrorKind` Classification (`C61`):** Map any `Err` with `br.overran() == true` to `NeedMoreOrTrunc` (worker) / `Truncated` (fallback) — prevents `NoSymbols` on `10 B` truncations or valid seam aborts.
    6. **`owned_end_byte` Early `cur_win` Release (`C62`):** Store `owned_end_byte = next_abs_base` on `WindowBuf` and release old `cur_win` **before** waiting on `win_log` (deadlock-free even at `L=1`, releases `cur_win` 1 block earlier for `L>=2`).
- **`decode_parallel(&[u8])` (`src/lib.rs`):** Use generic zero-copy **`SliceWindow<I: InverseBwt>`** ([Cycle 66](2026-11-22-wormhole-cycle66-decode-parallel-slicewin.md) — `0 B` input copy, `0` window buffers, `0` feeder/scanner threads, `0.35 ms` lazy `2 MiB` strides; `-12..-48 MiB` RSS, `-48%..-74%` `vcsw`, `0.935×` `random-l9`, `0.917–0.946×` with `DP_Chunks`).

### Action 4 — Next Frontier After Transport Saturation: `S3 OpenCells` (`64.3%` of Remaining `1T` CPU)
With `99.15%` Amdahl parallelism and `~3.8 B/B` `10W` DRAM (`19.1 ms` DDR4 floor), the `~48.5–50.0 ms` `10T` i9 runtime is pure kernel compute (`S3 OpenCells 64.3%`, `S1 #745 13.5%`, `S4 ZeroShift52+AVX2 ~9.7%`, `S2 runs::front 8.7%`). Porting Round 4 `WormholeNeonCells` (`0.870×` Open) + Teamwork `BSAC` + `Eta` projects **`~39–45 ms`** at `10T` on the i9-10900K.

---

## 2. Complete Round 6 Keep / Kill Ledger (`Cycles 51–70` — 14 Honest Kills)

| Cycle | Candidate / Idea | Verdict | One-Line Evidence |
|---:|---|---|---|
| C52 | **`ZeroShift52` (Thm 52.1)** | **PROMOTED** | `-46.6%` `copy_within` (`-1.56 B/B`), `0.8944×` RLE1+CRC, `-1.57%` wall at `4W` on live `a3f26aa01`, `0` pre-shift, `0` `s.ll` fallback. |
| C52 | `bzcat_8M_noz` | **KILLED** | `0.9985× [0.974, 1.023]` — noise on 2 chunks (`1 MiB` kept strictly for `-9.9%` RSS / `-11.2%` `minflt`). |
| C53 | `L1_AtomicFast` & `Str_TrimLock` | **KILLED** | `0.990–1.006×` across `4W L9/L1, 10W, 16W, ParScan` (`0` hot `notify_all()` remain; saves `~8.6 µs`). **Port nothing more from PR 2.** |
| C54 | `D1_SliceSrc` alone | **KILLED** | `0.9948× [0.976, 1.014]` — neutral wall alone. |
| C54 | `D2_AdaptCap` & `D3_Chunks` | **PROMOTED (nuanced in C66)** | `0.9786×` / `0.9540×` on `#739`; inside C66 `SliceWindow`, Linux `mremap` makes `D2_AdaptCap` wall-neutral (`D3_Chunks` wins `0.917–0.946×`). |
| C56 | Hardcoded `trimmed_here` table | **KILLED** | Replaced by `28`-arm experimental i9 grid (the i9 decides). |
| C57 | `P0_WholeIn` (`#741` whole-input model) | **KILLED** | Reproduces `#741`'s `1.32×` RSS at `10W` (`1.72×` at `16W`) and neutral wall (`1.0011×`). |
| C57 | `P1_Win1_Bar` (`C38b` 1-window barrier) | **KILLED** | Fixed RSS, but epoch drain barriers pay `2×` `vcsw` (`238–379`) and lose wall to Ping-Pong everywhere. |
| C58–62 | **`PR4_W2M_L3` (`2 MiB × 3 = 6 MiB`) & `W4M_L2` (`4 MiB × 2 = 8 MiB`) + Invariants #1–#6** | **PROMOTED** | `9/9` adversarial pass, `973/973` (`100.0%`, `fb=0`) on live `a3f26aa01`, **`0.728×–0.976×` peak RSS** (`55.3 MiB` at `10W`), **`-42%..-68%` `vcsw`**, **`-28%..-48%` `minflt`**. |
| C59, C62 | `W1M_L3`, `W4M_L1`, `W2M_L2`, `W4M_L3`, `W8M_L2` | **KILLED** | `W1M_L3`/`W4M_L1`/`W2M_L2` starve `hetero`/`random` (`1.042–1.062×`); `W4M_L3`/`W8M_L2` waste `+4..10 MiB` RSS for `0` wall gain. |
| C65 | **`Pr4StreamDecoder` (`PR4_Push_64K`, `0` scanner thread)** | **PROMOTED** | Exact drop-in `StreamDecoder` API (`17 B..4 MiB` `6/6` parity, `0.702×–0.840×` RSS, `-51.1%` `minflt`). |
| C65 | `PR4_Push_2M` (`2 MiB` feeder bounce) | **KILLED** | Extra `2 MiB` feeder bounce adds `+2.4..3.6 MiB` RSS, `+670..930` `minflt`, `1.0210×` `hetero-l9`. |
| C66 | **`SliceWindow<I>` (`DP_SliceWin` & `DP_Chunks`)** | **PROMOTED** | `0 B` input copy, `0` window bufs, `0` feeder/scanner threads; `-12..-48 MiB` RSS, `DP_Chunks` wins `0.917–0.946×`. |
| C67 | `LPT_Batch` & `LPT_Tail` (within-window LPT) | **KILLED** | `0.994–1.010×` (all 95% CIs span `1.000`), `+12%` `vcsw`, `+0.6..2.8 MiB` RSS — keep strict **FIFO**. |

---

## 3. Clean Stop Declaration

Round 6 (`Cycles 51–70`, 20 cycles total) is **complete**. Every cycle is committed (`wip(wormhole/neon): cycle N ...`), validated (`check_manifest.py` `PASS`), logged in `FRONTIER.md`, verified strictly `docs/`-only vs `origin/master`, and pushed to `origin/research/carbon-wormhole`. Stopping cleanly here.
