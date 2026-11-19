# Fused induction has one global set of holes

Let U be a nonempty contiguous interval, and V_j=Ψ^j(U) for 0≤j≤h. Suppose all ranks in V_j emit the same forward byte c_j for 0≤j<h. Write P=c_0…c_{h−1} and V=V_h. Ψ advances the sorted ranks of V_j in order because its restriction to an F-byte bucket is strictly increasing. For every j the same ordinal t names the rank V_j[t] in the sorted section. The positive first-return lengths, word multiplicities and induced permutation in compressed ordinals are invariant under this conjugacy. The word names rotate by the prefix traversed, and the marked entry can move. [One-step conjugacy](../rotation.md)

## A conditional root contraction

Start the backward trie at destination V[t]. The first j≤h backward steps, *if reached*, lead to V_{h−j}[t] with known word P[h−j:h]. Its earliest positive return in these layers is the least such j with V_{h−j}[t]∈V. Emit that exact spelling for this destination, record its source as the ordinal of V_{h−j}[t] in V, and remove this destination ordinal from the remaining layers. After h layers, every survivor is at U[t] with known trailing spelling P. The initial coordinates of this remaining search are therefore a subset of one contiguous original interval. Continuing one LF step from a live parent and finding a terminal V gives the just-discovered reversed path followed by P in forward spelling order. Stable-order consumption of the residual root stream and the known early words restores destination order.

An unchecked jump to U can pass a complete return. For a one-row fixed point it does so immediately. With several types some destinations can terminate in the known prefix while others must continue. The native small test found 2,434 binary section/depth cases with both an early return and a survivor at h≥2. Coincident sections and cycles with several returns shorter than h use the same least-positive rule. The input run selector supplies all V_j bands. The contraction walks these bands and surviving ordinal intervals in sorted order and tests membership only where ordinal or section boundaries change.

## Exact surviving partition

For a rank x on a U-touching cycle, let d_U(x) be the nonnegative forward distance to the next U. Let w(x) be the spelling of these d_U(x) steps. In particular d_U(x)=0 and w(x)=ε if x∈U. The ordinary backward trie on contiguous U partitions all such ranks into disjoint original intervals J_w={x:w(x)=w}. This holds on arbitrary stable columns and arbitrary contiguous U. [Original contiguous construction and endpoint proof](../../clipped/README.md)

Define F_h=⋃_{j=1}^h V_j, with F_0 empty. For any x in F_h there is a positive j≤h with Ψ^{−j}(x)∈U. Hence Ψ^{h−j}(x)∈V and d_V(x)<h. Conversely any d_V(x)<h implies x∈V_{h−d_V(x)}⊆F_h. For x outside F_h on a touching cycle, d_V(x)≥h. If Ψ^k(x)∈V for k≥h then Ψ^{k−h}(x)∈U. Taking the least such k proves d_V(x)=d_U(x)+h. The forward spelling from this x to V is exactly w(x)P.

Thus the rank set of every fused live state is precisely **J_w minus F_h**. A node with no survivors disappears. All surviving nodes retain their original relative order within the parent interval. The result covers old section ranks with short incoming arcs, since F_h uses positive shifts and d_U allows distance zero. A cycle disjoint from U is also disjoint from every V_j. No dense rank table or inverse-byte oracle is required by the constructor.

Let f be the number of connected rank components of F_h. For any nonempty J_w minus F_h, every extra interval beyond the first charges a complete component of F_h strictly inside J_w. No component can be charged twice because the J_w are disjoint. Therefore the number of surviving original-rank intervals exceeds the number of surviving states by **at most f across the entire trie**.

A constructor can retain only the two hull endpoints of each live state and one bit saying that adjacent arrivals left a gap. Hulls of distinct states are disjoint. A state with a gap recovers every live span by skipping components of the one shared F_h. A state without a gap needs no mask query. The sparse implementation constructs F_h on the first observed hole. A global projection deleting F_h from the rank universe would make every surviving state contiguous in compressed coordinates, provided that terminal arrivals still publish their original V ordinals.

## Carrying the actual mark

The compressed marked state consists of the ordinal of the nearest shifted-section rank at or behind p in LF order, together with its forward byte distance to p. Since LF^k(p)∈V_h iff LF^k(LF^h(p))∈U, the ordinary U mark of LF^h(p) has exactly the required ordinal and skip. The check is inclusive after the initial h backward moves. This holds for every finite period, including periods shorter than h and marks on cycles disjoint from the section. A decoder can therefore transfer the mark using only U membership and its ordinary LF navigator. The structural verifier compares this formula with both the native V membership search and the independent dense marked result on every small test mark and the ten real primary ranks.

## Bound on paid original LF translations

Let T_0 be total intersections of ordinary live intervals with the original L runs. Let ΔH be the number of old intervals that disappear, and s the number of V components. Deleting an entire old interval removes at least one intersection. Within the remaining intervals, removing one global F_h component can add at most one intersection, and then only when it cuts through the interior of one old interval and one original L run. Distinct old intervals share no ranks. Finally label each original LF run by whether its *image* belongs to V. Each of the at most 2s original-coordinate V boundaries can cut the image of at most one LF run, since the images partition all rows. Each such added run boundary can be scanned inside at most one surviving interval. Hence

```text
T_shift ≤ T_0 − ΔH + f + 2s.
```

This bounds the lookup/translation phase for a fused traversal with precolored LF runs. Selector consumption, shifted-image setup, root contraction, mark transfer, writing words and reconstructing output still incur their own costs. The conditional bound can establish fewer paid LF pieces before timing.

## How many classes get partially cut?

For one shift, each distinct old word c·v loses exactly those ranks one edge after its source. Those ranks are in the old spelling class v. If v is nonempty and maximal among the tails under proper-suffix inclusion, that whole class is removed. Every occurrence in the class must arise from that same word, because a distinct word contributing a deeper occurrence would have a tail properly extending v. For each nonmaximal tail the corresponding class has both removed and retained ranks. An empty tail in a multiword dictionary also represents a partial root. If all words have length one, monochromaticity forces K=1 and there is no partially cut class.

For K>1, with M maximal nonempty tails, exactly K−M old classes are partially cut in this one shift. The one-step formula Q_old−Q_new=M−1 turns this into K−1−(Q_old−Q_new). The same formula gives zero partial nodes when K=1. After h root layers, every surviving original class maps injectively to one live class with a common trailing spelling. A newly partial original class must become partial at one of those single steps. The number of partially cut original classes is therefore at most the sum of these h one-step deficits. A full K−1 saving in each layer only removes whole classes, even if the shifted rank section itself has many components.

Frozen h=3 reaches Q=91,849, with 91,846 surviving original classes after contraction. Exactly 52 old classes are partially cut. Only 29 extra intervals remain. The complete upper bound gives 102,932 original LF pieces, and the native traversal needs 101,238, compared with 110,061 on the original section. [All rankwise partition checks and bounds](partition.csv) · [Small multi-cycle and overlap checks](partition-small.txt)
