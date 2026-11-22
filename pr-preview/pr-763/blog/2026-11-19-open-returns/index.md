[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-763/blog/index.md>)

# Open Returns: decoding a block from a tenth of its rows

mibz2 0.5.7 decodes Wikidata up to 11% faster on ten cores by keeping a tenth of each block's rows and letting the rest travel together in convoys.

19 November 2026 · 10 min read · [Research result](<#5-what-did-it-buy>)

Shipped in mibz2 [0.5.7](<https://lmmx.github.io/mibz2/pr-preview/pr-763/benchmarks/history/#release-0-5-7>) ([changelog](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#057---2026-11-19>))

**The setting.** bzip2 compresses text in blocks of up to 900,000 bytes. For each block it sorts every rotation of the text and stores only the last column of the sorted rows, the Burrows–Wheeler transform (BWT), plus the row that holds the original text. Decoding means walking a map called **LF**, which sends each row to the row of the rotation one byte earlier. Each step reads one byte, so the walk spells the block backwards, but each step also needs the row the previous step found. That chain of dependent memory reads is most of the cost of decoding. Repeated letters sit together in the last column as *runs*, and mibz2's kernels so far, eager Cells and LazyCells, save work by reasoning about runs instead of single rows ([A new way back from the BWT](<https://lmmx.github.io/mibz2/pr-preview/pr-763/blog/2026-11-10-rauzy-veech-reconstruction/index.md>), [Lazy letters](<https://lmmx.github.io/mibz2/pr-preview/pr-763/blog/2026-11-16-lazy-letters/index.md>)).

**What Open does.** mibz2 0.5.7 adds a third kernel, Open. It keeps only the rows that sit in *short* runs, about a tenth of a Wikidata block, and for each kept row works out its **return**: the next kept row the walk will reach, and the bytes it spells on the way. Decoding then jumps from kept row to kept row, writing whole words at a time. The rows it drops are the ones in long runs. Neighbouring rows of one long run move together under LF, like a convoy, so Open traces one member of each convoy and the rest of it comes for free: on Wikidata, one traced letter spells about sixteen bytes.

**What it bought:**

- **0.5.7 decodes Wikidata 7–11% faster than 0.5.6 from 8 to 20 threads on a ten-core i9, with 23% less peak memory, and Open alone is the cause.** The release's only other decoder change affects 6 threads, which this comparison doesn't run, and a test of Open alone against 0.5.6 found the same gain. Below 8 threads nothing changes: Open runs only on the parallel workers' route.
- **It is used only where it pays.** Open has fixed setup costs, and on text, where runs are short, it is up to 5× slower. So each block is checked first: Open runs only if its run records are between 1/64 and 15% of its rows, the range the existing LazyCells kernel already used. Two integer comparisons, no new tuned constant.
- **The caveats:** one machine (an i9-10900K) and one workload (a pinned Wikidata sample). The first registered test of Open failed on a badly designed guard rule, and that verdict stands; a second test, with its rule fixed in advance, passed.

Measured on the i9-10900K: 0.5.7 against 0.5.6 on the Wikidata sample, the median of 10 paired rounds at each thread count (release-scaling/run-20261119T203443Z). The two releases are level up to 4 threads, where Open is not on the route. From 8 threads up, the shaded gap is Open: 2,557 MB/s against 2,356 at 10 threads.

[Section 1](<#1-how-can-a-decoder-skip-nine-rows-in-ten>) works a small example by hand. [Section 2](<#2-why-do-long-runs-travel-together>) explains the convoys and what they look like in real blocks, and [section 3](<#3-how-are-the-returns-found-without-walking>) how the returns are computed without walking. [Section 4](<#4-why-not-use-open-everywhere>) covers the gate, and [section 5](<#5-what-did-it-buy>) the road from a research idea to the release.

## 1\. How can a decoder skip nine rows in ten?

Take the 13 bytes `bananabandana`. Sort its rotations and read down the last column: `nndnbbaanaaaa`. The text itself is row 6. Its runs are `nn`, `d`, `n`, `bb`, `aa`, `n` and `aaaa`.

On real blocks Open keeps every run of at most 16 rows. In a block this small, that would keep everything, so the toy keeps runs of one row only, plus the start row 6. That keeps rows 2, 3, 6 and 8 and drops the other nine.

The worked example. Kept rows are blue; the runs Open drops are grey. The amber arrows are LF on the long run of four a's: rows 9 to 12 go to rows 2 to 5, still side by side, as LF does to any run (section 2).

For each kept row, follow LF until it lands on another kept row, and write down the letters read on the way. That trip and its word are the row's **return**:

| kept row | its letter | LF goes through | returns to | word |
| ---: | :---: | :--- | ---: | :--- |
| 6 | a | 0 (n), 9 (a) | 2 | `ana` |
| 2 | d | – | 8 | `d` |
| 8 | n | 12 (a), 5 (b), 7 (a), 1 (n), 10 (a) | 3 | `nabana` |
| 3 | n | 11 (a), 4 (b) | 6 | `nab` |

Decoding is now four jumps instead of thirteen steps:

The four returns, chained from the start row. Joined, the words read the block backwards, exactly as a row-by-row walk would. Every byte is still written; what shrinks is the number of dependent jumps and the rows the decoder has to keep.

The words still have to come from somewhere, and in this toy they were found by walking. The point of Open is that on real blocks the words can be built far more cheaply than by walking, because the dropped rows travel in convoys.

## 2\. Why do long runs travel together?

Within one run, every row holds the same letter, and LF sends consecutive rows with the same letter to consecutive rows. So a run is carried by LF as a block: its rows stay side by side until the convoy reaches a place where the runs around it change, and splits. In the toy, the four a-rows 9–12 arrive together at rows 2–5. The earlier post called these groups *bundles*; a convoy of width *b* that travels *h* steps spells *b·h* bytes, but finding its route means tracing only one member, *h* steps.

Long runs make long convoys. Keeping the short runs removes most of the places where convoys would split, and everything else rides through. Medians over the 233 large Wikidata blocks in the sample ([counts](<https://github.com/lmmx/mibz2/blob/0ebbe09f75cdc7c1b26c26972c237a92d07ecec5/docs/journal/2026-11-18-astra-investigation-2.md>)):

Counted, at Open's threshold of 16 rows: medians over 233 large Wikidata blocks and six prose blocks. Wikidata keeps 11.3% of its rows; 88.7% ride in convoys, of which Open traces 5.5% of the block itself. Each traced letter spells about 16 bytes (median 15.9). In prose the convoys are about as wide, but they cover only a third of the block.

That is the whole source of the saving, and also of its limit: a block whose runs are short has little to drop.

## 3\. How are the returns found without walking?

The returns split cleanly into two independent pieces, a result proved in [Cutting the loop open](<https://lmmx.github.io/mibz2/pr-preview/pr-763/blog/2026-11-18-cutting-the-loop-open/index.md>):

- **The kept rows among themselves.** The order in which kept rows hand over to each other depends only on their own letters, a stable sort of the kept part of the column. Open builds it as the run records stream in: each kept row joins a queue for its letter.
- **The convoys' routes.** Where each convoy enters and leaves the dropped region, and the word it spells, are found by a small auxiliary computation on the long runs alone. It needs at most about three records per long run, however the kept rows are wired. The words are stored as a dictionary of shared pieces, not spelled out per row.

On one test block of 899,931 rows, that meant 89,874 kept rows, an auxiliary problem of 7,786 records solved in 11,215 steps, and 7,334 dictionary rules. The decoder then walks the kept rows' returns and copies words forwards.

Less to hold means less memory traffic. On that block, a full decode asked for 6.3 MB of extra heap against 10.4 MB for eager Cells and 15.4 MB for LazyCells, and in a cache simulation it executed 11% fewer instructions with 27% fewer first-level data-cache misses ([measurements](<https://github.com/lmmx/mibz2/blob/0ebbe09f75cdc7c1b26c26972c237a92d07ecec5/docs/journal/2026-11-18-open-return/evidence/>)).

## 4\. Why not use Open everywhere?

Because its setup has to pay for itself. Every block was timed both ways, on one thread ([census](<https://github.com/lmmx/mibz2/blob/0ebbe09f75cdc7c1b26c26972c237a92d07ecec5/docs/journal/2026-11-19-open-admission.md>)):

| corpus | blocks | Open's time ÷ the existing kernel's |
| --- | ---: | ---: |
| Wikidata, admitted | 228 | **0.972** |
| Wikidata, too many runs (small blocks, 2% of the rows) | 155 | 1.005 |
| text | 48 | 5.11 |
| pattern (very few runs) | 88 | 1.55 |
| one byte repeated | 2 | 1.87 |

Open won exactly where LazyCells already admitted blocks, and nowhere else. LazyCells decides with two tests on a block's **record density**, its run records *R* divided by its rows *n*. A record holds a run of up to 127 rows, so *R* is close to the number of runs, and the decoder already has it. Open's admission reuses those two lines unchanged: 1/64 ≤ *R*/*n* \< 0.15, that is, 64·*R* ≥ *n* and 20·*R* \< 3·*n*.

The admission test, on a log axis of record density. Below 1/64 the block is nearly all one letter and LazyCells handles it well; at 15% and above, runs are too short for convoys to pay. The dots are examples: 50,000 records in a 900,000-row block is 5.6%, admitted; 300,000 records is 33%, not.

The test is a cheap predictor, not a guarantee: two blocks with the same density can differ in how their convoys split. The 228 admitted Wikidata blocks hold 98% of the sample's rows, so Open does almost all of Wikidata's work while every other block keeps the kernel it had.

## 5\. What did it buy?

Getting from the idea to a release took seven steps over two days:

From research to release, 18–19 November 2026. The research decoder used unsafe code; the production version has none, and passed Miri, fuzzing and an exhaustive audit of every return on small columns before any timing counted. The first registered test's guard rule compared single estimates with a 2% bound across 39 noisy cells, a rule a change with no effect would fail about 97% of the time. Its verdict stands; the second test, registered separately with its rule fixed in advance, is the one that adopted Open.

The release-on-release run that every mibz2 release gets, on the same i9, agrees with the adoption test:

Measured on the i9-10900K, 10 paired rounds (release-scaling/run-20261119T203443Z): 0.5.6's wall time over 0.5.7's on Wikidata, with 95% intervals. At 1–4 threads the intervals span 1. From 8 threads up, 0.5.7 is 7–11% faster. The release's attribution ledger credits the step to Open's merge alone.

**What comes next.** Open still builds and walks a table of every kept row's return, and that is now where its time goes. A research variant, scalar-cap, measured 2–5% faster than Open at 7–10 workers in the research harness on the same i9. It will need the same block-by-block checks before it can follow Open into a release.
