# DRAFT: does `mtf::STEPS`' 4 KiB placement against `counts` cost time? A bounded CI diagnostic

**Status: DRAFT only, revision 3.** It takes in two REVISE reviews of revision 2 (41be4c27):
- the Guardian's **D090** (REVIEW on #145, comment 6763239132; text at `state/decisions.md`
  D090 and `state/evidence/D090-probes.json`, commit 1e7d39f3 on `team-state/guardian`);
- the Researcher's **REVIEW 6763280324** (#145; report
  `state/reports/steps-placement-delta-6763141779/report.md` and `probes.json`, commit
  e5409a3f on `team-state/researcher`).

The revision log (§8) maps each of their points to its change. Revision 2 took in the
Researcher's REVIEW 6763089509, item 1 (`state/reports/integration-priorities-6762860125/01-steps.md`
at 07b453d1).

D082 and D086 hold unchanged. Nothing here is built, run or registered.

**What it is not:**
- not a fourth #324 run;
- not a qualification of any production change, and not a qualified base for any successor
  (D086, §5.5);
- not an authorisation to build or time.

It needs the Researcher's PROCEED and the Guardian's admission (D082, D086, D090) before
any of it is built or run. It follows the Optimiser's REVIEW 6761484547 (report
`state/records/reviews/inbox-6761295924/report.md` at d7492ec3, §3).

## 1. The question, and why this design

**The question:** on master, at 1 thread on CI's `random` corpus, does moving `mtf::STEPS`
from a 4 KiB residue window that overlaps `counts` to one that doesn't, with everything else
held, change the decoder's measured decode time? If it does, is the net change at least a
reference budget scaled from #324's harm (§5.3)?

Only a net placement contrast is identifiable by this intervention. It is not a causal
measurement of the false-dependence mechanism, and no outcome attributes or excludes #324's
harm (§5.4).

**The pair in question.** #324's head raised `mtf_avx2`'s 4 KiB D←H candidates (`STEPS` row
loads after `counts` stores) from 13,829 to 56,154 per 890,000-byte random block: an excess
of 0.0476 a byte (`2026-11-10-run-front-fallback.md`, §"Corrected counts").

**The reference budget's source** (the Optimiser's arithmetic): explaining all of `random`'s
1-thread harm would need about 0.51–0.56 ns a byte, which is 28–39 nominal cycles per excess
candidate, if that harm were carried by these candidates at a common linear cost. Nothing has
measured such a cost. Candidates are trace counts, not outstanding stores or stalls.

**Why moving `STEPS` and not `counts`:**
- **Moving `counts`** (separate builds, or `Scratch` changes) also moves `counts` against
  the stack, where the corrected counts show far larger classes (S←H 16–21×). It moves
  caller code too.
- **Moving `STEPS` by exactly 2,048 bytes, in one binary, with one selected pointer:**
  - its residues modulo 1,024 and 64 are unchanged (2048 ≡ 0 mod 1024);
  - its residue modulo 4,096 moves to the other half-page;
  - `counts`, the frame and the hot code stay as they are. The stack's placement is
    randomised per process in both arms; it is not held equal, only not moved by design
    (§3.2, §4).

## 2. The construction (a diagnostic branch, never merged)

The diagnostic branch is cut from one master commit, recorded in the stage-1 manifest (§6).

**One binary, two placements** (`StepsBank`):
- `#[repr(C, align(4096))] static STEPS_BANK: StepsBank`, where
  `struct StepsBank { lo: [Step; 32], hi: [Step; 32] }` holds two identical const-initialised
  copies.
  - `lo` sits at residues [0, 2048) modulo 4096 and `hi` at [2048, 4096).
  - Compile-time assertions: `size_of::<Step>() == 64`, `align_of::<StepsBank>() == 4096`,
    `size_of::<StepsBank>() == 4096`, `offset_of!(StepsBank, lo) == 0`,
    `offset_of!(StepsBank, hi) == 2048`. The three row loads are at byte offsets 0, 16 and 32
    of a `Step`, 16 bytes each, as on master (D086 §3); the listing confirms it (§3.1).
  - The symbol's alignment is read from the ELF symbol table (`st_value mod 4096 == 0`, and
    its section's alignment ≥ 4096), not assumed. The runtime check is §3.2.
