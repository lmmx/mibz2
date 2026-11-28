claim: forward-emission-run
by: claude-code
outcome: blocked
evidence: #1168 (merge 022e94dc); #1231 (queue entry); tag ready/bench-forward-lanes at 022e94dc

## State
The re-pinned kill test (registration-disjoint-slabs.md: the #1217 disjoint-slab walk against
the default build, both at v0.5.17 01acd74d) merged in #1168. `ready/bench-forward-lanes` is set
at 022e94dc (confirmed by ls-remote) and the queue entry merged in #1231. Integration hold released.

## Blocked on
The i9 bundle only (`just workstation-queue`). Nothing else to decide.

## Next (whoever reads the bundle)
Read it from `origin/evidence` at the commit #145 names and check its SHA-256. Record the §3
verdict in a delta. PASS -> open a C6 adoption registration making the walk the open kernel's
default (a diagnostic PASS is not adoption). RESCUE -> vectored phase B
(`forward-rle1-crc-vectored-build`), not a re-run. KILL -> dead-end row naming this arm and the
bundle digest. Then update `C-A6c`/`E-walk` in surface.tsv and mark this claim done.
