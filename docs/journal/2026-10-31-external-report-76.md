# External research report 76: the P74-C research handoff of 31 October (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 02:36 UTC.
- **Source:** a handoff the maintainer relayed (Codex Cloud's implementation of P74-C).
- **In reply to:** research/p74-c-implementation at bb17a24.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
## P74-C research handoff — 31 October 2026

While you were unavailable, Codex Cloud implemented and experimentally evaluated P74 Option C on a separate research branch.

**Repository:** `lmmx/mibz2`  
**Branch:** `research/p74-c-implementation`  
**Latest commit:** `bb17a240ccc9967a60528d72008279e1dc41e126`

Please fetch and inspect that branch and its research documentation before doing further work.

### Implementation

Codex implemented T0/T1/T2 plain and checkpoint-aware MTF research arms, including immediate P74-to-shipped handoff with the frozen selector. Production dispatch and master remain unchanged.

Correctness tests pass. The original strict assembly gate failed by +1 instruction per ordinary low-rank iteration in both variants; an explicitly documented amendment permits performance investigation without declaring that gate passed.

### Preliminary measured result

Codex ran one authorized exploratory experiment on an AMD EPYC 9V74 Zen 4 KVM cloud VM.

On fitted Wikidata:

- Plain T2 versus shipped T0: **23.62% less MTF-stage time**.
- Checkpoint T2 versus shipped T0: **26.54% less MTF-stage time**.
- All 12 paired checkpoint rounds favored T2.
- T1 checkpoint was 3.27% faster than T0.

Random and records were near parity, but random checkpoint did not establish the registered 3% upper regression bound.

**Critical limitation:** CPU steal time was 6.71%, above the predeclared 1% threshold. The experiment is formally inconclusive as performance evidence. These are MTF-stage measurements, not whole-decoder results.

The report, raw data, protocol, frozen corpus hashes and reproducible harness are all committed on the branch:

`docs/journal/2026-10-31-p74-c-exploratory/`

### Requested next step

Review the Codex implementation and experiment for methodological or correctness issues, then plan independent replication on a better-controlled runner.

Do not discard the Codex results, but do not treat their nominal confidence intervals as qualification evidence.

Priorities:
1. Verify that the measured T0/T1/T2 calls and units are correct.
2. Identify an appropriate runner and its actual CPU topology, affinity and interference characteristics.
3. Preserve the frozen selector and performance thresholds.
4. Reproduce the same stage experiment with the same corpus hashes and balanced paired schedule, after explicit runner authorization.
5. Obtain independent Wikidata data for out-of-sample selector validation.

Do not run benchmarks on the i9 without explicit authorization. Do not start unrelated optimization experiments, change production dispatch, merge master or modify PR #117.

Report your proposed execution environment and plan before launching new performance workloads.

**Research objective:** determine whether the observed ~26.5% checkpoint-stage time reduction survives controlled replication, whether negative controls meet the 3% regression requirement, and whether a separate end-to-end decoder experiment is justified.
````
