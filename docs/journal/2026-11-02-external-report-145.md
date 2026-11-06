# External research report 145: the guardian on report 144: an in-process collector as the likely longer-term direction, not an interruption of the probe (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** report 144.

**Evidence cutoff:** 2 November 2026. No new run or rewrite authorized; the probe at 902a911 only.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
**The researcher’s assessment is sound. Your question identified a genuine architectural alternative—not a misunderstanding.** The useful distinction is not “CLI versus modern library”; it is **external control of a measurement window versus control from the code that owns that window**.

I checked the `perf-event2` documentation: it supports grouped enable/disable operations, structured reads, and requests enabled/running-time metadata. It also supports processor-specific raw event configurations. Those are the essential capabilities our proposed Rust collector would need. [Docs.rs](https://docs.rs/perf-event2/latest/perf_event/struct.Group.html)

## Where I agree most strongly

**An in-process collector is a more natural fit for repeated measurements of particular Rust functions.** That is an engineering judgment, not a claim that it would automatically produce more accurate results.

The benchmark already knows when setup finishes, when walking starts, and when validation should happen. Giving it ownership of the counter group could remove the FIFO communication layer entirely. The documented library interface supports exactly that pattern: configure the group beforehand, enable it around the workload, disable it, then read the results. [Docs.rs](https://docs.rs/perf-event2/latest/perf_event/struct.Group.html)

But the researcher is also right about what it would **not** fix:

**Permissions remain.** Direct library access is subject to Linux’s performance-monitoring access controls, just as the external tool is. [Linux Kernel Documentation](https://docs.kernel.org/admin-guide/perf-security.html)

**Counter constraints remain.** An incompatible or oversized group still cannot be scheduled simply because Rust requested it. [Docs.rs](https://docs.rs/perf-event2/latest/perf_event/struct.Group.html)

**Event definitions remain our responsibility.** Raw configurations are processor-specific; preserving an event’s name in a report is insufficient unless the underlying configuration and counting scope match. [Docs.rs](https://docs.rs/perf-event2/latest/perf_event/events/struct.Raw.html)

## What I would do now

**Finish the already-prepared probe before commissioning a replacement.** That recommendation is about obtaining useful evidence from a completed repair—not defending the FIFO design because we have already invested in it.

The next decision should be explicit:

| Situation | My recommendation |
|---|---|
| The repaired probe works, and the collector satisfies the diagnostic’s measurement requirements | Complete this bounded diagnostic with it, after the remaining protocol review and freeze. |
| Further control or measurement-boundary problems appear | Reassess the collection architecture rather than automatically adding another patch. |
| We decide to reuse phase-level PMU measurement across many future experiments | Favour a small in-process Rust backend, subject to an equivalence check. |

We should not build two complete benchmark systems. A future migration should replace **only counter configuration, control and reading**. The workload kernels, correctness tests, orchestration, archive format and scientific decision rules should remain independently identifiable.

For that migration, I would require agreement on event encodings, thread/CPU scope, user/kernel policy and group membership; checks for unsupported and unscheduled events; and a measured control-overhead budget. Cross-checking against `perf` would be useful, but we should seek agreement within measurement variability—not bit-identical cycle counts from separate executions.

## The practical conclusion

**Your library idea is worth retaining as the likely longer-term direction if this becomes reusable infrastructure.** We should have considered it explicitly as the harness grew.

For now, however, the immediate instruction remains unchanged: **run only the reviewed counter probe at `902a911`, upload its tarball, and leave the full walker capture unauthorised.** This report does not require you to run anything additional or ask Claude to start another rewrite.

The aim is simpler, trustworthy measurement—not another research project about building measurement tools.
`````
