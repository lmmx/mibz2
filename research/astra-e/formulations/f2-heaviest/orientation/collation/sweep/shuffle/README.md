# Two ordered masks recover the original return map after cuts move

Let U be a contiguous interval of m stable BWT ranks in a single first-column byte c. Write A_i for the bytes from source i∈U up to its next positive return τ(i). Ranks i inside U below refer to its order-compressed offsets. Choose bits s_i and move the cut to g(i)=Ψ^{s_i}(i). Require distinct moved cut positions, with physical section V=g(U). The theorem below permits arbitrary occurrence bits. The requested typewise and follower restrictions give a simpler corollary.

## Geometry and interface

Supply the old distinct words in **original source-bucket order**, with their multiplicities; this determines A_i by a range lookup. The earlier interval collation theorem obtains that order from the original word names and a rank itinerary in U. Also supply the source mask S[i]=s_i, the physical mask T[k]=s_{g⁻¹(V[k])}, and the exact incoming transformed word Z[k] at each physical V rank k. Token IDs indexing a spelling dictionary are sufficient. A transformed name need not identify an original parent type.

Stationary ranks are increasing in original U order; shifted ranks are increasing because Ψ is a stable select on the single F-byte bucket c. Thus σ(i), the physical V rank of g(i), is the position of the rank_S(s_i,i)-th copy of s_i in T, counting earlier entries from zero. Linear stable merging of unchanged coordinates with Ψ of the shifted coordinates constructs V, σ and T from U, S and one-step Ψ queries. No old positive-return permutation is needed. A colliding cut is exactly a length-one old arc whose first cut moves and whose successor cut stays. With no collision, one-byte motion cannot overtake the next cut in orbit order. The new arc from g(i) ends at g(τ(i)) and spells A_i[s_i:] followed by c when s_{τ(i)}=1. This also applies on a one-cut cycle.

## Recover original τ from physical incoming words

Set H_i=A_i[s_i:]. In PHYSICAL SOURCE order k, append σ⁻¹(k) to a FIFO keyed by its H_i. In ORIGINAL DESTINATION order j, take Z[σ(j)] and remove its last byte when S[j]=1. This byte must equal c. The resulting H identifies a queue. Pop its next old source i and record τ(i)=j.

To prove this pairing, every g(i) carrying H has exactly that H-prefix and Ψ^|H|(g(i)) is the old destination τ(i). Stable Ψ^|H| is strictly increasing on its prefix cylinder, even on tied rotations. Thus physical sources sharing H have their old destinations in precisely the same order. The argument covers H empty since Ψ⁰ is the identity. Two different old words H and cH may strip to the same H; they share a queue in physical order and are distinguished by the popped old source. Even context-dependent transformed names and coincident spellings of two old types are allowed under this exact two-mask interface. The returned τ can be materialized, or its old incoming token at j can be written as A_i to recover the original token last column and choose a compressed residual representation.

In the requested injective named case, when all followers of A have the same bit f(A) and a transformed ID identifies A and its new spelling, f(A)=|new(A)|−|A|+s(A). T[k] is therefore determined by the physical incoming ID. S is also determined by the original source counts and each type's bit. All A destinations have one color; σ preserves their mutual order. For each physical incoming A at k, its old source is C[A] plus the number of earlier physical incoming copies of A. Its old destination is the position of the number of preceding T[k]-colored physical visits in the corresponding source-mask color. This gives a one-pass stable-counting implementation with two source-mask iterators. Explicit allocation of two m-bit vectors is a sufficient price, not a necessary representation size.

For arbitrary bzip2 primary ranks, the induced data also needs a mark bridge. One option spells the original marked prefix up to the first hit of U, then uses τ and the original words, stopping after exactly n original bytes. Another supplies an old return occurrence and the mark's offset within its spelling. A mark on a cycle disjoint from U cannot be recovered from these section data. Original LF/Ψ fallback, an independently supplied marked cycle, or equivalent bytes are necessary. For example L=00001 and L=00002 with U=[0,1), q=4, all cuts stationary, have the same U words and masks but produce different marked outputs. No cycle-length divisibility is assumed.

## Accounting and scope

