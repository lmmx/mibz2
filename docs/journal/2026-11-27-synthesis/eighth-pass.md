# Synthesis, eighth pass: unifying the 4-lane vectored pipeline, closing the 6 GB/s contour with paid anchors and hoisted `OutQ16`, Intel JCC alignment control, and exact PR integration order

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml), 27 November 2026. Follows the [seventh pass](seventh-pass.md) (`a8bcdd7`, #1052). Analytical and read-only across open PRs: no `src/` edits, no new hardware runs, no frozen gate reopened (C2). **(derived)** marks cross-source arithmetic or conditional projections kept strictly separate from established i9-10900K measurements per [`frontier/DECODER.md`](../../../frontier/DECODER.md).*

## Primary sources read (post-`a8bcdd7` / #1052)

- **Living state:** `frontier/DECODER.md`, `frontier/surface.tsv`, `frontier/dead-ends.tsv`, `frontier/claims/*.toml`.
- **Merged on `origin/master` since seventh pass (`a8bcdd7..63e94c1`):**
  - `#1054` (`0cceaa1`): `docs/site/blog/2026-11-27-reopening-a6b-and-segmented.md` (published A6b + Segmented post).
  - `#1045` (`e61ec12`): `docs/journal/2026-11-27-explore/forward-rle1-crc-fusion.md` (`forward-rle1-crc-fusion` PASS for 4-lane vectored zero-scratch tile fusion; 1-lane, FSM, stitched STOP; `forward-rle1-crc-vectored-build` at `idea`).
  - `#1056` (`72286cf`): `docs/journal/2026-11-27-explore/slack-hybrid-scan-feasibility.md` (`explore`: slack-only hybrid scanning Q1–Q3, `140 ms` contour, `k = 6..10` safety, `Arc<Chunk>` + feeder `OnceLock` `Finder`).
  - `#1055` (`3a2b770`): `docs/journal/2026-11-27-explore/mtf-l1-batch-rescue-probe.md` (`Claim: none` PASS: L1d `[u32; 256..512]` packed maximal MTF fold + `OutQ16` `496/512` guard zone, `objdump` zero-spill audit).
  - `#1057` (`7f5a1c6`): `docs/journal/2026-11-27-explore/forward-4lane-anchor-probe.md` (`forward-4lane-anchor-probe` PASS: paid `S = 16` bit-31 sampled-anchor `A6c+A7`, dynamic 4-lane `32 KiB` slab continuation).
  - `#1058` (`28c4c8c`): `docs/journal/2026-11-27-explore/vectored-delivery-boundary.md` (`Claim: none`: zero-cost vectored delivery, `+6.25%` slab sizing vs `span/4` `1.178×` RSS, Python `#1033` crossings, `ReadyBlock` spec).
  - `#1059` (`63e94c1`): `docs/journal/2026-11-27-explore/mtf-l1-batch-and-outq32.md` (`Claim: none`: Intel JCC-erratum 32B alignment control, hoisted single-branch `OVF_BIT` `f + w <= C` `OutQ16` PASS, standalone `shipped_q496_hoist`, `OutQ32` STOP).
- **Open PR queue inspected (read-only metadata; unmerged, not treated as master evidence):** `#1031`, `#1033`, `#1034`, `#1041`, `#1047`, `#1053` (and in-flight `#1060`, `#1061`, `#1062`).

---

## 1. The unstated architectural mismatch across `#1045`, `#1057`, and `#1058` — and how one unified slab-vectored pipeline resolves it

Pass 7 predicted that `forward-4lane-anchor-probe`, `forward-emission`, and `forward-rle1-crc-vectored-build` would converge on one 4-lane vectored boundary. Now that `#1045`, `#1057`, and `#1058` have all landed on `master`, reading them side by side exposes **three concrete incompatibilities that none of them mentions alone**:

| Interface parameter | `#1045` (`forward-rle1-crc-fusion`) | `#1057` (`forward-4lane-anchor-probe`) | `#1058` (`vectored-delivery-boundary`) | **Unified production spec required** |
|---|---|---|---|---|
| **Segment lengths known upfront?** | Yes (prebuilt `K = 4` equal-stride oracle) | **No** (paid `S = 16` uniform anchors have exponential order-stat lengths, `L_max/(m/4) = 0.818×`) | Assumed pre-sized contiguous slots for `K = 4` or `S = 16` | **Dynamic `32 KiB` (or `16 KiB`) slab-continuation** from `#1057`; no pre-sized segment slots |
| **Output slice count / block** | `4` slices (`~220 KiB` each) | **`41.6` ordered slices/block** (`16` segments + `25.6` slab switches; `~21.6 KiB` avg) | `ReadyBlock` hard-capped **`MAX_BLOCK_SLICES = 16`** (`160 B` inline) | **Raise cap to `MAX_BLOCK_SLICES = 64`** (`544 B` inline, `< 18 KiB` across 32 slots) or carry a recycled `Vec<(u32, u32)>` beside `buf` in `free_out` |
| **Buffer slack & RSS gate (`<= 1.05×`)** | Used `span/4` (`+25%`) per lane → **`1.178×` shipped RSS** (fails `<= 1.05×` gate!) | `max_arena = 992 KiB` inside `out` (`1.062×` max block `934 KiB`; `<= 1.02×` process RSS) | Proved `+6.25% + 4 KiB/lane` = `1.017×` for known-span lanes | **Run `#1045`'s zero-scratch in-place RLE1 compaction + `crc::update_with` per `32 KiB` (or `16 KiB`) slab as it closes** (spilling rare net-growth suffix to one tiny `32 KiB` worker scratch); bounds buffer to `n + ~3–4 × SLAB` (`~964–992 KiB`, **`<= 1.05×` RSS**) with `0` `span/4` over-allocation |
| **Python `#1033` & `Read::read` crossings** | Deferred to `parallel.rs` | Deferred to downstream | Said `#1033` needs **no change at `K = 4`** (`+0.0%` at 256K), but warns `S = 16` inflates 64K `+62.9%` / 256K `3.49×` | At `#1057`'s `41.6` slab slices (`~21.6 KiB`), single-`fill_buf()` would multiply 256K/1M calls `~10–40×`: **`StreamDecoder::read`, `BzReader::read`, AND Python `#1033` `Reader::read(size)` MUST scatter-fill across slices within the current `ReadyBlock`** (holding calls flat at `3,200` / `800`) |

**Why slab-level RLE1+CRC fusion inside `#1057`'s dynamic 4-lane emitter is strictly cleaner than fixed-span lanes:**
1. Within any single anchor segment (`seg 0..15`), consecutive `32 KiB` slabs are produced sequentially by the **same lane**, so the lane carries its exact `(last_byte, run_len)` RLE1 state and running `u32` CRC across slab boundaries with **zero intra-segment straddles** and **zero per-slab `crc32_combine`**.
2. Only the **`S - 1 = 15` inter-segment anchor joins** (vs `3` in `K = 4`) need `#1045`'s `O(1)` straddle repair (`0.24%` run-bearing words → near-zero hits, exact repair when hit) and `15` GF(2) `crc32_combine` calls (`~300 ns/block` = `0.07 ms` across 233 blocks).
3. Compacting each `32 KiB` slab in place while it is still **L1d/L2-hot** preserves `#1045`'s `-53.4%` write cut (`1,718 → 800 MiB`), L2-hot CRC win (`+49.9 ms` warm / `+31.8 ms` cold native), and `#1058`'s `0 ms` vectored delivery (`0.86–1.05×` vs `+33..+80%` stitched `memmove`) — while solving **both** unknown paid-anchor segment lengths AND the `<= 1.05×` RSS gate simultaneously.

---

## 2. Reconciling `#1055` and `#1059`: why hoisted single-branch `OutQ16` and Intel JCC-erratum control are mandatory

Reading `#1055` (`mtf-l1-batch-rescue-probe`) and `#1059` (`mtf-l1-batch-and-outq32-probe`) together resolves an apparent discrepancy and surfaces a repo-wide benchmarking hazard:

1. **The Intel JCC-erratum 32-byte alignment landmine (`20–40 ms` swings):** Both the Haswell Xeon VM and Louis's Comet Lake **i9-10900K** carry Intel's JCC-erratum microcode mitigation (jumps crossing or ending on a 32-byte boundary drop out of the DSB uop cache). In `#1059`, two instruction-for-instruction identical `flush_batch` functions differed by **`20.9–27.9 ms`** on default `-C target-cpu=native` (`0x3395c..0x33965` crossing `0x33960`); compiling with `-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp` collapsed the gap to **`0.04 ms`** (`127.58` vs `127.62 ms`) and dropped `shipped_split` itself by **`-40.0 ms`** (`195.76 → 155.81 ms`). On `wikidata ×4` (`1,306 ms` 1W), a `20–40 ms` alignment swing equals **`1.5%–3.1%` of 1W wall** — the entire width of a `2.5%–3.0%` adoption gate!
2. **Why `#1055`'s unaligned PASS had hidden fragility — and how `#1059` locks in `2×` alignment-invariant headroom:**
   - Under JCC-aligned control, `#1048`'s baseline proposal without hoisting (`batch256_q496`) saves only **`-17.18 ms`** (`-10.54%` `A5+A6a`, `-12.75 ms` bidi / `-16.00 ms` fwd joint) — roughly **`1.85%–2.49%` of 1W worker CPU**, sitting right on or below `mtf-fold/registration.md`'s frozen `2.5%` K1 bar if an i9 release binary aligns `shipped_split` favorably.
   - Combining all **five safe-Rust codegen invariants** from `#1055` + `#1059` **doubles the true alignment-invariant gain**:
     1. Out-of-line `#[cold] #[inline(never)] second_cold` + direct `emit!` inside `if w != 0` (`#1055`: `24`-insn pure-128b-XMM hot loop, `0` `f0`/`f1` spills, `0` `vzeroupper` in `fill_batch`).
     2. Fused Pass-1 `lits` popcount (`#1059`: `recs = lits + extra_recs`, drops separate `f.lit` popcount and per-literal `recs += 1`).
     3. Batch-hoisted `p` (`cur > p || cur + batch_rows <= p`) and `EPOCH` (`lo + max_m_add < EPOCH`) checks once per 256 runs (`#1059`).
     4. **`OVF_BIT` single-branch `OutQ16::push_hoisted` enforcing `f + w <= C` (NOT `f < C`)** (`#1059`: `let nf = f.wrapping_add(w); if nf <= C as u32` checks uninitialized, full/straddling, and overflow in one add + compare, keeping `fill[a] <= C` so `Reader` stays constant-`C` and `f + 16 <= S`).
     5. Register-local `m`, `in_out`, and batched `f.runs += batch.len()` (`#1055` + `#1059`).
   - Result on JCC-aligned native: **`batch256_q496_hoist` `-28.22 ms` (`-18.09%`)** and **`batch256_q1008_hoist` `-31.98 ms` (`-20.80%`)** on `A5+A6a`; joint `A5+A6a+A6c` **`-28.27 ms` (`-14.20%`) bidi** and **`-25.03 ms` (`-15.34%`) forward** (`~-4.1%..-4.6%` of 1W worker CPU; derived i9 `wikidata ×4` projection **`-60..-69 ms` 1W / `-8.6..-9.9 ms/w` 8W** — clearing K1 with `~1.8–2×` margin regardless of code alignment!).
3. **Standalone safe-Rust fallback (`shipped_q496_hoist`) & `OutQ32` scoped STOP:**
   - Even *without* `mtf-fold`, hoisting `OutQ16<496, 512>` alone delivers **`-15.85 ms` (`-9.54%`)** on `A5+A6a` and **`-12.5..-12.6 ms` (`-6.8%`)** joint (`~-32 ms` 1W / `-4.5 ms/w` 8W derived) with zero `unsafe`, zero feature flags, and zero `text` fallback impact.
   - Run-encoded `OutQ32` (`(m << 4) | (w - 1)`) is a **scoped STOP**: `-20.7%` on `A5+A6a` explodes `A6c` by `+94..+118 ms` (net **`+40.6%` bidi / `+34.6%` forward**) by destroying SIMD `vpmovzxwd` sequential wiring. *(Pending row for `frontier/dead-ends.tsv` once open PR `#1060` merges cleanly.)*

---

## 3. Updated 1W / 8W / 9W system projections **(derived)** — hoisted `mtf-fold` is the decisive swing lever for 6 GB/s at `k = 8`

Every unpaid gap from Pass 7 is now replaced by a measured PASS probe (`#1057` paid `S = 16` anchors replacing the unpaid 4-lane oracle; `#1059` JCC-aligned hoisted `mtf-fold` replacing the `40–85 ms` census range; `#1056` validated discrete-event replay replacing the rough `+5 ms` coupling estimate).

**Established i9-10900K baselines (`wikidata ×4`, 839 MB; `c71c4a56`, `cc8ba137`, `character-stage1` `bb72107b`, `results-2`):**
- 1W wall = `1,306 ms` (`A5 132.4`, `A6a 200.3`, `A6b 216.0`, `A6c 161.4`, `A7 490.3`, `A9 66.6–74.4`, `D 29.1 ms`).
- 8W shipped wall `none(8) = 192.9–194.6 ms` (`4.31–4.35 GB/s`); `S_total = 185.0 ms` (`S_core 174.8 + in_maint 10.2–11.2`); `W/8 = 184.9–185.7 ms`.
- Target milestone: `6 GB/s` = **`140.0 ms`** on `839 MB`.

**End-to-end 4-lane vectored worker pipeline **(derived)** (all stages conditional on composed measurement and frozen i9 runs):**

| Stage / Claim | Mechanism (all probes PASS, 0 new `unsafe`) | Derived 1W saving (`%` of `1,306 ms`) | Derived 8W saving (`ms/w` off `185.0`) | L2/L3 traffic & RSS effect |
|---|---|---:|---:|---|
| **`A5+A6a`** (`mtf-fold-l1-batch-redesign`, `#1055`+`#1059`) | `[u32; 256..512]` L1d batch + `second_cold` + fused `lits` + hoisted `p`/`EPOCH` + `OVF_BIT` `f+w<=C` `OutQ16<496,512>` + local `m` | **`+60.0..+69.0 ms`** (`+4.6%..+5.3%`) *(standalone `shipped_q496_hoist`: `~+32 ms`)* | **`+8.6..+9.9 ms/w`** *(standalone: `~+4.5 ms/w`)* | `−124.5 MiB` `runs` traffic; removes `362 KiB/w` HWM |
| **`A6b`** (`a6b-endpoint-build`, `#1046` → `#1053`) | Fused setup + packed `u64` + safe 64K `u16`-link `inline15` SoA + `aw==bw` in-place + gen-tagged L1d dict | **`+48.7..+67.7 ms`** (`+3.7%..+5.2%`) | **`+8.2..+9.1 ms/w`** | `+273–420 KiB/w` touched during A6b (`16.5%` time) |
| **`A6c+A7`** (`forward-emission`, `#1057` **PAID**) | Unmodified SIMD `Reader::wire` + `O(S)` bit-31 `S=16` anchors + dynamic 4-lane `32 KiB` slab-continuation (`S16_vec`) | **`+208.4..+235.4 ms`** (`+16.0%..+18.0%`) *(stitched Phase A: `+174.9..+208.3 ms`)* | **`+28.4..+32.8 ms/w`** *(stitched: `+23.5..+28.9`)* | `<= 992 KiB` arena inside `out` (`0 B` extra buf) |
| **`A9` fused** (`forward-rle1-crc-vectored-build`, `#1045`) | Zero-scratch in-place slab RLE1 compaction + 64 KiB tiled CRC32 + `15×` GF(2) `crc32_combine` | **`+31.8..+49.9 ms`** (`+2.4%..+3.8%`) | **`+4.0..+6.7 ms/w`** *(before L3 relief)* | **`−918.3 MiB`** (`−53.4%`) `E+A9` writes; drops cold A9 reread |
| **`D`** (`ReadyBlock`, `#1058`) | Zero-stitch vectored `ReadyBlock` (`>= 64` slices) + scatter-fill `Read`/Python | **`0.0 ms` tax** (avoids `−64..−88 ms` stitch) | **`0.0 ms/w`** | `<= 1.05×` RSS with slab-level compaction |

**Plugging the paid worker stack into `#1056`'s validated 8W slack-hybrid contour (`wall_8W ≈ (8·W'/8 + S')/9 + 4.85 ms`, or `+3.50 ms` with small chunk 0; `S' = 145 ms` [range `143–150 ms`] from `#1031 + #1034`) **(derived)**:**

| Worker stack configuration | Derived 1W wall | Derived `W'/8` | Serial 8W wall (`max + ~5.5 ms`) | Hybrid `H1` 8W (`4 MiB` / small chunk 0, `S'=145`) | Meets `<= 140.0 ms` (`6 GB/s`) @ `k = 8`? |
|---|---:|---:|---:|---:|---|
| **3-mech (no `mtf-fold`):** `a6b` + PAID `S16_vec` + vec `RLE1+CRC` | `953–1,017 ms` (`+22.1–27.0%`) | **`136.4–144.4 ms`** | `150.5–150.5 ms` | **`142.2–149.3 ms`** / `140.9–148.0 ms` | **NO** (sits `0.9–9.3 ms` above `#1056`'s `W'/8 <= 133.9–135.4 ms` cutoff) |
| **3-mech + standalone `shipped_q496_hoist`** (no `mtf-fold`) | `921–985 ms` (`+24.6–29.5%`) | **`131.9–139.9 ms`** | `150.5 ms` | **`138.2–145.3 ms`** / `136.9–144.0 ms` | **Borderline** (optimistic half crosses `140 ms`) |
| **3-mech + hoisted `mtf-fold` (no `RLE1+CRC` fusion yet)** | `917–989 ms` (`+24.3–29.8%`) | **`133.2–139.8 ms`** | `150.5 ms` | **`139.4–145.2 ms`** / `138.0–143.9 ms` | **Borderline** (knife-edge; needs `RLE1+CRC` for margin) |
| **Full 4-mech stack (with hoisted `mtf-fold` + vec `RLE1+CRC`)** | **`884–957 ms`** (`+26.7–32.3%`) | **`126.5–135.8 ms`** | `150.5 ms` | **`133.4–141.7 ms`** / **`132.1–140.3 ms`** (`5.98–6.35 GB/s`) | **YES** across nearly the entire band (up to `+7.9 ms` margin; **`~124–133 ms`** / `6.3–6.8 GB/s` at `k = 9`) |

**Cross-cutting conclusion none of `#1056`, `#1057`, or `#1059` stated alone:**
- Pass 7 used the *unpaid* 4-lane oracle (`+28.9..+36.5 ms/w`) and concluded that a no-fold worker stack might already reach `6 GB/s` at 8 workers.
- Once `#1057` prices paid `S = 16` anchors (`+28.4..+32.8 ms/w`) and `#1056` prices the exact `+4.85 ms` hybrid coupling floor (`2.62 ms` startup + `1.15 ms` handoff residual + `~1.08 ms` drain/sched), **all four worker mechanisms — specifically including `#1059`'s hoisted `mtf-fold-l1-batch-redesign` AND `#1045`'s vectored `RLE1+CRC` — are needed together** to push `W'/8` cleanly below `#1056`'s `133.9 ms` (`135.4 ms` small chunk 0) `6 GB/s` contour at `k = 8`!
- In addition, `#1056`'s **small initial chunk 0** (`in_wait`: `1.40 → 0.05 ms` in `src/read.rs:13`) hits 8W wall **1:1** (`-1.35 ms`, worth `12.1 ms` of scanner work under `÷9` elasticity!) and is a trivial, zero-risk addition to `pooled-scan-hybrid` (or an independent smooth-verified micro-PR).

---

## 4. Exact file-level integration DAG for the 6 ready PRs and 5 next builds

Inspecting the live file diffs and CI status of the 6 ready open PRs (`#1031`, `#1033`, `#1034`, `#1041`, `#1047`, `#1053`) shows **three completely disjoint PRs** and **one 3-PR overlap cluster (`#1031`–`#1041`–`#1053`)**:

### 4a. Ready PR integration order (minimizing rebases and i9 idle time)

| Wave | PR & claim | Touched production / shared files | CI & gate status | Unlocks |
|---|---|---|---|---|
| **Wave 0 (parallel, 0 conflicts)** | **`#1047`** (`release-terminal-holds`) | `frontier/tools/frontier.py`, `tests/test_frontier.py` | **24/24 green**; ordinary integration | Reconciles stale `hold/archive-team-strata` → unblocks **`migrate-rules`** |
| **Wave 0 (parallel, 0 conflicts)** | **`#1033`** (`python-read-single-copy`) | `src/python.rs`, `python/tests/test_decode.py` | CI running on `a0370c77`; 0 `unsafe` (RULES O1) | Ships `-3.1..-7.0%` Python delivery win; frees `delivery` mechanism |
| **Wave 0 (parallel, 0 conflicts)** | **`#1034`** (`scanner-coalesced-u8-decode`) | `src/bits.rs`, `src/block.rs`, `src/huff.rs` | **29/29 green**; needs B5 review of BMI2 dispatch + recipe check | Queues `~25–30 ms` serial-scan cut (`S` → `~143–150 ms` with `#1031`) |
| **Wave 1 (parallel pair, disjoint!)** | **`#1031`** (`scanner-seam-window`) | `src/parallel.rs`, `src/trace.rs`, `.just/bench.just`, `docs/ledger.md` | **32/32 green**; 1 C6 adoption review | Immediately queues **i9 #1** (`bench-scanner-seam-window`, `-11.2 ms` `in_maint` + `2.15 ms` coupling); provides seam window required by `pooled-scan-hybrid` |
| **Wave 1 (parallel pair, disjoint!)** | **`#1053`** (`a6b-endpoint-build`) | `src/open.rs`, `src/open/endpoint.rs`, `Cargo.toml`, `.github/workflows/{miri,test}.yml` | **31/31 green**; 0 `unsafe`, research feature | Immediately queues **i9 #2** (`a6b-endpoint-build`, `+49..+68 ms` 1W / `+8.2..+9.1 ms/w` 8W); frees `src/open/endpoint.rs` |
| **Wave 2 (1 rebase onto Wave 1)** | **`#1041`** (`mtf-fold-run`) | Overlaps `#1031` (`src/parallel.rs`, `src/trace.rs`, `.just/bench.just`) AND `#1053` (`src/open.rs`, `Cargo.toml`) + owns `src/runs.rs`, `src/runs/fold.rs`, `src/scan_oracle.rs` | **32/32 green**; needs B5 review of `scan_oracle.rs` FFI | Merge **WITHOUT queueing `#1010` on i9**: lands reusable zero-copy `S`/`M`/`C` oracles + `bench-mtf-fold` AND frees **`src/runs.rs`** (`segmented-fallback-build`) and **`src/runs/fold.rs` + `src/open.rs`** (`mtf-fold-l1-batch-redesign`, `forward-emission`) |

*(Merging Wave 1 `#1031` + `#1053` before Wave 2 `#1041` requires only **one** rebase (`#1041`) instead of two and starts i9 runs on `#1031` and `#1053` immediately. In-flight smooth PRs `#1060` [edits `dead-ends.tsv`], `#1061` [blog], and `#1062` [composed 4-mechanism stack probe] are disjoint from Waves 0–2.)*

### 4b. Sequenced execution order of the 5 next builds

1. **`segmented-fallback-build`** (`src/bwt.rs`, `src/runs.rs`; P2, open): Start immediately after `#1041` frees `src/runs.rs`. Zero i9 slot needed (CI paired bench under `PERFORMANCE-POLICY.md` §2); ships `-7.5%..-10.3%` derived worker time on primary `text` (`0` extra memory, `0` Wikidata impact).
2. **`mtf-fold-l1-batch-redesign`** (`src/runs/fold.rs`, `src/open.rs`, `src/open/outq.rs`; P2, promote from `idea` after `#1041` + `#1053`): Build the **hoisted single-branch** design (`#1055` + `#1059`: `[u32; 256..512]` + `second_cold` + fused `lits` + batch-hoisted `p`/`EPOCH` + `OVF_BIT` `f + w <= C` `OutQ16<496, 512>` + local `m`), pass `objdump` zero-spill + **JCC-aligned local gate** (`-18.1..-20.8%` A5+A6a), then run `bench-mtf-fold` on i9 under frozen `mtf-fold/registration.md` §3. **Must precede `forward-emission` on `src/open/outq.rs`** so `Reader::wire` is built once against the `496/512` chunk layout (`#1059` already verified `-15.34%` on forward `A5+A6a+A6c`).
3. **`forward-emission`** (`src/open.rs`, `src/open/outq.rs`; P3 → **candidate for P2 bump**, single largest worker lever at `+208..+235 ms` 1W / `+28.4..+32.8 ms/w` 8W): Follows `mtf-fold-l1-batch-redesign` on `src/open.rs`. Implements `#1057`'s paid `S = 16` bit-31 anchor tagging + unmodified SIMD `Reader::wire` + dynamic 4-lane `32 KiB` slab-continuation emitter. Keep the internal emitter slice-producing (avoid baking a permanent contiguous stitch into `Open`) so Step 4 plugs in directly.
4. **`forward-rle1-crc-vectored-build`** (`src/open.rs`, `src/rle.rs`, `src/crc.rs`, `src/parallel.rs` + `src/read.rs` / `src/lib.rs` / `src/python.rs`; `idea`): Follows `forward-emission` (and `#1031` on `src/parallel.rs`). Implements §1's unified slab-level zero-scratch in-place RLE1 compaction + 64 KiB tiled CRC32 + `15×` GF(2) `crc32_combine` + `ReadyBlock` (`MAX_BLOCK_SLICES >= 64` or recycled `Vec<(u32, u32)>`) + scatter-fill in `StreamDecoder::read`, `BzReader::read`, and Python `Reader::read(size)`.
5. **`pooled-scan-hybrid`** (`write`, `idea`): Admitted once measured i9 worker runs cross `DECODER.md`'s **`S'(8) <= 165 ms`** trigger (which `#1053` + `forward-emission` clear even before `mtf-fold`). Draft `pooled-scan/hybrid-registration.md` from `#1056`'s six Q1–Q3 invariants (feeder-offloaded async `OnceLock` `Finder`, zero-copy `Arc<Chunk>` skipping ~11 seam blocks left to `#1031`, `full_only` `>= 512 B`, Policy A jump-ahead, `max_lookahead = 8..16`, worker-local `ScanScratch` + `Vec<u16>` running `#1034`, plus small initial chunk 0).

---

## 5. Living-state updates made in this pass

- **`frontier/DECODER.md`:** Concisely updated complementary paths (unified paid-anchor 4-lane + slab-fused RLE1+CRC + vectored `ReadyBlock` pipeline; hoisted L1d-batched `mtf-fold` + `OutQ16`; `#1056`'s slack-hybrid contour & feeder architecture) and added the **Intel JCC-erratum 32-byte alignment control** methodological note for Haswell/Comet Lake tight-loop benchmarks.
- **`frontier/surface.tsv`:** Updated rows `S`, `C-A6a`, `C-A6b`, `M`, `R`, and `D` with merged post-`#1052` evidence (`#1055`, `#1056`, `#1058`, `#1059`, and PR `#1053` status).
- **Conflict hygiene:** Left `frontier/dead-ends.tsv` untouched to avoid conflicting with open PR `#1060` (noting `OutQ32`'s scoped STOP from `#1059` should be appended once `#1060` merges), touched zero held claims' files, and kept recurring `synthesise.toml` at `status = "open"`.
