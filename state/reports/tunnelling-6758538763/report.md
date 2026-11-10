# Tunnelling: repair the certificate before pricing the walk

Inbox **6758538763**, dispatching **6758215880**. Specifications pinned at
`a2da49638ac54d7763de1c167bd054f67ffdc763`. Reviewed Pathfinder's journal and
`examples/phrase_census.rs` at **f666ea9f3e611e114211c58172d5d2482ce15010**.

**REVISE the census and the equivalence claim. Keep the hybrid open.** The census's
whole-cell word is one byte too long. Its cost ranking therefore does not establish
an absent advantage. A fixed-k table and tunnelling exploit the same local parallelism,
but are different representations with different selection, construction and navigation costs.
No decoder implementation, compilation, corpus timing or production qualification here.

## 1. A valid BWT refutes the word and landing certificate

Use zero-based cyclic BWT of `aab`: `L=baa`, `F=aab`, `Psi=[1,2,0]`.
The one-step cells are `[0,2)` and `[2,3)`; their start set is `{0,2}`.
The source's recurrence computes `K=[0,1,0]`.

| start row | first emitted byte | next row | second byte | row after two steps |
|---|---|---|---|---|
| 0 | a | 1 | a | 2 |
| 1 | a | 2 | b | 0 |

The multi-row cell supports **one** common byte and one affine step. The source gives
it `raw=min(K[q]+1)=2`. Its supposed common word would be `aa` at row 0 and `ab` at
row 1; translating the two-step target from row 0 predicts row 3 instead of row 0.
This is not a sentinel-only artefact. The sentinel-terminated `aa$` fails too.

The defect is at `phrase_census.rs:398` (`split_*`) and `:435` (`TUNNEL`):
both add one to the certified length. The TUNNEL loop at lines 455–474 advances
`t` through the precomputed correct trajectory; it neither copies a representative
word to verify its bytes nor checks the translated landing. The earlier equality
check of the ordinary trajectory against `bwt::walk` cannot validate these jumps.

**Re-derivation.** Let S be starts of maximal intervals on which F is constant and
Psi is a translation. For q>0 define

    K(q) = min { t >= 0 : Psi^t(q) belongs to S }.

Until that first boundary, the two adjacent rows stay adjacent, emit equal bytes,
and have a translated successor. Hence a non-singleton cell [a,b) has safe common
word/affine-landing length

    ell = min { K(q) : a < q < b },

not ell+1. The extra state at time ell is an endpoint, not another certified
emission and successor. Singletons can be extended separately, but the proposed
policy explicitly makes them one-byte steps.

`certificates.py` independently constructs BWTs by sorted rotations. On **510** binary
strings of lengths 1–8 with a unique sentinel, it checks the source's reverse-trajectory
K recurrence against direct boundary hitting, the ell-byte certificate, and the
fixed-k equivalence below for k=1..4. All checks pass; **969 multi-row cells** refute
the ell+1 certificate. These are finite mathematical checks, not corpus reproduction.
Run `python certificates.py`; results are in `certificates.json`.

## 2. What D shares with tunnelling, exactly

For the labelled k-step map, the cut set is

    S_k = union_{j=0}^{k-1} Psi^{-j}(S).
    r_k = 1 + #{q in [1,n) : K(q) < k}.

Two adjacent rows share a piece iff K(q)>=k. Each piece has one k-byte word and
`Psi^k(i)=base+i-first`. Conversely, equal words keep the two trajectories in the
same symbol bucket at each step; stable select cannot shrink their positive row
distance. An adjacent final pair therefore stayed adjacent throughout. This proves
the maximality for **word plus endpoint**, not for the endpoint permutation alone.
For cyclic `aab`, Psi^3 is identity everywhere but three-step words differ: a map-only
piece can be much larger than a decoder's labelled piece.

Reverse a certified Psi path to obtain the corresponding LF path, reversing the
word and using its image interval as the start. Thus the labelled LF/Psi fixed-k
partitions correspond and have equal counts. A greedy variable-length walk does
**not** inherit equal visited-cell frequencies from reversing the direction.

The defensible wording is **D is a fixed-length word-and-translation table over
parallel paths**. D4-delta packs the four bytes and a 20-bit modular translation;
it retains the original row space. It neither removes BWT rows nor chooses a mutually
compatible collection of tunnels. Its pieces partition start rows but their paths
can overlap, even with themselves on cyclic inputs. Those overlaps are harmless for
a table of separately valid jumps; they are not a licence to collapse all those paths.
Carrying an original row/offset avoids a tunnel stack precisely because it does not
identify the original rows. That benefit is real; it is not a new theorem about
compressed tunnel navigation.

## 3. Primary literature and what its numbers mean

