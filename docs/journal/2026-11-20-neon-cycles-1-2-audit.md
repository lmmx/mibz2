# Neon's Antigravity cycles 1–2: prior-art and engineering audit (PROVISIONAL)

*20 November 2026. The Coordinator.*

**Status: provisional, partly confirmed against the artifacts.** Neon pushed at 5cdd0cb7 (contains b69ffcd). This audit is based on two things only:
- Neon's handoff report as Louis relayed it;
- production source on master.

Neon's committed artifacts are on `research/neon-bipolar-contraction` (b69ffcd, 5cdd0cb). They were not on origin when this was written, so none of its verifiers, manifests or C models have been re-run here. Each finding is marked as one of:
- **prior art:** an identification of ancestry, not a falsification;
- **equivalence:** a claim that two constructions are the same object (each one still to be checked);
- **correctness:** an objection to a stated claim;
- **cost model:** a limit of a simulated figure;
- **engineering:** an actionable next step.

**Structural resemblance is not proof of equivalence.** Naming an ancestor explains where an idea comes from, but not necessarily all that it contributes. A BWT-native construction of a known object can still be a distinct result.

## 1. Queue-Free Backward Open: the one engineering candidate

**Prior art: the inversion-direction duality.** There are two classical ways to invert a BWT.
- **Walk LF, write the text backwards.** This is Burrows and Wheeler's 1994 decoder (Algorithm D). It needs only C[c] and a running rank, both sequential.
- **Walk Ψ = LF⁻¹, write the text forwards.** This is bzip2's decoder. It first builds the T-vector, `tt[cftab[ll[i]]++] = i`, a counting scatter whose only job is to invert the map.

**Engineering: Open made the bzip2 choice at the level of retained rows.** The source confirms this:
- `emit` (`src/open.rs:451`) follows `edges[j]` and appends words forwards;
- `edges` is filled as `edges[dst] = j`, with the inverse arrows (the `QUEUE` block of `construct`);
- `proof.md` states it: "these inverse arrows spell the marked text forwards";
- the stored words are reversed for forward emission (`open.rs:156`).

The per-symbol queues play the part of bzip2's `cftab` scatter. Backward Open takes Burrows and Wheeler's side instead: it stores g, as `edges[j] = dest`, written sequentially in j, walks g, and writes the output right to left.

**Novelty in mibz2: none found.** No backward or descending emission appears in `src/open*` or the Open journals.

**Equivalence, to check: the nearest existing control is `i16`** (`Open<16, INDUCE=true, QUEUE=false>`).
- Its wiring loop already visits thin rows in domain order, keeps one band cursor per symbol (`cursors[a]`), and computes `dest = z + delta`.
- Neon's 256-entry ActiveBand table appears to be that cursor array.
- The difference: `i16` writes `edges[dest] = j` (scattered) and walks forwards. Backward Open writes `edges[j] = dest` (sequential) and walks backwards.