- **Both arms run the same instructions.** `mtf_avx2` (and `mtf_checkpoints_avx2`, D083)
  takes its table base from `steps_base()`: one relaxed load of a process-global
  `AtomicPtr<Step>`, read once per call outside the literal loop, in both arms.
  - The pointer is set once per process from `MIBZ2_DIAG_STEPS`, which must be `lo` or `hi`
    (any other value aborts), at the start of `ab`'s `child()`, before the timing boundary
    (§5.1) opens. The two values have equal length; that equalises the environment's size,
    not the stack's placement, frames or microarchitectural state.
  - That load, where master has a rip-relative `lea`, is the binary's only difference from
    master's MTF. It is shared code in both arms, which controls for it as an instruction
    difference. It is not shown to cancel in time: its cost may interact with placement.
    It is disclosed, not claimed equal to master.

**`counts` at a fixed residue.**
- Where `counts` lands follows allocation history and the input path's length (0x060 or
  0x0a0 in our probes). So the diagnostic binary allocates `Scratch::counts` inside a
  `#[repr(C, align(4096))]` page struct with `counts` at byte offset 0x100: residues
  [0x100, 0x500), i.e. [256, 1280).
  - Compile-time assertions: `align_of` of the page struct is 4096, and `offset_of!` of
    `counts` within it is 0x100. The outer allocation's alignment alone does not prove the
    field's offset; both are checked, and the address is checked at run time (§3.2).
- Full-extent geometry (D090 §1, reproduced in its probes): `lo` overlaps `counts` modulo
  4096 across 1,024 bytes; `hi` overlaps it across 0 bytes. So:
  - **arm `lo` is aliased:** `counts` meets `STEPS` rows 4–19 modulo 4 KiB;
  - **arm `hi` is disjoint.**
  These are geometry facts, not a guarantee of the dynamic pair exposure, which §3.3
  measures.
- The page is the same in both arms, and identical code reaches it.

**Stop and redesign** if, from the listing, the seam changes `mtf_avx2`'s hot loop
between arms (it can't, being one function) or can't be hoisted out of the literal loop.

## 3. Untimed checks: all must pass before any timing

Stage 1 (§6) runs these on its runner, from the one built binary. §6.2 says which of them
stage 2 revalidates on its own runner before timing.

1. **Listings.** One `mtf_avx2` and one `mtf_checkpoints_avx2` in the binary. `steps_base()`'s
   load sits outside the loop. Record the frame sizes. From the listing, record as
   (symbol, offset from the symbol, access width):
   - the `counts` store instruction(s) in `mtf_avx2` (expected width 4 bytes, `u32`);
   - the three `STEPS` row-load instructions (expected width 16 bytes each, at `Step`
     offsets 0, 16, 32).

   These are **the named PCs**, frozen in the stage-1 manifest before any trace runs. Under
   lackey and natively they are located as the symbol's load address plus the frozen offset,
   so ASLR relocation does not change their identity. If the listing differs from the
   expected widths or offsets, the design stops for re-review (no trace).
2. **Native addresses**, each process logging to stderr at setup, before the timing boundary
   opens (§5.1). In **every process** of either arm:
   - `STEPS_BANK ≡ 0 (mod 4096)`;
   - `selected − STEPS_BANK` equals **0 in arm `lo`** and **2048 in arm `hi`**, computed
     within that process;
   - `&counts ≡ 0x100 (mod 4096)`.

   Absolute selected addresses are never compared across processes: under ASLR two fresh
   processes' bases need not differ by 2,048 (the Researcher's probe: 2,099,200 in one
   example).

   **Stack geometry, a smoke check only.** A gdb probe as in `stackprobe324.gdb` records
   `mtf_avx2`'s frame base (`rsp`) residue modulo 4096 in five processes an arm. Five samples
   cannot show the two arms' stack distributions are equal, and are not read as doing so.
   Each timed process also logs, before the boundary, the residue modulo 4096 of a local in
   `child()`'s frame (a cheap proxy, not `mtf_avx2`'s frame). These are recorded and
   reported per arm; they do not gate. The stack's relation to `STEPS` stays a disclosed
   confound (§4).
