# Literature study: compact ψ, move structures and their construction, one paper at a time

A reading programme run with the research chatbot. It works through the papers behind the
compact-ψ research (`2026-10-29-succinct-review`, PR #94) one at a time, each read in
depth, with its predecessors, its implementation and secondary sources. The user relays
each reading into the working session, and it is kept verbatim in
[`2026-10-29-literature-study/`](2026-10-29-literature-study/). This entry is the index,
and it checks each reading against what this journal has already measured.

Labels: **source** (the researcher's reading, which we have not yet checked against the
paper unless said), **measured** (ours, said where), **hypothesis** (untested).

## The readings

| # | file | paper | status |
|---|---|---|---|
| 0 | [`00-overview-research.md`](2026-10-29-literature-study/00-overview-research.md) | where the programme stands, and the reading list | read |
| 1 | [`01-brown-langmead-2026-length-capping-research.md`](2026-10-29-literature-study/01-brown-langmead-2026-length-capping-research.md) | Brown & Langmead, "Bounding the Average Move Structure Query for Faster and Smaller RLBWT Permutations", SEA 2026 (doi:10.4230/LIPIcs.SEA.2026.9; arXiv:2602.11029); the Orbit library | read |
| 2 | [`02-optimal-time-construction-research.md`](2026-10-29-literature-study/02-optimal-time-construction-research.md) | Brown, Sanaullah, Zhang & Langmead, "Optimal-Time Move Structure Construction", arXiv v4, 22 October 2026. With alt text for its Figures 2.1, 2.2, 3.1 and 3.2, and the researcher's "why this paper" | read |
| — | [`03-research-trail-research.md`](2026-10-29-literature-study/03-research-trail-research.md) | the thread joining papers 2 to 5: building a compressed structure without first doing the work it exists to avoid | read |
| 3 | [`04-rlbwt-tricks-research.md`](2026-10-29-literature-study/04-rlbwt-tricks-research.md) | Brown, Gagie & Rossi, "RLBWT Tricks", SEA 2022 | read |
| 4 | [`05-partitioned-elias-fano-research.md`](2026-10-29-literature-study/05-partitioned-elias-fano-research.md) | Ottaviano & Venturini, "Partitioned Elias–Fano Indexes", SIGIR 2014 | read |
| 5 | [`06-gog-navarro-petri-research.md`](2026-10-29-literature-study/06-gog-navarro-petri-research.md) | Gog, Navarro & Petri, "Improved and Extended Locating Functionality on Compressed Suffix Arrays", Journal of Discrete Algorithms, 2015 | read |
| 6 | [`07-sadakane-csa-research.md`](2026-10-29-literature-study/07-sadakane-csa-research.md) | Sadakane, "New Text Indexing Functionalities of the Compressed Suffix Arrays", Journal of Algorithms, 2003; SDSL's `csa_sada`. The researcher read the abstract, secondary accounts and SDSL, not the full article | read |
| 7 | [`08-patched-coding-plateaus-research.md`](2026-10-29-literature-study/08-patched-coding-plateaus-research.md) | patched integer coding (Lemire & Boytsov, "Decoding Billions of Integers per Second Through Vectorization", SPE 2015; Brisaboa, Ladra & Navarro, "DACs", 2013) and the residual-plateau record | read |
| 8 | — | cache-line Elias–Fano and CPU-aware decoding, the last item on the original list | next |
| — | — | the overview's other items (cacheline Elias–Fano, the LCSA line, the Orbit implementation in its own pass) | queued |

The user calls paper 2 "the first paper", as the first they worked through with the
researcher; the researcher's numbering is kept here.

## What the readings change, checked against this journal

**Length capping (paper 1) was already built and measured here.** `2026-10-27-move-structure-walk.md`
(#42, measured locally) built a length-capped move structure for ψ:
- walked as one chain from `orig_ptr`, it cost −59% to −63% against the plain one-chain walk;
- walked as 32 room-style lanes, build plus walk cost +90% to +108% against
  `bwt::build` + `bwt::walk_segments_room`. That is negative on #42's criterion (−10% or
  better).

This is the distinction the researcher draws between bounding aggregate work and hiding
latency. The paper bounds the work of a full traversal. mibz2's walk is already
latency-hidden across 32 lanes, so cheaper dependent steps buy little and dearer
interleaved steps cost a lot. Nothing in papers 1 or 2 reopens #42. Its reopening
condition stands as written in `2026-10-27-walk-sweep.md`.

**Construction (paper 2) is relevant to building compact ψ, not to the walk.** Its idea
is to keep the forward and inverse interval structures together, with cross-links, and
sweep both in order, so that each predecessor search becomes a local repair. That matches
what #81 found by measurement, that construction cost moves a lot when redundant work goes.
- **Measured** (PR #95, journal `2026-10-28-cooperative-inversion.md` §16.4, on the i9):
  the fused cooperative build x3x3 is 1.643× p4 for inversion. Checkpoints were free, and
  MTF was untimed.
- **A correction** to the figure quoted in the readings: the 31.3% gain is against q, the
  given-counts build, which carries its own implementation overhead (x4x1 ≈ p4). The
  figure against production is x's.
- **Hypothesis:** compact ψ built directly from L, in one ordered pass with temporary
  per-byte state, needs no reorganisation pass afterwards.

**RLBWT Tricks (paper 3) adds three things to test, and one warning.**
- **Carrying an interval as lane state** ((run, offset) in place of an absolute row) removes
  a column from the table. mibz2's lanes carry absolute rows, which keeps addressing and
  prefetch cheap.
  - The narrow question for us: is any metadata recovered at every step that could be set
    once a segment, when a lane starts?
  - **Hypothesis**, untested.
- **Select on a small bitvector** beat interpolation and directly addressable codes in
  their tests. The hybrid-record prototype (the e and h arms, 64-byte RUN/FOR/EF records)
  should time a select path alongside FOR, as a whole local decode path in one cache line.
  Their result doesn't show which wins on the i9.
- **The cost distribution of exceptions, not only their frequency.** The census gives
  coverage (77.4% of 32-row groups translation-eligible, census §13, PR #94; the
  researcher's figure matches). It does not yet give what each non-translation group would
  cost to decode. The coordinator's structural census step (r, d, K32, the run-derived
  check, A2) is where to add it.
- **The warning:** their datasets have 222–1,329 rows a run. Wikidata's blocks have n/r =
  13.85, the median block r/n = 0.0623 (`2026-10-27-mlp-scheduling-and-run-structures.md`,
  measured). That is below even paper 1's smallest dataset (29.2). Run-based results from
  genomic collections can't be carried over to bzip2 blocks without measuring them.

**Partitioned Elias–Fano (paper 4) is prior art for translation groups, and gives a
cheap census question.**
- **Prior art:** a chunk that fills its local universe stores no values, only its
  boundaries. That is a translation group, published in 2014. The census (77.4% of
  32-row groups, §13, PR #94) makes it the common case on Wikidata's blocks, not an edge
  case.
- **Fixed against adaptive boundaries:** their uniform chunks are about 12% larger than
  their near-optimal ones (Table 2, as the researcher reads it; Gov2 5.17 against
  4.65 GB). Their index is built once and queried many times. mibz2 builds ψ in every
  decode and may walk it once, and fixed groups keep the group address arithmetic
  (g = j >> 5), with no predecessor search on the chain.
  - **Hypothesis**, checkable offline without touching the decoder: computing the
    near-optimal variable partition of the Wikidata blocks' ψ lists measures what fixed
    addressing costs in bytes. A census figure, not a decoder change.
- **The residual R(j) = ψ(j) − j.** Inside one F bucket ψ is strictly increasing, so R is
  nondecreasing, and it is constant exactly where ψ is a translation.
  - A group's plateaus are one more than the breaks inside it: ψ's discontinuities (d)
    and bucket boundaries. The census already counts the breaks (d) and the groups that
    hold one (K32), so a histogram of breaks per group is a small addition to the
    structural census step. It answers the reading's new hypothesis: whether most
    non-translation groups are two or three translations, which a record could hold as a
    few (slot, displacement) pairs in place of FOR or EF offsets.
  - **Hypothesis**, untested. Such a record has to find the slot's plateau on the chain's
    critical path, so the census decides only whether it is worth a prototype arm
    (alongside e and h), not whether it wins.

**Gog, Navarro & Petri (paper 5) is the closest precedent: compressed ψ, by name.**
- **Prior art, again:** their uniform blocks are our translation groups. They apply them to
  Elias-δ, OptPFor and partitioned Elias–Fano, over blocks of 128 ψ values. Any novelty
  here is in the construction, the cache-line layout and the cooperative walk, not in
  spotting consecutive ψ values.
- **Speed is not size:** in their isolated random-access tests, partitioned Elias–Fano is
  the fastest of the three and about 5–10% larger than the smallest (Figure 3, as the
  researcher reads it). Their texts are 2.1–4.6 GB under single random queries; ours is a
  block of at most 900,000 rows, walked by 32 lanes. The ranking carries over as a reason
  to time select fairly, not as a figure.
- **Bucket boundaries are bounded for bzip2 (ours, arithmetic).** A block has at most 256
  byte buckets, so at most 255 boundaries, so at most 255 of a full block's 28,125 aligned
  32-row groups straddle one: 0.91% at most. Their word alphabet (about 3.9 M symbols)
  has no such bound. A plain fallback record for straddling groups costs at most that
  fraction.
- **The whole lookup, not ψ alone:** their faster ψ gained less in full suffix-array
  operations, because the operations also read sampling data.
  - Today's T-vector entry is one u32 a row, ψ in its top 24 bits and the output byte in
    its low 8 (`src/bwt.rs`), so one load gives both.
  - A compact record has to give ψ and the byte from the same line, and leave the walk's
    segment handoffs as cheap as they are. The hybrid prototype's e arm (embedded-byte
    control) exists to measure exactly this.
- **The taxonomy of non-translation groups** gives the census step its three classes:
  - jumps inside one bucket (a few plateaus of R);
  - increasing but spread (many breaks);
  - straddling a bucket boundary.

  A breaks-per-group histogram, split by whether the group straddles a boundary,
  measures all three. That is the residual-plateau question from paper 4, and the
  researcher now ranks it first.

**Sadakane (paper 6) is the origin of compressed ψ, and sharpens the output-byte question.**
- **The lineage:** ψ stored as per-byte increasing lists, as sampled differences (γ or δ
  codes, a sample every 128 or so). A lookup is a reconstruction, not a load: fine for an
  index queried many times, a serial cost on every step for a decoder. Translation groups
  invert the trade: the common case is one base and no differences at all.
- **The output byte comes free on almost every group (ours, from the code and the bound
  above).**
  - In `src/bwt.rs`, row k's entry holds ψ(k) in its top 24 bits and L[k] in its low 8.
    The walk loads the destination's entry and emits its low byte, L[ψ(r)], which is F[r].
    So the byte emitted at row r depends only on r's bucket.
  - A group inside one bucket has a single F byte, which its descriptor can carry once.
    Only the straddling groups (at most 0.91% of a full block's) need anything per row.
  - So for a 24-bit link-only table (#79) the researcher's warning, a symbol costing extra
    memory accesses, applies to at most those groups. Whether the walk's segment markers
    can be carried the same way is still to check against the room walk's handoffs.
- **Construction is per block, not amortised.** An encoding that saves bits at the price
  of an extra pass over ψ is a loss here (#81's fused build is the baseline to beat). Our
  input is already the BWT, so the build is a scan of L, not suffix sorting.

**Patched coding (paper 7) turns the plateau question into candidate records.**
- **From the literature:** keep the common case fixed-width and cheap, and handle the
  exceptions apart (PFOR). Encoding cost counts: OptPFD's encoding is 20–50× slower than
  SIMD-FastPFOR in Lemire & Boytsov (the researcher's reading). Bulk-decode throughput
  doesn't carry over to a dependent chain, which needs one value at a time (DACs' problem,
  and ours).
- **The plateau record (the researcher's design, a hypothesis):** a 32-bit break mask M
  and the plateau values V, so ψ(a + i) = a + i + V[popcount(M & (2^(i+1) − 1))]. It is
  exact for any group within one bucket, and small when the plateaus are few:
  - 32 + 20k bits for k plateaus (20 bits suffice for a 900,000-row block);
  - the two-plateau case is 45 bits (two 20-bit bases and a 5-bit split), decoded with
    one compare and an add.

  Against 448 bits for 32 FOR-14 offsets, that is a large saving only where the record
  size can shrink (the two-tier design's variable payloads). A fixed 64-byte line gains
  speed from it, not space.
- **A correction to §10's table:** log2 C(31, k − 1), the bits needed to place k − 1
  breaks, is 12.13 for k = 4, 21.33 for k = 8 and 28.16 for k = 16. The reading gives
  12.31, 21.47 and 28.54. The 2- and 3-plateau entries (4.95, 8.86) are right, and nothing
  in its argument depends on the three that are off.
- **The census it asks for** is the histogram set out under paper 4: plateaus per group,
  1, 2, 3–4, 5–8 and 9 or more, with straddling groups apart. A group's plateaus are its
  breaks plus one. This is the fourth reading in a row to end on it.

## Open checks

- **Citations:** paper 1's are checked (`2026-10-29-expert-review.md` §1). We have not yet
  checked paper 2's arXiv v4 revision date, RLBWT Tricks' venue and the readings' figures
  against the papers. The research trail dates RLBWT Tricks 2024, and deep dive 2 gives
  SEA 2022. Paper 4's Table 2, paper 5's Figure 3 and paper 7's encoding-speed ratios are the researcher's reading.
- **Cyclic BWT:** bzip2 sorts rotations with no sentinel. A periodic block gives ψ several
  cycles (for `abab`, LF = [2, 3, 0, 1], two cycles of 2), and the decoder walks one of
  them n times from `orig_ptr`. The papers' move structures assume a sentinel and a single
  cycle. This doesn't affect the representation, but any construction argument that
  relies on one cycle has to be re-checked for bzip2.
- **Lost mathematics:** the chat copy dropped most inline symbols, so some sentences in the
  verbatim files have gaps. Where a reading's claim depends on a bound, check it against
  the paper, not the copy.
- **Convention:** paper 5 defines ψ on a suffix array with an end marker. bzip2 has
  rotations and `orig_ptr`. Monotonicity within a bucket holds in both, but traversal and
  termination differ (the cyclic-BWT check above).
