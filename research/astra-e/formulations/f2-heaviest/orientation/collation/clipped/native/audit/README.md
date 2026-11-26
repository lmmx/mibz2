# Where the two possible mixed branches occur

Let U = [a,b) be a nonempty rank interval, m = b−a, and let Ψ be the inverse of stable LF. At a current row x, a backward step takes byte c = L[x], moves to LF(x), and terminates precisely when a ≤ LF(x) < b. Neither eligibility nor the byte depends on the parent trie node. All statements below include stable columns with periodic or disjoint cycles.

**Preimage tagging.** The kth occurrence of c in L maps to rank C[c]+k in its F bucket (k starts at zero). Intersect the consecutive F ranks of an original run with U to tag the corresponding input positions. For a boundary e in {a,b} strictly inside an F byte bucket, let x− = Ψ(e−1) and x+ = Ψ(e). They are consecutive occurrences of that byte in L. There is an additional interior cut of a maximal L run before x+ precisely when x+ = x−+1. If the preimages are separated by rows of other bytes, the two equal-byte occurrences belong to distinct original runs. If e is 0, n or an F bucket boundary, it introduces no same-byte run cut. The maximal (byte,termination) run count is therefore exactly r plus the number of additional interior cuts, at most r+2. Two cuts can split the same input run into three pieces.

The capped-input adapter can do the same work while scanning its records. Split each record at boundary ranks strictly inside its F image; a boundary aligned to the cap end changes the tag for the following cap. Coalesce consecutive records whenever both byte and tag agree. This yields the same normalized strips regardless of cap length. The F-byte shortcut only needs a partial classification for the one or two byte buckets containing a strict section endpoint. A section may cross all 256 F buckets.

**Which (parent,byte) pairs are mixed.** A positive backward walk beginning at each destination in U takes steps until its first return to U. The sets of pre-step rows of these walks are disjoint. On a cycle they partition the original section-to-section arcs with each section root included once. Consequently each visited original row belongs to a unique live parent, even across depths. A parent contains one interval I by induction. For byte c the LF images of all c occurrences in I form one consecutive F interval J. Its width is at most |I| ≤ m. If it contains both terminal and live rows, exactly one endpoint of U lies strictly inside J. In particular J cannot contain both endpoints strictly: that would require width at least m+2. The live complement of the terminal part of J is a single interval, completing the induction.

For each eligible e, its unique possible mixed branch exists **if and only if** x− and x+ above are processed in the same live parent. This follows because crossing e requires both consecutive occurrences, and the two pre-step rows already determine the parent and byte uniquely. The criterion still applies when intervening input rows belong to other bytes. Each e accounts for at most one mixed (parent,byte). Both can belong to different byte branches of the same parent; if both are in the same F bucket they require different parents. A singleton section has none. A section-disjoint cycle can contain the partner preimage or divide it from its mate, leaving a physical input split with no mixed branch.

The streams for a parent are merged in the original order of its selected occurrences. Within one c branch, LF preserves their order. Its terminal and live occurrences occupy consecutive portions, so prepending or appending the terminal word segment to that child's stream gives the correct selector consumption across every input-run and cap boundary. A branch that is wholly terminal contributes no continuation state. A parent can have both a terminal byte and a continuing *different* byte without having any mixed (parent,byte) pair.

**Three physical counts and the normalized output.** A physical split counts an endpoint strictly inside a maximal input run's LF image. A mixed parent/run piece counts a split strictly inside a single input run after restricting it to one live parent. It occurs exactly at globally mixed boundaries for which x− and x+ are adjacent in L. A mixed (parent,byte) branch can additionally draw them from separate runs. The destination-ordered token run count counts maximal equal incoming words after the entire construction. Every internal boundary in that final stream walks a pair of adjacent rows backwards through equal bytes and equal return decisions until the first tagged L seam. Distinct root gaps cannot first hit the same seam: in reverse, the earlier root visit would stop the other pair. Thus normalized root runs are at most normalized tagged L runs. This last bound and its sharp witness are already in the earlier reverse-gap work, pinned to d849241. [Reverse-gap argument](../../../../../../f2-dual/README.md)

| Stable column L | U | Extra physical cuts | Full input runs split | Mixed parent/run pieces | Mixed (parent,byte) | Root token runs |
|---|---|---:|---:|---:|---:|---:|
| 000 | [1,2) | 2 | 1 | 0 | 0 | 1 |
| 101010 | [1,5) | 0 | 0 | 0 | 2 | 4 |
| 11000000 | [2,5) | 2 | 1 | 2 | 2 | 3 |
| 111000 | [1,5) | 2 | 2 | 2 | 2 | 4 |

The third example puts both possible cuts in the same maximal input run. The left boundary is crossed by the root's 0 branch at input ranks 2–4. The right boundary is crossed by the 0 branch of backward prefix 01 at input ranks 6–7. The fourth example attains r+2 output runs. The dense audit forms each live parent's membership by walking original rows from every destination; it does not call the native or structural clipped-interval constructor.

All binary columns through length nine, all additional ternary columns through length seven, and every nonempty interval give 113,448 exhaustive instances. They contain 26,992 cases with one mixed branch and 1,690 with two. There are 9,575 instances with two full-run interior cuts and 630 with two actual mixed parent/run pieces. The program checked cap sizes 1, 2, 3 and 127 on each input, plus 12 larger cases crossing actual 127-byte cap boundaries or as many as 256 distinct first symbols. It also checks that each reported mixed branch is predicted exactly by its two original-row preimages.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/clipped/native/audit
python3 -B "$d/endpoints.py" > /tmp/astra-clipped-endpoints.json
cmp /tmp/astra-clipped-endpoints.json "$d/exhaustive.json"
```

[Independent audit program](endpoints.py) · [Counts and first witnesses](exhaustive.json)

## Detecting and naming the two exceptions without building the trie

For a visited pre-step row x, its live parent's backward spelling is the reverse of the raw forward bytes from x to its nearest U row, including x when checking whether a root is already reached. Compare these two forward strings for x− and x+ at each eligible endpoint. A byte disagreement, only one rank entering U, or a complete cycle missing U rules out a mixed extension. When both arrive together without a byte disagreement, the common parent and extension byte identify the unique mixed branch. At each step the rank separation of equal-symbol paths cannot shrink. If it exceeds m−1, both ranks cannot finish in U, so the comparison can terminate early.

This predictor recovered the independently observed branches on every synthetic case and the published total on each of 40 systematic block/section combinations. For the 20 endpoints of each real section family, quote used 40 forward rank probes (bucket boundaries alone suffice), frequent 80, fifth 594 and half 438. Each arbitrary family had 16 mixed endpoints. The longest observed probe used 164 forward rank accesses, recovering a depth-81 parent. This count assumes a Ψ/select facility. The accompanying adapter builds an indexed table of r forward strips in O(r) time and space, then uses binary search per probe. An implementation having only an LF lookup table must price the additional select work before using this test for dispatch.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/clipped/native/audit
python3 -B "$d/preflight.py" research/astra-e/dense/target/wikidata > /tmp/astra-clipped-preflight.csv
cmp /tmp/astra-clipped-preflight.csv "$d/wikidata.csv"
```

[Direct endpoint predictor](preflight.py) · [All 80 real endpoint traces](wikidata.csv)
