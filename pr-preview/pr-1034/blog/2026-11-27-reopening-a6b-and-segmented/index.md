[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1034/blog/index.md>)

# Reopening Open's middle and the off-Wikidata decoder

A safe fixed-size redesign cuts Open's middle stage by 28% and a fallback stack speeds off-Wikidata blocks by 16%, while contraction hits a floor.

27 November 2026 · 8 min read · [Research result](<#4-what-ships-next>)

**Four merged results show how research persistence works in both directions.** Off Wikidata, we shipped a safe cap precheck that removes wasted work on refused dense blocks. We also dissected the Segmented fallback into a zero-memory stack that cuts prose and source decoding by 15.4 to 16.1%. Back inside Wikidata's Open kernel, redesigning all three sub-phases of its middle stage together reopens a phase that earlier isolated tweaks could not push past 3%. In parallel, a rescue of contracted return-map constructors trimmed real waste yet stopped cleanly once a bare pointer-chasing floor exceeded the budget.

This post builds on [Off Wikidata and inside the eight-worker tie](<https://lmmx.github.io/mibz2/pr-preview/pr-1034/blog/2026-11-27-off-wikidata-and-eight-worker-coupling/index.md>) and [Complementary bottlenecks](<https://lmmx.github.io/mibz2/pr-preview/pr-1034/blog/2026-11-26-complementary-bottlenecks/index.md>). One evidence boundary matters up front. The dense-band cap precheck (`#1040`) is already merged into production and measured on pinned CI hardware. The Segmented dissection (`#1042`), the `A6b` redesign (`#1046`) and the contraction rescue (`#1038`) are diagnostic VM probes with exactness verified on every tested block. Both passing probes now move to production build pull requests and 10-core Intel i9-10900K workstation confirmation.

[Section 1](<#1-off-wikidata-dense-cap-precheck-and-segmenteds-16-comeback>) covers the shipped dense-band precheck, Segmented's three-lever comeback and why a two-step `LF²` table failed. [Section 2](<#2-reopening-opens-middle-stage-with-a-safe-64k-redesign>) explains how combining all three `A6b` sub-phases cuts `A6b` by 27.4 to 28.8% with zero `unsafe`. [Section 3](<#3-honest-contrast-when-rescue-hits-a-structural-floor>) contrasts that win with `#1038`'s scoped stop on contracted return-map constructors. [Section 4](<#4-what-ships-next>) sets out the admitted follow-on builds and workstation checks.

**Takeaways**

- **Dense-band refusals off Wikidata are gone (`#1040`, shipped).** Checking the dense-band cap from run records cuts median refused-block inverse time from 1.536× to 1.027× with zero `unsafe`. On pinned CI hardware, four-thread `text` runs 3.7% faster and Wikidata stays unchanged.
- **Segmented fallback drops 15.4 to 16.1% on prose and source (`#1042`, PASS).** Sequential `LF` construction, AVX2 reverse assembly and invariant-checked run fills speed all 78 off-Wikidata `Segmented` blocks by 18.4 to 18.5% at zero extra memory. A 2-step `LF²` table blows up construction 3.9 to 4.2×.
- **Reopening `C-A6b` needed all three sub-phases together (`#1046`, PASS).** Fused setup, safe fixed-size `Box<[T; 65536]>` `u16`-link `inline15` SoA steps and generation-tagged caches cut `A6b` by 27.4 to 28.8%. Full `C+E` drops 5.4 to 7.6% with zero `unsafe` and zero `A7` regression.
- **Layout and deduplication both matter in `A6b`.** Structure-of-arrays beats 24-byte and 32-byte array-of-structures layouts by 4 to 7 ms. Dropping `contents` deduplication inflates dictionary entries by 73.3% and regresses downstream emission.
- **Persistence includes knowing when a floor binds (`#1038`, scoped STOP).** Cleaning up contracted return-map construction cut contraction by 9.0 to 13.9% and fixed `runs.bz2`. Even so, bare 4-lane pointer chasing alone takes 30.6 ms and already exceeds the \~24 to 25 ms VM budget.

## 1\. Off Wikidata: dense-cap precheck and Segmented's 16% comeback

Our [previous post](<https://lmmx.github.io/mibz2/pr-preview/pr-1034/blog/2026-11-27-off-wikidata-and-eight-worker-coupling/index.md>) showed a stark corpus split. On Wikidata, every full block enters our [Open Returns](<https://lmmx.github.io/mibz2/pr-preview/pr-1034/blog/2026-11-19-open-returns/index.md>) kernel. Off Wikidata, most full blocks fall back to the `Segmented` inverse-BWT decoder. Worse, every off-Wikidata block in Open's dense band (\\(r/n \\in \[0.15, 0.28\]\\)) paid for a failed first pass of Open before falling back.

### Eliminating dense-band refusal penalties (`#1040`)

The first fix is already shipped in [`src/open.rs`](<https://github.com/lmmx/mibz2/blob/2915d7e79f7d53f0bfc9155c3005d47d4fdc895a/src/open.rs>) and recorded in the [`dense-cap-precheck` delta](<https://github.com/lmmx/mibz2/blob/2915d7e79f7d53f0bfc9155c3005d47d4fdc895a/frontier/deltas/2026-11-27-dense-cap-precheck.md>). A safe single-pass helper called `within_cap` counts retained thin-run rows and thick strips straight from the run records. It runs only in the dense admission band (`cap.m != u32::MAX`) and adds zero `unsafe` code.

We measured 24 dense-band refused blocks across five local corpora (`prose`, `pysrc`, `rustsrc`, `json` and `elf`) over seven interleaved repeats. The median per-block ratio against direct `Segmented` fell from **1.536× to 1.027×**. The summed inverse-time ratio fell from **1.404× to 0.984×**, leaving every corpus at 1.040× or below:

| Corpus (refused dense blocks) | Blocks | Before (`prod / Segmented`) | After (`prod / Segmented`) |
| --- | ---: | ---: | ---: |
| `prose` | 7 | 1.486× | **0.950×** |
| `pysrc` | 3 | 1.164× | **0.888×** |
| `rustsrc` | 3 | 1.445× | **1.040×** |
| `json` | 3 | 1.492× | **0.873×** |
| `elf` | 8 | 1.432× | **1.036×** |
| **Median of block ratios** | **24** | **1.536×** | **1.027×** |
| **Summed inverse ratio** | **24** | **1.404×** | **0.984×** |

*(Diagnostic VM repeats using `ROUTE_SHADOW=1` on the same 24 dense-band refused blocks.)*

Wikidata pays almost nothing for this check. Guarding Wikidata's five dense blocks adds about 0.27 ms in total, or 0.03% of single-worker wall time. All 233 full Wikidata blocks still enter Open. On CI's pinned AMD EPYC runner across ten paired rounds, one-thread `wikidata` wall time shifted **−0.2%** (`−1.0%` to `+0.6%`, rising from 666 to 672 MB/s). On the same runner, four-thread `text` ran **−3.7% faster**.

### Dissecting the Segmented fallback (`#1042`)

Removing wasted refusals still leaves `Segmented` running on most off-Wikidata blocks. The [Segmented fallback dissection](<https://github.com/lmmx/mibz2/blob/9f89ba509f8f0348d1b4358b95a48b7199112940/docs/journal/2026-11-27-explore/segmented-fallback-dissection.md>) tested six `bzip2 -9` corpora totalling 82.18 MB raw. Across those files, **78 of 86 full blocks (90.7%)** ran `Segmented`, split into 48 `RunsFallback` blocks and 30 `Column` blocks. That covered all 25 blocks of `ab gen text` and 50 of 51 blocks across prose and source code.

Profiling those 78 blocks exposed where the time goes. Inverse BWT takes 52.5 to 65.8% of worker time on text, prose, source and JSON, and 42.3% on ELF binaries. Within inverse BWT, building the navigation table (`build`) alone takes **35 to 46%** of the stage.

Three zero-memory mechanisms stack into one winning arm called `C_trustfill+rev`. It matched `bwt::walk` byte for byte on all 78 blocks and touches zero Wikidata blocks:

1. **Sequential `LF` column construction (`A_col_lf`).** Replacing the 256-stream random-scatter \\(\\Psi\\) build with a sequential `LF` pass plus an 8-byte equal-run fast path (`w == w.rotate_left(8)`) removes scattered writes. On the 30 `Column` blocks, this cuts steal-free thread CPU by **−23.4% baseline** and **−24.8% native**.
2. **AVX2 32-byte reverse assembly (`B_col_lf+avx2rev`).** Walking `LF` emits each segment backwards. Reversing chunks with AVX2 `pshufb` and `vperm2i128` (with an 8-byte scalar `swap_bytes` fallback) cuts baseline assembly from **13.68 ms to 7.54 ms (−45%)**.
3. **Invariant-checked `runs::fill_avx2` (`C_trustfill+rev`).** When run records come from `runs::front`, the buffer bounds and alphabet limits are already established in \\(O(1)\\). Skipping redundant per-run tiling checks and `lim[256]` guards cuts the 48 `RunsFallback` blocks by **−13.5% baseline** and **−14.0% native**.

Off-Wikidata Segmented fallback: steal-free CPU reduction with C\_trustfill+rev (7 interleaved VM repeats)

Source: `docs/journal/2026-11-27-explore/segmented-fallback-dissection.md` (\#1042, 7 interleaved repeats on a 2-vCPU Xeon VM). Primary comprises `text`, `prose`, `pysrc` and `rustsrc` (50 Segmented blocks).

On the primary `prose/text + source` suite of 50 blocks, `C_trustfill+rev` cuts steal-free thread CPU by **−15.44% baseline** (`195.57 → 165.37 ms`) and **−16.05% native** (`189.88 → 159.40 ms`). Both builds show non-overlapping spreads. Across all 78 blocks in the six corpora, the saving reaches **−18.43% baseline** and **−18.54% native** with zero extra memory.

The same probe also ruled out a classic literature idea: Kärkkäinen–Kempa–Puglisi 2-step `LF²` stored in `Vec<u64>`. That `D_lf2_u64` arm costs an extra 3.6 MiB per worker. Walking two bytes per pointer chase did cut walk time by **−15.0 to −15.7%** (`120.85 → 101.89 ms` baseline, `115.14 → 97.87 ms` native). Yet `build_lf2` blew up **3.9 to 4.2×** (`75.1 → 289.4 ms` baseline, `73.4 → 308.4 ms` native). Total inverse time ran **\+104.5 to +121.4% slower**, and the negative result now lives in [`frontier/dead-ends.tsv`](<https://github.com/lmmx/mibz2/blob/9f89ba509f8f0348d1b4358b95a48b7199112940/frontier/dead-ends.tsv>).

## 2\. Reopening Open's middle stage with a safe 64K redesign

Back on Wikidata, the [ideal-parts matrix](<https://lmmx.github.io/mibz2/pr-preview/pr-1034/blog/2026-11-26-complementary-bottlenecks/index.md>) priced Open's middle stage — endpoint induction (`C-A6b`, in `src/open/endpoint.rs`). On our 10-core i9-10900K, `C-A6b` takes **216.0 ms** on one worker (16.54% of the 1,306 ms wall) and **30.4 ms/worker** at eight workers. Earlier isolated Round 2 tweaks fell short of our 3% whole-decoder gate. In isolation, `inline7_node20` projected +2.48% and `no_contents_dedup` projected +0.99%.

The [whole-`A6b` redesign probe](<https://github.com/lmmx/mibz2/blob/05c13b98253215fd66ae2024ff79b7e6adbfe947/docs/journal/2026-11-26-explore/a6b-combined-redesign.md>) (`#1046`) redesigned all three sub-phases of `A6b` together: `setup`, `steps` and `dict`. It priced the full `C + E` pipeline (`A6a + A6b + A6c + A7`) under both shipped bidirectional emission and matched forward emission.

The winning design, `comb_nobc_soa_gen`, uses **zero `unsafe`**. It passed byte-exact verification against `bwt::walk` on all **431 of 431** Wikidata blocks:

1. **`A6b-setup` (`26.5 → 15.4 ms`, −41.7%).** A single forward pass over `strips_raw` fuses strip-order building with strip and domain-gap creation. Scattering a packed `u64` into `image_order64` turns image-gap construction into a sequential scan with zero random gathers. Sentinel-0 list appends remove branches and direct `u16` links replace bit-packed read-modify-write updates.
2. **`A6b-steps` (`64.2 → 43.8 ms`, −31.1 to −31.8%).** Step time falls from 24.1 to 16.4 ns/step by storing the structure-of-arrays tables in reusable `Box<[T; 65536]>` arenas indexed by `usize::from(u16)`. Because every `u16` is statically below 65,536, LLVM removes **100% of bounds checks** in safe Rust and saves roughly 8 ms over dynamic `Vec`. An `inline15` `u128` handle keeps short joins in registers and cuts rule allocations by **−54.6%** (`1,647,746 → 747,845`), while equal-width steps (`aw == bw`, 36.98% of steps) update in place.
3. **`A6b-dict` (`40.7 → 34.8 ms`, −13.5 to −14.5%).** Short dictionary entries build straight from `u128` handles without `pairs` lookups. Persistent generation-tagged L1d-hot caches (`contents[4096]` at 16 KiB and `pairs[1024]` at 8 KiB) eliminate 24 KiB of per-block zero-fill. Dead `stats` counters are gated behind the `census` feature.

A6b sub-phase ablation ladder on 233 admitted Wikidata blocks (baseline x86-64, shipped bidirectional emission)

Source: `docs/journal/2026-11-26-explore/a6b-combined-redesign.md` (\#1046). Moving from dynamic `Vec` (`combined_soa_gen`) to safe `Box<[T; 65536]>` (`comb_nobc_soa_gen`) removes roughly 8 ms of bounds checks in `steps` with zero `unsafe`.

We timed all four build × emission cells across seven interleaved repeats on a pinned 2-vCPU Xeon VM. In every cell, `comb_nobc_soa_gen` cuts `A6b` by **−27.4 to −28.8%** (`0.712×–0.726×`) and full `C + E` by **−5.4 to −7.6%** (`0.925×–0.946×`). Spreads are disjoint on both wall clock and thread CPU in all four cells:

| Build × emission cell | `A6b` baseline → redesign (ms) | `A6b` ratio | `C + E` baseline → redesign (ms) | `A7` Δ | i9-anchored 1W Δ (`%1W`) | Modeled 8W Δ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Baseline × shipped bidi | 131.66 → **95.08** | 0.722× | 566.31 → **533.05** | \+0.32% | **\+48.7 ms (+3.73%)** | \+8.2 ms/w |
| Baseline × forward 1-lane | 130.41 → **94.71** | 0.726× | 508.05 → **473.78** | \+0.01% | **\+55.5 ms (+4.25%)** | \+8.3 ms/w |
| Native × shipped bidi | 131.58 → **94.89** | 0.721× | 593.72 → **561.47** | \+0.15% | **\+51.7 ms (+3.96%)** | \+8.4 ms/w |
| Native × forward 1-lane | 131.00 → **93.26** | 0.712× | 522.68 → **483.24** | −0.44% | **\+67.7 ms (+5.18%)** | \+9.1 ms/w |

*(Diagnostic VM probe timings on 233 admitted Wikidata blocks. The i9-10900K columns are projections anchored to the 1,068.0 ms `C + E` baseline, not direct i9 runs yet.)*

Two structural ablations explain why `comb_nobc_soa_gen` beat its sibling designs:

- **SoA beats AoS by 4 to 7 ms.** Keeping `links` (8 B/node, \~78 KiB live) and `widths` (4 B/node, \~39 KiB live) separate from 16-byte `u128` words keeps neighbour edits compact in L2. Both 32-byte `AoS32` and 24-byte `AoS24-inline7` pull wider cache lines on every link update.
- **Generation-tagged deduplication beats dropping dedup.** Dropping `contents` dedup (`comb_nobc_soa_nod`) shaves another 5.5 ms in `A6b-dict` (`34.8 → 29.3 ms`). Yet it inflates dictionary entries by **\+73.3%** (`246,680 → 427,381`) and regresses `A7` emission by **\+1.47 to +2.10%**, breaching our 1.0% guard. Generation-tagged caches preserve dictionary compactness (`246,683` vs `246,680` entries, `64,314` long entries and `9,888.3 KiB` of spelling) while keeping `A7` neutral (`−0.44%` to `+0.32%`).

One real caveat remains for the production build. Three `Box<[T; 65536]>` arenas reserve **1.75 MiB of virtual memory per worker**. On Wikidata, each block touches only `0..nodes` (about 273 KiB average and 420 KiB max) and drops those arenas before `A6c` and `A7`. Verifying RSS and 8- to 10-worker L3 cache scaling on the i9-10900K is an explicit gate for the follow-on build.

## 3\. Honest contrast: when rescue hits a structural floor

Reopening a stage only stays trustworthy if failed rescues are priced just as strictly. Alongside `#1046`, we ran a constructor rescue ([`bwt-contraction-constructor-rescue`](<https://github.com/lmmx/mibz2/blob/3bb552a1612e058b362471aa866930af48154989/docs/journal/2026-11-24-algorithm-level/bwt-contraction-rescue/results.md>), `#1038`) on `#1013`'s contracted return-map idea. That approach removes low-cost `F`-buckets from Open's edge graph to shorten emission `A7`. Every arm used `#![forbid(unsafe_code)]` and passed **8,406 of 8,406** byte-identical exactness checks across both builds.

Cold dissection on the training split (blocks `0..117`) found real contingent waste in `#1013`. Of its `37.6 MiB` of spelling writes, **72.4%** (`27.3 MiB`) were redundant re-copies of already-stored `LONG` prefix words. In addition, the **15.2%** of chains longer than 8 hops carried roughly **69%** of all eliminated hops.

Fixing that waste helped substantially. The cleaner 1-lane redesign (`r1_clean1_2part`) cut contraction time by **−9.0% baseline** (`108.69 → 98.86 ms`) and **−13.9% native** (`111.44 → 95.94 ms`). An \\(O(1)\\) pre-construction guard (`m >= 32,768`, `elim_frac >= 0.55`) also removed `#1013`'s `runs.bz2` regression (`1.586× → 1.015×` baseline, `1.375× → 1.018×` native).

Yet on held-out Wikidata across 116 blocks and seven interleaved repeats, neither pre-registered redesign approached forward emission:

| Held-out arm (116 Wikidata blocks) | `contr` (ms, base / nat) | `A7` (ms, base / nat) | `C + E` (ms, base / nat) | vs 1-lane fwd (base / nat) | vs 4-lane oracle (base / nat) |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ctrl_fwd_1lane` | 0.0 / 0.0 | 108.5 / 121.4 | 242.3 / 278.0 | 1.000× / 1.000× | 1.138× / 1.163× |
| `ctrl_fwd_4lane_ora` | 0.0 / 0.0 | 79.1 / 83.9 | 212.4 / 237.5 | 0.878× / 0.860× | 1.000× / 1.000× |
| `ref1013_fb75_L16` | 108.7 / 111.4 | 38.3 / 42.8 | 283.8 / 311.9 | 1.171× / 1.127× | 1.336× / 1.310× |
| `r1_clean1_2part` | **98.9 / 95.9** | 41.6 / 44.6 | 275.2 / 299.2 | **1.142× / 1.079×** | **1.299× / 1.261×** |
| `r2_mlp4_buf_bump` | 104.6 / 111.2 | 44.0 / 47.7 | 285.3 / 315.5 | 1.177× / 1.140× | 1.346× / 1.317× |

On 24 forced blocks of `text.bz2`, `r1_clean1_2part` still regressed **1.327× baseline** and **1.257× native** against 1-lane forward. Worse, the 4-lane MLP arm (`r2_mlp4_buf_bump`) ran slower than 1-lane `r1` on both builds. Variable chain lengths from 1 to 219 hops end a lane on **67.7%** of 4-lane iterations, causing branch mispredictions that flush out-of-order memory-level parallelism.

Why post-A6c contraction hits a structural floor (baseline x86-64 VM ms; dashed line = max budget vs 4-lane oracle)

Source: `docs/journal/2026-11-24-algorithm-level/bwt-contraction-rescue/results.md` (\#1038). Even with zero byte extraction, dictionary hashing or spelling writes, bare 4-lane pointer chasing (30.6 ms) already exceeds the entire \~24 to 25 ms VM budget needed to clear the +39.18 ms i9 gate over 4-lane forward emission.

The stop verdict is therefore structural rather than a missed micro-optimisation. Fusing elimination into `A6c` breaks `Reader::wire`'s SIMD loop, adding `+3.7` to `+16.1 ms` to save `3.4 ms` of tagging. Running elimination after `A6c` hits a **30.6 ms** bare 4-lane chase floor (`prep_tag` 3.4 ms plus `pure_chase4` 27.2 ms) with zero byte work, dictionary hashing or spelling writes. That bare floor already exceeds the \~24 to 25 ms VM budget.

Put `#1046` and `#1038` side by side and the boundary is sharp. `A6b` reopened because removable bookkeeping, bounds checks and allocations dominated three coupled sub-phases. Post-`A6c` contraction stopped because its irreducible chase floor exceeded what emission could give back.

## 4\. What ships next

These four merged results leave a clean, prioritized engineering queue. Every VM probe stays clearly separated from shipped production code and i9-10900K adoption gates:

1. **Already shipped (`#1040`).** The `within_cap` dense-band precheck is live in `src/open.rs`. It stays neutral on Wikidata and speeds four-thread CI `text` by **−3.7%**.
2. **Admitted build — [`segmented-fallback-build`](<https://github.com/lmmx/mibz2/blob/87401be2505b69d1bd153995bc8e0b984dd09aa9/frontier/claims/segmented-fallback-build.toml>) (`#1049`).** Port `C_trustfill+rev` into `src/bwt/` and `src/runs.rs` at zero extra memory. Confirm the gains across pinned CI corpora and the i9 workstation.
3. **Admitted build — [`a6b-endpoint-build`](<https://github.com/lmmx/mibz2/blob/0eaf87f993ed7d6156bb48e5d608428ce0304253/frontier/claims/a6b-endpoint-build.toml>) (`#1051`).** Port `comb_nobc_soa_gen` into `src/open/endpoint.rs` with zero `unsafe`. Verify RSS and 8- to 10-worker L3 cache scaling on the i9-10900K, where `A6b` sits on the worker half of the eight-worker tie.
4. **Admitted probe — [`forward-4lane-anchor-probe`](<https://github.com/lmmx/mibz2/blob/f2845b84bd3e8958745488f34af644d55481d005/frontier/claims/forward-4lane-anchor-probe.toml>) (`#1050`).** `#1038`'s own control table exposed a **\+110.7 to +129.0 ms** i9 gap between 1-lane forward emission and 4-lane oracle forward emission. This probe tests whether cheap in-`A6b`/`A6c` anchors can capture that 4-lane emission win without contraction's pointer-chase tax.
