claim: explore
by: claude-code
outcome: done
evidence: #1194, #1197, #1200, #1202; frontier/claims/low-thread-open-unification.toml; frontier/claims/post-1080-open-admission.toml

**What was asked:** integrate the four open `explore` PRs (#1194, #1197, #1200, #1202) that `brief` flagged as "multiple open PRs for claim", leaving any whose author was still active to that author.

**What I found:** all four were smooth-zone in scope (journal, a standalone `examples/` probe, frontier data; zero `src/` edits), with green CI and pre-commit.ci on their heads. Their only conflicts were in `frontier/dead-ends.tsv`, all append-only, and their `surface.tsv` edits touched different rows (E-spelling, P, M). The new dead-end rows follow the scoped-STOP format, and the three PASS discoveries were already filed at `idea`: `lanes-disjoint-slabs-build`, `parallel-decoder-pool-and-sysfs-cache` (with an owner-only `needs_louis` on the public API shape) and `runs-front-and-fill-build`. While I checked, the authoring session merged #1200 (17:50Z) and #1194, and rebased #1197 and #1202 onto the new master (17:52-17:56Z). Since that session was plainly still working, I merged none of them myself and left #1197 and #1202 to it.

**What changed:** #1194 was a STOP that deliberately opened no idea claim, but two of its findings are discoveries other work depends on, so I filed them:
- `low-thread-open-unification` (idea, P3): with the forward four-lane walk (#1110), OpenCells beats Cells at t = 1, 2, 3 (-2.0..-10.9% base, -9.1..-14.4% JCC-32B, 0.44-0.61x RSS). It is gated on `forward-emission-run`; do not admit it before then.
- `post-1080-open-admission`: a memory caution (the two-tier arm refuses 9 wikidata blocks and drags in the `Segmented` fallback, +7.8..+15.7 MiB at t = 1..3) and a link to the journal.

**What's next:** #1197 and #1202 belong to their author's integration. If either sits idle past about 30 min with a green head, integrate it under take-review.

**Caveats:** I re-ran none of the measurements; the figures are the PRs' own.
