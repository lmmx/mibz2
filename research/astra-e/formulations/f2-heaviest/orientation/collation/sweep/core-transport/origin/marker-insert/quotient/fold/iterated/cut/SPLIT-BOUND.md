# A run/alias lower bound for a single geometric cut

The protected family F=a^N b, L=b a^N has two incoming runs. Its stable permutation moves from physical row x to x+1 modulo N+1. The destination of the final a is b and the destination of b is the first a. Mark the b row.

Allow any preprocessing that splits the initial a bucket into **t−1 contiguous intervals** and retains b as one outside singleton. Give each interval its own alias and choose any subset of aliases as the delimiter set. There are now t source intervals on the physical circle. Membership is constant inside each interval. Because every permutation edge goes to the adjacent physical row, a crossing edge can occur only at an interval boundary. An ingress to the delimiter set alternates with an egress, so the number of contracted rows in a single outside→inside batch is at most floor(t/2). Alternating colors starting with b outside attains that bound regardless of the interval widths.

Thus even arbitrary placement of the alias endpoints cannot guarantee eliminating a positive fraction c of the N+1 orbit rows unless t ≥ 2c(N+1). The unrefined t=2 case in the original falsifier removes exactly one. This bounds an entire class of attempted repairs which preserve contiguous source aliases and use a single ordinary cut. It leaves open a method that emits a repeated word as a power, or that computes many contractions symbolically without constructing the corresponding aliases. [Original family and numerical checks](README.md)

The executable finite check verifies every positive interval-length composition and anchor-respecting coloring through 12 original rows. It tests exact rank-crossing counts and the sharp per-partition maximum. [Source](split_bound.py) · [Results](split-bound.json)
