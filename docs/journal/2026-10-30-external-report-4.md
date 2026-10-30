# External research report 4: scan-based BWT inversion and the breakthrough's enabling conditions (verbatim)

A research input, not a finding of this repository. It was relayed by the maintainer on
2026-10-30 from the research assistant's session, in answer to the maintainer's prompt
"search more deeply and think hard about what you find in this vein, seek the breakthrough
enabling conditions in precedent". "This vein" is the scan-based (streaming-pass) BWT
inversion family, which the coordinator had proposed as a portfolio candidate. The
maintainer asked for it to be recorded verbatim.

It is source material for the research portfolio. Its
claims are checked against the primary sources and against this journal's history in the
portfolio's register. They are not taken as findings.

The report reached us as rendered text. Its citation markers (the `[image](…)` favicon
links, the bare site names after them, and the `&#x20;` entities) are kept as received.

## The report, as received

````text
# The breakthrough search: what scan-based BWT inversion actually teaches us

I found a useful distinction in the literature, and it changes how I would pursue Claude's new idea.

The strongest opportunity is not necessarily to replace every random Ψ lookup with sequential scans. It is to discover a way of reconstructing long stretches of output without having to follow every link in those stretches.

That sounds similar, but it's algorithmically different.

The distinction emerged from three lines of precedent:

1. External-memory BWT inversion, where sorting and scanning replace pointer chasing.
2. Parallel list ranking, where the permutation is broken into independently traversable pieces.
3. Work-efficient parallel BWT decompression, where apparently serial stages are reformulated as prefix computations.

The third line is particularly interesting because it extends the search beyond inverse BWT itself.

## 1. What the scan-based inversion paper actually proves