3. **The corrected pair contrast (`agg3`)** per arm, under lackey, over **the whole timed
   file:** `ab child mibz2 1 corpora/random.bz2`, all of CI's `random` (9 blocks, 8,000,000
   output bytes), one lackey process an arm.
   - **The tool:** `docs/journal/2026-11-10-run-front-fallback/agg3/agg3.c`, SHA-256
     `1ac18c5cc33def89ebb890cc993ae5cada81e833f3f4e9bffd64ea98521f9ca0`, with its tests
     `agg3/test_agg3.sh` (SHA-256 `bb2aa4aacdfcc74d5d0bcdd81b9bb627fa69a6a58ebfa492f39dc39ba034971f`)
     passing on the stage-1 runner before any trace is classified. Regions: S (stack,
     ≥ 0x1000000000), D (the static image, where `STEPS_BANK` lies), H (everything else,
     where `counts`' page lies). Moduli 4096 and 1024.
   - **The named counts:** `N_lo`, `N_hi` = in that arm, the sum over the named
     (store PC, load PC) pairs of `agg3`'s pair count for (`mtf_avx2`, load region D, store
     region H, modulus 4096). This is the definition #324's corrected counts used.
     `c_diag = (N_lo − N_hi) / 8,000,000`, in candidates per output byte.
   - **Exposure predicates** (integers; no rounding):
     - **E1:** `N_lo ≥ 10 · N_hi`. With `N_hi = 0` it reduces to `N_lo ≥ 0`, so E2 carries
       the nonzero requirement.
     - **E2:** `c_diag ≥ 0.0476`, the first-block excess #324's head had. Then
       `Δ_req ≥ 0.51 ns/B` (§5.3).
   - **The held classes, 2% each.** For each function F in {`mtf_avx2`, `read_item`,
     `bwt::build`, `room_segments`}, each modulus m in {4096, 1024}, each load region l and
     store region s in {S, D, H} (9 classes), and F's load count per load region:
     with x, y the two arms' counts, **pass iff |x − y| ≤ 0.02 · max(x, y)**. Both zero
     passes; exactly one zero fails. **Exempt:** `mtf_avx2`'s D←H at 4096 (the intervention,
     gated by E1–E2 and S1) and `mtf_avx2`'s D←S at 4096 and at 1024 (gated by S1). Every
     other class is held, including `mtf_avx2`'s D←H at 1024 (`STEPS`' residues modulo 1024
     do not move). That is 4 × 2 × 9 = 72 classes less 3 exempt, 69 class comparisons, plus
     12 load-count comparisons; every one is reported.
   - **S1, specificity, at named moduli.** With `C = N_lo − N_hi` (the named contrast at
     4096), each of the following absolute count differences, on the same whole file,
     compared **separately, not summed**, must be ≤ C:
     - `mtf_avx2` D←S at 4096: |`DS4_lo` − `DS4_hi`|;
     - `mtf_avx2` D←S at 1024: |`DS1_lo` − `DS1_hi`|;
     - `mtf_avx2` D←H at 4096 outside the named pairs: |(D←H4_lo − N_lo) − (D←H4_hi − N_hi)|.

     Otherwise the intervention is not specific, and the run is invalid. Passing S1 bounds
     counts, not latency: a smaller count change can carry a larger per-event cost, and
     these gates do not bound opposing timing effects (§4, §5.4).
   - **Stack classes do not transfer.** Lackey's stack is one environment's (valgrind's
     deterministic layout), not the native per-process randomisation of either stage. The
     S-region counts are a check on this trace, not a certificate for any native process.
   - **No search:** the offsets (0x100; `lo` against `hi`) are fixed here, before any data.
     If they fail, the design is revised and re-reviewed. Others are not tried until one
     passes.
4. **Same outputs:** both arms decode every CI corpus to the same output byte count and
   SHA-256, each equal to master's reference for that corpus; for `random`, 8,000,000 bytes.
   All hashes are recorded in the manifest.

**A failure of 1–4 is an invalid intervention, not a negative causal result.** No timing
follows it.

## 4. Confounds that can't be removed, disclosed

- **L1 set mapping.** A 32 KiB, 8-way L1 data cache is indexed by address bits 6–11, the
  same bits as the 4 KiB residue.
  - `lo` puts `STEPS`' 32 lines in sets 0–31, sharing sets 4–19 with `counts`' 16 lines;
    `hi` puts them in sets 32–63.
  - So any change to the 4 KiB overlap also changes L1 set co-location. Other hot lines
    (`rows` on the stack, the `ll` and symbol streams) share sets too.
  - **A positive result means net sensitivity to this placement, false dependences and set
    placement together**, with no quantitative bound on either's share or sign.
  - CI's runners have exposed no PMU, so L1 misses can't be read. If a PMU is available
    on the timing runner, its counts are recorded; none are assumed.
