# 2026-11-09: The certified-packet census

**Result: on Wikidata, certified packets copy almost nothing that a plain pair step doesn't.**
Across the 233 full blocks (209,697,662 rows), on the walk's actual trajectory, capped at
production's markers:
- **One-cell pair batching never repeats.** Not one Ψ² visit stays in its cell for a second step:
  0 bytes in packets of 4 bytes or more.
- **Composition along k cells barely helps.** Repeated packets of 4 bytes or more cover 618 bytes
  with k ≤ 4, and 738 with k ≤ 8: 3.5 × 10⁻⁶ of the output.
- **On production's own one-hop table (single steps), the same.** With k ≤ 16, 977 bytes.
- **In the cost model,** a packet consumer does 0.50013 dependent loads a byte against the pair
  walk's 0.50013. Certification removes 314 loads in 209.7 M bytes. All of the halving against
  production's one load a byte comes from pairing, which #272 already measured as a loss
  (R / P 1.895).

So the packet consumer **can't reach C31's target**, which gates the MTF stage and the
construction, not the consumer. On this evidence it can't beat the room walk either.

Only the periodic synthetic corpora gain: one byte (mean packet 757 B) and pattern (177 B).
Even those need longer paths than the Researcher's k ≤ 4 for pairs: their RLE1 periods are 5 and 7.

**The input:** the Researcher's review 6731804749 and its report (state/reports/pair-result-
consumers-6731657578 at bc9cc4b2): certify output packets on the ≤ 3r − 2 refined cells, and
compose cells for longer repeats. Its admission condition: **actual marker-capped trajectory
coverage, not cell sizes or r/n.** The journal's target 5
(`2026-11-09-pair-from-runs-result.md`) states the same condition.

**What changed:** nothing in the decoder. `examples/packet_census.rs` counts;
`bench/packet_census.py` makes the tables. No timing was taken or read.

