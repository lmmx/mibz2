# Research context for exact inverse-BWT reconstruction in mibz2

*20 November 2026. The Coordinator. A source-grounded record of what the kernel research has
established, for a fresh researcher. The method is [METHOD.md](METHOD.md), the living record of
questions and obstructions is [FRONTIER.md](FRONTIER.md), and the feasibility leads are
[OPEN-QUESTIONS.md](OPEN-QUESTIONS.md).*

**How to read the claims.** Each is tagged:
- **proved:** a proof exists and was independently checked;
- **verified:** checked exhaustively on a bounded domain, or on real blocks;
- **measured:** a native timing or counter;
- **refuted:** shown false, with the counterexample or data;
- **open:** not settled.

Record numbers (`record NN`) are MONODROMY's verification records, cited by their REVIEW comments
on issue #145. The repository on `master` is authoritative over anything summarised here.

## 1. The problem and its entry state

- **The setting:** a bzip2 block of n ≤ 900,000 bytes reaches the inverse BWT as **capped run
  records**: R of them, each a (byte, length ≤ 127) pair, with R ≥ r, the number of maximal runs.
- **Also at entry:** the primary row p, n, and the symbol counts C where the route has them (one
  sequential scan of the records otherwise).
- **The job:** write the n bytes of the block exactly. The marked LF orbit from p, read in reverse,
  is the text.
- **The entry state I₀** is these objects and nothing else. Text-order positions, row labels, LF
  orbit order, copy sources, suffix-array samples and dependency graphs are **not** in I₀. Each costs
  something to obtain ([ACQUISITION-FIRST.md](../ACQUISITION-FIRST.md) §1).
- **The regime that matters:** Wikidata JSON. Production admits the Open/LazyCells route at
  1/64 ≤ R/n < 0.15. That is 228 of the bench's 233 large Wikidata blocks, at r/n of about
  0.057–0.143. Prose and code sit at r/n of 0.28 or more and take other routes.
- **The test data:** six authenticated admitted blocks, 0, 2, 85, 143, 207 and 210, on branch
  `data/admitted-bwt` (`admitted-bwt/manifest.json` gives n, p, r and SHA-256).
  - They are raw last columns, one byte per row.
  - LF is stable rank: LF(i) = C[L[i]] + #{j < i : L[j] = L[i]}.
  - Block 0 is the "frozen" block: n = 899,931, r = 51,184, R = 56,386, p = 264,045.

## 2. The production kernels

**Cells / LazyCells** (`src/cells.rs`, `src/cells_lazy.rs`): Rauzy–Veech induction on the discrete
interval exchange that LF is, cut at the run boundaries. Return words are stored as a
straight-line program (SLP), and a one-scan writer emits the output.
- **The induction schedule is near its floor (proved and verified, records 79–80):**
  - Neon's exact work identity counts the steps of any monotone component-suffix schedule.
  - Shipped Cells takes 1.013–1.019r accelerated (winner-run) steps, against a floor of r.
  - Unaccelerated, it takes 1.68–1.87r steps, against a floor of 2r − |D̄∩C̄| (≈ 1.21r on block 0).
  - **This is a structural bound on one schedule class, not a bound on decode time.**
- **Its paid cost is elsewhere (measured, Astra D):** about 7.33 modelled DRAM bytes per output byte
  on block 0, against a compulsory floor near 1 B/B. Grammar rules, the word arena and the writer
  dominate.
- **Prefetching:** Cells fetches about 12 slots ahead. Real Wikidata exposes 4–5 independent
  induction fronts, so a parallel-front schedule (Theorem 3′, verified) shortens the dependent chain
  3.8–4.9×. But Cells is not latency-bound on the cloud CPU (IPC ≈ 2.9), so this buys little there
  (record 79).

**Open** (`src/open.rs`, `src/open/endpoint.rs`, proof in
`docs/journal/2026-11-18-open-return/proof.md`) is the production kernel since 0.5.7 on admitted
blocks.
- **The construction:**
  - retain every thin run (length ≤ LIMIT = 16) plus the primary row;
  - the m retained rows are about 10–11% of n;
  - eliminate the H thick strips by endpoint induction on the strips and the gaps between them.
    That is at most 3H + 2 arrows. Thin runs never enter the exchange;
  - each retained row gets its **first return**: the next retained row, plus the word spelled on
    the way. The factorisation is g = P∘LF_{L[S]};
  - a band dictionary spells the words; queued wiring fills one u32 edge per retained row.
- **Emission:** walk the retained rows from the mark, appending each word.
- **Shipped arm:** `Open<16, INDUCE=true, QUEUE=true>`.
- **What it gained (measured):** 0.5.7 against 0.5.6 is 7–11% faster on Wikidata at 8–20 threads.
  Peak RSS at 8 threads fell from 125 to 96 MB. Admitted blocks run at about 0.97× the time of the
  kernel before; text would run at 5.1× if it weren't gated out.
