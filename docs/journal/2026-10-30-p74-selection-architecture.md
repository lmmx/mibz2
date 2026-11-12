# P74 after Gate 1: where the selection's information could come from (read-only assessment)

Requested by the maintainer after Gate 1's negative verdict (external report 41). Nothing
is implemented or measured here; every cost below is an estimate. Any experiment that
follows needs its own pre-registration, and must charge the full cost of classification
wherever that cost lands, the scanner included.

## The constraint Gate 1 measured

On random and records the selector sent every block to the shipped kernel, so its whole
cost there was classification:
- **The registered 3% limit** means classification plus dispatch must cost ≤ 0.03 × the
  shipped kernel on those corpora: about 0.17 ns/B on Zen 5's 5.5 ns/B.
- **These corpora carry about one symbol per output byte** (0.99 on random, 0.997 on
  records; Wikidata 0.06 explicit tokens plus its RUNA/RUNB digits). So the budget there
  is about **0.17 ns per symbol**, roughly 0.8 cycles at Zen 5's clock.
- **The frozen count cost about 0.4–0.5 ns per symbol:** baseline SSE2 with u64 sums,
  about 30 instructions per four symbols.

## Where the ranks become available

`read_item` in `src/block.rs`: the scanner's per-group loop, on the scanner's single
serial thread. Each `step!` is one 10-bit table lookup that yields one or two symbols:
- `e >> SYM1 & 0x1ff`, then `e >> SYM2` when the entry is a pair;
- each written to `out[n]` in `syms`' spare capacity;
- plus the slow path's one symbol (`decode_entry_slow`) and the group's odd last symbol
  (`t.decode`).

An explicit rank is q = s − 1 for 2 ≤ s < eob. So every symbol the count needs passes
through registers in that loop, once.

## Option B: count in the scanner, as the symbols are produced

**What it takes:**
- **The counting:** two counters (explicit, high) updated from each decoded symbol's value
  (2 ≤ s < eob, s ≥ 33). That is about three or four ALU operations per symbol: compares
  and adds, branch-free, off the bit-position dependency chain (`skip_short(e & 31)`).
- **No per-entry shortcut.** Precomputing the classes into the table entries doesn't fit:
  - the entry's bits are taken (0–4 length, 6–10 first length, 11–12 count, 13–21 first
    symbol, 22–30 second, 31 BAD); only bit 5 is spare;
  - a parallel per-table class array would add 1 KB per table (6 KB for six) beside the
    24 KB of lookup tables the scanner keeps in L1;
  - u64 entries would double the 24 KB.
- **Storage:** two u32s per block in `Symbols`.
- **No synchronisation:** `Symbols` moves by value from the scanner to the worker that
  decodes the block.

**What it costs and risks:**
- **API:** `Symbols` is `pub` with all-`pub` fields, so new fields break struct-literal
  construction downstream and in the crate's own tests (`tests/mtf_checkpoints.rs`
  builds them). That is an API change.
- **The serial stage.** The scanner is the pipeline's one serial stage, so the cost lands
  there:
  - **Wikidata:** the CI model has the workers binding at 4 threads and the scanner from
    about 12. Added scanner work is hidden in wall time below that, but counted in CPU-s/GB.
  - **Random and records:** the scanner does a symbol per output byte. The added
    3–4 ops a symbol are a share of its per-symbol work that wasn't measured, and the
    full-pipeline accounting would have to show it.
- **The rule is unchanged:** exact F and E for the whole block, before MTF.

## Option A: a standalone count with AVX2 and 32-bit sums

**What it takes:**
- One pass over `syms` (u16) at the start of the worker's MTF stage, in a
  `#[target_feature(enable = "avx2")]` function.
- 16 symbols per vector: three compares and two masked subtracts into u16 lane
  counters, widened to u32 every few thousand iterations.

