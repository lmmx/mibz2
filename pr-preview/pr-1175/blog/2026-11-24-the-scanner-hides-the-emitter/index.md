[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1175/blog/index.md>)

# Masked bottlenecks in a two-stage decoder

Supplying both the serial scanner's output and the emission codes cuts whole-decoder time by 26% at ten workers, more than either stage supplied alone.

24 November 2026 · 7 min read · [Research result](<#5-what-it-changes>)

Our first supplied test, handing mibz2's emitter its decoding order for free made the whole decoder 16% faster on one thread and barely faster on ten ([the supplied test](<https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060>)), which suggested that at ten workers there was little to gain from cheaper emission. A four-arm test now shows that suggestion was wrong. At ten workers the decoder waits on its serial scanner, the single thread that reads the compressed bits ahead of the workers, and the scanner hid the emitter's gain. Supplying the scanner's output alone cuts whole-decoder time by 19%; supplying it together with the emission codes cuts it by 26%, and the codes, which bought 2% on their own, then buy a further 8%. These are oracle bounds, measured with the supplied work done outside the clock, so they say how much time a better algorithm could free, not that we have one.

**Takeaways**

- At 1 and 6 workers the workers are the bottleneck: supplied codes save 15 to 17%, and a supplied scanner saves 2% at most.
- At 10 workers the scanner is the bottleneck: a supplied scanner saves 19%, and supplied codes alone save nothing distinguishable from zero.
- Together they save 26% at 10 workers, 8% more than the scanner alone, which by the rule fixed before the data is an interaction.
- The gain is there to be had only by an algorithm that changes both stages at once, and the scanner's serial cost comes first.

## 1\. The decoder as a small factory

mibz2 decodes a bzip2 file in two kinds of work. One thread, the scanner, reads the compressed file in order and decodes each block's Huffman codes into symbols ([the Huffman stage](<https://lmmx.github.io/mibz2/pr-preview/pr-1175/blog/2026-11-05-huffman-bit-buffer/index.md>)). The workers, running in parallel, take one block each: they undo the move-to-front and run-length coding, invert the Burrows–Wheeler transform and emit the text ([why the scanner is serial](<https://lmmx.github.io/mibz2/pr-preview/pr-1175/blog/2026-11-07-why-mibz2-stops-scaling/index.md>)). The scanner is the factory's doorman, unpacking each parcel at the door, and the workers are its assemblers.

One doorman, many assemblers: where the decoder waits

Schematic. With one worker, blocks queue for the worker, so making the worker's job cheaper (arm B) saves time and making the scanner's cheaper (arm C) does not. With ten workers, the workers wait for the scanner, so only a cheaper scanner helps until the door is cleared, and then a cheaper worker job helps again.

## 2\. The test: four arms, two stages supplied

The question was whether removing two costs together frees time that removing either alone does not. The two costs are the scanner's Huffman decoding and the code ordering in [Open](<https://lmmx.github.io/mibz2/pr-preview/pr-1175/blog/2026-11-19-open-returns/index.md>)'s emission, the stage the earlier supplied test had halved. One binary carries four arms ([the registration](<https://github.com/lmmx/mibz2/blob/5731db1908d58fa4a51d22baf473d2f61ccd670e/docs/journal/2026-11-24-four-arm-registration.md>)):

| Arm | Scanner | Emission |
| --- | --- | --- |
| **A** | as shipped | as shipped |
| **B** | as shipped | its codes supplied, in output order |
| **C** | its output supplied: each block's recorded symbols, replayed through the production job queue | as shipped |
| **D** | supplied | supplied |

In arms C and D the scanner thread reads no bit and decodes no Huffman code; it copies each block's recorded symbols into the same buffer the scanner would have filled and sends the block on, so everything downstream runs as in production and stays on the clock. The run decoded our Wikidata benchmark four times over, at 1, 6 and 10 workers, for 20 rounds with the arms in a balanced order, on one i9-10900K. Every round's output matched the reference: 838,800,784 bytes, with no mismatch.

The verdict was fixed before any data. At 10 workers the result is an **interaction** if D's upper 95% bound is at most 0.90 of A, D sits more than 0.03 below the better single arm, and D beats that arm outright; it is additive if only the first holds, and there is no potential if D's lower bound is above 0.95. The run is also invalid unless arm B reproduces the earlier supplied test, which it does: 0.976 against 0.988, with overlapping intervals.

## 3\. Results

Which supplied stage frees time, by worker count

Measured. Whole-decoder wall time against arm A, with 95% intervals, on Wikidata ×4 on one i9-10900K, 20 rounds. At 1 and 6 workers supplied codes (B) carry the whole gain and a supplied scanner (C) adds almost nothing; at 10 workers the order reverses, and both together (D) beat either alone.

| Workers | B: codes supplied | C: scanner supplied | D: both | D against the better single arm |
| --- | --- | --- | --- | --- |
| 1 | 0.830 \[0.825, 0.834\] | 0.996 \[0.991, 1.001\] | 0.825 \[0.820, 0.831\] | 0.994 against B |
| 6 | 0.850 \[0.844, 0.855\] | 0.982 \[0.979, 0.986\] | 0.830 \[0.827, 0.834\] | 0.977 against B |
| **10** | **0.976 \[0.936, 1.017\]** | **0.806 \[0.782, 0.831\]** | **0.741 \[0.717, 0.766\]** | **0.920 \[0.901, 0.939\] against C** |

At 10 workers all three conditions of the rule hold: D's upper bound is 0.766, D is 0.065 below C, and D against C is 0.920 with its interval below 1. The verdict is interaction ([the verdict](<https://github.com/lmmx/mibz2/pull/145#issuecomment-7144221012>)). Peak memory was 3 to 5% lower in the arms that replay the scanner's output.

## 4\. Why ten workers reverses the picture

With one worker, the worker is the slow stage, so the scanner always has the next block ready and removing its work changes nothing, while making emission cheaper shortens the slow stage directly. With ten workers, the ten together consume blocks faster than one scanner can produce them, so the workers stand idle at the door. Making their work cheaper then only lengthens their wait, which is why the supplied codes bought nothing in the first test and nothing distinguishable from zero in arm B. Once the scanner's work is removed in arm C, the workers become the slow stage again, and the same supplied codes buy 8% more in arm D.

The earlier null at ten workers was therefore masked rather than absent: the emission gain was there, hidden behind the scanner. Together the two stages put up to about a quarter of whole-decoder time on the table at ten workers.

## 5\. What it changes

The gain is an oracle bound. Both supplied stages were computed outside the clock, and removing the scanner also removed serial bit-decoding that any real decoder must still do somewhere, in parallel or more cheaply. What the test establishes is where the time is and how much of it there is, and that the two stages must be changed together.

It changes the order of the search ([what else moves together](<https://lmmx.github.io/mibz2/pr-preview/pr-1175/blog/2026-11-24-what-else-moves-together/index.md>)). At ten workers the first target is the scanner's serial cost, and only behind it does cheaper emission pay. The question it puts to the mathematics is a case of enrichment: what does the scanner already see while it decodes each block that the workers later have to work out again, and could it pass that along cheaply? Earlier attempts to split the scanner's work across threads failed on scheduling and memory rather than on the size of the prize, which this test has now measured ([journal §44](<https://github.com/lmmx/mibz2/blob/2f03450661a08ebd7f42e8d0d7050fb96ac25418/docs/journal/2026-11-24-algorithm-level.md>)).

---

**Sources:**

- [The registration](<https://github.com/lmmx/mibz2/blob/5731db1908d58fa4a51d22baf473d2f61ccd670e/docs/journal/2026-11-24-four-arm-registration.md>), frozen before the data, with the arms, the rule and the harness.
- [The verdict](<https://github.com/lmmx/mibz2/pull/145#issuecomment-7144221012>), and the run's bundle, `four-arm/run-20261124T204154Z`, on the [evidence branch](<https://github.com/lmmx/mibz2/tree/c930253967e56d1b9eef955adbd3c68fb354eb0f/four-arm>).
- [Journal §44](<https://github.com/lmmx/mibz2/blob/2f03450661a08ebd7f42e8d0d7050fb96ac25418/docs/journal/2026-11-24-algorithm-level.md>), with the interpretation and the caveats.
- [The earlier supplied test](<https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060>), which arm B reproduces.
