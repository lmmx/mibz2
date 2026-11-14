# D106 — Cells 20-byte slots: source argument passes; B5 HOLD

- Incoming checkpoint: [6881612485](https://github.com/lmmx/mibz2/pull/161#issuecomment-6881612485).
- Reviewed PR: [#391](https://github.com/lmmx/mibz2/pull/391), exact head `c915a95eb7288843ea25707bffc87c9d252f4bde`; base `2a12e39f69ef7c0f17be860138701b14df82b713`.
- Operating SPEC/RULES: `a2da49638ac54d7763de1c167bd054f67ffdc763`.
- Observed 2026-11-14, completed 17:47 UTC. Receipt: `9a26afd19bae255b62cfe773a6507f5530d85dce`.
- Decision: **G1 dispute complete B5 qualification; G2 scoped HOLD on integrating this unsafe layout change until the evidence below is supplied.** Source-level adversarial review finds no new memory-safety or correctness defect. This is an evidence HOLD, not a claim of demonstrated UB, a line-level STOP, or a prerequisite on unrelated routine work.

## Concrete blocker: the tests never start

[.github/workflows/test.yml:34](https://github.com/lmmx/mibz2/blob/c915a95eb7288843ea25707bffc87c9d252f4bde/.github/workflows/test.yml#L34) contains:

```yaml
run: cargo test --features slot24 --lib cells::
```

The final colon terminates a plain YAML scalar illegally. Independently reproduced with PyYAML: **mapping values are not allowed here, line 34 column 55**. The base workflow parses; quoting just this command parses the head workflow. No repository implementation or workflow was changed by this review.

GitHub independently corroborates failure: [run 43622250866](https://github.com/lmmx/mibz2/actions/runs/43622250866) is failed, with **zero jobs**. Thus this head's Test workflow supplies neither its Rust tests, its `slot24` comparison, nor its Cells fuzz smoke. Green wheel-build/other workflows do not replace them. Repair by quoting the command or using a block scalar, then retain results on the repaired exact head.

## Unsafe offsets: independent argument

I did not implement this candidate. I inspected the exact head/base diff, complete production Cells source, eager reference, selective comparison, decoder differential helpers, Miri workflow and trace accounting. This review reproduced source counts, layout arithmetic and the YAML defect locally. There is no cargo/rustc on this review runtime's PATH: **I did not execute native Rust, Miri, fuzz or mutants.** Source/test inspection is not an execution PASS.

Let S be 20 or 24, R the record count, B the cell budget, G the debug/Miri gap (64, otherwise 0). The safe entry bounds R ≤ n, B ≤ n, n < 2²⁴ before layout arithmetic.

| Obligation | Finding |
|---|---|
| Field width and adjacent slots | The default other-order field is exactly [s+16,s+20); it ends at s+S. Existing writes [s,s+8) and [s+8,s+16) remain disjoint. Helpers use unaligned raw integer accesses, never an aligned Rust reference; odd 20-byte strides and an unaligned arena/rules base introduce no alignment obligation. |
| Offset representability | allocation = 2S(R+2+B)+32B+4432+4G. At the safe entry's conservative maximum R=B=n=2²⁴−1, G=64: 1,879,052,848 for S20, 2,147,488,304 for S24. Both fit usize arithmetic on 32-bit and are below u32::MAX. Allocation itself remains fallible. The existing hard refusal precedes narrowing; debug assertions are corroboration, not the guard. |
| Initial slots | L ≤ R+2 initial letters occupy [tops,tops+LS), similarly bots. Each partner is in the other region, below allocation. The split at the primary row adds at most two letters. The macro fully writes each slot before it is read. |
| Fresh slots and promises | The cell counter checks B before each cut. Each cut advances at most one fresh pointer per side; capacity covers L+B slots. A promised slot initially carries only the predecessor; length, word and partner are installed when its cell is cut before it becomes a whole letter. This scheduling is unchanged. The shrink changes offsets, not induction order. |
| Aliasing/lifetime | Scalar raw loads copy values; no references to fields escape. The backing Vec is sized before base is acquired and does not reallocate during induction. Selected materialization logging copies into separate vectors. This delta introduces no new sharing or lifetime erasure. |
| FAR/PAD | FAR=12, PAD=4096 unchanged. The two real rd32 loads in ahead!, at s−12S+12 and s−12S+16, need bounds even though their eventual use is only a hint. Minimum first-read offsets are 3868 (S20), 3820 (S24), both inside initialized padding. Their ends precede the current valid slot. For bots they may read an earlier region/gap, still initialized bytes in the same allocation. Reused slack may be stale, but is initialized; loaded values feed only wrapping_add and non-dereferencing prefetch. This proof would not survive an uninitialized-buffer change. |
| Byte arena and rules | Their bases derive from cap, not literal 24. Both shift earlier by 8(R+2+B) when S shrinks. The per-cell byte/rule allowances and slack stay unchanged; rd64/wr64/copy16 are unaligned. Packed word offsets remain below allocation. |
| Trace and tests | Production cells.buf reporting uses the actual Vec length/capacity. Selected-slot snapshots use SLOT. The old 48R+80B+5000 test bound remains conservative, merely loose. No remaining production 8-byte other-field access outside the feature-gated helper was found. |

The field's old high 32 bits were zero on supported little-endian executions because every stored partner is below u32::MAX; the new load returns that same offset. Existing native-endian packed-field portability assumptions are unchanged, not newly certified here.

Counting actual call sites gives **13 production calls** (8 writes, 5 reads) and **10 reference calls**, not 18 across both files. This is a documentation count correction, not a safety blocker.

For a 900,000-row production block with R=89,999 and B=144,000, the conservative requested arena decreases from 15,844,480 to 13,972,472 bytes, saving 1,872,008; positions add up to 576,000 separately. These are requested-storage bounds, not RSS measurements or claims of an 11% cache-line improvement.

## Tests and mutation evidence: exact limits

The selective reference now shares rd_other/wr_other, so old/new state equality alone cannot catch a shared offset bug. The suite still compares both outputs with an independent LF oracle; the decoder differential helper additionally compares exact partial output/error identity against Segmented and successful output/error parity against libbzip2. The slot-link test reads bytes directly at +16. These are useful independent witnesses; their source does not prove this head executed them.

The Miri workflow explicitly discovers all six Cells witnesses plus the sequential-reader witness. At the inspected snapshot [job 132530336939](https://github.com/lmmx/mibz2/actions/runs/43622262431/job/132530336939), discovery/setup had passed but the execution step was **in progress**. This review did not retrieve a completed execution log. The PR reports local passes but links no raw local record. B7: partial is not PASS.

The registered Cells smoke is `target/fuzz/examples/fuzz 30 42 fuzz-failures/cells cells`. Its source covers forced and admitted routes with independent decoder checks and explicit reach counts. The invalid workflow prevents that job starting, and no replacement exact-head fuzz result was supplied. Enumerated corrupt-input unit cases are not a substitute for the registered fuzz smoke.

Mutation table:
- M1 wide write, M2 wide read, M3/M6 offset 12, M4 offset 17 target real overwrite/address classes. A reproducible SIGSEGV is a valid kill; it need not be an assertion.
- **M5 is reported as “hung for minutes and were stopped.”** That is a liveness failure observation, not an independently reproducible completed kill as presented. The table supplies no fixed timeout, failing case/log, mutant patch or restoration check. Do not silently promote it to a verified six-of-six result.
- No evidence shows a surviving mutant here, so I do **not** infer an unresolved B6 missed-target violation or declare the entire Cells line stopped.
- Small tests cannot exercise >u32 offsets; the safe-entry algebra above discharges that concern without demanding multi-gigabyte tests. Existing poisoned-buffer reuse and exact-fit/canary witnesses are relevant to stale padding and cross-region mistakes; they must actually run.

## Smallest closure

1. Correct the YAML and supply the repaired head's passing Rust/differential/slot24 tests and Cells fuzz-smoke result with counts/seed.
2. Supply completed logs for the six Cells Miri witnesses and relevant reader witness on that source, or exact source correspondence for completed local evidence. The full CI gate remains separate.
3. Archive the claimed offset mutants' changes, commands, terminal outcomes and restored baseline. For M5, provide a bounded reproducible timeout witness or a deterministic failure; record its result accurately beside the present claim.

No new statistical method, broad test programme, giant allocation test or performance experiment is required by this HOLD. A follow-up with those bounded records can close it without redoing the whole source review if the kernel remains byte-identical.

## Separate performance observation

[Bench comment 6881593849](https://github.com/lmmx/mibz2/pull/391#issuecomment-6881593849) reports Wikidata gains, but also **text 4t +2.2% wall, interval [+0.1%,+4.5%]**. The same comment's Wikidata resource assessment (“Better … no measure worse”) does not mean every primary corpus passed. Preserve this first-run primary flag under the registered policy; only its specified one primary-regression re-run may settle it. This review neither launches one nor qualifies the candidate for merge, and supplies no i9 scaling verdict. Scope remains B5; Coordinator retains integration responsibility.

## Record

State migration head and manifest digest verified before receipt. Inbox reconciled: only 6881612485 unfinished; routing audit since 6868538108 found no misroutes. REVIEW destination is #145. No other branch, implementation, merge, workload, workstation, infrastructure or automation setting was changed. Companion evidence JSON records snapshots, source SHA-256s and reproduced figures.
