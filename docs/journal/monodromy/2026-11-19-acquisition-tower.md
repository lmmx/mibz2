# 2026-11-19 · MONODROMY 84 · Acquisition-first, first deliverable. The refinement tower from I₀ does not explode: K₁ = 1.21–1.25 r, K₂ = 1.34–1.42 r and K₃ = 1.46–1.58 r on all six blocks, by an exact formula. Its constructive end is an arc-synchronous LF walk, exact on all six blocks, which passes the availability gate but is a lane walk with free boundaries, not a new cost class. The registry's open edge `copy_run_labels` is closed by a reduction: the labels determine the text, so no copy replay that needs them can beat inversion

**The request.** The Coordinator's CHECKPOINT 7021146886: acquisition-first synthesis (#645,
`docs/team/ACQUISITION-FIRST.md`). Three parts, in order:
1. a falsifier;
2. a backward analysis;
3. one candidate or obstruction, with a manifest the checker judges.

**Labels:** **proved** · **verified** (exhaustive on a bounded domain, plus the six blocks) ·
**measured** (counted on the six authenticated blocks; no timings here) · **assumed**.

**Files** (in `2026-11-19-acquisition-tower/`):
- `acq1.py.txt`: the tower, the arc walk and the checks;
- `arc-walk.toml` and `copy-replay-via-labels.toml`: the manifests;
- `registry-edges.toml`: new and changed registry edges.

## 1. The falsifier: does affine refinement explode?

**The tower.**
- *The cut sets:* Cut₀ = D̄, the run heads. Then Cut_{k+1} = Cut_k ∪ LF(Cut_k): one more image of
  every cut, merged in.
- *Its first level is `partition_overlaps`:* Cut₁ = D̄ ∪ C̄, the run partition merged with its
  LF-image partition, and K_k = |Cut_k| is the event count of the k-th merge.
- *Each level from I₀ alone:* LF on a row-sorted set is affine per run. Bucketing the images by
  symbol and concatenating them in symbol order leaves them sorted, since within one symbol the
  images rise with the row. So level k costs one two-pointer merge of O(K_k) with at most σ
  output streams: no sort, no search, no n-size table.

**Proved: K_k = r + Σ_{c∈C̄} min(k, ℓ(c)),** where ℓ(c) is the length of the arc
A(c) = (c, LF c, …, d), from the codomain start c to its first run head d (record 80's
persistence arcs).
- *Proof:* LF^j(D̄) = C̄ moved j − 1 steps along each arc, and an arc's points before its end are
  not heads. So Cut_k adds, beyond D̄, exactly the first min(k, ℓ(c)) points of each arc. □
- *Two consequences:*
  - K₁ = 2r − |D̄ ∩ C̄|, record 80's floor.
  - Each level adds #{c : ℓ(c) ≥ k}. That is at most the k = r − |D̄ ∩ C̄| nontrivial arcs, and it
    falls with k.
- *Verified:* against direct counting by the merges, on 23,093 small single-cycle cases (every
  column over 2 letters to 12, 3 to 8, 4 to 6, at every start row), and on the six blocks below.