- **The phase split of shipped Open (measured, PR #653, six blocks, single thread):**

  | Stage | Share of total |
  |---|---|
  | Run pass, including the queue fill | 28–36% |
  | Elimination, with its setup and bands | 8.5–18.5% |
  | Spelling | 4–9% |
  | Wiring scatter | 3–5% |
  | **Emission (the walk)** | **36–48%**, with the most data-cache misses |

  The total is 1.6–3.7 ms per block.

**Scalar-cap** is Astra E's fused front-and-Cells worker (lineage d0a42980). It is a run-record
kernel, not a dense LF table.
- **Measured:** 0.86–0.88× LazyCells' whole-decode time on the i9.
- **Its engineering trade-off:** it fuses the front end with the inverse, and walks capped records
  with several couriers.
- **No harness exists** to time it as a column-in, text-out kernel against Open.

## 3. Results that survive (keep these, don't re-derive them)

| Result | Status | Where |
|---|---|---|
| The open first-return factorisation g = P∘LF_{L[S]}; endpoint elimination in ≤ 3H + 2 arrows | proved | open-return proof.md |
| Exact work identity for monotone induction schedules; floors 2r − \|D̄∩C̄\| and r (accelerated) | proved and verified | records 79–80; gemini-neon round 4 |
| Cells within about 2% of the accelerated floor | verified | record 80 |
| Run-boundary bound r(T_X′) ≤ r + (c − 1) + \|∂X′ \ (D ∪ C)\| | verified | record 78 |
| Parallel multi-front RV (Theorem 3′): exact | verified on 29,395 batches | records 78–79 |
| Weight-only RV recovers the text positions of all run heads and tails and the mark, with integer state, in Cells' step count | verified (corrected) | record 81. These are the r-index's run-boundary SA samples. Not new, but a neat O(r)-state route. |
| NOP-7 / Φ copy scheme from those samples: r literals, k = r − \|D̄∩C̄\| copies | verified | round 5 |
| \|D̄∩C̄\|/r ≈ 0.75–0.79 on Wikidata | measured | record 80 |
| k-step persistence identity \|U_k\| = Σ min(k, ℓ(c) + 1) = r_k^word | verified on the six blocks and exhaustively; the corrected Theorem 1′ (r_k^aff ≤ r_k^word) holds | NSPC round |
| Repeated open cuts shrink fast: 899,931 → 110,273 → 17,657 → 3,916 → 767 rows (block 0) | measured | astra-investigation-3 |

## 4. Refuted or rejected (don't revisit without a new idea)

Each row is correct mathematics that failed as engineering, unless marked otherwise.

| Candidate | What it tried | Why it lost | Evidence |
|---|---|---|---|
| Sparse exceptions, copy seeds (Astra E) | Reconstruct from rare exceptions to a simple rule | Acquiring the exceptions cost about 2.35× the inversion | Astra E ledger |
| Rigid tiles and modular translations (Astra B) | Replace LF by a tiled shift plus exceptions | The best rotation agrees on 26,413 of 899,931 rows; a packed model still needs ≥ 37,807 cache lines | astra-b-collective |
| Forest and boundary parentheses (Astra inv. 3) | Erase interior LF edges; match parentheses | 15× slower than Cells: deriving the forest is the cost | astra-investigation-3 |
| Rank-order local elimination, two-list chains | Other sections | 3.8–4.1× slower than Cells | astra-investigation-3 |
| First-occurrence writer | A different writer schedule on the same grammar | 1.79× slower: mispredicts 6× as often despite 14% fewer instructions | first-occurrence-writer |
| RV-Copy (Neon round 5) | Replay Φ copies by peeling | 3.4–8.7r fragments of about 1.8 B; refuted by its own registered rule | gemini-neon round 5 |
| In-place Φ copy schedule (round 6) | Potential τ = θ + φ(run) | Valid schedules of 0.87–2.15r fragments, but building τ needs text-order run labels, an O(n) arc walk, and run labels determine the text (record 84) | records 82, 84 |
| NSPC (non-splitting affine contraction) | Precompute K-step jumps per capped record | Byte-exact, with 87–96% of rows at full stride; native time 1.82–2.38× Open. Building the jumps cost about one Open, and the walk's scans another | #652 |
| DOE (decoupled Open exchange) | Exchange on thick strips only | **Not a benchmarked loss:** it re-derived Open's own construction. Identical exit partition (MONODROMY). Its sharper bound was false; the corrected one is ≤ c(E) + \|∂U\| ≤ 4m. Its direct wiring was already measured, at 1.21× | MONODROMY REVIEW 7028091072 |
| F: fuse band construction with wiring | Remove the `bands` and `e.bottom` arrays | 1.02–1.04× Open: the fused loop is slower than the two loops it replaced | #654 |
| Q: packed run-granularity queues | One u32 per thin run | Fill −46–54%, wiring 4.5–6.4× slower; 1.05–1.09× Open | #654 |
| One-step pipelined emit | Load the next edge before spelling the current word | Exact over 64.35M cases. Lost the inlining of `append_code`: 48 against 26 instructions per row. Never timed | #656 |
| Adaptive thin-run LIMIT | Choose 8, 12, 16 or 24 per block | 16 is the best fixed limit. An honest per-block choice gives 0.992, and hindsight only 0.990 | #657 |

## 5. The method lessons, in the order they were learned

1. **Acquisition first** ([ACQUISITION-FIRST.md](../ACQUISITION-FIRST.md)): no object enters a
   decoder without a costed derivation from I₀.
   - The cost vector has five terms: W (work), D (dependency depth), B (bytes moved), Q (irregular
     accesses) and M (peak memory). Never convert between them.
   - **Never pool sequential O(n) with dependent O(n).**
   - The manifest checker, `docs/team/acquisition/check_manifest.py`, catches unpaid edges and
     paths that "emit in disguise": any object that determines the text, such as run labels.
2. **Producer plus consumer cost** (`docs/journal/2026-11-20-producer-consumer-cost.md`): cheap
   information can be expensive to consume. A proposal must show where work **moves**, not only
   where it disappears.
3. **Three levels of progress.** Almost every disappointing idea passed the first and failed the
   second or third:
   - **mathematical possibility:** can it be described and proved?
   - **computational feasibility:** can its information be obtained and consumed economically?
   - **engineering competitiveness:** does the whole decoder beat Open, measured?
4. **Structural bounds are not time bounds.** "Within 2% of an induction-step floor" says nothing
   about whole-decoder time. A dependent-load count says nothing about cycles when the CPU already
   overlaps the loads.
5. **Check the headroom before predicting.** The LIMIT study died at "would a perfect choice even
   be worth it?" Ask that first.
6. **Noise:** 2-lap medians on a shared VM produced a false "a third of blocks prefer LIMIT=8".
   Paired, interleaved, many-round designs with honest out-of-sample selection are the standard.
7. **Ambition is set separately from scoring** ([METHOD.md](METHOD.md) §1–3).
   - The gates decide what earns engineering investment; they don't decide what gets explored.
   - After the incremental attempts on Open, the programme deliberately favours foundational and
     architectural change.
   - It does this through a recurring explore → triage → deepen → confront → learn cycle, with
     two tracks.

## 5a. The fidelity ledger

Cheap predictions set against native outcomes, for the candidates that reached a native test. This
is a record, not a gate.

| Candidate | Cheap prediction | Native outcome | Agreed? |
|---|---|---|---|
| NSPC (#652) | dependent steps 0.25–0.34× (Python op counts) | 1.82–2.38× Open's time | no |
| Q, packed queues (#654) | 8–15% faster (Neon's estimate) | 1.046–1.093× Open | no |
| `i16` against `qi16` (#653; calibration) | `i16` better on L2 misses, DRAM traffic and work (Neon's C cache model) | 1.10–1.16× slower | no |
| LIMIT=8 (#657) | wins on 30% of blocks (2-lap medians) | wins on 0 of 228 | no |
| Backward Open (Neon cycle 2) | work 0.72–0.74×, L2 misses 0.82–0.88× (C cache model) | 1.10–1.15× slower than `qi16` on all six blocks (#659): the second pass over the records costs about as much as the queues it replaces | no |

## 6. Tools and where things are

| What | Where |
|---|---|
| Production kernels | `src/open.rs`, `src/open/endpoint.rs`, `src/cells.rs`, `src/cells_lazy.rs` |
| Open's exhaustive arrow audit | `src/open/checks.rs` |
| Open's timed phase split | branch `research/open-phase-split` (#653), feature `open-phases` |
| Timed census and limit sweep | branch `bench/open-admission`, `ab census-timed`, `ab limit-sweep` |
| Acquisition registry and manifest checker | `docs/team/acquisition/` |
| Neon's earlier rounds, with verifiers and outputs | `docs/journal/2026-11-19-gemini-neon/` |
| Astra's investigations | `docs/journal/2026-11-18-astra-*` |
| The open-return research arms and evidence | `docs/journal/2026-11-18-open-return/` |
| Real blocks | branch `data/admitted-bwt` |
| Performance policy (how a change is adopted) | `docs/team/PERFORMANCE-POLICY.md` |
