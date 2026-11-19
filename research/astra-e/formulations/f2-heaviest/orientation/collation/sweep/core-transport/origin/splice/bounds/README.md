# What deletion of q ordinary token ranks can change

Let an ordinary token BWT on the ordered section J have K_J nonempty source buckets and R_J maximal runs in the incoming column. A source bucket shares a word and sends its source ranks to increasing destinations. Keep H = J ∖ X with |X| = q. There is an exact ordinary token BWT for the first return to H with **K_H ≤ K_J + 2q** ordered buckets and **R_H ≤ R_J + 3q** incoming runs. Both increases are simultaneously sharp for arbitrary q, even on a single primitive cycle. The words assigned to different ordered IDs can be equal. Requiring a unique ID per spelling would lose the contiguous-source, stable-destination property in general.

## Proof and a tighter count

Write T for the original successor, E = {s in H : T(s) in X} for the surviving sources whose outgoing arc changes, and L = {T(x) in H : x in X} for the surviving destinations reached directly from a hole. Each path entering a nonempty group of consecutive holes exits at a unique landing in L. Hence |E| = |L| = k ≤ q. The arcs along these k paths are disjoint. An all-hole cycle contributes no entry or landing. If H is empty the new alphabet and column have size zero.

In surviving source order, make each s in E a singleton with the concatenated spelling along its changed arc. Split each old source bucket around these singletons and retain its remaining nonempty pieces. For each surviving untreated piece the old successor still lies in H. Its rank after deleting X is an increasing function of the old destination, so its induced successor is increasing. The new pieces and singletons, numbered in source order, form an ordinary alphabet. Let A be the number of old bucket names with untreated survivors and let u be the number of extra untreated pieces beyond those A. Each new separation needs a changed source between two untreated pieces of the same old name, giving u ≤ k. The canonical alphabet has exactly **A + u + k** buckets.

If the physical holes are a small union of intervals I, there is another bound on u. Inside an old source bucket, the sources with T(s) in any one interval I are contiguous, by stability. Removing source holes preserves that contiguity in the remaining source order. Thus u is also at most the sum over hole intervals I of the number of distinct old incoming labels on I. One physical hole interval can split several old buckets. Taking the incoming column `abcabcabc` and deleting its middle three slots splits the a and c source buckets, so u = 2 even though there is only one hole interval.

Start an independent incoming-column count by deleting the destination slots X **and** L. This leaves precisely the incoming symbols from untreated surviving sources. Deletion cannot increase the number of runs. Rename these symbols by their new source pieces. For each old label, its surviving incoming occurrences appear in original source order, so the renaming traverses its untreated pieces once in order. At most u boundaries can appear in previously equal adjacent symbols. Now reinsert the changed labels at L. Each is a distinct singleton source ID. A contiguous group of t reinsertions adds t runs, plus one if it separates equal renamed IDs which had been adjacent after deletion. Write b for the number of groups with equal flanking IDs, counting only groups with both flanks present. The canonical column therefore satisfies

```text
R_H = runs(untreated renamed incoming sequence) + k + b
    ≤ R_J + u + k + number of landing groups in surviving destination order
    ≤ R_J + 3k.
```

For the coarsest ordered alphabet, merge neighboring sources whenever their full new spellings agree and their induced destinations increase strictly across the boundary. Every legal ordered partition must keep the remaining boundaries. This merges whole canonical IDs, and replacing each old ID by its merged ID cannot introduce an incoming run boundary. Both upper bounds follow for this minimum partition as well.

**Source and destination positions differ.** For original incoming column `baa`, T = [1,2,0]. Delete original rank 2. The changed surviving source is 1, but its new word `ab` must be put in incoming slot 0. The other surviving source emits `a` into slot 1. Applying an edit at source rank 1 would change the wrong incoming symbol. In the general case L and E can overlap, agree, or differ.

## Two sharp families

For any positive q put m = 2q + 1. The tied-rotation family comes from the cyclic word (ab)^m. Its ordinary BWT is b^m a^m and T adds m modulo 2m. Delete m+1, m+3, …, m+2q−1. In the initial a source bucket, q isolated sources now spell ab, alternating with q+1 nonempty unchanged a pieces. The surviving b sources form one more bucket. The first m incoming slots alternate between b and the q new singleton labels. The remaining incoming slots show the q+1 different ordered a IDs. The resulting minimum K and R are 2+2q and 2+3q, while both original counts were two.

For a **primitive** family, take n = 2m+1, B_J = b^m a^(m+1) and delete the q ranks m+2j for 1 ≤ j ≤ q. The successor adds m modulo n and visits every rank since gcd(m,2m+1)=1. Following the single cycle gives a cyclic word with exactly this ordinary BWT. The changed sources are 2j and their distinct landing slots are 2j−1. As before there are q isolated ab sources, q+1 necessary unchanged a pieces and one b bucket. The first m incoming slots again contribute 2q+1 runs and the other surviving slots contribute q+1. This realizes equality simultaneously for **every** q on a single cycle and on the minimum ordered partition. Both families are sharp over arbitrary deleted subsets. Stronger hypotheses on which subsets a particular shortened-core policy can generate would need a separate realizability argument.

## Marks and exact tests

At a surviving mark on a token boundary the induced walker emits its ordinary new word sequence. At a deleted mark on a cycle meeting H, first emit the old prefix up to the first surviving source, then use the new walker. If the raw mark begins inside an original token, first emit the remaining suffix of that token and advance one old arc. A cycle with no surviving source must use its original words or an equivalent cycle record until the output reaches its prescribed byte length. The retained induced object alone cannot recover such a mark. For instance, the ordinary columns `ab` and `ac` have identity successors. Deleting their second rank leaves the same retained a object, while a mark on the deleted rank spells repeated b in the former and repeated c in the latter.

The independent program constructs dense first returns and their full concatenations. It verifies the safe and coarsest alphabet by stable-sorting each resulting incoming column. It independently deletes and renames unaffected destination slots, counts the exact contribution of changed landing groups, and checks variable-length old byte spellings at every in-word offset for more than two original output lengths. The exhaustive panel has 27,268 distinct column/deletion instances, 262,534 marked byte offsets, 16,298 cases with at least one all-hole cycle, and 446 empty sections. The proof and bounds concern the resulting representation. Locating the holes, computing rank/select, materializing changed spellings and handling marks are separate computational costs. [Validator](check.py) · [Raw receipt](small.json) · [Explicit source/landing, all-hole, and equal-payload witnesses](witnesses.json)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/splice/bounds
python3 "$d/check.py" | diff - "$d/small.json"
python3 "$d/check.py" --witnesses | diff - "$d/witnesses.json"
```
