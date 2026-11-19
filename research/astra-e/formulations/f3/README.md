# F3 · Can bzip2 inversion split into independent alphabet shuffles?

**Object and intended saving.** Divide the alphabet into two consecutive nonempty intervals, coarsen each byte of the last column L to its group bit B, and stably filter L into the two within-group byte strings. The proposed decoder would invert a binary skeleton and the two projected texts independently, then interleave the child outputs. That would allow three smaller, potentially parallel inversions in place of global marked LF navigation. All three input columns are available in one partition of the actual run records. The issue is the rotation order inherited by the children.

## Three-row obstructions

Use groups {0,1} and {2}, coding the latter 1. All text examples use cyclic bzip2 rotations. The marks below use zero-based row ranks.

* For primitive text `012`, its column is `201` and primary 0. Keeping only bytes 0 and 1 gives subcolumn `01`. Its only possible two-byte inverses are `00` and `11`. The required projected text is `01`, whose own column is `10`. Allowing any new child primary does not repair the stable subsequence.
* For primitive text `102`, its column is `120` and primary 1. Coarsening the column gives `010`. Its possible marked texts are `000`, `010` and `101`. Coarsening the actual text yields `001`, whose own column is `100`. The skeleton can itself be wrong for every primary.
* In arbitrary column `210`, primary 1 is a fixed point. The marked output is `111` and the group skeleton must be `000`. Coarsening produces column `100`, on which every possible mark emits a rotation of `001`. A rule using only the global group population also predicts the wrong marked multiplicities.

These primitive examples have the smallest possible text length using three distinct symbols. They exhibit the missing order even when child marks are free. [Exhaustive checks and exact receipts](exact.json)

## An exact factorization and its limit

Let S sort a sequence stably by symbol, viewed as the permutation from row to stable-sort destination. On L this is LF. Let β be the stable destination map of B. Let D be the block-diagonal permutation whose two diagonal blocks are the stable destination maps of the two stably filtered child columns. β sends row x to the start of its group plus its occurrence number among earlier rows of that group. D then sorts this group by true symbol, preserving equal-symbol occurrence order. Therefore `LF_L = D ∘ β`. In the forward direction write `τ = β⁻¹`, `σ = D⁻¹`. The real forward successor is `Ψ = τ ∘ σ`. The three columns encode the original column losslessly. The factorization requires composition at each step and supplies neither a marked output coordinate nor an independent projected child cycle.

There is an exact condition for all starts to use the uncorrected binary skeleton. Let λ(x) be x's first-column group, and let `K(x) = λ(x) λ(τx) λ(τ²x)…` be its binary future under τ. **The two marked group codings agree for every start if and only if `K(x) = K(σx)` for every x.** If σ preserves these futures, then `K(Ψx) = K(τσx)` is exactly the suffix of `K(x)` after its first byte. Induction traverses that suffix for any length. Conversely, if the true coding is `K` at every start, the future after x's first byte is `K(Ψx)`, hence also `K(τx)`. Put λ(σx)=λ(x) in front of `K(τσx)=K(τx)` to obtain the condition. Infinite periodic futures can be compared by length n here. Two states on one cycle need at most its period; two different cycles have period lengths summing to at most n, so the Fine–Wilf bound suffices. This condition concerns the unchanged starting rank for every mark. A single marked orbit could pass even if another start fails.