I read Section 5 of [Ferragina, Gagie and Manzini — Lightweight Data Indexing and Compression in External Memory](https://arxiv.org/pdf/0909.4341?utm_source=chatgpt.com) (2010 preprint; Algorithmica, 2012).

The paper's starting observation is exactly ours:

Reconstructing the original text from BWT can be reduced to ordering a linked list whose successor relation is Ψ.

Instead of traversing that list one link at a time, the authors maintain a collection of substrings and progressively join them.

Their scheme has roughly logarithmically many growth rounds, and uses sorting/list-ranking operations to establish which fragments belong next to one another. With sequential-only external storage, their pass bounds are \\(O(n/M)\\) on one disk or \\(O(\log^2 n)\\) on multiple disks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://arxiv.org\&sz=32)

arxiv.org+1



The important point is what this doesn't demonstrate.

It doesn't give us a constant-number-of-scans, fast CPU decoder for 900 KB blocks.

The original paper addresses a setting in which the entire problem does not fit in internal memory. Its algorithm deliberately pays additional computation to avoid random disk access and reduce external working space.

On our workstation, by contrast, the navigation array is already in RAM, and our cooperative walker deliberately hides many of its access latencies.

So directly substituting the external-memory algorithm is unlikely to be a breakthrough.

But the paper contains a powerful enabling idea:

> Reconstruction can be expressed as a problem of growing, ordering and joining already-decoded fragments, rather than necessarily emitting one character per traversal step.

That opens up a different family of algorithms.

## 2. The critical bottleneck isn't finding fragments. It's discovering their order.

This is the most important deduction from the paper.

Imagine that we have broken a BWT permutation cycle into many short chains.

Each chain can be decoded independently.

That is essentially what our current cooperative segmented walker already accomplishes.

But after computing the chains, we must determine which follows which.

The algorithmic challenge becomes:

\\[ \text{fragment }i \longrightarrow \text{fragment }j \longrightarrow \text{fragment }k. \\]

If that fragment-level successor graph is still an unpredictable permutation, we've merely moved the pointer-chasing problem to a smaller scale.

That can be useful—but it is not automatically transformational.

Three levels of reconstruction

Fragment decomposition reduces the number of links in the final ordering problem, but doesn't by itself eliminate the work of decoding fragments or discovering their successors.

The literature on practical parallel list ranking makes this trade-off explicit.

For example, [Rehman, Kothapalli and Narayanan — Fast and Scalable List Ranking on the GPU](https://people.cs.uchicago.edu/~suhail/publication/rehman-fast-2009/?utm_source=chatgpt.com) (2009) contrasts repeated pointer jumping with a recursive fragment-based method derived from Helman–JáJá.

The authors report that naive pointer jumping suffers from excessive work and scattered memory accesses. Their fragment-based approach performs local rankings, constructs a smaller list of fragments, ranks that list, and propagates the positions back.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://people.cs.uchicago.edu\&sz=32)

Suhail Rehman



That's a useful proof of concept for the shape of the solution.

But it also shows why our cooperative decoder is already partway along this path.

We have independently decoded segments and a mechanism for joining them.

So an additional algorithm must do something genuinely new:

- Make the local traversal cheaper.
- Make fragments substantially longer without traversing every byte.
- Or derive fragment order without another expensive permutation walk.

Simply adopting a more elaborate list-ranking procedure may add work we don't need.

## 3. What would make a scan-based method competitive?

Let's make the physical cost explicit.

For a full block:

\\[ n=900{,}000. \\]

A 32-bit navigation table occupies:

\\[ 4n=3.6\text{ MB}. \\]

At an illustrative sustainable sequential bandwidth of 28 GB/s, a single read-only sweep would have an ideal transfer time of approximately:

\\[ \frac{3.6\text{ MB}}{28\text{ GB/s}} \approx0.129\text{ ms}. \\]

But that's just the minimum data-transfer time, assuming the quoted bandwidth applies to the relevant memory level and nothing else competes for it.

A real pass may read and write multiple arrays, perform scatter operations, move fragments, execute prefix computations and synchronize threads.

If an algorithm needs 20 full-table sweeps, even its idealized read traffic alone costs roughly 2.6 ms per block. Additional writes, sorting and instruction costs make it worse.

That gives us a sharp enabling condition:

A streaming approach needs either very few whole-block passes, exceptionally compact per-pass data, or a way to avoid processing most rows in later passes.

The third possibility is intriguing.

Suppose the first pass identifies long fragments whose internal order can be determined cheaply.

Later passes might manipulate only the fragment headers.

That moves us toward the reduced-list strategy in practical parallel list ranking.

However, the first pass still needs to discover enough structure to justify its cost. It cannot merely copy the existing T-vector into another representation.

# 4. The most promising possibility: exploit BWT structure before doing list ranking

This is where our own Ψ research might offer something beyond a generic list-ranking algorithm.

We know that Ψ has substantial translation structure:

\\[ \Psi(j)=j+d \\]

over many intervals.

This is not true of an arbitrary random permutation.

Consequently, generic list-ranking complexity may overstate the work required to exploit the structure of our particular permutation.

But there's a trap.

A translation interval is contiguous in row order, not necessarily in traversal order.

Knowing that neighbouring rows have neighbouring successors doesn't establish that following one row for several steps remains inside translation intervals.

For a multi-step shortcut, we need a sequence of compatible interval mappings:

\\[ I_0\xrightarrow{\Psi}I_1 \xrightarrow{\Psi}I_2 \xrightarrow{\Psi}\cdots \\]

If each mapped image remains inside a translation interval, their composition is again a translation.

For instance, if:

\\[ \Psi(j)=j+a,\qquad \Psi(j+a)=j+a+b, \\]

throughout a suitable source interval, then:

\\[ \Psi^2(j)=j+a+b. \\]

This permits a two-step successor to be encoded by a single displacement over that entire interval.

More generally:

\\[ \Psi^k(j)=j+\sum\_{t=1}^{k}d_t \\]

when the translated images remain in compatible intervals.

That is an actual mathematical mechanism for compressing multi-step navigation.

It need not require storing an independent Ψ² pointer for every row.

But whether it is useful depends on the survival of those intervals under repeated composition.

If their lengths collapse rapidly, the representation becomes a collection of tiny exceptions and loses its appeal.

If they survive for multiple steps across a substantial portion of the rows, we may have found a nontrivial way to reduce dependent loads.

This is precisely why Claude's proposed interval-composition census is worth doing.

It should count not just surviving intervals, but how many useful navigation transitions each shortcut would eliminate, and how much metadata is needed to recognize and execute it.

## 5. A second enabling condition: exploit repeated traversal paths, not merely repeated rows

There is another way to obtain long fragments.

The 2011 and 2012 BWT inversion papers established that repeated text can create parallel traversal paths.

If two paths maintain a fixed offset through Ψ for several steps, we can potentially reuse characters already reconstructed along one path.

This is the mathematical basis of Precopy.

The crucial difference from the translation-composition approach is that it doesn't necessarily compress Ψ² itself.

Instead, it substitutes previously reconstructed output for navigation through a repeated path.

For highly repetitive data, the saving could be enormous.

But mibz2's earlier Precopy experiments were negative. The research journal records substantial construction and bookkeeping costs, including previous approaches that did not implement precisely the lightweight detection described in the original literature.

That gives us two genuinely different structural shortcuts:

| Mechanism               | What is avoided                        | Main obstacle                                                          |
| ----------------------- | -------------------------------------- | ---------------------------------------------------------------------- |
| Translation composition | Repeated dependent successor loads     | Compatible intervals may disappear rapidly                             |
| Repeated-path copying   | Revisiting Ψ to reconstruct known text | Detecting reuse and finding correct output locations may cost too much |

Both need a profitability model that includes construction and per-step detection.

And, crucially, neither should automatically be treated as a new candidate merely because we express it in a different mathematical language.

The novelty audit must establish what materially differs from the previous experiments.

## 6. Another discovery: the front end also has scan-based formulations

This is a particularly useful expansion of the research search.

I found [Edwards and Vishkin — Parallel Algorithms for Burrows–Wheeler Compression and Decompression](https://doi.org/10.1016/j.tcs.2013.10.009?utm_source=chatgpt.com) (Theoretical Computer Science, 2014).

The paper presents work-optimal PRAM algorithms for BWT compression and decompression.

Its abstract explicitly identifies three relevant techniques:

- Parallel Huffman decoding.
- Associative prefix computations for several stages.
- List ranking for inverse BWT.

It also notes that translating theoretical parallelism into practical speedups on commercial hardware has proven difficult.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.sciencedirect.com\&sz=32)

ScienceDirect



This matters because Claude's updated budget says Huffman and MTF together represent roughly four cycles per output byte—comparable to the walk.

There's even more direct practical precedent in [Parallel Lossless Data Compression on the GPU](https://escholarship.org/uc/item/77c4w5rz?utm_source=chatgpt.com), which discusses a scan-based parallel formulation of reverse move-to-front decoding.

Instead of treating the MTF state as necessarily requiring serial execution of every symbol, the authors represent the transformation of MTF state over a sequence in a form suitable for prefix-style composition.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://escholarship.org\&sz=32)

escholarship.org+1



This introduces a conceptually different research direction for mibz2:

Instead of optimizing how quickly we update the 256-byte MTF list, can we compute the effects of multiple encoded symbols together?

There is a serious practical challenge.

A general transformation on a 256-symbol move-to-front state can be expensive to represent and compose. A prefix-scan algorithm may gain parallel depth at the price of substantially more total work.

For our four-core target, that could be a bad exchange.

But it is precisely the kind of precedent that deserves examination when searching for a breakthrough.

The useful question is not merely whether MTF is parallelizable.

It is whether its restricted operations admit a compact composition representation—particularly given the actual MTF-index distributions in Wikidata.

That would be a much more targeted investigation than porting a GPU algorithm to Rust.

# 7. The crucial synthesis: there are two ways to change the cost model

I think the literature reveals two distinct classes of breakthrough.

Class I — Reduce the work that must be done

Exploit mathematical structure so that several symbols or traversal transitions can be reconstructed from one cheaper operation.

Examples: composable Ψ translations, profitable repeated-path reuse, compact multi-step shortcuts.

Class II — Change the dependence structure

Reformulate serial operations into independent or associative computations that can be executed in parallel, batched or streamed.

Examples: fragment-level list ranking, scan-based state composition, parallel Huffman decoding.

The second class can reduce wall time without reducing CPU-seconds, and sometimes increases total work considerably.

That's why, given our new target, Class I deserves particular attention.

But an unusually efficient Class II algorithm could still produce a major speedup if it replaces expensive unpredictable loads with a small number of cheap sequential operations.

The decisive metric is total useful work and traffic—not asymptotic parallel depth alone.

## 8. A useful feasibility matrix from the precedent

These are my research rankings, not measured mibz2 results.

| Candidate                       | Breakthrough enabling condition                                                       | Main reason it might fail                            | First falsification                                    |
| ------------------------------- | ------------------------------------------------------------------------------------- | ---------------------------------------------------- | ------------------------------------------------------ |
| Composed Ψ intervals            | Long, frequently usable translations survive multiple steps with small metadata       | Intervals fragment under composition                 | Read-only composition census                           |
| Structure-assisted list ranking | BWT structure yields long fragments or cheap ordering without full pointer traversals | Adds passes without eliminating enough work          | Bound passes, headers and bytes moved                  |
| Streaming BWT inversion         | Few scans replace enough expensive random accesses                                    | Too many global sorting/merging rounds               | Cost model against measured walk time                  |
| Repeated-path reuse             | Large fraction of output can be copied with cheap detection                           | Detecting and managing reuse dominates               | Reconcile Precopy results and measure profitable reuse |
| Scan-based reverse MTF          | Compact composable state summaries make batched decoding cheap                        | Prefix operations cost more work than sequential MTF | Algebraic state-size and operation-count analysis      |
| Parallel Huffman decoding       | Efficient chunk-boundary recovery exposes independent decoding work                   | Unknown codeword boundaries and recovery overhead    | Paper-specific method audit plus bitstream statistics  |

For each candidate, I'd require two ceilings.

The first is its algorithmic ceiling: how much work it could theoretically remove from the target stage.

The second is its decoder ceiling: how much that could improve a full decode if the rest of the pipeline stays unchanged.

That avoids confusing a profound result about Ψ with a guaranteed application-level speedup.

## 9. One additional mathematical test I would add

For the Ψ-composition avenue, I would not stop at counting intervals surviving under Ψ².

I'd define a quantity that directly measures the opportunity.

Let \\(k(j)\\) be the maximum useful shortcut length available at row \\(j\\), subject to a specified metadata budget and fast-path lookup cost.

Then estimate:

\\[ S=\sum_j\left(1-\frac{1}{k(j)}\right) \\]

as a preliminary measure of avoidable navigation work.

There is a qualification: this expression is only a heuristic potential measure, because overlapping shortcuts cannot all be selected simultaneously, and a real traversal must avoid double-counting positions.

A proper read-only analysis would take the actual permutation cycle, choose nonoverlapping eligible shortcuts, and count the navigation loads eliminated.

It would also count the extra metadata lookups and output-symbol operations introduced.

That gives a much more meaningful result than simply saying “60% of rows lie in composable intervals.”

The benchmark-worthy statistic is:

> How many dependent memory operations can be removed from a complete traversal, after accounting for the cost of recognizing and executing the shortcuts?

That is the quantity with a possible path to a breakthrough.

# 10. What I would tell Claude

I've investigated the scan-based and parallel-inversion literature more deeply. Please add the following findings to the research corpus for reconciliation, not as authorization to implement anything.

**1. Ferragina, Gagie and Manzini (2012), Lightweight Data Indexing and Compression in External Memory.** The scan-based BWT inversion in Section 5 uses fragment growth, list-ranking and sorting over multiple rounds. Its stated bounds are O(n/M) sequential passes with one disk or O(log² n) passes with multiple disks. This establishes a precedent for replacing pointer traversal with fragment assembly, but it does not provide a cheap constant-pass decoder for in-memory 900 KB blocks.

Source: [https://arxiv.org/pdf/0909.4341](https://arxiv.org/pdf/0909.4341)

**2. Practical parallel list ranking.** Rehman, Kothapalli and Narayanan (2009) compare repeated pointer jumping with recursive Helman–JáJá fragment-based ranking. The latter ranks local sublists, constructs a reduced fragment list, ranks that list and propagates positions back. Compare this with mibz2's existing segmented cooperative walker before claiming novelty.

Source: [https://people.cs.uchicago.edu/\~suhail/publication/rehman-fast-2009/](https://people.cs.uchicago.edu/~suhail/publication/rehman-fast-2009/)

**3. Structural multi-step shortcuts.** If Ψ acts as a translation across successive compatible intervals, its powers remain translations there. A read-only census should measure how rapidly compatible intervals fragment, and whether a nonoverlapping covering of actual traversal steps can eliminate substantial dependent loads within a realistic metadata budget. This is a mathematical hypothesis, not a published mibz2 performance result.

**4. Front-end scan formulations.** Edwards and Vishkin (2014) describe work-optimal PRAM BWT decompression using parallel Huffman decoding, associative scans and list ranking. Separate GPU work on parallel lossless compression also describes a scan-based reverse-MTF formulation. Investigate whether the specific operations admit compact state summaries that reduce work on a four-core CPU; do not assume parallel scan is automatically faster.

Sources:\
[https://doi.org/10.1016/j.tcs.2013.10.009](https://doi.org/10.1016/j.tcs.2013.10.009)\
[https://escholarship.org/uc/item/77c4w5rz](https://escholarship.org/uc/item/77c4w5rz)

Please keep these as distinct candidate mechanisms: composition-based shortcutting, fragment/list-ranking inversion, and front-end state composition.

The breakthrough condition is not merely less serial dependence. It is a material reduction in full-decoder work or memory cost, with a plausible route to improved CPU-seconds per GB.

For each, identify what fundamentally new assumption or structural property would make it outperform the earlier mibz2 attempts. If no such condition exists, preserve it as literature context rather than a new experiment.

## My research verdict

After following this literature, I would not put generic scan-based BWT inversion at the top of the engineering queue.

The paper is more valuable as a conceptual guide to fragment assembly than as a directly transferable decoder.

The more intriguing opportunities are:

First, exploiting structured compositions of Ψ. This is the strongest connection to the mathematical evidence we've already collected. It could, if sufficient structure survives, reduce actual dependent navigation work rather than merely overlap it.

Second, finding a compact algebra for reverse MTF or Huffman decoding. This is a more radical opening into the front end, which has received less attention and accounts for a substantial fraction of decoder CPU.

Third, investigating a genuinely work-reducing hybrid of fragment assembly and repeated-path reuse. The condition here is strict: it must avoid more pointer traversals than it adds in discovery, bookkeeping and ordering work.

The deepest commonality is this:

A breakthrough becomes possible when we discover that the state needed to perform many serial operations can be summarized much more cheaply than performing those operations individually.

For Ψ, that state might be an interval translation or repeated traversal fragment.

For MTF, it might be a compact composition of state transformations.

For Huffman, it might be a small summary of what a chunk does to the decoder's bitstream state.

That's the enabling condition I would ask the portfolio to search for—not simply more parallelism, and not just smaller tables.

I haven't found evidence that any of these conditions holds strongly enough on Wikidata to deliver the breakthrough yet. But the precedents show where the necessary mathematical structure would have to appear, and give us concrete ways to falsify each possibility before committing the workstation.
````
