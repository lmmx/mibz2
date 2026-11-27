# Synthesis, fourteenth pass: i9 verdicts fix the 143.5 ms serial floor and the L3-footprint law, post-#1080 routes and A3/A6c levers stack cleanly, and a `pyproject.toml` overlap blocks P1 release 0.5.15

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml), 27 November 2026. Read with the
[thirteenth pass](thirteenth-pass.md) (#1133) and [twelfth pass](twelfth-pass.md) (#1109). Primary
sources since #1133 (`f00aaaf8`): two physical i9-10900K verdicts in #1151 (`d6f3a00f`:
`origin/evidence` `9de7b6e0` for `#1034` **STOP** + owner override in #1149 /
`2026-11-27-scanner-coalesced-kept-by-owner.md`, and `origin/evidence` `a6e427b1` for `#1053`
**STOP/KILL**); #1129 (`1429f54f`, `ready-cap-worker-gate` merged); #1134 (`8cb3d2d8`); #1135
(`2fed5f18`, `text-knee-i9` admitted); #1136 (`15617f17`); #1137 (`3decbce8`, `ci-pattern-4t-arm-bias`
**NOT REPRODUCED**); #1138 (`f0fdd8b9`, `mtf-fold-run` shallow hardening); #1140 (`15d80be7`); #1141
(`d5f8575b`, `in-maint-residual-dissection` **PASS**) + #1143 (`d2e5f463`); #1142 (`f6822b61`,
`pooled-scan-hybrid` frozen pre-data registration); #1145 (`521f5ca3`, `route-crossover-post-1080`
**PASS**); #1146 (`379f0d46`), #1148 (`f4022dbb`), #1149 (`269caf09`), #1150 (`71849dfa`) (blog/notebook
split, ledger chain, P1 `release-0-5-15`); #1147 (`6e8cc8e0`, `python-binding-dissection` **PASS**);
#1152 (`5c2e4ad4`, `a6c-wire-dissection` **PASS**); #1153 (`af169be2`, `scanner-a3-post1034-dissection`
**PASS**); #1154 (`15a0c1ce`, `runs-takes-runs-post-1080` **PASS**); and active open PRs #1065, #1093,
#1117, #1119, #1124, #1132, #1139, #1144, #1155, #1156. **(derived)** marks arithmetic across sources;
VM/CI diagnostics are strictly separated from physical i9-10900K hardware measurements (C2/C5).*

## 1. Physical i9-10900K verdicts #2 and #3 (`#1151`): the `143.5 ms` serial floor and the L3-footprint law

Three physical i9-10900K (Comet Lake-S, stepping 5, 10C/20T, `rustc 1.99.1`, default x86-64 codegen)
verdicts have now returned on `origin/evidence`:

