[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-734/blog/index.md>)

# Sixteen at a stroke: fixed-width writes that stop the processor guessing

Writing sixteen at a stroke and keeping only what it needs spares the processor a guess per run, so mibz2 0.5.8 decodes Wikidata 5% faster on ten cores.

21 November 2026 · 11 min read · [Research result](<#5-what-did-it-buy-on-ten-cores>)

Shipped in mibz2 [0.5.8](<https://lmmx.github.io/mibz2/pr-preview/pr-734/benchmarks/history/#release-0-5-8>) ([changelog](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#058---2026-11-21>))

**What changed.** mibz2 0.5.8 changes how its decoder writes short, variable-length things into memory. It used to write a run of row numbers one at a time, stopping when the run ended. Now it always writes sixteen consecutive numbers in one fixed copy, then moves its write position on by the run's true length. Whatever was written past that point is spare, and the next write covers it. The same trick applies to the decoded text. Each word of up to 16 bytes is copied as a whole 16-byte block, and the output position moves on by the word's real length. Nothing about what is decoded changes. The bytes are identical; only the shape of the writes is new.

**Why that helps.** A loop that runs a different number of times on every call makes the processor guess, each time, where it will stop. A modern processor works ahead of the instruction it is finishing, so it must decide early whether the loop goes round again. When run lengths vary at random between 1 and 16, it often decides wrongly. It then discards the work it did past the real end and starts again. A fixed-width write has no such decision to make. That is our explanation of the gain, reasoned from how processors work and supported by a controlled test in a C harness. It has not been measured separately with the processor's own counters ([section 2](<#2-why-would-writing-more-be-faster>)).

**Where it applies.** The change is inside Open, the decoder's newest way of undoing the Burrows–Wheeler transform (BWT), which [Open Returns](<https://lmmx.github.io/mibz2/pr-preview/pr-734/blog/2026-11-19-open-returns/index.md>) introduced. bzip2 stores each block of up to 900,000 bytes as the last column of the sorted rotations of its text. Equal bytes on neighbouring rows form *runs*. Open keeps the rows of narrow runs, those at most 16 rows wide, which make up about a tenth of a Wikidata block. The rest travel through the decoding in groups and come for free. It files each kept row into one of 256 queues, one per byte value. Later passes wire those queues into a table that says where each kept row leads. A final walk follows the table and writes out each step's word. Filing the kept rows and building that table take half or more of Open's time, as [Research by obligation](<https://lmmx.github.io/mibz2/pr-preview/pr-734/blog/2026-11-20-research-by-obligation/index.md>) recorded. Open runs only when the decoder has many parallel workers. That means 8 workers or more on most machines, and 6 threads or 8 and up on the ten-core i9 we test releases on.

**What it bought:**

- **Open itself got about a fifth faster.** Over the whole Wikidata benchmark file on one thread, its time fell from 433–441 ms to 347–359 ms. Those are four runs of each build on a cloud container.
- **The decoder got 5% faster at ten threads,** mibz2's default on the i9-10900K. Its wall time was 0.947 of the old, with a 95% interval of 0.939 to 0.955, and it used 5% less processor time. The ratio was 0.922 at 6 threads and 0.974 at 8. Peak memory stayed within 3.4%, and every output byte was identical.
- **The scope:** one machine and one corpus, a pinned Wikidata sample. Below 8 workers on other machines nothing changes, because Open doesn't run there. The worker counts the decoder picks per machine are being re-checked for 0.5.8.

The decoder on the ten-core i9: new wall time over old, with 95% intervals

Measured on the i9-10900K, 16 paired rounds at each thread count, decoding four copies of the Wikidata sample end to end (839 MB out). Green is wall time, blue is processor time. Every interval lies below 1, so the new build is faster beyond noise at all three counts. Ten threads, production's default, was the count the registered rule judged.

**Where it came from.** Our research line works through candidate ideas under the [research charter](<https://lmmx.github.io/mibz2/pr-preview/pr-734/blog/2026-11-20-research-by-obligation/index.md>). It found the cost while explaining why a different idea had failed. It measured the fix in a C harness, and a first port to Rust then showed nothing on CI's benchmark. That null turned out to be expected: the benchmark never runs Open. Native timing showed the kernel gain was real, and a registered test on the i9 adopted it. [Section 4](<#4-where-did-the-idea-come-from>) tells the story in order.

[Section 1](<#1-what-does-open-do-with-a-narrow-run>) works the queue fill on a toy column, the old way and the new. [Section 2](<#2-why-would-writing-more-be-faster>) explains why writing more can cost less, and [section 3](<#3-the-other-two-parts-a-counter-and-a-16-byte-copy>) the change's other two parts. [Section 4](<#4-where-did-the-idea-come-from>) traces the idea to the release, [section 5](<#5-what-did-it-buy-on-ten-cores>) gives the i9 result in full, and [section 6](<#6-what-comes-next>) says what comes next.

## 1\. What does Open do with a narrow run?

Take a toy block whose last column has ten rows: `aabaaacabb`. Its byte `a` has three runs, of widths 2, 3 and 1. On real blocks Open keeps every run of at most 16 rows. Every run here is that narrow, so every row is kept, and a row's number among the kept rows is simply its row number.

Open reads the block's runs in order, and each kept row joins the queue for its byte. Reading the queues back in byte order gives the kept rows sorted by byte, with ties in row order. That sorted order is what tells each kept row where the decoding goes next ([Cutting the loop open](<https://lmmx.github.io/mibz2/pr-preview/pr-734/blog/2026-11-18-cutting-the-loop-open/index.md>) works it through). So the queue for `a` must end up holding the rows of its three runs, in order.

The old fill wrote each run's rows with a loop, one row per turn. After each row it asked whether the run had another:

The toy column, and the old fill of a's queue

Conceptual: the toy column and the old fill, one row per turn of the loop. Each run ends with a "no", and the "no" comes after a different number of rows each time. In code this was one line, `queue.extend(m..m + w)`, a loop whose trip count is the run's width w.

The new fill makes no per-row decision. A narrow run has at most 16 rows. So the fill writes 16 consecutive numbers, starting from the run's first row, all at once at the queue's current end. Then it moves the end on by the run's real width. The numbers written past the new end are spare. The next run starts writing exactly there, so it covers them:

The same queue, filled sixteen at a stroke

Conceptual: the new fill on the same three runs. The first run writes 0 to 15 into slots 0 to 15 and keeps two; the second writes 3 to 18 from slot 2 and keeps three; the third writes 7 to 22 from slot 5 and keeps one. The queue's contents up to its end, 0, 1, 3, 4, 5, 7, are what the old fill gave. The stamp analogy holds for the shape of the work; it stops holding on cost, since the spare numbers are real writes to the cache, 64 bytes per run where the old loop wrote 6 to 9 on average.

Two details make this safe. The queue's buffer is grown before the write whenever fewer than 16 slots remain past its end, so the fixed write never runs off the buffer. And the spare numbers are never read, because everything downstream reads a queue only up to its end. In Rust the fill is a few lines ([`push_rows`](<https://github.com/lmmx/mibz2/blob/70b18ca4411fa10c9f29b80821c835358dde2f9e/src/open.rs#L233>)):

```rust
let iota: [u32; 16] = std::array::from_fn(|i| m.wrapping_add(i as u32));
queue[at..at + 16].copy_from_slice(&iota);
// the caller then moves the end on: thin[a] += w
```

The 16 numbers are 4 bytes each, so each run is a fixed 64-byte copy with no loop over the width. It is safe Rust, with every index checked.

## 2\. Why would writing more be faster?

The new fill writes seven to ten times as many numbers as it keeps. That it is faster says the old fill's cost was never the writing.

A processor doesn't run one instruction at a time. It fetches and starts instructions well ahead of the one it is finishing. To get past a decision such as "go round the loop again?", it guesses the answer and carries on. A component called the branch predictor makes those guesses from the pattern of past answers. When a guess is right, the work done ahead counts. When it is wrong, the processor throws away everything it started past the decision and begins again from the right place. That restart costs many cycles, far more than writing a few numbers.

The old fill's loop ended at a point set by the run's width, and the widths of consecutive narrow runs follow no pattern. The research line studied six Wikidata blocks. Each had 48,000 to 126,000 narrow runs, averaging 1.6 to 2.2 rows. Most loops therefore ran once, twice or three times, in no predictable order:

Why the loop's end is hard to guess, and why the fixed write has nothing to guess

Conceptual, with made-up widths in the real range. The upper lane is the old fill: every amber "?" is a run's end, which the processor must guess before it knows. The lower lane is the new fill: one fixed write per run, whatever its width. The clerk analogy stops holding in one respect: the processor's guesses are often right, and the cost is only in the wrong ones.

The new fill moves the width out of the decisions and into arithmetic. The width still sets where the queue's end goes, but that is an addition, and nothing branches on it. The processor can run ahead through run after run with nothing to mispredict.

**How we know, and what is not yet known.** The research line tested this in its C harness before anything was ported. It built the same queues two ways on three of the six blocks. With a plain loop over each run's rows, a rival design tied the shipped one. Giving each the fixed 16-number write cut its construction time by 23–28%. The rows written were the same, so the cost was not the queues' memory traffic; the loop is what changed. It is still an inference. No one has yet counted the mispredicted branches before and after with the processor's performance counters. So the mechanism above is our reasoned account, not a measurement. The speed-up itself is measured.

## 3\. The other two parts: a counter and a 16-byte copy

The release changes two more things in Open, both following from the same idea of keeping decisions and bookkeeping off the per-run path.

**The queue's end kept in a counter.** A growable array in Rust, a `Vec`, records its own length in memory. Every append reads that length, checks the buffer has room, writes, and stores the new length back. The first port did that twice per run, once to append the 16 numbers and once to cut the queue back to its true width. Open already counts each byte's kept rows as it goes, for its own bookkeeping. So the new queues use that count as their end, and the buffer itself is kept at its full length. A run is then one bounds check and the 64-byte copy, and no length is written back. The rewrite also spelled out the per-run step so that its two common cases compile inline, without a function call.

**The 16-byte emit.** Open's last pass walks the table of kept rows and appends each step's word. A word is either a single byte or a longer piece from a dictionary. The old walk asked at every step which kind it had, then copied that many bytes. The new walk keeps every word as a 16-byte slot, padded with blanks, beside its true length. A step copies the whole slot to the write position and moves the position on by the length. The next copy covers the blanks. On the six blocks, 92.5–94.4% of steps had words of 16 bytes or fewer. The rest, longer words, take a separate path that copies them in full. Here it is on the four words from Open Returns' toy, whose walk spelled `ana`, `d`, `nabana` and `nab`:

The walk's output, copied in 16-byte slots

Conceptual: the 16-byte emit on the toy from Open Returns. Each copy writes all 16 bytes of a slot; only the first "length" of them are kept, and the next copy starts at the amber line. The buffer is 16 bytes longer than the block so that the last copy fits, and it is cut back to the block's length at the end ([`emit`](<https://github.com/lmmx/mibz2/blob/70b18ca4411fa10c9f29b80821c835358dde2f9e/src/open.rs#L653>)).

**Which part did the work.** The two rounds of porting let us split the gain. The fixed fill and the 16-byte emit came first, and the counter queues second. On one thread, over the whole corpus, the kernel's time moved like this:

Open's time over the whole Wikidata corpus, one thread

Measured on a shared 4-CPU cloud container (Xeon, 2.10 GHz), one pinned thread, each block's best of 5 summed over the corpus's 431 blocks, four interleaved runs per build. Indicative: CI and the i9, not this container, are where decisions are made.

The emit is the smallest of the three. In the C harness, the 16-byte copy on its own barely changed the walk's time, at 0.96–1.0 of the old. So in the harness, most of the first round's gain came from the fill. The native timings don't separate the fill from the emit, because the two were ported together.

## 4\. Where did the idea come from?

It came from a failure. [Research by obligation](<https://lmmx.github.io/mibz2/pr-preview/pr-734/blog/2026-11-20-research-by-obligation/index.md>) described how we now judge ideas by the work they remove. It listed "building the map of retained rows" as half or more of Open's time. One attempt on it, building the map backwards ([\#659](<https://github.com/lmmx/mibz2/pull/659>)), had lost natively: it took 1.10–1.15 times as long. The research line's second cycle asked why.

Its answer was the ablation in section 2. On that account, the rival design and the shipped queues had both been paying for the same mispredicted loop exits. A fixed 16-number write removed them from either. The research line then built that write and a 16-byte copy for the walk into a test arm. It was the single-walk control for a larger candidate that ran eight walks at once. On its own, that arm took 0.84–0.86 of Open's time to build and walk each of the six blocks, over 25 interleaved rounds ([the research journal](<https://github.com/lmmx/mibz2/blob/8b54b0243c2448b647e53c81e95b6e999ae54ef0/docs/journal/2026-11-20-carbon-cycle2-bvsmc.md>)). The records call that arm U16Q, and the counter queues added later Vec16.

The road from there to the release had a dead end in the middle:

From research idea to release, 20–21 November 2026

The chain of evidence. Amber marks the step that showed nothing; it was explained two steps later, not overruled. Each figure is from the step's own record: the research journal, CI's paired bench on \#664, the calibration and stage-attribution journals, \#666's stage table and the i9 evidence bundle.

**The first port showed nothing.** The safe Rust port ([\#664](<https://github.com/lmmx/mibz2/pull/664>)) passed every test, including 200,000 random walks checked byte for byte against the old emit. CI's paired benchmark then found no change: −0.1% (−0.4%, +0.1%) at one thread, and nothing beyond noise at two or four. The first reading was that the C harness had flattered the idea.

**The harness was fair.** A calibration timed the C harness's copy of Open against the real Rust kernel on the same six blocks. They agreed within a few percent on five blocks. The ported kernel ran at 0.86–0.91 of the old Rust kernel's time, a 9–14% gain ([the calibration](<https://github.com/lmmx/mibz2/blob/70b18ca4411fa10c9f29b80821c835358dde2f9e/docs/journal/2026-11-20-open-u16q.md>)). So the gain was real in the kernel, and the question became why it didn't reach the benchmark.

**The benchmark was blind to it.** A stage-by-stage timing of the decoder answered that ([stage attribution](<https://github.com/lmmx/mibz2/blob/70b18ca4411fa10c9f29b80821c835358dde2f9e/docs/journal/2026-11-20-open-stage-attribution.md>)). CI's benchmark runs 1, 2 and 4 threads. Below 8 workers the decoder uses not Open but Cells, the kernel that is faster for one worker. Open's smaller memory footprint pays only when many workers share the processor's cache. So CI had timed code the change never touched, and its null was the expected result. Over the whole corpus, the same timing found the first port cut Open by 11%, with every other stage level.

**The second round.** A later cycle of the research line proposed the counter queues. With them came a fused front end, feeding runs straight into the queue fill without the list of runs in between. Ported natively ([\#666](<https://github.com/lmmx/mibz2/pull/666>)), the counter queues paid and the fusion didn't. The fused route took 408–433 ms, against 384–398 ms for the two stages done separately. So the fusion was dropped, and the release carries the fill, the counter queues and the emit ([\#668](<https://github.com/lmmx/mibz2/pull/668>)).

## 5\. What did it buy on ten cores?

Because CI can't see Open, the decision went to the maintainer's i9-10900K, a ten-core desktop processor with twenty hardware threads. The test ([\#669](<https://github.com/lmmx/mibz2/pull/669>)) compared the release candidate against master, changing `src/open.rs` and its tests alone, on the whole Wikidata sample decoded four times end to end. It ran 16 rounds at each of 6, 8 and 10 threads, alternating the two builds. At those counts the i9 runs Open, with 6 workers at 6 threads and 8 at 8. From 10 threads the decoder trims its workers to 7, to fit the cache.

The rule was fixed before any data. To be adopted, the change had to make the 10-thread wall time faster beyond noise, with 6 and 8 threads not slower. Median peak memory could grow by at most 10% at each count. The outcome, beside that rule:

| threads | role | wall time, old → new (median, s) | wall ratio \[95%\] | processor time ratio \[95%\] | peak memory ratio |
| ---: | --- | ---: | ---: | ---: | ---: |
| 6 | must not be slower | 0.327 → 0.301 | 0.922 \[0.915, 0.929\] | 0.935 \[0.932, 0.938\] | 1.034 |
| 8 | must not be slower | 0.294 → 0.285 | 0.974 \[0.961, 0.986\] | 0.966 \[0.954, 0.978\] | 1.002 |
| 10 | must be faster | 0.299 → 0.284 | 0.947 \[0.939, 0.955\] | 0.952 \[0.943, 0.961\] | 1.015 |

All three counts were faster beyond noise, memory stayed within 3.4%, and the output was byte-identical in every round. The verdict was adopt ([evidence bundle](<https://github.com/lmmx/mibz2/tree/2d6e5503c2c49db8e79d889e084da7569291ef5d/open-vec16>)). The intervals describe variation across the 16 rounds, on one machine and one corpus.

**Why a fifth off the kernel is 5% off the decode.** Open is about three quarters of a worker's computing on its route. But a decode at ten threads also waits on the single thread that reads the compressed stream ahead of the workers, and this change doesn't touch it. A simple model in the stage attribution puts the balance between that reader and the workers near seven workers. That is where the i9 runs from ten threads. So at ten threads only part of the kernel's gain can show in wall time. The model assumes the container's stage ratios hold on the i9. At 6 threads, with fewer workers sharing the work, more of it shows: 7.8%. The records don't explain why 8 threads gained least, at 2.6%.

**What it costs.** Peak memory rose by at most 3.4%, at 6 threads. The queue buffers stay at full length between blocks, and each carries 16 spare slots, so a little more memory stays allocated. No part of the change adds unsafe code.

## 6\. What comes next

mibz2 0.5.8 ships the change. Its worker-count recalibration, the check of how many workers each machine should run, and its release-on-release scaling run are queued on the i9.

The chain, in short, runs as follows. Open spent much of its time filing narrow runs into queues, and the filing loop's end was unpredictable. Writing a fixed sixteen and moving on by the true width removed the guess. The same shape was applied to the queue's length and to the walk's output. A benchmark that never ran Open nearly hid the gain, until a stage timing showed where to look. The step not yet taken is measuring the mispredictions themselves, with the processor's counters, so that the explanation in section 2 stands on its own data.

The general lesson is about short loops whose length the data decides. Where such a loop has a small known maximum, it can often become one fixed-width operation and a variable advance. The buffer needs room for the overrun, and nothing may read past the true end. The decoder has other loops whose length the data decides, and we have not yet studied them this way. One candidate is the last stage, which expands bzip2's run-length codes into 4 to 255 copies of a byte. Its maximum is larger, so a fixed write there would need a different width, or a split into short and long runs.

---

**Research records:** [the research line's cycle 2 journal](<https://github.com/lmmx/mibz2/blob/8b54b0243c2448b647e53c81e95b6e999ae54ef0/docs/journal/2026-11-20-carbon-cycle2-bvsmc.md>) (the ablation and the C harness); [the first port and its calibration](<https://github.com/lmmx/mibz2/blob/70b18ca4411fa10c9f29b80821c835358dde2f9e/docs/journal/2026-11-20-open-u16q.md>) (\#664); [the stage attribution](<https://github.com/lmmx/mibz2/blob/70b18ca4411fa10c9f29b80821c835358dde2f9e/docs/journal/2026-11-20-open-stage-attribution.md>); [the counter queues and the fused front](<https://github.com/lmmx/mibz2/blob/ef248f786c72c55d0ff08bc0f0925a0dd6087e92/docs/journal/2026-11-20-open-lrb-vec16.md>) (\#666); [the i9 test's registration](<https://github.com/lmmx/mibz2/blob/df4accbbe72b7f06879115fe44656b8cd4135154/docs/journal/2026-11-21-open-vec16-registration.md>) (\#669) and [its evidence](<https://github.com/lmmx/mibz2/tree/2d6e5503c2c49db8e79d889e084da7569291ef5d/open-vec16>); [the code as released](<https://github.com/lmmx/mibz2/blob/70b18ca4411fa10c9f29b80821c835358dde2f9e/src/open.rs>) (\#668).
