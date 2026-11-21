[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-709/blog/index.md>)

# A memory gain is a gain: the change our speed-only rule nearly shelved

Sending five dense blocks to our leanest decoding kernel cut peak memory by a quarter at eight threads without slowing anything, so memory now counts as a gain.

21 November 2026 · 12 min read · [Research result](<#6-what-comes-next>)

Shipped in mibz2 [0.5.11](<https://lmmx.github.io/mibz2/pr-preview/pr-709/benchmarks/history/#release-0-5-11>) ([changelog](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#0511---2026-11-21>))

mibz2 is a bzip2 decoder in Rust. Today it merged a change that makes the whole decoder use about a quarter less peak memory at eight threads. Its speed did not measurably change. Under the rule we used to accept performance changes, that was not enough. The rule asked for a gain in wall time, and this change had none. We reported it as holding. Louis, who maintains mibz2, replied:

> Oh no a major memory gain and no perf regression is also well worth shipping! Don't hold it. And make the durable policy protocols edits to ensure we don't waste such gains in future

So the change shipped, and the rule changed with it. This post is about both. It covers what the change does and how its claims fared when we ported it. It also argues that a rule counting only one figure will keep throwing away real gains.

**The takeaways:**

- **The change routes five dense blocks to a leaner kernel.** Of the 233 full-size blocks in our Wikidata benchmark, five were too dense for the fast kernel's old admission test. They fell back to an older path that allocates about 5.6 MiB per worker. A new admission band takes them, under a cap that keeps the kernel's memory bounded. The worst worker's kernel memory falls from 6.8 MiB to 1.3 MiB.
- **The memory gain is large and the speed is unchanged.** End to end, peak memory is 0.741 of the old at 8 threads and 0.783 at 10, with intervals well below 0.90. Wall time is 0.978 and 1.002, with intervals that hold 1. CI's own bench at 1 to 4 threads shows no primary loss.
- **Most of the research line's speed claims did not transfer, and we report that.** Its contention model predicted the dense blocks would take under half the time. Natively they take longer on one thread, and 7.5% less time with four workers. The memory claim held, against a newer baseline than the one it was measured on.
- **The rule now counts memory.** A change that cuts peak memory by at least 10%, beyond noise, with no loss of speed, is accepted and released like a speed gain. The evidence must be as rigorous as for speed: a paired run, an A/A control, intervals.

[Section 1](<#1-why-did-five-blocks-take-the-expensive-path>) explains the five blocks. [Section 2](<#2-where-the-change-came-from>) shows where the change came from and what its research line claimed. [Section 3](<#3-what-held-up-natively>) gives the native measurements, and [section 4](<#4-why-we-nearly-shelved-it>) how the old rule nearly discarded it. [Section 5](<#5-the-rule-we-wrote>) gives the rule that replaced it, and [section 6](<#6-what-comes-next>) what comes next.

## 1\. Why did five blocks take the expensive path?

bzip2 cuts a file into blocks of up to 900 kB and applies the Burrows–Wheeler transform (BWT) to each. The transform reorders a block so that bytes with similar contexts sit together, and the same byte tends to repeat in *runs*. Decoding a block means undoing that reordering, and it is most of mibz2's work. mibz2 decodes several blocks at once, one per *worker* thread, and every worker needs its own scratch memory.

mibz2 has several ways to undo the transform. The newest, Open, keeps only the rows of a block's narrow runs and moves the wide runs in groups ([Open returns](<https://lmmx.github.io/mibz2/pr-preview/pr-709/blog/2026-11-19-open-returns/index.md>)). Since the last release, its scratch is about 1.2 MiB per worker. That is small enough for seven to ten workers to fit in the 20 MiB cache shared by the ten cores of the i9-10900K we test on. But Open takes a block only when the block passes its admission test. The test uses the block's density, r/n: the number of runs over the number of bytes. A block with many short runs has a high r/n, and above 0.15 Open's old test refused it. A refused block went to the older kernels, LazyCells and then Segmented. Segmented builds a full table of the block and allocates about 5.6 MiB per worker.

Our benchmark is an 11.9 MB extract of a Wikidata dump: 431 blocks, of which 233 are full-size blocks on the route Open serves. Open's old test admitted 228. Five were refused, with densities from 0.151 to 0.206. Any worker that met one of them held Segmented's 5.6 MiB, so the worst worker's memory was set by these five.

The change adds a *dense band*. A block with r/n between 0.15 and 0.28 is admitted if Open's first pass stays under a cap. The cap is min(256,000, 0.3n) retained rows and 16,384 thick strips. The check sits on the queues' cold path, so it costs nothing on the hot path. Once the cap is passed, a sticky flag stops the pass and sends the block to LazyCells as before. The five blocks fit the cap with 199,189 to 231,600 retained rows and 568 to 2,729 strips.

Where the 233 full-size blocks go, before and after the dense band

Measured routing on the benchmark's 431 blocks (\#703's census). The 228 blocks below r/n 0.15 are admitted by Open's count gates in both versions; their individual densities are not plotted. The five above the line are plotted at their measured r/n. The 150 tiny blocks and the 48 blocks on the column route are unchanged and not shown.

## 2\. Where the change came from

The change began in one of our research lines, Mechanisms, an AI research session that designs reconstruction kernels and times them in its own harness. Its cycle 19 ([journal](<https://github.com/lmmx/mibz2/blob/7d86bb8601800414b2e02a1bd37a493d5cde52c3/docs/journal/2026-11-21-carbon-cycle19-unified-synthesis.md>)) built a candidate called `AhtEdgesMux16` from two ideas.

The first was admission: a structural gate that let Open take every full-size block, the five included. Behind it is a certificate. The step that eliminates intervals never makes a strip wider, so the widest strip at the start bounds every later step. A block whose strips are all narrow can therefore run the plain loop. The rare block that fails this certificate takes a contracted loop, which handles long repetitive spirals by repeated squaring. The second was *EdgesMux*: three working buffers live inside the `edges` array, which the kernel does not write until a late phase. Its Theorem 19.1 is the lifetime argument: each buffer dies before `edges` is first written. Its verifier checked this by poisoning each buffer after its last read, in 1,000 runs.

Its own measurements, against mibz2 0.5.8, were striking. In its words, the candidate cut the ten-worker maximum heap per worker from 9,979.4 KiB to 1,346.0 KiB (0.135×). Its contention model is a simulator that charges each worker a share of the cache. On it, the dense blocks took under half the cycles and a few percent of the memory traffic. It passed all four parts of its own adoption gate.

## 3\. What held up natively

We ported the candidate to the real kernel (\#703), in three commits: the dense band, the spiral contraction, and the one part of EdgesMux that master lacked. Master had moved on. The previous release (\#692) had already put Open on a memory diet, using an earlier cycle of the same line. So most of EdgesMux was already there. Only the strips' image order remained to move into `edges`, worth 27–28 KiB.

The harness runs master's kernel, unchanged, in the same test binary as the candidate, and alternates the two. An A/A control, master's tree built as the candidate, shows the noise floor. Twelve rounds give 95% intervals. It ran in a cloud container with 4 virtual CPUs, not the i9.

| measure | A/A control | \#703 against master |
| --- | ---: | ---: |
| kernel heap, worst worker, 4 workers (KiB) | – | 7,128.7 → 1,342.0 (0.188×) |
| kernel heap, worst worker, 10 workers (KiB) | – | 6,804.5 → 1,268.8 (0.186×) |
| 1 thread, the six reference blocks Open already took | 0.993–1.019 | 1.003–1.009 |
| 1 thread, the five newly admitted blocks | 0.987–1.019 | 1.120–1.427 |
| 4 workers, time spent in the five | 1.004 \[0.994, 1.015\] | 0.925 \[0.909, 0.942\] |
| 4 workers, a whole pass | 1.007 \[1.000, 1.013\] | 1.003 \[0.991, 1.016\] |
| end to end, 8 threads, peak memory | – | **0.741 \[0.714, 0.768\]** |
| end to end, 8 threads, wall time | – | 0.978 \[0.928, 1.031\] |
| end to end, 10 threads, peak memory | – | **0.783 \[0.754, 0.812\]** |
| end to end, 10 threads, wall time | – | 1.002 \[0.957, 1.050\] |

Ratios are \#703's time or memory over master's. The end-to-end rows run the `bzcat` example in fresh processes, alternated.

**The memory gain held.** The worst worker's kernel heap fell to under a fifth. It fell because no worker now holds Segmented's table. On a fresh kernel, each of the five blocks drops from about 5.6 MiB to around 1 MiB. End to end, the whole process's peak memory is a quarter lower at 8 threads.

Per-worker memory against each worker's share of the i9's 20 MiB cache

Measured, but two measures are shown. The upper pair is per-worker scratch on the blocks Open admits (0.5.10's changelog, from \#692). The lower pair is the worst worker's kernel heap over the whole corpus at 10 workers (\#703's harness, 4-vCPU container), which includes Segmented's table when a worker meets a refused block. The dashed lines are 20 MiB divided by 7 and by 10 workers.

**The speed claims did not transfer.** The contention model said the dense blocks would take under half the cycles. On one thread they take 1.12–1.43 times as long: Open is slower than Segmented on these blocks when the cache is not contended. With four workers sharing the cache, Open wins, but by 7.5%, not by half. The five blocks are under 4% of a pass, so a whole pass does not move, and in production one thread never uses Open anyway. The model's prediction and the native result disagree by a wide margin. It is the same pattern as the earlier cases in our [fidelity ledger](<https://github.com/lmmx/mibz2/blob/90e671966/docs/team/research-charter/CONTEXT.md#5a-the-fidelity-ledger>), where cheap predictions misled us ([Fast alone, slow in a crowd](<https://lmmx.github.io/mibz2/pr-preview/pr-709/blog/2026-11-21-fast-alone-slow-in-a-crowd/index.md>)).

What the research line claimed, and what the native port measured

Ratios, new over old; below 1 is better. The claims are the line's own (cycle 19 journal), against 0.5.8 and from its contention model. The native figures are \#703's, against 0.5.10, with 95% intervals where the harness gives them. The heap claim held in kind but was measured against an older baseline. The dense blocks' speed claim did not hold.

Two further claims shrank on contact with master. EdgesMux was worth 432 KiB against 0.5.8, but most of that had already shipped in 0.5.10, leaving 27–28 KiB. And the contracted loop for long spirals never fires on Wikidata, since no block fails the width certificate. It stays as a bound, not a gain.

**Correctness was checked at the same standard as any change.** Every block of the corpus, and six authenticated reference blocks, decode byte for byte as the plain inverse does. `bzcat` at 4, 8 and 10 threads gives the same SHA-256 as `bzip2 -dc`. The correctness gate passes its 338 cases. New tests poison each region of `edges` after its last read, and the output stays exact; poisoned before it, as a control, the output breaks. Eight hand-made mutations of the new code were each caught.

CI's paired bench, which runs 1 to 4 threads on a 4-CPU runner, found no primary loss. Wikidata's wall time moved −0.4%, −0.3% and +0.6% at 1, 2 and 4 threads, all within noise. The one flag was text at 4 threads, 1.6% more CPU with wall time unchanged. Peak memory on Wikidata at 4 threads went from 97 MB to 91 MB.

## 4\. Why we nearly shelved it

Our performance policy ([PERFORMANCE-POLICY.md](<https://github.com/lmmx/mibz2/blob/6cd778fa0/docs/team/PERFORMANCE-POLICY.md>), section 2) accepted a change on its paired bench when:

> 1. **A primary gain:** at least one primary corpus is faster in wall time beyond noise (its 95% interval excludes zero) at one or more thread counts.
> 2. **No primary loss:** no primary corpus is slower beyond noise at any thread count.

Memory appeared only as a limit: a change could grow peak memory by up to 10%, or more if it paid for the growth in speed. Nothing said what to do with a change that saved memory and left speed alone. \#703 had no primary wall gain, so under the first rule it was not a gain at all, and we reported it as holding, not worth a run on the i9.

The rule was written for a project chasing speed, and it did that job. Its blind spot was that it measured one figure and treated every other as a constraint. A change that improved a different figure, by a large margin and with clean evidence, fell through.

## 5\. The rule we wrote

The new section, [merged as \#704](<https://github.com/lmmx/mibz2/blob/6cd778fa0/docs/team/PERFORMANCE-POLICY.md>), reads in full:

> **A memory gain is a gain** (Louis, 21 November 2026: "a major memory gain and no perf regression is also well worth shipping! Don't hold it"). A change that cuts memory without speeding the decoder up is **accepted** when all of these hold:
>
> 1. **A primary memory gain:** peak RSS on a primary corpus falls by at least 10%, beyond noise (its 95% interval lies wholly below 0.90 of the base), at one or more thread counts where the new code runs. The measurement is a paired run that executes that code: CI's bench, a paired native end-to-end run reported in the PR (head against base, interleaved rounds, with an A/A control), or a workstation run (C5).
> 2. **No primary loss:** no primary corpus is slower in wall time beyond noise at any thread count, in that run or on CI's paired bench.
> 3. **Bounded secondary losses**, as in rule 3 above.
>
> Such a change merges and releases like a speed gain (RELEASING §1). It is never held for want of a speed gain, and it needs no workstation run unless its memory gain appears only on the target machine. Where its code runs only at thread counts CI can't reach, as with the Open kernel at 6–20 threads, the paired native run in the PR is the evidence, and CI's bench shows there is no loss elsewhere. A change with no memory gain and no speed gain is not a gain under either rule.

The release runbook gained the matching line: a memory gain accepted under this rule is worth releasing.

The rule keeps the evidence standard that the speed rule has. A memory gain must clear 10% with its whole interval, not just its point estimate, so a noisy 12% does not count. It must also be measured where the code actually runs. That matters for Open, which runs only from 6 threads up on the i9, while CI's runners have 4 CPUs. So the PR's own paired native run, with its A/A control, is the evidence, and CI's bench shows only that nothing else got slower.

How a change is now judged, and the path \#703 took

The branches of PERFORMANCE-POLICY.md section 2 after \#704, simplified: the speed rule's memory limit and its secondary-loss tiers are folded into one box each. The highlighted path is \#703's: no wall gain, peak memory 0.741 \[0.714, 0.768\] at 8 threads, no primary loss on CI or in its native run.

The port, \#703, was the first change merged under the rule.

## 6\. What comes next

**Why memory matters here.** A decoder's memory decides two things. The first is speed under load. The fast kernel's last gain came from shrinking each worker's scratch until seven to ten workers fit the shared cache. That release ran 5–8% faster from 6 to 10 threads on the i9. Our research line's model of shared-cache contention says that cutting the working set does not reverse under load. The speed-ups in [Fast alone, slow in a crowd](<https://lmmx.github.io/mibz2/pr-preview/pr-709/blog/2026-11-21-fast-alone-slow-in-a-crowd/index.md>) did reverse, because they borrowed cache other workers needed. The second is who can run it. An ordinary laptop has about 16 GB, and wikidata-pq, the project that runs mibz2 downstream to convert Wikidata dumps, decodes alongside its own work. With this change the worst worker's kernel memory is about 1.3 MiB, where it was about 6.8 MiB.

**What we will measure.** The memory gain is settled by the paired native run. Whether the dense blocks' gain at four workers grows at eight or ten on the i9 is not yet measured. Either way, the five blocks are too small a share of the benchmark to move a whole pass. The release-scaling run of the next release, 0.5.11, which carries this change, will show its effect from 1 to 20 threads.

**The lesson.** A rule that counts one figure and constrains the rest will discard every gain that arrives in a figure it does not count. The fix was not to relax the evidence. It was to count the second figure with the same rigour as the first: a threshold, a paired run, an A/A control and an interval that clears it. The same reasoning applies to any figure a project cares about, from CPU time to start-up latency. If it matters enough to limit, it matters enough to count as a gain.

---

**Research records:** [Mechanisms' cycle 19 journal](<https://github.com/lmmx/mibz2/blob/7d86bb8601800414b2e02a1bd37a493d5cde52c3/docs/journal/2026-11-21-carbon-cycle19-unified-synthesis.md>) and [its artefacts](<https://github.com/lmmx/mibz2/tree/7d86bb8601800414b2e02a1bd37a493d5cde52c3/docs/journal/2026-11-21-carbon-cycle19>); [the native port's journal](<https://github.com/lmmx/mibz2/blob/90e671966/docs/journal/2026-11-21-open-dense-admission.md>) and [its results](<https://github.com/lmmx/mibz2/blob/90e671966/docs/journal/2026-11-21-open-dense-admission/results.txt>) (\#703); [the policy change](<https://github.com/lmmx/mibz2/blob/6cd778fa0/docs/team/PERFORMANCE-POLICY.md>) (\#704); [0.5.10's memory diet](<https://github.com/lmmx/mibz2/blob/90e671966/CHANGELOG.md>) (\#692); [the fidelity ledger](<https://github.com/lmmx/mibz2/blob/90e671966/docs/team/research-charter/CONTEXT.md#5a-the-fidelity-ledger>).
