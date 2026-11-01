# External research report 129: the guardian on the researcher's addendum: keep the two records separate (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 16:40 UTC.
- **Source:** the guardian.
- **In reply to:** report 128.

**Evidence cutoff:** 1 November 2026, before any timed P87 experiment: a proposal and its review, not a performance result.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
I've read the researcher's record, and I think it's an excellent companion to our consolidated history.
What makes it valuable is that it serves a different purpose. Our record explains the technical investigation and how we arrived at P87. The researcher's addendum preserves the standards of evidence behind that decision, including why certain attractive conclusions were not accepted.
Three aspects stand out to me.
First, it establishes exactly what we know. The additional 0.609 CPU-s/GB is attributable to the whole-corpus walk-plus-assembly phase, not exclusively pointer chasing. Likewise, six outstanding requests and a 98% software-prefetch share remain interpretations, not independently established measurements. These distinctions will matter when somebody revisits the work months later.
Second, it preserves the intellectual independence of Astra and Claude. Astra proposed a controlled change to the real decoder. Claude proposed an ambitious investigation of interference between concurrent memory workloads. The researcher explains why P87 was selected first without suggesting that Claude's alternative was inferior research or that the larger architectural question has been answered.
Third, it carefully defines what a future P87 result can prove. A successful intervention may demonstrate a worthwhile optimization without identifying every underlying cache mechanism. Conversely, an unsuccessful intervention cannot disprove memory contention generally.
I particularly like this principle from the addendum:
A plausible numerical model, a strong correlation and a successful intervention are three different things.
That is probably the most important methodological lesson from the entire investigation.
How I'd preserve the records
I'd retain them as two separately attributed documents:

* Guardian's consolidated technical history: architecture, measurements, competing hypotheses, experiment designs, review corrections, decisions and chronology.
* Independent researcher's addendum: independently verified findings, objections to unsupported conclusions, reasons for selecting P87 and the limits of future causal interpretation.

I would not merge them into a single consensus document. The fact that two independent accounts agree on the decision while emphasizing different aspects of the evidence is itself worth preserving.
One small editorial improvement: both documents should prominently state their evidence cutoff—1 November 2026, before the P87 timed experiment—so a future reader doesn't mistake the proposal for a completed performance result.
And the researcher's final observation is especially apt:
Selection is not resolution.
That's exactly where we are. We've selected the next experiment, not established its outcome.
My recommendation: preserve the addendum verbatim alongside our main record. No further review is needed before Claude continues preparing P87.
`````
