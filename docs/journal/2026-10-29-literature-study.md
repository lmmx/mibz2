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
| 4 | — | Ottaviano & Venturini, "Partitioned Elias–Fano Indexes", SIGIR 2014 | next |
| 5 | — | Gog, Navarro & Petri, "Improved and Extended Locating Functionality on Compressed Suffix Arrays", 2015 | queued |
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

## Open checks

- **Citations:** paper 1's are checked (`2026-10-29-expert-review.md` §1). We have not yet
  checked paper 2's arXiv v4 revision date, RLBWT Tricks' venue and the readings' figures
  against the papers. The research trail dates RLBWT Tricks 2024, and deep dive 2 gives
  SEA 2022.
- **Cyclic BWT:** bzip2 sorts rotations with no sentinel. A periodic block gives ψ several
  cycles (for `abab`, LF = [2, 3, 0, 1], two cycles of 2), and the decoder walks one of
  them n times from `orig_ptr`. The papers' move structures assume a sentinel and a single
  cycle. This doesn't affect the representation, but any construction argument that
  relies on one cycle has to be re-checked for bzip2.
- **Lost mathematics:** the chat copy dropped most inline symbols, so some sentences in the
  verbatim files have gaps. Where a reading's claim depends on a bound, check it against
  the paper, not the copy.
