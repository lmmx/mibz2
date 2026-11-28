# Synthesis — twenty-ninth pass

**Date:** 2026-11-28
**Scope:** 17 commits merged on `master` since `synthesise-28` (`80222230..c3b8f708`: `#1375` through `#1390`), plus live open PRs (`#1341`, `#1328`, `#1282`, `#1311`, `#1343`, `#1065`, in-flight `#1391`), `bench/workstation-queue.txt`, and the claim ledger (`8/12` open, `4` blocked, `1` review, `27` idea, `91` done, `11` dropped).

---

## 1. Workstation & `v0.5.19` critical path (`#1375`, `#1382`, `#1324`, `#1390`)

1. **Physical i9 `bench-block-size-route` returned `STOP` (`#1375` `3c7212bb`, `origin/evidence 7f03816b`):**
   - Under `strict_i9 = true` (3 rounds × 2 orders = 6 sweeps, 7/7 SHA-256 parity), no `<400k` row bin met the `>= 1.05` admission threshold: the highest ratio was `1.0321` on `text -1`, identical to `text -9` (`1.0359`). On `wikidata`, `-1` was `1.0149` while `-2` (`0.8795`), `-3` (`0.8457`), `-5` (`0.8685`), and `-9` (`0.8276`) all favoured `Open`.
   - The 2D `(row_bin × r/n)` grid is flat across `100k..900k` rows (crossing `1.0` near `r/n ~ 0.10` at every block size), proving `#1355`'s `1.17–1.20×` `-1` regression was a 2 MiB VM L2 artefact (8× the i9-10900K's 256 KiB L2), not a block-size routing defect.
   - `open-admission-block-size` was dropped (`#1375`); `continuation-runbook` was marked `done`.
2. **P1 `v0.5.19` gatekeeper (`#1324` `bench-forward-adopt-rescue`) is now queued on Louis's physical i9 (`#1382`, `#1324`, `#1390`):**
   - Independent C6 review (`#1382` `f47a2fda`) caught two verdict-accounting guards (`mark_unregistered` and missing `parity.jsonl` VOID), fixed in `474cba46`, and cleared `#1324` (`575fa98a`, frozen Clause 3 C6 registration & recipe, 100% green CI).
   - `#1390` (`c3b8f708`) tagged `ready/bench-forward-adopt-rescue` at `575fa98a` (run `46166183485`), appended `bench-forward-adopt-rescue` to `bench/workstation-queue.txt`, and moved `forward-emission-bounded-slab-rescue` to `blocked` awaiting the hardware bundle.
   - Because `#1324` (`forward-anchor-lanes`, MLP transfer prior `1.50–1.97×` per `#1368`) is the single lever that flips 8W Wikidata from worker-bound to scanner-bound, every downstream Station `S` investment (`#1343`, `u8-syms`, `scanner-adaptive-seam-composed`, `#1377`) now waits cleanly on this one queued i9 run.

---

## 2. Complete post-`M5` algorithmic stage map (`#1377`, `#1379`, `#1381`, `#1383`, `#1384`)

Five consecutive smooth-zone dissections mapped every remaining stage of the composed `M5` decoder (`#1362`), cleanly separating closed inner loops from the four remaining real levers:

