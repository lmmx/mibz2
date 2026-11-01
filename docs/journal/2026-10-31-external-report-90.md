# External research report 90: P74-C cleared for its definitive i9 trial (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 12:47 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** P74-C's amended exploratory CI.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Worked for 14s

This is a major milestone. P74-C is now cleared for its definitive i9 full-decoder trial. And the exploratory results are genuinely encouraging.

Three independent CI hosts completed the amended runner successfully, with correctness, kernel-selection, thread-affinity and assembly checks passing.

Most importantly, the full-decoder results suggest that P74-C isn't merely shifting work between stages:

| Exploratory CI, different-dump Wikidata | Result             |
| --------------------------------------- | ------------------ |
| 1 worker, throughput                    | +4.7% to +5.7%     |
| 2 workers, throughput                   | +2.3% to +4.0%     |
| 1 worker, CPU-s/GB                      | 4.1% to 4.7% lower |
| 2 workers, CPU-s/GB                     | 3.3% to 4.4% lower |

That's exactly the combination we've been pursuing: more throughput and less CPU work per decoded gigabyte.

The outstanding questions are whether the gain survives four workers on your i9 and whether the strict RSS criterion holds. The small-control RSS regressions on CI are real measurements worth investigating, but they do not invalidate the throughput and CPU-efficiency gains.

### What I'd do next

1\. Run the i9 trial when you're back at the workstation.

Use Claude's exact pinned command:

```bash
cd ~/dev/mibz2
git fetch origin
git switch research/p74c-decoder
git pull --ff-only
git rev-parse HEAD
# Must print 3870c1225d4cfb1a87ade57651eb1165ce451692

just bench-p74c-decoder i9-10900K
```

Keep the machine relatively quiet. This is the primary four-physical-core experiment, expected to take roughly 30–60 minutes. The runner writes to `bench/results/p74c-decoder/i9-10900K/`.

2\. Give Astra the BWT evidence pack whenever convenient.

I've confirmed the README is accessible at the specified research commit:

[BWT room-walk machine-code evidence pack](https://github.com/lmmx/mibz2/blob/c4018b54594f2ba7a6e50ad364b328f53a8980f6/docs/research/2026-10-31-astra-room-walk/README.md?utm_source=chatgpt.com)

Ask Astra to inspect the entire directory, including the disassembly and source extract. Its assignment is read-only diagnosis, not implementation.

3\. Preserve the new experimental record.

Claude has handled the two harness defects correctly: neither was disguised as a performance result, and the subsequent amended CI run validated the corrected runner while preserving the original evidence.

One operational recommendation: after checking out the research branch, leave it at the exact pin until the i9 trial finishes. No pulls, edits or branch updates between SHA verification and execution.

### The RSS question

I wouldn't dismiss the 2–8% increases on small controls. Claude's explanation—that faster workers allow additional decoded blocks to accumulate in the output queue—is plausible, but unverified.

If that is the mechanism, the right eventual fix may concern queue occupancy rather than the MTF implementation. However, we should not change buffering for this frozen trial. Let's first see the i9 results.

If the four-core throughput and CPU-efficiency gates pass while RSS fails, we'll have learned something important: P74-C may be economically worthwhile, but the production system needs an explicit memory trade-off or a separate buffering improvement before promotion.

My recommendation: proceed. We've now demonstrated positive end-to-end effects on three exploratory hosts, and the definitive i9 trial is ready. This is the strongest position P74-C has reached so far.
````
