# DRAFT: does `mtf::STEPS`' 4 KiB placement against `counts` cost time? A bounded CI diagnostic

**Status: DRAFT only.** Nothing here is built, run or registered.

**What it is not:**
- not a fourth #324 run;
- not a qualification of any production change;
- not an authorisation to time.

It needs the Researcher's PROCEED and the Guardian's admission (D082 and D086) before any
of it is built or run. It follows the Optimiser's REVIEW 6761484547 (report
`state/records/reviews/inbox-6761295924/report.md` at d7492ec3, §3).

## 1. The question, and why this design

**The question:** on master, at 1 thread on CI's `random` corpus, does moving `mtf::STEPS`
from a 4 KiB residue window that overlaps `counts` to one that doesn't, with everything else
held, change the decoder's elapsed time? If it does, by how much, against what #324's harm
would need?

**The pair in question.** #324's head raised `mtf_avx2`'s 4 KiB D←H candidates (`STEPS` row
loads after `counts` stores) from 13,829 to 56,154 per 890,000-byte random block: an excess
of 0.0476 a byte (`2026-11-10-run-front-fallback.md`, §"Corrected counts").

**What explaining the harm would take** (the Optimiser's arithmetic): explaining all of
`random`'s 1-thread harm would need about 0.51–0.56 ns a byte, which is 28–39 nominal
cycles per excess candidate. Nothing has measured such a cost.

**Why moving `STEPS` and not `counts`:**
- **Moving `counts`** (separate builds, or `Scratch` changes) also moves `counts` against
  the stack, where the corrected counts show far larger classes (S←H 16–21×). It moves
  caller code too.
- **Moving `STEPS` by exactly 2,048 bytes, in one binary, with one selected pointer:**
  - its residues modulo 1,024 and 64 are unchanged (2048 ≡ 0 mod 1024);
  - its residue modulo 4,096 moves to the other half-page;
  - `counts`, the frame, the stack and the hot code stay as they are.

## 2. The construction (a diagnostic branch, never merged)

**One binary, two placements** (`StepsBank`):
- `#[repr(C, align(4096))] static STEPS_BANK: StepsBank`, where
  `struct StepsBank { lo: [Step; 32], hi: [Step; 32] }` holds two identical const-initialised
  copies.
  - `lo` sits at residues [0, 2048) modulo 4096 and `hi` at [2048, 4096).
  - Static alignment to 4,096 is to be verified from the ELF symbol table, not assumed.
- **Both arms run the same instructions.** `mtf_avx2` (and `mtf_checkpoints_avx2`, D083)
  takes its table base from `steps_base()`: one relaxed load of a process-global
  `AtomicPtr<Step>`, read once per call outside the literal loop, in both arms.
  - The pointer is set once at process start, before any decode and outside the timed
    region, from `MIBZ2_DIAG_STEPS`, which must be `lo` or `hi` (equal length, so the
    environment's size and the stack's start are the same in both arms).
  - That is the diagnostic binary's only difference from master's MTF: a load where master
    has a rip-relative `lea`. Both arms share it, so it cancels in the contrast. It is
    disclosed, not claimed equal to master.

**`counts` at a fixed residue.**
- Where `counts` lands follows allocation history and the input path's length (0x060 or
  0x0a0 in our probes). So the diagnostic binary allocates `Scratch::counts` as a
  `#[repr(C, align(4096))]` page with `counts` at byte offset 0x100: residues [0x100, 0x500).
- That window lies wholly in `lo`'s half and is disjoint from `hi`'s, so:
  - **arm `lo` is aliased:** `counts` meets `STEPS` rows 4–19 modulo 4 KiB;
  - **arm `hi` is disjoint.**
- The page is the same in both arms, and identical code reaches it.

**Stop and redesign** if, from the listing, the seam changes `mtf_avx2`'s hot loop
between arms (it can't, being one function) or can't be hoisted out of the literal loop.

## 3. Untimed checks before any timing (all to pass, on the CI runner, from the exact binary)

1. **Listings.** One `mtf_avx2` and one `mtf_checkpoints_avx2` in the binary. `steps_base()`'s
   load sits outside the loop. The hot functions' code is identical whichever arm runs (one
   binary). Record the frame sizes.
2. **Native addresses** (logged at setup to stderr, outside the timed region, one process per
   arm):
   - the selected `STEPS` base;
   - `&counts`;
   - `STEPS_BANK`'s address.

   They must show:
   - `STEPS_BANK ≡ 0 (mod 4096)`;
   - the selected base equal to `lo` or `hi` as intended;
   - `&counts ≡ 0x100 (mod 4096)`;
   - the two arms' bases differing by exactly 2,048.
   
   A gdb probe as in `stackprobe324.gdb` records `mtf_avx2`'s frame base (`rsp`) per arm,
   to show the stack's distribution is the same in both (per-process randomisation, in five
   processes an arm).