| Stage / Target | Verdict & Claim | Key Primary-Source Finding | Sequenced Behind |
|---|---|---|---|
| **Station `S` (post-`#1343`)** | **PASS → `scanner-post-a3-direct-u8-avx2`** (P2 `idea`, `#1377` `ce7195b9`) | Killed wide root tables (`B=10 K=4` `u64` and unconditional `B=11 K=3` `u32`, `48 KiB > 32 KiB L1d`). Four orthogonal wins across all 4 quadrants (`BASE/JCC-32B × warm1/cold32`, `n=11`): direct `SymBuf::buf` write, 4-step overlapped refill on `!multi`, inline `SubLut` hit, safe single-write `K=3` + AVX2 `vpaddd` table builder (`-19.8..-27.4%` build). `G1_safe_sub` (`0` new `unsafe`): `JCC-32B cold32` `-4.32%` `wikidata`, `-6.36%` `text`, `-14.31%` `records`, `-14.62%` `random`; `G3_ptr_sub`: `-7.46%` `wikidata` (`2.50 ns/sym`), `-7.73%` `text`, `-14.45%` `records`, `-13.17%` `random`, `-6.29%` `runs`. Under `#1368`'s `1.01–1.05×` Station `S` prior, projects `-4.5..-8.0%` off serial `S` (derived i9 scanner ceiling `~7.0–7.3 GB/s` stacked on `#1343` + `seam`). | `#1343` + `u8-syms` (+ `#1324` i9 flip) |
| **Station `W` — `A6b` (post-`#1328`)** | **PASS → `a6b-post1328-fast-build`** (P2 `idea`, `#1379` `ede5c08e`) | Post-`#1328` `overlay_aos` (`37a2a37a`, `forward-anchor-lanes`, 233 Wikidata Open blocks): `dict_aos` is `#1` (`40.0–41.4%`, `47.0–47.7 ms`) ahead of `elim_aos` (`37.7–39.5%`), with zero wasted bidirectional work. Zero-RSS `#![forbid(unsafe_code)]` prototype `S1_setup + D2_dict_ctx` (per-`ch` disjoint `13 KiB` L1d tables inside existing `16 KiB` overlaid `edges[0..4096]` + O(1) 4-word sampled long hash + `DictCtx` outlining + `chunks_exact` setup) cuts residual `A6b` **`-14.42%` `base` warm / `-10.2%` `base` L3-cold / `-15.33..-15.72%` `JCC` warm / `-12.35%` `JCC` L3-cold** (`dict` `-32.5%`) and 8W E2E `wikidata` CPU **`-1.34%` `base` / `-2.56%` `JCC`** at `1.0019×` `VmHWM` (`d990b66d` parked locally). Killed `16 B` inline-5 and `32 B` packed-`u16` `u128` `elim_aos` layouts. | `#1328` |
| **Station `W` — `A7` (`Walk::run<4>`)** | **`STOP`** (`#1384` `f0a340b0`) | Tested **34 variants** of `#1312` `Walk::run<4>` (1-step `edges[next]` prefetch `T0/T1/T2`, cold `LONG`, packed state, zero-`memcpy` overlapping-16B `<=32B` `LONG`, 4-lane batch guard, safe 1-step pipelining). `W0` sits on a tight x86-64 16-GPR Pareto optimum (`81` stack loads + `33` stores hit L1d and hide behind the 4 random `edges[j[0..3]]` L2/L3 loads). Best finalist `W4a_t2` (`n=11` paired): `JCC-32B` `0/4` cells disjoint faster (`1W` warm `-0.88%`, `8W` warm `-2.58%`, `8W` cold `+1.07%`); `base` `2/4` cells **disjoint slower** (`8W` warm `+2.58%`, `8W` cold `+4.63%`). **`Walk::run<4>` inner loop is closed.** | Closed |
| **Off-Wikidata `Column` fallback** | **`STOP`** (`#1383` `8102f207`) | `0` native-`Column` full blocks on `wikidata`, `text`, `prose`, `elf`, `runs`, `pattern`, `onebyte`; only high-entropy `random` (18 blks) & `records` (22 blks) hit `Column` (both already scanner-bound `>= 4W` on i9: `k* = 3.90 / 3.68`). Shipped `Column` already uses 32-lane MLP `room_segments`; true bottleneck is serial `mtf::mtf` (`62.1–66.7%`). Best fused zero-`ll` stack (`R2_outl_rel_signed7`) saves `-0.86 MiB/w` (`-6.9 MiB` at 8W) but caps at `-4.21%` warm / `-3.25%` cold `random` (`JCC-32B`) and `-0.66..-0.95%` `records` (`~-1.5% / ~-0.4%` i9 1W, `~0%` `>= 4W`). | Closed |
| **Pipeline admission (runts)** | **PASS → `runt-exempt-window-rows`** (P3 `idea`, `#1381` `77de5b69`) | Exact BWT row count `RUNT_ROWS = 1_200` captures **198/198 (100%)** Wikimedia runts (`1..1,185` rows) and **0/68 (0%)** expansive run-only blocks (`onebyte`/`pattern`/`zeros-20blk`/`runs`), capping exempt memory at `<= 64,562 B < 64 KiB` (`zeros-20blk` `VmHWM` `1.000×` at `t4`, `1.021×` at `t8`). Paired VM wins: `-11.87%` (`t4 s8`), `-4.90%` (`t4 s16` eager), `-3.26%` (`t4 s16` `#1282` lazy, `1.004×` RSS), `-4.98%` (`4× t8` `forward-anchor-lanes`). Caveat: `t2 s8` eager `+3.46..+4.50%` on loaded 2-vCPU VM (gate to `threads >= 4` if needed). Dropped premise-defective `runt-exempt-window`. | `#1282` |

**Strategic takeaway:** Two major `W` inner loops (`A7` `Walk::run<4>` and `Column`) are now definitively closed on primary evidence, preventing wasted follow-up cycles. Post-`M5` performance work concentrates cleanly on four live frontiers:
1. **Station `S`:** `#1343` → `u8-syms` → `#1377` (`scanner-post-a3-direct-u8-avx2`) + `seam`.
2. **`A6b`:** `#1328` → `#1379` (`a6b-post1328-fast-build`, pre-built at `d990b66d`).
3. **`A5+A6a`:** in-flight dissection `#1391` (`explore/a5a6a-residual`) + `default-a5a6a-fast-build`.
4. **Pipeline admission:** `#1282` → `#1381` (`runt-exempt-window-rows`, pre-built on `explore/runt-window`).

---

## 3. Swarm porosity & frontier protocol wave (`#1378`, `#1380`, `#1385`, `#1386`, `#1387`, `#1388`, `#1389`)

`#1378` (`bd75fc83`) audited `qu-reeds/asoc-autom` and opened four concrete protocol claims plus three ideas. Six follow-on PRs landed the same day:

