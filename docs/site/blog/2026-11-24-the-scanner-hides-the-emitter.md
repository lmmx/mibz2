---
title: "At ten workers the serial scanner hides the emitter's gain: both stages supplied save 26%"
image: assets/social/blog-2026-11-24-the-scanner-hides-the-emitter.png
description: "Supplying both the serial scanner's output and the emission codes cuts whole-decoder time by 26% at ten workers, more than either stage supplied alone."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# At ten workers the serial scanner hides the emitter's gain: both stages supplied save 26%

Supplying both the serial scanner's output and the emission codes cuts whole-decoder time by 26% at ten workers, more than either stage supplied alone.
{: .post-dek }

24 November 2026 · 7 min read · [Research result](#5-what-it-changes)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

Our first supplied test, handing mibz2's emitter its decoding order for free made the whole decoder 16% faster
on one thread and barely faster on ten
([the supplied test](https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060)), which suggested that at ten workers there was little to gain from
cheaper emission. A four-arm test now shows that suggestion was wrong. At ten workers the decoder
waits on its serial scanner, the single thread that reads the compressed bits ahead of the
workers, and the scanner hid the emitter's gain. Supplying the scanner's output alone cuts
whole-decoder time by 19%; supplying it together with the emission codes cuts it by 26%, and the
codes, which bought 2% on their own, then buy a further 8%. These are oracle bounds, measured with
the supplied work done outside the clock, so they say how much time a better algorithm could free,
not that we have one.

**Takeaways**

- At 1 and 6 workers the workers are the bottleneck: supplied codes save 15 to 17%, and a supplied
  scanner saves 2% at most.
- At 10 workers the scanner is the bottleneck: a supplied scanner saves 19%, and supplied codes
  alone save nothing distinguishable from zero.
- Together they save 26% at 10 workers, 8% more than the scanner alone, which by the rule fixed
  before the data is an interaction.
- The gain is there to be had only by an algorithm that changes both stages at once, and the
  scanner's serial cost comes first.

## 1. The decoder as a small factory

mibz2 decodes a bzip2 file in two kinds of work. One thread, the scanner, reads the compressed file
in order and decodes each block's Huffman codes into symbols
([the Huffman stage](2026-11-05-huffman-bit-buffer.md)). The workers, running in parallel, take one
block each: they undo the move-to-front and run-length coding, invert the
Burrows–Wheeler transform and emit the text
([why the scanner is serial](2026-11-07-why-mibz2-stops-scaling.md)). The scanner is the
factory's doorman, unpacking each parcel at the door, and the workers are its assemblers.

<figure class="bf-fig" id="factory">
<p class="bf-title">One doorman, many assemblers: where the decoder waits</p>
<svg viewBox="0 0 720 330" role="img" aria-label="The decoder as a small factory. One scanner thread reads the compressed bits and decodes each block's Huffman codes in order, and queues the blocks. Workers undo the move-to-front and run-length coding, invert the block and emit its text. With one worker, blocks queue at the worker; with ten workers, the workers wait for the scanner."><text class="bf-t-strong" x="20" y="22" style="font-size:14px">One worker: the worker is the bottleneck</text><text class="bf-tm" x="20" y="39" style="font-size:12px">supplied codes save 17%; a supplied scanner saves nothing</text><rect x="20" y="56" width="150" height="84" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text class="bf-t-strong" x="95" y="82" text-anchor="middle" style="font-size:13px">the scanner</text><text class="bf-tm" x="95" y="100" text-anchor="middle" style="font-size:11.5px">reads the bits, decodes</text><text class="bf-tm" x="95" y="116" text-anchor="middle" style="font-size:11.5px">Huffman: one thread</text><text class="bf-tm" x="95" y="132" text-anchor="middle" style="font-size:11px;fill:var(--bf-amber)">arm C supplies this</text><line x1="176" x2="372" y1="98" y2="98" style="stroke:var(--bf-frame);stroke-width:2"/><path d="M372,92 l9,6 l-9,6 z" style="fill:var(--bf-frame)"/><rect x="350" y="84" width="22" height="14" rx="2" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel)"/><rect x="320" y="84" width="22" height="14" rx="2" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel)"/><rect x="290" y="84" width="22" height="14" rx="2" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel)"/><rect x="260" y="84" width="22" height="14" rx="2" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel)"/><rect x="230" y="84" width="22" height="14" rx="2" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel)"/><text class="bf-tm" x="274" y="122" text-anchor="middle" style="font-size:11.5px">blocks queue for the workers</text><rect x="400" y="56" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;"/><text class="bf-tm" x="427" y="78" text-anchor="middle" style="font-size:11px">working</text><text class="bf-tm" x="470" y="72" style="font-size:11.5px">a worker: undoes MTF and RLE2,</text><text class="bf-tm" x="470" y="88" style="font-size:11.5px">inverts the BWT, emits the text</text><text class="bf-tm" x="470" y="104" style="font-size:11px;fill:var(--bf-steel)">arm B supplies its codes</text><text class="bf-t-strong" x="20" y="178" style="font-size:14px">Ten workers: everyone waits at the door</text><text class="bf-tm" x="20" y="195" style="font-size:12px">a supplied scanner saves 19%; with it, supplied codes save 8% more</text><rect x="20" y="212" width="150" height="84" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/><text class="bf-t-strong" x="95" y="238" text-anchor="middle" style="font-size:13px">the scanner</text><text class="bf-tm" x="95" y="256" text-anchor="middle" style="font-size:11.5px">reads the bits, decodes</text><text class="bf-tm" x="95" y="272" text-anchor="middle" style="font-size:11.5px">Huffman: one thread</text><text class="bf-tm" x="95" y="288" text-anchor="middle" style="font-size:11px;fill:var(--bf-amber)">arm C supplies this</text><line x1="176" x2="372" y1="254" y2="254" style="stroke:var(--bf-frame);stroke-width:2"/><path d="M372,248 l9,6 l-9,6 z" style="fill:var(--bf-frame)"/><text class="bf-tm" x="274" y="278" text-anchor="middle" style="font-size:11.5px">no blocks waiting</text><rect x="400" y="212" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;opacity:0.55"/><text class="bf-tm" x="427" y="234" text-anchor="middle" style="font-size:11px">waiting</text><rect x="462" y="212" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;opacity:0.55"/><text class="bf-tm" x="489" y="234" text-anchor="middle" style="font-size:11px">waiting</text><rect x="524" y="212" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;opacity:0.55"/><text class="bf-tm" x="551" y="234" text-anchor="middle" style="font-size:11px">waiting</text><rect x="586" y="212" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;opacity:0.55"/><text class="bf-tm" x="613" y="234" text-anchor="middle" style="font-size:11px">waiting</text><rect x="648" y="212" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;opacity:0.55"/><text class="bf-tm" x="675" y="234" text-anchor="middle" style="font-size:11px">waiting</text><rect x="400" y="256" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;opacity:0.55"/><text class="bf-tm" x="427" y="278" text-anchor="middle" style="font-size:11px">waiting</text><rect x="462" y="256" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;opacity:0.55"/><text class="bf-tm" x="489" y="278" text-anchor="middle" style="font-size:11px">waiting</text><rect x="524" y="256" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;opacity:0.55"/><text class="bf-tm" x="551" y="278" text-anchor="middle" style="font-size:11px">waiting</text><rect x="586" y="256" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;opacity:0.55"/><text class="bf-tm" x="613" y="278" text-anchor="middle" style="font-size:11px">waiting</text><rect x="648" y="256" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;opacity:0.55"/><text class="bf-tm" x="675" y="278" text-anchor="middle" style="font-size:11px">waiting</text></svg>
<figcaption>Schematic. With one worker, blocks queue for the worker, so making the worker's job
cheaper (arm B) saves time and making the scanner's cheaper (arm C) does not. With ten workers,
the workers wait for the scanner, so only a cheaper scanner helps until the door is cleared, and
then a cheaper worker job helps again.</figcaption>
</figure>

