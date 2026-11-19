# Two retained rows can require a linear table scan on a true primitive BWT

There are binary, one-cycle cyclic BWTs with K=2ᵏ positive k-contexts, one true exceptional row e, one synthetic primary p and two elementary event ranges. Given all context boundaries, both exceptional positions and their poststates, and the promise of a single primitive text, computing the weighted return between them can require Ω(K) entry probes in the explicit, uncontracted context table. Depth k grows in this family. One probe may read both alternatives of a paired binary predecessor assignment. The bound therefore also applies if each probe reads one ordinary table row. It concerns this input interface. A precomputed return index, word-packed batch queries and other source representations have different costs.

## A single exceptional row

Let q=2ᵏ, N=q−1. Use a primitive binary right-shift register T whose nonzero vectors form one orbit. It acts as multiplication by a generator α of GF(q). For an explicit bit convention, encode field element X by the k bits Tr(αʲX), with j=0 the low bit. The nondegenerate trace pairing and the basis 1,α,…,αᵏ⁻¹ make this encoding invertible. Multiplication by α shifts the vector right and adds a new high bit. Let v be the field element with bit vector 1. The sources x and x+v have consecutive bit labels 2u and 2u+1. T assigns them to 0u and 1u in one of the two orders. Swapping the two assignments changes just that pair. Any choices of these pair assignments are stable LF: among sources assigned a given high bit, their source order is exactly the increasing order of u. The positive k-future of row x is its bit label read from high to low, because each inverse shift moves the next bit into the high position.

Write zᵢ=Tⁱ⁺¹(v) for i=0,…,N−1, so the last element is v. Initially zero is a fixed row separate from this orbit. Swapping the assignments at 0 and v inserts zero after v and before z₀. Replace that visit to zero with two consecutive visits in LF order

    v → f → e → z₀.

In physical rank order use e=0, f=1, and rank old_label+1 for every nonzero old label. The two new L entries are 1 and 0 respectively. The old nonzero labels retain the corresponding predecessor of the spliced orbit (0 for v). Stable byte counting gives precisely the displayed arcs. In forward order e has k+1 leading zeros and f has k zeros followed by one, so these are the two sorted physical members of 0ᵏ. Every other k-context has its original single label. The stable column has one cycle. It is the ordinary cyclic BWT of a primitive binary text: sorted ranks have their distinct k-futures apart from e,f, whose order is decided by the next bit. One full physical traversal realizes those rotations.

Ties in the repeated context select zero. Thus e is its sole true exception. Set the synthetic primary p to the expanded physical rank of z at index floor(N/2). All other rows, including p, have their modal predecessor. The total size is q+1.

## Paired switches that change a required distance

Divide the indices 0,…,N−1 into four consecutive quarters with boundaries floor(jN/4). Exclude p and v from all chosen endpoints. Choose a chord x→x+v from the first to the third quarter and another y→y+v from the second to the fourth. The endpoints occur in order x,y,x+v,y+v along the original cycle. Switch the outgoing arcs at both chord pairs. Each is a legal stable assignment. Cutting the original cycle after these four endpoints and joining the four arcs in switched order again gives exactly one cycle. Neither switch touches e, f, v or p. Thus the sections, their poststates, counts, ordered context boundaries, actual exceptional byte and true-one-cycle promise agree across the two instances.

In the new orbit the path from e to p follows the old path to x, follows the segment after x+v through y+v, then follows the segment after y through p. Its length changes by

    i(y+v) − i(x+v) − i(y) + i(x),

where i(·) is the old z index. This difference cannot even be zero modulo N. Equality modulo N would imply (y+v)/(x+v)=y/x as multiplicative field elements, forcing x=y. Thus the required marked weighted return changes, as does its complementary return from p to e. No test of cycle multiplicity distinguishes the instances.

## There are linearly many disjoint choices

For any two fixed quarters I,J, their physical field elements are two intervals of multiplicative exponents. The number of X in I with X+v in J is

    (q−2)|I||J|/(q−1)² + O(√q log²q).

For completeness, expand interval indicators in the multiplicative characters on the cyclic group GF(q)*, extending all characters by zero at the field zero. An exponent interval has Fourier coefficient sum of absolute values O(log q) under the 1/N normalization. Indeed its jth geometric sum divided by N is at most min(|I|/N, 1/(N |sin(πj/N)|)). The two trivial characters contribute the displayed main term. Each other correlation Σ χ(X)ψ(X+v) has absolute value at most √q. Changing variables X=vY reduces it to the ordinary Jacobi sum up to a unit factor. With just one nontrivial character the sum has modulus at most one, as it excludes one otherwise complete character sum. For two nontrivial characters with trivial product use the bijection Y→Y/(1+Y) on the field excluding one to obtain modulus one. For nontrivial product, G(χ)G(ψ)=J(χ,ψ)G(χψ) follows by grouping the two Gauss sums by the sum of their arguments. Each nontrivial multiplicative-character Gauss sum against a nontrivial additive character has modulus √q: in its squared modulus put X=tY, sum over Y≠0, and use multiplicative character orthogonality in t. This proves the error term uniformly for the actual quarter endpoints.

Each of the two chord families therefore has q/16+O(√q log²q) members. Removing p and v costs O(1) candidates. Within each family the endpoints are disjoint because X→X+v is an involution and the two quarter sets differ. Across the two families the quarters are disjoint. Match first-family chords to second-family chords one-to-one until one family is exhausted. For sufficiently large k this gives Ω(K) alternatives, each changing two distinct table-pair entries, with disjoint changed sets between alternatives. A correct deterministic algorithm on the base table must probe at least one altered entry for every alternative. The proof is a table-inspection lower bound on the favorable base case. Reading the whole uncontracted table takes O(K) time on the same interface, so its order is sharp here. The all-unanimous, one-text shortcut still applies when only the synthetic primary is retained. [All-unanimous case](../table/README.md)

## Exact witnesses

The verifier generates primitive feedback masks, the full orbit and disjoint witness pairs for k=3 through 16. Through k=10 it independently reconstructs the stable LF and Ψ, lexicographically sorts every cyclic rotation of every tested switched text, verifies the entire L column and marked primary, checks all k-futures and the common two poststates, and measures the actual changed distance. There were 115 such literal switched BWTs plus eight base columns. At k=16 the fixed schedule produces 4,101 disjoint switch witnesses on 65,536 contexts, checked by orbit index differences. [Per-depth counts](checked.csv) · [Verifier](check.py)

```bash
e=$(dirname "$(rg --files | rg '/all-marks/exceptions/PROOF.md$')")
d="$e/acquire/ranges/bound/primitive"
python3 -B "$d/check.py" | diff - "$d/checked.csv"
```