- **`brief-state-first` (`#1380` `93453956`):** per-checkout SHA-256 rule cursor inside `frontier/tools/frontier.py` shrinks repeat `brief` output from `24,355 B` to `8,474 B` (`-65%`, `126/126` tests), eliminating ~16 KiB of boilerplate per turn.
- **`prove-frontier-cold` (`#1385` `16cf96ff`):** cold stranger walkthrough on a private bare remote found **16 defects** (`5 breaks, 6 misleads, 5 friction`) and filed `frontier-cold-loop-fixes` (P2 `idea`, behind `#1341`).
- **`prove-frontier-damaged` (`#1386` `f346d653`):** 144 single hand-editing damages across private bare remotes found **19 defects** (`10 breaks, 7 misleads, 2 friction`), with a tested scratch patch cutting raw tracebacks from **`40/144` to `0/144`** (`90/90` `test_frontier.py` + `34/34` `test_continuation.py` PASS); filed `frontier-damage-hardening` (P2 `idea`, behind `#1341`).
- **Immediate `continuation.py` hardening (`#1387` `d8954247`):** resolved all four proven `continuation.py` / `CONTINUATIONS.md` defects from `#1385` and `#1386` with zero overlap against `#1341` — isolated malformed `wait/*` refs (`#1386` Defect #9), added `--manual` resume (`#1385` B1), bounded `0 < --hours <= 8` (B2), added `--session` to `done` hints (M3), and surfaced ISO `show` timestamps + `--next` (F5), `38/38` `test_continuation.py` PASS.
- **`correspondence-kinds` (`#1388` `9d1858c1` + `#1389` `ea4049b5`):** classified every row of `frontier/README.md`'s hybrid-system correspondence table (`0` identities, `3` models, `3` analogies, placed outside `<!-- brief -->` so `brief` stays compact), `92/92` `test_frontier.py` PASS.

**Sequencing:** all remaining `frontier/tools/frontier.py` hardening (`continuation-runbook--brief-after-1341`, `swarm-quiet-state` + Defect #10, `frontier-damage-hardening`, `frontier-cold-loop-fixes`) shares one clean gatekeeper — P1 `#1341`. Once `#1341` merges, those four items unblock sequentially without rebase churn.

---

## 4. Publication, live open PRs, and parked pre-builds

- **Blog (`#1376` `701366e1`):** published *"When the virtual machine lies: why mibz2 now gates releases on real hardware"*, capturing the `v0.5.18` hardware-gating shift for external readers.
- **Active open PRs & unblocking order:**

| PR | Priority / Claim | Head / State | Action & What It Unblocks |
|---|---|---|---|
| `#1341` | P1 `ship-completion-pressure` | `56f7a573` `MERGEABLE` (`0` fail) | Merge once final check completes; unblocks `continuation-runbook--brief-after-1341`, `swarm-quiet-state`, `frontier-damage-hardening`, `frontier-cold-loop-fixes`. |
| `#1311` | P2 `segmented-fused-postwalk-rescue` | `df4f0bbd` `MERGEABLE` (`CLEAN`) | Pure-CI adoption candidate; merging (with `#1282`) clears `src/bwt.rs` / `src/lib.rs` / `src/parallel.rs` for the `-2,334 LOC` §3.3 dead-feature cleanup (`23ed42bc`). |
| `#1328` | P2 `a6b-zero-rss-endpoint-build` | `54033d62` `MERGEABLE` (`0` fail) | Synced onto `#1324`'s `Justfile` import (`0` `src/` diff); independent review + merge unblocks `a6b-post1328-fast-build` (`#1379`, parked at `d990b66d`). |
| `#1282` | P2 `plan-schedule-and-lazy-threshold` | `cfc2723c` `MERGEABLE` (`0` fail) | Synced onto `#1324`'s `Justfile` import (`0` `src/` diff); independent review + merge unblocks `runt-exempt-window-rows` (`#1381`, parked on `explore/runt-window`) and §3.3 cleanup (`23ed42bc`). |
| `#1343` | P2 `scanner-a3-k3-pipe-u8` | `3d2afb8a` `MERGEABLE` (`CLEAN`) | Still awaits `#1345`'s 3 C6 recipe fixes before hardware queueing; unblocks `scanner-post-a3-direct-u8-avx2` (`#1377`). |
| `#1391` | P3 `explore` (`A5+A6a` residual) | `MERGEABLE` (CI running) | Completes the last remaining stage dissection (`A5+A6a` with `F8` and `u8 SymBuf`). |
| `#1065` | P2 `migrate-rules` (`review`) | `ec7bbc09` `CONFLICTING` | Owner-gated (`RULES D`); awaits Louis's decision and rebase. |

- **Claim ledger:** `8/12` open (`ship-completion-pressure`, `a6b-zero-rss-endpoint-build`, `plan-schedule-and-lazy-threshold`, `scanner-a3-k3-pipe-u8`, `segmented-fused-postwalk-rescue`, `swarm-quiet-state`, `synthesise`, `explore`), `4` blocked (`forward-emission-bounded-slab-rescue`, `mtf-fold-run`, `branch-protection`, `three-arm-experiment`), `1` review (`migrate-rules`). No new claims needed from this pass — every actionable follow-on already has a filed claim and an explicit predecessor.
