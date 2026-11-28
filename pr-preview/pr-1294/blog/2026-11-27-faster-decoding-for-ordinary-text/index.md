[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1294/blog/index.md>)

# Faster decoding for ordinary text, not just Wikidata

Our fastest kernel passes over five in six blocks of ordinary text, so mibz2 now foresees its refusals and gives the general-purpose path a speed-up of its own.

27 November 2026 · 12 min read · [Research synthesis](<#5-what-this-changes>)

mibz2 is a bzip2 decoder, written in Rust, that decodes a file's blocks in parallel. Each bzip2 block holds up to 900,000 bytes, stored in the order produced by the Burrows–Wheeler transform (BWT), and most of the work of decoding a block is undoing that transform. The classical way back is a walk through the sorted rows of the block, one row at a time, each step a lookup that depends on the one before; [The mathematical structure of inverse-BWT navigation](<https://lmmx.github.io/mibz2/pr-preview/pr-1294/blog/2026-11-08-inverse-bwt-navigation/index.md>) works it through. This month mibz2 found a faster route. Adjacent rows that hold the same byte, a *run*, move together, so a kernel can follow one row of each run and let the rest come for free ([A new way back from the BWT](<https://lmmx.github.io/mibz2/pr-preview/pr-1294/blog/2026-11-10-rauzy-veech-reconstruction/index.md>)). The kernel built on that idea, published as [Open](<https://lmmx.github.io/mibz2/pr-preview/pr-1294/blog/2026-11-19-open-returns/index.md>), keeps only the rows that sit in short runs and jumps between them, writing whole words at a time. Below we call it the fast kernel.

The fast kernel was designed and tuned on one corpus, an 11.9 MB extract of a [Wikidata](<https://www.wikidata.org/>) dump that decompresses to 210 MB of JSON. There it takes all 233 full-size blocks, and in our single-thread timings it inverts them in under half the time of the older kernel it replaced. That older kernel, the general-purpose kernel, builds a table with one entry per byte of the block and walks it in 512 segments, 32 at a time ([32 lanes, 8.6 chains](<https://lmmx.github.io/mibz2/pr-preview/pr-1294/blog/2026-10-31-room-walk-overlap/index.md>)). It can decode any block, and the decoder falls back on it whenever the fast kernel declines one.

This post asks what happens on everything else: prose, source code, JSON, executables. The answer is that the fast kernel mostly does not apply. Across seven ordinary corpora it took only 17 of 107 full blocks, about 16%, and the general-purpose kernel did the rest. Worse, on the dense blocks typical of prose and source code, the fast kernel ran its first pass up to a limit before giving up, so those blocks paid for both kernels. We found that every such refusal can be predicted, in one cheap pass, from a list the decoder already holds. We then turned to the kernel that ordinary text actually uses, and made it faster. Both changes are now in mibz2's default decoder on its main branch.

**Takeaways**

- **The fast kernel is a Wikidata specialist.** It takes all of Wikidata's full blocks, but only 16% of those in seven other corpora; the other 84% go to the general-purpose kernel.
- **On dense blocks, it wasted work before declining.** All 25 dense blocks it met outside Wikidata were refused only after its first pass had reached its limit. On a single thread, that added up to 48% to their inversion time and made a whole prose file 1.09 to 1.32 times slower to decode than with the general-purpose kernel alone.
- **One pass over the block's runs foresees every refusal.** Checking the limit before the fast kernel starts brings those blocks down to the general-purpose kernel's own time. On the project's cloud benchmark, four-thread decoding of a text corpus became 3.7% faster, and Wikidata was unchanged.
- **The general-purpose kernel's hidden cost is building its table.** Three changes that need no extra memory cut its inversion time by 15 to 16% on prose and source code in a controlled probe. In the shipped decoder, the cloud benchmark measured text decoding 2.7% faster on one thread and 4.3% faster on two.

## 1\. Which blocks can the fast kernel take?

The fast kernel reads a block as runs rather than as bytes. While the decoder undoes bzip2's move-to-front stage, it can note at almost no cost where each run of repeated bytes begins and how long it is. [Reading the BWT's runs off bzip2's move-to-front codes](<https://lmmx.github.io/mibz2/pr-preview/pr-1294/blog/2026-11-08-bwt-runs-from-mtf-codes/index.md>) shows why the codes already contain that list, which we call the block's *run records*. A block of long runs has few records for its length. The ratio of records to rows, written \\(r/n\\), therefore measures how dense a block is with short runs.

**Run records** are pairs of a byte and a length. The six bytes `nnbaaa` make three of them: (`n`, 2), (`b`, 1) and (`a`, 3). A Wikidata block of 900,000 bytes, whose \\(r/n\\) sits near 0.068, has about 61,000.

The fast kernel's advantage comes from long runs, which it can skip, so it accepts only blocks of low density. Between 1/64 and 0.15 it takes a block outright. Between 0.15 and 0.28, the *dense band*, it must also check how many rows it would have to handle one by one: those lying in runs of at most 16. It takes the block only if that count stays within \\(\\min(256{,}000,\\ 0.3n)\\), a limit set when [dense blocks were first admitted](<https://lmmx.github.io/mibz2/pr-preview/pr-1294/blog/2026-11-21-a-memory-gain-is-a-gain/index.md>) so as to take Wikidata's five. Blocks outside both ranges go to the general-purpose kernel. So do blocks of nearly random bytes, for which the decoder never forms run records at all, since they would cost more than they save; we call that the *column route*.

To see how often each case occurs, we built seven corpora from files any Linux machine holds: prose from documentation, Python's standard library, Rust source from the cargo registry, JSON, ELF executables, a DNA-like stream of four letters, and random bytes. We compressed each with `bzip2 -9` and logged the route of every full block, meaning every block of at least 100,000 bytes. Where each corpus's full blocks went:

Where the full blocks go: Wikidata against seven ordinary corpora

Measured: the route each full block took, from the route census (the number after each name is its count of full blocks). "Refused after a wasted first pass" are the dense-band blocks; "run route, sent to fallback" are blocks outside the fast kernel's range of densities; the column route never forms run records.

On Wikidata the fast kernel takes everything, but elsewhere it takes 17 of 107 blocks. The column route takes 55, nearly all from the executables, the DNA-like data and the random bytes. The run route sends another 35 to the general-purpose kernel, and 25 of those are dense-band blocks from prose, source code, JSON and executables. Where the fast kernel does run, it is as good off Wikidata as on it. It inverts JSON blocks in 0.38 of the general-purpose kernel's time and Rust source in 0.76, against 0.48 on Wikidata itself. The kernel is not weak on ordinary text; it is simply rarely eligible.

## 2\. What does a refusal cost, and can it be foreseen?

All 25 dense-band blocks outside Wikidata were refused, and every one was refused late. The fast kernel learns how many rows it must keep by running its first pass, and stops only when the count crosses the limit. By then the pass has done most of its work, and the general-purpose kernel starts again from nothing. We timed each refused block twice on one thread, once through the decoder as shipped and once through the general-purpose kernel directly:

| corpus | refused blocks | as shipped, ms | general-purpose alone, ms | ratio |
| --- | ---: | ---: | ---: | ---: |
| prose | 7 | 21.0 | 17.1 | 1.23 |
| Rust source | 8 | 28.0 | 20.6 | 1.36 |
| JSON | 4 | 17.6 | 11.8 | 1.48 |
| ELF executables | 4 | 16.4 | 14.5 | 1.13 |
| Python source | 2 | 7.0 | 6.9 | 1.02 |

Together the refusals added about 19 ms over the 25 blocks, roughly 0.8 ms a block. Prose suffered most, because nearly all its full blocks are dense: the whole file decoded 1.09 to 1.32 times slower than it would have with the general-purpose kernel alone. These timings come from a virtual machine with four virtual processors. They are good for ratios rather than absolute speeds, and the whole-file range is wide because the machine is noisy. The direction, though, is not in doubt. On ordinary prose, the decoder's own specialised kernel was making it slower.

The remedy follows from what the first pass counts. The rows it keeps are those in maximal runs of at most 16 rows, plus the one row that marks where the text begins. That count can be read off the run records directly. One merges adjacent records of the same byte, since the format splits long runs, and adds up the lengths of the short ones. It is a single pass over a list several times shorter than the block, made before the fast kernel allocates or computes anything.

The two populations separate cleanly. The least of the refused blocks keeps 257,495 rows, against a limit of 256,000, and the prose blocks keep between 311,318 and 380,411. Wikidata's admitted dense blocks, by contrast, keep between 199,189 and 231,600. A second limit, on a structure the kernel calls thick strips, never came close to binding.

The decoder now makes that count for dense-band blocks before the fast kernel starts, and refuses on the spot any block that would exceed the limit. We measured the effect over seven interleaved repeats, on corpora rebuilt on a fresh machine, which held 24 dense refusals. The refused blocks' inversion time fell to the general-purpose kernel's own:

Dense blocks the fast kernel refuses: before and after counting first

Measured on a virtual machine, one thread, seven interleaved repeats. Each point is the summed inversion time of a corpus's refused blocks as the decoder runs them, over the general-purpose kernel's time on the same blocks (block counts in brackets). Values a little below 1 are within the machine's noise.

Taken block by block, the median ratio fell from 1.536 to 1.027. Where the check passes, it costs almost nothing. On Wikidata's five dense blocks, which are still admitted, it adds about 0.27 ms in all, some 0.03% of a single-thread decode, and all 233 full blocks still take the fast kernel.

The project's cloud benchmark then compared the decoder before and after the change, over ten paired rounds on a four-core runner. Its text corpus is built from mibz2's own source code and documentation. Four-thread decoding of it was 3.7% faster, with a 95% interval of 0.9% to 6.4%. At one and two threads the benchmark detected no change on that corpus, and it detected none on Wikidata at any thread count.

## 3\. Where does the general-purpose kernel spend its time?

With the wasted pass gone, the general-purpose kernel does nearly all the work on ordinary text, yet its costs on such text had never been dissected. A second census, over six corpora of 82 MB in all, found that it inverted 78 of their 86 full blocks: 48 from run records and 30 from the column route. On text, prose, source code and JSON, undoing the BWT took 52 to 66% of a worker's time. Within that, the first step, building the table that the walk follows, took 35 to 46%.

The table holds, for each row, its byte and the row the walk visits next. Three changes, none of which needs extra memory, attacked the cost of building and using it.

**Building in order.** On the column route the kernel built a table that walks forwards. That meant reading the block's bytes and scattering each entry into one of 256 places in the table, a random write per byte. The *LF* table, which sends each row to the row one byte earlier in the text, can instead be built in a single sequential pass, with a shortcut for eight identical bytes in a row. On the 30 column-route blocks this change alone cut the kernel's processor time by 23.4%.

**Reversing with vector instructions.** Walking LF spells each segment of the block backwards, so every segment must be reversed as it is copied into place. The existing loop reversed a byte at a time, and the compiler did not vectorise it. Reversing 32 bytes at a time with AVX2 shuffles, with an eight-byte fallback for other processors, cut that step by 45% on the prose, text and source-code blocks.

**Trusting what is already known.** On the run route, the table is filled from the run records. The filling loop checked, on every record, that it stayed inside the block and inside its byte's share of the table: about 225,000 checks a block on the text corpus. Those facts are already established by the code that produced the records. The records therefore now carry a flag, set only by that code and cleared by anything that changes them, and a flagged list is filled without the checks. With all three changes, the run-route blocks took 13.5% less processor time.

**The flag** is a memory-safety argument, not a shortcut taken on trust. The new unchecked loops and the vector code were reviewed for soundness and run under [Miri](<https://github.com/rust-lang/miri>), Rust's interpreter for detecting undefined behaviour, before they were merged. Any list built or altered elsewhere still takes the checked path.

We timed the kernel's own processor time, so that the host's interruptions did not count, over seven interleaved repeats on a virtual machine with two virtual processors. Together the three changes cut inversion time on the 50 prose, text and source-code blocks by 15.4% in a build for any x86-64 processor, and by 16.1% in one tuned to the machine. On all 78 blocks the cuts were 18.4% and 18.5%. Every corpus improved by at least 10.5%, the executables most, by about a quarter, and the output matched the existing kernel byte for byte on every block.

A classic idea from the literature shows why construction, not the walk, was the cost to attack. [Kärkkäinen, Kempa and Puglisi](<https://researchportal.helsinki.fi/en/publications/slashing-the-time-for-bwt-inversion/>) speed up BWT inversion with a table that advances two bytes per lookup, halving the walk's chain of dependent reads. We tried it. The walk became 15 to 16% faster, but building the larger table took about four times as long. The inversion as a whole ran more than twice as slow, with 3.6 MiB more memory per worker. Where building the table is already a third or more of the cost, a richer table is the wrong trade.

## 4\. How much of that reached the decoder?

The probe measured the kernel in isolation. The changes were then built into the decoder, and the cloud benchmark compared it with its predecessor over ten paired rounds, on a four-core runner with two physical cores. Every corpus gave the same output from both builds, and peak memory showed no significant rise. On the text corpus the decoder was 2.7% faster on one thread, with a 95% interval of 0.1% to 5.2%, and 4.3% faster on two, with one of 0.6% to 7.8%. At four threads its 2.9% gain was within noise. The random-bytes corpus, whose blocks take the column route as random bytes did in our census, was 5.3% faster on one thread and 6.2% on four.

That is less than the probe implied. Since undoing the BWT is about two-thirds of a worker's time on the text corpus, the probe projected a gain of 8 to 10% there, and a quick, informal check on a development machine suggested 7 to 9% on one thread. The controlled measurement, the one we stand by, came out at a third to a half of the projection. We have not yet dissected the difference. The probe and the benchmark ran on different processors, and they measure different things: the kernel's own processor time in one, the whole decoder's elapsed time in the other.

Two smaller observations remain open. Wikidata, whose full blocks all take the fast kernel, came out 1.7% faster on one thread, a gain the benchmark's stage-by-stage timings do not locate. At four threads, the benchmark also flagged a 1.8% rise in the extra processor time Wikidata costs as threads are added, which only a machine with more cores can judge.

## 5\. What this changes

The argument forms a short chain. A kernel tuned on one corpus became the decoder's pride, but on ordinary files it is eligible for only one full block in six. Where it hesitated over a dense block, it paid for a pass it then threw away, and made prose slower than the fallback alone. That refusal turned out to be a simple count over a list the decoder already holds, so it is now made before any work is wasted. The fallback, which is what ordinary text actually runs, spends a third or more of its inversion building its table, and three memory-neutral changes have made that markedly cheaper.

None of these figures yet comes from the ten-core workstation on which mibz2's releases are measured. They come from virtual machines and the project's cloud benchmark, and both changes will first be measured on the workstation with the next release. The broader lesson concerns benchmarks more than kernels. A corpus chosen because it matters can quietly decide which code is ever examined, and the next speed-up may be found by decoding something else.

---

**Sources:**

- The route census: [the research note](<https://github.com/lmmx/mibz2/blob/b8fd08af523b3d501d5f797faaa1aa96348f70c7/docs/journal/2026-11-27-explore/route-census.md>), its [per-block timings](<https://github.com/lmmx/mibz2/blob/b8fd08af523b3d501d5f797faaa1aa96348f70c7/docs/journal/2026-11-27-explore/route-census/run-route-blocks.tsv>) and [the census program](<https://github.com/lmmx/mibz2/blob/b8fd08af523b3d501d5f797faaa1aa96348f70c7/examples/route_census.rs>) ([\#1023](<https://github.com/lmmx/mibz2/pull/1023>)); 4-vCPU Xeon virtual machine, single thread.
- The early refusal check: [the change to the fast kernel's admission](<https://github.com/lmmx/mibz2/blob/2915d7e79f7d53f0bfc9155c3005d47d4fdc895a/src/open.rs>) and [its record](<https://github.com/lmmx/mibz2/blob/2915d7e79f7d53f0bfc9155c3005d47d4fdc895a/frontier/deltas/2026-11-27-dense-cap-precheck.md>) ([\#1040](<https://github.com/lmmx/mibz2/pull/1040>)), with the [benchmark report](<https://github.com/lmmx/mibz2/pull/1040#issuecomment-7207435327>) (AMD EPYC 9V45 runner, 10 paired rounds).
- The general-purpose kernel's dissection: [the research note](<https://github.com/lmmx/mibz2/blob/9f89ba509f8f0348d1b4358b95a48b7199112940/docs/journal/2026-11-27-explore/segmented-fallback-dissection.md>) and [the probe](<https://github.com/lmmx/mibz2/blob/9f89ba509f8f0348d1b4358b95a48b7199112940/examples/segmented_dissection_probe.rs>) ([\#1042](<https://github.com/lmmx/mibz2/pull/1042>)); 2-vCPU Xeon virtual machine, seven interleaved repeats. The two-step table is recorded in [the dead ends](<https://github.com/lmmx/mibz2/blob/9f89ba509f8f0348d1b4358b95a48b7199112940/frontier/dead-ends.tsv>).
- The change in the decoder: [\#1080](<https://github.com/lmmx/mibz2/pull/1080>), its [benchmark report](<https://github.com/lmmx/mibz2/pull/1080#issuecomment-7211122449>) (Intel Xeon 6975P-C runner, 10 paired rounds), its [safety review](<https://github.com/lmmx/mibz2/pull/1092>) and [closing record](<https://github.com/lmmx/mibz2/blob/1a459081ccf203b9f98706073595b4fcc6701257/frontier/deltas/2026-11-27-segmented-fallback-build-done.md>).
- The dense band's limit: [Admitting dense blocks to the Open kernel](<https://lmmx.github.io/mibz2/pr-preview/pr-1294/blog/2026-11-21-a-memory-gain-is-a-gain/index.md>).
- Juha Kärkkäinen, Dominik Kempa and Simon J. Puglisi, [Slashing the Time for BWT Inversion](<https://researchportal.helsinki.fi/en/publications/slashing-the-time-for-bwt-inversion/>), DCC 2012.
