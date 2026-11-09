# Run-fed construction: source audit and ranked proposals

Assignment [6729426658](https://github.com/lmmx/mibz2/pull/160#issuecomment-6729426658).
Source pin: **187c0ba517c3324aa28c5e7ccd2667f6c0572145** (#269).
Spec: **a4f97714cc14366984015869d9d094d89488ac57**.
Status: in progress; source-derived findings only. No implementation, compilation or timing.

## Findings established at the pin

1. The run emitter and plain MTF share the two-register list kernel; checkpointed MTF has a different memory-list kernel. The indicative emitter/checkpoint ratio bundles kernel choice, column stores and checkpoint work. A matched modern-list checkpoint control is the cheapest way to distinguish removable checkpoint work from a simpler kernel improvement.
2. The wide fill writes W = sum over cells of 8 ceil(length/8) entries, not n entries. Thus n <= W <= n + 7m, where m <= 2r-1. The logical table remains 4n bytes. Overstores are safe by the serial overwrite order; they cannot be carried across independently executed share boundaries.
3. fill_rows is a scalar append interface. Tests append shares serially. This proves neither parallel AVX2 ownership nor pipelined performance.
4. Q2 prices emit+sort on the leader. A negative Q2 does not falsify emit on the leader with sort elsewhere. Q1 likewise tests one worker. Preserve their verdicts but constrain their scope.
5. There is a sort-free proposal: assign contiguous L-record ranges to helpers, compute per-helper byte LENGTH totals, and prefix them. Every run's F destination follows directly. Fill each disjoint F interval while locating its L-byte intervals through predecessor queries over the original records. This eliminates the F-order [a,p] materialization, at the price of predecessor lookup, less monotone stores, and ownership/false-sharing work. No constructor has been implemented.

## Direct-placement proof sketch

For helper h and byte c, let L_h[c] be the sum of lengths of c-runs owned by h. Ranges are ordered in L. Let C(c) be the count prefix, and t_h,c the lengths of c-runs already processed within h. A run (c,p,length) starts at:

a = C(c) + sum_{g<h} L_g[c] + t_h,c.

Helpers process their own runs in L order. The resulting F intervals are disjoint and tile each byte bucket, provided the original L tiling and aggregate byte lengths have been validated. At q in [a,a+length), the high field is p+q-a. The low field is L[q], found in the ORIGINAL run list, not the source run's c. Intersect that F interval with the L runs and store each exact intersection. Their total number remains m, even though visited out of F order. This is a reformulation, not a new table representation or a claim of speed.

Pending: finish the critical-path price, compare parallel stable sorting and direct placement, enumerate falsifiers and publish the final ranking.
