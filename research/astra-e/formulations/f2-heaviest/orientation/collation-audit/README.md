# Independent review of return-word collation on an interval

The proposed order is valid for every nonempty contiguous rank interval U in a finite stable cyclic BWT. U can cut a tied-rotation class, span several LF cycles, or fail to be a prefix cylinder. Any q in U gives the same order of distinct positive first-return spellings W by the infinite keys W Q, where Q is the forward code of q. Each spelling occupies a contiguous bucket in source-rank order. The method still has to recover the destinations and the within-bucket occurrence order for a reconstruction.

## Proof at a prefix conflict

Write C(i) for the infinite forward spelling from rank i, and Ψ for stable select. On any finite prefix cylinder D(v), Ψ to the power |v| is strictly increasing. This follows by composing stable occurrence order once per byte. D(v) is contiguous, so this map is also noncontracting in integer rank distance. Rank order refines lexicographic order of infinite codes; equal-code ranks retain separate identities.

Take two occurrences with distinct return words. A first mismatch before either word ends decides their order in both the original source ranks and the proposed keys. Otherwise write the words W and Wv, with v nonempty. After spelling W, the short occurrence is at y in U, while the long occurrence is at j outside U, reaching z in U only after spelling v. Because both sources share W, their rank comparison equals that of y and j. It also equals the comparison of q and j, since q and y are in U and j is outside.

If the initial |v| bytes of Q differ from v, the comparison of Q with vQ equals that of C(q) and C(j), and hence the rank comparison of q and j. Otherwise put q′=Ψ^|v|(q). Now Q versus vQ reduces to C(q′) versus C(q). If j is to the left of U, then q′−z ≥ q−j by noncontraction on D(v). Consequently q′−q ≥ z−j > 0. If j is to the right of U, the same inequality with the ranks reversed gives q′−q ≤ z−j < 0.

Strict rank difference here also gives strict code difference. If C(q′)=C(q), then shifting |v| symbols maps the finite tied-code class of q into itself. It acts strictly increasingly on that class, as the whole class shares prefix v. An increasing injection from a finite ordered set into itself is its identity, so q′=q, a contradiction. This argument continues to hold when the source ranks are tied, even when q lives on neither source's cycle.

Thus distinct spellings compare strictly in the observed rank direction. Equal spellings give the same key. It follows that a different spelling cannot intervene between two occurrences of one word. For comparison, a cache of the forward prefix of q and the ranks along it through Lmax steps suffices. Only the rank after |v| steps is needed when the prefix bytes match v; the trace may cycle many times before Lmax.

## Which tails work?

For bytes, use positional base 257 so every infinite byte sequence has an order-preserving numeric value without ambiguous trailing maximal digits. For a finite string v of length k, let A(v) be the value of its first k fractional digits. Writing t for a continuation, the comparison of t and vt has the sign of

```
(1 − 257^(-k)) t − A(v).
```

Its threshold is the value of the periodic word v repeated forever. Therefore every required prefix comparison imposes one strict lower or upper bound on t. All tails giving the source order are characterized by the intersection of these half-lines and the fixed first-mismatch comparisons. For arbitrary U this is a post-hoc condition given the return words and their source sequence. A freely chosen real t may fall outside the set of attainable row codes.

For contiguous U the intersection contains the closed numeric span of C(min U) through C(max U), since both endpoints satisfy all strict comparisons. An outside rank tied in code with an endpoint also works. For a single prefix conflict the proof even allows a hole when j remains strictly outside the hull of q, y and z. General holed sections and general outside ranks can fail.

## Independent falsifiers

The exhaustive program derives Ψ by stable sorting (byte, rank) pairs for every binary last column through length 9 and ternary last column through length 6. It enumerates every positive first return from the actual ranks. The independent oracle compares exact fractions for WQ in base 4, with Q taken from the actual marked cycle. Separately it checks the finite-prefix-plus-rank comparison. Adjacent source comparisons suffice by transitivity. Distinct codes and genuine prefix cylinders are enumerated explicitly.

Across 1,988 columns, 55,796 intervals and 177,210 admissible reference marks, all 599,484 adjacent checks passed. There were 35,158 noncylinder intervals, 26,831 intervals spanning multiple cycles and 11,460 intervals cutting equal-code ranks. Among 124,278 distinct prefix comparisons, 22,042 needed the rank tie-break, 15,824 had q on neither source cycle and 5,072 involved tied source codes. All are marked-rank tests on arbitrary L columns, including multicycle permutations.

| Dropped condition | Last column L | Section and q | Failure |
|---|---|---|---|
| q belongs to the interval | 0100 | U=[0,2), q=3 | Source words 0,001 reverse under these keys. |
| q belongs to the interval | 0101 | U=[0,2), q=3 | Source words 0,01 have equal keys. |
| Contiguous section | 0100 | U={0,1,3}, q=0 or 1 | Source words 0,00 tie for q=0 and reverse for q=1. |
| Contiguous section | 01000 | U={0,1,3,4} | Source-ordered words 0,00,0,1 split the 0 bucket. |
| Rank tie-break after a matching prefix | 10001 | U=[1,5), q=1 | Source ranks 3<4 emit 10,1. Appending the Lmax=2 cached bytes 00 gives 1000,100, whose ordinary finite-string order reverses them. |

Run `python3 -B audit.py | diff - receipt.json` from this directory. The receipt includes exact fractions and witness ranks. [Program](audit.py) · [Receipt](receipt.json)
