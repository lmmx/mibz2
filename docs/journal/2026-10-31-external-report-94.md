# External research report 94: one amended timing experiment for research/walk-unroll (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 17:16 UTC.
- **Source:** the maintainer's authorisation.
- **In reply to:** the walk-unroll structural audit.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
I've reviewed "research/walk-unroll" at "5711606" and the structural audit in "docs/journal/2026-10-31-walk-unroll/audit.md".

I authorize one amended timing experiment, with no further kernel optimization.

Preserve the original registered structural-stop verdict: S3 and S5 failed as written. Do not relabel them as passes.

Before any timing, commit an explicit amendment that:

1. Corrects S5's stack-access accounting so the shipped and unrolled implementations count accesses to stack-resident lane state consistently, including indirect addressing through a register.
2. Records S3's actual 2.16 stores per lane step as an accepted structural deviation caused by approximately five per-round register spills, rather than pretending the original 2.1 limit was met.
3. Establishes that the intended constant-index transformation is present: 32 T loads per round, 10-instruction ordinary steps, no lane-loop bookkeeping, no additional calls, and unchanged prefetching.
4. Preserves all original performance criteria, inputs, correctness requirements, paired schedule and stop rules.

Freeze the corrected audit, candidate and harness before timing. Then run the one authorized three-job exploratory CI measurement on Zen 3 where available, preserving each host's CPU identity and binary provenance.

Report complete room-walk costs, paired intervals, cycles/byte and ns/byte, along with the registered performance verdict. Do not use my i9, retune the candidate, change production decoding or merge.

If the implementation does not establish the registered minimum useful improvement, archive this arm as a negative or inconclusive result as appropriate.

Keep this completely independent of the frozen P74-C full-decoder experiment.
````
