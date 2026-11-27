[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1117/blog/index.md>)

# The architecture of a 6 GB/s bzip2 decoder

Five merged studies show how four-lane forward emission, fused slices and batched MTF pair with slack-only scanning to reach 6 GB/s.

27 November 2026 · 10 min read · [Research synthesis](<#6-how-the-five-pieces-fit-and-what-builds-next>)

**Five freshly merged studies turn mibz2's ideal-parts matrix into one coherent 6 GB/s architecture.** On our 10-core Intel i9-10900K, shipped mibz2 decodes 839 MB of Wikidata (`wikidata ×4`) in 192.9 to 194.6 ms at eight workers. That is 4.31 GB/s. Reaching 6 GB/s means finishing the same workload in 140 ms. No single stage can carry that drop alone. Five designs fit end to end across the worker pipeline and the eight-worker schedule. Together they pair sampled-anchor four-lane forward emission and vectored RLE1+CRC fusion with a zero-copy boundary, L1-batched MTF folding and slack-only hybrid scanning.

This post builds on [Reopening Open's middle and the off-Wikidata decoder](<https://lmmx.github.io/mibz2/pr-preview/pr-1117/blog/2026-11-27-reopening-a6b-and-segmented/index.md>) and [The decoder's true floor and the limits of a pooled scan](<https://lmmx.github.io/mibz2/pr-preview/pr-1117/blog/2026-11-26-true-floor-and-pooled-scan/index.md>). One evidence boundary belongs up front. The i9-10900K baselines (`1,306 ms` single-worker and `192.9–194.6 ms` eight-worker) are direct workstation measurements. The four stage studies (`#1057`, `#1045`, `#1058`, `#1055` and `#1048`) are steal-free probes on a 2-vCPU Haswell Xeon VM. Every arm passed byte-exact parity on every tested block. The eight-worker trajectory (`#1056`) replays `src/parallel.rs` deterministically against primary i9-10900K traces. Every projection below awaits production builds and direct 10-core i9-10900K confirmation.

[Section 1](<#1-sampled-anchor-four-lane-forward-emission-1057>) shows how 16 scalar anchor tags unlock 4-lane forward emission without breaking SIMD wiring. [Section 2](<#2-upstream-vectored-rle1crc-fusion-1045>) explains why zero-scratch tiled fusion beats both 1-lane emission and contiguous stitching. [Section 3](<#3-the-zero-cost-vectored-delivery-boundary-1058>) prices a 160-byte vectored `ReadyBlock` across six real consumer paths and memory budgets. [Section 4](<#4-rescuing-mtf-folding-with-l1-batching-and-guard-zones-1048-1055>) diagnoses why direct MTF folding spilled registers and how L1 batching rescues it. [Section 5](<#5-closing-the-loop-to-6-gbs-with-slack-only-hybrid-scanning-1056>) maps how slack-only helper scanning clears the 140 ms wall without regressing below the knee. [Section 6](<#6-how-the-five-pieces-fit-and-what-builds-next>) summarizes the combined architecture and its workstation gates.

**Takeaways**

- **Four-lane forward emission is paid in full (`#1057`, PASS).** Tagging 16 uniform BWT-row anchors in bit 31 keeps unmodified SIMD wiring and cuts `A6c + A7` thread CPU by 31.0 to 35.8% with zero `unsafe`. Native L3-cold runs reach 97.4% of a prebuilt oracle and project +208 to +235 ms saved on one i9 worker.
- **Fusion only pays off if delivery stays vectored (`#1045`, PASS / scoped STOP).** Zero-scratch in-place tile compaction with 64 KiB PCLMULQDQ CRC32 cuts `E + A9` writes by 53.4%. It projects +31.8 to +49.9 ms saved on one i9 worker, whereas contiguous stitching moves 74.9% of the output and erases the win.
- **Vectored delivery costs nothing at the boundary (`#1058`).** An inline 160-byte `ReadyBlock` keeps all six consumer paths at 0.86× to 1.01× of contiguous delivery while holding RSS at 1.017× with zero Wikidata overflows.
- **L1 batching and guard zones rescue MTF folding (`#1048`, `#1055`, PASS).** Direct per-run folding spilled `runs::front_avx2`'s 15 live registers and regressed up to +4.3%. A 2 KiB L1d batch plus a 496/512 `OutQ16` guard zone cuts `push_slow` 6.22×. Together they drop `A5 + A6a` by 12.1% baseline and 19.1% native.
- **Slack-only hybrid scanning crosses 6 GB/s (`#1056`).** Offloading candidate finding to the 91.6%-idle reader thread and scanning only from worker `job_wait` slack pushes eight-worker wall time to 131.8 ms (6.36 GB/s) at `W'/8 = 125 ms`. Below the knee, `k = 6` stays at 0.997× to 0.999×.

End-to-end 6 GB/s decoder architecture assembled from five merged studies

Each stage removes a separate currency: random-scatter stalls and register spills in `A5+A6a`, bounds checks in `A6b`, XOR wiring and 1-lane dependency chains in `A6c+A7`, full-block re-reads in `A9`, and serial-scan idle slack across 8 workers.

## 1\. Sampled-anchor four-lane forward emission (`#1057`)

The single largest worker prize in mibz2 sits inside `A6c + A7` (`651.7 ms` on one i9 worker, or half of total single-worker time). Earlier work showed why shipped bidirectional emission is expensive. To walk two lanes per edge without knowing endpoints in advance, `wire_xor` stores XOR-linked adjacency in `A6c`. That scalar XOR wiring breaks SIMD throughput. Forward 1-lane emission removes XOR wiring yet pays a long pointer-chasing dependency chain across `A7`.

The [sampled-anchor 4-lane probe](<https://github.com/lmmx/mibz2/blob/7f5a1c6afac636c24a7db14987cca1681e9d9e8b/docs/journal/2026-11-27-explore/forward-4lane-anchor-probe.md>) (`#1057`) resolves that tension cleanly with `#![forbid(unsafe_code)]`. Every arm matched byte for byte across all 11,961 checks on Wikidata and synthetic edge cases.

### Keeping SIMD wiring untouched

First, `A6c` keeps the unmodified SIMD `outq::Reader::wire` loop. On the VM, SIMD `Reader::wire` takes roughly **30 ms warm** and **40 ms L3-cold** across the 233 Open-admitted Wikidata blocks. Shipped `wire_xor` takes 91 ms warm and 108 ms cold. That is **2.7× to 3.0× slower**.

Second, the probe checks bit 31 of `edges[u]`. Every Wikidata Open block packs a destination index (`max_bits = 18`) and a dictionary index (`max_dict = 1,712`). Because \\(18 + 11 = 29 \\le 31\\), bit 31 is free on all 233 blocks.

Third, after `Reader::wire` finishes, the probe marks \\(S\\) uniform BWT-row anchor indices:

\\\[a\_k = \\left(\\text{mark} + \\left\\lfloor \\frac{k \\cdot m}{S} \\right\\rfloor\\right) \\bmod m, \\qquad k = 0, \\dots, S - 1\\\]

Tagging \\(S = 16\\) anchor entries takes just 16 scalar writes per block. That adds roughly **30 nanoseconds per block**, or 7 microseconds across all 233 blocks.

### Why \\(S = 16\\) beats \\(S = 4\\) by order statistics

Choosing \\(S\\) uniform BWT-row indices partitions the single cycle of length \\(m\\) into \\(S\\) random segments whose lengths behave like independent exponential spacings. With only \\(S = 4\\) segments, four parallel lanes must wait for the longest of four exponential pieces. Its expected length follows the fourth harmonic number \\(H\_4 = 1 + \\tfrac{1}{2} + \\tfrac{1}{3} + \\tfrac{1}{4} = 2.083\\times\\) the mean segment length. Measured across Wikidata at \\(S = 4\\), the longest segment is indeed **\\(2.081\\times\\)** the mean, leaving three lanes idle for over half the tail.

Oversampling to \\(S = 16\\) fixes load balance dynamically. Four lanes pull from the queue of 16 segments as each segment ends. Whenever any lane hits an anchor edge, a single sign-bit check catches all four lanes at once:

```rust
if ((ed0 | ed1 | ed2 | ed3) as i32) < 0 { /* retire hit lane, pull next anchor */ }
```

That branch fires only \~16 times per block (**0.014%** of iterations). At \\(S = 16\\), the longest single segment shrinks to **\\(0.818 \\times (m / 4)\\)** and effective lane parallelism rises to **`eff_K = 3.153`** across the whole block (including the `3 → 2 → 1` drain). Each segment writes into a **32 KiB slab-continuation arena** (`max_arena = 992 KiB`, averaging `41.6` ordered slices per block regardless of \\(S\\)).

| `A6c + A7` arm (233 Wikidata Open blocks) | Base warm (ms) | Base L3-cold (ms) | Native warm (ms) | Native L3-cold (ms) | Modeled i9 1W Δ (warm / cold) |
| --- | ---: | ---: | ---: | ---: | ---: |
| `shipped_bidi` (baseline) | 335.89 | 486.28 | 328.07 | 472.47 | 0.0 / 0.0 ms |
| `fwd_4lane_oracle` (prebuilt ceiling) | 222.84 | 323.46 | 212.50 | 295.66 | — |
| `fwd_4lane_sampled_S16_vec` (**PASS**) | **231.80 (−31.0%)** | **323.61 (−33.5%)** | **221.83 (−32.4%)** | **303.41 (−35.8%)** | **\+208.4 / +221.3..+235.4 ms** |
| `fwd_4lane_sampled_S16_stitch` | 249.60 (−25.7%) | 351.43 (−27.7%) | 239.44 (−27.0%) | 323.49 (−31.5%) | \+174.9 / +208.3 ms |

*(Diagnostic VM medians of steal-free thread CPU across 7 interleaved repeats; 100% disjoint spreads against `shipped_bidi`. Modeled i9 columns scale VM ratios to the 651.7 ms `A6c + A7` i9-10900K baseline.)*

The cold-cache result is striking. Under L3-cold pressure, `fwd_4lane_sampled_S16_vec` reaches **1.000×** of the prebuilt oracle on baseline (`323.61` vs `323.46 ms`). On native it reaches **1.026×**, or 97.4% of the oracle (`303.41` vs `295.66 ms`). That projects **\+208 to +235 ms** saved on one i9 worker. That is 16.0% to 18.0% of single-worker time, or **\+29 to +33 ms/worker** at eight workers.

## 2\. Upstream vectored RLE1+CRC fusion (`#1045`)

Once emission runs forward in four lanes, `A9` (RLE1 expansion plus CRC32) sits right behind it. On one i9 worker, `A9` takes 66.6 to 74.4 ms. Earlier standalone post-emit work (`#988`) stopped because a separate pass over the output buffer hit a 72.3% L1d-hot compute floor. Moving RLE1+CRC upstream into the 4-lane emitter seemed promising. Yet [the fusion probe](<https://github.com/lmmx/mibz2/blob/e61ec1276a59ed3d164fe165de869d147555a95f/docs/journal/2026-11-27-explore/forward-rle1-crc-fusion.md>) (`#1045`) showed that **three of four candidate designs fail**. Only one clears the gate. Every arm passed all 13,360 exactness checks across Wikidata and edge-case corpora.

### Why three designs stopped

1. **1-lane fused (`fused_1lane` — STOP).** Fusing RLE1+CRC into a single forward lane surrenders 4-lane memory-level parallelism. It regresses by **−92 to −186 ms** projected on one i9 worker.
2. **Inline per-step FSM (`fused_4l_fsm` — STOP).** Running the RLE1 state machine byte-by-byte inside the 4-lane pointer-chase loop bloats `WordMeta` and spills live lane state. It regresses by **−126 to −205 ms** projected on one i9 worker.
3. **4-lane contiguous stitched (`fused_4l_tile4096_stitch` — STOP).** Even when each lane fuses RLE1+CRC cleanly in 4 KiB tiles, compacting the four output segments back into one flat `Vec<u8>` destroys the win. On Wikidata, **86.4%** of lanes change length after RLE1 expansion. Re-packing lanes 1 to 3 moves 74.53 of 99.56 MiB (**74.9%** of all output). Across `wikidata ×4`, that copies **598.9 MiB** of extra bytes. Write savings drop from −53.4% to −18.6% (`1,718.2 → 1,398.8 MiB`), leaving timing flat at −23 to +11 ms projected.

### Why `fused_4l_tile4096_vec` passes

The winning arm (`fused_4l_tile4096_vec`) keeps the 4-lane pointer-chase loop clean and delivers vectored slices without stitching:

- **Zero-scratch single-pass in-place tile compaction.** Each lane emits raw BWT bytes straight into its own output slice until it fills a 4 KiB tile. Then it expands RLE1 in place. Because **89.4%** of Wikidata 4-byte runs have `extra == 0` (four equal bytes followed by a zero repeat count), expansion shrinks the stream so `dst <= src`. Only when a rare long repeat causes net growth does the tile spill its short suffix to a tiny scratch buffer. Across all 431 Wikidata blocks, **zero** RLE1 runs straddled a lane boundary.
- **L2-hot 64 KiB PCLMULQDQ CRC32 + GF(2) combine.** Every 64 KiB of finalized lane output feeds hardware-accelerated CRC32 while still hot in L2 cache. At block end, four `crc32_combine` calls over \\(\\text{GF}(2)\\) fold the four lane CRCs into the exact bzip2 block CRC in microseconds.

| `E + A9` arm (native VM, 233 Wikidata blocks) | Warm (ms) | L3-cold (ms) | Logical writes (`wikidata ×4`) | Modeled i9 1W Δ (warm / cold) |
| --- | ---: | ---: | ---: | ---: |
| `base_4lane_sep` (unfused baseline) | 112.35 | 150.98 | 1,718.2 MiB | 0.0 / 0.0 ms |
| `floor_4lane_plus_crc` (zero-RLE1 floor) | 101.20 | 139.89 | — | — |
| `fused_4l_tile4096_vec` (**PASS**) | **99.54 (0.886×)** | **139.89 (0.927×)** | **799.9 MiB (−53.4%)** | **\+49.9 / +31.8 ms** |
| `fused_4l_tile4096_stitch` (STOP) | 116.40 (1.036×) | 148.55 (0.984×) | 1,398.8 MiB (−18.6%) | −23.0 / +11.0 ms |

*(Diagnostic VM medians of steal-free thread CPU across 7 interleaved repeats. On warm runs `fused_4l_tile4096_vec` beats even the zero-RLE1 separate-CRC floor because 64 KiB tiled CRC stays L2-hot. On cold runs it matches the floor to 0.00 ms.)*

## 3\. The zero-cost vectored delivery boundary (`#1058`)

Both `#1057` and `#1045` point to the same architectural need: stop forcing workers to stitch emitted slices into one flat `Vec<u8>`. Before touching `src/parallel.rs`, we priced the vectored delivery boundary in [a dedicated probe](<https://github.com/lmmx/mibz2/blob/28c4c8c43e633a04dd3b7aa14b639a2d5fa8201f/docs/journal/2026-11-27-explore/vectored-delivery-boundary.md>) (`#1058`). It uses `#![forbid(unsafe_code)]` and tests 200 MiB of real Wikidata blocks plus synthetic guards (`runs`, `onebyte`, `max16`).

### A 160-byte inline `ReadyBlock`

Instead of heap-allocating a `Vec<Vec<u8>>` per block, a worker hands the reader an inline 160-byte struct:

```rust
pub struct ReadyBlock {
    buf: Vec<u8>,
    ranges: [(u32, u32); 16],
    count: u8,
    total_len: u32,
}
```

It allocates **zero** extra heap buffers. When the consumer finishes draining the last range, `into_buf()` recycles `buf` directly into `free_out` with zero changes to pool accounting, while `Ready::size` tracks logical `total_len`.

### Zero consumer tax and bounded RSS

We timed six real consumer paths across contiguous, stitched, 4-lane vectored and 16-segment vectored delivery. Those paths cover zero-copy `fill_buf_hash`, `read_64k` and `read_256k` (both single-slice and scatter-fill) and `decode_parallel`:

| Consumer path (native L3-cold, 200 MiB Wikidata) | `contiguous_1slice` | `vectored_4lane` | `vectored_16seg` | `stitched_4lane` |
| --- | ---: | ---: | ---: | ---: |
| `fill_buf_hash` (zero-copy `BufRead`) | 1.00× | **0.92×–1.00×** | **0.94×–1.00×** | \+33%..+80% |
| `read_64k` / `read_256k` (scatter-fill) | 1.00× | **0.86×–1.01×** | **0.88×–1.01×** | \+33%..+80% |
| `decode_parallel` (parallel sink) | 1.00× | **0.95×–1.00×** | **0.96×–1.01×** | \+33%..+80% |

*(Stitched 4-lane adds +16 to +22 ms per 200 MiB, or roughly +64 to +88 ms across `wikidata ×4`. Vectored 4-lane and 16-segment stay at parity with contiguous delivery.)*

Python's single-copy `read(size)` path (`#1033`) also stays clean. At `K = 4`, single-`fill_buf` call counts rise **\+0.0%** at `read(256K)` (`1,130` calls) and **\+13.5%** at `read(64K)`. At `S = 16`, scatter-fill inside `Read::read` keeps calls completely flat at `3,200` (`64K`) and `800` (`256K`).

Slab sizing turned up an equally important guard against naive headroom. A flat `+25%` (`span / 4`) per lane inflates Wikidata RSS to **1.178×** (`1,100 KiB` per block), breaching our `<= 1.05×` budget. Worse, it still overflows pathological `runs` (`2.07×`) and `onebyte` (`51.8×`). Keeping shipped `+6.25% + 64 B` gives zero Wikidata overflows at **1.0002×** (`K = 4`) and **1.0010×** (`S = 16`). Adding `4,096 B` per lane for in-place tile headroom (`+16 KiB` per 900 KB block) holds peak RSS at **1.017×** with zero overflows, cleanly inside the `<= 1.05×` gate.

## 4\. Rescuing MTF folding with L1 batching and guard zones (`#1048`, `#1055`)

Earlier in the programme, `mtf-fold` (`#1010`) tried to remove the 1 MiB `runs` buffer between `A5` (`runs::front_avx2`) and `A6a` (`Open::from_runs`). It streamed each run straight into an inline `FoldSink`. In theory, folding should remove memory traffic. In practice, [our cold dissection](<https://github.com/lmmx/mibz2/blob/5c57286dffa7c44a2efb88ec1e80c73499f60766/docs/journal/2026-11-27-explore/mtf-fold-and-outq-dissection.md>) (`#1048`) caught that direct folding **regressed** `A5 + A6a`. Baseline ran **\+1.61%** slower across all seven repeats, and native ran **\+4.33%** slower.

### Why direct folding regressed

Disassembling `runs::front_avx2` exposed three compounding costs hidden inside the hot loop:

1. **Register exhaustion (15 live x86-64 GPRs).** Standalone `runs::front_avx2` is a tight 22-instruction loop that already occupies all 15 general-purpose registers. On baseline, inlining `FoldSink + Pass::take + OutQ16` spilled the `sink` pointer (reloaded 10 times per literal) plus seven GPRs. On native, it spilled the hot MTF-front registers `f0` and `f1` on every literal.
2. **Zero-hit dead check.** Direct folding ran a `c == self.c` merge check on every emitted run that triggered **0 times in 15,141,602 runs**.
3. **Tail-overrun `push_slow` storm and `vzeroupper`.** `OutQ16` called `push_slow` **417,041 times** across 233 Wikidata blocks. **84.3%** of those calls (`351,722`) were pure end-of-chunk `f + 16 > 512` tail-overrun guards, not genuine thick-strip overflows. On baseline x86-64, every cold call across the AVX2 boundary also paid `vzeroupper`.

### The two-part rescue (`#1055`)

[The L1-batch rescue probe](<https://github.com/lmmx/mibz2/blob/3a2b770282ce2e56c52f7e74380b599eaac8face/docs/journal/2026-11-27-explore/mtf-l1-batch-rescue-probe.md>) (`#1055`) fixed both halves together:

- **L1d-resident `[u32; 512]` packed run batch (2 KiB).** `front_avx2` stays completely decoupled, emitting packed maximal runs `(c << 24) | total` into a 2 KiB L1d stack buffer with `second_cold` kept out of line. That produces a 24-instruction hot loop with **zero `f0`/`f1`/GPR spills and zero `vzeroupper`**. When the 512-run batch fills, `flush_batch_direct` consumes it from L1d with `m`, `in_out` and `f.runs` hoisted in registers.
- **Power-of-2 `OutQ16` guard zone (`496 / 512`).** Reserving 16 guard slots at the end of each 1,024-byte chunk (`CHUNK = 496`, `STRIDE = 512`) lets the 16-wide SIMD write land unconditionally until `f >= 496`. That cuts `push_slow` calls **6.22×** (`417,041 → 67,001`), driving `tail_guard` and `ovf_chunks` to **zero** with zero `A6c` readback regression.

| `A5 + A6a` arm (233 Wikidata Open blocks) | Base CPU (ms) | Base ratio | Native CPU (ms) | Native ratio | Modeled i9 1W Δ (base / nat) |
| --- | ---: | ---: | ---: | ---: | ---: |
| `shipped_two_pass` (1 MiB `runs` vec) | 188.79 | 1.0000× | 187.58 | 1.0000× | 0.0 / 0.0 ms |
| `direct_fold_1010` (`#1010` regression) | 191.83 | 1.0161× | 195.70 | 1.0433× | −5.4 / −14.4 ms |
| `l1_batch_512_only` (no guard zone) | 176.95 | 0.9373× | 162.80 | 0.8679× | \+20.9 / +44.0 ms |
| `l1_batch_512_guard496` (**PASS**) | **166.88** | **0.8793×** | **151.41** | **0.8087×** | **\+40.2 / +63.6 ms** |

*(Diagnostic VM medians of steal-free thread CPU across 7 interleaved repeats; 100% bit-exact output and disjoint spreads on both builds. Both halves are required. L1d batching alone reaches only +1.60% 1W on baseline, below the 2.5% gate. Adding `guard496` clears +3.08% baseline and +4.87% native, or +5.7 to +9.1 ms/worker at 8W.)*

## 5\. Closing the loop to 6 GB/s with slack-only hybrid scanning (`#1056`)

Stacking the four worker wins (`#1046`, `#1057`, `#1045` and `#1055`) pulls per-worker compute \\(W'/8\\) below the post-`#1031`/`#1034` serial scan floor (\\(S' \\approx 145\\text{–}150\\text{ ms}\\) on the i9-10900K). At that point, the serial scanner caps throughput around 150.9 ms (5.56 GB/s). Workers sit idle in `job_wait` for **10.9 to 20.4 ms/worker**.

Our earlier pooled-scan prototype (`#942`) tried to break that ceiling by having workers scan unconditionally. It failed because helper scanning stole cycles below the knee (`1.105×–1.149×` slower at `k = 6..7`). It also paid 5.27 ms of `pool_copy` overhead plus 4.82 ms of candidate finding.

[The slack-only hybrid scan study](<https://github.com/lmmx/mibz2/blob/72286cff15a6048c801def3549df86875806c5fb/docs/journal/2026-11-27-explore/slack-hybrid-scan-feasibility.md>) (`#1056`) replayed `src/parallel.rs` deterministically against primary i9-10900K traces (`character-stage1` `r0..r3` and `pooled-scan` `c71c4a56`). It found that **63.1% to 70.2%** of post-worker `job_wait` idle time arrives in continuous bursts at least one full-block scan wide.

### The realized 8-worker wall-time law

Workers drain `job_queue` at priority 1. Only when `job_queue` is empty do they pull speculative scan candidates at priority 2. That lets the serial scanner and eight workers share scanning at 9-way capacity:

\\\[\\text{wall}\_{8\\text{W}} \\approx \\frac{8 \\cdot (W'/8) + S'}{9} + 4.85\\text{ ms}\\\]

The `4.85 ms` coupling residual breaks into `2.62 ms` startup, `1.15 ms` validated handoff residual and `0.83–1.18 ms` scheduling and drain tail. Using a smaller initial chunk 0 drops `in_wait` to `0.05 ms` and trims the residual to **\+3.50 ms**.

Breaking **6 GB/s** (\\(\\le 140.0\\text{ ms}\\)) at \\(S' = 145\\text{ ms}\\) requires \\(W'/8 \\le 133.9\\text{ ms}\\), or \\(\\le 135.4\\text{ ms}\\) with small chunk 0. With L1-batched MTF (`W'/8 = 125 ms`), eight-worker wall time reaches **131.82 ms (6.36 GB/s)**. That clears the 6 GB/s target with **\+8.18 ms** to spare.

Coupled eight-worker wall time on wikidata ×4 (839 MB): trace-replay trajectory across the 6 GB/s line

Source: `docs/journal/2026-11-27-explore/slack-hybrid-scan-feasibility.md` (\#1056). Shipped baseline is directly measured on the 10-core i9-10900K; post-worker and hybrid bars are discrete-event replays of `src/parallel.rs` driven by primary i9 traces at `S' = 145 ms` and `W'/8 = 135 ms` (no fold) or `125 ms` (with L1-batched MTF).

### Zero-plumbing design and below-the-knee safety

Four concrete design choices eliminate every overhead that killed `#942`:

1. **Offload AVX2 `Finder` to the 91.6%-idle reader thread (`0.00 ms` tax).** Primary `per_k.tsv` traces show `mibz2-read` is active for just **12.34 ms** and sleeps in `feed_wait` for **134.10 ms** at `k = 8`. Wrapping each read chunk in `Arc<Chunk { base, bytes, cands: OnceLock<Vec<Cand>> }>` lets `mibz2-read` push chunk 0 immediately and populate K4's 4.82 ms AVX2 `Finder` (`9.84 GB/s`) while waiting on backpressure.
2. **Eliminate `pool_copy` by skipping seam blocks (`5.27 → 0.00 ms`).** Fewer than 0.7% of blocks (11 of 1,724) straddle a 4 MiB chunk boundary. Helpers simply skip seam-straddling candidates and leave them to the serial scanner's existing seam window (`#1031`).
3. **Priority-1 decode draining plus `full_only` filtering.** Helpers touch scan candidates only when `job_queue` is empty. At `k = 6`, that runs just **3 to 4 helper blocks** (`0.5–0.7 ms`). Wall time across `k = 6..7` stays at **0.997× to 0.999×**, versus `#942`'s `1.105×–1.149×`. Filtering `gap >= 512 B` (`full_only`) cuts helper handoffs by **−35% to −41%**.
4. **Policy A jump-ahead scanner and worker-local scratch.** Instead of stalling on speculative validation (`#770`'s `+1.2–2.3 ms` `serial_wait`), the scanner verifies completed helper bit-chains asynchronously and jumps straight past validated blocks. Each worker keeps a thread-local `ScanScratch` and `Vec<u16>` (`~200 KiB`, L2-hot), avoiding cross-core `free_syms` coherence ping-pong (`#1034`).

## 6\. How the five pieces fit and what builds next

Viewed together, the five studies expose a clean architectural stack where each win amplifies the next:

| Stage / boundary | Primary study | Mechanism | VM / replay finding | i9-10900K gate |
| --- | --- | --- | --- | --- |
| `A5 + A6a` | `#1048`, `#1055` | 2 KiB L1d run batch + `OutQ16` 496/512 guard | −12.1% base / −19.1% native (`A5 + A6a`) | ≥ 2.5% 1W, 0 `A6c` regression |
| `A6b` | `#1046` | Fused setup + safe 64K SoA + gen-tagged dict | −27.4%..−28.8% `A6b`, −5.4%..−7.6% `C+E` | RSS &amp; 8–10W L3 scaling |
| `A6c + A7` | `#1057` | SIMD `Reader::wire` + `S=16` bit-31 + 4 lanes | −31.0%..−35.8% `A6c+A7` (97.4% of oracle) | \+208..+235 ms 1W, 0 `unsafe` |
| `E + A9` | `#1045` | Zero-scratch 4K tiles + 64K CRC + GF(2) | −53.4% writes, +31.8..+49.9 ms 1W proj. | Vectored only (no stitch) |
| Delivery | `#1058` | 160 B inline `ReadyBlock`, `+16 KiB` tile slab | 0.86×–1.01× consumers, RSS 1.017× | ≤ 1.05× RSS, 0 overflows |
| 8W scan | `#1056` | `mibz2-read` Finder + slack-only Policy A | 131.8 ms (6.36 GB/s) @ 8W, 0.998× @ 6W | ≥ 6 GB/s @ 8W, ≤ 1.005× @ 6W |

The dependency order for production builds is equally crisp. First, ship the independent worker kernels (`a6b-endpoint-build`, opt-in `forward-emission` and `mtf-l1-batch-build`) alongside the vectored `ReadyBlock` boundary (`#1058`). That lets upstream RLE1+CRC fusion (`#1045`) land without paying stitching's 74.9% copy tax. Second, verify on the 10-core i9-10900K that the worker stack pulls \\(W'/8\\) below the post-`#1031`/`#1034` serial scan floor. Then enable `pooled-scan-hybrid` (`#1056`) to turn newly exposed `job_wait` slack into 6 GB/s throughput with zero below-the-knee regressions.
