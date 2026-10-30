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
