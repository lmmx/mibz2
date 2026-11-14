# Reduced permutations versus weighted LF first returns

**Recommendation: revise.** Reject deletion plus positional lifting as a general
LF shortcut. Exact, labelled interval refinement has a sound compositional
foundation and can reuse previously constructed returns. It needs more state
than a reduced permutation and gray placements, and this experiment establishes
no general cheap-construction bound or decoder speedup. Restrict any future
claim to aligned intervals or amortised reuse of already-paid return maps.

This is the user's independent, bounded research request, based on master
`2635f04e9155dd31388b24fd3f7248f87b3ddcda`, under the
[pre-data plan](PLAN.md). It neither completes the Codex production assignment
nor executes the queued real-block probe. No production, benchmark, team-state
or workstation files changed.

## Correspondence and its boundary

For column L, use the journal convention
`LF(x) = C[L[x]] + rank_before(L[x], x)`. Each maximal equal-byte run is a
labelled interval. Top order is L order; bottom order is stable byte order;
run lengths supply the weights. LF translates each run between these orders.
The decoder's selected origin p emits `L[LF^(-(j+1))(p)]` for `0 <= j < n`.
This formula matters when an arbitrary column's origin cycle does not divide n.

Delete gray run labels in both orders to get τ. Our ĉ stores each gray label's
two endpoints as `(preceding_black_count, ordinal_in_gap)`, including exterior
gaps. This symmetric insertion convention is equivalent in information to
left/right gap positions; it is not an implementation of the thesis operators.
Assertions reinsert every gray label and recover both original orders.

