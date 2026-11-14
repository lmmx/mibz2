# Labelled origin transport and boundary-local contraction

**Completed research comparison; revise both candidates.** Origin annotation
is exact without changing the delimiter set, but exchanges a small preliminary
search for more marker comparisons and loses early rejection. Stable interval
identities preserve every required split and final coalescing opportunity, but
this treap implementation introduces substantial tree traffic and metadata.
Neither result qualifies a speedup. Reusable sub-exchange summaries are
**deferred**: the prerequisite cost evidence does not support another candidate.

This follows [relay6798202916](https://github.com/lmmx/mibz2/pull/145#issuecomment-6798202916),
[scheduling clarification6799687151](https://github.com/lmmx/mibz2/pull/145#issuecomment-6799687151)
and Louis's direct instruction to complete the work. Earlier production and
real-block tasks remain pending, not completed by this research. Master base:
`2635f04e9155dd31388b24fd3f7248f87b3ddcda`; isolated branch
`research/codex-labelled-contraction`. No production, workstation, default,
policy, other-role state or graph/alternation work occurred.

## Sources and reproducibility

The [plan](PLAN.md) was committed at
`8d4e7eb1a95342b160a9e010b6afb7b40c9d1be1` before candidate results and posted
as [C1a6799878946](https://github.com/lmmx/mibz2/pull/145#issuecomment-6799878946).
The unchanged [vendor](vendor/) files are `phrase_induction.py` and
`batch_induction.py` at `af3327d8aee9fffe26d2d6de76818b6ac69cc145`.
The five [supplied](supplied/) files match every relay byte count and SHA-256
in [supplied-manifest.json](supplied-manifest.json). The ZIP container was not
supplied or verified. Both supplied scripts reproduced their saved JSON
byte-for-byte, twice: splice: 873 permutations/10,463 eligible deletion sets/
27,005 nested cases; origin: 7,172 cases/6,168 annotations/1,004 declines.

The implementation uses only Python's standard library:

- [origin.py](origin.py): instrumented search and marking variants of the
  original delimiter constructor, with its word DAG and token-order test.
- [contraction.py](contraction.py): faithful packed algorithm and stable
  two-order implementation. [common.py](common.py) defines count units/oracles.
- [freeze.py](freeze.py) generated [schedules.json](schedules.json) using only
  the original algorithm, before candidate execution. Its SHA-256 is
  `00ad44509c9c10519dc83d6e218153752bcf3acd7f26135171cfe17eb35f398f`.
- [run.py](run.py), [results](evidence/results.json) and compressed
  [per-case counters](evidence/case-costs.jsonl.gz) provide reproducible evidence.
  The initial passing results are preserved in [evidence/initial](evidence/initial/).

From this directory, using Python3.10+ **without `-O`**:

```sh
python3 supplied/check_splice.py > /tmp/labelled-splice.json
cmp supplied/results.json /tmp/labelled-splice.json
python3 supplied/check_origin_labels.py > /tmp/labelled-origin.json
cmp supplied/origin_results.json /tmp/labelled-origin.json
python3 run.py /tmp/labelled-reproduction  # destination must not already exist
cmp evidence/results.json /tmp/labelled-reproduction/results.json
cmp evidence/case-costs.jsonl.gz /tmp/labelled-reproduction/case-costs.jsonl.gz
python3 audit_results.py.txt
sha256sum -c SHA256SUMS
```

No fixture census/benchmark entrypoint in the vendor scripts is invoked. Python
3.12.14 validation passed, including Ruff and an independent audit of every
saved anchor/no-anchor verdict and every aggregate. Both comparison runs
finished by15:41:30UTC, within the frozen budgets. The five new source files
were formatted between runs; ASTs are identical. All measurements and compressed
case data reproduced byte-for-byte; only source-digest provenance changed.
The final evidence names the final formatted source. See [VALIDATION.md](VALIDATION.md).
Exact compressed-byte reproduction was checked on Python3.12.14/Linux; across
Python/zlib versions compare decompressed JSON as well, since gzip headers or
compression can differ without changing the measurements.
Ruff passes for the new research files. A separate repository-wide lint check
fails with264 existing violations in46 files; every affected file byte-matches
master. The [baseline lint record](evidence/repository-lint.json) preserves that
limitation. No unrelated lint cleanup was made.

## What is proved, and what is only suggested by the thesis

The relay's readings of thesis Lemma49/Theorem153 remain **relay-only**: HAL
access denied direct thesis verification. Earlier directly read author
[Paper I §3.4](https://arxiv.org/pdf/1705.01641v2) and
[Paper II §§2,5](https://arxiv.org/html/1801.05973v2) concern compatible labels
and specified permutation operators. No theorem from them is transplanted to
weighted LF. The correspondence here is a design suggestion: preserve attachment
identities while changing a local representation. The arguments below are
elementary first-return facts, independently of that literature.

Let F be a finite permutation and A its retained rows. Consecutive visits to A
partition each cycle meeting A into excursions, including each starting anchor
and excluding its ending anchor. Thus a marked row p occurs in exactly one
excursion, or its cycle misses A. If a queued interval at coordinate x, source
anchor rank s and prefix word w contains p, record
`(q,h) = (s+p-x, length(w))`. Its particular anchor a satisfies `F^h(a)=p`.
This is an **occurrence**, independent of shared grammar-word identities.

If the full forward cycle emitted from a is E, the inverse-LF cycle from a is
`reverse(E)`. The cycle from p is its left rotation by `-h mod |E|`; repeat and
truncate to the original n. This covers periodic inputs and cycle lengths not
dividing n. Failure to encounter p returns the same decline as the original
constructor. A decoder must still take its existing fallback; neither prototype
implements a new production fallback.

The delimiter section is unchanged. Its dictionary, lexicographic token order
and exact return map match the original for every successful case. We retain
the supplied `bba`, origin1, delimiter `a` counterexample: adding origin1 as an
anchor would give tokens `b,ba` whose token LF is identity although the induced
map exchanges the rows. Marking during construction avoids that invalid change.

For local contraction, let D satisfy `D ∩ F(D) = ∅`. From surviving x, the next
survivor is F(x), or F(F(x)) when F(x) lies in D. Concatenate exactly those
transition words. This proves destination, time and itinerary preservation.
The original color rule selects incoming word-color0/outgoing word-color1,
so no selected point immediately follows another selected point; excluding the
origin preserves this property. For nested sections B⊆A, visits to B are a
subsequence of visits to A: `(F_A)_B = F_B`, with word concatenation.

Packing by increasing retained rank h merely conjugates the map:
`F_packed = h F_sparse h^-1`. Our stable representation stores two endpoint
orders and widths, rather than every current coordinate. It is **implicitly
packed**, not affine in the original coordinates across deleted holes. This
distinction lets it coalesce across those holes without rewriting every piece.
Source/image boundary crossings split stable edges. A splice retains the
incoming source endpoint and outgoing image endpoint, deletes the other two,
and concatenates words. Only changed adjacencies need coalescing checks. Two
equal-word edges merge iff adjacent, in the same order, in both endpoint orders.
Split/merge operations transport the origin's `(edge identity, offset)`;
the selected origin is never removed. Its word ID is not its routing identity.
Unchanged edges retain their identities; splits allocate and merges retire
identities. Only the single selected external marker is maintained here.
Forwarding other cached references to retired identities would require
additional state/work, not supplied free by the word DAG or this prototype.

This generic batch contraction makes **no token-BWT claim** for its arbitrary
retained section. The delimiter/token claim belongs to the unchanged delimiter
construction above. No claim that arbitrary local surgery preserves token
ordering is needed or established.

## Fixed comparison and correctness evidence

New comparisons use all binary columns through length6 and ternary through
length4, every origin/delimiter, and the fixed seven synthetic families at
sizes1,2,4,8,16,32,64. Raw arbitrary columns include multiple cycles; periodic
families use cyclic BWTs. Four original color rounds per case share exactly
the frozen deletions. Counts include initialization and final byte output.

| Check | Result |
|---|---:|
| Origin cases | 7,021 |
| Successful origin/byte/token-map checks | 6,227 |
| Correct declines | 794 (336 absent delimiter;458 unanchored selected cycle) |
| Contraction cases | 3,108 |
| Exact contraction rounds | 12,429 |
| Deleted rows across schedules | 76,751 |
| Independent row-oracle transitions checked during contraction | 894,922 |

Every round agrees with the unchanged original constructor and the instrumented
packed copy, including canonical piece records, grammar nodes/IDs and marker
position. Each represented return also matches independent row LF in destination,
byte itinerary and time; every final decode matches inverse-LF bytes. Thus the
stable version loses **no final coalescing opportunity** in these tests.
The [saved audit](audit_results.py.txt) separately checks both successes and
declines using individual-row stable sorting, without either LF run builder.

PR369 is regression evidence, not rediscovered work: `ba`, `baa`, gray-only and
nondividing cycles, and the long excursion remain covered. Its102 costly cached
updates were not rerun. The first eligible witness, [babaaa](pr369-adversary.json),
was recovered from the old model before candidate data: retain runs[0,1,3],
delete run0, old cached5 versus fresh4 visits. Our single frozen splice matches
the packed result. It also has two occurrences of word `ab` with different
destinations, directly ruling out word content alone as a routing-summary key.

## Whole-construction count trade-offs

Numbers below are sums across cases, **not simultaneous memory**. They are
logical operation counts, not instruction counts, timings or a combined score.
Oracle/assertion/export work is separate. Global selection remains global in
both algorithms and is charged; no precomputed schedule is used to skip it.

| Origin cost | Separate search | Construction marking |
|---|---:|---:|
| Separate LF steps | 7,486 | 0 |
| Separate search queries/comparisons | 13,713 /26,115 | 0 /0 |
| Marker checks/comparisons | 0 /0 | 64,849 /100,574 |
| Constructor interval steps | 63,513 | 64,849 |
| Grammar nodes created | 16,532 | 16,894 |
| Output bytes | 494,467 | 494,467 |

The extra1,336 constructor steps happen on declined cycles. For `(ab)^64`,
origin0, delimiter `b`, the old search rejects after one LF step and eight
binary comparisons; marking constructs127 interval steps, performs1,778
constructor-search comparisons and creates six grammar nodes before declining.
Even `b a^64`, delimiter `b`, origin64 trades65 separate lookups/66 comparisons
for65 marker checks/130 comparisons. Each successful annotation stores one
two-scalar occurrence record; prefix length comes from the existing word DAG.
The DAG's avoidance of PR369's quadratic flat-word copying is **already in the
baseline**, not a gain invented by marking.

| Contraction cost, including setup/output | Packed | Stable orders |
|---|---:|---:|
| Selection input visits | 222,766 | 222,766 |
| Overlay records | 118,213 | 118,213 |
| Sort comparisons | 406,949 | 99,440 |
| Persistent coordinate fields written | 105,708 | 0 |
| Selection coordinate prefix additions | 0 | 222,766 |
| Packed rank queries/comparisons | 50,030 /173,944 | 0 /0 |
| Tree rank queries/ancestor hops | 0 /0 | 24,803 /25,116 |
| Tree predecessor queries/visited nodes | 0 /0 | 29,456 /116,915 |
| Tree pointer/size writes | 0 /0 | 883,875 /893,297 |
| Record allocations | 52,854 pieces | 37,934 edges +75,868 tree nodes |
| Grammar nodes created | 2,774 | 2,774 |
| Final map records summed across cases | 24,212 | 24,212 |
| Output bytes | 223,735 | 223,735 |

Stable maintenance performs8,895 splits,8,541 splices and5,181 merges. The
packed implementation performs185 final merges; stable intermediate splits
and merges can cancel before reaching the identical canonical map. Word
interning follows the original selection order so later color seeds remain
identical. Stable application additionally pays8,541 prepared-word assignments
and8,541 lookups. Output walks directly through the trees, charging rank and
predecessor operations; the validation export is never its execution path.

An edge has five fields and each of its two tree nodes six, versus four fields
per packed record. Cumulative record-field capacity is therefore644,878 versus
211,416 slots, before dictionaries, marker state or Python object overhead.
Peak *active* stable edges/nodes are128/256; packed peak live piece records322.
Peak transient vector entries are417 versus577, but element sizes differ.
Retired stable objects can remain referenced by selection snapshots or await
Python cyclic GC: active counts are **not heap measurements**. Cumulative
allocations expose that retention risk; no memory saving is asserted.

The per-case evidence also charges word expansion, sorts, queries, local
adjacency checks, rotations, marker updates, repeated output and temporary
index records. `tree_neighbor_hops` counts loop hops; query counts also cover
boundary-link operations. Input run discovery, fixture generation, frozen
schedule generation, validation and instrumentation overhead are common/test
setup, not attributed savings. Object allocation latency, Python memory bytes
and dictionary internals are unmeasured. There is no deferred construction
phase hidden beyond the counted global selection and final output.

## Recommendations and cheapest falsifiers

1. **Origin marking: revise, retain the correctness invariant.** It removes
   a separate traversal but is not a cheaper general replacement in this census.
   Before another implementation, price a marker test shared with an existing
   interval-boundary test, disabled after finding the occurrence, against these
   same saved cases. The no-anchor `(ab)^64` case must remain; ignoring its
   wasted construction would falsify the claimed saving immediately.
2. **Stable identities/local contraction: revise the representation, stop this
   treap backend as the proposed next production candidate.** Avoided sorts and
   coordinate writes are real, but initialization, tree updates, intermediate
   splits/merges and metadata are substantial. Cheapest next falsifier: use the
   existing trace to price batch-local boundary edits instead of per-splice
   split/merge. If most of the tree writes remain, do not build another backend.
   This is an engineering recommendation, not a proof that stable labels cannot
   help or a wall-clock rejection of the LF research line.
3. **Reusable summaries: defer.** The prerequisite experiment gave no convincing
   inexpensive maintenance path. No extra component experiment was launched.
   A later equal-width two-port summary would have to state positive equal
   lengths, exact exit offsets, word orientation and marked-occurrence phase;
   unequal lengths require boundary refinement and zero width is an empty case.
   Its recognition/build/application costs must beat direct processing. The
   existing distinct-route `ab` occurrences already reject content-only reuse;
   permutation shape alone is also insufficient. This is a future contract,
   not a proved economical summary or a new authorised assignment.

The next Coordinator decision is disposition of these research recommendations.
Independent mathematical review and direct thesis-page verification remain
absent. No production compatibility, B5, performance qualification, automatic
restart or completion of an unrelated inbox assignment is claimed.
