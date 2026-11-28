---
title: "Faster decoding for ordinary text, not just Wikidata"
image: assets/social/blog-2026-11-27-faster-decoding-for-ordinary-text.png
description: "Our fastest kernel passes over five in six blocks of ordinary text, so mibz2 now foresees its refusals and gives the general-purpose path a speed-up of its own."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Faster decoding for ordinary text, not just Wikidata

Our fastest kernel passes over five in six blocks of ordinary text, so mibz2 now foresees its refusals and gives the general-purpose path a speed-up of its own.
{: .post-dek }

27 November 2026 · 12 min read · [Research synthesis](#5-what-this-changes)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

mibz2 is a bzip2 decoder, written in Rust, that decodes a file's blocks in parallel. Each bzip2
block holds up to 900,000 bytes, stored in the order produced by the Burrows–Wheeler transform
(BWT), and most of the work of decoding a block is undoing that transform. The classical way back
is a walk through the sorted rows of the block, one row at a time, each step a lookup that depends
on the one before; [The mathematical structure of inverse-BWT navigation](2026-11-08-inverse-bwt-navigation.md)
works it through. This month mibz2 found a faster route. Adjacent rows that hold the same byte, a
*run*, move together, so a kernel can follow one row of each run and let the rest come for free
([A new way back from the BWT](2026-11-10-rauzy-veech-reconstruction.md)). The kernel built on
that idea, published as [Open](2026-11-19-open-returns.md), keeps only the rows that sit in short
runs and jumps between them, writing whole words at a time. Below we call it the fast kernel.

The fast kernel was designed and tuned on one corpus, an 11.9 MB extract of a [Wikidata](https://www.wikidata.org/) dump that decompresses
to 210 MB of JSON. There
it takes all 233 full-size blocks, and in our single-thread timings it inverts them in under half
the time of the older kernel it replaced. That older kernel, the general-purpose kernel, builds a
table with one entry per byte of the block and walks it in 512 segments, 32 at a time
([32 lanes, 8.6 chains](2026-10-31-room-walk-overlap.md)). It can decode any block, and the
decoder falls back on it whenever the fast kernel declines one.

This post asks what happens on everything else: prose, source code, JSON, executables. The answer
is that the fast kernel mostly does not apply. Across seven ordinary corpora it took only 17 of
107 full blocks, about 16%, and the general-purpose kernel did the rest. Worse, on the dense blocks
typical of prose and source code, the fast kernel ran its first pass up to a limit before giving
up, so those blocks paid for both kernels. We found that every such refusal can be predicted, in
one cheap pass, from a list the decoder already holds. We then turned to the kernel that ordinary
text actually uses, and made it faster. Both changes are now in mibz2's default decoder on its
main branch.

**Takeaways**

- **The fast kernel is a Wikidata specialist.** It takes all of Wikidata's full blocks, but only
  16% of those in seven other corpora; the other 84% go to the general-purpose kernel.
- **On dense blocks, it wasted work before declining.** All 25 dense blocks it met outside
  Wikidata were refused only after its first pass had reached its limit. On a single thread, that
  added up to 48% to their inversion time and made a whole prose file 1.09 to 1.32 times slower to
  decode than with the general-purpose kernel alone.
- **One pass over the block's runs foresees every refusal.** Checking the limit before the fast
  kernel starts brings those blocks down to the general-purpose kernel's own time. On the
  project's cloud benchmark, four-thread decoding of a text corpus became 3.7% faster, and
  Wikidata was unchanged.
- **The general-purpose kernel's hidden cost is building its table.** Three changes that need no
  extra memory cut its inversion time by 15 to 16% on prose and source code in a controlled probe.
  In the shipped decoder, the cloud benchmark measured text decoding 2.7% faster on one thread and
  4.3% faster on two.

## 1. Which blocks can the fast kernel take?

The fast kernel reads a block as runs rather than as bytes. While the decoder undoes bzip2's
move-to-front stage, it can note at almost no cost where each run of repeated bytes begins and how
long it is. [Reading the BWT's runs off bzip2's move-to-front codes](2026-11-08-bwt-runs-from-mtf-codes.md)
shows why the codes already contain that list, which we call the block's *run records*. A block of
long runs has few records for its length. The ratio of records to rows, written $r/n$, therefore
measures how dense a block is with short runs.

<aside class="il-gloss" markdown>**Run records** are pairs of a byte and a length. The six bytes
`nnbaaa` make three of them: (`n`, 2), (`b`, 1) and (`a`, 3). A Wikidata block of 900,000 bytes,
whose $r/n$ sits near 0.068, has about 61,000.</aside>

The fast kernel's advantage comes from long runs, which it can skip, so it accepts only blocks of
low density. Between 1/64 and 0.15 it takes a block outright. Between 0.15 and 0.28, the *dense
band*, it must also check how many rows it would have to handle one by one: those lying in runs of
at most 16. It takes the block only if that count stays within $\min(256{,}000,\ 0.3n)$, a limit
set when [dense blocks were first admitted](2026-11-21-a-memory-gain-is-a-gain.md) so as to take
Wikidata's five. Blocks outside both ranges go to the general-purpose kernel. So do blocks of nearly random
bytes, for which the decoder never forms run records at all, since they would cost more than they
save; we call that the *column route*.

To see how often each case occurs, we built seven corpora from files any Linux machine holds:
prose from documentation, Python's standard library, Rust source from the cargo registry, JSON,
ELF executables, a DNA-like stream of four letters, and random bytes. We compressed each with
`bzip2 -9` and logged the route of every full block, meaning every block of at least 100,000
bytes. Where each corpus's full blocks went:

<figure class="bf-fig" id="routes">
<p class="bf-title">Where the full blocks go: Wikidata against seven ordinary corpora</p>
<!-- fig:routes -->
<svg viewBox="0 0 720 372" role="img" aria-label="Stacked bars of where each corpus's full blocks go. Wikidata: all 233 to the fast kernel. Across seven ordinary corpora, 107 full blocks: 17 to the fast kernel, 25 refused after a wasted first pass, 10 more sent from the run route to the fallback, and 55 on the column route."><rect x="20.0" y="12.0" width="12.0" height="12.0" rx="2" style="fill:var(--bf-vz-1)"></rect><text class="bf-tm" x="38.0" y="22.0" text-anchor="start" style="font-size:12px">fast kernel</text><rect x="150.0" y="12.0" width="12.0" height="12.0" rx="2" style="fill:var(--bf-vz-2)"></rect><text class="bf-tm" x="168.0" y="22.0" text-anchor="start" style="font-size:12px">refused after a wasted first pass</text><rect x="390.0" y="12.0" width="12.0" height="12.0" rx="2" style="fill:var(--bf-vz-3)"></rect><text class="bf-tm" x="408.0" y="22.0" text-anchor="start" style="font-size:12px">run route, sent to fallback</text><rect x="20.0" y="32.0" width="12.0" height="12.0" rx="2" style="fill:var(--bf-vz-4)"></rect><text class="bf-tm" x="38.0" y="42.0" text-anchor="start" style="font-size:12px">column route, fallback</text><text class="bf-tm" x="690.0" y="42.0" text-anchor="end" style="font-size:12px">share of a corpus's full blocks</text><line x1="170.0" y1="58.0" x2="170.0" y2="346.0" style="stroke:var(--bf-vz-axis);stroke-width:1"/><text class="bf-tm" x="170.0" y="362.0" text-anchor="middle" style="font-size:11px">0%</text><line x1="300.0" y1="58.0" x2="300.0" y2="346.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="300.0" y="362.0" text-anchor="middle" style="font-size:11px">25%</text><line x1="430.0" y1="58.0" x2="430.0" y2="346.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="430.0" y="362.0" text-anchor="middle" style="font-size:11px">50%</text><line x1="560.0" y1="58.0" x2="560.0" y2="346.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="560.0" y="362.0" text-anchor="middle" style="font-size:11px">75%</text><line x1="690.0" y1="58.0" x2="690.0" y2="346.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="690.0" y="362.0" text-anchor="middle" style="font-size:11px">100%</text><text class="bf-tm" x="162.0" y="79.0" text-anchor="end" style="font-size:12px">Wikidata (benchmark) · 233</text><rect x="170.0" y="64.0" width="520.0" height="22.0" rx="0" style="fill:var(--bf-vz-1)"><title>Wikidata (benchmark): 233 of 233 full blocks, fast kernel</title></rect><text x="430.0" y="79.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-1-ink)">233</text><text class="bf-tm" x="162.0" y="111.0" text-anchor="end" style="font-size:12px">prose · 8</text><rect x="170.0" y="96.0" width="65.0" height="22.0" rx="0" style="fill:var(--bf-vz-1)"><title>prose: 1 of 8 full blocks, fast kernel</title></rect><text x="202.5" y="111.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-1-ink)">1</text><rect x="235.0" y="96.0" width="455.0" height="22.0" rx="0" style="fill:var(--bf-vz-2)"><title>prose: 7 of 8 full blocks, refused after a wasted first pass</title></rect><text x="462.5" y="111.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-2-ink)">7</text><text class="bf-tm" x="162.0" y="143.0" text-anchor="end" style="font-size:12px">Python source · 11</text><rect x="170.0" y="128.0" width="47.3" height="22.0" rx="0" style="fill:var(--bf-vz-1)"><title>Python source: 1 of 11 full blocks, fast kernel</title></rect><text x="193.6" y="143.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-1-ink)">1</text><rect x="217.3" y="128.0" width="94.5" height="22.0" rx="0" style="fill:var(--bf-vz-2)"><title>Python source: 2 of 11 full blocks, refused after a wasted first pass</title></rect><text x="264.5" y="143.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-2-ink)">2</text><rect x="311.8" y="128.0" width="330.9" height="22.0" rx="0" style="fill:var(--bf-vz-3)"><title>Python source: 7 of 11 full blocks, run route, sent to fallback</title></rect><text x="477.3" y="143.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-3-ink)">7</text><rect x="642.7" y="128.0" width="47.3" height="22.0" rx="0" style="fill:var(--bf-vz-4)"><title>Python source: 1 of 11 full blocks, column route, fallback</title></rect><text x="666.4" y="143.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-4-ink)">1</text><text class="bf-tm" x="162.0" y="175.0" text-anchor="end" style="font-size:12px">Rust source · 16</text><rect x="170.0" y="160.0" width="260.0" height="22.0" rx="0" style="fill:var(--bf-vz-1)"><title>Rust source: 8 of 16 full blocks, fast kernel</title></rect><text x="300.0" y="175.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-1-ink)">8</text><rect x="430.0" y="160.0" width="260.0" height="22.0" rx="0" style="fill:var(--bf-vz-2)"><title>Rust source: 8 of 16 full blocks, refused after a wasted first pass</title></rect><text x="560.0" y="175.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-2-ink)">8</text><text class="bf-tm" x="162.0" y="207.0" text-anchor="end" style="font-size:12px">JSON · 16</text><rect x="170.0" y="192.0" width="227.5" height="22.0" rx="0" style="fill:var(--bf-vz-1)"><title>JSON: 7 of 16 full blocks, fast kernel</title></rect><text x="283.8" y="207.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-1-ink)">7</text><rect x="397.5" y="192.0" width="130.0" height="22.0" rx="0" style="fill:var(--bf-vz-2)"><title>JSON: 4 of 16 full blocks, refused after a wasted first pass</title></rect><text x="462.5" y="207.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-2-ink)">4</text><rect x="527.5" y="192.0" width="162.5" height="22.0" rx="0" style="fill:var(--bf-vz-4)"><title>JSON: 5 of 16 full blocks, column route, fallback</title></rect><text x="608.8" y="207.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-4-ink)">5</text><text class="bf-tm" x="162.0" y="239.0" text-anchor="end" style="font-size:12px">ELF binaries · 33</text><rect x="170.0" y="224.0" width="63.0" height="22.0" rx="0" style="fill:var(--bf-vz-2)"><title>ELF binaries: 4 of 33 full blocks, refused after a wasted first pass</title></rect><text x="201.5" y="239.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-2-ink)">4</text><rect x="233.0" y="224.0" width="31.5" height="22.0" rx="0" style="fill:var(--bf-vz-3)"><title>ELF binaries: 2 of 33 full blocks, run route, sent to fallback</title></rect><text x="248.8" y="239.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-3-ink)">2</text><rect x="264.5" y="224.0" width="425.5" height="22.0" rx="0" style="fill:var(--bf-vz-4)"><title>ELF binaries: 27 of 33 full blocks, column route, fallback</title></rect><text x="477.3" y="239.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-4-ink)">27</text><text class="bf-tm" x="162.0" y="271.0" text-anchor="end" style="font-size:12px">DNA-like · 14</text><rect x="170.0" y="256.0" width="37.1" height="22.0" rx="0" style="fill:var(--bf-vz-3)"><title>DNA-like: 1 of 14 full blocks, run route, sent to fallback</title></rect><text x="188.6" y="271.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-3-ink)">1</text><rect x="207.1" y="256.0" width="482.9" height="22.0" rx="0" style="fill:var(--bf-vz-4)"><title>DNA-like: 13 of 14 full blocks, column route, fallback</title></rect><text x="448.6" y="271.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-4-ink)">13</text><text class="bf-tm" x="162.0" y="303.0" text-anchor="end" style="font-size:12px">random bytes · 9</text><rect x="170.0" y="288.0" width="520.0" height="22.0" rx="0" style="fill:var(--bf-vz-4)"><title>random bytes: 9 of 9 full blocks, column route, fallback</title></rect><text x="430.0" y="303.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-4-ink)">9</text><line x1="20.0" y1="314.0" x2="690.0" y2="314.0" style="stroke:var(--bf-vz-axis);stroke-width:1"/><text class="bf-t-strong" x="162.0" y="335.0" text-anchor="end" style="font-size:12px">all seven · 107</text><rect x="170.0" y="320.0" width="82.6" height="22.0" rx="0" style="fill:var(--bf-vz-1)"><title>all seven: 17 of 107 full blocks, fast kernel</title></rect><text x="211.3" y="335.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-1-ink)">17</text><rect x="252.6" y="320.0" width="121.5" height="22.0" rx="0" style="fill:var(--bf-vz-2)"><title>all seven: 25 of 107 full blocks, refused after a wasted first pass</title></rect><text x="313.4" y="335.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-2-ink)">25</text><rect x="374.1" y="320.0" width="48.6" height="22.0" rx="0" style="fill:var(--bf-vz-3)"><title>all seven: 10 of 107 full blocks, run route, sent to fallback</title></rect><text x="398.4" y="335.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-3-ink)">10</text><rect x="422.7" y="320.0" width="267.3" height="22.0" rx="0" style="fill:var(--bf-vz-4)"><title>all seven: 55 of 107 full blocks, column route, fallback</title></rect><text x="556.4" y="335.0" text-anchor="middle" style="font-size:11.5px;font-weight:600;fill:var(--bf-vz-4-ink)">55</text></svg>
<figcaption>Measured: the route each full block took, from the route census (the number after each
name is its count of full blocks). "Refused after a wasted first pass" are the dense-band blocks;
"run route, sent to fallback" are blocks outside the fast kernel's range of densities; the column route never forms run
records.</figcaption>
</figure>

On Wikidata the fast kernel takes everything, but elsewhere it takes 17 of 107 blocks. The column
route takes 55, nearly all from the executables, the DNA-like data and the random bytes. The run
route sends another 35 to the general-purpose kernel, and 25 of those are dense-band blocks from
prose, source code, JSON and executables. Where the fast kernel does run, it is as good off
Wikidata as on it. It inverts JSON blocks in 0.38 of the general-purpose kernel's time and Rust
source in 0.76, against 0.48 on Wikidata itself. The kernel is not weak on ordinary text; it is
simply rarely eligible.

## 2. What does a refusal cost, and can it be foreseen?

All 25 dense-band blocks outside Wikidata were refused, and every one was refused late. The fast
kernel learns how many rows it must keep by running its first pass, and stops only when the count
crosses the limit. By then the pass has done most of its work, and the general-purpose kernel
starts again from nothing. We timed each refused block twice on one thread, once through the
decoder as shipped and once through the general-purpose kernel directly:

| corpus | refused blocks | as shipped, ms | general-purpose alone, ms | ratio |
|---|---:|---:|---:|---:|
| prose | 7 | 21.0 | 17.1 | 1.23 |
| Rust source | 8 | 28.0 | 20.6 | 1.36 |
| JSON | 4 | 17.6 | 11.8 | 1.48 |
| ELF executables | 4 | 16.4 | 14.5 | 1.13 |
| Python source | 2 | 7.0 | 6.9 | 1.02 |

Together the refusals added about 19 ms over the 25 blocks, roughly 0.8 ms a block. Prose suffered
most, because nearly all its full blocks are dense: the whole file decoded 1.09 to 1.32 times
slower than it would have with the general-purpose kernel alone. These timings come from a virtual
machine with four virtual processors. They are good for ratios rather than absolute speeds, and the
whole-file range is wide because the machine is noisy. The direction, though, is not in doubt. On
ordinary prose, the decoder's own specialised kernel was making it slower.

The remedy follows from what the first pass counts. The rows it keeps are those in maximal runs of
at most 16 rows, plus the one row that marks where the text begins. That count can be read off the
run records directly. One merges adjacent records of the same byte, since the format splits long
runs, and adds up the lengths of the short ones. It is a single pass over a list several times
shorter than the block, made before the fast kernel allocates or computes anything.

The two populations separate cleanly. The least of the refused blocks keeps 257,495 rows, against
a limit of 256,000, and the prose blocks keep between 311,318 and 380,411. Wikidata's admitted dense
blocks, by contrast, keep between 199,189 and 231,600. A second limit, on a structure the kernel
calls thick strips, never came close to binding.

The decoder now makes that count for dense-band blocks before the fast kernel starts, and refuses
on the spot any block that would exceed the limit. We measured the effect over seven interleaved
repeats, on corpora rebuilt on a fresh machine, which held 24 dense refusals. The refused blocks'
inversion time fell to the general-purpose kernel's own:

<figure class="bf-fig" id="refusals">
<p class="bf-title">Dense blocks the fast kernel refuses: before and after counting first</p>
<!-- fig:refusals -->
<svg viewBox="0 0 720 304" role="img" aria-label="Dot plot of inverse time on dense blocks the fast kernel refuses, relative to the fallback alone. Before the change: prose 1.49, Python source 1.16, Rust source 1.45, JSON 1.49, ELF 1.43, all 24 blocks 1.40. After: 0.95, 0.89, 1.04, 0.87, 1.04, and 0.98 overall."><circle cx="26" cy="18" r="5" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:2"/><text class="bf-tm" x="38.0" y="22.0" text-anchor="start" style="font-size:12px">before: the fast kernel's first pass, then the fallback</text><circle cx="26" cy="38" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"/><text class="bf-tm" x="38.0" y="42.0" text-anchor="start" style="font-size:12px">after: refused from the run records, straight to the fallback</text><line x1="190.0" y1="48.0" x2="190.0" y2="260.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="190.0" y="276.0" text-anchor="middle" style="font-size:11px">0.8×</text><line x1="315.0" y1="48.0" x2="315.0" y2="260.0" style="stroke:var(--bf-vz-axis);stroke-width:1.5"/><text class="bf-tm" x="315.0" y="276.0" text-anchor="middle" style="font-size:11px">1.0×</text><line x1="440.0" y1="48.0" x2="440.0" y2="260.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="440.0" y="276.0" text-anchor="middle" style="font-size:11px">1.2×</text><line x1="565.0" y1="48.0" x2="565.0" y2="260.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="565.0" y="276.0" text-anchor="middle" style="font-size:11px">1.4×</text><line x1="690.0" y1="48.0" x2="690.0" y2="260.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="690.0" y="276.0" text-anchor="middle" style="font-size:11px">1.6×</text><text class="bf-tm" x="319.0" y="44.0" text-anchor="start" style="font-size:11px">same time as the fallback alone</text><text class="bf-tm" x="180.0" y="77.0" text-anchor="end" style="font-size:12px">prose (7)</text><line x1="283.7" y1="73.0" x2="618.8" y2="73.0" style="stroke:var(--bf-vz-axis);stroke-width:2"/><circle cx="618.8" cy="73.0" r="5" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:2"><title>prose: before 1.49×</title></circle><circle cx="283.7" cy="73.0" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>prose: after 0.95×</title></circle><text class="bf-tm" x="628.8" y="77.0" text-anchor="start" style="font-size:11px">1.49×</text><text class="bf-tm" x="273.7" y="77.0" text-anchor="end" style="font-size:11px">0.95×</text><text class="bf-tm" x="180.0" y="111.0" text-anchor="end" style="font-size:12px">Python source (3)</text><line x1="245.0" y1="107.0" x2="417.5" y2="107.0" style="stroke:var(--bf-vz-axis);stroke-width:2"/><circle cx="417.5" cy="107.0" r="5" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:2"><title>Python source: before 1.16×</title></circle><circle cx="245.0" cy="107.0" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>Python source: after 0.89×</title></circle><text class="bf-tm" x="427.5" y="111.0" text-anchor="start" style="font-size:11px">1.16×</text><text class="bf-tm" x="235.0" y="111.0" text-anchor="end" style="font-size:11px">0.89×</text><text class="bf-tm" x="180.0" y="145.0" text-anchor="end" style="font-size:12px">Rust source (3)</text><line x1="340.0" y1="141.0" x2="593.1" y2="141.0" style="stroke:var(--bf-vz-axis);stroke-width:2"/><circle cx="593.1" cy="141.0" r="5" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:2"><title>Rust source: before 1.45×</title></circle><circle cx="340.0" cy="141.0" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>Rust source: after 1.04×</title></circle><text class="bf-tm" x="603.1" y="145.0" text-anchor="start" style="font-size:11px">1.45×</text><text class="bf-tm" x="330.0" y="145.0" text-anchor="end" style="font-size:11px">1.04×</text><text class="bf-tm" x="180.0" y="179.0" text-anchor="end" style="font-size:12px">JSON (3)</text><line x1="235.6" y1="175.0" x2="622.5" y2="175.0" style="stroke:var(--bf-vz-axis);stroke-width:2"/><circle cx="622.5" cy="175.0" r="5" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:2"><title>JSON: before 1.49×</title></circle><circle cx="235.6" cy="175.0" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>JSON: after 0.87×</title></circle><text class="bf-tm" x="632.5" y="179.0" text-anchor="start" style="font-size:11px">1.49×</text><text class="bf-tm" x="225.6" y="179.0" text-anchor="end" style="font-size:11px">0.87×</text><text class="bf-tm" x="180.0" y="213.0" text-anchor="end" style="font-size:12px">ELF binaries (8)</text><line x1="337.5" y1="209.0" x2="585.0" y2="209.0" style="stroke:var(--bf-vz-axis);stroke-width:2"/><circle cx="585.0" cy="209.0" r="5" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:2"><title>ELF binaries: before 1.43×</title></circle><circle cx="337.5" cy="209.0" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>ELF binaries: after 1.04×</title></circle><text class="bf-tm" x="595.0" y="213.0" text-anchor="start" style="font-size:11px">1.43×</text><text class="bf-tm" x="327.5" y="213.0" text-anchor="end" style="font-size:11px">1.04×</text><line x1="20.0" y1="226.0" x2="690.0" y2="226.0" style="stroke:var(--bf-vz-axis);stroke-width:1"/><text class="bf-t-strong" x="180.0" y="247.0" text-anchor="end" style="font-size:12px">all 24 blocks</text><line x1="305.0" y1="243.0" x2="567.5" y2="243.0" style="stroke:var(--bf-vz-axis);stroke-width:2"/><circle cx="567.5" cy="243.0" r="5" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:2"><title>all 24 blocks: before 1.40×</title></circle><circle cx="305.0" cy="243.0" r="5" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2"><title>all 24 blocks: after 0.98×</title></circle><text class="bf-tm" x="577.5" y="247.0" text-anchor="start" style="font-size:11px">1.40×</text><text class="bf-tm" x="295.0" y="247.0" text-anchor="end" style="font-size:11px">0.98×</text><text class="bf-tm" x="690.0" y="294.0" text-anchor="end" style="font-size:11px">inverse time on the refused blocks, over the fallback's own time on the same blocks</text></svg>
<figcaption>Measured on a virtual machine, one thread, seven interleaved repeats. Each point is
the summed inversion time of a corpus's refused blocks as the decoder runs them, over the
general-purpose kernel's time on the same blocks (block counts in brackets). Values a little below
1 are within the machine's noise.</figcaption>
</figure>

Taken block by block, the median ratio fell from 1.536 to 1.027. Where the check passes, it costs
almost nothing. On Wikidata's five dense blocks, which are still admitted, it adds about 0.27 ms in
all, some 0.03% of a single-thread decode, and all 233 full blocks still take the fast kernel.

The project's cloud benchmark then compared the decoder before and after the change, over ten
paired rounds on a four-core runner. Its text corpus is built from mibz2's own source code and
documentation. Four-thread decoding of it was 3.7% faster, with a 95% interval of 0.9% to 6.4%. At
one and two threads the benchmark detected no change on that corpus, and it detected none on
Wikidata at any thread count.

## 3. Where does the general-purpose kernel spend its time?

With the wasted pass gone, the general-purpose kernel does nearly all the work on ordinary text,
yet its costs on such text had never been dissected. A second census, over six corpora of 82 MB in
all, found that it inverted 78 of their 86 full blocks: 48 from run records and 30 from the column
route. On text, prose, source code and JSON, undoing the BWT took 52 to 66% of a worker's time.
Within that, the first step, building the table that the walk follows, took 35 to 46%.

The table holds, for each row, its byte and the row the walk visits next. Three changes, none of
which needs extra memory, attacked the cost of building and using it.

**Building in order.** On the column route the kernel built a table that walks forwards. That
meant reading the block's bytes and scattering each entry into one of 256 places in the table, a
random write per byte. The *LF* table, which sends each row to the row one byte earlier in the
text, can instead be built in a single sequential pass, with a shortcut for eight identical bytes
in a row. On the 30 column-route blocks this change alone cut the kernel's processor time by 23.4%.

**Reversing with vector instructions.** Walking LF spells each segment of the block backwards, so
every segment must be reversed as it is copied into place. The existing loop reversed a byte at a
time, and the compiler did not vectorise it. Reversing 32 bytes at a time with AVX2 shuffles, with
an eight-byte fallback for other processors, cut that step by 45% on the prose, text and
source-code blocks.

**Trusting what is already known.** On the run route, the table is filled from the run records.
The filling loop checked, on every record, that it stayed inside the block and inside its byte's
share of the table: about 225,000 checks a block on the text corpus. Those facts are already
established by the code that produced the records. The records therefore now carry a flag, set
only by that code and cleared by anything that changes them, and a flagged list is filled without
the checks. With all three changes, the run-route blocks took 13.5% less processor time.

<aside class="il-gloss" markdown>**The flag** is a memory-safety argument, not a shortcut taken on
trust. The new unchecked loops and the vector code were reviewed for soundness and run under
[Miri](https://github.com/rust-lang/miri), Rust's interpreter for detecting undefined behaviour,
before they were merged. Any list built or altered elsewhere still takes the checked path.</aside>

We timed the kernel's own processor time, so that the host's interruptions did not count, over
seven interleaved repeats on a virtual machine with two virtual processors. Together the three
changes cut inversion time on the 50 prose, text and source-code blocks by 15.4% in a build for any
x86-64 processor, and by 16.1% in one tuned to the machine. On all 78 blocks the cuts were 18.4% and
18.5%. Every corpus improved by at least 10.5%, the executables most, by about a quarter, and the
output matched the existing kernel byte for byte on every block.

A classic idea from the literature shows why construction, not the walk, was the cost to attack.
[Kärkkäinen, Kempa and Puglisi](https://researchportal.helsinki.fi/en/publications/slashing-the-time-for-bwt-inversion/)
speed up BWT inversion with a table that advances two bytes per lookup, halving the walk's chain of
dependent reads. We tried it. The walk became 15 to 16% faster, but building the larger table took
about four times as long. The inversion as a whole ran more than twice as slow, with 3.6 MiB more
memory per worker. Where building the table is already a third or more of the cost, a richer
table is the wrong trade.

## 4. How much of that reached the decoder?

The probe measured the kernel in isolation. The changes were then built into the decoder, and the
cloud benchmark compared it with its predecessor over ten paired rounds, on a four-core runner with
two physical cores. Every corpus gave the same output from both builds, and peak memory showed no
significant rise. On the text corpus the decoder was 2.7% faster on one thread, with a 95% interval
of 0.1% to 5.2%, and 4.3% faster on two, with one of 0.6% to 7.8%. At four threads its 2.9% gain
was within noise. The random-bytes corpus, whose blocks take the column route as random bytes did in our census, was 5.3% faster on
one thread and 6.2% on four.

That is less than the probe implied. Since undoing the BWT is about two-thirds of a worker's time
on the text corpus, the probe projected a gain of 8 to 10% there, and a quick, informal check on a
development machine suggested 7 to 9% on one thread. The controlled measurement, the one we stand
by, came out at a third to a half of the projection. We have not yet dissected the difference. The
probe and the benchmark ran on different processors, and they measure different things: the
kernel's own processor time in one, the whole decoder's elapsed time in the other.

Two smaller observations remain open. Wikidata, whose full blocks all take the fast kernel, came out
1.7% faster on one thread, a gain the benchmark's stage-by-stage timings do not locate. At four
threads, the benchmark also flagged a 1.8% rise in the extra processor time Wikidata costs as
threads are added, which only a machine with more cores can judge.

## 5. What this changes

The argument forms a short chain. A kernel tuned on one corpus became the decoder's pride, but on
ordinary files it is eligible for only one full block in six. Where it hesitated over a dense
block, it paid for a pass it then threw away, and made prose slower than the fallback alone. That
refusal turned out to be a simple count over a list the decoder already holds, so it is now made
before any work is wasted. The fallback, which is what ordinary text actually runs, spends a third
or more of its inversion building its table, and three memory-neutral changes have made that
markedly cheaper.

None of these figures yet comes from the ten-core workstation on which mibz2's releases are
measured. They come from virtual machines and the project's cloud benchmark, and both changes will
first be measured on the workstation with the next release. The broader lesson concerns benchmarks
more than kernels. A corpus chosen because it matters can quietly decide which code is ever
examined, and the next speed-up may be found by decoding something else.

---

**Sources:**

- The route census: [the research note](https://github.com/lmmx/mibz2/blob/b8fd08af523b3d501d5f797faaa1aa96348f70c7/docs/journal/2026-11-27-explore/route-census.md), its [per-block timings](https://github.com/lmmx/mibz2/blob/b8fd08af523b3d501d5f797faaa1aa96348f70c7/docs/journal/2026-11-27-explore/route-census/run-route-blocks.tsv) and [the census program](https://github.com/lmmx/mibz2/blob/b8fd08af523b3d501d5f797faaa1aa96348f70c7/examples/route_census.rs) ([#1023](https://github.com/lmmx/mibz2/pull/1023)); 4-vCPU Xeon virtual machine, single thread.
- The early refusal check: [the change to the fast kernel's admission](https://github.com/lmmx/mibz2/blob/2915d7e79f7d53f0bfc9155c3005d47d4fdc895a/src/open.rs) and [its record](https://github.com/lmmx/mibz2/blob/2915d7e79f7d53f0bfc9155c3005d47d4fdc895a/frontier/deltas/2026-11-27-dense-cap-precheck.md) ([#1040](https://github.com/lmmx/mibz2/pull/1040)), with the [benchmark report](https://github.com/lmmx/mibz2/pull/1040#issuecomment-7207435327) (AMD EPYC 9V45 runner, 10 paired rounds).
- The general-purpose kernel's dissection: [the research note](https://github.com/lmmx/mibz2/blob/9f89ba509f8f0348d1b4358b95a48b7199112940/docs/journal/2026-11-27-explore/segmented-fallback-dissection.md) and [the probe](https://github.com/lmmx/mibz2/blob/9f89ba509f8f0348d1b4358b95a48b7199112940/examples/segmented_dissection_probe.rs) ([#1042](https://github.com/lmmx/mibz2/pull/1042)); 2-vCPU Xeon virtual machine, seven interleaved repeats. The two-step table is recorded in [the dead ends](https://github.com/lmmx/mibz2/blob/9f89ba509f8f0348d1b4358b95a48b7199112940/frontier/dead-ends.tsv).
- The change in the decoder: [#1080](https://github.com/lmmx/mibz2/pull/1080), its [benchmark report](https://github.com/lmmx/mibz2/pull/1080#issuecomment-7211122449) (Intel Xeon 6975P-C runner, 10 paired rounds), its [safety review](https://github.com/lmmx/mibz2/pull/1092) and [closing record](https://github.com/lmmx/mibz2/blob/1a459081ccf203b9f98706073595b4fcc6701257/frontier/deltas/2026-11-27-segmented-fallback-build-done.md).
- The dense band's limit: [Admitting dense blocks to the Open kernel](2026-11-21-a-memory-gain-is-a-gain.md).
- Juha Kärkkäinen, Dominik Kempa and Simon J. Puglisi, [Slashing the Time for BWT Inversion](https://researchportal.helsinki.fi/en/publications/slashing-the-time-for-bwt-inversion/), DCC 2012.
