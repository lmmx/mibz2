# The prompt sent to Gemini 4 Neon

Sent by Louis on 2026-11-19, as prepared in the Coordinator's session. It is verbatim apart from this heading.

```text
Mathematical research challenge: Can a better induction schedule beat Rauzy–Veech BWT reconstruction?

You are being invited to contribute independently to an active research programme in advanced algorithms, symbolic dynamics, interval exchanges and combinatorics on words.

I am particularly interested in your capacity for original mathematical reasoning: finding useful invariants, proving theorems, constructing counterexamples, recognising connections to existing research, and developing algorithms whose full computational costs withstand scrutiny.

Please approach this as a research mathematician rather than a conventional software optimisation assistant.

1. The baseline is already an induction

Consider a byte-valued Burrows–Wheeler last column L of length n, with a designated primary row and its stable-rank LF permutation.

In the relevant formulation, LF is a discrete interval exchange: partition the rows into the r maximal constant-symbol BWT runs. On each interval, LF acts as an affine translation.

The existing production decoder, Cells, reconstructs the original text by Rauzy–Veech induction on this exchange.

It progressively shrinks the section, takes first returns, records return words, and reconstructs the marked output from the induced structure.

This is not a hypothetical approach. It is implemented in a high-performance Rust bzip2 decoder:

* Repository: https://github.com/lmmx/mibz2
* A new way back from the BWT: https://lmmx.github.io/mibz2/blog/2026-11-10-rauzy-veech-reconstruction/
* Rauzy technical companion: https://lmmx.github.io/mibz2/blog/2026-11-12-rauzy-technical-companion/
* Induction width: https://lmmx.github.io/mibz2/blog/2026-11-15-induction-width/

Read these articles carefully, especially their treatment of return words, interval structure, reconstruction and induction width.

Relevant mathematical background includes Rauzy (1979), Veech (1982), Lopez–Narbel (1999), and branching Rauzy induction as studied by Dolce and Perrin (2017).

Do not assume that the finite, marked and potentially multi-cycle BWT setting automatically inherits properties of minimal interval exchanges. State the precise hypotheses under which any transferred result holds.

2. The research generalisation: choosing different sections

A separate mathematical programme explores induction on sections other than the shrinking end-cut intervals used by Cells.

One important construction selects a word cylinder I(P), corresponding to occurrences of a delimiter word P.

The complete words between successive positive returns to this section form a derived alphabet. Their source order induces a token BWT.

This construction can be iterated. Alternatively, several induction levels can sometimes be collapsed into a compound delimiter in the original byte BWT.

Established results include:

* A single nonempty word cylinder induces at most r runs in its token BWT, where r is the original maximal BWT run count.
* This bound is specific to word cylinders. An arbitrary contiguous non-cylinder section can exceed r, with an established upper excess of two runs for one interval.
* The most frequent return word can be selected using at most ⌊n/f⌋ expanded FM intervals, where f is its frequency.
* Recursive selection can substantially reduce the residual number of tokens.
* Collapsing levels avoids intermediate constructions but can greatly increase the cost of discovering and representing long return-word spellings.
* Further constructions use weighted returns, deterministic successor folding, concatenation grammars and sparse exceptional returns.

These constructions may have close connections to derived sequences, return substitutions, prefix-free parsing, and inducing discrete interval exchanges. Investigate the exact relationships rather than assuming novelty.

Research journal:
https://github.com/lmmx/mibz2/blob/research/astra-e-moonshot/research/astra-e/formulations/LEDGER.md

Continuation record:
https://github.com/lmmx/mibz2/blob/research/astra-e-moonshot/research/astra-e/HANDOFF.md

You may inspect the research to establish precise definitions and avoid duplicating known failures, but your investigation should develop its own argument.

3. The central challenge: optimise the induction schedule

Is there a choice of induction sections, their order, and the sharing of return-word information across levels whose complete computational cost is lower than Cells' existing Rauzy–Veech reconstruction?

The important object is the induction schedule itself, not merely the eventual size of the induced permutation.

A schedule may use end cuts, word cylinders, compound markers, arbitrary contiguous sections, adaptive interval refinements, or other mathematically justified sections.

You may formulate a broader class of induction operations if useful.

The challenge is to understand the trade-off between:

1. The cost of locating and inducing on a section.
2. The cost of determining the exact return permutation and token ordering.
3. The cost of recovering, sharing or representing return-word spellings.
4. The amount of residual reconstruction required.
5. The transport of the primary mark and byte offsets across induction levels.
6. The maximum live state and working memory required by the schedule.

Look for structural invariants, amortised bounds, optimality conditions or lower bounds.

In particular, investigate whether a schedule can avoid repeatedly recovering information already represented at earlier levels.

You are free to conclude that a broad class of alternative schedules cannot improve on the existing approach.

4. Existing performance evidence

Several mathematical shortcuts have been constructed and measured.

* First-level delimiter construction alone has previously consumed approximately 97% of the LazyCells inverse-stage CPU on a measured set of blocks.
* Dynamic interleaving of independent rank searches substantially reduced that cost.
* Selecting two deeper return markers can reduce a representative residual section from 135,716 rows to 3,232 with very little selection work.
* Skipping two intermediate token BWTs reduced final token visits by 87%, but increased original-column discovery lookups from 28.1 million to 89.0 million. The complete inverse took approximately 2.36× the fixed-marker version.
* A grammar-folding approach eliminated the residual permutation but was substantially slower than the incumbent once construction was charged.

The recurring observation is that eliminating an intermediate structure often makes the information it carried more expensive to recover elsewhere.

Explain whether this reflects a fundamental mathematical obstruction, an unfortunate induction schedule, or an avoidable implementation cost.

Do not assume a conservation law without proving an appropriate statement.

5. Sparse exceptions and predictive induction

A more recent direction uses deterministic default predecessor transitions on intervals of physical BWT ranks.

Most positions follow the predicted transition. Exceptions are represented separately, and their weighted return structure can itself be induced.

For an eight-byte context model, 4.55% of source positions were exceptional across ten measured blocks.

This introduces another possible class of induction schedules: select a predictable component, induce on the exceptional structure, and recursively reconstruct the complete marked output.

Relevant questions include the relationship between exceptions and BWT run boundaries, LF images of boundaries, induced return maps and move structures.

This direction is available for investigation, but you should not assume it is better than other section choices.

6. Practical performance constraints

The intended application is high-throughput bzip2 decoding.

* Each block has at most 900,000 source bytes.
* A dense 32-bit LF successor table fits in approximately 3.6 MB.
* Seven to ten workers can execute concurrently, competing for cache capacity and memory-system resources.
* Unpredictable accesses may be L2 or L3 misses rather than DRAM accesses.
* A mathematically superior asymptotic complexity does not guarantee better performance at these block sizes.

The strongest independently measured current implementation, Scalar-cap, achieves approximately 1.14–1.16× the reference decoder's throughput, with about one-third less peak memory.

The research objective is at least 1.25× throughput at each of 7–10 workers, without increasing peak memory.

For perspective, prepared sparse-exception emission has been measured at approximately 0.628× Scalar-cap's inverse-stage CPU, while acquiring the required events in the current eight-byte-context approach costs approximately 2.35×. If these costs are additive, acquisition would need to fall below roughly 0.37× merely to break even. The complete paid construction is decisive.

When analysing an algorithm, distinguish:

* Sequential memory traffic.
* Dependent or unpredictable memory accesses.
* Arithmetic and branching.
* Parallel depth and available independent work.
* Construction and intermediate-representation costs.
* Retained memory and peak storage.

Do not charge an expensive operation to an unmeasured preprocessing oracle.

7. Research method

Begin with the precise mathematical structure of Cells' induction and the alternative return-word inductions.

Then identify the most promising mathematical obstacle or opportunity.

Choose one substantive direction and investigate it deeply.

For any important claim, distinguish between:

* A proved theorem, with a complete proof.
* A computationally verified result, with explicitly bounded scope.
* A conjecture.
* A refuted proposition, with a reproducible counterexample.

Seek counterexamples before presenting general theorems.

In particular, account carefully for tied rotations, periodic inputs, multiple LF cycles, and primary marks that may lie outside a selected return section.

Explore appropriate mathematical connections, including Rauzy–Veech induction, derived sequences, symbolic dynamics, return substitutions, interval exchanges, prefix-free parsing, move structures and graph-width measures.

Check actual literature before making claims about prior art or novelty.

8. Requested research output

Produce a self-contained mathematical research note with:

1. Formal model: A precise definition of the admissible induction operations and schedules.
2. Key insight: The strongest mathematical statement or construction you can establish.
3. Proof or falsifier: A rigorous proof, finite exhaustive check or explicit counterexample.
4. Cost model: Complete construction and reconstruction complexity, including peak storage.
5. Comparison with Cells: Explain concretely what your result does differently from the existing Rauzy–Veech schedule.
6. Relevant prior art: Identify established results and distinguish them from your own deductions.
7. Decisive experiment: Propose the smallest controlled test that would reveal whether the construction has practical merit at n ≤ 900,000.
8. Research judgement: State what remains unknown, what is likely to fail, and what would justify further work.

A negative result establishing an obstruction is as valuable as a positive construction.

An exact but slower reconstruction is not a performance breakthrough.

You are not required to modify the repository or compete with another research agent. This is an independent mathematical investigation.

Your central task is to understand whether a better induction schedule exists—and to make the strongest defensible mathematical progress towards answering that question.

Prioritise one deep result over a large collection of speculative ideas.
```
