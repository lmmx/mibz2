# The rule, frozen before any data (RULES C1a: diagnostic; it decides nothing about production)

*Written 25 November 2026, before any run of this design. A change after the data is a new registration with its reason.*

**For each part P and count k,** its room is the wall with P alone ideal over all real, a geometric mean of the paired ratios with its 95% t-interval. Its label:
- **ROOM** iff the interval's upper bound is ≤ 0.95: making P free would save at least 5% at k;
- **NO ROOM** iff the interval lies within [0.98, 1.02];
- **MASKED** iff P has NO ROOM alone but its interaction with some part Q (the 2fi on log wall) has an interval wholly below 0, and P is ROOM with Q ideal. Its room appears only behind Q;
- **UNRESOLVED** otherwise.

**For each pair P, Q and count k,** the interaction (log scale) is labelled:
- **INTERACTS** iff its interval excludes 0;
- **ADDITIVE** iff the interval lies within ±0.01;
- **UNRESOLVED** otherwise.

**The floor** (all ideal over all real) at each count is reported, as the decoder's residual: memory, synchronisation and the reader.

**VOID** (the whole run) if any of these:
- any arm's output is not byte-identical to the shipped decoder's;
- a removed part shows more than 5% of its real busy time in its ideal arm's ledgers;
- the machine is not the i9-10900K;
- the pin is broken in more than 10% of traced decodes (2b's PIN-BROKEN label, here a VOID).

**INCOMPLETE** if any identity fails in a traced decode.

**Reported beside the labels,** never part of them: the oracle's memory and handover cost; the clock split; the unpinned link.
