# Fixed first-hit cylinders and their earlier controls

For a byte string B, let I(B) contain the ranks whose forward periodic byte stream starts with B. For U=I(P) and a nonempty word C, let Z(C) contain all starts that spell C and first hit U at positive time |C|. A start in U is permitted. Every row of I(CP) has fixed bytes through its landing and through every earlier test for P. Therefore:

* If CP contains P starting at any offset j with 0<j<|C|, then Z(C) is empty.
* Otherwise Z(C)=I(CP).

This test includes copies overlapping the landing boundary and words longer than a cycle. It works independently in every cycle of a stable marked column. Equivalently, start at U and prepend C backwards using stable byte ranks. Before every extension after the first, remove U from the live ranks. Retain a final start inside U. For a literal landing the membership of each intermediate state is fixed by the string, so checking the offsets once replaces all clipping. Given U, each payload byte costs two original endpoint ranks in the ordinary FM constructor. Recognizing U from a byte histogram adds two endpoint ranks per byte of P after the last. With this implementation J1 costs two endpoint ranks from scratch and J2 costs 26. This counts the queries and excludes index preparation.

For J1, P is the quote and C is the comma. For J2, P is `223a22` and C is `222c2270726f7065727479`. Neither joined word has a forbidden interior occurrence. J1 is exactly I(`2c22`), and J2 is exactly I(`222c2270726f7065727479223a22`). Dropping the landing changes the test. In the synthetic column below, six rows begin with the J2 payload and four of them land in U. A three-row binary witness shows why a general landed pattern still needs the interior test. For L=(1,0,0), P=(0) and C=(1,0), the sole row spelling CP visits U after one byte. Its positive first hit occurs too early.

## Which cuts were already tested

| Earlier construction | Relation to the two fixed cuts |
|---|---|
| Most frequent quote return, then most frequent return of that section | On all ten historical tapes the first winner gave U=I(`223a22`). The third named section was M=I(ACP), A=`223a2276616c7565`. Advancing each source by eight bytes injects M into J2. On frozen every J2 start has the required preceding A. The two sections are conjugate, with different physical ranks and byte offsets. |
| Trailing-quote token `2c22` selected at the first level | The source used by the fixed token screen is I(`222c22`). The label omits the leading quote and includes the landing quote. With the cut shifted one byte forward, a comma in J1 must still have a preceding quote to qualify. Frozen J1 has 14,737 other rows. The analogous trailing-colon-quote token `3a22` selects I(`223a22`), the parent U of J2. |
| Bounded shortened-core selection | That search already computes the entire Z(C) before retaining the selected physical points H. Quote h4/h8/h12 on frozen retained 28,909/29,516/30,581 of J1's 30,581 rows. Frequent h8 on frozen retained all 3,232 rows of J2. On the previously published ten-tape h8 panel, Ψ⁸(M) omits 4 J2 rows at ordinal 24, 1 at 72 and 4 at 144. It equals J2 on the other seven. |

The ranked top-one search chooses a terminal *source* bucket inside its parent. The earlier twice-heaviest native worker paid both searches and reconstructed from M. For two consecutive sources separated by an old return W, the displaced return Y satisfies WA=AY because both sides read the same |W|+8 bytes. This argument gives the bijection of marked return systems when the displaced source set is the whole new cut. On a tape with holes it gives the return system on Ψ⁸(M) only. Inserting an extra J2 row splits a previous return edge and can change the nearest marked source. Geometry alone supplies no timing comparison. [Ranked selector at 34310a6](https://github.com/lmmx/mibz2/blob/34310a6ad12e15733fda6e293106701366f74f1e/research/astra-e/formulations/f2-heaviest/union/direct.py) · [Third marker at cb2e513](https://github.com/lmmx/mibz2/blob/cb2e513cdda4afbf4a27496d34b4f128f195ad22/research/astra-e/formulations/f2-heaviest/wikidata.csv) · [Paid native at 2b19f2e](https://github.com/lmmx/mibz2/blob/2b19f2e2f3430b9efe23467de968959c6059a272/research/astra-e/formulations/f2-heaviest/candidate.rs)

The composition proof also identifies the after-occurrence convention of a token writer. For a complete quote-return label T ending in quote, the byte cylinder for its occurrence is I(quote·T). Advancing this section by |quote·T| gives the after-occurrence boundary. Such uniform rotations retain the prior-occurrence condition. The fixed token screen itself enumerated the starts at the leading quote. The earlier positive first-hit proof allowed any interval landing, and the hole census preserved its full and selected counts. [Word composition at ee35db0](https://github.com/lmmx/mibz2/blob/ee35db00e5e8b9b726d6bc1c16817a9fabb0d5cc/research/astra-e/formulations/f2-word/README.md) · [Selection at 67fbfac](https://github.com/lmmx/mibz2/blob/67fbfac0783c6b1faac5ed057ffa5bd64e3837a2/research/astra-e/formulations/f2-heaviest/union/selection/workload.py) · [First-hit proof at 3fce1c0](https://github.com/lmmx/mibz2/blob/3fce1c0a4f3690f3eef83caaba30f10e59f24d26/research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/README.md) · [Hole census at fb6a1da](https://github.com/lmmx/mibz2/blob/fb6a1da1c8c037b9015e376cb7fdf2b3df194ef4/research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/holes-wikidata.csv)

## Independent checks

| Frozen object | Ranks | Rows |
|---|---|---:|
| Old trailing comma-quote source | [3977,19821) | 15,844 |
| Full J1 | [141987,172568) | 30,581 |
| Old third named source M | [39058,42290) | 3,232 |
| Full J2 | [11107,14339) | 3,232 |

The one-byte shifted trailing-token source occupies 113 separate components inside J1. Bare I(C) happens to equal J2 on frozen. On a cyclic source containing both ACP and CP with an unrelated preceding context, the older source can miss exact J2 starts. The test constructs a column with two copies of that cycle and one disjoint fixed point. The old third, full J2 and bare I(C) have 2, 4 and 6 ranks respectively. Further tests cover absent landing, present landing with absent payload, periodic covered and disjoint marks. All 49,784 ordered payload/landing cases on every binary last column through length seven agree between direct positive-hit walks, stable backward extensions with clipping and the literal theorem. Of these, 3,220 have a nonempty joined cylinder killed by an interior hit. The all-mark tests independently emit exactly n raw bytes from first-return edges and check uncovered marked cycles separately. [Source](fixed.py) · [Exact output and dense-L digest](verified.txt)

From the worktree root, with the frozen tape generated by the dense census:

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/direct-native/audit
python3 "$d/fixed.py" research/astra-e/dense/target/wikidata/0000.irec.xz | diff - "$d/verified.txt"
```
