# External research report 89: a harness-lifecycle defect, not a decoding failure (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 11:52 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** P74-C's exploratory CI.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
That's an excellent catch by Claude. This looks like a harness-lifecycle defect rather than a P74-C decoding failure, and catching it on CI before your definitive i9 run is exactly why we did the exploratory validation.

The reproduction results are persuasive:

| Harness  | Assertion failures |
| -------- | ------------------ |
| Original | 3 / 45 runs        |
| Amended  | 0 / 45 runs        |

The fix appears effective, though 45 successful runs cannot prove the race has been eliminated entirely.

The distinction between the two execution paths is important: the `--check` phase processes many files in one process, where thread teardown can overlap the `/proc/self/task` snapshot. The timed phase uses a fresh process per decode, so it has a different lifecycle.

I support committing this as a documented harness amendment, with the original frozen SHA, failed CI artifacts and reproduction results preserved.

There is one detail I'd want Claude to verify: the amended assertion must continue checking that every observed thread's `Cpus_allowed_list` is contained in the allocated CPUs. Fixing the thread-count race should not weaken the actual four-core containment guarantee.

And importantly, there's no need to repeat P74-C stage benchmarks or change any selection thresholds.

Once the amended runner passes its correctness checks and the affinity audit, I'd clear it for the i9 full-decoder experiment.

Meanwhile, the completed exploratory r3 job has already shown promising Wikidata throughput gains. The primary question remains whether those gains persist at four workers on your i9 without increasing CPU-seconds/GB or RSS beyond the registered limits.

This is good progress. Claude is resolving a real experimental-integrity issue before it can contaminate the decisive result.
````