**Source boundary:** HAL denied access to the requested thesis PDF, so its
§4.1.4–4.1.5 and Chapter 5 were **not directly verified**. The directly read
author sources are [Paper I, §3.4](https://arxiv.org/pdf/1705.01641v2) and
[Paper II, §§2 and 5](https://arxiv.org/html/1801.05973v2).
Paper I describes black-edge restriction, gray endpoint positions, proper
colorings excluding gray pivots, and boosted powers of L/R. Paper II requires
compatible labelling and a commuting reduction/lift construction. These are
claims about specified combinatorial operations. They do not supply weighted
LF return times, words or origin lifting. Its classification cycle labels must
not be identified with LF's ordinary permutation cycles.

The local starting points are the
[Rauzy–Veech journal](../monodromy/2026-11-10-rauzy-veech.md),
[MONODROMY orientation](../../team/MONODROMY-ORIENTATION.md), and
[#145/6793678261](https://github.com/lmmx/mibz2/pull/145#issuecomment-6793678261).
The Coordinator's
[6793788598 decision](https://github.com/lmmx/mibz2/pull/145#issuecomment-6793788598)
keeps the real-block first-return probe queued; no part of that probe ran here.

## Exact construction and obligations

Let A be the union of black run intervals. Independently, the row oracle walks
LF from each x in A until its first positive return. A finite permutation
guarantees this return. Its answer is `(destination, time, run itinerary)`;
projecting run identities to bytes gives the return word.

The interval constructor starts with A's intervals. It composes translations,
splitting wherever a current image crosses a base-piece boundary or an A
boundary, and stops each fragment at its first return. Each fragment stores
source endpoints, displacement and the full run itinerary. Inductively these
are constant on that fragment; splitting preserves coverage and disjointness.
Stopping at the first hit therefore gives the oracle map. At most |A| unit
fragments suffice. This is a proof of finite exactness, **not** an O(run-count)
construction bound.

τ and ĉ alone are unweighted. Attach **all** original lengths and byte labels
and they recover the full LF input, hence determine its returns, but computing
those returns remains necessary. A directly usable return map additionally
needs split coordinates, images and words/times. Origins need an offset and
phase: for a gray origin p, find its next black anchor `LF^d(p)`, expand inverse
returns there, discard d bytes, then emit n bytes. Cycles missing A have no
anchor; the experiment explicitly retains full LF for their fallback and for
origin search. It does not pretend that black-only state decodes every origin.

For B contained in A, exact first returns satisfy `(LF_A)_B = LF_B`: visits
to B form a subsequence of visits to A. Concatenating cached itineraries proves
the word/time identity too. All tested single-run deletions verify this without
accessing original gray run records during the update. They still read cached
fragments, copy gray labels and pay boundary searches. This is nested induction,
not a transferred theorem about de Mourgues's boosted L/R dynamics; recoloring
that adds intervals or changes LF is outside the claim.

## Minimal failures

Minimality is exhaustive by length, column order and color mask in the tested
binary/ternary alphabets (and thus up to ordered alphabet renaming through
these witness sizes). These refute the stated naive LF constructions, not the
author's theorems with their own hypotheses.

* `L = ba`, retain `b`: LF exchanges the two rows. Deletion predicts a
  one-step self-return with word `b`; the actual return takes two steps, `ba`.
* `L = baa`, retain `aa`: LF is `[2, 0, 1]`. A single retained interval becomes
  two return fragments:

  | source row | destination | time | forward return bytes |
  |---:|---:|---:|---|
  | 1 | 2 | 2 | ab |
  | 2 | 1 | 1 | a |

  Deleting `b` produces the identity on the two retained rows. Copying the
  left endpoint's displacement/time/path to the other offset also fails.
  This is already a valid cyclic BWT, of `aab`. Changing `ba` to `baa` leaves
  the unweighted orders/placements unchanged but changes the induced map.
* `L = ab`, retain `a`: origin 1 is in an entirely gray fixed-point cycle.
  An origin cannot always be lifted through black returns. `L = bab` also
  exercises a two-cycle in a three-row column; reversing n forward emissions
  is not generally the decoder's inverse-LF output.

Keeping the run permutation's pivots black does not remove the weighted
obligation: `L = cbaa`, retain `c` and `aa`, defer `b`. The first-domain and
last-image edge are both `c`, retained. Nevertheless row 2 returns to 0 in one
step (`a`), while row 3 returns to 2 in two (`ab`): the `aa` interval must split.
This column is already within the exhaustive ternary suite. This observation
still does not identify LF induction with a sequence of the author's operators.

## Evidence and counts

[experiment.py](experiment.py) uses only the Python standard library.
[results.json](results.json) contains the deterministic counters and full
minimal witnesses. Exhaustion covers raw columns and cyclic BWTs of all binary
strings through length 7 and ternary strings through length 5, every nonempty
run subset and every origin; it also checks empty retention and the fixed
periodic families. Duplicate columns from different words remain separate
test cases. The fixed synthetic cost families are additional to this table.

| Suite | models | colorings | origin checks | nested updates |
|---|---:|---:|---:|---:|
| Binary raw | 254 | 4,118 | 26,894 | 9,310 |
| Binary cyclic BWT | 254 | 3,086 | 20,078 | 6,332 |
| Ternary raw | 363 | 4,323 | 20,625 | 8,160 |
| Ternary cyclic BWT | 363 | 3,547 | 16,841 | 6,290 |
| Explicit periodic BWT | 20 | 68 | 828 | 60 |
| **Total** | **1,254** | **15,142** | **85,266** | **30,152** |

All assertions pass: **45,920** row-return triples, every origin's reconstructed
bytes, and every nested update match independent row LF. There are 556 models
with multiple cycles and 1,410 input origins on cycles whose lengths do not
divide n. Nonempty-color tests encounter 8,408 unanchored origins; empty
retention adds 6,574 origin checks. Naive deletion fails time/word on 12,556
colorings and destination on 5,902; positional lifting fails on 3,822.

Aggregate representation counts across colorings: 34,214 retained runs,
55,420 distinct retained endpoints (shared boundaries counted once),
38,314 final induced fragments, 68,490 stored itinerary symbols, and 30,152
gray edge records containing 60,304 endpoint records. These are logical record
counts, not concurrent memory or Python heap bytes; ĉ still stores the gray
part. Each fragment has three scalar fields plus its word (time is its length).
Lengths, bytes,
origin state and the retained full LF are additional storage.

Construction takes 64,082 base-piece visits versus 76,858 row-oracle LF
transitions, with 117,674 copied itinerary symbols, 153,829 binary-search
comparisons, 306,688 target endpoint tests and 134,335 membership tests.
Updates take 94,202 cached-piece visits versus 134,394 for direct rebuilding
of the same targets, and copy 192,966 symbols versus 277,654. Visits decrease
on 20,612 updates, tie on 9,438 and **increase on 102**: the cached partition
can require extra splits before final coalescing. This is not a uniformly
cheaper update method.

All-origin reconstruction additionally pays 69,624 forward LF visits to find
anchors, 302,454 inverse-fragment visits, 48,716 fallback inverse visits,
398,620 inverse-word symbol copies and 565,894 expanded symbols. The defined
**total map-transition visits** are 579,078: construction + updates + those
three reconstruction traversal categories. It excludes independent oracle and
assertion work, and does not conflate comparisons/copies with map transitions.
The JSON records those other operations separately. Sorting, allocations,
tuple-equality/coalescing, the per-construction base scan for the termination
bound, and test setup are not instruction-counted. Input
run discovery scans 6,574 rows; row oracles and BWT rotation sorting deliberately
materialise rows. No total-runtime or memory-performance inference is valid.

Two exact count families distinguish representation from construction:

| Column / retention, m=64 | boundaries | gray edges | final fragments | base visits | row-oracle visits | copied labels |
|---|---:|---:|---:|---:|---:|---:|
| `b^m a^m`, retain a | 2 | 1 | 1 | 2 | 128 | 3 |
| `b a^m`, retain b | 2 | 1 | 1 | 65 | 65 | 2,145 |

In the aligned family each whole run maps to a whole run: one itinerary serves
every offset, so two run transitions suffice **after runs are available**.
In the excursion family the same gray run is revisited m times. Construction
takes m+1 visits and flat-word concatenation copies `(m+1)(m+2)/2` labels,
despite one output fragment and two original runs. Its stored word has m+1
symbols. At m=64, searching all origins costs another 2,080 LF visits.
Compression could change the label cost; no such improvement is claimed or
implemented here. Outputting bytes still costs output work in both families.

## Reproduction and handoff

From this directory, with assertions enabled:

```sh
python3 experiment.py > reproduced.json
cmp results.json reproduced.json
sha256sum -c SHA256SUMS
```

Validated with Python 3.12.14 and Ruff formatting/lint. The initial passing
model/evidence is preserved at commit `51a1528`; subsequent changes add cycle,
update-comparison and copying counters, a termination assertion and formatting,
without changing the input bounds or verdict. The final output reproduced
byte-for-byte. Digests cover the script, plan, note, results and validation log.

The next useful decision is whether to retain this **restricted exact-cache
formulation** as research context. It does not justify another real-block task
or reopening decoder rollout. Direct thesis-page verification remains missing;
independent mathematical review has not been performed. No claim about Miri,
B5, production CI or whole-decoder performance is made. Existing assignments
and the queued probe remain with the Coordinator.