- **`STEPS` against the stack.** That relation (D←S) changes too. Its counts are measured
  (§3.3) and disclosed; their cost is not.
- **The pointer load.** Shared code in both arms; its interaction with placement is not
  shown to be nil (§2).
- **Hosts.** #324's harm was flagged on EPYC 7763 and 9V45. Both stages' CPUs are recorded.
  A result reads only for the timing host. No re-dispatch to reach a particular CPU.

## 5. Estimand, rounds and the prospective decision rule

### 5.1 The timing boundary and the estimand

**The timing boundary, used throughout:** the `wall` field of the `ab-child/2` record
printed by `examples/ab.rs`'s `child()` in its 1-thread in-process branch
(`None if threads == 1`): from `let t0 = Instant::now()` immediately before
`decode(arm, threads, path, &mut h, ...)` to `t0.elapsed()` immediately after it returns.
Process start, argument parsing, the `MIBZ2_DIAG_STEPS` pointer set-up, the stderr address
log (§3.2), hashing set-up and the record's output all fall outside it. "Before the
boundary" anywhere in this draft means before `t0`. `ab.rs`'s blob hash at the diagnostic
branch is recorded in the manifest. `wall` is printed to 4 decimal places of a second, a
resolution of 0.0125 ns/B on 8,000,000 bytes, disclosed and not corrected for.

**The estimand:** Δ, the additive difference aliased (`lo`) minus disjoint (`hi`) in `wall`
per output byte (ns/B):
- each slot is one fresh process, `ab child mibz2 1 corpora/random.bz2` (8,000,000 output
  bytes), pinned to CPU 1 with `taskset -c 1` (the bench's default 1-thread CPU), with
  `MIBZ2_DIAG_STEPS` set to the slot's arm;
- its `wall` × 10⁹ / 8,000,000.

### 5.2 The design and the analysis

**80 slots in 20 blocks of 4.** Each block is ABBA (`lo hi hi lo`) or BAAB (`hi lo lo hi`),
10 of each. **The order, frozen here** (A = `lo`, B = `hi`), blocks 1–20 in run order:

```
ABBA BAAB ABBA BAAB BAAB ABBA ABBA ABBA BAAB BAAB
BAAB ABBA BAAB ABBA ABBA BAAB BAAB BAAB ABBA ABBA
```

This listing is authoritative. It was generated by CPython 3.13.16's
`random.Random(20261110).shuffle(b)` on `b = ['ABBA']*10 + ['BAAB']*10`, recorded for
provenance only; the stage-2 supervisor reads the listing, not a generator.

**The block statistic:** `d_b` = mean of block b's two `lo` slots − mean of its two `hi`
slots, in ns/B. The weights on the block's four slots are (½, −½, −½, ½) for ABBA (negated
for BAAB).
- They cancel a drift linear in **slot index** exactly: their dot products with (1,1,1,1)
  and (1,2,3,4) are 0.
