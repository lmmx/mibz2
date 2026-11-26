# Five BWT runs force arbitrarily many changes of direction

Take the ordered alphabet d<c<x. For t≥1 choose m=2t+2 and the cyclic word

    d^m (c c x)^t c.

The word is primitive, since it has exactly one maximal cyclic d-run. Its unique heaviest symbol is d, as c has 2t+1 occurrences and x has t. The ordinary maximum-frequency second-induction policy therefore picks d.

The marked cycle's BWT has just five runs, for every t. Among d-starting rotations the longest d prefix comes first and is preceded by the final c, while each of the remaining m−1 starts is preceded by d. Among c-starting rotations the trailing c before d comes first and is preceded by x. The t starts of ccx come next. They sort by increasing number of remaining ccx copies, because the first difference is d<c. All except the last are preceded by x; the last is preceded by d. The t remaining c starts precede x and are themselves preceded by c. Every x-starting rotation also has predecessor c. Thus the full last column is

    c d^(m−1) x^t d c^(2t).

The unique long excursion into the d bucket requires visiting all 2t+1 suffixes that start with c, in increasing length. Their next symbol, in that necessary ancestor order, is d, x, c, x, c, …, x, c. The first comparison of consecutive c-led suffixes is therefore decided by their second letters. The required source-rank motion alternates strictly up and down for 2t steps. Any forward-only scanner of the c bucket must reset at least t times, and the same holds for reverse-only. Even a freely reversing scanner changes direction at least 2t−1 times. The graph here is one primitive marked cycle, the pivot wins strictly, every queried c interval has width one, and the bound grows with t while the input R stays at five.

The obstruction concerns schedules that discover each interval from its already processed parent and scan each original source class with one cursor. An implementation with a tiny run index can jump directly on this family. Two cursors, stored out-of-order rank answers, or speculative child discovery have different costs. The structural consequence is that run count alone cannot bound cursor resets for the proposed index-free ordering.

The verifier sorts every actual cyclic rotation through t=128 and checks the closed-form five-run column. It tests the complete induced graph and exact compulsory sign sequence up to t=1024. [Verifier](family.py) · [Raw family sizes and turns](family.csv)