**Cost model: what the C model may miss.** `i16` lost on hardware.
- **Its loss:** wiring cost 1.6–1.8× queued wiring, and the total was 1.10–1.16× `qi16` (PR #653).
- **What Backward Open removes:** its scattered write.
- **What it appears to keep:** `i16`'s second pass over the run records, needed because the bands exist only after the elimination.
- **The decisive unknown:** how `i16`'s loss divides between those two costs. The model's 0.73× in primitive work does not answer it.

**Integration checks:**
- **The forward pipeline:** `finish_block` (`src/parallel.rs:1021`) runs `unrle` and the CRC as a separate forward pass over the finished inverted buffer, so filling it backwards breaks no fusion.
- **The output buffer:** `emit` now does `out.clear(); out.reserve(n)` and pushes. Writing backwards in safe Rust needs a sized buffer. Without reusing a worker's buffer, that means zeroing about 900 KB, roughly 1–2% of a 1.6–3.7 ms block. It belongs in the measurement.

**Engineering: the proposed native experiment.** It is research only and is registered before its data. It compares three variants on the six blocks, with the same harness, cachegrind runs, and the phase split.

| Arm | Construction | Edges | Output |
|---|---|---|---|
| `qi16` | queued (shipped) | inverse | forward |
| `i16` | queue-free | inverse, scattered | forward |
| Backward Open | queue-free | direct, sequential | backward |

- **Separating the two changes:** report phase-level timings for the wiring and emission stages separately, so that any gain can be assigned to the edge orientation or to the emission direction.
- **Memory:** include the output buffer and its zeroing or reuse.
- **Reading the outcome:**
  - beats both: the direction reversal is worth having in mibz2;
  - beats `i16` only: a real repair of a design that had lost, but not a production win;
  - loses to both: the cost model did not represent the hardware.

**Status: the G5 miss stands as recorded** (block 0, by 1–4%). This experiment would be a separate, prospective gate. It does not reinterpret G5.

## 2. Bipolar and Half-Bipolar contraction

**Prior art: the SA-IS and Ko–Aluru type structure.**
- **The type correspondence:** Ψ(j) is the row of the next suffix. Ψ(j) > j means that suffix is smaller than its successor (SA-IS's S-type); Ψ(j) < j is L-type.
- **The Monotone Sign Decomposition** (Ψ(j) − j non-decreasing within each F-bucket) matches induced sorting's bucket property: L-type suffixes at each bucket's head, S-type at its tail.
- **The "valleys"** (R reached from L) match LMS positions, and the ≤ n/2 halving matches the LMS bound.
- **Half-Bipolar,** which keeps the larger type, matches Ko–Aluru (2003), which recurses on one type.
- **Neon's own label:** it called the construction an "inverse dual of SA-IS".

**Equivalence: open.** The partition, the recursion and the halving match. Whether Neon's exact interval-exchange contraction, with its return words and its handling of the mark, is equivalent to an SA-IS or Ko–Aluru reduction needs the return-word transformations examined. Sent to MONODROMY.

**Possibly new:** using that reduction as an exact inverse-BWT contraction. Okanohara–Sadakane (2009) used induced sorting to *construct* a BWT. A literature check on inversion is for the Researcher.

**Engineering: rejected, correctly.** Dependency depth falls to about 100. But L2 misses rise 6.3–9.1×, DRAM bytes per output byte 3.4–11×, and peak memory reaches 4–9 MB. Neon killed it itself.

## 3. Monotone alphabet-cut BWT recompression

**Prior art: Jeż's recompression.** This is the pair-compression step with an alphabet partition: merge every pair ab with a ∈ Σ_L and b ∈ Σ_R. Because of the partition, pairs cannot overlap.
- **Nearby work:** prefix-free parsing (Boucher et al.), which builds a BWT from a parse's BWT.
- **In mibz2:** Astra's token BWT over word cylinders.

**Possibly distinct:** doing the step directly on the BWT, as one order-preserving run-level scan. The new symbols are interleaved (a < X_{a,b} < a+1) and the primary row is carried through. Neon verified it on 5,227 exhaustive cases. It may be a result in its own right; literature check pending.

**Engineering: rejected, correctly.** It deletes about 20% of rows per round, so it needs more than 24 rounds. The alphabet then grows to thousands of concurrent read cursors.

## 4. Other Cycle 1 findings

- **The metadata-density obstruction:**
  - *Prior art, within the project:* it is the reason Open beat LazyCells (6.29 MB heap against 15.39 MB), and the subject of Astra D's memory ledger.
  - *Value:* it is a useful unifying statement.
- **The displacement-rank obstruction:**
  - *Prior art:* classical structured-matrix theory (Kailath–Kung–Morf, 1979). An r-piece permutation has low displacement rank.
  - *Correctness:* "Ω(n log n)" holds for the explicit generator representation it examined. It is not a bound on every method that exploits the structure.
- **The Φ overlap count:**
  - *Engineering, and a positive result:* the overlap transitions number 1.25–1.37r on the six blocks. That is O(r), not n-scale, which answers part of record 82's question: the object is small.
  - *What remains open:* obtaining its coordinates. The boundary positions cost Cells' step count (record 81).
  - *Status:* killing it on headroom is fair for now. The count belongs in FRONTIER.md as an open, positively measured lead.

## 5. Cycle 2's foundational claims

- **"Entropy barrier":** a correctness objection, as stated.
  - *The claim:* log₂(m!) ≈ 410 KiB exceeds L2.
  - *The problem:* the same report gives the retained column v = L[S] as 88–199 KiB and L2-resident, and v with P determines g. So the bound applies to an **explicit** edge table only.
  - *The counterexample:* the SIMD-rank alternative it set aside is classical occurrence-table LF on v. It is slower per step, but its existence refutes "incompressible".
  - *Restate as:* "an explicit edge table does not fit L2".
- **Incompressible residual embedding:** a correctness objection to how it was used, not to the construction.
  - *What it shows:* plausibly, that hard worst-case instances exist in the admitted family.
  - *What it doesn't show:* that Wikidata blocks are hard in that way, or that every algorithm must pay Open's costs on them.
- **The No-Widening Lemma:** a correctness objection to its scope.
  - *What it refutes:* cutting g again by **width** (OPEN-QUESTIONS.md §1 as worded).
  - *What it doesn't touch:* a second cut by section **structure**. Astra investigation 3's cascade (110k → 17.6k → 3.9k rows) used chord sites, not width.
  - *Follow-up:* §1 should be reworded accordingly.

## 6. What Neon contributed

| Item | Classification |
|---|---|
| Backward Open | Known duality, newly applied to Open, untried in mibz2. The one engineering candidate |
| Half-Bipolar contraction | SA-IS/Ko–Aluru type structure; possibly a new use for inversion; equivalence open; correctly rejected |
| Alphabet-cut recompression | Jeż recompression carried out in BWT space; possibly distinct; correctly rejected |
| Metadata density | Project knowledge, restated well |
| Φ overlaps = 1.25–1.37r | New measurement, positive for an open question |
| Entropy, incompressibility, displacement and no-widening claims | Overclaimed: narrow each to the construction it examined |
| G5 miss | Stands as recorded |

**The research process was good.** Neon falsified its own leading candidate (Half-Bipolar) on cost, and it reported its G5 miss. The point is to strengthen the claims, not to tighten the charter.

## Next

Once the branch is on origin:
1. **Fetch and re-run:** fetch the exact commits, and re-run the verifiers and `check_manifest.py`.
2. **Read the model:** read the C cost model against Open's source.
3. **Send to MONODROMY:** the Backward Open identity, primary-mark transport, short cycles, and the Half-Bipolar equivalence question.
4. **Finalise:** update this audit and merge it to master, so Neon reads it at its next cycle boundary.
5. **Native experiment:** register the three-arm experiment (§1) before any data.

## Artifact checks (5cdd0cb7)

- **Commits:** both are present and authored as Louis, with no attribution lines.
- **Manifests:** `check_manifest.py` passes `half_bipolar.toml` and `backward_open.toml`.
- **G5 falsifier:** `g5_backward_open.c` reproduces Neon's table exactly, including its
  pre-registered verdict, **G5: FAIL**.

### Calibrating the cost model on a known pair

[`calib_i16.c`](2026-11-20-neon-cycles-1-2-audit/calib_i16.c) includes Neon's model unchanged and
adds an `i16` arm: Open with QUEUE=false, a second run pass, and scattered writes `edges[dest] = j`.
All three arms produce byte-identical output on all six blocks.

The model's ratios to `qi16`:

| Block | i16: L1 misses | i16: L2 misses | i16: DRAM | i16: work | i16: memory | Backward Open: L2 misses | Backward Open: DRAM | Backward Open: work |
|---|---|---|---|---|---|---|---|---|
| 0 | 0.94 | 0.86 | 0.83 | 0.88 | 0.84 | 0.86 | 0.84 | 0.73 |
| 2 | 0.93 | 0.87 | 0.81 | 0.86 | 0.82 | 0.85 | 0.82 | 0.73 |
| 85 | 0.90 | 0.83 | 0.75 | 0.86 | 0.76 | 0.82 | 0.75 | 0.72 |
| 143 | 0.94 | 0.89 | 0.74 | 0.88 | 0.77 | 0.88 | 0.74 | 0.73 |
| 207 | 0.94 | 0.87 | 0.79 | 0.90 | 0.80 | 0.87 | 0.79 | 0.74 |
| 210 | 0.93 | 0.87 | 0.72 | 0.90 | 0.75 | 0.86 | 0.72 | 0.73 |

**The model ranks the known pair wrongly.**
- **The model's view:** `i16` is better than `qi16` on every metric.
- **Native:** `i16` is **1.10–1.16× slower** (PR #653).

**It cannot tell Backward Open from `i16` on cache traffic.**
- Its L2 and DRAM ratios for Backward Open are within about 0.01 of those for `i16`.
- The only difference it shows is work: 0.73 against 0.87, from sequential rather than scattered
  edge writes.

**Verdict on the model: diagnostic only (METHOD.md §5).** It cannot predict native competitiveness
for this family. Neon's G5 figures for Backward Open are no evidence either way.

**What the native question reduces to:** whether sequential edge writes, plus backward emission,
recover more than `i16`'s native loss. Only the native three-arm experiment (§1) can answer that.