- A drift linear in **elapsed time** cancels only if the two arms' mean start times in the
  block agree, i.e. with symmetric slot times. Fresh processes and unequal durations need
  not give that: at start times (0, 1, 2, 4) a unit slope leaves 0.5 (D090's probe), at
  (0, 1, 3, 6) it leaves 1 (the Researcher's probe).
- So the design relies on the slot-index approximation. Every slot's start and end
  (CLOCK_MONOTONIC, from the supervisor) is recorded, and each block's timing asymmetry
  `a_b = ((s₁ + s₄) − (s₂ + s₃)) / 2` in seconds is reported. It is descriptive only: it
  changes no estimate, deletes no slot and triggers no rerun.

**The analysis:** Δ̂ = the mean of the 20 `d_b`; s = their sample standard deviation;
the 95% interval [L, U] = Δ̂ ± t₀.₉₇₅,₁₉ · s / √20 with t₀.₉₇₅,₁₉ = 2.0930240544;
half-width h = (U − L) / 2. Computed in IEEE double from the 80 `wall` values as printed,
with no intermediate rounding, in ns/B directly (no log ratio, no scaling).

**The model this interval is conditional on, stated:**
- the 20 block contrasts are **independent and identically distributed**, and **normal** for
  exact t coverage, or **approximately normal** for the interval to be a reasonable
  approximation;
- drift within a block is linear in slot index;
- slots within a block aren't treated as independent.

Independence and exchangeability alone are not enough. The Researcher's witness: iid block
contrasts from 0.99 · Uniform[−0.001, 0.001] + 0.01 · (point mass at 10 ns/B) have mean
0.10, yet with probability 0.99²⁰ = 0.8179 all 20 come from the small component and U is at
most 0.00148, missing 0.10. Nor does a balanced randomised order guarantee anything about
this realised sequence: arbitrary dependence, order-correlated drift or carryover can bias
Δ̂ or the interval. The reading is conditional on the model; it is not a general 95%
guarantee. All 20 block contrasts are reported in chronological order with their slots'
timestamps.

**No trimming, no reruns:** all 80 slots enter the analysis. No outlier deletion, no slot
replacement, no extension, no added blocks, and no rerun for precision or for any result.

**The precision target, fixed now:** **h ≤ 0.10 ns/B** (equivalently s ≤ 0.2136686 ns/B).
If h > 0.10, the result is INCONCLUSIVE (precision not met), whatever Δ̂ is.

**The ratio form** (the geometric mean of the per-block ratios of mean `lo` to mean `hi`
`wall`) is reported beside Δ̂ for reference. No predicate uses it.

### 5.3 The reference budget

`Δ_req = 0.51 ns/B × (c_diag / 0.0476)`, computed from stage 1's counts and written into the
manifest before stage 2 is dispatched.
- 0.51 is the lower of the Optimiser's two random-harm conversions.
- `c_diag` is the named pairs' whole-file per-byte contrast on this binary (§3.3).
- 0.0476 is #324's first-block excess on #324's head.
- E2 gives `Δ_req ≥ 0.51`.

**The transport assumption, stated beside every comparison with Δ_req:** Δ_req is a
**reference scaled under an assumed common linear marginal cost per counted candidate**,
carried from #324's head (and its first block's count) to this binary and host. It is not
a measured stall price. Whole-file exposure fixes this diagnostic's own denominator; it does
not justify the transport. Candidates are not outstanding stores or stalls.

### 5.4 The frozen predicates and what they mean

Let L, U, h be as in §5.2 and Δ_req as in §5.3. Comparisons are on the unrounded doubles.

| name | predicate | meaning when true |
|---|---|---|
| **V** valid | every §3 predicate passed in stage 1; every §6.2 revalidation passed; all 80 slots completed, each with `random`'s reference hash and byte count, and each with the §3.2 in-process address predicates passing; no failure after slot 1 started | the run is readable |
| **P** precise | h ≤ 0.10 (equality passes) | the precision target is met |
| **S+** | L > 0 (L = 0 is not positive) | positive net placement sensitivity, conditional on the interval model |
| **B−** | U < Δ_req (U = Δ_req is not below) | the net diagnostic placement contrast is below the scaled reference budget on this host, conditional on the interval model and the transport assumption |
| **B≥** | U ≥ Δ_req | the reference budget is not excluded |
| **B+** | L ≥ Δ_req | the interval lies at or above the reference budget |

S+ and B− are independent and **both can hold** (D090: L = 0.20, U = 0.40, Δ_req = 0.51; the
Researcher: a precise interval [0.04, 0.06]). Each is reported as true or false whenever V
and P hold.

**What none of them means.**
- **B− does not rule out the pair's sufficiency for #324,** nor show that this pair can't
  explain #324's harm. A net contrast can hide a large pair component: the Researcher's
  example, a pair component of +0.60 ns/B and other placement effects of −0.55, gives a net
  of 0.05; twenty block contrasts alternating 0.04 and 0.06 give h = 0.0048 and
  U = 0.0548 < 0.51, while the pair component alone exceeds 0.51. (D090's: +1.0 and −1.0,
  net 0.) The §3.3 count gates do not bound the confounds' time.
- **S+ is not stall attribution** and not attribution of #324's harm (§4).
- **B≥ is not demonstrated sufficiency.** B+ is the only predicate that says the interval
  supports at least the reference size, and even it is a net contrast under the transport
  assumption, not a causal verdict.
- **This diagnostic is not #324.** Its binary differs from #324's head and from master (the
  table base is a load, there are two `STEPS` copies, `counts` sits on a pinned page), and
  #324's head also moved `counts` against the stack, which this design holds fixed.

**Outcomes, in precedence order** (the first that applies):

| # | condition | reported as |
|---|---|---|
| 1 | not V | **INVALID.** No predicate is evaluated; nothing is read from any timing. |
| 2 | V, not P | **INCONCLUSIVE (precision not met).** Δ̂, L, U reported descriptively; no predicate is read. |
| 3 | V, P | S+ and B− (and B≥, B+) each reported as true or false, with the meanings above. |

An INVALID or imprecise run can't qualify any branch below, whatever its point estimate.

### 5.5 What the team does next (a resource-allocation rule, not a finding)

Fixed before data; it replaces the action table of LOG 6763024788 as corrected by
LOG 6763118519, and its phrase "Δ covering Δ_req" with the predicates above.

| outcome | next step |
|---|---|
| 1 or 2 | Retained as is; no retry, no rerun. Next-ranked mechanism. |
| 3 with B− (whether or not S+) | **Takes precedence.** Next-ranked mechanism in Pathfinder's ranking, by its cheapest discriminating check. No contract on this pair alone. This is a decision on where to spend effort, not an elimination of this mechanism. |
| 3 with S+ and B≥ (so not B−) | "Positive net sensitivity, reference budget not excluded" (with B+ noted if it holds). The pair stays a candidate. The permitted next step is **D086's first experiment only**: register contract A as an independent master-side change and qualify it against its pinned master baseline, judging its full costs under the standing CI, no-harm and applicable B5 obligations. |
| 3 with neither S+ nor B− | Unresolved. Next-ranked mechanism. |

**D086's order is preserved.**
- **This diagnostic is not a qualified base,** and a positive diagnostic is not a
  qualification of anything. It counts as no run of any registered sequence.
- **Only after** contract A is qualified on master may a materially changed fallback
  successor be registered **against that qualified contract base**, under D082's one
  registered sequence: the unchanged benefit and no-harm grid, two planned runs and a narrowly
  conditional third, stop by three, no retry to pass, the original failure disclosed.
- Neither the contract's qualification nor the successor's grid establishes causal
  attribution of #324's old regression. #324's failure, D086's failed restoration screen and
  the unresolved stack classes stand.

## 6. Execution: one experiment, two dispatches

Both stages, their dependencies, deadlines, budgets and failure policy are frozen together
with this registration, before the first dispatch. Stage 2 is the second half of one
experiment, never a fresh attempt after seeing stage 1.

### 6.1 Stage 1 (untimed)

Build the diagnostic binary once; publish it as a workflow artifact with its SHA-256. Run
§3.1–§3.4 on that exact binary. Write **the manifest**, uploaded as an artifact, holding:
- the master commit, the diagnostic branch commit, toolchain version, build command and
  flags, and the binary's SHA-256;
- the input corpora's SHA-256 and byte sizes, and every output's byte count and SHA-256 per
  arm;
- `agg3.c`'s and `test_agg3.sh`'s SHA-256 and the test output; valgrind's version; the gdb
  probe's SHA-256;
- the named PCs (symbol, offset, width), the full lackey trace inventory (file names, line
  counts, SHA-256), every `agg3` class count per arm, E1, E2, the 69 + 12 held comparisons,
  S1, `c_diag` and Δ_req;
- the §3.2 address logs and the stack smoke samples; stage 1's CPU model, kernel and runner
  image;
- every predicate's pass or fail, and the frozen block order (§5.2).

Raw lackey traces are not kept (too large); their inventory and digests are.

If any §3 predicate fails, the experiment ends INVALID at stage 1. Stage 2 is not dispatched.

### 6.2 Stage 2 (timed), gated on stage 1

Dispatched only after stage 1 completes with every predicate passing, never concurrently
with stage 1 or any other heavy job of ours (E4). Within one job, in this fixed order:
1. Download the stage-1 binary artifact and manifest. **No rebuild.** Verify the binary's,
   the corpora's and the manifest's SHA-256 against the manifest.
2. Record this runner's CPU model, kernel, runner image and PMU availability. The two stages'
   runners need not be the same machine; both are recorded.
3. **Revalidate on this runner** the conditions that do not transfer by hash alone:
   - §3.2's in-process address predicates, one process per arm;
   - §3.4's outputs: both arms, every CI corpus, byte count and SHA-256 equal to the manifest,
     in a fixed order (corpora alphabetically, `lo` then `hi` for each);
   - the stack smoke samples (recorded, not gating).

   What transfers by the binary's and input's hashes: the listing and named PCs (§3.1), and
   the D/H-region trace predicates (E1, E2, the held D/H classes, S1's D←H term), which are
   fixed by the binary, the input path and the byte residues that the revalidation re-checks
   natively. What does not transfer: anything stack-dependent. S-region classes and D←S were
   measured under lackey's own stack and certify no native process on either runner (§3.3).

   **A failed revalidation ends the experiment INVALID.** No re-dispatch to another runner;
   no host search.
4. Run the 80 slots in the frozen order, nothing else in between: no warmup slots, no
   unregistered observations. The revalidation decodes in step 3 are the only work before
   slot 1, and they are the same for both arms.

### 6.3 Failures, deadlines and budget

- **Before any timed slot starts** (stage 1 at any point, or stage 2 before slot 1), an
  infrastructure failure before the project ran (CI-INCIDENTS class B) may re-run that stage
  with the same commit, the same artifacts and the same order, paced under CI-INCIDENTS: at
  most **2 re-runs per stage**. A re-run takes whatever runner it is given and must pass its
  own revalidation there. A re-run is never used to select a geometry or a host. A check
  that fails (class A) is not re-run: the experiment ends INVALID.
- **Once slot 1 has started,** nothing restarts. Any failure, partial run, invalid slot or
  missed precision is retained as is and ends the sequence (outcome 1 or 2).
- **Deadline:** stage 2 is dispatched within 7 days of stage 1's manifest. Otherwise the
  experiment ends incomplete, retained, with no timing.
- **Job limits** (the workflows' `timeout-minutes`): stage 1, 180 minutes; stage 2,
  30 minutes. A timeout before slot 1 counts as a class-B failure under the rule above; a
  timeout after slot 1 ends the sequence.

**Estimated cost:**

| stage | step | approximate time |
|---|---|---|
| 1 | build | about 3 minutes |
| 1 | §3.1–3.2 (listing, addresses, smoke samples) | under 1 minute |
| 1 | §3.3 (two lackey traces of the whole 8 MB file, `agg3` and its tests) | about 60–90 minutes (one block took 2–5 minutes a trace on the shared machine; 9 blocks an arm) |
| 1 | §3.4 (outputs) | under 1 minute |
| 2 | artifacts, hash checks, revalidation | about 2 minutes |
| 2 | the timed 80 slots | under 1 minute, at 0.08–0.15 s a decode plus process start |
| | **total** | **about 1.5–2 hours of runner time, nearly all of it stage 1's lackey traces** |

## 7. Scope

**Out of scope:**
- `records` and other hosts. Only if this result is informative, as a separate
  pre-declared extension.
- Any production change, contract A or B, and any #324 successor, each of which keeps its own
  gates (§5.5).
- The stack-side classes (`counts` against `mtf_avx2`'s frame, `read_item`'s 1 KiB S←S),
  which the Optimiser ranks next.

**Twins (D083):** the seam changes `mtf_avx2` and `mtf_checkpoints_avx2` alike. The run
exercises only the plain 1-thread worker path (`mtf_avx2`), and claims nothing for the team
path.

## 8. Revision log

**Revision 3** (this), against revision 2 (41be4c27). Guardian D090 (REVIEW 6763239132) and
Researcher REVIEW 6763280324 (report at e5409a3f).

| review point | change |
|---|---|
| D090 §1 / Researcher 1: U < Δ_req does not rule out the pair's sufficiency | §5.4: B− now means only that the net diagnostic contrast is below the scaled reference budget, conditional on the interval model and the transport assumption. Removed "this pair alone can't explain…", "full attribution is rejected" and "a rejection of the pair's sufficiency". Both reviews' cancellation counterexamples added. §1 reworded to match. |
| D090 §1 / Researcher 1: Δ_req's transport assumption | §5.3: stated as a reference under an assumed common linear marginal cost per candidate, carried from #324's head; whole-file exposure fixes only the diagnostic's own denominator. §1 says the same of the Optimiser's figure. |
| D090 §1 / Researcher 3: "it cancels" | §2, §4: the shared pointer load is controlled as an instruction difference, not shown to cancel in time. |
| D090 §2 / Researcher 1: "Δ covering Δ_req" | §5.4: frozen predicates V, P, S+, B−, B≥, B+ with equality cases and a precedence order (INVALID, then precision, then the predicates). §5.5: the allocation rule uses B≥ ("budget not excluded"), not called demonstrated; B+ alone says the interval supports the required size. S+ with B− takes the next-mechanism branch. INVALID or h > 0.10 qualifies no branch. Findings (§5.4) are kept apart from the allocation rule (§5.5). |
| D090 §2 / Researcher 2: t19's assumptions | §5.2: iid, normal (exact) or approximately normal (approximate) block contrasts, stated; exchangeability alone is not enough (the Researcher's iid mixture witness). The unconditional "a trend… doesn't bias Δ̂" is deleted. |
| D090 §2 / Researcher 2: ABBA and drift | §5.2: weights (½, −½, −½, ½); exact cancellation in slot index only; elapsed-time drift needs symmetric slot times, with D090's (0,1,2,4) → 0.5 and the Researcher's (0,1,3,6) → 1. Slot timestamps and per-block asymmetry reported, descriptive only. No trimming, no precision reruns. |
| D090 §3 / Researcher 3: ASLR | §3.2: `selected − STEPS_BANK` ∈ {0, 2048} within every process, plus the residues; no cross-process comparison of absolute bases. Compile-time size, stride, offset and alignment assertions (§2); ELF alignment read from the symbol table. |
| D090 §3 / Researcher 3: stack samples | §3.2: five gdb samples an arm are a smoke check only; the claim of equal distributions is dropped. Each timed process logs a stack residue before the boundary, recorded, not gating. Equal environment length no longer claimed to equalise the stack (§1, §2). |
| D090 §3 / Researcher 3: 10×, 2%, D←S | §3.3: E1 `N_lo ≥ 10·N_hi` (N_hi = 0 included), E2 `c_diag ≥ 0.0476`; the 2% rule `|x − y| ≤ 0.02·max(x, y)`, both zero passing, one zero failing; every held class named (69 classes and 12 load counts); S1 compares absolute D←S differences at 4096 and at 1024 separately, and D←H outside the named pairs, each against the named contrast. Counts bound counts, not latency. |
| D090 §3: PCs, inventories, identities | §3.1: named PCs frozen as (symbol, offset, width) before tracing, relocation-independent. §3.3: `agg3.c` and its tests by SHA-256. §6.1: the manifest's full contents. |
| D090 §4 / Researcher 3: order, not seed | §5.2: the full 20-block order frozen as a listing; the generator recorded for provenance only. |
| D090 §4: two dispatches | §6: stage 1 untimed, stage 2 timed and gated on it; one binary artifact, no rebuild; manifests of binary, inputs and tools; fixed order, deadlines, job limits and retry budget frozen together. |
| D090 §4: the later runner does not inherit validity | §6.2: stage 2 revalidates the native address predicates and outputs on its own runner, in the same job as the timing; says which trace predicates transfer by hash and residue, and that stack-dependent ones do not. A failed revalidation is INVALID: no re-dispatch, no host search. |
| D090 §4: retry cutoff | §6.3: class-B re-runs only before slot 1 starts, at most 2 a stage, same artifacts; after slot 1, nothing restarts. The broader "re-runs as the same run" wording removed from §5. |
| D090 §5: D086 | §5.5: a positive diagnostic permits only D086's first experiment (qualifying contract A on master); a successor comes only after, against that qualified base, under D082's sequence; the diagnostic is no qualified base and no run. |
| Researcher 3: the timing boundary | §5.1: named as `ab-child/2`'s `wall`, from `t0` before `decode(...)` to `t0.elapsed()` after it, in `child()`'s 1-thread branch; "before the boundary" used consistently (§2, §3.2); "whole-process wall time" dropped, as it was not what `wall` measures. Its 4-decimal resolution disclosed. |
| Researcher 3: estimates are not deadlines | §6.3: frozen job limits, deadline and retry budget. |

**Choices made in this revision that the reviews left to the team,** for the Researcher and
the Guardian to confirm or correct before freeze:
- the allocation rule uses B≥, not B+ (§5.5);
- the 2% rule's form (the Researcher's example form);
- S1 as "≤ C" for each term separately;
- stage 2's deadline (7 days), job limits (180 and 30 minutes) and the class-B retry budget
  (2 a stage);
- CPU 1 for pinning, and the stack residue proxy in `child()`'s frame.

**Revision 2** (41be4c27): the Researcher's REVIEW 6763089509 item 1: a fixed precision
target; the additive contrast analysed directly, in 20 four-slot blocks; whole-file exposure;
a statement of what a net effect can't show.

**Revision 1** (72eb8af4): the first draft.