**Measured on the six authenticated blocks** (record 80's export):

| block | r/n | K₁ | K₂ | K₃ | nontrivial arcs | max ℓ + 1 |
|--:|--:|--:|--:|--:|--:|--:|
| 0 | 0.057 | 62,138 = **1.214 r** | 69,598 = 1.360 r | 76,243 = 1.490 r | 10,954 | 4,714 |
| 2 | 0.061 | 67,755 = 1.242 r | 77,052 = 1.413 r | 85,244 = 1.563 r | 13,223 | 10,435 |
| 85 | 0.088 | 96,494 = 1.216 r | 107,672 = 1.356 r | 117,500 = 1.480 r | 17,119 | 848 |
| 207 | 0.097 | 107,914 = 1.239 r | 122,058 = 1.401 r | 134,142 = 1.540 r | 20,803 | 4,305 |
| 143 | 0.120 | 135,299 = 1.249 r | 154,302 = 1.424 r | 170,878 = 1.577 r | 26,964 | 17,545 |
| 210 | 0.143 | 155,480 = 1.208 r | 172,785 = 1.342 r | 187,637 = 1.458 r | 26,748 | 6,845 |

**The answer: the affine endpoint geometry stays small for any fixed number of refinements.** Each
level adds 0.12–0.17 r, and less at each level. But it never closes. The tower reaches the text
order only at k = max ℓ + 1 (848–17,545 levels), by which point it holds all n rows. Refinement
alone is an LF walk run arc by arc in parallel, no shortcut.

## 2. Backward: what is sufficient? (proved or literature; requirements, not a representation)

**What any sequential chunked emission needs.** An emission writes the text as chunks, each a
literal or a copy, of mean length L. Three requirements:
1. **Text-order destinations.** For each of the ~n/L chunks, its place in the text, or its
   successor chunk.
   - *What I₀ holds:* row-ordered objects only: partitions, affine maps, counts.
   - *What §1 adds:* row-sorted refinement produces text-order information only along arcs: the
     successor head of c is known only after ℓ(c) + 1 levels.
   - *The measured edges to text-order anchors:* (a) Rauzy–Veech induction, S = 1.68–1.87 r
     dependent steps (records 80 and 81); (b) walking the arcs (§3).
2. **Materialised sources.** For a copy chunk emitted left to right, a source already written, or
   held in a buffer whose filling is charged.
   - *Chunk count:* at least z, the self-referential LZ77 phrase count, which the greedy parse
     attains for left-to-right copying (literature).
   - *With Φ-sources only:* a copy is emittable at its turn only if its source precedes it in the
     text. Round 6 showed the Φ scheme is bidirectional, so this fails for most copies.
3. **Dependency depth D.** For one sequential writer, D is irrelevant to correctness, and the
   cost is the chunk count. For parallel emission, D is the longest chain of copies through
   their sources: for the Φ scheme, at least λ_max − 1 (record 82).

**The reduction that settles the copy-replay line (proved).** Let `run_label_sequences` give,
for each text position j, the run of its row.
- *A run fixes its byte,* so text[j] = symbol(label[j]): one sequential map, W = n, D = 1.
- *So acquiring the labels emits the text.* Any decoder that needs them, as round 6's potential
  schedule does, has already paid for inversion before its replay begins.
- *The registry's open edge `copy_run_labels`* is therefore no longer open: its cost is at least
  that of the target.

**What escapes the reduction.** The schedule needs only its fragment boundaries (about
0.12 n of them, record 82) and their times, not every label. An o(n) route to those boundaries
would have to identify, without the per-byte transitions, which run-to-run transitions along
each arc the potentials satisfy. **Open.**

## 3. Where the two meet: the arc-synchronous walk

**The construction.**
- *What it uses:* the free objects of §1 only.
- *The arcs:* every arc starts at a codomain start c ∈ C̄ and ends at its first run head d ∈ D̄.
  Both ends are known from I₀; no start bitmap is needed.
- *Each round:* every live arc advances one row. The frontier stays row-sorted by the
  symbol-bucket concatenation of §1. Each row's run is found by a pointer that only advances.
- *The chain:* arc A(c), ending at d, is followed by the arc starting at LF(d). Joining them from
  the primary row's arc gives the text: r dependent hops.

**Verified.** The text is exact against the LF walk on all 23,093 small cases and on all six
blocks.

**Its manifest** (`arc-walk.toml`), checked with #645's `check_manifest.py` against its registry:

```
PASS: arc-synchronous LF walk from the free arc boundaries (record 84)
path to 'text': arc_synchronous_walk <- lf_image_partition <- lf_shifts <- counts_from_records <- maximal_runs
  dependent/n-scale: arc_synchronous_walk: D = lmax + r, Q = n (verified)
```

**Cost: (W, D, B, Q, M) = (n, max ℓ + 1 + r, n, n, n).**
- *W:* n row visits.
- *D:* the rounds plus the r-hop chain.
- *Q:* one run lookup and one scattered byte write per row.
- *M:* round-major bytes are stored, and the transposition scatters them; or the walk runs twice,
  once for the arc lengths and once to write in place.

**The paid-cost falsifier, measured:** the frontier profile. The likeliest hidden cost is the
latency tail: rounds too narrow to overlap their loads.

| block | rounds | mean frontier | rounds narrower than 64 | visits in rounds narrower than 64 | narrower than 256 | Cells' steps S |
|--:|--:|--:|--:|--:|--:|--:|
| 0 | 4,714 | 190.9 | 3,819 | 27,196 (3.0%) | 87,051 | 92,571 |
| 2 | 10,435 | 86.2 | 9,597 | 58,962 (6.6%) | 106,227 | 101,907 |
| 85 | 848 | 1,061.2 | 437 | 4,108 (0.5%) | 4,902 | 140,544 |
| 207 | 4,305 | 209.1 | 2,679 | 81,781 (9.1%) | 192,629 | 153,650 |
| 143 | 17,545 | 51.3 | 16,474 | 111,204 (12.4%) | 191,066 | 193,866 |
| 210 | 6,845 | 131.5 | 6,280 | 52,008 (5.8%) | 94,701 | 216,769 |

**Its verdict: a candidate by the gate, but not a new cost class.**
- *Depth:* 4–165× shorter than Cells' S.
- *Work:* n visits, against Cells' S dependent steps plus its sequential n-byte emission.
- *Balance:* the arcs are badly unbalanced. Most rounds are narrower than 64, and on block 143,
  12% of all visits fall in those latency-exposed rounds.
- *What it is:* shipped Segmented's skeleton, a multi-start walk in lanes. Segmented takes
  random starts, which are balanced, and an n-entry T-vector. The arc walk takes free starts and
  ends, which are unbalanced, and a run lookup per row.
- *Its one possible advantage:* no n-entry table and no start bitmap.
- *The deciding measurement:* the per-visit cost of a run lookup by an advancing pointer or a
  gallop, against a T-vector load. It decides whether that advantage pays for the imbalance. That
  is a cheap timing on block 0 (not run here). Until then, the walk stays on record as mathematics
  with a passing manifest.

## 4. The checker, and the registry

**`copy-replay-via-labels.toml`** replaces round 6's open edge by the walk plus the reduction. It
**passes** availability, yet its path emits the text (`labels_determine_text`) before the replay
does.

**So the checker has a gap:** it cannot see that a target-type object was produced on the way to
the target. **Proposed check:** fail, or at least report, any manifest whose path produces the
target object, or an object marked as equivalent to it, before its final edge. With that check
the reduction would be caught mechanically.

**New and changed edges** (`registry-edges.toml`, for the Coordinator to merge into
`primitives.toml`; #645 is not mine to edit):

| edge | status | what it records |
|---|---|---|
| `refinement_tower` | **proved** | K_k = r + Σ min(k, ℓ(c)); per level a σ-stream merge, measured on the six blocks |
| `arc_synchronous_walk` | **verified** | (n, max ℓ + 1 + r, n, n, n), exact on six blocks, with the frontier profile |
| `labels_determine_text` | **proved** | run labels to text, one sequential map |
| `copy_run_labels` | open → **closed by reduction** | o(n) for all labels is impossible without an o(n) inversion; o(n) for the fragment boundaries alone stays open |
| `partition_overlaps` | K measured | K = 1.208–1.249 r on the six blocks |

## What this decides

Nothing in production.
- *The falsifier:* the endpoint geometry is small and does not explode. That is a sound base
  for Neon round 7.
- *What the base does not give:* text order, short of max ℓ + 1 levels.
- *The copy-replay line* is closed for any form that needs per-byte labels.
- *The arc walk* passes the gate, but is Segmented with worse balance. One timing decides it.
