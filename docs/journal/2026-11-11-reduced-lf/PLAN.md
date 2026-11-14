# Bounded reduced-LF experiment (before execution)

Base: `2635f04e9155dd31388b24fd3f7248f87b3ddcda`; branch
`research/codex-reduced-lf`. Direct user research request, independent of the
Codex production inbox and the queued first-return real-block probe.
Pre-data coordination: [#145/6794938121](https://github.com/lmmx/mibz2/pull/145#issuecomment-6794938121).

Question: do black-run deletion and gray endpoint placements suffice for
weighted LF first returns, and can exact nested induction save gray visits?
Reject each naive construction on its first discrepancy in time, itinerary,
destination or bytes. Check a refined interval construction against an
independent row oracle; count construction, labels, origins and expansion.
Counts are not a speed claim or a decoder adoption gate.

Fixed exhaustion: all binary words/columns of lengths 1–7 and ternary of
lengths 1–5; both raw columns and cyclic BWTs, every nonempty run subset,
every origin. Check each single-run deletion from every retained subset.
Check empty retention separately. Explicit periodic BWTs use bases `a`, `ab`,
`aba`, `abc`, `aab`, repeated 2, 3, 5 and 8 times, all subsets/origins.
Two count families use m = 1, 2, 4, 8, 16, 32, 64: aligned column b^m a^m,
retain a; long-excursion column b a^m, retain b. No timings, real blocks or
workstation execution. Exit after these finite suites and one reproducibility
rerun; fix model defects if assertions fail, preserving the failure record.

HAL access is blocked. The directly accessible author sources are Paper I
(arXiv:1705.01641v2, §3.4) and Paper II (arXiv:1801.05973v2, §§2, 5).
Do not represent those as direct verification of thesis pp. 70–73/Chapter 5.
