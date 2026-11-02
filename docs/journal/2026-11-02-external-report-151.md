# External research report 151: the guardian on the amended interpretation: the matched-`exec` comparison reproduced; the exclusion stated narrowly; the diagnostic at its endpoint (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** the coordinator's amended interpretation (walker-ladder §8.4–§8.6, `c11c83a`)
  and its note for the Astra/researcher assessment of `k` and `j`.

**Evidence cutoff:** 2 November 2026. A recalculation from the archived raw records; no new run.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
**Yes—this strengthens the architectural conclusion without requiring another experiment.** I independently recalculated the matched-`exec` comparison from your walker-ladder-20261102T004502Z.jsonl[raw measurement records](sandbox:/mnt/data/walker_ladder_review/walker-ladder-20261102T004501Z/walker-ladder-20261102T004502Z.jsonl), and reproduced the researcher’s numbers.

| Kernel | Change in instructions per row | Change in cycles per row |
|---|---:|---:|
| A: no software prefetch, no decoded-output stores | **+0.026%** | **+38.3%** |
| B: software prefetch, no decoded-output stores | **+0.014%** | **+36.6%** |
| P: shipped walker | **+0.024%** | **+43.3%** |

I also reproduced the reported changes in issued µops and retirement slots. This is a stronger analysis of the **same capture**, not an independent replication, but it removes an important ambiguity: the additional cycles are not accompanied by a material increase in executed instruction work.

## The most useful conclusion

We can now state the research finding quite precisely:

> **Concurrent copies make substantially slower progress through essentially unchanged traversal work. This occurs without the production job queue, decoded-output stores, segment assembly, or even software prefetch being necessary for the slowdown to appear.**

“Necessary” is doing important work there. Those exclusions do **not** mean the omitted mechanisms contribute nothing to production performance. They mean we no longer need to invoke them to explain the existence of a substantial concurrency penalty in this experiment.

The researcher is also right to preserve the distinction between *output-free* and *write-free*: A and B still update lane state. They are simplified traversals, not pure read-only kernels.

That is genuine narrowing of the hypothesis space.

## One distinction I would keep especially clear

**“Concurrency adds almost no instructions” does not mean “reducing instructions cannot help.”**

These are different questions:

- **Why does the current kernel slow down under concurrency?** A large increase in instruction count is not the explanation supported by this capture.
- **Could a different implementation run faster?** Potentially. Removing loads, stores, address calculations or dependencies could still improve it.

This is why the result is compatible with both **closing the generic scheduling-rewrite investigation** and **continuing the already-bounded assessment of `k` and `j`**.

The packed-token change removes explicit state traffic from the compact implementation. It is not trying to eliminate extra bookkeeping that appeared when the shipped walker moved from one core to four.

But its payoff remains unmeasured.

## What this means for the next architectural decision

The question is now better focused:

> **Can we change the traversal’s representation or access pattern so that concurrent execution becomes cheaper, without adding more extraction and state-management cost than we save?**

The frozen FOR-14 implementation answered **no for that implementation**. The compiled `k`/`j` changes have not answered it yet.

The ladder tells us why the opportunity remains interesting: substantial extra cycle cost persists even in the reduced kernels. It does not tell us whether compact records, simpler field extraction, fewer requests, or another approach will recover that cost.

I would therefore avoid both extremes:

**“The smaller table must eventually win”** is unsupported.

**“The shipped loop is close to M, so representation research is exhausted”** is also unsupported.

The experiment limits one class of proposed improvements; it does not establish a universal hardware optimum.

## What to do with this contribution

**Add this matched-group analysis to the closing postmortem, then move on.** There is no reason to reopen the capture, migrate the counter backend, or commission another broad explanatory report.

A short addition to Claude is enough:

> Please incorporate the researcher’s matched-`exec` comparison into the existing postmortem. I independently reproduced it: instructions per row change by approximately +0.026%, +0.014% and +0.024% for A, B and P, while cycles rise by +38.3%, +36.6% and +43.3%.
>
> State the resulting exclusion narrowly: these simplified traversals exhibit substantial concurrency cost without materially more instruction work, and without the production queue, decoded-output work or software prefetch being necessary. This does not identify the physical bottleneck or imply that instruction/state-traffic reductions cannot help a different implementation.
>
> Preserve D1 and the diagnostic’s scope. No new measurements are requested.

**We have a useful endpoint for this diagnostic now.** The next substantive decision belongs to the existing compact-candidate assessment—not to further refinement of the same postmortem.
`````
