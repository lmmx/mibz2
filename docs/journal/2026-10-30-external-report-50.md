# External research report 50: the guardian on the shipped-side correction (verbatim)

The guardian's message, relayed by the maintainer after the reconciliation's post-#114
shipped-side figures.

**What it says:**
- **More bad news for N1's construction economics,** though not a definitive failure. The
  worsening traffic ratios are model corrections, not measured runtime regressions; bytes
  don't translate directly into time.
- **P^{2|4} is the only version worth serious consideration:** 1.66–2.20× the shipped
  build's modelled traffic, against 3.6–4.3× for full P⁸. The 6% shipped-build correction
  moves the whole-corpus threshold only from about 8.8 to 8.6 cycles per record.
- **N1's central advantages remain predictions.** There is no measured construction speed,
  no measured walk speed and no validated retained-memory solution.
- **Gate B is no longer the default after correctness passes.** The reconciliation must
  first show a credible path for P^{2|4} on construction plus walk, CPU efficiency and
  high-water memory.
- **Memory:** the fallback must avoid the allocation before it occurs. Choosing the shipped
  path after building a large table doesn't solve it. If no such plan is credible without
  expensive selection work, close N1 without Gate B.
- **If both answers survive,** a small, independently pre-registered builder test might
  still be justified, as a falsification experiment, not an implementation commitment.
- **The verdict:** N1 is close to a research stop. The burden of proof is on N1. There is no
  meaningful turnaround yet.

**The coordinator's action:** the reconciliation was asked, within its task 2, whether a
rule using only what is known before construction (r from the L scan, n) can keep every
N1 block under a fixed budget. It reports the false-fallback rate, the bytes kept on N1 and
the worst-case overshoot. Nothing new was started.

## The message, as received

More bad news for N1's construction economics, I'm afraid. Not a definitive failure, but
the new evidence moves the case further against it. I wouldn't try to dress this up as
progress toward a breakthrough.

There is one important qualification: the worsening traffic ratios are model corrections,
not measured runtime regressions. The current one-pass shipped builder is a stronger
baseline, but the extra bytes N1 moves don't translate directly into proportional
execution time.

Where N1 stands now

Question	Current evidence	Assessment
Can partial Ψ upgrades be correct?	Checker still running	Unresolved
Can the builder achieve roughly 9 cycles per record?	Untuned historical estimate: 16–27	Difficult, not disproven
Can construction plus walking beat shipped?	More construction traffic; stronger shipped baseline	Increasingly doubtful
Can it preserve peak RSS?	P⁸ exceeds shipped on high-r blocks; partial-plan accounting pending	Serious risk
Can it win at four workers?	Original large contention advantage was based on flawed assumptions	Unsupported

The important distinction

The new traffic comparison is especially unfavorable for full : roughly 3.6–4.3 times the
shipped build's modeled traffic at the median.

But is materially less bad, at 1.66–2.20 times. That's still the only version I would
seriously consider.

And the current-master timing correction is smaller than the traffic figures initially
suggest. If the reported 6% shipped-build improvement transfers across runners, the
whole-corpus pass threshold moves from approximately 8.8 to 8.6 cycles per record.

That's a tightening, but not a collapse. The difficult part was already achieving a
builder that cheap.

The larger issue is that the research has no measured N1 construction speed, no measured
N1 walk speed, and no validated retained-memory solution. We have repeatedly refined the
models, but the candidate's central advantages remain predictions.

My recommendation has changed

I would no longer make Gate B the default next step once correctness passes.

I'd first require the completed reconciliation to demonstrate a credible path for on all
three constraints: construction-plus-walk performance, CPU efficiency and high-water
memory.

In particular, if a worker must retain more memory after processing a high-rank block, N1
needs either a bounded-allocation strategy or a fallback that avoids that allocation
before it occurs. Simply choosing the shipped path after constructing a large table
doesn't solve the problem.

If that memory plan cannot be made credible without adding expensive selection work, I'd
recommend closing N1 without Gate B.

If it can, the roughly 8.6-cycle whole-corpus target is challenging enough that a small,
independently preregistered builder test might still be justified. But that's a
falsification experiment, not an implementation commitment.

Is there anything positive here?

Yes, but only in a limited sense.

The correction confirms that is significantly more economical than deeper tables. The
existing partial-depth approach is therefore the right place to concentrate any remaining
analysis.

Also, the shipped decoder has improved. The one-pass build is a real production advance,
even though it makes the speculative N1 replacement less attractive.

My current verdict: N1 is close to a research stop, but the final two outstanding answers
matter. If partial-upgrade correctness fails, or if retained-memory accounting rules out ,
I see little reason to spend more effort on it.

If both survive, there's still a narrow economic question worth measuring. But the burden
of proof is now firmly on N1.

So to answer your question plainly: no meaningful turnaround yet. This is more unfavorable
evidence, and I share your skepticism.
