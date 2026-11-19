# Modular phase geometry, closed residue groups and multiplicity

The actual surgery trajectory has a sublinear representation bound when the original run count is small, although its strict operations and grammar can still be linear. Quadratic-size powers distinguish setup, discarded grammar and the terminal marked word. [Counts and proof](TRAJECTORY.md)

## Recognize a residue factor before constructing q-way progression lists

In the current m-rank interval exchange let f be the labeled permutation and p the primary. An arc [s,s+w) translated to [t,t+w) has displacement d=(t−s) mod q. Its source residues form a cyclic interval of min(w,q) positions, and at residue r the destination residue is r+d mod q. For each source residue count the covering arcs, counting each arc just once even when it contributes several source ranks. Also sum their displacements and squared displacements as ordinary nonnegative integers. Each arc contributes (1,d,d²) to at most two ordinary residue ranges. Difference arrays and one prefix scan compute all three values in O(B+q) work and O(q) space for B current bands, given both endpoints. Building the source/destination coordinates from two existing interval orders costs one prefix pass per axis. An inhabited residue r has exactly one destination residue iff N Σd² = (Σd)²; every contributing displacement must equal its mean. Use sufficient-width integers, e.g. a wide product when q or B is large. An empty residue is ignored.

If following unique full-group transitions from p mod q closes a cycle of h distinct residue groups, their widths agree. Each group injects into the next and they return to the first, so all such injections are bijections. Every retained row returns in exactly h current steps; all residue groups outside the cycle can be omitted for this layer. A deterministic cycle disjoint from the mark can similarly be excluded even if the marked group branches. This statement requires the transitions of entire groups; an agreement on the single marked row gives no such proof.

Project the visited bands into their respective groups in both preserved orders. Each map is an ordinary labeled interval exchange on the same width. Compose a pair by intersecting the bottom intervals of the first map with the top intervals of the next. Two tilings with k and l intervals have at most k+l−1 nonempty intersections. On each the composed source and target are translations with the concatenated chronological rules. To publish the two orders, bucket intersections by first-arc ID and enumerate the buckets in first top order, then bucket by second-arc ID and enumerate in second bottom order. Within each bucket the increasing intermediate coordinate also increases the source or target. Thus no sorting is required given the two input orders. When h>1 a balanced composition schedule has O(A log h) work and newly stored rule handles, and at most A−h+1 final bands, for A total projected input pieces in the cycle. At each tree level every surviving subchain has at most the sum of its leaf counts. Input projection, the gate and ordering all carry costs; when a root class is wholly invariant (h=1) projecting it needs no joins. The small Python oracle uses explicit sort to independently check the two stable bucket output orders.

Across 5,226 candidate marked groups on tiny original columns, 3,206 had a deterministic class cycle, including 1,296 of length at least three. Every projected partner and actual word matched literal marked positive first returns. Real eligibility is a separate structural census. [Finite class checks](classes.json) · [Recognition and composition](classes.py) · [Real residue census](residues/)

## Decimating rows in a thick cover

If every maximal input run has width divisible by g, row gx+j maps under LF to g·LF₀(x)+j. The quotient source mark is floor(p/g), the within-lane offset has no effect on its bytes, and the return word must still be repeated/truncated to the original horizon n. In particular for a genuine single text the gcd of run widths is exactly its primitive multiplicity. This is the previously established periodic quotient, available by a single original run scan. Heterogeneous gcd quotients can have several cycles. [Prior periodic proof](../../../../../../periodic/PROOF.md)

On an unmodified tower, a level with cumulative modulus M=qʲ selects x in lane j₀ precisely when g x+j₀≡p modulo M. Put h=gcd(g,M). The lane is absent if j₀≢p modulo h. Otherwise it selects one reduced quotient residue,

    x ≡ ((p−j₀)/h) · (g/h)⁻¹ modulo M/h.

For M/h=1 every quotient rank is retained. Within a fixed lane the marked partners and raw words are exactly those of first return to that quotient residue, regardless of whether the actual marked lap divides the original n. Parts of the original orbit can miss a section for a different lane entirely. This explains why full-group surgery can create large amounts of dead grammar on powers. The congruence was checked on 101,728 finite lane/phase instances. [Checks](classes.json)

For heterogeneous columns with relatively prime multiplicities, the original-run gap method can instead remove *all* tied infinite-word fibers. Its base paths contain exactly the d−1 boundaries between distinct rotations, and identifying all distinct fibers costs O((d+R) log(R+1) + d log(d+1)) using the ordered-run predecessor implementation. Scanning the sorted lower endpoints gives the quotient L, one byte per fiber. Both LF and Ψ carry each fiber increasingly onto another entire equal-width fiber. They commute with quotient rank and preserve lane. Mapping p to its fiber ordinal, decimating this new d-row exchange to its marked return, then spelling to the original horizon is exact for every mark. This can change which physical rows the intermediate sections visit; it preserves the requested marked word. A bounded recognition attempt can count visited path vertices and fall back exactly if the budget is exhausted. A primitive block has d=n, so paying an unrestricted recognition scan there has no compression benefit. No faster production path is inferred. [Original gap theorem and implementation](../../../interval-states/refine/proof/chains/README.md)

For L=0ᵍ1ᵍ⁺¹0ᵍ⁺¹ the gcd is one, yet its fiber quotient L is 010, with two distinct necklaces of periods one and two. At g=10¹²+7 the run-based constructor reduced 3,000,000,000,023 rows to three and checked arbitrary boundary marks in both periodic components. Across 3,928 enumerated mark/radix settings, the quotient's terminal word reproduced all 22,600 finite-horizon original bytes, including 1,496 genuine compressions with gcd one. This distinguishes uniform gcd recognition from recovery of all fibers. [Marked tests](fibers.json) · [Source](fibers.py)

```bash
p=$(dirname "$(rg --files | rg '/induce/modular/PLAN.md$')")
d="$p/theory"
python3 -B "$d/check.py" | diff - "$d/checked.json"
python3 -B "$d/layers.py" | diff - "$d/layers.json"
python3 -B "$d/classes.py" | diff - "$d/classes.json"
python3 -B "$d/fibers.py" | diff - "$d/fibers.json"
```
