# Labelled construction comparison — frozen before candidate data

Direct user start, 2026-11-11 15:21 UTC; master
2635f04e9155dd31388b24fd3f7248f87b3ddcda. Relay6798202916,
scheduling clarification6799687151, manual-start receipt6799750846.
Formal inbox reconciliation is pending; earlier assignments remain pending.
End implementation/analysis by 17:21 UTC, aggregate candidate execution at most
30 minutes. One pass and one exact reproduction; one targeted correctness
correction permitted, preserving the failure. No cost-driven retries.

Sources: unchanged phrase_induction.py and batch_induction.py from
af3327d8aee9fffe26d2d6de76818b6ac69cc145, all five supplied relay files, and
PR369 at85735b38ebfa804acd31a6291bb7c08b6bf57760. Both supplied checks have now
reproduced byte-for-byte, twice. Their source/input identities stay separate
from candidate results. No real blocks or vendor benchmark entrypoints run.

Questions / decisions:

1. Can occurrence marking during the existing delimiter constructor replace
   its separate origin search without changing its section or token-LF claim?
   Compare instrumented search and mark variants with the unchanged original
   constructor and an independent stable-row LF oracle. Count marker tests,
   origin lookups/steps, constructor work including declined cycles, words,
   metadata and output. Shared word IDs must never identify occurrences.
2. Can two stable interval orders avoid packed-coordinate rebuilding while
   retaining exact non-adjacent splice and coalescing across removed gaps?
   Implement one deterministic implicit-treap representation with stable edges,
   shared word DAG and separate origin occurrence. Preserve splits on genuine
   source/image boundaries. Selection remains a global ordered overlay and is
   charged; only contraction maintenance is local. Compare with the unchanged
   batch constructor plus a faithful instrumented packed implementation.
   Count sorting/comparisons, searches, rank/parent hops, tree writes/rotations,
   selection, splits, merges, copied/rebuilt records, word operations, live and
   cumulative persistent metadata, transient arrays and output separately.
3. Only if supported by (1)/(2), consider one repeated equal-width two-port
   translation component, with exact route/word/marker contract and equality
   cases. Otherwise defer; do not add an experiment to manufacture support.

Fixed new input scope: every binary column length1..6, ternary length1..4,
all origins, both/all three delimiter choices (including absent delimiters).
Fixed synthetic sizes m=1,2,4,8,16,32,64: raw b^m a^m, b a^m and (ab)^m;
cyclic BWTs of (ab)^m, (aba)^m, (abc)^m, (aab)^m, all origins and symbols
from that family's alphabet. No whole PR369 baseline rerun.

Local contraction schedules: exactly four original color-seed rounds1,2,3,4
per column/origin, keeping the marked row. Freeze actual D intervals using
only the original constructor before either candidate comparison; save all
input columns, origins, seeds, retained-row identities and removed original
row IDs in schedules.json. Both implementations must take those exact sets.
Charge selection even though the schedule also acts as a test oracle.

Regressions: ba/baa, bba origin1 delimiter a (adding origin breaks token LF),
ab gray-only cycles, bab nondividing cycle, long-excursion copy obstruction.
From PR369's102 expensive cached updates, select the first binary witness
by (length,column,black mask,removed run): babaaa, black runs[0,1,3], remove
run0; cached5 visits versus fresh4, deletion non-adjacent. This is recovered
baseline evidence, not a new candidate finding. Exercise its already-induced
map with a single frozen splice: retain marker row1, color only the one-letter
b word1 and all other words0, hence delete original row0. Compare the unchanged
batch implementation under that explicitly supplied color function. Include
its saved witness/prior counts; do not extrapolate to all102 cases.

Any destination, itinerary, time, origin or reconstructed-byte mismatch rejects
that candidate pending the one correction. Exact outputs must also preserve
the delimiter token-ordering claim. Report componentwise trade-offs, not an
arbitrary combined score. Pursue only a named restricted case showing reduced
targeted maintenance/search work with its induced costs disclosed; otherwise
revise/stop that candidate. Counts do not qualify speed, safety or integration.
No graph/alternation extension, production edits, policy changes, workstation
jobs, merge/release, other-role state edits or independent self-certification.
