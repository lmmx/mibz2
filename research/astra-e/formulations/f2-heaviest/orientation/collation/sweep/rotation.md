# Moving the entire section forward preserves its residual permutation

Let U be any nonempty set with a common outgoing byte c on a labelled permutation Ψ. Put V=Ψ(U). A source x∈U returns after l>0 steps. The source Ψ(x)∈V returns after exactly l steps, since Ψ^j(Ψ(x))∈V iff Ψ^j(x)∈U. This remains true on every cycle meeting U and V. If the U word at x is c·v, the V word at Ψ(x) is v·c. Its last byte is the outgoing byte at the original return destination, which also lies in U. The rotation map is one-to-one on this family. It preserves each type's multiplicity, K and D.

In a stable byte BWT, Ψ is strictly increasing on ranks in one F-byte bucket. It therefore sends the sorted list of U to the sorted list of V. Conjugacy gives the same first-return permutation on their order-compressed positions. The source buckets of U and the incoming word sequence on its destinations carry over entry-for-entry under the spelling rotation, even when V has holes. This result uses order preservation on the *whole preimage*. Moving each range through different bytes would require a separate analysis. The marked starting position and the output fragment until the selected section still have to be recovered.

## Exact suffix-node savings

Let T be the set of tails v of the distinct U words c·v. If some tail is nonempty, let B be the set of all nonempty suffixes of members of T. Let M count the nonempty members of T that are maximal under the proper-suffix relation on T. A U word contributes all suffixes of v to its proper-suffix set, so Q(U)=1+|B|. The nonempty proper suffixes of shifted words v·c are the one-byte word c and t·c for every nonempty proper suffix t of a tail. The latter set of t omits exactly the M maximal nonempty tails from B. Thus

```text
Q(U) − Q(V) = M − 1,     0 ≤ Q(U) − Q(V) ≤ K − 1.
```

If every tail is empty both dictionaries reduce to the one-byte word c and Q=1. Zero savings, intermediate savings and the full K−1 savings all occur on stable columns even when both sections are contiguous. For example Ψ=[1,2,0] with outgoing bytes [0,0,1] and U=[0,2) has two return types and no savings after the shift. The exhaustive check covers 42,754 monochromatic section moves, including 10,648 holed images, and preserves witnesses for all three regimes. [Exact independent rotation checks](rotation.py) · [Small witnesses](rotation-small.json)

For repeated shifts, require a common outgoing byte throughout each current section. Induction preserves the same compressed residual permutation and gives a Q saving of at most h(K−1) after h shifts. A contiguous U remains contiguous across one step precisely when its selected ranks fit in a single affine original select run within the common F-byte bucket. Two original endpoint translations and an equality of widths check that condition. A fragmented V still inherits the induced order, at the cost of representing, locating and advancing its components. A one-point cycle can return to an earlier section. An implementation searching over shifts can stop when the set repeats.

## Frozen shift chain

The named compound covers ranks [21,229,45,394) and its common forward prefix consists of three bytes, quote, colon and quote. The input tape supplied the original stable column. The independent verifier rebuilt full orbit arcs, their compressed destination index and the spelling at every source for every shift. After the third shift the section spans 60 outgoing byte types.

| Forward shifts | Rank intervals | K | D | Q | Q saved at this step |
|---:|---:|---:|---:|---:|---:|
| 0 | 1 | 2,950 | 146,183 | 100,644 | — |
| 1 | 1 | 2,950 | 146,183 | 97,695 | 2,949 |
| 2 | 135 | 2,950 | 146,183 | 94,746 | 2,949 |
| 3 | 511 | 2,950 | 146,183 | 91,849 | 2,897 |

The first image begins at 336,231, the first position of an original quote run of length at least m. It can be selected from the named U using one select-run lookup. At depth three the dictionary saves 8,795 proper-suffix nodes while introducing 510 extra rank intervals. A decisive construction test would convey those images by LF-preimage membership against the single named interval or explicitly search their sorted components, charging recognition, original-run clips, publication, marked alignment and emission. No decoder timing follows from the reduced Q alone. [Exact rank and spelling checks](rotation-frozen.json)
