# A primitive quarter cut can grow the geometric representation

The first arc-identity quarter cut of L=1110000 with zero-based mark 4 has four bands after mark isolation and self powers. It returns five bands after removing two of seven rows. Stable LF is the rotation by four modulo seven, so the physical column is primitive. The sweep visits six source/destination pair pieces and seven triple pieces. All five retained rows occupy singleton child bands. The frozen true-block panel showed contraction in every nonterminal power-to-quarter-cut layer, but physical mass reduction alone cannot certify band-count reduction.

In general let B denote the number of powered bands and E the number of pair pieces. The two partitions whose intersection yields the triples each contain E nonempty intervals and both include every old source seam. If c counts their further common interior seams, inclusion-exclusion gives triples = 2E−B−c ≤ 3B−2 because E≤2B−1. The existing true-block panel had 393 nonterminal layers, 152 attaining 2E−B, with largest triples/B about 1.39882 and largest output/B about 0.93293. This bound applies before any native child-band coalescing. It does not prove an optimal worst-case constant for the number of child bands.

The binary oracle enumerates primitive columns in length order and invokes the frozen power and cut code once for each mark. Its first witness gives the fields L, mark, n, isolated bands, powered bands, pairs, triples, removed rows and returned bands. [Source](check.cpp) · [Completed result](result.txt)

```sh
c++ -std=c++20 -O3 -I../.. check.cpp -o /tmp/astra-arc-width
/tmp/astra-arc-width | diff - result.txt
```
