# External research report 153: the researcher on which top-tier questions theory can answer, and the order of work: P87, then wake placement, then compact navigation (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with reports 154–156.
- **Source:** the independent researcher of reports 138–152.
- **In reply to:** two questions from the maintainer about report 152's top tier. The first
  is quoted below. The second, about which candidate to take first, was not relayed.

> ok easy quick question: out of these, which can potentially be answered theoretically, and which empirically (by running code/experiments in practice)?

**Evidence cutoff:** 2 November 2026. A reading of the records; no new run.

It is kept exactly as received. The two replies are recorded in the order relayed.

## The first reply, as received

`````text
All three can be investigated theoretically and empirically, but only experiments can ultimately establish whether they improve performance.

| Candidate                     | Theoretically answerable                                                                                                                  | Empirically required                                                                                                                     |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| 1. P87 buffer reuse           | Prove memory lifetimes and correctness; calculate eliminated allocations, copies and theoretical savings ceiling.                         | Establish actual throughput, CPU and RSS gains, plus no regressions.                                                                     |
| 2. Wake placement             | Model scheduling, identify avoidable delays, derive upper bounds on recoverable utilization and design a work-conserving policy.          | Establish whether Linux actually schedules workers better and whether whole-decoder throughput improves without greater contention.      |
| 3. Compact `k`/`j` navigation | Derive extraction and state-update costs, compare dependency chains, calculate construction budgets and prove representation correctness. | Determine whether `k` or `j` is faster on the i9, whether smaller tables overcome extraction costs, and whether a full decoder benefits. |

## My suggested division of work

- Theory first: Wake placement and compact navigation. Both still have important questions that can be narrowed through modelling, source inspection and existing records before writing more experimental code.
- Experiments next: P87. Much of its theoretical case is already established, and the production candidate exists. Its main remaining uncertainty is empirical.
- Supporting research: The physical cause of concurrent traversal slowdown is primarily an empirical identification problem. Theory can constrain explanations, but existing counters do not isolate the cause.

In short: P87 is closest to needing a final practical verdict; wake placement needs a better policy hypothesis; compact navigation needs its remaining execution-versus-memory economics resolved. None requires a new i9 run just to make progress on the theoretical questions.
`````

## The second reply, as received

`````text
P87 buffer reuse comes first.

It has the strongest combination of measured evidence, implementation readiness and limited remaining work.

The order I recommend is:

1\. P87 — production validation first. The diagnostic already measured +1.35% throughput and −1.71% CPU cost on four-worker Wikidata. The production candidate exists and passes correctness checks. The next task is to establish whether that benefit survives in the actual decoder without regressions.

2\. Wake placement — second. This has a credible opportunity to improve utilization, but we still need to formulate one specific intervention from the scheduling evidence before implementing it.

3\. Compact navigation (`k`/`j`) — third. It has the greatest architectural ambition, but also the highest risk. The original implementation lost substantially, and the improved variants remain untimed. It deserves a tightly bounded investigation, not an open-ended optimization campaign.

One distinction: if we prioritize potential breakthrough size rather than expected return on the next investment, I would put compact navigation first. But for disciplined, sequential engineering, P87 → wake placement → compact navigation is my choice.

I'd have Claude finish P87's production-validation protocol next, stopping for your authorization before any new i9 experiment.
`````
