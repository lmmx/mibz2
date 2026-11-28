# When the hardware says no: an i9 rejection, a 1.27 GB input, and a Round-0 bias

*Published on the blog on 27 November 2026 and retired the same day under the blog's admission test (Louis, 27 November 2026): kept here verbatim as a notebook entry. Its news, where it had any, is now told in the rewritten posts.*


<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# When the hardware says no: an i9 rejection, a 1.27 GB input, and a Round-0 bias

A physical i9 run rejects a scanner change while controlled probes isolate a 1.27 GB window blowup and a cold-start CI bias.
{: .post-dek }

27 November 2026 · 10 min read · [Research synthesis](#4-corpus-knees-forward-lanes-and-shallow-runners)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

**Real hardware and adversarial workloads just overruled three clean models.** Our previous synthesis mapped out the worker stack and serial parser on cloud VMs in [The 256 KiB L2 cliff, the 15-register wall, and the flipped eight-worker knee](2026-11-27-l2-cliff-register-wall-and-scanner-a2.md). Since then, the first paired workstation verdict landed on the physical 10-core Intel i9-10900K. Controlled probes also exposed a run-only window blowup and solved a phantom `+7.3%` CI regression across 240 paired rounds. Along the way, we confirmed station-uneven branch alignment and hardened shallow workstation checkouts.

One evidence boundary belongs up front. Only the `scanner-seam-window` verdict (`#1118`, `origin/evidence` `c0b2341c`) comes from the physical 10-core Intel i9-10900K workstation. The `pattern` four-thread dissection (`#1108`, `#1111`, `#1137`) analyzes 24 paired CI runs on 4-vCPU Xeon and EPYC runners. The run-only memory probe (`#1094`) and per-block scanner timing (`#1111`) ran on cloud VMs. So did the controlled A/A checks (`#1137`) and aligned station-balance ratios (`#1107`). Every VM diagnostic remains clearly separated from physical workstation verdicts.

[Section 1](#1-when-the-hardware-says-no-the-first-i9-verdict-1118-1133) reports the physical i9 rejection of `scanner-seam-window` and how it reopens serial scanner headroom at eight workers.
[Section 2](#2-a-1522-byte-input-that-ate-127-gb-of-ram-1094-1103) traces how run-only bzip2 blocks bypass the scanner's byte cap and caught a latent doubling trap before any code shipped.
[Section 3](#3-the-240-pair-detective-story-behind-a-phantom-73-ci-regression-1108-1111-1137) resolves the `#1034` four-thread `pattern` CI spike into a bimodal backlog regime and a Round-0 cold-start bias.
[Section 4](#4-corpus-knees-forward-lanes-and-shallow-runners) covers aligned station balance (`#1107`), merged four-lane forward emission (`#1110`), and the shallow-runner invariant (`#1122`–`#1127`).

**Takeaways**

- **The physical i9 rejected `scanner-seam-window` (`#1118`, `NOT ADOPTED`).** On `wikidata ×4` over 16 paired rounds, `10T` ran `1.89%` faster (`0.9811`), while `12T` missed the frozen gate (`0.9898 [0.9726, 1.0073]`). Scanner maintenance fell only `29.9%` (`10.69 → 7.49 ms`) instead of the modeled `80%`. PR `#1031` therefore closed unmerged, keeping serial `S'` `~10–13 ms` higher at `k = 8`.
- **A `1,522`-byte bzip2 stream expanded to `1.27 GB` of RAM (`#1094`, `#1103`).** `Shared::full` checks `READY_BYTES` only when the scanner issues a job. Run-only blocks (`~46.6 MB` each) fill the whole `slots` window before workers finish. Peak RSS reached `943 MiB` at `t = 4 / 32 slots` (`+24%–28%` slower from page faults) and `1,273 MiB` at the 8-thread default. Synthesis also caught a matching premise trap in `runt-exempt-window` before implementation.
- **A phantom `+7.3%` CI spike broke into two measurable harness mechanisms (`#1108`, `#1111`, `#1137`).** On `pattern`, `#1034` actually cut `A2` setup by `88.9%` (`9.0×` faster) and total `S` by `77.1%` (`4.36×` faster). Across 24 non-scanner CI runs, the spike comes from a symmetric bimodal `ready`-backlog RSS burst (`R² = 0.706`, slope `0.881`). It also reflects a `p = 0.0114` Round-0 head-first cold-start penalty (`×1.0605`), while warm `r1..r9` strict-null pairs sit at `1.0012` (`33/99` vs `33/99`, `p = 1.0000`).
- **Codegen shifts, feature-gated lanes, and shallow runners tightened the pipeline.** JCC 32-byte alignment pulls `wikidata` `k*` down `0.871×` baseline and `0.911×` native on VM silicon (`#1107`, `CONFIRMED`). Four-lane forward emission merged behind `--features forward-anchor-lanes` (`#1110`). Finally, an `INCOMPLETE` shallow-clone run turned `--depth 1` source fetches into a permanent recipe invariant (`#1122`–`#1127`).

## 1. When the hardware says no: the first i9 verdict (`#1118`, `#1133`)

Good stop tests earn their keep when they reject code that already passed unit tests, Miri, and review. On paper, `scanner-seam-window` (`#1031`) looked solid. Pre-`#1034` traces showed scanner window maintenance (`in_maint`) taking over `11 ms`. Replacing contiguous compaction with a chunked seam window promised an `80%` cut in maintenance overhead.

The physical i9-10900K tested candidate `3ae44390` against post-`#1034` base `95f4637b` across `16` paired rounds on `wikidata ×4`. All 16 untimed checks matched libbz2 byte for byte across `838,800,784 B`. [The recorded verdict](https://github.com/lmmx/mibz2/blob/4b53096b9ab181eaf099faaaa63aa905b95a53f6/frontier/deltas/2026-11-27-scanner-seam-window-i9-verdict.md) (`#1118`, archive `origin/evidence` `c0b2341c`) and [the thirteenth synthesis pass](https://github.com/lmmx/mibz2/blob/f00aaaf8b3e0a8fa6801f54926984469fb08e73b/docs/journal/2026-11-27-synthesis/thirteenth-pass.md) (`#1133`) give the complete five-count table:

| Threads | Role | Workers | Wall base → head (median) | Wall head/base `[95% CI]` | Verdict | CPU head/base `[95% CI]` | Peak RSS ratio |
|---:|---|---:|---:|---:|---|---:|---:|
| `6` | supporting | `6` | `0.244 → 0.244 s` | `0.9969 [0.9923, 1.0015]` | INCONCLUSIVE | `0.9907 [0.9835, 0.9980]` | `0.9689` |
| `8` | supporting | `8` | `0.197 → 0.195 s` | `1.0000 [0.9843, 1.0159]` | INCONCLUSIVE | `1.0000 [0.9893, 1.0107]` | `0.9885` |
| `9` | supporting | `9` | `0.196 → 0.190 s` | `0.9663 [0.9472, 0.9859]` | FASTER | `0.9815 [0.9606, 1.0029]` | `0.9695` |
| **`10`** | **judged** | **`8`** | **`0.197 → 0.195 s`** | **`0.9811 [0.9670, 0.9954]`** | **FASTER** | **`0.9891 [0.9798, 0.9986]`** | **`0.9846`** |
| **`12`** | **judged** | **`8`** | **`0.197 → 0.195 s`** | **`0.9898 [0.9726, 1.0073]`** | **INCONCLUSIVE** | **`0.9953 [0.9857, 1.0049]`** | **`0.9929`** |

The pre-registered rule required a 95% upper bound below `1.000` at both judged counts (`10` and `12` threads, both running eight workers). Ten threads cleared the bar (`−1.89%`), while twelve threads straddled parity (`1.0073`). Peak RSS improved at all five counts (`0.969–0.993`), yet the wall effect was too narrow to justify permanent complexity in the scanner's input loop. Under the frozen rule, the verdict was **`NOT ADOPTED`**, and PR `#1031` closed without merging.

### Why `in_maint` fell only `29.9%` and how it moves the eight-worker knee

The bundle's own timeline trace (`in-maint.txt`, eight workers, 32 slots) pinpointed why the wall gain shrank. Median scanner window maintenance fell from `10.689 ms` to `7.490 ms`. That `3.20 ms` (`−29.93%`) cut fell far short of the `80%` (`−11.2 ms`) drop projected from pre-`#1034` traces. Once `#1034` already tightened symbol scanning on the baseline, the remaining seam overhead was much smaller than earlier subtractions assumed.

That single rejection immediately reshaped our system budget in `#1133`. Earlier synthesis passes subtracted `#1031`'s projected `−13.35 ms` (`11.2 ms` maintenance plus `~2.15 ms` coupling) from post-`#1034` serial `S'`. Removing that unlanded subtraction puts post-`#1034` `S'` back `~10–13 ms` higher (derived). That leaves `S'` near `141–148 ms` on default codegen and `135–143 ms` aligned before `scanner-a2-fast-build`. Direct scanner headroom reopens at `k = 8`. Safe `A2` header parsing and slack-only hybrid scanning now regain direct eight-worker value instead of paying off only at `k = 9–10`.

## 2. A 1,522-byte input that ate 1.27 GB of RAM (`#1094`, `#1103`)

While `#1118` tested throughput on Wikidata's `~1 MB` blocks, [an adversarial memory probe](https://github.com/lmmx/mibz2/blob/0b1c0813c1c43a4c0c538103e317dbde3ac5fec8/docs/journal/2026-11-27-explore/expansive-block-memory.md) (`#1094`) asked what happens off the map. Every bzip2 block carries at most `900,000` RLE1-encoded bytes. Because five RLE1 bytes can expand to `259` output bytes, a single run-only block decodes to `~46.6 MB` from just a few dozen Huffman symbols.

In `#1094`, `2,147.5 MB` of zeros compressed with `bzip2 -9` into **`1,522` bytes** (`~46` blocks of `~46.6 MB`). Decoding that `1,522`-byte file with `bzcat` on a 4-vCPU Xeon VM produced a dramatic contrast against normal text (`211.2 MB` repo source, `~1 MB` blocks):

| Corpus | Threads | Slots | Wall (`ms`) | Throughput (`GB/s`) | Peak RSS (`MiB`) | Minor faults |
|---|---:|---:|---:|---:|---:|---:|
| `text` | `4` | `8` | `553–561` | `0.38` | `99–103` | `26.7k` |
| `text` | `8` | `32` | `590` | `0.36` | `148` | `42.5k` |
| `zeros` (`1,522 B`) | `1` | `1..16` | `778–863` | `2.5–2.8` | `149–150` | `38.0k` |
| `zeros` (`1,522 B`) | `4` | `4` | `426` | `5.04` | `457` | `116.7k` |
| `zeros` (`1,522 B`) | `4` | `8` | `402–413` | `5.2–5.3` | `591` | `151.0k` |
| `zeros` (`1,522 B`) | `4` | `16` | `513` | `4.18` | `591` | `151.0k` |
| `zeros` (`1,522 B`) | `4` | `32` | `497` | `4.32` | **`943`** | `241.1k` |
| `zeros` (`1,522 B`) | `8` | `32` | `597` | `3.60` | **`1,273`** | `325.6k` |
| `zeros` (`1,522 B`) | `16` | `64` | `685` | `3.14` | **`1,818`** | `532.4k` |

At the library's default 8-thread window (`32` slots via `SLOTS_PER_THREAD = 4`), that `1,522`-byte stream claimed **`1,273 MiB` (`1.27 GB`)** of RAM. That is an `~840,000×` input amplification. At `16` threads (`64` slots), memory hit `1,818 MiB`, bounded only because the stream held 46 blocks. Worse, the extra memory actively hurt speed. At `t = 4`, raising slots from `8` to `16` or `32` slowed wall time by **`24%–28%`** (`402 → 513 / 497 ms`). The kernel had to zero `~370 MB` of extra first-touch pages (`151.0k → 241.1k` minor faults).

### Why `READY_BYTES` missed it — and the premise trap caught before code

The bug sits cleanly in `src/parallel.rs`. `Shared::full` enforces the `12 MiB` `READY_BYTES` cap only when the *scanner* issues a new block, and only above a floor of `threads + 2` in flight. Run-only blocks take microseconds to scan, so the scanner floods the entire `slots` window with jobs before a single worker finishes block zero. Workers never check `READY_BYTES`; each expands its block into a `~46 MB` output vector, and `free_out` caches up to `slots` such buffers until the decoder drops.

Nobody had caught this on CI because `onebyte.bz2` at `SCALE = 4` is `20 MB` — less than one `~46 MB` block. Even better, [the eleventh synthesis pass](https://github.com/lmmx/mibz2/blob/583e9b83293997aa9bcf24945c15fe9f7938ffb8/docs/journal/2026-11-27-synthesis/eleventh-pass.md) (`#1103`) spotted a dangerous interaction with an active proposal before one line of code was written. The `runt-exempt-window` idea exempted tiny lbzip2 blocks from `slots` by checking the header's symbol bound (`n_sel × 50 < POOLED_FROM`). RLE2 encodes a `~46 MB` zero run in logarithmically few symbols (`~33` bytes compressed, `n_sel ≈ 1`). That header test therefore classifies every run-only block as a "runt" and would have issued `2 × slots` of them, roughly doubling the `1.27 GB` blowup. Synthesis stopped that trap cold and re-ordered the fix. Workers must gate on true decoded bytes first (`ready-cap-worker-gate`, admitted in `#1121`). Only after rows or decoded bytes are known can an exemption safely classify runts.

<figure class="bf-fig">
<div class="bf-title">When hardware and adversarial inputs overrule paper models (#1118, #1094)</div>
<svg viewBox="0 0 640 318" role="img" aria-label="Two-panel chart. Top panel shows the physical i9-10900K scanner-seam-window verdict (#1118): predicted in_maint drop was 80 percent (-11.2 ms) versus measured 29.9 percent (10.69 to 7.49 ms, -3.20 ms), while judged 10T wall ratio 0.9811 [0.9670, 0.9954] passed and 12T ratio 0.9898 [0.9726, 1.0073] crossed 1.000, yielding NOT ADOPTED. Bottom panel shows peak RSS from #1094 on a 1,522-byte zeros stream versus text: the 12 MiB scanner READY_BYTES cap is bypassed so zeros grows with slots from 457 MiB (4 slots) to 591 MiB (8 slots), 943 MiB (32 slots, t=4), 1,273 MiB (32 slots, t=8 default), and 1,818 MiB (64 slots, t=16), versus 99 to 148 MiB on text.">
  <!-- Panel A: i9 verdict (#1118) -->
  <text class="bf-t-strong" style="font-size:10.5px" x="18" y="18">A. PHYSICAL i9-10900K VERDICT ON scanner-seam-window (#1118 — NOT ADOPTED)</text>

  <!-- Left subpanel: in_maint predicted vs measured -->
  <line class="bf-rule" x1="128" y1="28" x2="128" y2="92"/>
  <line class="bf-grid" x1="188" y1="28" x2="188" y2="92"/>
  <line class="bf-grid" x1="248" y1="28" x2="248" y2="92"/>
  <line class="bf-grid" x1="308" y1="28" x2="308" y2="92"/>
  <!-- Scale: 0..12 ms -> 180 px (15 px/ms) -->
  <text class="bf-tm" x="122" y="44" text-anchor="end">Base in_maint</text>
  <rect x="128" y="33" width="160.3" height="13" rx="2" fill="var(--bf-vz-2)" opacity="0.8"/>
  <text class="bf-ttiny" x="293" y="43">10.69 ms</text>

  <text class="bf-tm" x="122" y="64" text-anchor="end">Predicted (−80%)</text>
  <rect x="128" y="53" width="32.1" height="13" rx="2" fill="var(--bf-green)" opacity="0.45" stroke="var(--bf-green)" stroke-dasharray="3 2"/>
  <text class="bf-ttiny" x="166" y="63">~2.1 ms</text>

  <text class="bf-t-strong" style="font-size:10px" x="122" y="84" text-anchor="end">Measured i9</text>
  <rect x="128" y="73" width="112.4" height="13" rx="2" fill="var(--bf-vz-s4)"/>
  <text class="bf-ttiny" x="246" y="83">7.49 ms (−29.9%)</text>

  <text class="bf-ttiny" x="128" y="106" text-anchor="middle">0</text>
  <text class="bf-ttiny" x="188" y="106" text-anchor="middle">4</text>
  <text class="bf-ttiny" x="248" y="106" text-anchor="middle">8</text>
  <text class="bf-ttiny" x="308" y="106" text-anchor="middle">12 ms</text>

  <!-- Right subpanel: 10T and 12T 95% CI strips around 1.000 -->
  <!-- Scale: 0.94..1.02 -> 200 px (2500 px per unit), x(1.00) = 420 + 0.06*2500 = 570 -->
  <line class="bf-rule" x1="570" y1="26" x2="570" y2="96"/>
  <line class="bf-grid" x1="470" y1="26" x2="470" y2="96"/>
  <line class="bf-grid" x1="520" y1="26" x2="520" y2="96"/>
  <line class="bf-grid" x1="620" y1="26" x2="620" y2="96"/>
  <text class="bf-ttiny" x="570" y="108" text-anchor="middle">1.00</text>
  <text class="bf-ttiny" x="520" y="108" text-anchor="middle">0.98</text>
  <text class="bf-ttiny" x="470" y="108" text-anchor="middle">0.96</text>
  <text class="bf-ttiny" x="620" y="108" text-anchor="middle">1.02</text>
  <text class="bf-ttiny" x="415" y="45" text-anchor="end">10T (8W)</text>
  <line x1="487.5" y1="42" x2="558.5" y2="42" style="stroke:var(--bf-green);stroke-width:3"/>
  <circle cx="522.8" cy="42" r="4" fill="var(--bf-green)"/>
  <text class="bf-t-green" style="font-size:9.5px" x="487" y="34">0.981 (pass)</text>

  <text class="bf-t-strong" style="font-size:10px" x="415" y="79" text-anchor="end">12T (8W)</text>
  <line x1="501.5" y1="76" x2="588.3" y2="76" style="stroke:var(--bf-vz-s5);stroke-width:3"/>
  <circle cx="544.5" cy="76" r="4" fill="var(--bf-vz-s5)"/>
  <text class="bf-ttiny" x="501" y="68">0.990 (crosses 1.00)</text>

  <!-- Divider -->
  <line class="bf-rule" x1="18" y1="124" x2="622" y2="124"/>

  <!-- Panel B: 1,522-byte run-only RSS blowup (#1094) -->
  <text class="bf-t-strong" style="font-size:10.5px" x="18" y="142">B. 1,522-BYTE RUN-ONLY INPUT PEAK RSS vs 12 MiB CAP (#1094 — VM, MiB)</text>
  <line class="bf-rule" x1="184" y1="152" x2="184" y2="294"/>
  <line class="bf-grid" x1="274" y1="152" x2="274" y2="294"/>
  <line class="bf-grid" x1="364" y1="152" x2="364" y2="294"/>
  <line class="bf-grid" x1="454" y1="152" x2="454" y2="294"/>
  <line class="bf-grid" x1="544" y1="152" x2="544" y2="294"/>

  <!-- Scale: 0..2000 MiB -> 360 px (0.18 px/MiB) -->
  <line x1="187" y1="150" x2="187" y2="294" style="stroke:var(--bf-green);stroke-width:1.4;stroke-dasharray:3 2"/>

  <text class="bf-tm" x="176" y="168" text-anchor="end">text (t=4/8 · t=8/32)</text>
  <rect x="184" y="157" width="26.6" height="14" rx="2" fill="var(--bf-green)"/>
  <text class="bf-ttiny" x="217" y="168">99–148 MiB (normal ~1 MB blocks)</text>

  <text class="bf-tm" x="176" y="191" text-anchor="end">zeros t=4, 4 slots</text>
  <rect x="184" y="180" width="82.3" height="14" rx="2" fill="var(--bf-vz-1)"/>
  <text class="bf-ttiny" x="272" y="191">457 MiB (426 ms)</text>

  <text class="bf-tm" x="176" y="214" text-anchor="end">zeros t=4, 8 slots</text>
  <rect x="184" y="203" width="106.4" height="14" rx="2" fill="var(--bf-vz-1)"/>
  <text class="bf-ttiny" x="296" y="214">591 MiB (402 ms · fastest)</text>

  <text class="bf-tm" x="176" y="237" text-anchor="end">zeros t=4, 32 slots</text>
  <rect x="184" y="226" width="169.7" height="14" rx="2" fill="var(--bf-vz-s3)"/>
  <text class="bf-ttiny" x="360" y="237">943 MiB (497 ms · +24% slower)</text>

  <text class="bf-t-strong" style="font-size:10px" x="176" y="260" text-anchor="end">zeros t=8, 32 slots (def)</text>
  <rect x="184" y="249" width="229.1" height="14" rx="2" fill="var(--bf-vz-s4)"/>
  <text class="bf-ttiny" x="419" y="260">1,273 MiB (~840,000× input)</text>

  <text class="bf-t-strong" style="font-size:10px" x="176" y="283" text-anchor="end">zeros t=16, 64 slots</text>
  <rect x="184" y="272" width="327.2" height="14" rx="2" fill="var(--bf-vz-s5)"/>
  <text class="bf-ttiny" x="517" y="283">1,818 MiB (46 blocks)</text>

  <text class="bf-ttiny" x="184" y="308" text-anchor="middle">0</text>
  <text class="bf-ttiny" x="274" y="308" text-anchor="middle">500</text>
  <text class="bf-ttiny" x="364" y="308" text-anchor="middle">1,000</text>
  <text class="bf-ttiny" x="454" y="308" text-anchor="middle">1,500</text>
  <text class="bf-ttiny" x="544" y="308" text-anchor="middle">2,000 MiB</text>
</svg>
<figcaption>Top: physical i9-10900K verdict on <code>scanner-seam-window</code> (<code>#1118</code>, 16 paired rounds on <code>wikidata ×4</code>). Bottom: peak RSS on a <code>1,522 B</code> run-only stream (<code>#1094</code>, 4-vCPU VM), where scanner-only <code>READY_BYTES</code> (dashed green line at <code>12 MiB</code>) lets memory scale with <code>slots × 46 MB</code>.</figcaption>
</figure>

## 3. The 240-pair detective story behind a phantom `+7.3%` CI regression (`#1108`, `#1111`, `#1137`)

[`#1034`](https://github.com/lmmx/mibz2/commit/5ed20c1d51afc56c088ae244ba0bf40f37cb93b1) merged coalesced same-selector runs, byte-aligned `u8` Huffman tables, and BMI2 bit extraction. Its exact-head CI run (`45947633032`, 10 paired rounds) turned up a puzzling red flag. Wikidata and text sped up, yet `pattern` at four threads flagged **slower** by **`+7.3%` (`+2.6%` to `+12.3%`)**. Even stranger, its peak RSS jumped **`+7.9%`** while total CPU *fell* **`−2.2%`**.

More wall time for less CPU is the classic signature of scheduling latency, not extra compute. Moreover, `pattern.bz2` (`SCALE = 4`, `b"0123456"` repeated to `19.6 MB`) compresses to just `1,116` bytes across `22` blocks. It holds only `2,661` Huffman symbols total and spends **`0.1%`** of single-thread decode time in the scanner. Three independent probes took that paradox apart piece by piece.

### Step 1: The scanner did not slow `pattern` — it made it `4.36×` faster (`#1111`)

[The per-block fixed-cost probe](https://github.com/lmmx/mibz2/blob/3047905859494ea9e532b29ce10906623c182551/docs/journal/2026-11-27-explore/scanner-pattern-regression.md) (`#1111`, `REFUTED`) timed `#1034` against its direct parent in `#![forbid(unsafe_code)]`. Both arms passed exact symbol parity across all `455` blocks (`25,044,344` symbols) of `pattern`, `runs`, `onebyte`, and `wikidata`. On `pattern`, every block has `n_sel = 3`, so `#1034`'s `PAIR_GATE_SEL = 64` skips 2-symbol pair-table construction on all `22/22` blocks:

- **Per-block `A2` setup:** dropped from `3.93` to `0.44 µs/block` (**`−88.9%`**, `9.0×` faster). Ranges were disjoint across 9 interleaved repeats on baseline and native builds.
- **Total serial scan `S`:** fell from `96.0` to `22.0 µs` across all 22 blocks. That is a **`−77.1%`** (`4.36×`) disjoint speedup.
- **Cold first-block latency:** fell from `30.4` to `17.1 µs` (`−43.9%`).

A scanner spending `0.022 ms` total cannot cause a `+1.5 ms` four-thread regression.

### Step 2: Neither binary identity nor arm labels explain it (`#1108`, `#1137`)

Before `#1111` even finished, [a null-pool survey](https://github.com/lmmx/mibz2/blob/d3bbe33f4676b28a2392c6d8999737af23c9a73d/docs/journal/2026-11-27-explore/ci-pattern-4t-null-bias.md) (`#1108`) checked `18` recent CI runs that never touched the scanner. Across all `21` corpus-by-thread cells, `pattern` at four threads was the lone outlier. Its head/base wall ratio hit `×1.0186` (run-cluster 95% CI `1.0027–1.0338`) with median RSS `×1.0169`. It flagged "slower" in `3/18` runs — including Python-only PR `#1033` (`45933661519`, zero Rust changes), which fired **`+6.0% [+0.2%, +12.2%]`** wall and `+5.9%` RSS.

That pointed straight at the benchmark harness: could building `base/ab` inside `.base-src` or spawning `current_exe()` for head versus `mibz2@base/ab` for base bias the head arm? [The controlled A/A probe](https://github.com/lmmx/mibz2/blob/3decbce86db308a7b455766a02eace11f01dfab1/docs/journal/2026-11-27-explore/ci-pattern-4t-arm-bias.md) (`#1137`, `NOT REPRODUCED`) audited both binaries and ran a four-arm factorial test at commit `d3bbe33f`:

- **ELF layout:** `bin/ab` and `.base-src`'s `base/ab` place all `1,606 / 1,606` `.text` symbols at identical addresses with identical opcodes (`diff_addr = 0`). The 10-byte source-path difference shifts only `.debug_str` and cold `.rodata` panic-string pointers.
- **Controlled 4-arm A/A (`100` `pattern` 4T pairs, 2-vCPU VM):** plain A/A (same binary) gave `0.9709 [0.9297, 1.0140]` in Arm B and `0.9784 [0.9115, 1.0503]` in Arm C. Two-build A/A gave `0.9910 [0.9531, 1.0303]` in Arm A and `0.9678 [0.9007, 1.0400]` in Arm D. All four 95% intervals cleanly contained `1`, with zero 10-round flags.
- **Out-of-sample CI check:** six subsequent strict-null CI runs (`60` pairs on real 4-vCPU runners, zero decoder edits) gave **`wall = 0.9970 [0.9865, 1.0075]`**. Peak RSS sat at `1.0028`, and **`0/6`** runs flagged.

### Step 3: The twin culprits across 24 CI runs (240 pairs)

We downloaded all 24 non-scanner CI artifacts (`#1108`'s 18 plus the 6 out-of-sample strict-null runs across five Xeon and EPYC 4-vCPU CPUs). That combined dataset exposed both physical mechanisms in full:

1. **A symmetric bimodal 6-on-4 `ready`-backlog RSS burst (`R² = 0.706` on strict-null).** At `t = 4`, `ab` runs seven threads (`main`, `mibz2-scan`, four workers, `sampler`) on 4 vCPUs / 2 physical cores. Whenever preemption delays `main` or the head-of-line worker, the remaining workers race ahead and allocate fresh unrecycled `956 KiB` output vectors. Peak RSS then jumps from a low mode (`36–38 MB`) into a high mode (`>= 39 MB`, up to `45 MB`). Across `110` strict-null pairs, the effect is completely symmetric. `both_low` (`n = 50`) sits at `wall = 1.0001 [0.9869, 1.0135]`, while `h_hi_b_lo` (`n = 27`) runs `1.1075` (`cpu 0.9693`) and `h_lo_b_hi` (`n = 22`) runs `0.9019` (`cpu 1.0280`). OLS regression of `log(wall)` on `log(rss)` explains `70.6%` of paired variance (`slope = 0.881`), leaving an RSS-adjusted `head/base` intercept of **`0.9964 [0.9872, 1.0057]`** (`1.0001 [0.9920, 1.0082]` across all 240 pairs).
2. **A genuine Round-0 (`r0`) cold-start asymmetry (`p = 0.0114`).** `ab run` has no warm-up round and always runs arm 0 (`head`) first in `r0`. In `r0`, `pattern` 4T is the suite's first invocation above `~34 MB` of anonymous memory. It also hits cold OS page allocation right after `t = 1` and `t = 2` leave two vCPUs idle. Across all 24 CI runs, `r0` alone shows **`wall = 1.0605 [1.0027, 1.1216]`** and `rss = 1.0542`. Head hits `>= 39 MB` in **`12/24`** runs vs **`3/24`** for base (Fisher exact `p = 0.0114`; strict-null `5/11` vs `0/11`, `p = 0.0351`). Once warm in **`r1..r9`**, all 11 strict-null runs (`99` pairs) collapse to **`wall = 1.0012 [0.9831, 1.0196]`** and **`33/99` vs `33/99`** high-RSS (`p = 1.0000`).

| CI `pattern` 4T slice | Pairs (`n`) | Wall head/base GM `[95% CI]` | RSS GM | High RSS (`>= 39 MB`) H / B | Fisher `p` |
|---|--:|--:|--:|--:|--:|
| All 24 runs · **`r0` only** | `24` | **`1.0605 [1.0027, 1.1216]`** | `1.0542` | **`12 / 3`** | **`0.0114`** |
| All 24 runs · `r1..r9` | `216` | `1.0081 [0.9938, 1.0221]` (boot) | `1.0112` | `77 / 62` | `0.1492` |
| All 11 strict-null · **`r0` only** | `11` | `1.0381 [0.9869, 1.0921]` | `1.0594` | **`5 / 0`** | **`0.0351`** |
| All 11 strict-null · **`r1..r9`** | `99` | **`1.0012 [0.9831, 1.0196]`** | `1.0042` | **`33 / 33`** | **`1.0000`** |
| OOS 6 strict-null · all rounds | `60` | **`0.9970 [0.9865, 1.0075]`** | `1.0028` | `18 / 17` | `0.840` |

That places `#1034`'s exact-head run (`45947633032`) cleanly in context. In that run, head hit `>= 39 MB` on `5/10` rounds versus `0/10` for base (`wall ×1.0734`, `rss ×1.0794`, `cpu −2.2%`). Across all three `#1034` CI runs (`45921211004`, `45947633032`, `45949114795`), its pooled high-RSS rate is `11/30` (`36.7%`) versus `30/80` (`37.5%`) on null controls. Adjusting `45947633032` by the strict-null wall–RSS slope (`0.881`) gives **`1.0035 [0.9725, 1.0355]`**. No bench or policy change follows from the probe alone, and the acceptance flag remains open for independent policy disposition.

<figure class="bf-fig">
<div class="bf-title">Anatomy of the CI pattern 4T spike: bimodal RSS symmetry and Round-0 cold start (#1111, #1137)</div>
<svg viewBox="0 0 640 314" role="img" aria-label="Two-panel chart from 24 non-scanner CI runs (240 paired rounds) on pattern at 4 threads. Top panel shows the symmetric bimodal RSS regime across 110 strict-null pairs (R^2 = 0.706, slope 0.881): both_low (n=50) sits at 1.0001, both_high (n=11) at 1.0035, head-high/base-low (n=27) at 1.1075 (+10.8% wall, -3.1% CPU), and head-low/base-high (n=22) at 0.9019 (-9.8% wall, +2.8% CPU); #1034 run 45947633032 drops from 1.0734 raw (5/10 vs 0/10 high-RSS) to 1.0035 [0.9725, 1.0355] RSS-adjusted. Bottom panel shows the Round-0 head-first cold-start penalty (r0 across 24 runs: 50% head vs 12.5% base high-RSS, wall 1.0605, p=0.0114; strict-null r0: 45.5% vs 0%, p=0.0351) collapsing to exact 33/99 vs 33/99 symmetry (33.3% vs 33.3%, wall 1.0012, p=1.0000) in rounds 1 through 9.">
  <!-- Panel A: Symmetric bimodal RSS regime (110 strict-null pairs + #1034) -->
  <text class="bf-t-strong" style="font-size:10.5px" x="18" y="18">A. BIMODAL RSS REGIME ON 110 STRICT-NULL CI PAIRS (WALL H/B, R² = 0.706)</text>
  <!-- Scale: 0.85..1.15 -> 360 px (1200 px per unit); x(1.00) = 196 + 180 = 376 -->
  <line class="bf-grid" x1="256" y1="26" x2="256" y2="152"/>
  <line class="bf-grid" x1="316" y1="26" x2="316" y2="152"/>
  <line class="bf-rule" x1="376" y1="26" x2="376" y2="152"/>
  <line class="bf-grid" x1="436" y1="26" x2="436" y2="152"/>
  <line class="bf-grid" x1="496" y1="26" x2="496" y2="152"/>
  <line class="bf-grid" x1="556" y1="26" x2="556" y2="152"/>

  <text class="bf-tm" x="188" y="42" text-anchor="end">Head low / Base high (n=22)</text>
  <line x1="228.6" y1="38" x2="288.6" y2="38" style="stroke:var(--bf-vz-1);stroke-width:3"/>
  <circle cx="258.3" cy="38" r="4" fill="var(--bf-vz-1)"/>
  <text class="bf-ttiny" x="295" y="42">0.902 (CPU +2.8%)</text>

  <text class="bf-t-strong" style="font-size:10px" x="188" y="64" text-anchor="end">Both low &lt;39 MB (n=50)</text>
  <line x1="360.3" y1="60" x2="392.2" y2="60" style="stroke:var(--bf-green);stroke-width:3"/>
  <circle cx="376.1" cy="60" r="4" fill="var(--bf-green)"/>
  <text class="bf-t-green" style="font-size:10px" x="399" y="64">1.000 [0.987, 1.014]</text>

  <text class="bf-tm" x="188" y="86" text-anchor="end">Both high ≥39 MB (n=11)</text>
  <line x1="324.4" y1="82" x2="438.9" y2="82" style="stroke:var(--bf-vz-2);stroke-width:3"/>
  <circle cx="380.2" cy="82" r="4" fill="var(--bf-vz-2)"/>
  <text class="bf-ttiny" x="445" y="86">1.004</text>

  <text class="bf-tm" x="188" y="108" text-anchor="end">Head high / Base low (n=27)</text>
  <line x1="470.9" y1="104" x2="539.9" y2="104" style="stroke:var(--bf-vz-s5);stroke-width:3"/>
  <circle cx="505.0" cy="104" r="4" fill="var(--bf-vz-s5)"/>
  <text class="bf-ttiny" x="545" y="108">1.108 (CPU −3.1%)</text>

  <text class="bf-t-strong" style="font-size:10px" x="188" y="138" text-anchor="end">#1034 raw → RSS-adjusted</text>
  <line x1="343.0" y1="134" x2="418.6" y2="134" style="stroke:var(--bf-green);stroke-width:3"/>
  <circle cx="380.2" cy="134" r="4" fill="var(--bf-green)"/>
  <circle cx="464.1" cy="134" r="4" fill="var(--bf-vz-s4)" opacity="0.85"/>
  <line x1="458" y1="134" x2="387" y2="134" style="stroke:var(--bf-vz-s4);stroke-width:1.3;stroke-dasharray:3 2"/>
  <text class="bf-ttiny" x="472" y="138">1.073 raw → 1.004 adj</text>

  <text class="bf-ttiny" x="256" y="164" text-anchor="middle">0.90</text>
  <text class="bf-ttiny" x="316" y="164" text-anchor="middle">0.95</text>
  <text class="bf-ttiny" x="376" y="164" text-anchor="middle">1.00</text>
  <text class="bf-ttiny" x="436" y="164" text-anchor="middle">1.05</text>
  <text class="bf-ttiny" x="496" y="164" text-anchor="middle">1.10</text>
  <text class="bf-ttiny" x="556" y="164" text-anchor="middle">1.15</text>

  <!-- Divider -->
  <line class="bf-rule" x1="18" y1="176" x2="622" y2="176"/>

  <!-- Panel B: Round 0 vs Rounds 1..9 high-RSS rate (%) -->
  <text class="bf-t-strong" style="font-size:10.5px" x="18" y="194">B. ROUND-0 COLD START vs WARM ROUNDS 1–9 (% OF RUNS ≥ 39 MB PEAK RSS)</text>
  <line class="bf-rule" x1="188" y1="204" x2="188" y2="294"/>
  <line class="bf-grid" x1="268" y1="204" x2="268" y2="294"/>
  <line class="bf-grid" x1="348" y1="204" x2="348" y2="294"/>
  <line class="bf-grid" x1="428" y1="204" x2="428" y2="294"/>
  <line class="bf-grid" x1="508" y1="204" x2="508" y2="294"/>

  <text class="bf-t-strong" style="font-size:10px" x="180" y="220" text-anchor="end">r0 Head (all 24 · p=0.011)</text>
  <rect x="188" y="209" width="266.7" height="13" rx="2" fill="var(--bf-vz-s5)"/>
  <text class="bf-ttiny" x="461" y="219">50.0% (12/24 · wall ×1.061)</text>

  <text class="bf-tm" x="180" y="238" text-anchor="end">r0 Base (all 24)</text>
  <rect x="188" y="227" width="66.7" height="13" rx="2" fill="var(--bf-vz-1)"/>
  <text class="bf-ttiny" x="261" y="237">12.5% (3/24)</text>

  <text class="bf-t-strong" style="font-size:10px" x="180" y="264" text-anchor="end">r1–r9 Head (11 strict-null)</text>
  <rect x="188" y="253" width="177.8" height="13" rx="2" fill="var(--bf-green)"/>
  <text class="bf-t-green" style="font-size:10px" x="372" y="263">33.3% (33/99 · wall ×1.001, p=1.00)</text>

  <text class="bf-tm" x="180" y="282" text-anchor="end">r1–r9 Base (11 strict-null)</text>
  <rect x="188" y="271" width="177.8" height="13" rx="2" fill="var(--bf-vz-1)"/>
  <text class="bf-ttiny" x="372" y="281">33.3% (33/99 · exact symmetry)</text>

  <text class="bf-ttiny" x="188" y="306" text-anchor="middle">0%</text>
  <text class="bf-ttiny" x="268" y="306" text-anchor="middle">15%</text>
  <text class="bf-ttiny" x="348" y="306" text-anchor="middle">30%</text>
  <text class="bf-ttiny" x="428" y="306" text-anchor="middle">45%</text>
  <text class="bf-ttiny" x="508" y="306" text-anchor="middle">60%</text>
</svg>
<figcaption>Source: <code>docs/journal/2026-11-27-explore/ci-pattern-4t-arm-bias.md</code> (<code>#1137</code>) across 24 non-scanner CI runs (<code>240</code> paired rounds on five Xeon/EPYC 4-vCPU runner models) plus <code>#1034</code> run <code>45947633032</code>. Panel A shows symmetric wall flips across the <code>39 MB</code> peak-RSS threshold; Panel B isolates the head-first Round-0 cold start versus exact symmetry in Rounds 1–9.</figcaption>
</figure>

## 4. Corpus knees, forward lanes, and shallow runners

Three more merged results tightened how the decoder is built, measured, and dispatched to the i9 workstation.

### JCC 32-byte alignment tips `k*` unevenly (`#1107`, `#1109`)

[The aligned station-balance probe](https://github.com/lmmx/mibz2/blob/e4bef3e614b5ffa3b79feb4cd0d49e47dc03df60/docs/journal/2026-11-27-explore/station-balance-aligned.md) (`#1107`, `CONFIRMED`) tested Intel JCC 32-byte branch alignment (`+2.9%` `.text`) on commit `f3055a45`. It ran `-C llvm-args=-x86-branches-within-32B-boundaries` across all eight corpora from [`#1075`](https://github.com/lmmx/mibz2/blob/534bc652f5dfadcd5a6b07253e7e91c6200d8d34/docs/journal/2026-11-27-explore/station-balance-off-wikidata.md), and [`#1109`](https://github.com/lmmx/mibz2/blob/578e5951ec380e94dd4339ad24987513882ffb45/docs/journal/2026-11-27-synthesis/twelfth-pass.md) folded the result into the knee map.

On `wikidata`, alignment sped up `OpenCells` workers `3.4–3.9×` more than the scanner. Baseline workers dropped **`−16.41%`** vs scanner `−4.17%`, pulling `k* = W/S` from `10.15` to `8.84` (**`r_aligned = 0.8709`**, disjoint). Native workers dropped **`−12.01%`** vs scanner `−3.50%`, pulling `k*` from `9.20` to `8.38` (**`r_aligned = 0.9109`**, disjoint). Across all seven off-Wikidata corpora (`prose` through `elf`), the steal-free min-pass ratio moved downward on **`16 / 16`** cells. Shifts stayed milder on Segmented-heavy text (`0.922–0.939` baseline across `json`, `mdjournal`, `logs`, and `elf`). Scaling from the shipped i9 knee (`k* ≈ 8.0`) puts aligned `wikidata` at **`k* ≈ 6.97–7.29`** (derived). Direct hardware testing on Comet Lake-S stepping 5 comes next.

<figure class="bf-fig">
<div class="bf-title">Station-uneven JCC 32-byte alignment shift across corpora (#1107 — VM baseline)</div>
<svg viewBox="0 0 640 228" role="img" aria-label="Grouped bar chart from #1107 comparing default versus JCC 32-byte aligned k-star (W/S) on VM baseline across five disjoint corpora. On wikidata (OpenCells), k-star drops from 10.15 to 8.84 (r_aligned = 0.871, workers -16.4% vs scanner -4.2%). On Segmented-heavy off-Wikidata corpora, the shift is milder: logs drops 6.52 to 6.01 (0.922), json 5.86 to 5.50 (0.939), mdjournal 4.67 to 4.31 (0.923), and elf 4.16 to 3.89 (0.935).">
  <text class="bf-t-strong" style="font-size:10.5px" x="18" y="18">DEFAULT vs JCC-32B ALIGNED k* = W / S (DISJOINT BASELINE CORPORA, #1107)</text>
  <line class="bf-rule" x1="156" y1="28" x2="156" y2="196"/>
  <line class="bf-grid" x1="236" y1="28" x2="236" y2="196"/>
  <line class="bf-grid" x1="316" y1="28" x2="316" y2="196"/>
  <line class="bf-grid" x1="396" y1="28" x2="396" y2="196"/>
  <line class="bf-grid" x1="476" y1="28" x2="476" y2="196"/>
  <line class="bf-grid" x1="556" y1="28" x2="556" y2="196"/>

  <text class="bf-t-strong" style="font-size:10.5px" x="148" y="47" text-anchor="end">wikidata (OpenCells)</text>
  <rect x="156" y="32" width="338.3" height="11" rx="2" fill="var(--bf-vz-2)" opacity="0.8"/>
  <rect x="156" y="45" width="294.7" height="11" rx="2" fill="var(--bf-green)"/>
  <text class="bf-t-green" style="font-size:10px" x="500" y="54">10.15 → 8.84 (0.871×)</text>

  <text class="bf-tm" x="148" y="81" text-anchor="end">logs (7.22×)</text>
  <rect x="156" y="66" width="217.3" height="11" rx="2" fill="var(--bf-vz-2)" opacity="0.8"/>
  <rect x="156" y="79" width="200.3" height="11" rx="2" fill="var(--bf-vz-1)"/>
  <text class="bf-ttiny" x="380" y="88">6.52 → 6.01 (0.922×)</text>

  <text class="bf-tm" x="148" y="115" text-anchor="end">json (8.11×)</text>
  <rect x="156" y="100" width="195.3" height="11" rx="2" fill="var(--bf-vz-2)" opacity="0.8"/>
  <rect x="156" y="113" width="183.3" height="11" rx="2" fill="var(--bf-vz-1)"/>
  <text class="bf-ttiny" x="358" y="122">5.86 → 5.50 (0.939×)</text>

  <text class="bf-tm" x="148" y="149" text-anchor="end">mdjournal (3.90×)</text>
  <rect x="156" y="134" width="155.7" height="11" rx="2" fill="var(--bf-vz-2)" opacity="0.8"/>
  <rect x="156" y="147" width="143.7" height="11" rx="2" fill="var(--bf-vz-1)"/>
  <text class="bf-ttiny" x="319" y="156">4.67 → 4.31 (0.923×)</text>

  <text class="bf-tm" x="148" y="183" text-anchor="end">elf (2.96×)</text>
  <rect x="156" y="168" width="138.7" height="11" rx="2" fill="var(--bf-vz-2)" opacity="0.8"/>
  <rect x="156" y="181" width="129.7" height="11" rx="2" fill="var(--bf-vz-1)"/>
  <text class="bf-ttiny" x="302" y="190">4.16 → 3.89 (0.935×)</text>

  <text class="bf-ttiny" x="156" y="212" text-anchor="middle">0</text>
  <text class="bf-ttiny" x="236" y="212" text-anchor="middle">2.4</text>
  <text class="bf-ttiny" x="316" y="212" text-anchor="middle">4.8</text>
  <text class="bf-ttiny" x="396" y="212" text-anchor="middle">7.2</text>
  <text class="bf-ttiny" x="476" y="212" text-anchor="middle">9.6</text>
  <text class="bf-ttiny" x="556" y="212" text-anchor="middle">12.0</text>
</svg>
<figcaption>Diagnostic VM baseline medians (<code>#1107</code>, 3 interleaved rounds pinned to CPU 0). Gray upper bars show default codegen; colored lower bars show <code>-x86-branches-within-32B-boundaries</code>. Because <code>OpenCells</code> carries more 32-byte branch crossings than <code>Segmented</code>, alignment drops <code>wikidata</code> <code>k*</code> hardest (<code>0.871×</code> baseline, <code>0.911×</code> native).</figcaption>
</figure>

### Four-lane forward emission lands, and shallow clones become a harness invariant

Two operational milestones rounded out the cycle:

- **Four-lane forward emission merged (`#1110`, [`7a7a207a`](https://github.com/lmmx/mibz2/commit/7a7a207adfc2fee830b1b46f7fa6707d61271f0a)).** The Open kernel's `S = 16` sampled-anchor four-lane walk landed behind `--features forward-anchor-lanes` with zero `unsafe`. All three worker pillars (safe `A6b`, L1-batched MTF, and four-lane forward emission) now sit on concrete workstation validation tracks.
- **Shallow-runner invariant (`#1122`, `#1126`, `#1127`).** The first A6b workstation run ended [`INCOMPLETE`](https://github.com/lmmx/mibz2/blob/7ad3e0fa328ea3f9686c95316633931cc8c81be8/frontier/deltas/2026-11-27-a6b-endpoint-i9-incomplete.md) before compiling because the runner uses a `--depth 1` shallow checkout. Fetching a commit into `FETCH_HEAD` and running `git clone --local` dropped the fetched object. Both active recipes ([`#1126`](https://github.com/lmmx/mibz2/commit/c3db2e396ad61b6fbcdef58d8a5f9285234b22be) for `bench/scanner_coalesced.py`, [`#1127`](https://github.com/lmmx/mibz2/commit/9e5db348d75a56c6600eb3ea7fce861ce5c9d51f) for `bench/a6b_endpoint.py`) now fetch exact commits directly into a private temporary repo. Each script also ships a detached `--depth 1 --no-local` unit test, making shallow safety a standing invariant for every future i9 recipe.

### What stays with us

| Study | Environment | Verdict | Operational lesson carried forward |
|---|---|---|---|
| `#1118` `scanner-seam-window` | **Physical i9-10900K** | **`NOT ADOPTED`** | Keep pre-registered two-count gates; `in_maint` fell `29.9%`, restoring `~10–13 ms` of `k = 8` scanner room |
| `#1094`, `#1103` run-only window | 4-vCPU VM + synthesis | **ADMITTED FIX** | Gate parallel workers on true decoded bytes, never header symbol counts (`1,522 B → 1.27 GB`) |
| `#1108`, `#1111`, `#1137` `pattern` 4T | 24 CI runs + VM A/A | **`REFUTED` / `NOT REPRODUCED`** | Separate scheduling RSS bursts (`R² = 0.706`) and Round-0 cold starts (`p = 0.0114`) from compute regressions |
| `#1107` aligned station balance | 2-vCPU VM | **`CONFIRMED`** | Track codegen-induced `k*` shifts (`0.871×`/`0.911×` on `wikidata`) before attributing knee moves |
| `#1122`, `#1126`, `#1127` shallow runner | i9 runner + unit tests | **INVARIANT** | Test every workstation benchmark script against a detached `--depth 1 --no-local` checkout before dispatch |

Each of those five outcomes narrowed the gap between what a clean model predicts and what silicon actually executes. Pre-registered hardware stops keep marginal complexity out of `src/`, and byte-bounded worker admission protects against `840,000×` adversarial inputs. Round-0 and RSS telemetry keep noisy CI cells legible, while shallow-clone unit tests prevent wasted workstation cycles. With the repaired i9 queue now running, the next wave of physical hardware verdicts can speak cleanly.