**Baier, CPM 2018**, [paper](https://doi.org/10.4230/LIPIcs.CPM.2018.3), Definitions 8–10,
Algorithm 1, Theorem 13, Figure 5/Table 1: height is the number of parallel rows;
width is the number of equal-symbol columns. Pathfinder interchanges these names.
Tunnelling retains the top row and both endpoint columns, with two marker vectors;
inversion saves offsets on a stack for compatible nested collisions and performs
generalized LF per emitted character. Linear inversion uses precomputed LF and
nearest-unmarked-predecessor arrays. It is not one LF operation per entire word.
Average encoded-size reductions are **8.4% bwz, 11.26% bcm, 16.48% wavelet-tree**.
For 1,024-MB English: bwz **1.84→1.66**, bcm **1.56→1.34**, wt **2.45→1.99 bits/symbol**.
For 40-MB Webster, bwz remains **1.48**. Encoding time/space roughly doubles;
decoding decreases slightly. These are compressed sizes on large inputs, not
resident inversion-table bytes or coverage of 900k-row blocks. The wt variant
was not yet a fully functional text index.

**Baier–Dede, DCC 2019**, *BWT Tunnel Planning is Hard But Manageable*:
[authors' slides](https://www.uni-ulm.de/fileadmin/website_uni_ulm/iui.inst.190/Intern/medien/slides_dcc_19.pdf),
slides 7–12, distinguish block cover/coverage complexity and offer a simpler
benefit-versus-marker-cost heuristic. In particular, maximum overlapping coverage
does not solve net compressed size. I inspected the authors' presentation, not the
full proceedings paper; no numeric decoder-cost claim is imported from it.

**Alanko–Gagie–Navarro–Seelbach Benkner, DCC 2019**,
[Tunneling on Wheeler Graphs](https://arxiv.org/pdf/1811.02457), pp. 5–8:
disjoint blocks preserve Wheeler structure. Traversal carries a tunneled node and
offset. Entry determines the offset; internal moves retain it; exit selects the
proper original continuation, using rank/select and marker structures. Their
general-graph representation is `n_t log sigma + o(n_t log sigma) + O(n_t)` bits
with pattern existence in `O(|P| log log_w sigma)` time. String-specific entry/exit
edge bitvectors simplify; samples allow tunnel skipping in `O(log n_t)` steps.
That skip supports locating; extraction still has to produce the skipped bytes.
This is theory, not a measured x86 instruction budget or corpus coverage percentage.

**Baier–Büchler–Ohlebusch–Weber, DCC 2020 / later journal version**,
[Edge minimization in de Bruijn graphs](https://arxiv.org/pdf/1911.00044), inspected
v3, experimental section and Figure 5: restricted disjoint-prefix tunnelling is
connected to minimizing edge count over de Bruijn orders. Reported reductions:
**5–45%** edges on non-repetitive files, **60–95%** on repetitive files; index-size
reduction about **80%** on very repetitive inputs. The tunneled DNA index can be
larger: two additional bitvectors per remaining symbol outweigh weak reduction.
English's optimal graph order is 28. That order is a context parameter, not a
recommendation to make every inversion step 28 bytes. The FM-index size here is
a wavelet-tree BWT representation, not a 900k-row integer table.

**Goga–Baláž, WABI 2022**,
[Prefix-Free Parsing for Building Large Tunnelled Wheeler Graphs](https://doi.org/10.4230/LIPIcs.WABI.2022.18),
Tables 1–2: PFP trades a larger graph for cheaper construction. For 100 chromosome-19
copies (5,322 MB), reported graph sizes are **162 MB** for the earlier Baier approach
and **2,123 MB** for PFP; construction time **7,263 vs 644 seconds**, memory **6,748 vs
1,319 MB**. On 5,000 Salmonella sequences (12,053 MB), sizes are **1,275 vs 4,189 MB**.
These demonstrate the selection/build tradeoff on genomic collections, not text-like
block performance or per-byte LF cost. No transfer of those percentages to Wikidata.

**Inference for mibz2:** none of these results supplies 94.5% removable coverage,
1.2 MB resident state, or an Ir/output-byte guarantee for this decoder. They establish
prior art for parallel-path compression and expose the metadata/planning cost which
a plain coverage number omits.

## 4. Audit of the census and the paper price

| Claim at f666ea9f3 | Independent disposition |
|---|---|
| 94.5% Wikidata / 81.0% text rows in tunnels | These equal `1-singleton_cells/n`: 1−.0548 and 1−.1899, rounded. They are membership in multi-row one-step cells, not selected removable coverage or long-word coverage. This statistic is independent of the bad +1. |
| 20.5 / 3.6 summed area | Overlapping candidate score, not achievable deletion. Additionally raw word length is wrong. Literal CPM-2018 deletion of an isolated height-H, width-W block keeps both endpoint columns, yielding (H−1)max(W−2,0) removed positions, derived directly from the retained boundary. Do not equate a rectangular score with either that deletion or graph edge savings. |
| .148 steps/row, .096 short steps/row, word-pool sizes | Computed using invalid ell+1 jumps. Recount the actual valid traversal; changing lengths changes later landing cells, so subtracting one from the mean is not a correction. |
| 48.1% bytes in length>=33 words; .0086 entries/row | The cited 2026-11-09 entry does not expose these exact totals. If from `split_*`, the same +1 defect applies. If from neighbour-copy phrases, those are a different policy, not certified whole-cell words. Require exact field, rows and corrected certificate. |
| 0.74–1.26 MB | Paper allocation estimate, not measured capacity. The original design-C directory's start64+rank32+flags64 is **20**, not 16, bytes/64 rows. A separate pre-marker bitmap adds .125n if not already charged. Prefix flags alone do not provide the shortened word's landing. |
| 51 Ir/step; 16–22 Ir/row | A reused-loop model. The listed maxima sum to 52–54 (12–14 +6 +4–6 +4 +22), while 51 is plausible only with favourable components. At the stated rates, 51a+30W gives 16.0/17.6/22.2 before emitter, directory, marker construction and other omissions. No exact tie follows. |
| whole-cell advantage disproved | Unsupported. Neither faulty traversal counts nor this particular construction establish a lower bound over whole-cell representations or memoized builds. C3 permits deprioritizing without a universal impossibility claim. |

Extra packing obligation: explicit start20 + target20 + word-offset19 + length6 is
**65 bits** at cap64 and the stated .488n word pool, before flags. An 8-byte record
needs a changed encoding, such as modular delta in place of both endpoints, or a
separate field. Such a repair may help; it must be counted. The hybrid likewise
must charge word loads, lane state and marker/tail handling, not just rank lookups.
Wide unaligned overcopies need input padding and output ownership/slack, especially
at markers; nominal word bytes do not cover those obligations.

I inspected source and recomputed arithmetic; I did **not** rerun the corpora.
The inspected tree has no matching archived TUNNEL per-block JSONL. The source
samples every sixth **eligible** block (n>=65,536), not every sixth physical block;
the 233-block and 39-block summaries are different populations. Also its
`landings.len()==cells` assumes the origin visits all rows: periodic cyclic BWTs
need explicit handling before claiming a universal census (e.g. all-equal text has
identity Psi). Report these scope limits with corrected per-block data and hashes.

## 5. What transfers, and the cheapest falsifiable prediction

Keep ordinary bzip2 input and the row permutation. Build only certified common-word
plus affine-endpoint jumps, or an explicit dictionary for selected longer paths.
Unlike compression-time tunnelling, decoder-time selection/build is paid afresh
for every block. Keep n-row / pre-RLE1 bytes separate from final decompressed bytes.
Preserve primary index, exact n emissions, cyclic/periodic inputs, segment ownership,
partial-word LF^j/Psi^j endpoints and bounded tail stores. No format change or
full text-index locating machinery is needed for sequential inversion.

**Hybrid accounting, derived.** For stride k, let admitted non-overlapping visits
copy q_j complete k-byte passes after one entry lookup. Let `c=k sum(q_j)/n` be
their actual full-pass byte fraction and `lambda=number_of_visits/n`. Ignoring
separately charged marker/tail lookups,

    rank/table lookup events per row A = (1-c)/k + lambda.

That counts events, not all memory loads or misses. Word-copy loads remain.
For k=8 and lambda=.0086, A<=.08 requires **c>=.4288**. Pathfinder's .481 is not
this c. Even if those words were valid and their starts aligned, rounding each
length down to a multiple of eight can lose up to 7lambda=.0602 of coverage:
c>=.4208 would certify only A<=**.0810**, before additional start misalignment,
markers and selection effects. The .0735 ideal calculation is a hypothesis,
not the claimed upper bound. The proposed 'under a third' entry-share test is
not interchangeable with saved full-pass coverage.

**Prediction worth testing, not a gate amendment:** on the same admitted Wikidata
blocks, a corrected stride-8 census can achieve A<=.08 with distinct stored long
words <=.20 B/row. On text I predict that conjunction fails under the same policy;
81% multi-row membership alone cannot rescue it. These are explicitly unmeasured
targets; no literature result supplies their probability. Minimum information:
report c, lambda, q_j histogram, actual landing rows, bytes, distinct-word bytes,
markers lost and total final/peak capacities, separately per block/corpus.

**Ir prediction must be conditional:** use I_walk=A*J + c*C + E, with J the inclusive
ordinary/entry lookup price, C extra copying cost per copied byte, E switches,
tails and setup per row. For an illustrative J=60 Ir, C=.5 and E<=2, A<=.08 gives
I_walk<=**7.3 Ir/row** (using c<=1). At the pessimistic J=100 it gives <=**10.5**.
Both are paper hypotheses, not counts; an inclusive count above its stated bound
falsifies that hypothesis. Construction remains additional, and unknown. Preserve
the existing D4 probe's separate 15-Ir walk, 7.35-Ir build and 1.2-MB filters;
no result here relaxes them or authorizes timing.

**Next value:** repair and certify the census first, before a decoder prototype.
Validate each proposed word and landing for every row of small adversarial cells;
then archive corrected corpus statistics on identical samples. Price the selected
hybrid against D4 with total live memory and dependent loads exposed. No production
gap closed today; the advance is a concrete false certificate removed and an exact
coverage-to-lookup equation replacing the unsupported 48%-to-.074 transfer.

## Protocol

Legacy head and migration-manifest digest matched before receipt. Pinned routing
audit returned no findings. Work confined to `team-state/researcher`; one REVIEW on
#145 follows completion. Automation and webhook configuration unchanged.
