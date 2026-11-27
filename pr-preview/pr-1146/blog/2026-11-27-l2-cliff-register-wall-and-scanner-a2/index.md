[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1146/blog/index.md>)

# The 256 KiB L2 cliff, the 15-register wall, and the flipped eight-worker knee

Composing the worker stack hits two sharp hardware walls while safe header decoding cuts serial parsing by a third and flips the eight-worker knee.

27 November 2026 · 11 min read · [Research synthesis](<#5-disciplined-stops-and-the-workstation-validation-queue>)

**Composing the full Open worker stack turned clean stage projections into concrete microarchitectural ceilings.** Our previous post, [The architecture of a 6 GB/s bzip2 decoder](<https://lmmx.github.io/mibz2/pr-preview/pr-1146/blog/2026-11-27-six-gbs-decoder-architecture/index.md>), assembled five isolated designs across the worker pipeline and the eight-worker schedule. Since then, eight new studies composed the worker end to end and probed its cache and register ceilings. They also redesigned serial header parsing in safe Rust and audited branch alignment across the whole decoder.

One evidence boundary belongs up front. The 10-core Intel i9-10900K baselines are direct workstation measurements. On 839 MB of `wikidata ×4`, shipped mibz2 runs `1,306 ms` single-worker, `192.9–194.6 ms` eight-worker, and `179 ms` in the serial scanner `S`. The eight new probes ran on a 2-vCPU cloud VM. That host exposes Skylake/Cascade Lake Xeon silicon masked to Haswell CPUID `0x3F`, with 32 KiB L1d, 256 KiB L2, and 45 MiB L3. Every VM timing uses steal-free thread CPU across 7 interleaved repeats pinned to CPU 0. Every arm passed byte-exact parity against libbzip2, and every modeled i9 figure awaits direct workstation verification.

[Section 1](<#1-composing-the-worker-and-hitting-the-256-kib-l2-cliff-1062-1067>) composes the four worker mechanisms and diagnoses the 256 KiB L2 slab cliff and cold-path register spills. [Section 2](<#2-proving-four-coupled-lanes-are-the-x86-64-optimum-1068-1076>) explains why literal splitting, wider lanes, and decoupled batch-chase all hit hard hardware walls. [Section 3](<#3-safe-rust-header-parsing-flips-the-eight-worker-knee-1069-1074>) details three safe-Rust header mechanisms that cut serial `A2a..d` by a third and flip the eight-worker bottleneck. [Section 4](<#4-the-invisible-15-ghost-32-byte-branch-alignment-1077>) audits Intel's JCC-erratum 32-byte branch alignment across every station and separates VM silicon from target CPUs. [Section 5](<#5-disciplined-stops-and-the-workstation-validation-queue>) records two off-critical-path stops (`#1060`, `#1063`) and lists the clean workstation validation queue.

**Takeaways**

- **Worker composition is exact, and 256 KiB L2 caps lane slabs at 32 KiB.** Combining all four Open mechanisms (`#1062`, `#1067`) matches the sum of isolated wins within `0.07%` and halves unfused memory (`6,893 → 3,488.5 KiB/worker`). Enlarging lane slabs to `64 KiB` fills 100% of private L2 (`4 × 64 = 256 KiB`) and regresses `A7 + A9` by `+12.1` to `+22.2 ms`. Keeping `32 KiB` slabs and outlining cold paths cuts L3-cold worker CPU by `21.6%` to `24.6%`.
- **Four coupled lanes with unified `inline16` are the x86-64 optimum.** Splitting 1-byte literals (`#1068`, STOP) regressed all 24 isolated `A7` cells (`+4.1%` to `+39.0%`) by trading a `93.57%`-taken branch for a `44.59%` coin flip. Pushing to `K = 6, 8, 10` (`#1076`, STOP) hit the 15-GPR register-file wall (`+10.8%` to `+45.4%` slower in `A6c + A7 + A9`). Decoupling batch-chase broke out-of-order overlap (`+56.0 ms` even at `K = 4`).
- **Safe-Rust header parsing cuts `A2a..d` by a third and flips the eight-worker knee.** Three `#![forbid(unsafe_code)]` header kernels (`#1069`, PASS) cut standalone `A2a..d` by `31.6%` to `37.7%`. Full post-`#1034` serial `S'` drops `4.44%` to `7.38%`, or `−6.4` to `−10.7 ms` modeled on i9. Modeled `S'` falls to `134–139 ms` (`#1074`). That puts `k = 8` back on the worker side (`140.0–145.5 ms` L3-cold) and shifts slack-only hybrid scanning's main gain to `k = 9–10`.
- **32-byte branch alignment removes a 15% VM ghost.** Passing `-x86-branches-within-32B-boundaries` (`#1077`) removes `1,746` unfused crossings at `+3.4%` `.text`. It speeds up `12/12` off-Wikidata cells and cuts `OpenCells` by `13.7%` to `15.6%` on VM Xeon silicon. Comet Lake-S stepping 5 fixes the erratum in silicon, and AMD Zen pays padding overhead. Alignment is therefore a mandatory VM benchmark control rather than an untested release flag.
- **Two off-path stops preserve focus.** Column-route MTF (`#1060`, STOP) hits a `37%–50%` RLE2 branch-misprediction floor across 20 AVX2 arms. `Cooperative` (`#1063`, STOP) stays unreachable on `<= 10` cores and Amdahl-capped at `−5.2%` to `−6.2%` on two threads.

## 1\. Composing the worker and hitting the 256 KiB L2 cliff (`#1062`, `#1067`)

Isolated stage benchmarks can mislead if two optimizations fight for registers or cache capacity. [The composed worker stack probe](<https://github.com/lmmx/mibz2/blob/0549fdf1dfb7f1d2af1fd520721c2ebbe448ddc6/docs/journal/2026-11-27-explore/composed-worker-stack-probe.md>) (`#1062`) stacked all four Open worker wins inside one binary. It combined L1d-batched MTF with `OutQ16` guard zones (`#1055`), safe 64K SoA `A6b` (`#1046`), sampled-anchor 4-lane emission (`#1057`), and zero-scratch tiled RLE1+CRC fusion (`#1045`).

We tested all 233 Open-admitted Wikidata blocks (`209,691,304 B`) with byte-exact CRC32 parity and zero overflow chunks. On warm baseline x86-64, composition was nearly perfectly additive. Shipped `W0` dropped from `737.06 ms` to `548.46 ms` (`−25.59%`). The actual saving (`+188.60 ms`) matched the sum of the four isolated savings (`+189.11 ms`) to within `−0.52 ms`, or `−0.07%` of `W0`.

Fusing RLE1+CRC into the 4-lane emitter also rescued the working set. Unfused vectored emission (`W3b`) ballooned to `6,893.2 KiB/worker` (`53.9 MiB` across eight workers) because it buffered raw emitted slabs alongside expanded output. Fusing compaction in place (`W4`, `32 KiB` slabs) cut working set in half to `3,488.5 KiB/worker` (`27.3 MiB` across eight workers). That sits `913.9 KiB` below stitched `W3`.

| Composed Open worker arm (233 Wikidata blocks) | Working set / worker | 8W total | Key mechanism change |
| --- | ---: | ---: | --- |
| `W0_shipped` | `2,896.6 KiB` | `22.6 MiB` | Shipped `OpenCells` (`front.runs` + `wire_xor` + 2-lane) |
| `W1_plus_l1_mtf` (`+#1055`) | `2,140.2 KiB` | `16.7 MiB` | Drops 1 MiB `front.runs` (`−756.4 KiB`, `−26.1%`) |
| `W3_plus_fwd4_stitch` | `4,402.4 KiB` | `34.4 MiB` | 4-lane forward + contiguous stitch buffer |
| `W3b_plus_fwd4_vec_unfused` | `6,893.2 KiB` | `53.9 MiB` | Unfused vectored raw slabs + separate RLE1 output |
| `W4_v2_q1008_32k` (`#1067`, `32 KiB` slabs) | `3,486.5 KiB` | `27.2 MiB` | Full fused stack + `OutQ16<1008, 1024>` (`−2.0 KiB` vs `W4_v1`) |
| `W4_64k_slab` (`#1067`, `64 KiB` slabs) | `3,813.6 KiB` | `29.8 MiB` | `4 × 64 KiB = 256 KiB` active slabs (evicts private L2) |

### Why 64 KiB slabs fall off the 256 KiB L2 cliff

[The second composition round](<https://github.com/lmmx/mibz2/blob/c149eb696d6f474da619fff3b5ef7dbbaefec2ff/docs/journal/2026-11-27-explore/composed-worker-v2-probe.md>) (`#1067`) tested a tempting refinement. Doubling lane slabs from `32 KiB` to `64 KiB` would cut `compact_and_crc_slab` calls in half and emit fewer vectored slices.

Instead of speeding up `A7 + A9`, `64 KiB` slabs regressed it in every build and cache state. Native warm slowed by `+12.05 ms` (`232.93 → 244.98 ms`), and native L3-cold slowed by `+21.30 ms` (`261.28 → 282.58 ms`). Baseline L3-cold regressed `+20.03 ms` (`274.55 → 294.58 ms`), and JCC-aligned L3-cold regressed `+22.23 ms` (`244.74 → 266.97 ms`). Total worker CPU slowed by `+12` to `+25 ms` while adding `+325.1 KiB/worker`.

The arithmetic explains the cliff immediately. Both our Haswell-masked Xeon VM and our target Comet Lake i9-10900K have a **256 KiB private L2 cache per core**:

\\\[4\\text{ active lanes} \\times 64\\text{ KiB/slab} = 256\\text{ KiB} = 100\\%\\text{ of private L2}\\\]

While four lanes chase `edges` (`456 KiB` average per block) and write into four active slabs at once, `4 × 64 KiB` fills the entire private L2 cache. Earlier cache lines of each slab evict to L3 before the lane fills it. The in-place RLE1+CRC compaction pass then pays L3 read-modify-writes. Halving the slab to `32 KiB` keeps the four active slabs at `128 KiB`, or `50%` of private L2. The other half stays free for `edges` and `inline16`, so compaction reads hot L2 lines.

### What `objdump -d` caught in the hoisted `A5 + A6a` loop

Round 1 of `#1067` turned up a puzzling `+23.3 ms` regression on native. Folding `OVF_BIT` into `OutQ16::fill` and hoisting `EPOCH` across the L1d batch slowed `A5 + A6a` from `145.93` to `169.22 ms`. Disassembling the hot loop exposed two completely different culprits:

1. **A 32-byte branch-boundary crossing.** At `0x41e7a`, `cmp $0x7f, %r14; jbe` crossed `0x41e80`. Turning on 32-byte branch alignment dropped `A5 + A6a` by `−39.7 ms` (`169.22 → 129.52 ms`) without changing one line of Rust.
2. **Inlined cold-path register spills.** LLVM had inlined `flush_batch_fallback` and `Vec::reserve_exact` (`grow_edges`) into the hot batch-flush loop, spilling `batch.as_ptr()` to `0x28(%rsp)`.

Marking `compact_and_crc_slab`, `flush_batch_fallback`, and `grow_edges` as `#[cold] #[inline(never)]` eliminated both the stack spill and 5× macro code bloat. Against `W4_v1`, `W4_v2` cut total worker CPU across every profile. Native warm dropped `−17.25 ms` (`553.73 → 536.48 ms`), and native L3-cold dropped `−14.99 ms` (`609.75 → 594.77 ms`). JCC-aligned L3-cold dropped `−15.43 ms` (`566.87 → 551.44 ms`), and baseline L3-cold dropped `−35.15 ms` (`634.01 → 598.86 ms` with `OutQ16<1008, 1024>`). Keeping `SLAB = 32 KiB` with `#[cold]` compaction (`W4_b_32k_cold`) hit `503.65 ms` native warm (`−27.44%`, modeled `W'/8 = 133.89 ms`) and `566.09 ms` native L3-cold (`−24.59%`, modeled `W'/8 = 140.0 ms`). Across matched L3-cold builds, the composed worker saves `21.6%` to `24.6%`. That models `W'/8 = 140.0–145.5 ms` on the i9, or `148.9 ms` (`−19.8%`) against JCC-aligned `W0`.

## 2\. Proving four coupled lanes are the x86-64 optimum (`#1068`, `#1076`)

Even after `#1067`, `A7 + A9` still accounted for `46%` of composed worker time. Two clean questions remained. Could we skip the 16-byte dictionary lookup on 1-byte literals? Could we push lane count past `K = 4` toward all 10 Intel line-fill buffers? Both hypotheses got full probes, and both hit hard hardware ceilings.

### Why splitting 1-byte literals regressed 24 of 24 cells (`#1068`, STOP)

A census of `26,567,581` `A7` steps across Wikidata shows why direct-literal splitting looked attractive on paper. Single literal bytes (`code < 256`) make up `81.58%` of steps, with an average streak of `8.81`. Another `11.99%` emit a 2–16 byte inline word, and only `6.43%` hit the `LONG` slow path.

Yet [the direct-literal 4-lane probe](<https://github.com/lmmx/mibz2/blob/efb3dd9df2a9e31e1a39c30c9221f12db6e856f4/docs/journal/2026-11-27-explore/direct-literal-4lane-emit.md>) (`#1068`) found **all 24 isolated `A7` cells disjointly slower**. Across four candidate arms, three builds, and both warm and L3-cold caches, regressions ran `+4.12%` to `+38.98%`:

- `lit_branch_per_lane`: `+6.5%` to `+12.4%` slower.
- `lit_all4_fastpath`: `+12.6%` to `+23.5%` slower.
- `lit_streak_inner`: `+27.0%` to `+39.0%` slower.
- `lit_plus_compact_dict`: `+4.1%` to `+16.6%` slower.

The microarchitecture gives a two-part answer. First, the unified `inline16 + lens` dictionary averages only **`17.58 KiB` per block**, which sits 100% inside the 32 KiB L1d cache. Copying 16 bytes unconditionally from L1d runs almost for free inside the out-of-order shadow of four concurrent `edges[j]` cache misses. Second, the unified fast path takes one branch that succeeds **`93.57%`** of the time (`81.58% + 11.99%`). Splitting on `code < 256` checks whether all four lanes hold literals at once. That happens **`44.59%`** of the time — a near-maximal coin flip (`H = 0.992 bits`, average streak `2.66`). Every misprediction flushes the reorder buffer and stalls the in-flight `edges[j]` loads that hide memory latency.

### Why `K = 6, 8, 10` hit the 15-GPR register wall (`#1076`, STOP)

With unified `inline16` settled, [the MLP and layout probe](<https://github.com/lmmx/mibz2/blob/8e04854487ef92a54c0e30791fc8168fbf74ea90/docs/journal/2026-11-27-explore/w4-v3-mlp-layout-probe.md>) (`#1076`) tested 12 variants against `W4_v2_ctrl` (`K = 4, S = 16, SLAB = 32 KiB`). Each arm ran across all three builds and both cache states. The matrix covered wider coupled lanes (`K = 6, 8, 10` with `S = 48..64` and `K × SLAB <= 128 KiB`), decoupled batch-chase (`B = 8, 16`), 1-step software pipelining, hoisted `&mut arena`, and fused `Inline15Len`.

Not one of the 12 arms cleared the stop test. On JCC-aligned L3-cold runs, `W4_v2_ctrl` ran in **`551.46 ms`** (`A6c + A7 + A9 = 260.95 ms`), beating every wider or transformed variant:

- **Coupled `K = 6, 8, 10` hit the 15-GPR register wall.** Effective lane parallelism climbs from `3.15` at `K = 4` to `5.07`, `6.60`, and `7.68`. Yet coupled emission needs `4K = 24..40` live scalars (`j`, `ed`, `a`, `e`) on an architecture with 15 allocatable general-purpose registers. Disassembling `emit_klane_coupled::<8>` showed LLVM spilling all eight loaded edge words `ed0..ed7` (`0x30(%rsp)..0x180(%rsp)`) and reloading `a`/`e` every step. Because `edges` (`456 KiB`) mostly hits L3 (\~35 cycles) rather than DRAM, stack spills and `4×` more slab compactions swamped the extra LFB overlap. `A6c + A7 + A9` regressed **`+10.8%` to `+45.4%`**, slowing total L3-cold CPU by `+4.8%` to `+14.4%`.
- **Decoupled batch-chase (`B = 8`) destroys OoO overlap.** Separating edge chasing from dictionary expansion regressed `A7 + A9` by **`+56.0 ms` (`+22.7%`)** even at matched `K = 4, SLAB = 32 KiB`. At `K = 8`, total L3-cold CPU slowed `+8.2%` to `+14.7%`. Filling the reorder buffer with pure chases followed by pure expands prevents L1d copies from hiding behind L3 misses. Passing `&mut a` into the slow path adds a 5-cycle L1d store-forwarding stall on every step.
- **Secondary transforms also missed.** Hoisting `&mut arena` regressed `+4.1%` to `+20.7%` because a `6.43%` per-lane `LONG` rate exits the fast path on `23.3%` of `K = 4` rounds and `41.2%` of `K = 8` rounds. One-step software pipelining doubled live edge registers (`+3.8%` to `+15.3%`), and `S = 16 → 32` at `K = 4` stayed statistically flat (`+0.01%` to `+2.71%`).

Two hardware ceilings in the composed Open worker: the 256 KiB L2 cliff and the 15-GPR register wall

Diagnostic VM L3-cold medians of steal-free thread CPU across 7 interleaved repeats on all 233 Open-admitted Wikidata blocks (`#1067` and `#1076`). Both ceilings are architectural: `4 × 32 KiB = 128 KiB` fits inside half of private L2, and `K = 4` is the widest coupled lane count that keeps live lane state inside x86-64's 15 allocatable GPRs.

## 3\. Safe-Rust header parsing flips the eight-worker knee (`#1069`, `#1074`)

Once `#1067` and `#1076` anchored the composed worker floor around `140.0–145.5 ms` L3-cold (`134.7 ms` warm), attention turned back to the serial scanner. [`#1034`](<https://github.com/lmmx/mibz2/commit/5ed20c1d51afc56c088ae244ba0bf40f37cb93b1>) had just merged coalesced same-selector runs, byte-aligned `u8` Huffman tables, and zero-skip `syms` reuse. That left header parsing (`A2a..d`: block header, selector MTF, code-length deltas, and `Huff::build_into`) at roughly `14%` of post-`#1034` serial `S'`.

[The cold `A2` dissection](<https://github.com/lmmx/mibz2/blob/a34dda23177430691c20510cddaf1cd4dd5fa460/docs/journal/2026-11-27-explore/scanner-a2-dissection.md>) (`#1069`) audited all 431 Wikidata blocks and five off-Wikidata guard corpora using `#![forbid(unsafe_code)]`. Three structural wastes stood out, and three safe-Rust mechanisms eliminated them together:

1. **`A2b_direct_runs` (`lzcnt` selector runs).** On Wikidata, `62.4%` of `501,448` selectors are `0`-bit repeats of the active MTF front. Shipped `A2b` still decoded them one byte at a time into `sel_buf`, and `#1034` then re-scanned all `501,448` bytes in `A3`. Packing the 6-table MTF list into one `u64` and counting leading `0`-bits with `leading_zeros()` emits `(group, run_len)` directly. That cuts standalone `A2b` by **`−33.1%` to `−38.5%` disjoint** AND removes the `A3` re-scan.
2. **`A2c_lut_multi` (4 KiB 8-bit multi-symbol delta LUT).** On Wikidata, `94.2%` of `133,280` code-length deltas use 1-bit (`68.5%`) or 3-bit (`25.7%`) codes. A compile-time `const` 256-entry table (`4 KiB`, L1d-resident) decodes up to 8 consecutive deltas per lookup with one range check. It averages **`3.46` lengths per lookup** on Wikidata, `5.25` on `records`, and `4.77` on `random`. Lookups drop from `133,280` to `38,573`, cutting standalone `A2c` by **`−63.1%` to `−66.7%` disjoint** (`−47.2%` to `−80.0%` on guard corpora).
3. **`A2d_direct` (single-write fused `K = 2` LUT + tail-only fill).** Shipped `Huff::build_into` zeroed `lut[0..1024]` and `d_arr[0..1024]`, then wrote `K = 1` entries before overwriting the same `1,015` entries with `K = 2` pairs. Yet the mean tail is only `9.1` entries (`1.0` on `records`). Emitting paired `(s1, s2)` entries directly in one pass via `split_at_mut` removes `d_arr` and double-writes entirely. All `1,002` synthetic edge-case tables and `2,009` real tables matched bit-for-bit, cutting standalone `A2d` by **`−24.0%` to `−36.7%` disjoint**.

| Build (`wikidata.bz2`, 7 interleaved repeats) | Standalone `A2a..d` (`ms`) | `A2a..d` Δ | Full post-`#1034` `S'` (`ms`) | Full `S'` Δ |
| --- | ---: | ---: | ---: | ---: |
| **Baseline x86-64** | `8.946 → 6.057` | **−32.3% (DISJOINT)** | `79.83 → 75.27 / 74.72` | **−5.71% / −6.40% (DISJOINT)** |
| **Native (`target-cpu=native`)** | `8.917 → 5.560` | **−37.7% (DISJOINT)** | `76.44 → 70.80` | **−7.38% (DISJOINT)** |
| **Native + JCC-32B** | `7.434 → 5.082` | **−31.6% (DISJOINT)** | `72.57 → 69.35 / 68.96` | **−4.44% / −4.98% (DISJOINT)** |

*(Two `S'` figures show block-gated `g64` and per-table `t16` `K = 2` gating. Off-Wikidata guard corpora improved or stayed flat across all three builds: `text −2.5%..−4.0%`, `records −0.2%..−2.0%`, `random −0.4%..−0.6%`.)*

### Why this flips the eight-worker knee (`#1074`)

After `#1031` and `#1034`, modeled i9-10900K serial scan time sits around `145 ms`, down from `179 ms` shipped. Applying `#1069`'s `−4.44%` to `−7.38%` reduction saves another `6.4` to `10.7 ms`. That pulls modeled `S'` down to **`134–139 ms`**.

That single shift changes the eight-worker queueing balance analyzed in [`#1074`](<https://github.com/lmmx/mibz2/commit/b7e2187f3d70edada02c56cfe8c11563d37eea70>). Before `#1069`, a `145 ms` scanner sat above the composed worker stack (`W'/8 = 134.7 ms` warm, `140.0–145.5 ms` L3-cold), leaving workers idle in `job_wait`. After `#1069`, **`S' ≈ 134–139 ms` drops below L3-cold `W'/8 = 140.0–145.5 ms`**. The `k = 8` knee flips from scanner-bound to tied or worker-bound by `0–11 ms`.

Because slack-only hybrid scanning (`#1056`) beats pure serial scanning only when

\\\[\\frac{8 \\cdot (W'/8) + S'}{9} + 4.85\\text{ ms} \< S' \\iff S' - W'/8 \> 5.46\\text{ ms},\\\]

its main payoff moves one core to the right. At **`k = 9–10`**, `W'/9` drops to `124–129 ms`. That restores a `5–15 ms` scanner gap where idle workers can help clear the 6 GB/s wall.

Safe-Rust A2a..d speedups (\#1069) and the flipped eight-worker knee (\#1074)

Top: diagnostic VM reductions on `wikidata.bz2` from `#1069` (all 7-repeat spreads disjoint). Bottom: modeled i9-10900K balance from `#1074`; cutting `S'` to `134–139 ms` makes 8 workers worker-bound and moves slack-only hybrid scanning's primary gain to `k = 9–10`.

## 4\. The invisible 15% ghost: 32-byte branch alignment (`#1077`)

Four consecutive probes had surfaced `6%` to `20%` timing swings from tiny, unrelated code changes. [The whole-decoder JCC alignment audit](<https://github.com/lmmx/mibz2/blob/c1239ce0dfb9b7cbea70a55617900de11ca901d9/docs/journal/2026-11-27-explore/jcc-alignment-whole-decoder.md>) (`#1077`) isolated that noise cleanly. It measured `-C llvm-args=-x86-branches-within-32B-boundaries` across the shipped library, every pipeline station, and seven corpora.

### Static `objdump` census and VM speedup

First, the shorthand flag proved byte-for-byte identical to `-x86-align-branch-boundary=32 -x86-align-branch=fused+jcc+jmp` (`663,261 B` `.text`). It removed **100% of unfused and hardware-fusable 32-byte branch crossings**. Unfused violations dropped from `1,712` to `0` on baseline and `1,746` to `0` on native (`open` `418 → 7` non-fusable, `runs` `49 → 0`, `huff/block/bits` `115 → 1`). The size cost was just **`+3.43%` to `+3.46%` `.text`**, or `+0.52%` to `+0.53%` total binary size.

Second, on the VM, the speedup split cleanly by production kernel:

- **Off-Wikidata corpora (`6 corpora × 2 builds`):** **`12 / 12` cells faster** with zero regressions. Gains ran `−0.61%` to `−9.11%` across `text`, `prose`, `source`, `elf`, `records`, and `random`.
- **Eager `Cells` (1T `BzReader::new`):** modest `−2.85%` to `−3.01%` (overlapping spreads).
- **Production `OpenCells` (`4W–10W` worker):** baseline ran **`−15.64%` (`−118.3 ms`) disjoint faster** (`756.5 → 638.2 ms`), and native ran **`−13.70%` (`−99.2 ms`) disjoint faster** (`723.7 → 624.5 ms`). `CE_open` (`A6 + A7`) drove `75%` to `82%` of that gain (`−86.3` / `−81.8 ms`), while `unrle_in_place` added `−20.0%` to `−27.8%` (`−9.8` to `−16.5 ms`).

### Why VM Xeon silicon is not the i9-10900K

Treating a `15%` VM win as an automatic release flag would violate our evidence standard. Inspecting `/proc/cpuinfo` flags (`umip`, `mmio_stale_data`, `bhi`, `md_clear` under Haswell `0x3F` masking) shows the cloud VM runs **Skylake/Cascade Lake-era Xeon silicon**. On those chips, Intel's JCC microcode mitigation hard-evicts the entire 32-byte uop-cache line (DSB). Any macro-fused or unfused jump crossing or ending on a 32-byte boundary triggers that eviction.

Target desktop and cross-vendor silicon behaves differently:

- **Intel i9-10900K (Comet Lake-S, stepping 5):** carries the silicon-level JCC fix (model `0xA5`). Only \~`1%–4%` residual DSB fetch-window splits remain possible.
- **Skylake/Cascade Lake cloud fleets:** remain widely deployed on x86-64 servers and benefit substantially (`−13%` to `−16%` on `OpenCells`).
- **AMD Zen CPUs:** unaffected by Intel's erratum and pay \~`0.5%–2%` decode overhead from `+3.4%` NOP padding.
- **ARM64 / Apple Silicon:** LLVM flag is strictly x86-64-only.

The engineering rule is therefore two-pronged. First, JCC-32B alignment is now a **mandatory benchmark-hygiene control** on VM probes so `6%–20%` code-layout ghosts never masquerade as algorithmic wins or regressions. Second, no release or wheel build flag will ship until a single paired `ab` run on the i9-10900K measures the real target-silicon tradeoff.

Intel JCC-32B alignment (\#1077): VM per-station speedup and silicon-portability matrix

Source: `docs/journal/2026-11-27-explore/jcc-alignment-whole-decoder.md` (`#1077`). Bars plot the larger of baseline or native VM reductions (baseline for all stations except `S`). JCC-32B alignment is adopted immediately as a VM benchmark control; any release flag waits on paired i9-10900K testing.

## 5\. Disciplined stops and the workstation validation queue

Knowing where *not* to write production code saves as much engineering time as finding a breakthrough. Two off-critical-path directions ran full probes and closed cleanly under their frozen stop tests:

- **Column-route MTF front (`#1060`, STOP).** [Four rounds across 20 exact AVX2 arms](<https://github.com/lmmx/mibz2/blob/da48997293faf726d1d9a803d49674291edc7f00/docs/journal/2026-11-27-explore/mtf-column-front-dissection.md>) diagnosed the off-Wikidata column MTF path. Skeleton ablations proved that `49.5%` of `prose` and `37.4%` of `elf` are irreducible RLE2 branch mispredictions. Branchless RLE2 regressed `+5.7%` to `+57.1%`. An 8-YMM in-register MTF list regressed `+5.7%` to `+46.7%` from 12-cycle `vpblendvb` mux chains and Port-5 saturation. The best tail cleanups (`R_t32_flat_unal7` at `−8.45%`/`−9.36%` and `S_adapt_Q_P2` at `−6.32%`/`−10.36%`) missed the frozen `>= 15%` gate (`elf −2.8%..−3.9%`, `prose` flat). Clearing that wall would require AVX-512 `vpermb` or fusing `A3` into MTF.
- **Cooperative multi-thread block decoder (`#1063`, STOP).** [The cooperative dissection](<https://github.com/lmmx/mibz2/blob/371cede0a3af320df077c5d79c7e8500e4b45943/docs/journal/2026-11-27-explore/cooperative-decoder-dissection.md>) showed `Plan::for_threads` never triggers `Cooperative` on `<= 10` cores (unreachable on our i9-10900K). On a single Wikidata block, `OpenCells` (`3.07 ms`) beats `Cooperative<2>` (`8.22 ms`, `0.37×`). Even after cutting parallel `build + walk` by `13%` to `33%`, serial stages still hold `61%` to `81%` of two-thread wall time. Amdahl's Law therefore caps full-block wall gains at `−5.2%` to `−6.2%`, well below the `10%` gate.

### Where the whole programme stands

Every piece of the next workstation cycle is now cleanly scoped, verified on the VM, and separated from dead ends:

| Study | Scope | Verdict | Key VM / model finding | Next workstation check (i9-10900K) |
| --- | --- | --- | --- | --- |
| `#1062`, `#1067` | Composed Open worker `W4_v2` | **PASS** | `−21.6%..−24.6%` L3-cold (`32 KiB` slabs, `#[cold]` paths) | Measure real `W'/8` at 1W, 8W, 9W, 10W |
| `#1068` | Direct-literal 4-lane split | **STOP** | `24/24` `A7` cells `+4.1%..+39.0%` slower | None (closed) |
| `#1076` | `K = 6, 8, 10` &amp; decoupled `B = 8` | **STOP** | `+10.8%..+45.4%` in `A6c+7+9` (15-GPR wall) | None (`K = 4, S = 16, 32 KiB` locked) |
| `#1069`, `#1074` | Safe-Rust `A2a..d` header parse | **PASS** | `−31.6%..−37.7%` `A2a..d`, `−4.44%..−7.38%` `S'` | Verify `S' ≈ 134–139 ms` &amp; `k = 8 / 9` knee |
| `#1077` | JCC-32B branch alignment | **HYGIENE** | `−13.7%..−15.6%` `OpenCells` on VM Xeon | 1 paired `ab` check on Comet Lake stepping 5 |
| `#1060`, `#1063` | Column MTF &amp; Cooperative | **STOP** | RLE2 branch floor; Amdahl-capped `< 6.2%` | None (closed) |

The picture is sharper than at any point in the programme. We now know the exact hardware walls inside the Open worker: `256 KiB` private L2 and `15` x86-64 GPRs. We have a safe-Rust scanner upgrade that pushes `S'` below `140 ms` and flips the eight-worker knee. We have also removed a `15%` VM code-layout ghost from our measurements. The next step is no longer guesswork on paper: it is building the winning safe kernels and putting them directly on the 10-core i9-10900K.