**What it costs:**
- **Compute:** about 0.1–0.2 cycles per symbol.
- **Memory, probably the larger cost on random and records.** A block's `syms` there is
  about 1.8 MB (900k symbols × 2 B), written by the scanner's thread on another core, so
  the pass streams it from L3 or a remote L2. At an assumed 20–30 GB/s that is about
  0.07–0.1 ns per symbol: about 1–2% of shipped on random, inside the 3% limit but not by
  much. On Wikidata it is negligible (about 0.1 symbols a byte).
- **No API change,** the whole-block rule exactly as frozen, and the cost inside the MTF
  stage, where Gate 1's boundary measures it.

## Option C (not asked, but follows from the code): no classification pass at all

Run P74 from the block's start and count only what it already sees:
- F, incremented in the fallback branch only (rank ≥ 32), which has no cost on the fast
  path;
- E, one add per explicit token (or derived from the loop's state at a probe).

**The switch:**
- **When to decide:** at a fixed probe, for example after the first few thousand
  explicit tokens or at the first 16 KiB checkpoint the checkpoint kernel already stops
  at.
- **The decision:** if 10F > E, convert to the shipped kernel's list for the rest of the
  block. The conversion is exact, since P74's head and tail hold the same logical list:
  store the head's 32 bytes and copy the 224-byte tail once.
- **No extra pass.** `syms` is read once, by MTF, as now.

**What it costs:**
- **On random and records:** the cost is P74's slower fallback for the probe window only.
  At about 20% slower over the first few thousand of about 900k tokens, that is well
  under 0.1% of the block.
- **On Wikidata:** a counter in the rare fallback path, one add per token, and the probe.

**But it changes the rule.** A prefix decides, not the whole block's F/E. A block whose
rank mix shifts after the probe is misclassified for its remainder. That is a new rule,
needing its own pre-registration and its own measurement.

## Comparison

| | where the cost lands | extra pass | API change | rule | estimated cost on random/records against a 3% budget |
|---|---|---|---|---|---|
| A: AVX2 count | the MTF stage (measured by Gate 1's boundary) | yes, about 2 B/symbol from L3 | none | the frozen whole-block rule | about 1–2% (memory-bound estimate) |
| B: scanner count | the serial scanner | none | `Symbols` gains fields | the frozen whole-block rule | unmeasured share of the scanner's per-symbol work; needs full-pipeline accounting |
| C: online switch | the MTF stage | none | none | a prefix rule (new) | well under 0.1% (probe window only) |

**The coordinator's reading** (corrected after external report 42):
- **C has the lowest classification overhead in principle,** with no pass, no
  serial-stage cost and no API change. Calling it "the most economical design", as the
  first version of this note did, was premature. What C still needs:
  - **a decision rule** with a bounded loss. A fallback-heavy block of n explicit tokens
    pays about δ·p/n for a probe of p tokens (δ: P74's slowdown there), plus the switch's
    fixed cost over the shipped kernel's time;
  - **a short-block analysis,** since a block shorter than the probe pays δ in full;
  - **an answer to prefixes that misrepresent the block,** a rank-local prefix before a
    high-rank suffix;
  - **an exact handoff:** switching only between explicit tokens, with no run pending,
    keeps the output position, counts and checkpoints coherent;
  - **E's per-token count,** which is not free, though it can stop once the decision is
    made.
- **A has the stronger experimental case.** It applies the same whole-block rule, with
  continuity with Gate 1, and only its implementation changes. Its margin against the 3%
  limit is small.
- **A is the smallest change to what Gate 1 measured,** with its margin set by memory
  bandwidth, not by instructions.
- **B moves the cost onto the pipeline's serial stage and changes a public struct.** Of the
  three, it is the one whose cost is hardest to account for.

None of this is authorised. Phase 2, N1's Gate B, the i9 and production integration stay
unauthorised.

## Option C: the read-only feasibility analysis (authorised, external report 43)

The work is code reading plus `theory/c_prefix.py` over existing data:
- **Wikidata:** 60 of the 233 census blocks (every fourth), with their real explicit-rank
  streams (bzip2's used-symbol mapping);
- **the controls:** `ab gen`'s random, records and text corpora at scale 1 (3, 3 and 4
  blocks), the same generator as CI's scale-4 corpora.

Nothing is implemented or benchmarked. Each item is marked **measured** (from the
token sequences) or **estimate**.

### 1. Explicit tokens per block (measured)

| corpus | blocks | E per block: median [min, max] | high-rank share: median [min, max] |
|---|--:|---|---|
| Wikidata (full blocks) | 60 | 55,320 [49,209, 136,178] | 1.53% [0.81%, 1.96%] |
| random | 3 | 892,897 [198,413, 893,000] | 75.8% [75.7%, 76.0%] |
| records | 3 | 897,079 [598,139, 897,151] | 90.6% [90.1%, 90.6%] |
| text | 4 | 141,573 [31,641, 141,577] | 5.1% [5.1%, 8.7%] |

Short blocks occur at a stream's end. CI's 431-block Wikidata corpus also has 15 small
blocks, about 3.6 KB of L in all (Gate 1's selection line).

### 2. Do prefixes represent the block? (measured)

The whole-block decision is 10·F ≤ E; the prefix decision applies the same test to the
first p explicit tokens.

| corpus | p = 1,024 | p = 4,096 | p = 16,384 | p = 65,536 |
|---|---|---|---|---|
| Wikidata: prefix agrees with whole block | 85% | **100%** | **100%** | 100% (13 blocks with E > p) |
| Wikidata: median and max \|prefix − rest\| share | 6.0, 11.7 points | 2.0, 3.7 | 0.29, 0.79 | 2.0, 7.0 |
| random | 100% | 100% | 100% | 100% |
| records | 100% | 100% | 100% | 100% |
| text | 100% | 100% | **75%** (one block) | 100% |

- **A probe of 1,024 tokens is too short for Wikidata.** 15% of its blocks would be sent
  to the shipped kernel wrongly. The prefix's share is noisy (median error 6 points), and
  the block start's rank mix differs.
- **From 4,096 tokens the decision matches the whole block** on every Wikidata, random and
  records block measured.
- **Text has a misclassification at 16,384.** One block, its last and short one
  (E = 31,641, share 8.7%), has a prefix on the other side of 10%. Its prefix decision
  sends it to the shipped kernel, which loses gain but causes no regression.

### 3. Does a block's rank mix change part-way? (measured)

Each block was cut into consecutive 16,384-token windows.

| corpus | the median block's window-share range | windows overall | blocks with any window across 10% from the first |
|---|---|---|---|
| Wikidata | 1.7 points | 0.02% to 5.4% | **0 of 60** |
| random | 1.5 points | 74.6% to 76.9% | 0 of 3 |
| records | 0.8 points | 89.7% to 91.1% | 0 of 3 |
| text | 5.3 points | 2.0% to 7.3% | 0 of 4 |

- **On this data, a one-way switch after the probe would never have met a block whose
  mix changed regime.**
- **That holds for these corpora only.** A stream that concatenates unlike data inside one
  block (a JSON block holding a long base64 blob, say) is not among them, so the one-way
  limitation is unmeasured for such inputs.

### 4. The probe's cost on fallback-heavy blocks (estimate, from measured E and Gate 1's δ)

- **The loss** is about δ · p/E over the shipped kernel's time, plus the switch's fixed
  cost. δ is the branch-free arm's slowdown on the controls: about 20% on Zen 5.
- **Full blocks at p = 4,096:** random 0.2 × 4,096/893,000 ≈ **0.09%**; records ≈ 0.09%.
  Random's short last block (E = 198,413) ≈ 0.41%.
- **At p = 16,384:** 0.37% and 1.65%.
- **The switch:** a 32-byte head store plus a 224-byte tail copy, tens of cycles once a
  block. That is negligible against a block's 5 ns/B × ~1 MB.
- **Smaller bzip2 levels.** At -1 a block holds at most 100k bytes, so E ≤ 100k on high-rank
  data. A fixed p = 16,384 then costs up to δ · 16% ≈ 3.3%, past the 3% limit. A fixed
  p = 4,096 costs about 0.8%. **The probe must be small (about 4k tokens) or scale with
  the block's size** (the stream header gives its level).

### 5. The handoff boundary (code reading)

- **The plain kernel (`mtf_avx2` / `p74::avx2::mtf`).** The clean point is just after an
  explicit token's `n += 1`:
  - `run = 0` and `weight = 1`, with no RUNA/RUNB digits pending;
  - `out[..n]` and `counts` are consistent;
  - the `syms` iterator sits after the token.

  The shipped kernel needs only its 256-byte list: the head's 32 bytes stored to
  `base[0..32]` and the tail's 224 copied to `base[32..256]`.

  **The obstacle:** the shipped kernel is a separate function with its own loop. A switch
  needs one function with both inner loops sharing `n`, `run`, `weight`, the iterator and
  the counts. That is a restructure, though not a new algorithm.
- **The checkpoint kernel.** The outer loop already stops every 16 KiB of output. Its
  natural switch point is the end of an outer iteration, after the trailing `byte!(s)`:
  - `run` is reset;
  - the pending-run split at the checkpoint has been done;
  - `next` and `checkpoints` are coherent.

  Probing there adds no branch to the inner loop. The probe is then 16 KiB of output, not
  a token count. On random that is about 16k tokens, which by §4 is too long for -1
  blocks.

### 6. The denominator E (code reading, estimate)

- **Counting E per token** is one add per explicit token, off the state chain but an
  issue slot. Against P74's roughly 15–20 instructions a token, that is a few percent of
  the loop's instructions, and possibly visible on Wikidata.
- **A free substitute: S, the symbols consumed so far.** It is the `syms` iterator's
  offset, a pointer difference, with no per-token work. S = E + (RUNA/RUNB digits) + (0 or
  1 for the end symbol), so S ≥ E.
  - The test 10·F ≤ S is more permissive than 10·F ≤ E: it sends P74 more blocks.
  - **On these corpora it decides the same way.** Random and records have almost no runs
    (S ≈ E, shares 76–91% against 10%). Wikidata's F is 1.5% of E and so a smaller share
    of S.
  - It is a different rule from the frozen one, and would need registering as such.
- **F** is counted only in the fallback branch, already the slow path. In the shipped
  kernel, after a switch, the same branch is its `memmove` path, so a counter there costs
  next to nothing either.

### 7. One-way against reversible switching (code reading; §3 measured)

- **P74 → shipped:** a store of 32 bytes and a copy of 224, as in §5.
- **Shipped → P74:** a load of 32 bytes from `base[0..32]` and a copy of 224 back to the
  tail. It is about as cheap, so reversibility isn't held back by the cost of moving the
  state.
- **The real cost of reversibility is monitoring.**
  - The shipped kernel must keep counting F (nearly free, in its `memmove` branch) and E
    or S (S is free) over a window.
  - It must check at probe points. The checkpoint kernel already has them; the plain
    kernel would need a counter or a token-count branch.
  - A policy must avoid oscillating, with hysteresis such as switching back only under 5%
    after leaving above 10%.
- **The data in §3 gives no case where reversibility would have helped.** It would earn
  its monitoring only on inputs whose rank mix changes inside a block, which the measured
  corpora don't contain.

### Summary for a future pre-registration (none is proposed or authorised)

- **Measured:**
  - a 4,096-token prefix decides like the whole block on all 70 blocks measured;
  - no block changed regime part-way.
- **Estimated:**
  - the probe costs about 0.1% on full high-rank blocks and about 0.8% on the shortest
    (-1-sized) blocks at p = 4,096;
  - the switch costs tens of cycles a block;
  - S is a free substitute for E that decides the same way on these corpora.
- **Unknown until measured:**
  - restructuring the shipped kernel into a resumable loop, and what that does to code
    layout and register allocation;
  - inputs that change regime inside a block;
  - whether the checkpoint kernel's natural 16 KiB probe or a token-count probe is
    better.
