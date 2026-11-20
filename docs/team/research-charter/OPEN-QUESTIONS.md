# Open questions, ranked

*20 November 2026. The Coordinator.*

Each question is ranked by three things:
1. its potential end-to-end impact, bounded by measured shares;
2. whether its information can plausibly be acquired;
3. the cheapest experiment that could kill it.

None of these is a prediction. A fresh researcher may find a better question; these are where the
record points.

**These are the feasibility track's leads** ([METHOD.md](METHOD.md) §3): concrete enough to cost
now. The foundational questions, which challenge whether a current primitive is needed at all, are
in [FRONTIER.md](FRONTIER.md). Each cycle should draw on both.

## 1. A second open cut: recursive first returns

**The observation:**
- Open walks m ≈ 0.11n retained rows, one dependent edge load each. Emission is 36–48% of Open's
  time.
- Repeated open cuts on block 0 shrink the retained set 899,931 → 110,273 → 17,657 → 3,916.
  The return bands drop from 16,230 to 8,159 to 3,644 (astra-investigation-3).
- Open's first-return map g on the retained rows is itself a piecewise translation over its bands.
  That is information Open **already holds** after wiring.

**The question:** can g be cut again, using only its band structure, which is O(B) and not O(m)?
Then:
- the walk would chase m₂ ≪ m rows;
- the second level's words would be assembled from first-level words, not from bytes.

**What could kill it:**
- the second level's construction and word assembly might cost more than the m − m₂ dependent
  loads it saves;
- longer words might hurt the copy path;
- the forest implementation of repeated cuts was 15× slower than Cells. It derived the forest
  from rows, not from bands.

**The cheapest experiment, in Python on the six blocks:**
- from Open's bands, build the second-level exchange and count its pieces, its steps and the
  retained m₂;
- count the extra word-assembly bytes;
- then compare (m − m₂) saved dependent loads against the added construction events, in the same
  units as the phase split.

**Impact ceiling:** most of emission's dependent-load share. It cannot touch the n output byte
writes.

## 2. Overlap transitions in o(n): the in-place copy schedule's missing edge

**What is established:**
- Record 82 built valid in-place Φ-copy schedules of 0.87–2.15r fragments of about 7 B. That is
  fewer operations than Cells' steps from r/n ≈ 0.088 upward, and 0.52× Cells on block 210.
- Building the schedule needs each copy's sequence of run overlaps.
- Record 84: the full text-order run labels determine the text, so obtaining them **is** emission.

**The question:** can just the *transitions* between consecutive overlaps, about O(#overlaps), be
listed from the run-boundary positions? Those positions are obtainable with integer-only RV in
Cells' step count (record 81).

**What could kill it:**
- an information-theoretic obstruction: the transitions might determine the text, as the labels
  do;
- the overlap count might be n-scale.

**The cheapest experiment:** count #overlaps against n on the six blocks. Then try to prove or
refute "the transitions determine the text".

**Impact ceiling:** high-r/n admitted blocks only, where Open's gain is also smallest.

## 3. Memory traffic, not operations

**The observation:**
- Astra D modelled an exact reconstruction table of 260 KB that needs 3.44 DRAM bytes per output
  byte on block 0. Shipped Cells needs 7.33.
- Production runs 8–20 threads on an i9, where DRAM bandwidth, not dependent latency, may bind.

**The question:** is there a reconstruction whose working set stays L2-resident per worker, and
whose acquisition is affordable? Astra D's table was not built at an accounted cost.

**What could kill it:** building the table might cost a full inversion.

**The cheapest experiment:** first, on the workstation, find whether Open at 10–20 threads is
bandwidth-bound: compare per-thread throughput at 1, 4, 10 and 20 threads. If scaling is near
linear, this question is dead for now.

## 4. Output-sourced decoding: the self-sourcing cursor

**The observation (Astra C):** the marked output can supply its own byte sources. A cursor
emits a phrase, learns which other rows spell those bytes, and copies from completed output at
later visits. It saves 37% of modelled line transfers, but demand reads rise 25–87%.

**The question:** is there a variant whose discovery of other rows uses information Open already
has, such as its bands, rather than fresh LF probes?

**What could kill it:** the demand-read rise is structural.

## 5. Parallelism within a block

**The observation:**
- Theorem 3′ batches 4–5 fronts on real blocks.
- Open's walk is one chain, while production parallelises across blocks.
- At 20 threads, the across-block parallelism may already saturate the machine.

**Ask this only if question 3 finds the machine under-occupied.** Otherwise it buys nothing.

## Not on the list

These are closed by measurement (see CONTEXT.md §4):
- micro-rearrangements of Open's loops;
- LIMIT tuning;
- induction scheduling;
- direct Φ-copy peeling;
- jump tables built per capped record.