Labels: **checked** (by the census's own assertions or the validation, as counts),
**proved** (a short argument, given), **model** (stated assumptions, never measured).

## 1. What was counted

**The table and the walk.** These are production's (`bwt::build`, `bwt::walk`):
- the table is T[q] = L[q] | Ψ(q) << 8;
- the walk reads x₀ = Ψ(orig_ptr), then x_{t+1} = Ψ(x_t);
- each step emits L[x_t].

The census derives Ψ from the block's L itself. It checks, on every block, that its trajectory's
bytes are the crate's `walk` output, and that the block's CRC holds after RLE1.

**Two step models.**

| model | σ | a step emits | cells: maximal intervals with constant | bound (checked every block) |
|---|---|---|---|---|
| single | Ψ | L(q) | σ(q) − q and L(q) | ≤ 2r − 1 (Ψ's ≤ r translation intervals, cut by L's r − 1 boundaries) |
| pair | Ψ² | L(q), F(q) = L(Ψ(q)) | σ(q) − q, L(q) and F(q) | ≤ 3r − 2 (Ψ²'s ≤ 2r − 1 translation cells, cut by L) |

The Ψ² translation cells (constant δ alone) are counted too, with their bound 2r − 1.

**A packet from state q.** It is the Researcher's construction:
- a path of k cells c₀ … c_{k−1}, the ones the walk visits from q;
- its domain H = ∩_j ([u_j, v_j) − s_j), where s_j = δ₀ + … + δ_{j−1};
- its word W, the cells' bytes;
- its displacement Δ = s_k.

While q + iΔ ∈ H for every i < m, the walk emits W^m and reaches q + mΔ (proved: induction on the
cells). The repetitions are m = ⌊(v_H − 1 − q)/Δ⌋ + 1 for Δ > 0, ⌊(q − u_H)/(−Δ)⌋ + 1 for
Δ < 0, and unbounded for Δ = 0.

**The cap.** The copy stops at the bytes left before the next marker: a segment start that the
production walk flags, or the block's end. It may stop inside a repetition, copying a prefix of
W^m.
- **Segmented blocks** (n ≥ `MIN_SEGMENTED`, all 233 full Wikidata blocks) use production's starts:
  `random_rows(n, 511)` plus r0, sorted and deduplicated. This is `room_segments` with
  `FLAGGED_SEGMENTS` = 512, the generator copied because it is `pub(crate)`.
- **Shorter blocks** are one plain chain, with no marker but the end.
- **A pair walk's odd segment end,** one byte (PAIR_LF), counts as a packet of length 1.
- **The trajectory** is the joined output: n steps from r0, cut wherever it runs into a start.
  Segments that production walks but never joins (off r0's cycle, in periodic blocks) are not
  counted.

**The tiling.** Each segment is tiled greedily: at each packet start, the longest packet the model
allows. These models were counted:
- `single_k1`, `pair_k1`: one cell (the Researcher's single-cell batching);
- `…_k{2,3,4,8,16}r`: paths of up to k cells. A path of k ≥ 2 counts only when it repeats
  (m ≥ 2), so it is a certified repetition;
- `pair_k4path`: paths of up to 4 pair cells **without** repetition (m ≥ 1). This is an upper
  bound with a cost of its own (§5): a cached path is a super-alphabet entry, not a certified
  repeat.

**The ceiling, whatever the cells.** A repeated packet W^m with m ≥ 2 is a tandem repeat of
period |W| inside one segment. The census also counts the bytes inside any such tandem repeat,
of period ≤ 4, 8, 16, 32 and 64. Every repeated packet the models can find, with |W| at most that
period, is under this figure.

**Every packet counted is checked** on every block. The checker builds the packet's bytes from
its descriptors alone, then compares them with the walk's bytes, and its end state (q + mΔ, when
no marker cut it) with the walk's state there. The tilings also sum exactly to n.

## 2. Validation

The validation is `packet_census validate`, checked. The independent model takes Ψ from a stable
sort of L (no counting sort, no cells) and walks it. The model's walk equals the crate's
`bwt::walk` on the crate's own `build` table, from every origin.

**The columns:** every ternary column of lengths 1 to 9 (29,523), plus 120 family columns:
- the L columns of (ab)^j and (abc)^j, j ≤ 24, given directly (b^j a^j and c^j a^j b^j);
- the BWTs of (ab)^j, (abc)^j and (abc)^j ab.

**For each column, every origin and five marker sets.** The sets: none; r0 alone; production's
rule at 2 and at 4 segments; every row a start. At every trajectory position, for every model and
every path length k:

- **cells:** each cell is constant (bytes and δ, by the oracle's Ψ), maximal (its first row's
  key differs from the row before's), and the cells tile [0, n). Each bound holds:
  509,814 row checks;
- **correctness:** the packet's bytes, from the descriptors, equal the oracle's walk. Its end
  state is the oracle's, and no marker falls inside it;
- **maximality:** stepping the oracle's own Ψ, the path's cells repeat exactly m whole times
  before the marker, no more. So the census never under-reports a path.

**Totals:** 254,907 origins, 11,652,415 states and 182,754,446 path packets checked, of which
89,711,145 are composed (k ≥ 2) and 18,402,243 repeat. All pass (`validate.jsonl` in this entry's
directory).

**The families reproduce the Researcher's two counterexamples** (one chain, 60 copies):

| text | pair_k1 | pair_k3r | single_k2r | single_k4r |
|---|---|---|---|---|
| (ab)^60, Ψ² identity | 1 packet, 120 B | 1 packet | 1 packet | 1 packet |
| (abc)^60, long cells | 90 packets of 2 B (m = 1 every visit) | 1 packet (k = 3, Δ = 0) | 180 packets of 1 B | 1 packet (k = 3) |

**Mutations.** Each was caught by the validation, and the source restored after:
- repetitions + 1 (Δ > 0), or a wrong Δ < 0 branch: caught (bytes or end differ);
- no domain intersection (H from the last cell only): caught;
- under-reporting (Δ < 0 repetitions − 1): caught (not maximal);
- the cells' second byte taken from L(q): caught (a cell not constant);
- cells keyed on δ alone: caught (a cell not constant);
- cells split needlessly: caught by the bounds, and with the bounds removed, by maximality;
- the marker cap removed: caught.

This is a census's validation, not decoder safety: no unsafe code, no fuzzing.

## 3. The tables

The corpora:
- **Wikidata** is `corpora/wikidata.bz2`, SHA-256 `82f65ace…c29915` (bench/wikidata.json), 431
  blocks. The 233 full blocks hold 209,697,662 rows. The 198 short blocks, of 6 to 24 rows each
  (8,895 in all), are reported apart.
- **The synthetic corpora** are `tests/common::corpora(4)` compressed at -9, as `ab gen` at the
  bench's SCALE = 4.
  - The text corpus is read from this checkout's sources, which include this census: 12,299,623
    bytes, 2,128,278 compressed, 13 blocks. Master's text corpus differs slightly.
  - Its rows are a property of the checkout, not a pinned input.

**Cells (checked):** every bound holds on every block.

| corpus | rows | r/n | Ψ cells/r | single cells/r | Ψ² cells/r | pair cells/r |
|---|---:|---:|---:|---:|---:|---:|
| wikidata (233) | 209,697,662 | 0.0722 | 1.000 | 1.220 | 1.220 | 1.372 |
| text | 11,019,255 | 0.2337 | 1.000 | 1.486 | 1.486 | 1.815 |
| random | 8,000,003 | 0.9922 | 1.000 | 1.008 | 1.008 | 1.008 |
| one byte | 392,160 | 1.0e−5 | 1.000 | 1.750 | 1.750 | 2.500 |
| pattern | 19,600,000 | 8.9e−6 | 0.577 | 1.120 | 0.783 | 1.240 |
| runs | 254,432 | 0.1781 | 1.000 | 1.744 | 1.744 | 2.485 |
| records | 9,600,002 | 0.9968 | 1.000 | 1.003 | 1.003 | 1.003 |

Wikidata's Ψ² translation cells, 1.220 r, agree with the 2026-11-08 census's r₂/r = 1.22. The
refinement by L adds 0.15 r. The cells are short: a mean pair cell is n / 1.372r ≈ 10 rows.

**Coverage on Wikidata (233 blocks):** the share of output bytes in packets of at least L bytes.

| model | ≥ 2 | ≥ 4 | ≥ 8 | ≥ 16 | ≥ 64 | mean (B) | packets/B |
|---|---:|---:|---:|---:|---:|---:|---:|
| single_k1 | 94 B | 0 | 0 | 0 | 0 | 1.000 | 0.9999997 |
| single_k8r | 826 B | 732 B | 720 B | 54 B | 0 | 1.000 | 0.9999968 |
| single_k16r | 1,071 B | 977 B | 965 B | 299 B | 0 | 1.000 | 0.9999954 |
| pair_k1 | 0.99971 | **0** | 0 | 0 | 0 | 1.999 | 0.5001348 |
| pair_k2r | 0.99971 | 48 B | 48 B | 0 | 0 | 1.999 | 0.5001347 |
| pair_k3r | 0.99971 | 618 B | 618 B | 54 B | 0 | 1.999 | 0.5001336 |
| pair_k4r | 0.99971 | 618 B | 618 B | 54 B | 0 | 1.999 | 0.5001336 |
| pair_k8r | 0.99971 | 738 B | 738 B | 174 B | 0 | 1.999 | 0.5001333 |
| pair_k4path (no repeat) | 0.99993 | 0.99958 | 0.99800 | 36 B | 0 | 7.984 | 0.1252485 |

For the pair models, ≥ 2 is a single pair step. The remaining 0.03% are the segments' odd
PAIR_LF ends: 119,103 segments, 233 × ≈ 511.

**Per-block distribution, Wikidata's 233 full blocks (unweighted quantiles):**

| statistic | min | p10 | p25 | p50 | p75 | p90 | max |
|---|---:|---:|---:|---:|---:|---:|---:|
| r/n | 0.0533 | 0.0568 | 0.0585 | 0.0623 | 0.0724 | 0.1104 | 0.2000 |
| pair cells/r | 1.207 | 1.338 | 1.357 | 1.377 | 1.389 | 1.400 | 1.526 |
| single cells/r | 1.133 | 1.204 | 1.213 | 1.223 | 1.228 | 1.234 | 1.297 |
| pair_k1, share ≥ 4 B | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| pair_k4r, share ≥ 4 B | 0 | 0 | 0 | 0 | 0 | 0 | 0.0002 |
| pair_k8r, share ≥ 4 B | 0 | 0 | 0 | 0 | 0 | 0 | 0.0002 |
| single_k16r, share ≥ 4 B | 0 | 0 | 0 | 0 | 0 | 0 | 0.0002 |
| pair_k4r packets/B | 0.5001 | 0.5001 | 0.5001 | 0.5001 | 0.5001 | 0.5001 | 0.5002 |
| tandem ≤ 8, share | 0.048 | 0.054 | 0.058 | 0.062 | 0.064 | 0.066 | 0.074 |
| tandem ≤ 64, share | 0.049 | 0.090 | 0.098 | 0.102 | 0.105 | 0.109 | 0.114 |

**No block family gains.** The highest-r/n blocks (p90 and above, up to 0.20) and the lowest
cover the same: nothing beyond pairing.

The 198 short blocks (8,895 rows) reach 1.5% at ≥ 4 B (pair_k4r, 132 B). They are 4 × 10⁻⁵ of
the rows.

**The synthetic corpora's rows** (share of output bytes; packets/B):

| corpus | pair_k1 ≥ 4 | pair_k4r ≥ 4 | pair_k8r ≥ 4 | single_k16r ≥ 4 | pair_k8r packets/B | single_k16r packets/B | tandem ≤ 8 | tandem ≤ 64 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| text | 32 B | 0.0003 | 0.0003 | 0.0004 | 0.5000 | 0.9714 | 0.1606 | 0.1631 |
| random | 0 | 0 | 0 | 0 | 0.5001 | 1.0000 | 0.0078 | 0.0078 |
| one byte | 0 | 0 | **0.9999** | **1.0000** | 0.0013 | 0.0013 | 1.0000 | 1.0000 |
| pattern | 0 | 0 | **0.9642** | **1.0000** | 0.0210 | 0.0057 | 0.9642 | 0.9642 |
| runs | 0 | 0 | 0 | 0.0002 | 0.5005 | 0.9337 | 0.8833 | 0.8833 |
| records | 0 | 0 | 0 | 0 | 0.5001 | 1.0000 | 0.0079 | 0.0079 |

**Where the gains are.**
- **One byte and pattern** are certified almost entirely. One byte's RLE1 output, `aaaa` and a count
  byte, has period 5; pattern's has period 7. Their packets are k = 5 and k = 7 pair steps (or
  single steps), so pair composition at k ≤ 4 finds none of them.
- **`runs`,** 88% inside tandem repeats, is certified nowhere. Its repeats are RLE1's run heads
  (`xxxx`) and a long run's few chunks, each followed by different text: §4's mechanism, a
  hypothesis here.
- **The cutoffs, sorted by r/n.** Wikidata's top decile of blocks by r/n has 276 B at ≥ 4 B
  (pair_k8r) in 21.6 M rows. The bottom decile has 72 B. Every full block is one cycle (its
  cuts equal its starts).

## 4. Why: repetition in the output is not certification

**Observed:** on Wikidata, 6.1% of output bytes lie inside a tandem repeat of period ≤ 8, and
10.0% of period ≤ 64. Certified repeats cover 3.5 × 10⁻⁶. The text corpus shows the same gap:
16% against 0.03%.

**Mechanism (proved for one direction; the rest is a hypothesis consistent with the counts):**
- **What certification needs.** A certified W^m needs the m copies' start rows in one arithmetic
  progression inside H. Equivalently, the rows of the suffixes W^{m−i}X, i < m, must be evenly
  spaced, and their whole paths of k cells must coincide.
- **Long periodic stretches meet it.** They sort those suffixes into adjacent, evenly spaced
  rows: one byte, pattern, `abab`.
- **A short repeat in real text doesn't.** It is followed by text that differs (X), so the rows of
  W^{m−i}X fall wherever their continuations sort, among the other contexts beginning with W.
- **Long cells don't help.** A long cell certifies nothing unless the walk returns to it at a
  constant stride: the `abcabc` counterexample, on every block.

So the only mechanism that could reach Wikidata's 6–10% copies from output already emitted,
detected by content, not by cells. That is Kärkkäinen–Kempa–Puglisi's Precopy, or CCP 2011's
repetition recording. Its ceiling at period ≤ 64 is 10.0% of the bytes, before its detection and
copy costs. Detection costs are not counted here, and they would need their own census of
source availability across segments.

## 5. Recoverable work: a model, not a measurement

**Assumptions (model):**
- each packet costs one dependent load, its descriptor's acquisition;
- the row-to-cell directory, building the descriptors, the division for m, the stores, RLE1 and
  the segment joins are all free;
- production's room walk costs one dependent T-vector load an output byte; a pair walk costs one
  a pair step.

The model counts the dependent chain only. It gives no cycles, because #272 showed that loads
alone don't predict them.

| corpus | production | pair walk | best certified, cached (pair_k8r) | best certified, single steps (k16r) | path cache, pair_k4path |
|---|---:|---:|---:|---:|---:|
| wikidata (233) | 1 | 0.500134 | 0.500133 | 0.999995 | 0.1252 |
| text | 1 | 0.5002 | 0.5000 | 0.9714 | 0.1252 |
| random | 1 | 0.5001 | 0.5001 | 1.0000 | 0.1253 |
| records | 1 | 0.5001 | 0.5001 | 1.0000 | 0.1253 |
| one byte | 1 | 0.5003 | 0.0013 | 0.0013 | 0.1256 |
| pattern | 1 | 0.5027 | 0.0210 | 0.0057 | 0.1259 |

**The bound on Wikidata.**
- **The most a certified-packet consumer removes** is 0.49987 dependent loads a byte against
  production. Of that, 0.49987 is pairing and 1.5 × 10⁻⁶ is certification: 314 loads in 209.7 M
  bytes.
- **Rediscovering each path cell by cell** (Σk lookups, no path cache) saves 201 loads.
- **On production's own one-hop table,** certification removes 966 loads: 4.6 × 10⁻⁶ a byte.

**The path cache is not a certified packet.** `pair_k4path` reaches 0.125 loads a byte with
6.79 M distinct 4-paths, 0.45 r, about 29,000 a block. But a path is known only after its k cell
lookups (Σk is again 0.5001 a byte), unless a dense row-to-path map is built. That map is n
entries, a 4-step super-alphabet table (Kärkkäinen–Kempa–Puglisi's LF^k idea) with its own
construction. That is exactly the dense-table expense the Researcher warned of. This census
doesn't count it, and it is no argument for packets.

## 6. What it implies

**Against C31 (one-hop direct F, PR #277).**
- **What C31's gate measures.** It is D / A ≤ 0.95 at an entry point timing the MTF stage and the
  table's construction, with the room walk unchanged.
- **The packet consumer goes the other way on both counts.**
  - It needs the Ψ² cells or the pair table built. The run-built pair table cost 5.668 nominal c
    a row on Wikidata in #272, against the one-hop build's 2.213.
  - It changes the consumer, which C31's gate doesn't time.
- **Nothing to repay the build.** With certified coverage at 3.5 × 10⁻⁶, the consumer has no
  saving to repay a dearer build. **It can't beat C31's target, and it can't displace C31.** The
  Researcher's condition for displacing it ("greater recoverable whole-decoder work after all
  induced costs") fails at the first count.

**Against the room walk.**
- **A packet consumer on Wikidata is a pair walk,** load for load (0.500133 against 0.500134), with
  descriptor arithmetic added.
- **The pair walk lost at that load count.** It was measured at that same count in #272: R / P
  1.895 at the entry point, with its arithmetic residual about 2.1 c/B above production's.
- **So packets can't move the joint frontier.** That frontier needs the builder ≤ 1.904 c/B with
  the consumer at ≤ 3.971. Packets leave the consumer's load count where the pair walk's is.
- **Any pair-consumer gain has to come from the step itself:** C37's shared-offset rooms, C38's
  tags. It can't come from skipping steps.

**Where a packet path would win:** periodic blocks, like one byte and pattern. There it removes
98–99.9% of the loads, but only with paths as long as the period, so k ≥ 5 to 7 for these two.
- Wikidata's blocks have none; no full block exceeds 0.02% coverage.
- Only an admission rule that fires on such blocks could use them. That rule would have to be
  counted on release-like corpora first, and no one is proposing it.

**Recommendation:** close certified packets as a consumer angle for the run-fed line. Record it in
§11's ranked targets as counted and negative on Wikidata. Keep C31 first.

Content-detected repetition copying (Precopy) has a ceiling of 6.1% (period ≤ 8) to 10.0%
(period ≤ 64) of Wikidata's bytes. That is a different mechanism. It would need its own
detection-cost census before any registration. This entry doesn't propose one.

## 7. Not done

- No timing, no decoder change, no consumer implementation, no admission rule.
- No descriptor-format design: whether a descriptor fits a word, and the directory's cost.
- No count of the segments production walks but never joins (off r0's cycle). They exist only in
  periodic blocks; Wikidata's full blocks are one cycle, which is why their cuts are 511 a block.
- Not done for the cooperative kernel's ranges (`coop::walk_room`) or the parallel decoder's
  chunking. Their markers differ from `room_segments`' and could only cut packets shorter.
- No census of the Precopy mechanism's detection cost or source availability.
- The greedy tiling takes the longest packet at each start. A different tiling could change the
  periodic corpora's packet counts slightly, but not Wikidata's: no position there has a packet
  above 2 bytes, except the few hundred counted.

## Reproduce

```text
cargo build --profile fuzz --example packet_census    # overflow checks and debug assertions on
target/fuzz/examples/packet_census validate           > validate.jsonl
target/fuzz/examples/packet_census census corpora/wikidata.bz2 wikidata > wikidata.jsonl
target/fuzz/examples/packet_census synthetic 4        > synthetic.jsonl
python3 bench/packet_census.py wikidata.jsonl synthetic.jsonl > tables.md
```

The outputs are in `docs/journal/2026-11-09-packet-census/`:
- `validate.jsonl`;
- `tables.md`, the full tables, every model;
- the per-block lines, gzipped. Uncompressed SHA-256:
  - `wikidata.jsonl` `d14d92de15760e6f92c9428827bee2576f5efeb3ff41d24207dc610fd5bb3a2c`;
  - `synthetic.jsonl` `be645f548dfe14b2616bada38a57953768e9545d0f28997f1b6eb0fb4871cc97`.

The census source is `examples/packet_census.rs`, SHA-256 `a2cded2b…f2286e`. It ran on this
shared machine with LTO off for the build only. The counts don't depend on it.
