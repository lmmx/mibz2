# 2026-11-05: MTF on libbzip2's two-level list, its first 32 bytes in registers: merged

Registered before any CI timing of this branch. The decision rule may be amended only in a
commit made before the first bench comment on its PR, and each amendment says what changed
and why.

## The change

The move-to-front decode (`mtf_avx2`, `src/mtf.rs`) is one serial chain a symbol: each
symbol's move must finish before the next symbol's index can be read off the list.

- **On master,** the list is a 32-byte vector in memory. A move stores the blended 32 bytes,
  then a 1-byte front. The next symbol's 32-byte load spans both stores, so it cannot be
  forwarded. It waits for them to commit, and that store, commit and reload sits on the
  chain every symbol.
- **The branch keeps libbzip2's two-level list** (`mtfa`: 16 sub-lists of 16), vectorised:
  - **First level:** list bytes 0–31 are held in two xmm registers, `f0` and `f1`. A move of
    index nn < 32 is `f0' = pshufb(f0, a[nn]) | pshufb(f1, b[nn])` and
    `f1' = pblendvb(f1, palignr(f1, f0, 15), m[nn])`, from a 32-row static table (2 KB).
    There is no branch on nn and no load or store on the chain.
  - **Second level:** bytes 32–255 are 7 aligned 32-byte rows in memory. Each row is
    shifted in registers, taking the previous row's last byte (`f1`'s for row 0). For
    nn < 64 only row 0 moves.
