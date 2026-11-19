# Four runs require unboundedly many independently directed monotone cursors

Let q≥3, a=q², k=q+1 and m=a+1. Use d<c<x and the last column

    L = x d^m x^(k−1) c^a.

It has four maximal runs. The first column is d^m c^a x^k. The stock maximum-frequency policy selects d strictly.

Remove the first m source rows only for notation, writing t=0,…,a+k−1 for the remaining old coordinates. At old t>0, LF sends t to t−k modulo a+k, interpreted in the same old coordinates. This follows directly by counting previous c or x symbols in L. At t=0, LF instead enters the last d source row. Within d's source bucket each rank j>0 goes to j−1, while rank zero has incoming x and enters old t=a. Because gcd(k,a+k)=1, the old modulo translation consists of one cycle whose missing edge 0→a has been replaced by the full d-chain. Thus the actual BWT has one LF cycle and represents a primitive cyclic word.

Discovery on the root d bucket terminates its d pieces immediately. Its only nonterminal child is the singleton old t=a. From there every subsequent interval is a singleton, with a unique preceding parent, until the old zero enters d. No alternate ordering of concurrently ready trie nodes can change this chain. Within c's source bucket 0≤t<a, a step either subtracts k without leaving c or crosses one x row before returning to c. The resulting c ordinal is t−k modulo a. The mandatory sequence is therefore t_j=−k(j+1) modulo a, for j=0,…,a−1.

The sequence of c ordinals splits at its wraps into at most k+1=q+2 strictly decreasing segments. Every increasing subsequence has length at most q+2. For ordinal t in increasing order, the index of its visit is j=(q−1)t−1 modulo a, since −(q−1)(q+1)≡1 modulo q². This inverse sequence has at most q strictly increasing segments, so every decreasing subsequence of the original visit sequence has length at most q. Each independently forward- or reverse-directed cursor can cover only a monotone subsequence. Covering the a=q² requested c ordinals requires **at least ceil(q²/(q+2)) cursors**, even if their assignments and directions are chosen with full hindsight. A single cursor making fresh passes is subject to the same bound on the number of uninterrupted monotone pieces.

This lower bound applies to fetching already-discovered intervals monotonically. An implementation can exploit the four explicit LF translations, compute modular ranks directly, or buffer results obtained before their parents become active. The experiment restricts those operations to test the proposed replacement of random lookup by a fixed number of scans.

The verifier independently reconstructs LF over every position, checks the single cycle and the complete induction graph, confirms both forward and inverse modular orders, and computes exact longest increasing and decreasing subsequences. At q=128 the c bucket has 16,384 compulsory queries. [Verifier](lanes.py) · [Exact lengths and bounds](lanes.csv)
