# External research report 100: P74-C's i9 full-decoder result: NEGATIVE, with decoder-level gains (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 01:06 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** the P74-C i9 trial.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
```
## The i9 full-decoder result is in

Registered verdict: NEGATIVE

But there is some genuinely valuable progress here: P74-C improved whole-decoder throughput on the new Wikidata dump by approximately 3.6%, while reducing CPU consumption per GB by approximately 1.6%.

I inspected your uploaded archive, not just the terminal summary. The 903-second timing run completed, all correctness and affinity checks passed, and the results were archived successfully.

New-dump throughput

# +3.57%

Point estimate, 4 workers

CPU-seconds/GB

# −1.60%

New dump, 4 workers

Primary lower bound

# +1.19%

Required ≥ +2%

Here are the complete four-worker throughput results:

| Corpus                    | Throughput change | 95% interval     |
| ------------------------- | ----------------- | ---------------- |
| Wikidata, new Oct 26 dump | +3.57%            | +1.19% to +6.01% |
| Wikidata fitted           | +1.27%            | +0.37% to +2.19% |
| Wikidata held-out         | +1.86%            | +0.56% to +3.18% |
| Random                    | −1.32%            | −3.11% to +0.51% |
| Records                   | −2.79%            | −6.59% to +1.16% |
| Mixed independent         | +1.01%            | +0.36% to +1.66% |
| Text                      | +1.18%            | −1.61% to +4.06% |

The primary Wikidata result is statistically above no improvement, but its lower bound does not establish the registered minimum 2% gain.

The more serious acceptance issue is memory: the `mixed-independent` two-worker RSS result shows a paired estimate around +2.99%, with its confidence interval entirely above the +1% threshold. That is why the run receives an overall negative classification, rather than simply inconclusive.

There's an interesting complication worth examining: that same row's registered maximum-over-rounds RSS rule actually passes. This isn't necessarily an error—the maximum and paired-distribution rules answer different questions—but it means the negative verdict depends on how the protocol combines them.

## What I think this means

We have now demonstrated a real full-decoder improvement on the intended four-core i9 configuration. That's a substantial step beyond the earlier MTF-stage evidence.

The run also confirms:

- All output hashes, thread-affinity checks and kernel-selection checks passed.
- The A/A calibration passed.
- The three Wikidata datasets all improved at four workers.
- The primary corpus's CPU-seconds/GB result passed its strict gate.

However, the controls and RSS criteria did not all pass.

The most interesting RSS discrepancy is:

| Mixed independent, 2 workers    | Result           |
| ------------------------------- | ---------------- |
| Shipped maximum RSS             | 26,312 KiB       |
| P74-C maximum RSS               | 25,512 KiB       |
| Maximum-over-rounds comparison  | 3.0% lower       |
| Paired geometric-mean RSS ratio | 3.0% higher      |
| Paired 95% interval             | +2.11% to +3.87% |

These figures can coexist because the largest observation in each arm needn't occur in the same round. It is therefore possible for the maximum comparison to pass while the paired comparison indicates a consistent relative increase.

The frozen protocol classifies the latter as negative. We should preserve that verdict, but investigate and describe the discrepancy accurately rather than simply saying that P74-C increases peak RSS.

The random four-worker maximum-RSS comparison also reaches approximately +3.5%, although its paired distribution is much closer to parity.

I would not authorize another P74-C benchmark or a threshold adjustment. We agreed to one confirmatory full-decoder experiment, and it has produced its answer.

What I would authorize is a read-only postmortem of the existing archive: examine the RSS distributions, the relationship between corpus size and regressions, and the three-worker versus four-worker scaling. No new experiments are needed to do that.

The decision about shipping remains separate. The measured Wikidata benefit is real, but the implementation has not earned automatic promotion under the frozen criteria.

My recommendation: ask Claude to record this as a negative registered trial with meaningful decoder-level gains, preserve the full archive, perform a bounded read-only interpretation of the RSS and control failures, and then return engineering effort to the BWT walker.

This isn't the 1.5–2× breakthrough we're aiming for, but it is meaningful evidence: a roughly 3.6% four-core improvement on new Wikidata data, with lower CPU consumption, from a single isolated MTF optimization.
```
````