For known S with k moves, T can be stored in log₂ binom(m,k) bits plus rank/select support or as runs. In the abstract unrestricted two-list shuffle, distinct T give distinct source→physical permutations for fixed S. That is a conditional counting statement about the shuffle itself. Actual BWT geometry, supplied coordinates and label-derived T constrain it further. The two-mask theorem does not imply an unconditional lower bound of 2m bits for τ. Likewise a dense original successor is unnecessary for the one-step merge, but old type names, source counts, class dependencies, construction of physical incoming tokens and the mark are real inputs that must be produced and priced. Neither the theorem nor a dictionary geometry count removes that work.

[Independent two-mask implementation](interface.py) · [Exhaustive arbitrary-occurrence test](small.py) · [Receipt and witnesses](small.json)

## Fixed physical-geometry screens on Wikidata

The 20 original sections have 16–29 follower components for quote and 600–2,500 for the most frequent quote-return cylinder. On frozen ordinal 0 there are 19 classes for 2,716 quote types and 815 classes for 2,950 frequent types. The equivalence calculation scans original U ranks once with their original incoming and source type IDs; for each incoming type it unions all following types. This provides an actual class census once the original dictionary and incoming token column have been built. [Per-tape census](census.csv)

Six fixed policies per section include both uniform choices, the largest class by visit mass and its complement, and the singleton-group collection and its complement. Of 120 screens, 119 gave collision-free cuts and one distinct new spelling per old type. Moving singleton classes for quote on ordinal 24 collided. For all 119 cases we checked the reconstructed original successor at every source. On every tape and section, an independent LF walk to physical cuts discovered each incoming spelling directly and verified the original marked output bytewise, with 17,999,830 original LF row steps in total. Those audits selected the first nonuniform policy in fixed order. [All policies, mask runs, input runs, fragments and marked hashes](policies.csv) · [Reproduction program](real.py)

| Ten-tape section and policy | Dictionary bytes D | Proper-suffix nodes Q | Physical components | Source mask runs | Physical mask runs |
|---|---:|---:|---:|---:|---:|
| Quote, stay | 1,077,638 | 908,546 | 10 | 10 | 10 |
| Quote, shift all | 1,077,638 | 874,693 | 10,380 | 10 | 10 |
| Quote, move largest class | 1,051,947 | 881,185 | 7,557 | 360 | 20 |
| Frequent, stay | 1,926,457 | 1,296,362 | 10 | 10 | 10 |
| Frequent, shift all | 1,926,457 | 1,257,725 | 11 | 10 | 10 |
| Frequent, move non-singleton classes | 1,900,021 | 1,268,808 | 4,956 | 4,955 | 20 |

The mixed quote policy cuts dictionary bytes by 25,691 and physical fragmentation by 2,823 compared with the uniform shift; it adds 6,492 suffix nodes. The frequent mixed policy saves 26,436 dictionary bytes, but splits 11 fully shifted physical components into 4,956. No fixed mixed choice beat both uniform suffix-node counts on its own tape. The strongest Q could differ among other assignments. These results exclude original dictionary discovery, selector/follower construction, building the new physical incoming column, its ordering, and the marked inverse. The two-mask argument prices an algebraic interface, with no measured whole-stage or i9 improvement.

For binary last columns through n=7, exhaustive arbitrary occurrence bits gave 17,064 collision-free assignments and 111,502 exact marked outputs; 1,190 shuffles changed rank order and 9,276 assignments also met the original typewise, follower and injective-name restrictions. An independent transcript collision search through n=10 found no genuine distinct-τ pair with the same retained unlabelled data after dropping either or both masks under the original restrictions. This negative search establishes no necessity result. Dropping the physical incoming column altogether is demonstrably ambiguous: L=010 and L=100 with U=[0,2), both masks zero and ordered old words 0,01 of mass one each give the identity and the transposition respectively. [Transcript search](ambiguity.py) · [Counts](ambiguity.json)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/shuffle
python3 -B "$d/small.py" | diff - "$d/small.json"
python3 -B "$d/ambiguity.py" | diff - "$d/ambiguity.json"
python3 -B "$d/census.py" research/astra-e/dense/target/wikidata | diff - "$d/census.csv"
python3 -B "$d/real.py" research/astra-e/dense/target/wikidata | diff - "$d/policies.csv"
```