- **Unchanged:** the outer loop, run handling (#170's two-store short runs), every check
  (`RunTooLong`, `BlockTooLong`), `mtf_scalar` and `mtf_checkpoints`.
- **One guard, added after verification:** an index is clamped at 255. A decoded symbol is
  below the alphabet, so this never binds. A hand-built `Symbols` (its fields are public)
  with a larger symbol would otherwise read past the 224-byte rows.

## Measured, untimed or local

- **Where the indices fall** (OBSERVED, `examples/mtf_census.rs` on research/mtf-census,
  the bench corpus's 431 blocks):
  - byte symbols at index 1: 14.5%;
  - 2–15: 70%;
  - 16–31: 13.7%;
  - 32–63: 1.3%;
  - 64 and above: 0.2%.

  So 98.5% of moves take the register path.
- **Correctness** (OBSERVED, an independent differential):
  - master's AVX2, master's scalar and the branch's kernels were compared on 2 × 240,556
    cases, about 2.09 billion symbols: random streams, every index class, runs up to the
    limit and past it, nInUse 1–256, streams empty, unterminated or with trailing junk;
  - zero mismatches in result, `ErrorKind`, `ll` or counts, including the partial state left
    on an error;
  - whole-file decodes of 8 corpora are identical.
- **The chain** (OBSERVED, listings `final-mtf.asm` and `base-mtf.asm`; llvm-mca): only `f0`
  and `f1` carry from one byte symbol below 32 to the next.
  - The chain takes 3.0 cycles (znver4) and 4.0 (skylake), against master's 11.0 on both.
  - `f0` and `f1` are spilled only around the long-run memset.
- **Cycles a symbol** (OBSERVED, both kernels in one binary, `verify-synth.txt`):
  - index 1–31: about 17.5 → 10;
  - index 64: 17.3 → 22.6, slower;
  - index 100: 20.3 → 23.6, slower;
  - index 255: 34.5 → 26.1.
- **Local stage timing** (a shared VM; indicative only):
  - MTF + RLE2: −12.8% [−14.7, −10.8] over 16 paired rounds, faster in 16 of 16.
  - The phases total showed no gain locally: +1.5% [−2.5, +5.6]. The stages whose code did
    not change moved 2–6% slower in the branch's binary, consistent with a code-layout
    artefact on that VM's Skylake (the JCC erratum). Only CI can say whether it carries over.

## The prediction, recorded before any run

- **MTF + RLE2 stage, 1 thread:** 8–15% faster on every CPU model.
- **`wikidata` 1-thread wall:** 1–3% faster. MTF is about 16–24% of a 1-thread decode.
- **4-thread wall:** within ±1.5%.
- **Far-index-heavy corpora** (`random`; `records` in part): the stage may be neutral or
  slower, since index ≥ 64 is slower per symbol.
- **The other stages** do not move: their code is untouched.

## The decision rule

- **Correctness (required):** CI green, and the same bytes on every corpus in every bench run.
- **Benefit (required):** in at least one run, `wikidata` 1-thread wall or CPU is faster
  beyond noise, or the bench's 1-thread MTF + RLE2 stage is faster beyond noise while the
  whole-decoder 1-thread point estimate is not slower. The same figure must not be flagged
  slower in any other run.
- **Replication (required):** at least two runs, on two different CPU models. One run is not
  enough at 1 thread: A/A merges show a run-to-run spread of about 0.7% there, beyond one
  run's own interval.
- **What refutes the mechanism:**
  - an MTF + RLE2 cut under 5% in every run;
  - any other stage moving beyond noise in both runs, the same way (a layout artefact,
    reported, and judged with the wall rows).
- **No harm (required):** a wall row flagged slower is a harm signal, settled by one more run
  on the same head. Flagged again, it is a harm and the rule fails. At most three runs, each
  run's CPU named.

## Result

Merged as #207 (squash bc899400, at the benched head e6dd0492), after a separate
cross-model check. The rule above is left as written: its replication clause was not met
within it.

### The three runs under this rule

All three drew an AMD EPYC 7763 (run 42082730010, attempts 1–3).

| attempt | `wikidata` 1t wall | 2t wall | 4t wall | MTF + RLE2 | `records` 1t | `random` 1t |
|---|--:|--:|--:|--:|--:|--:|
| 1 | −5.2% (−5.9, −4.5) | −3.9% | −4.2% (n.s.) | −23.5% | −26.4% | −13.3% |
| 2 | −5.2% (−6.2, −4.2) | −4.9% | −1.2% (−2.0, −0.3) | −23.5% | −27.0% | −13.2% |
| 3 | −4.8% (−5.8, −3.7) | −4.4% | −1.5% (n.s.) | −23.1% | −28.5% | −13.1% |

- **Correctness and benefit** passed, and no refuter was hit.
- **No harm:** no wall row was flagged slower.
- **Replication on a second CPU model** could not be met. The rule allowed three runs, and the runner's model can't be chosen.

### The cross-model check

- **Registered** in [a comment on #207](https://github.com/lmmx/mibz2/pull/207#issuecomment-6671156633)
  before any of its runs, and authorised by the maintainer.
- **What was measured:** the frozen head against the same base, by `workflow_dispatch`.
- **Choosing the run:** runs went one at a time, with only the runner's CPU read first. A
  7763 run would have been set aside unread. The first run on another model decided, with
  a cap of 5 dispatches.
- **The rule:** this rule's own clauses, applied to that run.

One dispatch was needed. Run 42121493512 drew an AMD EPYC 9V45 (Zen 5):

| Clause | Result |
|---|---|
| Correctness | the same bytes on all 7 corpora at 1, 2 and 4 threads |
| Benefit | `wikidata` 1t wall −2.1% (−3.8, −0.4), faster beyond noise; MTF + RLE2 −18.5% (−21.8, −15.1) |
| Refuter (MTF cut under 5%) | not hit |
| Harm | no wall row flagged slower; no other stage moved beyond noise |

Other rows on the 9V45 at 1 thread: `records` −24.7%, `random` −6.0%, and the stage total
−5.3%.

### What it shows

- **The mechanism holds on Zen 3 and Zen 5.** Keeping list bytes 0–31 in registers
  removes the store and reload from the serial chain. It cuts the MTF stage by about a
  fifth on both models.
- **Zen 5 gains less:** −18.5% against −23% on the stage, and −2.1% against −5% on the
  1-thread wall. This fits a faster store-to-load path on Zen 5 leaving less to remove,
  but no hardware counters were read, so it is not shown.
- **The prediction was low on Zen 3.** It said 8–15% on the stage and 1–3% on the wall.
- **The far-index corpora gained instead of slowing:** `records` −25% to −28%. The
  per-symbol cycle counts above put index ≥ 64 slower. The likely cause is that removing
  the store from every move outweighs that slowdown, but this was not measured.
- **Caveat:** at 1 thread, the A/A runs put the between-run spread at about 0.7%. Widened
  by that, the 9V45 run's `wikidata` 1-thread interval would touch zero. Its own interval
  excludes zero, which is what the rule judges, and the three 7763 runs are well clear of
  it.
- **Not covered:** Intel. No run drew the Xeon 8370C.