| Candidate | Evidence bundle (SHA-256 prefix) | Frozen verdict (C2) | Measured i9 key result (`head / base`, 95% CI) | Peak RSS vs base | Disposition |
|---|---|---|---|---|---|
| `#1031` `scanner-seam-window` | `c0b2341c` (#1118) | **NOT ADOPTED** | 10T `0.9811 [0.9670, 0.9954]` FASTER; 12T `0.9898 [0.9726, 1.0073]` INCONCLUSIVE; `in_maint` `10.689 → 7.490 ms` (`-29.93%`) | `0.969–0.993×` | Closed unmerged; dissected in #1141 → redesigned in PR #1144 |
| `#1034` `scanner-coalesced-u8-decode` | `9de7b6e0` (`0a5e398f7bd4`, #1151) | **STOP** (missed `real/10` upper `<= 0.970` by `0.0042` & range overlap) | **`phases/wikidata/1` `0.8950 [0.8923, 0.8978]` (`-10.50%`, `160 → 143.5 ms` disjoint)**; `C/8 -7.80%`; `real/8` `0.9807 [0.9626, 0.9991]` (`-1.93%`); `real/10` `0.9598 [0.9457, 0.9742]` (`-4.02%`); `0/30` guard regressions (`random` 6/8T, `pattern` 6T, `onebyte` 8T FASTER; `pattern` 1/2/8/10T neutral) | `<= 1.012×` Wikidata (`<= 1.042×` all 35 cells) | **Kept in default & `release-0-5-15` by owner override** (`2026-11-27-scanner-coalesced-kept-by-owner.md`); frozen STOP preserved untouched; CI flag #1084 remains open for independent disposition |
| `#1053` `a6b-endpoint-build` | `a6e427b1` (`623a7096f412`, #1151) | **STOP/KILL** (failed `wikidata/1` and `wikidata/8/S` `<= 0.965` gates) | `wikidata/1` `0.9824 [0.9776, 0.9872]` (`-1.76%`, `+24.7 ms`); `wikidata/8` `1.0072 [0.9916, 1.0231]`; `wikidata/10` `1.0091 [0.9783, 1.0410]`; **`wikidata/8/S` `wkr_busy` `1.0106 [0.9931, 1.0283]` (`-2.1 ms/w`), wall `1.0124`**; `text/1` `0.9919`, `text/8` `1.0028` | **`1.0374×` (`+4.19 MiB`) at 8W; `1.0488×` (`+5.46 MiB`) at 10W** (`+524–546 KiB/worker`) | Remains feature-gated off by default; NOT promoted; compact-footprint rescue opened at `idea` |

Two system-level laws emerge from reading `#1034` and `#1053` together:

1. **The L3-footprint law (`<= 1.01×` 8W RSS is a hard architectural constraint on 20 MiB Comet Lake L3).**
   `#1053` proved that a clean 1-worker algorithmic win (`-24.7 ms` / `-1.76%` on `wikidata/1` on real
   hardware, `-27..-29%` isolated `A6b` on VM) can flip into a net **regression** at 8 workers
   (`wikidata/8/S` `wkr_busy = 1.0106`, `wall = 1.0124`) solely from per-worker working-set bloat.
   `#1053`'s fixed 64K SoA slabs (`Box<[NodeLinks/u32/u128; 65536]>`, `1.75 MiB` virtual) added
   `+524–546 KiB/worker` resident RSS (`+4.19 MiB` across 8W, `+5.46 MiB` across 10W), pushing Comet
   Lake's shared 20 MiB L3 past its contention cliff (confirming the pre-registered L3/RSS hypothesis
   in `a6b-endpoint-build.toml` and `worker-l3-footprint.md`). Contrast this directly with the
   compact-footprint designs that dominate the rest of the pipeline:
   - `#1152` (`a6c-wire-redesign`): **`0 B`** extra (`1.00×` scratch via safe `Cell::from_mut`),
   - `#1110` (`forward-emission`): `32 KiB` dynamic slabs inside `out` (`<= 992 KiB`),
   - `#1132` (`mtf-fold-l1-batch-redesign`): L1d `[u32; 256]` batch + `OutQ16` `1008/1024` guard zone,
   - `#1153` (`scanner-a3-k3-pipe-u8`): `+488 B` (`1.008×` `ScanScratch`),
   - `#1145` (`post-1080-open-admission`): **`0 B`** extra,
   - `#1144` (`scanner-adaptive-seam-window`): **shrinks** reserved window `-72.6%` (`16.0 → 4.4 MiB`),
   - `#1129` (`ready-cap-worker-gate`, merged): **shrinks** `zeros_2g` RSS `-51.2%` (`1,117.9 → 545.4 MiB`).

   Because `#1053`'s underlying algorithmic mechanisms (fused `A6b-setup`, `u16` links, `inline15`,
   generation-tagged `contents_4k`) delivered real 1W gain on i9 before fixed-64K slab bloat erased it at
   8W, this synthesis opens **`frontier/claims/a6b-compact-footprint-rescue.toml`** (`status = "idea"`,
   P3, smooth `explore`) to test `n_nodes`-sized compact slabs (`<= 1.01×` 8W RSS, optionally folding
   `#1079`'s `V2_u128` cleanup) before shelving `A6b`. `#1053`'s frozen `STOP/KILL` verdict stands
   untouched (C2).

2. **`#1034` closes the physical i9 serial-baseline uncertainty and clarifies registration calibration.**
   Post-`#1034` serial scan on physical i9-10900K is no longer derived — it is **measured at `143.5 ms`**
   (`phases/wikidata/1`, `[143.1, 144.1] ms`, `-10.50%` disjoint from `160.3 ms`). On real hardware,
   `#1034` also cleared every guard corpus (`0/30` regressions, `pattern` 6T `0.9724` FASTER, 1/2/8/10T
   neutral). Complementing this on CI, `#1137` (`ci-pattern-4t-arm-bias`, **NOT REPRODUCED** on plain and
   two-build A/A + 6 out-of-sample strict-null CI runs at `0.9970 [0.9865, 1.0075]`) decomposed the
   24-run CI 4T `pattern` spike into two non-scanner mechanisms: (a) a symmetric bimodal `ready`-backlog
   RSS regime (`36–38 MB` vs `>= 39 MB`, `R² = 0.706`, slope `0.881`, placing `#1034`'s RSS-adjusted wall
   at `1.0035 [0.9725, 1.0355]`) — directly targeted by merged `#1129` (`ready-cap-worker-gate`) — and
   (b) a `p = 0.0114` Round-0 cold-start head-first asymmetry (`~+0.6%` on a 10-round mean). The open CI
   flag `2026-11-27-scanner-coalesced-ci-acceptance.md` remains untouched for an independent agent to
   record its formal disposition. Note also Louis's guidance alongside his owner override: a gain beyond
   noise with no primary loss is a gain, and future registrations stricter than `PERFORMANCE-POLICY.md`
   should state why.

## 2. Updated serial scan `S'` envelope post-`9de7b6e0`: measured `143.5 ms` → derived `~115–119 ms`

Replacing Pass 13's derived `~141–148 ms` estimate with `9de7b6e0`'s **measured physical i9 `143.5 ms`**
post-`#1034` baseline shows three orthogonal, zero-L3-bloat serial levers now ready to stack on `S'`:

| Serial lever | Target phase | Status | Local / VM measured cut | Derived i9 `S'` delta *(derived)* | Memory impact |
|---|---|---|---|---|---|
| `#1034` `scanner-coalesced-u8-decode` | `A3` (`B=10 K=2`, BMI2) | **Merged & kept** (`5ed20c1d`) | **Measured i9:** `-10.50%` (`160.3 → 143.5 ms`) | Baseline **`143.5 ms`** *(measured i9)* | `<= 1.012×` RSS |
| `#1141` → PR `#1144` `scanner-adaptive-seam-window` | `in_maint` (`src/stream.rs`, `src/parallel.rs`) | **PR open, 31/31 CI green, C6 review** | `in_maint` `-96.1..-97.7%`; bytes `-95.7%`; `0` retries across 5 guards | **`~-10.3 ms`** (`10.689 → ~0.42 ms`, 3.2× `#1031`) | **`-72.6%`** reserved window (`16.0 → 4.4 MiB`) |
| `#1069` → PR `#1117` `scanner-a2-fast-build` | `A2a..d` (`src/block.rs`, `src/huff.rs`) | **PR open, parked on LTO alignment** | `A2a..d` `-31.6..-37.7%`, `S'` `-4.44..-7.38%` | **`~-6.4..-10.7 ms`** | `0 B` extra |
| `#1153` `scanner-a3-k3-pipe-u8` (`C6_k3hi_pipe4`) | `A3` (`B=10 K=3` u8-chunk + 4-step overlapped refill, `K3_GATE_SEL = 1024`) | **Probe PASS (all 4 builds), `idea` behind `#1117`** | Paid `A2d+A3` `-5.64%` base / `-6.15%` base+JCC / `-7.65%` nat / `-7.48%` nat+JCC (all disjoint; decode `-9.30..-11.33%`; `text` `-7.88..-11.61%`) | **`~-6.5..-8.8 ms`** (`-10..-11%` off scanner-bound `text` 8W) | `+488 B` (`1.008×` `ScanScratch`) |

- **Combined serial `S'` envelope *(derived, accounting for mild `A2d`/`A3` overlap)*:**
  `143.5 ms - ~24.5..28.5 ms ≈` **`115–119 ms`** on default x86-64 i9 — comfortably below the `140 ms`
  (`6 GB/s` on 839 MB) contour from pure serial work reduction alone, with zero pooling overhead and
  *lower* memory footprint than `master`.
- **Where `pooled-scan-hybrid` (`#1142`) fits:** `docs/journal/2026-11-24-algorithm-level/pooled-scan/hybrid-registration.md`
  is now merged and frozen pre-data (`pooled-scan-hybrid` `done`). Once `#1144`, `#1117`, and `#1153`
  drive serial `S'` toward `~115–119 ms`, slack-only hybrid scanning shifts from a Wikidata `k = 8`
  necessity (unless JCC-32B alignment drops `W'/8` below `~113 ms`) into a **`k = 9–10` and off-Wikidata
  `text` (`k = 8`) multiplier**, governed directly by the two upcoming attribution runs: `#1119`
  (`jcc-aligned-release-build`, P1) and `#1139` (`text-knee-i9`, P2).

## 3. Worker stack recalibration, the post-`#1080` non-Open revolution, and Python delivery

### 3a. Open worker stack after `#1053` `STOP/KILL`

Removing `#1053`'s projected `-8..9 ms/w` from the 8W worker stack leaves a cleaner, strictly
compact-footprint Open pipeline:
- **`#1132` (`mtf-fold-l1-batch-redesign`, PR open in B5 review, 29/29 CI green):** L1d-batched packed MTF
  fold + `OutQ16` `1008/1024` guard zone (`0` spills, `0` `vzeroupper`, Miri clean). Ready for B5 merge
  and then `mtf-fold-run` (already shallow-hardened in `#1138` after fixing `ic::store_size()`'s `SC`
  missing-count crash).
- **`#1110` (`forward-emission`, merged behind `--features forward-anchor-lanes`):** 4-lane sampled-anchor
  emission (`32 KiB` slabs), awaiting `forward-emission-run` (P2) once `.just/bench.just` is unheld.
- **`#1152` (`a6c-wire-redesign`, `idea`, P3, stacks after `#1132` on `src/open/outq.rs`):** 100% safe-Rust
  `Cell::from_mut` dual-slice zero-bounds-check `wire_xor` + hoisted `mark_byte` (`B1b`/`B6`:
  `-20.2..-27.4%` across base/JCC-32B warm & cold, `0 B` extra on `Q512`, `~33–44 ms` 1W / `~4.5–5.5
  ms/w` 8W derived on shipped `bidi`) plus register-cursor overlapping-tail `wire` (`F5`/`F6`:
  `-15.5..-33.2%`, `~9–20 ms` 1W / `~1.3–2.8 ms/w` 8W derived on post-`#1110` forward).
- **`forward-rle1-crc-vectored-build` (`idea`, behind `#1110`)** and **`a6b-compact-footprint-rescue`
  (`idea`, P3, opened here)** complete the zero-bloat Open stack without repeating `#1053`'s L3 cliff.

### 3b. The post-`#1080` non-Open / off-Wikidata revolution (`#1145`, `#1154`, open `#1155`)

Three independent probes converge on one structural discovery: shipping `#1080` (`Segmented` `-15.4..-18.5%`)
invalidated every pre-`#1080` route boundary on non-Wikidata corpora, leaving massive, near-zero-cost
wins in 100% safe Rust:

1. **`#1145` (`post-1080-open-admission`, `idea` P2, `owned_paths = ["src/open.rs"]`, ~10 LOC, `0 B` extra):**
   `OpenCells` vs `Segmented` crossover now sits at `m/n ≈ 0.18–0.20` (not `r/n = 0.28`). Replacing
   `OpenCellsOf::cap` with two-tier `R2_2tier_m22` (`r/n < 0.10` outright; `0.10 <= r/n < 0.15`
   `within_cap(m <= 0.22*n)`; `>= 0.15` `Segmented`) eliminates `OpenLow` `m/n = 0.42` traps, 67/67 losing
   dense-band blocks, and 58 wasted `RefDense` scans — cutting steal-free MTF+inv CPU **`-12.9..-20.2%`**
   on primary `text`, **`-12.5..-16.8%`** on `prose`/`rustsrc`/`logs`, and **`-1.2..-3.7%`** on
   `wikidata` (18/18 base+JCC cells faster).
2. **`#1154` (`post-1080-takes-runs`, `idea` P2, `owned_paths = ["src/runs.rs"]`, 1-line constant):**
   ~90% of the `Column` vs `Runs` gap on `mdjournal`/`elf` is branchless AVX2 bitmap RLE2 decoding in
   `runs::front` vs `mtf::mtf_avx2` branch mispredictions (`s <= 1 / run != 0`). Widening
   `src/runs.rs:78` from `2 * len > max_len` to `4 * len > 3 * max_len` (keeping `RUN_SHARE = 32`) cuts
   steal-free MTF+inv CPU **`-13.2..-32.4%`** on `mdjournal`, **`-14.5..-24.7%`** on `elf`, and
   **`-9.0..-9.9%`** on `json` across base/native/JCC-32B with **`0/549` blocks changed** on
   `wikidata`/`text`/`random`/`records`/`runs`/`onebyte`/`pattern`. Stacked with `#1145`
   (`COMBO_T5_R2m22`), off-Wikidata corpora drop **`-9%` to `-34%`** across the board. **`src/runs.rs` is
   currently unheld** — this 1-line safe-Rust change is the highest return-per-LOC uncontested win on
   the board once an open claim slot is available.
3. **Open PR `#1155` (`explore/bwt-segmented-dissection` → `segmented-fused-postwalk` at `idea`):**
   Proves shipped `room_segments::<32, PF_L1, true>` is already at the x86-64 silicon optimum (10 walk
   micro-redesigns STOP), while post-walk `assemble + RLE1 + CRC32` is `31.6–32.0%` of primary `text`
   (`79–84%` of `runs`/`onebyte`). Fusing AVX2 reverse into a 4 KiB L1d buffer + RLE1 + 64 KiB PCLMULQDQ
   CRC32 + `u64`-store-combining `build_lf` (`M4_buf4k_u64`, `0.87–1.002×` peak buffer) cuts primary
   `text` another **`-17.2..-18.9%`** (6/6 disjoint) and `random`/`records` **`-5.0..-11.9%`**
   baseline/JCC-32B (native warm `-2.3% / -1.0%` noted as C2 caveat).

### 3c. Python whole-file & `decompress()` delivery (`#1147`, `python-zero-extra-copy`)

`#1147` showed that whole-file `mibz2.open().read()`, `BZ2Reader.read()`, and `mibz2.decompress()` still
pay `2.0–2.3×` peak RSS (`441–480 MiB` on `wikidata.bz2`) and `+14..40%` wall/CPU over native Rust due to
inherited `io.RawIOBase.readall()` (25,829 × 8 KiB `readinto` calls) and `PyBytes::new` double-copying.
A 100% safe-Rust `PyBytes::new_with_writer` prototype eliminates both (`-36..-46%` peak RSS, `-14..-40%`
wall, `-14..-21%` CPU), gated on an owner packaging decision (`abi3-py310` vs per-version CPython wheels,
recorded in `needs_louis`). Meanwhile an `abi3-py310`-compatible `BZ2Reader.readall()` 1 MiB chunking
subset needs no packaging change and immediately recovers `-7..-11%` CPU.

## 4. Critical-path & dependency bottleneck map

### Bottleneck #1 (P1): `pyproject.toml` `owned_paths` overlap blocks `release-0-5-15`

`#1149` established that releases are P1 once recognised and opened `release-0-5-15` (`status = "open"`,
unheld, `owned_paths = ["CHANGELOG.md", "Cargo.toml", "pyproject.toml"]`). However, `jcc-aligned-release-build`
(P1, currently held while PR `#1119` awaits independent C6 review) lists
`owned_paths = ["pyproject.toml", ".github/workflows/CI.yml", ".just/bench.just", "bench/jcc_aligned.py", "bench/test_jcc_aligned.py"]`.
**PR `#1119` does not modify `pyproject.toml` or `.github/workflows/CI.yml`** — its body explicitly defers
release-flag edits to a follow-on PR after an i9 PASS — yet holding `pyproject.toml` mechanically blocks
`frontier.py claim release-0-5-15` (and its hold on `.just/bench.just` simultaneously blocks P2
`forward-emission-run` and P2 `mtf-fold-run`).

**Single highest-leverage unlock on the entire frontier:** complete independent C6 review and merge of
PR `#1119` (releasing its hold while awaiting the i9 run, or narrowing its `owned_paths` to the recipe
files until i9 PASS). That one review simultaneously unblocks:
1. **P1 `release-0-5-15`** (`pyproject.toml`),
2. **P1 i9 `jcc-aligned-release-build` queueing** (settling JCC-32B and `k = 8` attribution), and
3. **P2 `forward-emission-run`** (`.just/bench.just`).

### Bottleneck #2: Independent review queue gating the next hardware & code waves

| Open PR | Claim / Zone | Gate needed | What it directly unlocks on merge / release |
|---|---|---|---|
| **#1119** | `jcc-aligned-release-build` (P1) | Independent C6 review | **`release-0-5-15` (P1)**, i9 JCC/knee attribution, `forward-emission-run` (P2) |
| **#1144** | `scanner-adaptive-seam-window` (P2) | Independent C6 review (31/31 CI green) | `~-10.3 ms` derived `S'`, `-72.6%` window mem, releases `src/parallel.rs` (`slack-hybrid-scan`) |
| **#1132** | `mtf-fold-l1-batch-redesign` (P2) | Independent B5 review (29/29 CI green) | `mtf-fold-run` i9 validation, releases `src/open.rs` (`post-1080-open-admission`) & `src/open/outq.rs` (`a6c-wire-redesign`) |
| **#1139** | `text-knee-i9` (P2) | Independent C6 review (24/24 CI green) | Hardware confirmation of off-Wikidata `k* ≈ 4.6–6.8` scanner binding |
| **#1124** | `Claim: none` (`run_requests.py`) | Independent review (O2 security/authority fixes) | Safe ref-based workstation dispatch (4 fail-open defects in `#1104`) |
| **#1093** | `workstation-pressure` (P1) | Independent review (`frontier.py` + tests) | Automated workstation replenishment pressure in `brief`/`status`; releases `frontier/tools/` (`agent-presence`) |
| **#1117** | `scanner-a2-fast-build` (P2) | LTO/alignment check + registered i9 `S'` gate | `~-6.4..-10.7 ms` derived `S'`; releases `src/block.rs` & `src/huff.rs` (`scanner-a3-k3-pipe-u8`) |
| **#1156** | `shipped-ledger` (P1, smooth) | CI completion & merge | Unblocks P2 `claims-match-state` and P2 `blog-cards-cite-ledger` |
| **#1155** | `explore` (`bwt-segmented-dissection`) | CI completion & merge | Lands `segmented-fused-postwalk` (`idea`) and 10 scoped `room_segments` dead-ends |
| **#1065** | `migrate-rules` (P2) | Owner (`RULES D1`) | Migrates `RULES.md` off coordinator framing |

### Bottleneck #3: Ledger & communication pipeline (`#1146`, `#1148`, `#1149`, `#1150` → PR `#1156`)

`#1146` and `#1150` established the blog-vs-notebook boundary (external reader posts carry news/discoveries/advances
with zero internal shorthand; team digests and negative results go to `docs/notebook/`). `#1148` opened
the 3-claim ledger chain so blog news and releases draw directly from merged reality rather than
narratives: `shipped-ledger` (P1, PR `#1156` open, backfilling all 10 software PRs since `v0.5.14`: 5
`default` `#1040`, `#1033`, `#1034`, `#1080`, `#1129` and 5 `experimental` `#895`, `#1010`, `#1041`,
`#1053`, `#1110`) → `claims-match-state` (P2) + `blog-cards-cite-ledger` (P2). Once `#1156` lands, both
P2 claims can proceed immediately in parallel.

## 5. Release

**`release-0-5-15` is now P1** (`#1149`, opened under amended `docs/RELEASING.md` §1). Five default-path
changes (`#1129` bounded decoder memory, `#1080` faster `Segmented` fallback, `#1040` dense-band precheck,
`#1033` Python single-copy reads, and `#1034` coalesced scanner decode kept by owner override) plus five
off-by-default experimental/harness commits are ready to ship in `v0.5.15`, with `v0.5.14` vs `master`
paired CI and i9 release-scaling runs measuring the combined effect directly. Claim and execute
`release-0-5-15` immediately once `#1119`'s `pyproject.toml` hold clears.

## 6. Dead ends & newly implied claim

All 40 rows in `frontier/dead-ends.tsv` stand intact (C2); nothing is revived. Since Pass 13, five new
scoped rows landed: `#1141` non-adaptive `want=1`/`128 KiB` lookahead (`STOP`), `#1147` `Vec<Vec<u8>>`
chunked-drop & `PyFile` scratch-`PyByteArray` (`STOP`), `#1151` `#1034` i9 frozen gate (`STOP`, verdict
preserved beside Louis's separate owner keep), `#1151` `#1053` fixed-64K `1.75 MiB` A6b SoA (`STOP/KILL`),
and `#1153` `B=9 K=3` / `B=11 K=2` / split-24 KiB-LUT / 15th-GPR spill ablations (`STOP`). This pass
opens one newly implied claim at `status = "idea"`: **`frontier/claims/a6b-compact-footprint-rescue.toml`**
(P3, `explore`, smooth zone) to test compact `n_nodes`-sized `A6b` slabs obeying the `<= 1.01×` 8W RSS
L3-footprint law before shelving `A6b`.
