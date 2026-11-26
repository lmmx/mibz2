# Fixed unit charge and one primitive binary cyclic text

**Theorem.** For every multiple k of three with k≥36, a primitive binary BWT of length 3k²−1 has the following property. Starting from its F partition and charging one per added seam, every globally optimal neutral refinement requires a batch of at least k−1 seams. This holds even if immediately self-supporting additions are allowed. The proof gives a candidate with net gain k−1. It does not require uniqueness.

## One deleted row joins the cyclic cover

Set g=k, n=3kg−1 and B=n−3g. Take the last column

    L = 1^g 0^g 1^(2g) 0^(n−4g).

This removes one final zero from the stable thick cover. The F seam is B. In physical rank order, Ψ is given by

| Rank t | Ψ(t) |
|---|---|
| 0 ≤ t < g | t+g |
| g ≤ t < B | t+3g |
| B ≤ t < B+g | t−B |
| B+g ≤ t < n | t−B+g |

Every low rank eventually visits the high interval [B,n). Starting from B+j, the first low representative in [g,4g) is j+g, with one extra hop through j when j<g. Adding 3g until the next high visit gives new offset j'=(j+g−B) mod 3g = (j+g+1) mod 3g. This rotation of the high interval has one cycle since gcd(3g,g+1)=1 for g divisible by three. All n rows therefore form one physical LF cycle.

The only non-F seams with more than one possible supporter are g, with supporters [2g,4g], and B+g, with supporters [g,2g], inclusive. All other non-F interior seams t have unique supporter Ψ(t). The k seams

    T = {g} ∪ {4g−1,7g−1,…,B+g}

form a directed cycle in their own support graph. The seam g can use 4g−1, the middle seams advance by 3g, and B+g can use g. The base error n−S(P) is 3g. The refined errors are exactly the g zeros lying between g and 2g and the omitted last one at 4g−1. Thus S(P∪T)−S(P)−|T|=2g−1−k=k−1. Every global optimum therefore selects at most 3g−(k−1)=2k+1 seams.

## Near-optimal physical seams project to the small primitive column

Take any neutral selection with at most 2k+1 seams. Follow the unique supporters of any ordinary selected seam, stopping at g, B or B+g. Since Ψ is a single cycle, a stop is reached after at most 2k+1 steps along selected seams. In a high-to-low step Ψ changes the residue modulo g by +1. In all low steps it preserves the residue. Each high-to-low step lands below 4g, and a subsequent low step can advance at most 3g. Successive high visits are therefore at least k−1 steps apart. A path of length at most 2k+1 has at most three high-to-low steps for k≥36. The endpoint has residue 0 or −1, so every selected seam is within D=5 of an integer multiple of g. This bound also covers the initial F seam B.

Round physical t to the nearest integer row t/g of the length-3k primitive column with L₀=1011 followed by zeros. Denote its F seam by b=3k−3. Treat projected seams 0, b and 3k as existing interval endpoints. Every other projected dependency edge is an allowed edge of the small graph. For an ordinary small seam r outside {1,b+1}, its entire D-neighborhood lies inside one translation interval of the large Ψ, and a selected supporter rounds to the unique small supporter. Near r=1 its supporter rounds into {2,3,4}. Near r=b+1 it rounds into {1,2}. In the two latter cases the wide intervals at the actual large seam have these same possible rounded targets. For any original singleton schedule, append each projected seam when first seen and ignore existing endpoints or repeated projections. Support is old, earlier, or self. Hence singleton accessibility passes to the projected set U, with |U|≤|T|.

Bound the effect of rounding on modal mass. Scale the projected partition back by g and include its physical endpoints. Each of its mixed-byte intervals [l,r) contains at least one of the three original L transitions in its interior, so at most three such intervals exist. The trimmed core [l+D,r−D) of each mixed interval lies in one actual large source interval, because no selected seam can be more than D away from its projected coordinate. Removing at most 2D rows from an interval decreases its minimum binary prediction error by at most 2D. Summing these disjoint cores gives

    actual error ≥ g · (small projected error) − 6D.

A single actual cell containing multiple cores still owes at least the sum of their separate minima. The shortened final row is covered by the endpoint trimming. Both partitions are neutral over their respective F starts, so these preceding-byte errors are their actual joint errors. Their initial errors are 3g and 3 respectively.

## The small primitive column bounds all projected singleton schedules

Write h for raw modal hits gained over the small base. Its only possible values are 0,1,2,3. For any small singleton-accessible U, gh−|U|≤2.

- If h=1, at least one seam among 1,…,5 is necessary. The first six rows with no cut already give three errors. Each possible such seam forces at least k−2 seams. Seam 3 forces the 0-modulo-3 chain up to the old b. Seams 2 and 5 force a chain through b+2 and then through 3. Seam 4 forces at least k−1 through b+1. Seam 1 requires 2,3 or 4.
- If h=2, at least one of 2,4 is present, by the two disjoint opposing pairs used in the primitive family proof. Seam 2 forces 2k−2. With 4 present and 2 absent, the entire k-cycle {1,4,…,b+1} is compulsory. Its only remaining external supporter that can seed a singleton schedule is 3 for seam 1. The chain starting at 3 costs another k−2 distinct seams.
- If h=3, all 3k−2 new seams are forced. The h=0 case already satisfies the inequality.

It follows that for a singleton-accessible selection competitive with the explicit candidate, its net gain would be at most gh−|U|+6D≤32. This contradicts the available k−1≥35. Finally any rootless directed cycle in the actual graph has a high seam, since every low support edge strictly increases rank. Its next supporter is below 4g, and reaching the next high seam requires at least k−2 low steps. Thus its length is at least k−1. The sink-component characterization implies that every optimum requires a batch of at least k−1 seams. [Accessibility and small primitive-family proof](README.md)

## Exact finite checks

The short experiment prices the same penalized modal objective by enumerating every neutral choice that can achieve nonnegative gain. Outside the two branching seams and the physical endpoints, ordinary seams have deterministic successor Ψ. In the primitive case the ordinary vertices split into disjoint chains ending at those designated vertices. A selected ordinary set on each chain must be a suffix next to its selected endpoint. Two endpoints (0 and g) have empty chains. For each subset of the two branching seams, enumerate the two remaining suffix lengths and test every support. Cardinality greater than the entire base error can only produce negative net gain. The direct small oracle separately enumerates all physical subsets and counts actual (byte,destination) pairs.

With g=k, both single-row deletions and insertions were checked through k=17 wherever they are primitive. Deletion cases at k=24,36,42 extend the check above the proof threshold. The stored finite optima specify their exact width and any ties; the general lower bound above does not depend on these outputs. [Enumerator](primitive.py) · [Raw selections and oracle](primitive.json)

```sh
python3 -B primitive.py | diff - primitive.json
```
