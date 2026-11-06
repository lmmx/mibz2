# External research report 146: the guardian on the i9 probe's success, and a safeguard: event-specific control overhead (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** the walker counter probe at 902a911 (`walker-probe-20261102T003547Z.tar.gz`) and the coordinator's freeze.

**Evidence cutoff:** 2 November 2026, after the probe, before any capture. It asks for the capture to be frozen with control-only counts reported beside low-count events, and authorizes no capture.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
**Yes—and this time it genuinely worked.** I opened the walker-probe-20261102T003547Z.tar.gz[archive](sandbox:/mnt/data/walker-probe-20261102T003547Z.tar.gz), verified its manifest and all three file checksums, and checked the raw perf output rather than relying only on the summary table.

## What passed

| Check | Verified result |
|---|---|
| Source provenance | `902a911`, with no tracked changes recorded |
| Recipe completion | Exit status **0** |
| Control channel | Raw output shows **disabled → enabled → disabled** |
| Candidate configurations | **All 30 valid**, including measured and control-only windows |
| Seven primary event groups | All counted together, with **100% reported running time** |
| Multiplexing | None reported in these probe windows |
| Packaging | Console, JSON, Markdown and manifest correctly bundled |

The control-only cycle counts were approximately **0.042–0.058% of their corresponding 20-walk windows**—comfortably below the proposed 1% cycle-overhead limit for these probe conditions.

**The launcher, ACK handling and event-group scheduling have now worked on your actual workstation.** We don't need another tooling redesign before proceeding toward the planned diagnostic.

## One interpretation safeguard to retain

Small *cycle* overhead does not guarantee negligible overhead in **every individual event**.

For example, in the `stalls_l2` group:

| `cycle_activity.stalls_l3_miss` | Count |
|---|---:|
| Twenty-walk window | 28,608 |
| Control-only window | 17,636 |

The control-only count is about **62% of that particular event’s measured-window count**, despite contributing only about 0.054% as many total cycles.

That does **not** mean 62% of the workload is control overhead. It means this relatively small event count cannot safely be interpreted without considering the control measurement.

I would have the full report display control counts alongside low-count events and avoid strong causal conclusions where they are comparable. It should not blindly subtract the control either: these are separate windows, not an exact accounting of identical overhead.

**This is an interpretation limitation, not a failed probe or a reason to restart the instrumentation project.**

## Next: freeze the capture

Send this tarball to Claude. It now has the evidence needed to select the working primary groups, record the final protocol and return the pinned capture command. There is no need to repeat the fallback and individual-event searches during every full measurement.

A short instruction is enough:

> The revised probe passed on the i9. Please archive it and freeze the walker-ladder capture using the verified primary event groups. Retain the full measurement-validity checks, including four-core overlap and control overhead.
>
> One reporting safeguard: low overall cycle overhead does not establish negligible event-specific overhead. Preserve control-only counts alongside the measurements and qualify any event whose control count is comparable to its workload count; do not automatically subtract it or use it for a strong bottleneck classification.
>
> Return the final protocol, source pin and single capture command for approval. No additional collector rewrite or performance capture yet.

**Nothing further needs running locally right now.** This was the successful preflight we needed; the next workstation run should be the actual diagnostic, once its final configuration is frozen.
`````
