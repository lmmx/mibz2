# External research report 99: the walker's exploratory full-decoder results: Zen-specific (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 23:06 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** CI run 41243689201.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
This is a major result for the BWT walker—and a very useful warning against enabling it universally.

The replacement exploratory CI run succeeded, the lifecycle correction worked in real conditions, and we now have complete-decoder evidence across three AMD generations plus the earlier Intel host.

### What the results show

The encouraging part is that the stage-level optimization survives the real decoder on some architectures—spectacularly on Zen 5 and convincingly on Zen 3.

But the `records` regression is equally important:

* Zen 3: approximately 5.2% slower

* Zen 4: approximately 8.3% slower

* Intel Xeon: approximately 7.2% slower

* Zen 5: approximately neutral

These are not just statistical noise. On three hosts the regression intervals exclude the preregistered 3% tolerance.

So we have a genuinely valuable optimization, but not yet a safe universal replacement.

### My recommendation

Run P74-C v1.2 on the i9 next. Do not modify the walker or run its i9 trial yet.

The P74-C harness has passed targeted lifecycle tests, three CI integrity jobs, 272 local correctness checks and a 2,016-child synthetic test. That is a solid basis for proceeding with its already-registered experiment.

P74-C — i9 command

 Copy

```bash
cd ~/dev/mibz2
git fetch origin
git switch research/p74c-decoder
git pull --ff-only
git rev-parse HEAD
# Must be:
# 33e0618b1e70190f4e8390eb69ebe2778142fc77

just bench-p74c-decoder i9-10900K
```

Verify the full SHA before starting. Keep the machine otherwise idle for the registered four-core measurement.

### What to do with the walker

I would preserve it as a successful architecture-dependent research candidate, not archive it as an outright failure and not promote it into production.

The important remaining question is what explains the `records` slowdown. Potential explanations include different segment-boundary behaviour, prefetch effectiveness, instruction-cache pressure from the much larger unrolled function, and the accepted decoder inlining difference. None has been isolated yet.

A future targeted investigation could distinguish those mechanisms. But I'd resist launching it tonight. We have already collected enough evidence to establish that the current implementation behaves differently across workloads and CPUs.

There is also a crucial experimental distinction: the walker trial's formal deciding run is still the i9 four-worker experiment. The exploratory CI results cannot replace that verdict. Equally, its existing control regressions should remain visible even if an eventual primary throughput result passes.

### Direction for Claude

Excellent. Please close the second walker exploratory CI run as completed and preserve all its records unchanged.

I accept the harness-lifecycle validation and the reported cross-architecture results. The walker remains a successful stage-level optimization with mixed full-decoder evidence: strong Wikidata gains on Zen 5 and Zen 3, limited gains on Zen 4, neutral Intel Wikidata, and significant records regressions on three hosts.

Do not retune the walker, change any gates, introduce CPU dispatch or initiate another benchmark. Keep the i9 walker trial unrun and the candidate research-only.

Our immediate priority is the frozen P74-C v1.2 i9 trial at 33e0618b1e70190f4e8390eb69ebe2778142fc77. No further changes to that trial are requested.

After the P74-C result, we'll decide separately whether the unrolled walker warrants its registered i9 experiment or a new, explicitly defined research hypothesis. Preserve the evidence for both possibilities.

The bigger picture is encouraging: the Astra-guided transformation achieved a measured 17.9% full-decoder throughput improvement on Zen 5 without changing the underlying BWT algorithm. That's a substantial technical result even if this exact implementation never becomes the universal default.

For tonight, though, the most valuable next measurement is P74-C on your own i9.
````