3. **The corrected pair contrast (`agg3`)** on the first block of CI's `random`, per arm,
   under lackey:
   - **Valid only if** `mtf_avx2`'s 4 KiB D←H count for the named PC pairs (the `counts`
     store PC against `STEPS`' three row-load PCs) is at least 10× higher in `lo` than in
     `hi`, with `lo`'s rate at least the 0.0476 a byte excess #324 had.
   - Every other `agg3` class of `mtf_avx2`, `read_item`, `bwt::build` and `room_segments`
     (H←H, S←H, H←S, S←S, at 4 KiB and 1 KiB) within 2% between arms.
   - **The exception, D←S** (`STEPS` loads after stack stores): this class can move, since
     `STEPS` moves relative to the stack. It is reported per arm and disclosed as a
     confound. If it moves by more than the named pair's contrast, the intervention is not
     specific, and the run is invalid.
   - **No search:** the offsets (0x100; `lo` against `hi`) are fixed here, before any data.
     If they fail, the design is revised and re-reviewed. Others are not tried until one
     passes.
4. **Same outputs:** both arms decode every CI corpus to the same SHA-256.

**A failure of 1–4 is an invalid intervention, not a negative causal result.** No timing
follows it.

## 4. Confounds that can't be removed, disclosed

- **L1 set mapping.** A 32 KiB, 8-way L1 data cache is indexed by address bits 6–11, the
  same bits as the 4 KiB residue.
  - `lo` puts `STEPS`' 32 lines in sets 0–31, sharing sets 4–19 with `counts`' 16 lines;
    `hi` puts them in sets 32–63.
  - So any change to the 4 KiB overlap also changes L1 set co-location. With one `STEPS`
    line and one `counts` line a set, against 8 ways, conflict misses are unlikely, but
    other hot lines (`rows` on the stack, the `ll` and symbol streams) share sets too.
  - **A positive result means sensitivity to this 4 KiB placement, false dependences and
    set placement together.** It does not uniquely attribute false-dependence stalls.
  - CI's runners have exposed no PMU, so L1 misses can't be read. If a PMU is available
    on the runner, its counts are recorded; none are assumed.
- **`STEPS` against the stack.** That relation (D←S) changes too. It is measured (§3.3)
  and disclosed.
- **One host per run.** #324's harm was flagged on EPYC 7763 and 9V45. The runner's CPU is
  recorded. A result reads only for that host. No re-dispatch to reach an AMD runner.

## 5. Estimand, rounds and the prospective decision rule

**The estimand:** Δ, the paired difference aliased (`lo`) minus disjoint (`hi`) in
whole-process wall time per output byte (ns/B):
- `ab child mibz2 1 corpora/random.bz2` (8,000,000 output bytes);
- pinned to one CPU, as the bench's 1-thread rows are;
- reported with its ratio form, the geometric mean of the paired ratios.

**Rounds:**
- 40 paired rounds, the arms' order alternating ABBA by round, each a fresh process.
- The interval is the bench's: a 95% t-interval on the per-round log ratios, converted to
  ns/B at the rounds' median `hi` time.
- **Precision expected:** #324's 1-thread `random` rows had half-widths of ±0.8–1.2% at 10
  rounds. At 40 rounds, about ±0.6%: about ±0.1 ns/B at CI's ~16 ns/B. The Researcher sets
  the precision target, and the rounds if needed, before data.
- **One fixed sequence:** no retry, and no extension after seeing data.

**The required contribution, fixed now:**
`Δ_req = 0.51 ns/B × (c_diag / 0.0476)`.
- 0.51 is the lower of the Optimiser's two random-harm conversions.
- `c_diag` is the named pair's per-byte contrast (`lo` − `hi`) that §3.3 measures on this
  binary.
- If the contrast equals #324's excess, `Δ_req` = 0.51 ns/B.

**The decision rule:**
- **The upper 95% bound of Δ below `Δ_req`:** this pair alone can't explain all of #324's
  1-thread `random` harm on this host and in these conditions. Full attribution is rejected.
  A partial contribution stays possible.
- **The lower 95% bound of Δ above zero:** the decoder is sensitive to this placement
  (false dependence and L1 set placement together, §4). That is not attribution of #324's
  harm. A follow-up control or hardware evidence would have to separate the two.
- **Otherwise inconclusive:** a null result can't confirm or exclude a small contribution.
- **Not readable as a result:** a failed §3, or a run that doesn't complete as planned
  (setup failures re-run as the same run, CI-INCIDENTS class B).

## 6. Cost on CI, and scope

**One workflow dispatch on the diagnostic branch:**

| step | approximate time |
|---|---|
| build | about 3 minutes |
| §3.1–3.2 (listing and probes) | under 1 minute |
| §3.3 (two lackey traces of one block) | about 10 minutes |
| §3.4 (outputs) | under 1 minute |
| the timed 40 rounds × 2 arms | under 1 minute, at 0.08–0.15 s a decode plus process start |
| **total** | **under 20 minutes** |

**Out of scope:**
- `records` and other hosts. Only if this result is informative, as a separate
  pre-declared extension.
- Any production change, contract A or B, and any #324 successor.
- The stack-side classes (`counts` against `mtf_avx2`'s frame, `read_item`'s 1 KiB S←S),
  which the Optimiser ranks next if this pair can't carry the required budget.

**Twins (D083):** the seam changes `mtf_avx2` and `mtf_checkpoints_avx2` alike. The run
exercises only the plain 1-thread worker path (`mtf_avx2`), and claims nothing for the team
path.