## 2. The test: four arms, two stages supplied

The question was whether removing two costs together frees time that removing either alone does
not. The two costs are the scanner's Huffman decoding and the code ordering in [Open](2026-11-19-open-returns.md)'s emission,
the stage the earlier supplied test had halved. One binary carries four arms
([the registration](https://github.com/lmmx/mibz2/blob/5731db1908d58fa4a51d22baf473d2f61ccd670e/docs/journal/2026-11-24-four-arm-registration.md)):

| Arm | Scanner | Emission |
|---|---|---|
| **A** | as shipped | as shipped |
| **B** | as shipped | its codes supplied, in output order |
| **C** | its output supplied: each block's recorded symbols, replayed through the production job queue | as shipped |
| **D** | supplied | supplied |

In arms C and D the scanner thread reads no bit and decodes no Huffman code; it copies each block's
recorded symbols into the same buffer the scanner would have filled and sends the block on, so
everything downstream runs as in production and stays on the clock. The run decoded our Wikidata
benchmark four times over, at 1, 6 and 10 workers, for 20 rounds with the arms in a balanced order,
on one i9-10900K. Every round's output matched the reference: 838,800,784 bytes, with no mismatch.

The verdict was fixed before any data. At 10 workers the result is an **interaction** if D's
upper 95% bound is at most 0.90 of A, D sits more than 0.03 below the better single arm, and D
beats that arm outright; it is additive if only the first holds, and there is no potential if D's
lower bound is above 0.95. The run is also invalid unless arm B reproduces the earlier supplied
test, which it does: 0.976 against 0.988, with overlapping intervals.

## 3. Results

<figure class="bf-fig" id="four-arm-results">
<p class="bf-title">Which supplied stage frees time, by worker count</p>
<svg viewBox="0 0 720 380" role="img" aria-label="Whole-decoder time against arm A at 1, 6 and 10 workers, with 95% intervals. At 1 worker: codes 0.83, scanner 1.00, both 0.83. At 6: 0.85, 0.98, 0.83. At 10: codes 0.98, scanner 0.81, both 0.74."><line x1="70" x2="696" y1="302.0" y2="302.0" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="62" y="306.0" text-anchor="end" style="font-size:13px">0.70</text><line x1="70" x2="696" y1="262.3" y2="262.3" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="62" y="266.3" text-anchor="end" style="font-size:13px">0.75</text><line x1="70" x2="696" y1="222.6" y2="222.6" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="62" y="226.6" text-anchor="end" style="font-size:13px">0.80</text><line x1="70" x2="696" y1="182.9" y2="182.9" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="62" y="186.9" text-anchor="end" style="font-size:13px">0.85</text><line x1="70" x2="696" y1="143.1" y2="143.1" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="62" y="147.1" text-anchor="end" style="font-size:13px">0.90</text><line x1="70" x2="696" y1="103.4" y2="103.4" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="62" y="107.4" text-anchor="end" style="font-size:13px">0.95</text><line x1="70" x2="696" y1="63.7" y2="63.7" style="stroke:var(--bf-frame);stroke-width:2;"/><text class="bf-tm" x="62" y="67.7" text-anchor="end" style="font-size:13px">1.00</text><line x1="70" x2="696" y1="24.0" y2="24.0" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text class="bf-tm" x="62" y="28.0" text-anchor="end" style="font-size:13px">1.05</text><text class="bf-tm" x="76" y="57.7" style="font-size:12px">as shipped (arm A)</text><text class="bf-tm" x="18" y="163" text-anchor="middle" transform="rotate(-90 18 163)" style="font-size:12px">whole-decoder time ÷ arm A (lower is faster)</text><text class="bf-ts" x="174" y="326" text-anchor="middle" style="font-size:14px">1 worker</text><line x1="128" x2="128" y1="195.6" y2="202.7" style="stroke:var(--bf-steel);stroke-width:3"/><circle cx="128" cy="198.7" r="6" style="fill:var(--bf-steel)"/><text class="bf-t-strong" x="128" y="220.7" text-anchor="middle" style="font-size:12px">0.830</text><line x1="174" x2="174" y1="62.9" y2="70.9" style="stroke:var(--bf-amber);stroke-width:3"/><circle cx="174" cy="66.9" r="6" style="fill:var(--bf-amber)"/><text class="bf-t-strong" x="174" y="88.9" text-anchor="middle" style="font-size:12px">0.996</text><line x1="220" x2="220" y1="197.9" y2="206.7" style="stroke:var(--bf-green);stroke-width:3"/><circle cx="220" cy="202.7" r="6" style="fill:var(--bf-green)"/><text class="bf-t-strong" x="220" y="224.7" text-anchor="middle" style="font-size:12px">0.825</text><text class="bf-ts" x="383" y="326" text-anchor="middle" style="font-size:14px">6 workers</text><line x1="337" x2="337" y1="178.9" y2="187.6" style="stroke:var(--bf-steel);stroke-width:3"/><circle cx="337" cy="182.9" r="6" style="fill:var(--bf-steel)"/><text class="bf-t-strong" x="337" y="205.6" text-anchor="middle" style="font-size:12px">0.850</text><line x1="383" x2="383" y1="74.8" y2="80.4" style="stroke:var(--bf-amber);stroke-width:3"/><circle cx="383" cy="78.0" r="6" style="fill:var(--bf-amber)"/><text class="bf-t-strong" x="383" y="98.4" text-anchor="middle" style="font-size:12px">0.982</text><line x1="429" x2="429" y1="195.6" y2="201.1" style="stroke:var(--bf-green);stroke-width:3"/><circle cx="429" cy="198.7" r="6" style="fill:var(--bf-green)"/><text class="bf-t-strong" x="429" y="219.1" text-anchor="middle" style="font-size:12px">0.830</text><text class="bf-ts" x="592" y="326" text-anchor="middle" style="font-size:14px">10 workers</text><line x1="546" x2="546" y1="50.2" y2="114.5" style="stroke:var(--bf-steel);stroke-width:3"/><circle cx="546" cy="82.8" r="6" style="fill:var(--bf-steel)"/><text class="bf-t-strong" x="546" y="132.5" text-anchor="middle" style="font-size:12px">0.976</text><line x1="592" x2="592" y1="197.9" y2="236.9" style="stroke:var(--bf-amber);stroke-width:3"/><circle cx="592" cy="217.8" r="6" style="fill:var(--bf-amber)"/><text class="bf-t-strong" x="592" y="254.9" text-anchor="middle" style="font-size:12px">0.806</text><line x1="638" x2="638" y1="249.6" y2="288.5" style="stroke:var(--bf-green);stroke-width:3"/><circle cx="638" cy="269.4" r="6" style="fill:var(--bf-green)"/><text class="bf-t-strong" x="638" y="306.5" text-anchor="middle" style="font-size:12px">0.741</text><circle cx="80" cy="358" r="6" style="fill:var(--bf-steel)"/><text class="bf-tm" x="92" y="362" style="font-size:13px">B: codes supplied</text><circle cx="290" cy="358" r="6" style="fill:var(--bf-amber)"/><text class="bf-tm" x="302" y="362" style="font-size:13px">C: scanner supplied</text><circle cx="500" cy="358" r="6" style="fill:var(--bf-green)"/><text class="bf-tm" x="512" y="362" style="font-size:13px">D: both supplied</text></svg>
<figcaption>Measured. Whole-decoder wall time against arm A, with 95% intervals, on Wikidata ×4
on one i9-10900K, 20 rounds. At 1 and 6 workers supplied codes (B) carry the whole gain and a
supplied scanner (C) adds almost nothing; at 10 workers the order reverses, and both together (D)
beat either alone.</figcaption>
</figure>

| Workers | B: codes supplied | C: scanner supplied | D: both | D against the better single arm |
|---|---|---|---|---|
| 1 | 0.830 [0.825, 0.834] | 0.996 [0.991, 1.001] | 0.825 [0.820, 0.831] | 0.994 against B |
| 6 | 0.850 [0.844, 0.855] | 0.982 [0.979, 0.986] | 0.830 [0.827, 0.834] | 0.977 against B |
| **10** | **0.976 [0.936, 1.017]** | **0.806 [0.782, 0.831]** | **0.741 [0.717, 0.766]** | **0.920 [0.901, 0.939] against C** |

At 10 workers all three conditions of the rule hold: D's upper bound is 0.766, D is 0.065 below C,
and D against C is 0.920 with its interval below 1. The verdict is interaction
([the verdict](https://github.com/lmmx/mibz2/pull/145#issuecomment-7144221012)). Peak memory was 3 to 5% lower in the arms that replay the scanner's output.

## 4. Why ten workers reverses the picture

With one worker, the worker is the slow stage, so the scanner always has the next block ready and
removing its work changes nothing, while making emission cheaper shortens the slow stage directly.
With ten workers, the ten together consume blocks faster than one scanner can produce them, so the
workers stand idle at the door. Making their work cheaper then only lengthens their wait, which is
why the supplied codes bought nothing in the first test and nothing distinguishable from zero in
arm B. Once the scanner's work is removed in arm C, the workers become the slow stage again, and
the same supplied codes buy 8% more in arm D.

The earlier null at ten workers was therefore masked rather than absent: the emission gain was
there, hidden behind the scanner. Together the two stages put up to about a quarter of
whole-decoder time on the table at ten workers.

## 5. What it changes

The gain is an oracle bound. Both supplied stages were computed outside the clock, and removing the
scanner also removed serial bit-decoding that any real decoder must still do somewhere, in
parallel or more cheaply. What the test establishes is where the time is and how much of it there
is, and that the two stages must be changed together.

It changes the order of the search
([what else moves together](2026-11-24-what-else-moves-together.md)). At ten workers the first
target is the scanner's serial cost, and only behind it does cheaper emission pay. The question it
puts to the mathematics is a case of enrichment: what does the scanner already see while it
decodes each block that the workers later have to work out again, and could it pass that along
cheaply? Earlier attempts to split the scanner's work across threads failed on scheduling and
memory rather than on the size of the prize, which this test has now measured
([journal §44](https://github.com/lmmx/mibz2/blob/2f03450661a08ebd7f42e8d0d7050fb96ac25418/docs/journal/2026-11-24-algorithm-level.md)).

---

**Sources:**

- [The registration](https://github.com/lmmx/mibz2/blob/5731db1908d58fa4a51d22baf473d2f61ccd670e/docs/journal/2026-11-24-four-arm-registration.md), frozen before the data, with the arms, the rule and the harness.
- [The verdict](https://github.com/lmmx/mibz2/pull/145#issuecomment-7144221012), and the run's bundle, `four-arm/run-20261124T204154Z`, on the [evidence branch](https://github.com/lmmx/mibz2/tree/c930253967e56d1b9eef955adbd3c68fb354eb0f/four-arm).
- [Journal §44](https://github.com/lmmx/mibz2/blob/2f03450661a08ebd7f42e8d0d7050fb96ac25418/docs/journal/2026-11-24-algorithm-level.md), with the interpretation and the caveats.
- [The earlier supplied test](https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060), which arm B reproduces.
