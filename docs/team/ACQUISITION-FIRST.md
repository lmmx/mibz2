# Acquisition-first algorithm synthesis

*Adopted 19 November 2026, at Louis's direction: "Surely then we should be reverse engineering
the algorithmic ideas from what can be cheaply computed … Cuz we have kept finding Ideas only for
computations to prove too costly... This is working backwards!!!!"*

## Why

The kernel research keeps repeating one pattern:
1. find an elegant output representation;
2. prove it reconstructs the text;
3. show a large cut in some abstract operation count;
4. discover that **building** the representation costs the inversion it was meant to avoid.

Some examples:
- **Astra E's sparse exceptions and copy seeds.**
- **Neon's boundary-sampled copies** (round 5): its targets are exact, but RV-Copy's steps come
  to 1.68–1.87r.
- **Neon's run-overlap schedule** (round 6): a potential schedule of 0.87–2.15r fragments needs
  each copy's text-order run labels, which is an O(n) dependent walk.

In each case the mathematics was sound. What was wrong was the end the search started from.

## The rule

> **No object enters a proposed decoder unless it has a costed derivation from the decoder's
> entry state.**

The research starts from what is cheap and asks what that makes possible. It does not start from
what would be convenient and then ask what it costs.

## 1. The entry state, I₀

These are known when inverse-BWT starts, on the route in question:

| object | notes |
|---|---|
| `n`, `p` | block length, primary row |
| `records` | the encoded run records, R of them. Production caps a record at 127 rows, so **R ≥ r**, the maximal-run count. They are different quantities. |
| `C` | cumulative symbol counts, where the route has them (otherwise derive them: one scan) |
| `Σ` | the byte alphabet |

**Not in I₀**, and never to be assumed:
- text-order positions, row labels or LF orbit order;
- copy sources, reconstruction dependencies, SA samples.

## 2. The acquisition graph

Every useful object is a node. Every algorithm that derives an object from known objects is an
**edge**. An edge may have several inputs, so the graph is a hypergraph.

Each edge carries three things:
- a correctness argument;
- a status: `proved`, `verified` (exhaustively, on a bounded domain), `measured` (on the i9),
  `assumed` or `open`;
- a **cost vector**.

The cost vector has five terms:

| term | meaning |
|---|---|
| **W** | total work |
| **D** | serial dependency depth (the chain that can't be overlapped) |
| **B** | bytes moved or materialised |
| **Q** | irregular, data-dependent memory accesses (pointer chasing, scattered writes) |
| **M** | peak live memory |

Rules for the cost vector:
- **Express each term in n, r, R and the edge's own output size K.** An output-sensitive bound
  must bound K on the target data (the six authenticated blocks), not assume K is small.
- **Never convert between terms.** One random access is not "worth" k arithmetic operations.
  The i9 calibrations in the registry are the only bridge between them.
- **Never pool sequential O(n) with dependent O(n).** Emitting n bytes in order and following n
  unpredictable LF links are both O(n), and they are not the same cost. That distinction sank
  most of the failed moonshots.

**The registry** is [`docs/team/acquisition/primitives.toml`](acquisition/primitives.toml): the
edges already known, each with its status and evidence. A primitive in it is not on a whitelist.
New primitives are welcome, but each needs a derivation and a cost account before anyone treats
it as cheap.

## 3. The search: from both ends

- **Forward (what is affordable):** expand from I₀ only along edges whose cost is accounted for.
- **Backward (what is sufficient):** start from exact byte emission. Ask what minimal information
  lets the decoder emit large sequential chunks with short dependency chains.
- **A candidate** exists only where the two searches meet: a sufficient representation that is
  reachable at an accounted cost.

The forward direction alone produces interesting invariants that reconstruct nothing. The
backward direction alone produces representations nobody can afford to build.

## 4. The four certificates: the gate before any benchmark

A proposal reaches a benchmark registration only with all four certificates. This sits under
PERFORMANCE-POLICY §4a: the decision value and the causal relevance come after this.

1. **Availability.** The proposal comes with a manifest (§5): every object it uses, and its
   derivation from I₀. The validator passes on it. No unstated oracle, hidden text-order
   traversal or uncharged index.
2. **Cost.** (W, D, B, Q, M) for the **whole** decoder: construction, scheduling, emission and
   memory, not only the step it improves. Every intermediate structure's size is counted.
3. **Sufficiency.** Exact reconstruction is proved, including the primary mark, short cycles and
   blocks outside the admitted regime (which keep a fallback route).
4. **A paid-cost falsifier.** A small executable experiment aimed at the hidden cost most likely
   to defeat the proposal (the K of an output-sensitive bound, a refinement blow-up, a dependent
   chain). It runs **before** any decoder is built.

**A proposal that fails 1 or 2** stays on record as mathematics, in the journal, with its
verdict. It doesn't enter the optimisation pipeline.

## 5. The manifest

A proposal's manifest is a TOML file, checked by
`python3 docs/team/acquisition/check_manifest.py <manifest.toml>`. The checker:
- **requires** every node's inputs to be in I₀ or derived earlier (the graph is closed and
  acyclic);
- **requires** every edge to have a cost vector and a status;
- **reports** any edge marked `assumed` or `open` on the path to the output. Such a manifest
  fails the availability gate;
- **reports** the critical path: the summed D along the longest dependency chain. It also lists
  every edge whose Q or D is n-scale, so that each dependent O(n) is visible, and is never
  dropped as "preprocessing".

The example is [`acquisition/example-copy-replay.toml`](acquisition/example-copy-replay.toml).
It is round 6's potential schedule written as a manifest, and it fails on the text-order
run-label edge. That is the result we reached by hand, now caught mechanically.

## 6. Who does what

- **The Coordinator** owns the gate: no benchmark registration without the four certificates,
  and no paid run where §4a's decision question is unanswered.
- **MONODROMY** keeps the registry's proofs and verifies certificates.
- **The Researcher** reviews cost claims against measurements.
- **Calibration** of the registry's primitives on the i9 (sequential scan, dependent LF lookup,
  scattered write, two-pointer merge per event) is one short workstation recipe. It is
  proposed separately, under §4a.

## What this is not

- **It is not a ban on expensive steps.** An O(n) dependent pass is allowed when it is
  accounted for, and when what it buys is worth it. Cells itself pays one.
- **It is not a ban on exploratory mathematics.** Neon's and MONODROMY's results stay in the
  journal whatever their cost. The gate decides only what reaches engineering.
- **Near-optimality in one schedule class is not near-optimality in cost.** Cells runs within
  about 2% of the accelerated-step floor (record 80), but that floor is for one class of
  induction schedules. The representation of the steps, their locality, the output layout, and
  information the decoder already pays for and discards are all open.