When the condition holds, the binary skeleton can be computed on τ. Independent byte colours still require the genuine first-return successor of Ψ on the two first-column groups. Starting the two child cursors at the first *actual* visits to their groups gives an exact shuffle for the marked component. A group absent from this component requires no cursor. Computing, ordering and locating those returns is work. For group g, the old first-return restriction lemma already explains why a retained stable subcolumn can require a nontrivial transport P. Any decoder seeking an independent child needs to compute such transport or find a representation where the child emits the same output without it. The phrase/wavelet result from the first Astra investigation treats structured phrases with boundary recognition; its order property does not extend to arbitrary alphabet deletion. [Open-Return restriction and transport](https://github.com/lmmx/mibz2/blob/2b53453e124ffdc75cc4d2141b833a319e3cfd73/docs/journal/2026-11-18-open-return/proof.md) · [Earlier wavelet and phrase investigation](https://github.com/lmmx/mibz2/blob/d4642042ccf8d8d9fcc420c1717eb1bbb25fc145/docs/journal/2026-11-18-astra-investigation-1.md)

`exact.py` exhausts all 1,092 ternary columns of lengths 1 through 6 and 6,015 marks. It tests the permutation factorization, exact count of changed successor entries, all-mark equivalence criterion and the histogram mismatch formula below. There are 896 multiple-cycle columns. Only 661 columns have the all-mark binary coding preserved. The independent rotation sort constructs the primitive counterexamples' true BWTs. [Program](exact.py) · [Output](exact.json)

## Cheap structural falsifier on real inputs

Let m be the number of lower-group rows. For true symbol c, write `f_c` for its frequency and `t_c` for its count in the first m positions of L. Inside the true F bucket for c, the Ψ images are increasing. Exactly its first `t_c` positions have next colour zero. Inside a binary F bucket g, the corresponding τ sequence has its first `T_g = Σ_{c∈g} t_c` positions coloured zero. The two sets have equal cardinality. Their symmetric difference counts **all** ranks whose very next colour differs. In group-local coordinates with `s_c = Σ_{d<c,d∈g} f_d`, the count is `2 (T_g − Σ_c |[s_c, s_c+t_c) ∩ [0,T_g)|)`. Sum it over both groups. This uses byte frequencies and one prefix histogram, without rank/select tables or a full LF walk.

A separate count prices a literal correction to the binary successor table. Since τ is injective, Ψ and τ differ at exactly the support size of σ, also the support size of D. Within a run of c in the child last column, D is a fixed translation. In one run scan, subtract the number of group characters already seen from `s_c` plus the count of c already seen. A nonzero difference changes every row of that run. Coalescing adjacent equal differences in each child's order counts the actual translation bands. This is an exact price for explicit point edits to the specified table. Other corrections can store band boundaries, compose multiple edits, or preserve only a marked coding.

A prespecified balanced split chooses the byte cut closest to half the block by frequency, breaking ties toward the smaller cut. The first three large blocks in the already spaced Astra B verification sample are stream ordinals 1, 41 and 85. The cutoff is byte 96 on all three. Each reconstructed capped run tape independently matches Astra B's published SHA-256. The final column compares the class sequences starting at the actual primary, using run intervals for select. [Pinned sample and hashes](https://github.com/lmmx/mibz2/blob/c42e1c69cdad486834048434438976b0681c4493/docs/journal/2026-11-18-astra-b-collective/verified.csv)

| Stream ordinal | Rows | Changed successors | Correction translation bands | Wrong next colours | First wrong marked byte, zero-based |
|---:|---:|---:|---:|---:|---:|
| 1 (frozen) | 899,931 | 899,931 | 50,133 | 322,406 | 2 |
| 41 | 900,000 | 899,999 | 124,989 | 323,528 | 2 |
| 85 | 900,000 | 900,000 | 61,611 | 318,354 | 1 |

**Decision.** The exact marked colour sequence already diverges within three bytes on each real block. On frozen every successor changes. A per-row correction table is as large as an explicit original successor table before recording the binary skeleton. The band correction retains 50,133 bands versus 51,184 original runs. It now also requires binary select and interleaving. These costs reject independent stable-subsequence inversion and the sparse point-patch salvage for this balanced partition. A choice of input-dependent cuts or a direct compressed first-return transport remains open. Such a construction needs a cheap test of the actual marked route and a price for finding the order and producing output. No wall time, cache or i9 claim follows from this screen. [Records](wikidata.csv) · [Counter](screen.py)

```sh
python3 research/astra-e/formulations/f3/exact.py | diff - research/astra-e/formulations/f3/exact.json
bench/moonshot/target/release/moonshot extract --file corpora/wikidata.bz2 --blocks 1,41,85 --out target/f3.sym
python3 research/astra-e/formulations/f3/screen.py target/f3.sym | diff - research/astra-e/formulations/f3/wikidata.csv
```
