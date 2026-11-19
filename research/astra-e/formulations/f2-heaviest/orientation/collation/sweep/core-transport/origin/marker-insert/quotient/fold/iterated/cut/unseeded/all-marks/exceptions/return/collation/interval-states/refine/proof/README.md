# Exact adaptive refinement by stable occurrence endpoints

Let P be any interval partition of the physical rows of an arbitrary stable BWT L of size n. Use the same partition for physical sources and LF destinations. For each source state I choose the most numerous pair (a,J) of preceding byte and destination state. Let S(P) be the sum of those maxima, counting physical rows. This is a modal prediction-mass objective. It need not equal the proportion of hits along a particular marked lap in a heterogeneous column.

## Target changes that leave every source histogram intact

For byte a let C[a] be the starting rank of its F bucket and Occₐ(t) the number of a entries strictly before source rank t. Within a source interval I=[l,r), all a entries map increasingly onto

    B(I,a) = [C[a]+Occₐ(l), C[a]+Occₐ(r)).

The nonempty B(I,a) tile the entire physical target in byte order, then source-interval order. Every new target seam z either lies between tiles or cuts exactly one tile interior; splitting one tile can split an old (I,a,J) only in the old destination J containing z. Let Z(P) be the set of interior physical endpoints between nonempty tiles. They include all F-byte seams. Equivalently form all C[a]+Occₐ(t) for t a source boundary (including 0,n) and retain the distinct interior coordinates. The number of tiles is the sum of the supported bytes per source interval. It is at most R+|P|−1−s, where R is the original L-run count and s is the number of run seams shared with P. This follows by charging a supported byte to an original run/source intersection.

Choose **any subset simultaneously** of Z(P) as additional cuts T, and call the resulting interval partition Q. A source child I′ in Q lies in an old I. Its image for a fixed old label (a,J) is B(I′,a)∩J, which lies entirely in one refined target state. The old and new histograms on I′ therefore have the same counts up to relabeling. Different old labels cannot merge because Q refines P. It follows that S(Q) equals the sum over new source children of their modal counts **against the old target partition**. Each old parent winner is one of the choices within every child, so S(Q)≥S(P). This holds on periodic columns and columns with multiple LF cycles and requires no information about a marked source.

More generally, for *arbitrary* new seams T the exact histogram-preservation criterion is T⊆Z(Q). Indeed a seam missing from Z(Q) splits B(I′,a) for one refined source child and byte. Since the seam is new, it is strictly interior to its old destination state, so the two adjacent target ranks were in the same old histogram bin. The converse is the preceding tile argument with Q as the source. This criterion allows additions that become safe together. A non-F seam z in byte bucket a belongs to Z(Q) exactly when some source boundary t of Q has C[a]+Occₐ(t)=z. If u,v are the physical occurrences of a with LF ranks z−1,z, all such source positions t form the integer interval [u+1,v]. The equivalent condition can be tested solely on final boundary membership. A split nonwinning histogram bin may leave its modal count unchanged; the criterion characterizes preservation of the complete histogram.

## The optimal subset from the old safe seams

Fix a nonnegative charge λ per new seam. On each old source interval [l,r), the previous result reduces the score to an ordinary segmentation of its known label sequence t(x)=(L[x],state_P(LF(x))). Candidate boundaries are its interior members of Z(P), plus l,r as endpoints. Write N_b(u,v) for the mass of label b in [u,v). If V(l)=0 and each segment is charged λ, the exact recurrence is

    V(v) = max over allowed u<v and labels b of V(u)−λ+N_b(u,v).

Its last segment is nonempty and independently chooses its best label. Conversely every predecessor and label in the recurrence describes a legal segment. Add λ once per old source to convert from the per-segment convention to charging only new cuts. Maximizing S(Q)−λ|Q| differs by the fixed cost λ|P|. Backtracking one maximizing predecessor per chosen end recovers an actual optimum, including ties.

For an implementation, keep for each label its best score and last-segment start. At a permitted boundary u, offer the uniform restart floor V(u)−λ to every label. The prefix optimum V is nondecreasing after the first segment because its last label can always be extended, and λ≥0. Maintaining only the best offered floor and its start suffices even before the first allowance. When scanning a constant-label fragment ending at v, apply that pending floor to the encountered label if beneficial, then add the fragment width. Every other label's potential stays unchanged apart from its implicit floor. The best score over all labels can be maintained as a monotone maximum: a new floor never exceeds the preceding prefix optimum and only the observed label gains mass. Store a predecessor for every used boundary, or retain only successful floor increases and their chosen previous path. This yields the same recurrence with one label update per fragment. Source and target seams split each original L run at most 2(|P|−1) times **in total across the column**, as the inverse of any target boundary straddles at most one original affine run. The DP adds at most one fragment per allowed seam. With labels and allowed endpoints in order, it takes linear work in the resulting fragment count and per-source initialization for labels actually used. The method optimizes this particular penalized step; it does not enforce a fixed cardinality or optimize later refinement rounds.

With just one initial state the safe seams are F-byte boundaries. Taking all safe seams at every round then gives the familiar exact fixed-length prefix contexts (retaining their ancestors, which are already context boundaries). Selective safe refinement may choose different depths in different physical intervals. Enumeration of all fixed-context partitions remains avoidable within one registered optimization pass.

## What unrestricted cuts can cost

For every g≥1 let L=1ᵍ0⁴ᵍ⁺¹, n=5g+1. Its LF is the rotation by −g, a single cycle because gcd(g,n)=1, hence a primitive cyclic BWT. The initial F cut is 4g+1, with S=4g+1. Add the single seam z=2g. Counting against old destination states, the three new source intervals have maximum masses g, 2g+1, g. Against the refined target they are g, g+1, g. Thus a single unrestricted seam can decrease actual S by g while its source-only calculation predicts no loss. The smallest enumerated primitive loss from the F partition is L=10000 with seam 2 added to seam 4; S drops from four to three. Starting from an arbitrary partition the smallest example is L=1000 at the empty old seam set and new seam 2.

For a weaker promise choose one old winning label per source and forbid only the interiors of its target hit interval B(I,a)∩J. Every subset of the remaining seams still gives S(Q)≥S(P): the mass of that one old winner remains available within every source child. Other labels may split, so the independent source-only optimum can overestimate S(Q). On the primitive L=1010 with one old state and zero selected in the modal tie, seam 3 protects the winning zero image. Source-only mass rises from two to three, but the actual joint mass stays at two.

## Independent checks

Direct rowwise stable LF checks over 374 enumerated binary/ternary columns, 11,699 old partitions and 114,434 ordered pairs of nested partitions verified exact histogram neutrality. They include 42,265 old-safe subsets, of which 12,108 add at least two cuts at once. Five integral or half-integral charges gave 58,495 exact DP checks, including backtracking and recalculating the actual joint score. There were 1,870 single-cut mass decreases. The inherited-winner guarantee passed on 45,457 subsets; 846 of them fail the stronger source-only equality. A separately constructed loss family checks g through 128. [Raw counts and counterexamples](small.json) · [Independent oracle](check.py)

```bash
e=$(dirname "$(rg --files | rg '/all-marks/exceptions/PROOF.md$')")
d="$e/return/collation/interval-states/refine/proof"
python3 -B "$d/check.py" | diff - "$d/small.json"
```
