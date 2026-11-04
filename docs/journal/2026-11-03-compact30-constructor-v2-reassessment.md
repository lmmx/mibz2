# Compact30: Astra's reassessment of constructor v2, a design-only NO-GO (report 224), and the instruction-floor investigation (report 225)

**Report 224, Astra's reassessment: a design-only NO-GO** for another prototype under the
current economic targets. It answers [the request assembled in the capture
journal](2026-11-03-compact30-construction-v2-capture.md#the-request-to-astra-as-assembled).
- **The strongest remaining bounded change:** specialise and fuse ordinary group completion
  with the next descriptor transition.
- **Why NO-GO:** the change has a credible CPU-cost argument, but no supported route to the
  completion ceiling with margin. Modelled on the raw records, about 6.64 ns would have to
  disappear from every fast ordinary finish (about 33 cycles at an illustrative 5 GHz) just
  to reach the point ceiling.
- v2's capture remains INVALID.

Astra states that it downloaded and verified the archive (the SHA-256 and all 69 manifest
entries), examined all 75,087 records, and inspected the timed listings.

**Report 225, the instruction-floor investigation.** The user then sent a broader request to
Astra in a fresh session: "Assembly-Guided Instruction Elimination and the Practical
Construction-Cost Floor". It is the guardian's prompt with the researcher's seven amendments,
kept verbatim below. Its result is pending. No implementation, benchmark or i9 work is
authorised.

## A correction to Claude's appendix

**Astra found an error in the appendix that Claude supplied,** and the error is confirmed here.
- The appendix's "all-full" steps meant blocks of **exactly 900,000 rows**. The registered
  analyser's definition is `FULL = 899_000` (`bench/c30_construction.py` at `fa21f33`, line 81).
- So the appendix's "steps with a short block" class contained mostly *nearly* full blocks,
  such as 899,828 or 899,959 rows.

Recomputed from the raw records (warm W4, rounds 0–19, step completion = last end − first
start):

| step class | definition | steps/round | share of P's completion | C ÷ P completion | share of the excess (0.113) |
|---|---|--:|--:|--:|--:|
| all full | exactly 900,000 rows (the appendix) | 49 | 0.841 | 1.095 | 0.080 |
| all "short" | below 900,000 (the appendix) | 59 | 0.143 | 1.222 | 0.032 |
| **all full** | **≥ 899,000 (registered)** | 57 | 0.982 | 1.111 | **0.109** |
| **all short** | **below 899,000 (registered)** | 51 | **0.0025** | 2.074 | **0.003** |

**What this changes:**
- In conversation, Claude claimed that about 28% of the completion excess was fixed per-block
  overhead on tiny blocks. **That is wrong.** Genuinely short-only steps account for about
  0.25% of production's completion time and contribute about 0.003 of the 0.113 excess,
  roughly 2%. This agrees with Astra's figure of about 0.46% of the candidate's completion.
  The per-block fixed cost is real but economically negligible here.
- The claim did not enter the request that produced report 224. It did shape the advice Claude
  gave the user about report 225's prompt, but that prompt as sent doesn't include the 28%
  figure.
- The appendix's step-class table (also reproduced in report 225's prompt) should be read with
  the registered partition, as above.
- The appendix's other findings stand, and Astra reproduced them:
  - the aggregate ratios;
  - the slowest-to-mean imbalance (1.0467 against 1.0733);
  - the closeness of wall time to CPU time;
  - the bulk-share quartiles on exactly-900,000-row blocks (1.177 and 0.983 at the
    extremes).

## Report 224: Astra's reassessment, as relayed

Kept as relayed. The relay's escape characters are removed; the words are unchanged.

**Recommendation: design-only NO-GO for another prototype under the current economic targets.** The strongest remaining bounded change is to specialize and fuse ordinary group completion with the next descriptor transition. It has a credible CPU-cost argument, but I cannot justify expecting it to close the approximately 8.5% completion gap with statistical margin.

That is an engineering recommendation, not a reclassification: **v2’s capture remains INVALID**.

I downloaded the archive, verified its supplied SHA-256, checked all 69 manifest entries, and examined the 75,087 JSONL records. I also inspected the complete `build::<false>` listing, `finish`, `refill`, canonicalization, and the production builder. All diagnostic calculations below retain rounds 0–19.

**The completion problem is primarily a difference in per-block execution cost, amplified by the fixed schedule’s maximum-worker completion rule. Its hardware cause remains unresolved.**

The aggregate raw-record arithmetic reproduces the appendix:

| Warm W4, all rounds | C/P |
|---|---:|
| Summed thread CPU | 1.08607 |
| Summed step completion | 1.11342 |
| Sum of each step’s slowest worker duration | 1.11350 |

The weighted slowest/mean-worker factor rises from approximately **1.04670 to 1.07331**. This explains arithmetically why completion is worse than aggregate CPU cost. It does not identify why individual builds cost more.

There are three consequential qualifications.

First, the appendix’s “all-full” partition means **exactly 900,000 rows**, whereas the registered analyzer defines `FULL = 899_000`. Its figures reproduce under the exact-900k partition:

| Step population | CPU C/P | Completion C/P |
|---|---:|---:|
| Four exactly 900,000-row blocks | 1.07116 | 1.09458 |
| Four blocks meeting the registered full-block definition | 1.08309 | 1.11086 |

The appendix’s complementary “short” category therefore includes nearly full blocks. **Genuinely short-only steps consume just 0.166 ms of candidate completion per corpus pass, approximately 0.46%.** Eliminating their entire measured duration could not close the gap. Optimizing tiny-block initialization is consequently a distraction from the primary requirement.

Second, the adverse completion ratio is widespread. Using each step’s median across all twenty rounds, **53 of the 57 registered all-full steps exceed 1.02**. For exactly 900,000-row blocks, the all-round median CPU ratios remain strongly associated with bulk coverage: the lowest and highest bulk-share quartiles average approximately **1.177 and 0.983**, respectively. However, scalar appends, normalization counts and other group-boundary work covary with bulk share. This is not function-level attribution.

Third, production’s round-5 anomaly is substantially localized to **worker 3, pinned to CPU 4**. Its aggregate thread CPU is approximately **37.82 ms**, versus its all-round median of **30.92 ms**; the other production workers are near their usual totals. Wall time rises with CPU time. The archive cannot distinguish frequency changes, memory-service delays, or another source of on-CPU execution delay.

Aggregate wall-minus-thread-CPU differences are approximately **0.15% for P and 0.13% for C**, including clock-bracketing effects. That does not mean every individual record differs by only that amount, but off-CPU waiting cannot account for the aggregate completion gap.

These observations strengthen the case for reducing work on slower, lower-bulk large blocks. They do **not** establish bandwidth saturation, cache misses, synchronization, or any particular memory-system mechanism. The protocol also explicitly compares different build/check cache histories; “warm” does not mean identical cache residency between arms. [Protocol](https://github.com/lmmx/mibz2/blob/1dc2edf/docs/journal/2026-11-03-compact30-construction-protocol-v2.md)

**V2’s remaining work extends well beyond its twelve-instruction append.**

The census and matched assembly establish the following opportunities:

| Work | Actual scope | What remains unresolved |
|---|---|---|
| Ordinary normalization | 1,723,371 full groups and 42 partial groups; 51,701,707 fields | Its CPU share and cache residency |
| Scalar append | 38,294,883 rows | Frequency and cost of repeated-character pointer dependencies |
| Bulk emission | 5,127,300 records; 74.306% of rows | Whether instruction or store resources constrain this path |
| Exceptional finishing | 30,949 wide groups; 41,120 crossing portions | Actual exceptional-path time |
| Snapshots | 3,654 snapshots | Their measured contribution, especially on tiny blocks |

The ordinary full-group normalization performs **five field loads, five `psubw` operations and five field stores**, covering exactly 60 bytes each way. Across the corpus, normalization contributes approximately **103.4 MB of reads plus 103.4 MB of writes**, about **one extra explicit byte of traffic per input row**. Those are operation counts, not DRAM traffic.

The fast ordinary `finish` executes **81 instructions**, including its prologue and epilogue. A normal scalar `refill` contributes another **62 executed instructions excluding its callee**. Thus an ordinary scalar rollover can traverse approximately **143 instructions**, before caller-side handling. Not every normalization goes through `refill`; runs and final draining call `finish` separately.

Specific avoidable work is visible:

- The same-epoch branch still reconstructs both endpoints and tests their span.
- `finish` computes lengths and positions needed by its general exceptional paths.
- `refill` resets descriptor pointers to null, then reconstructs the immediately following descriptor.
- Reopening computes division by thirty and consults the RAW bitmap even when a natural-configuration interior bucket transition determines the next group’s class.

The twelve-instruction append also hides an extra memory operation:

```asm
mov    (...), %rax        ; load write pointer
cmp    (...), %rax        ; load end pointer
mov    %bx, (%rax)        ; field store
addq   $2, (...)          ; another pointer read, then pointer write
```

Keeping the loaded pointer and storing its increment would eliminate that second read. It would not eliminate the repeated-character store-to-load recurrence. The archive contains path counts, not the byte sequences needed to quantify that recurrence’s importance.

Finally, the complete build initializes a 10 KiB descriptor array and copies it into its working state; its stack frame is **21,736 bytes**. Each snapshot also uses a temporary array and copy. These are plausible contributors to tiny-block overhead, but tiny-block completion has insufficient weight to fund the target.

Sources: [frozen constructor](https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs), [matched build listing](https://github.com/lmmx/mibz2/blob/e035556/docs/journal/2026-11-03-compact30-construction-v2-harness/listings/ctor2-build-false.asm), [finish](https://github.com/lmmx/mibz2/blob/e035556/docs/journal/2026-11-03-compact30-construction-v2-harness/listings/ctor2-finish-false.asm), [refill](https://github.com/lmmx/mibz2/blob/e035556/docs/journal/2026-11-03-compact30-construction-v2-harness/listings/ctor2-refill-false.asm).

**The requested alternatives do not presently provide a stronger completion-cost case.**

- **Earlier finalization** could avoid reacquiring a record line after eviction. It removes no normalization loads or stores. Many run completions already occur promptly, and the archive does not record the remaining reuse distances. Immediate wide loads after narrow field stores can also encounter store-forwarding restrictions. Cache residency alone is insufficient justification.
- **Forming final offsets during append** could eliminate normalization traffic, but requires carrying and applying the base during appends, with correct recovery for eventual RAW groups. It trades amortized vector operations for additional work on the scalar recurrence. There is no established favorable trade here.
- **Simplifying the descriptor update** removes up to 38.29 million redundant pointer reads. That is a legitimate opportunity, but there is no basis for assigning it an 8.5% completion benefit.
- **Improving bulk emission** could remove 5.13 million extra stores and reduce pattern reloads. It preserves essentially the same destination cache-line traffic and disproportionately improves the part of the distribution already performing well.

I would not combine these into an optimization package whose mechanism could not subsequently be attributed.

**The strongest bounded design is a specialized ordinary group transition.**

Its common eligibility conditions are:

```text
natural configuration
current portion is a complete 30-row ordinary candidate
latest_snapshot[c] <= current_first_rank
next complete 30-row group remains inside the same character bucket
```

The operation would then:

1. Read the first temporary field.
2. Form the header from that field, the current snapshot epoch and `c`.
3. Normalize the thirty fields with the existing bounded SIMD operation.
4. Advance the record pointer by 64 bytes and destination rank by thirty.
5. Install the next descriptor directly, without null reset, division or bitmap lookup.

A specialized ordinary completion body could also serve run closure, without prematurely opening a group that the bulk emitter will handle.

The decisive invariant is that successors received in one epoch have low labels in `0..=65533`. Once same-epoch membership and ordinary ownership are established, the full-label span test is redundant. For the next group, remaining entirely within the same bucket proves homogeneous ownership in the natural configuration.

The generic machinery must retain responsibility for:

- Initial, partial, crossing and exhausted descriptors.
- Multi-epoch portions and wide promotion.
- Forced-RAW configurations.
- Correct rejection of malformed counts.

Header formation must use the **latest saved snapshot epoch**, not blindly the current source coordinate’s high bits: end-of-input can sit at an epoch boundary.

This preserves byte-identical arrays. Ordinary bases and offsets remain unchanged; exceptional paths and canonical RAW ordering remain intact.

The expected machine-code change is substantive: a short ordinary completion path, fewer preserved registers and nested calls, no redundant last-field/span calculation, and direct descriptor advancement. **The five normalization loads, five normalization stores, scalar pointer recurrence and bulk emitter remain.** Consequently, this design cannot claim to solve a hypothesized cache-line-revisit penalty.

It could improve completion because lower-bulk blocks have more group completions, and those blocks disproportionately determine step duration. That is a concrete distribution-specific reason to prefer it over bulk-emitter polishing.

**Its opportunity is nevertheless smaller than the required economic claim.**

The census contains **1,647,416 fast ordinary finishes**, corresponding to approximately 23.9% of input rows. This is an upper bound on exposure, not a CPU-time fraction. The subset permitting the full fused interior transition is smaller, and the archive does not count it separately.

An aggressive implementation might remove roughly **40–50 instructions from ordinary completion**, with further savings on eligible descriptor transitions. These are design estimates, not generated-code results. Applying the larger saving to every normalization would overstate the opportunity.

I used the raw records for a separate **budget calculation**: assume a fixed number of nanoseconds disappears from every fast ordinary finish, retain all recorded steps and starts, and recompute which worker would finish last. This is a hypothetical savings model—not a revised experimental result.

| Assumed saving per fast ordinary finish | Modeled completion reduction |
|---|---:|
| 2 ns | 2.53% |
| 4 ns | 5.05% |
| 6 ns | 7.58% |

Approximately **6.64 ns per fast ordinary finish** is required merely to reach the descriptive point completion ceiling. At an illustrative 5 GHz, that is approximately **33 cycles per event**. Statistical margin requires more; reduced eligibility raises the requirement further.

The assembly demonstrates removable work, but it does not demonstrate that much removable execution time. In particular:

- Some finishes lack the additional `refill` overhead.
- The field transformation and its memory dependencies remain.
- Much descriptor and snapshot work can overlap with other execution.
- New eligibility checks consume part of the saving.
- Any larger benefit from eliminating stalls is currently an unsupported assumption.

There is a credible route to the smaller CPU-cost gap. **I do not see a sufficiently supported route through this bounded change to the completion ceiling with margin.**

Before another capture, static inspection could falsify the mechanism if the proposed ordinary path still enters general `finish`, retains descriptor reset/reconstruction, introduces spills, or adds work to every scalar append. Oracle comparison would have to prove exact arrays across all existing boundary and exceptional cases.

However, passing those checks would still not establish the required approximately 6.6 ns saving. That is why I recommend **NO-GO for a correctness-and-compilation prototype under the present investment requirement**. The evidence that would reverse this decision is attribution showing sufficiently large, removable group-transition cost on the critical large-block builds. The current archive establishes where the performance problem appears, but cannot supply that attribution.

## Report 225: the instruction-floor request, as sent

The guardian consolidated its prompt with the researcher's seven amendments. The user sent it
to Astra in a fresh session at its highest reasoning setting, with the capture archive. It is
kept verbatim. Its step-class table, under "8. Analyze the critical path", uses the
appendix's exactly-900,000 partition corrected above.

### Compact30: Assembly-Guided Instruction Elimination and the Practical Construction-Cost Floor

### Your role and mission

You are conducting an intensive machine-code optimization investigation of the compact30 constructor in `lmmx/mibz2`, a performance-oriented Rust bzip2 decoder.

The target machine is an **Intel Core i9-10900K (Comet Lake), using four physical cores**.

**This is not an open-ended algorithm-ideation exercise.**

Our most successful optimization methodology has been:

1. Inspect the actual optimized x86-64 assembly produced by Rust.
2. Identify unnecessarily expensive instruction sequences, dependent operations, memory accesses and repeated calculations.
3. Find mathematically equivalent ways of producing exactly the same output with substantially less machine-level work.
4. Determine whether the compiler can generate the intended simpler instruction sequences.
5. Account for the total work eliminated, including work introduced elsewhere.

This methodology produced our most significant construction improvement: constructor v1's approximately **49-instruction scalar iteration** became v2's **12-instruction iteration**, while the directional four-worker construction-cost ratio improved substantially.

We want to push this approach much further.

**Your mission is to find the practical minimum execution work required to construct the exact frozen compact30 representation.**

Examine the entire constructor, not merely its scalar loop. Find where the remaining unnecessary instructions and dependencies are hiding.

Use your available reasoning depth extensively. A substantial, auditable technical investigation is preferable to a quick answer.

You may investigate multiple equivalent transformations, but they must arise from concrete instruction-elimination opportunities, not from a desire to invent novel algorithms.

The primary objective is **less dynamically executed work and shorter critical execution paths**, particularly for the workers that determine synchronized four-core completion time.

---

### 1. Project context and verified research

Repository:

[https://github.com/lmmx/mibz2](https://github.com/lmmx/mibz2)

Compact30 is a compact navigation-table representation for inverse BWT traversal.

Its frozen representation uses:

- 64-byte, 64-byte-aligned records.
- Thirty `u16` fields occupying record bytes 0–59.
- A four-byte header at bytes 60–63.
- Sparse 128-byte RAW groups for exceptional cases.
- Canonical RAW ordering.
- The existing, unchanged compact30 walker.

#### The traversal result

Prepared-table traversal previously achieved a **formally VALID ADVANCE** on the i9:

| Measure                               | Compact30 / conventional |
| ------------------------------------- | ------------------------ |
| Four-worker traversal CPU cost        | 0.84047                  |
| Four-worker traversal-step completion | 0.85601                  |
| One-worker traversal CPU cost         | 1.14845                  |

Thus compact30 used approximately 15.95% less four-worker traversal CPU time and 14.40% less four-worker traversal completion time, but was approximately 14.85% slower with one worker.

These results concern **prepared-table traversal**, not construction or whole-decoder performance.

The construction cost must be sufficiently low to preserve the multicore advantage. Real decoder concurrency may mix one, two, three and four active walkers, which is not established by these isolated experiments.

#### Constructor v1

V1 formed final offsets online while scattering source rows, repeatedly checking record headers, RAW status and span limits.

Its scalar iteration was approximately 49 static instructions along the cited common path.

The v1 construction capture was formally **INVALID**. Its directional four-worker ratios were:

- CPU construction cost C/P: **1.31583**
- Construction-step completion C/P: **1.36409**

The specific v1 implementation was stopped.

#### Constructor v2

V2 introduced deferred offset formation:

- Append temporary low-16-bit successor labels directly into final record fields.
- Retain the complete-record bulk emitter.
- Store sparse per-character destination-rank snapshots at 61,440-row epochs.
- Recover full labels when completing a portion.
- Classify ordinary versus wide groups from full labels.
- Normalize ordinary groups with SIMD subtraction.
- Preserve existing RAW canonicalization.

The compiled common scalar append is approximately **12 instructions**, without the previous repeated header/RAW/span dependency chain.

V2 passed extensive byte-exact oracle comparisons, independent source and memory-safety audit, and targeted Miri checks.

The first v2 i9 cost capture was also formally **INVALID**, because production's warm four-worker stability gates G1 failed. Its synchronization gates G2, correctness, identity, memory, faults and clock-overhead checks passed.

The capture recorded these **descriptive**, non-admissible performance figures:

| Four-worker measure         | V1 capture | V2 capture | Economic ceiling |
| --------------------------- | ---------- | ---------- | ---------------- |
| CPU construction C/P        | 1.31583    | 1.08619    | 1.06714          |
| Construction completion C/P | 1.36409    | 1.11437    | 1.02000          |

V1 and v2 were measured in separate, formally invalid captures. Their difference is not a controlled, registered v1-versus-v2 result.

The existing v2 architecture is on engineering **HOLD**, not declared exhausted. No further performance experiment is currently authorized.

---

### 2. Exact evidence to inspect

#### Frozen Rust sources

Production builder:

[https://github.com/lmmx/mibz2/blob/e003a32/src/bwt.rs](https://github.com/lmmx/mibz2/blob/e003a32/src/bwt.rs)

Constructor v1:

[https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs)

**Constructor v2 — primary optimization subject:**

[https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs](https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs)

Frozen representation, walker and oracle:

[https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk.rs](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk.rs)

#### V2 prototype and machine code

V2 prototype report, structural census and machine-code discussion:

[https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype.md](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype.md)

Full optimized v2 construction listing:

[https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-build-false.asm](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-build-false.asm)

V2 finalization listing:

[https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-finish-false.asm](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-finish-false.asm)

V2 refill listing:

[https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-refill-false.asm](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-refill-false.asm)

Annotated assembly excerpts:

[https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/excerpts.asm](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/excerpts.asm)

The later construction-cost harness confirmed that the frozen v2 constructor's relevant machine-code listings remained identical after address normalization.

#### Construction-cost experiment

Registered v2 protocol:

[https://github.com/lmmx/mibz2/blob/1dc2edf/docs/journal/2026-11-03-compact30-construction-protocol-v2.md](https://github.com/lmmx/mibz2/blob/1dc2edf/docs/journal/2026-11-03-compact30-construction-protocol-v2.md)

V2 capture report, per-round table and corrected Guardian disposition:

[https://github.com/lmmx/mibz2/blob/603d8c9/docs/journal/2026-11-03-compact30-construction-v2-capture.md](https://github.com/lmmx/mibz2/blob/603d8c9/docs/journal/2026-11-03-compact30-construction-v2-capture.md)

**Complete capture archive, including raw JSONL records:**

[https://github.com/lmmx/mibz2/blob/e035556/docs/journal/2026-11-03-compact30-construction-v2-capture/c30-construction-v2-20261103T222928Z.tar.gz](https://github.com/lmmx/mibz2/blob/e035556/docs/journal/2026-11-03-compact30-construction-v2-capture/c30-construction-v2-20261103T222928Z.tar.gz)

SHA-256:

`ebdb74549b39f83f07ae1f7d1c943916bac2ecb0be85868799aa30fe5e3d8af9`

Please examine the actual sources and listings rather than relying entirely on the reports.

If the archive is accessible, independently examine its raw build records. If you cannot obtain it, say so explicitly and distinguish findings grounded in summaries from findings independently derived from raw data.

**Do not claim to have inspected any source, listing or record that you could not actually access.**

---

### 3. The primary-corpus structural frequencies

The principal Wikidata corpus has 434 blocks and 207,006,515 input rows.

V2's untimed structural census records approximately:

| Construction path or operation    | Primary-corpus frequency |
| --------------------------------- | ------------------------ |
| Scalar append                     | 38.3 million rows        |
| Fragment append                   | 14.9 million rows        |
| Direct complete-record emission   | 5.13 million groups      |
| Ordinary full-group normalization | 1.72 million groups      |
| Wide-group finalization           | 30,949 groups            |

Approximately 74.31% of rows are covered by the direct complete-record emitter.

These counts are an essential starting point, **not a complete dynamic instruction trace**.

Other costs include:

- 256 per-character descriptor initializations per block.
- Descriptor refill and completion transitions.
- Epoch snapshots and searches.
- Run detection and coalescing.
- Crossing-group handling.
- Wide and forced-RAW paths.
- RAW canonicalization.
- Allocation, bitmap initialization, padding and final publication.

Some recorded categories are aggregates or count related events. Do not accidentally double-count work by assuming every counter represents an independent execution path.

Where structural counters cannot establish a precise branch frequency or loop count, derive bounds, estimate transparently or identify the missing information.

---

### PHASE I — ACCOUNT FOR THE MACHINE CODE

### 4. Build an execution-cost ledger

Your first substantial task is to account for the **actual work performed by the optimized constructor**.

Trace `Ctor2::build::<false>` through its inlined and out-of-line paths.

Investigate:

1. Scalar append and tile processing.
2. Uniformity detection, run scanning and run coalescing.
3. Fragment filling.
4. Direct complete-record emission.
5. `refill`, `open`, `close` and descriptor maintenance.
6. Ordinary `finish` and SIMD normalization.
7. Wide-group detection and RAW reconstruction.
8. Crossing-group and preset RAW handling.
9. Epoch snapshots and full-successor recovery.
10. Canonical RAW permutation.
11. Per-block initialization, validation, allocation and padding.

Also inspect production's construction code to identify operations that production avoids because of its simpler representation.

For each path, report:

- The exact assembly location and relevant loop boundaries.
- Instructions executed along its common path.
- Loads, stores, branches, address-generation operations, integer arithmetic and SIMD operations.
- Loop-carried dependencies and serial load/use chains.
- Relevant exceptional or slow-path dispatch.
- The structural frequency of the path.
- Estimated dynamically executed instructions and explicit memory operations across the primary corpus.
- Whether the count is exact, bounded or estimated.

**Do not mistake total function size for the number of instructions executed per call.**

For example, `finish` may be hundreds of instructions in total, but its common ordinary full-group path executes only a subset. That subset must be combined with the actual frequency of ordinary finishes, and slow paths must be accounted for separately.

#### Microarchitectural interpretation

On the i9-10900K's Comet Lake microarchitecture, reason where defensible about:

- Decoded micro-ops.
- Load/store throughput and store pressure.
- Integer and SIMD execution resources.
- Address-generation dependencies.
- Branch predictability.
- Loop-carried latency.
- Instruction-level parallelism.
- Cache-line locality and repeated field access.

Consult authoritative instruction or microarchitecture references if necessary.

**Do not invent latency numbers, port usage, cache-miss counts or runtime attribution.**

A shorter instruction stream need not mean a proportionally faster runtime. Conversely, eliminating a dependent load chain may matter more than eliminating several independent arithmetic instructions.

The objective is to understand both.

---

### PHASE II — MAINTAIN A CUMULATIVE EXECUTION BUDGET

### 5. Establish the baseline workload-weighted cost

Create a cumulative machine-code ledger that accounts for the constructor as a whole.

Use a table with columns equivalent to:

| Cost center | Frequency | Current instructions/event | Estimated total instructions | Loads/stores | Critical dependencies | Confidence |
| ----------- | --------- | -------------------------- | ---------------------------- | ------------ | --------------------- | ---------- |

Where necessary, break one path into common and exceptional cases.

Calculate the estimated contribution of each major path to:

- Total dynamically executed instructions.
- Total explicit loads and stores.
- Repeated dependent memory operations.
- Group-level and per-block fixed work.

Report the **coverage of your ledger**: which portions of execution it explains and which remain unknown.

An estimated total is acceptable if its assumptions are explicit. A falsely precise total is not.

For example, removing five instructions from 30,949 wide finishes is a much smaller aggregate opportunity than removing two instructions from 38.3 million scalar appends, before considering differences in instruction cost.

But an optimization that removes ten million instructions from appends and adds twenty million during finalization is not necessarily progress.

#### The running-budget requirement

**Whenever you propose an optimization, update the cumulative workload-weighted budget.**

Track:

- Instructions removed.
- Instructions introduced.
- Loads eliminated.
- Loads introduced.
- Stores eliminated.
- Stores introduced.
- Dependency chains shortened or added.
- Additional setup, exceptional-path and per-block costs.
- The estimated net reduction.
- The residual work after the proposed transformation.

**Never count the same saving twice across overlapping proposals.**

If two transformations are mutually exclusive, evaluate them as alternatives. If they can coexist, explain why their savings should be approximately additive—or why their interaction might reduce the combined benefit.

Continue maintaining this budget through the entire investigation.

Do not produce disconnected local optimization estimates that cannot be reconciled into a total.

---

### PHASE III — FIND THE UNAVOIDABLE FLOOR

### 6. Separate necessary work from implementation overhead

For every significant cost center, ask:

**What is the minimum work fundamentally needed to produce exactly the same compact30 output, and how much additional work does our current implementation perform?**

Distinguish four categories:

**A. Unavoidable work**

Operations required by the frozen output representation or by information-theoretic and ordering constraints.

Examples include constructing correct destination ranks, emitting required final records and representing exceptional RAW groups.

**B. Algorithmically removable work**

Repeated calculations, writes or checks that an equivalent construction strategy could avoid.

**C. Code-generation overhead**

Instructions arising from the current Rust expression, type structure, indexing, branching or compiler optimization choices that might be eliminated without changing the high-level algorithm.

**D. Costs not determined by static code**

Cache misses, bandwidth contention, execution-port pressure, frequency effects and other machine behavior requiring measurements to establish.

Investigate, in particular, whether the constructor fundamentally needs:

- Temporary field writes followed by rereads and normalization.
- Repeated descriptor loads and pointer updates.
- Repeated source-coordinate and destination-rank arithmetic.
- Per-portion classification and dispatch at the current frequency.
- The current RAW reconstruction sequence.
- The current initialization of metadata.
- The current number of record stores in the bulk emitter.

Prove lower bounds where possible.

For everything else, identify a **plausible practical execution floor**, not an unsupported global optimum.

The frozen final representation imposes costs, but the existing construction order does not automatically represent the cheapest possible way to pay them.

---

### PHASE IV — ELIMINATE INSTRUCTIONS AND DEPENDENCIES

### 7. Find the next major instruction-elimination opportunity

Explore transformations only when they emerge from an identified cost center.

Potential avenues include:

- Replacing multi-instruction arithmetic with simpler equivalent expressions.
- Eliminating redundant header or bitmap accesses.
- Reducing descriptor reads and memory read/modify/write dependencies.
- Amortizing source or destination bookkeeping across rows, groups or runs.
- Combining operations currently performed during append and finalization.
- Avoiding temporary writes or eliminating their later rereads.
- Reducing full-group normalization instructions or memory operations.
- Avoiding repeated group classification when enough information is already known.
- Simplifying common-case `refill` or `finish` transitions.
- Reducing bulk-emitter pattern loads, stores or address calculations.
- Removing unnecessary exceptional-path work from common paths.
- Exploiting invariants already guaranteed by bucket boundaries, epoch snapshots or the frozen record format.

These are **investigation targets, not a required wishlist**.

You may discover that some are already close to their practical floor.

You may also propose a different construction data flow, but only if it demonstrably removes substantial executed work rather than merely rearranging complexity.

#### Required analysis for every leading transformation

Specify:

**1. Existing assembly**

The exact sequence and location of the operations you intend to replace.

**2. Equivalent computation**

The mathematical or representation invariant that makes the replacement correct.

**3. Work eliminated**

Which instructions, loads, stores, branches and dependency edges disappear.

**4. Work introduced**

All additional setup, metadata maintenance, finalization, allocation, exceptional-path handling and memory traffic.

**5. Proposed machine-code shape**

Show an illustrative instruction sequence or pseudocode with precise machine-level expectations.

Do not invent a verified listing for code that hasn't been compiled.

**6. Workload-weighted benefit**

Use the structural census to estimate total savings and update the cumulative ledger.

**7. Four-worker relevance**

Explain whether the optimization affects low-bulk blocks, slow workers and synchronized completion, rather than only already-efficient paths.

**8. Falsification conditions**

State what would cause the proposal to fail its next untimed correctness-and-compilation checkpoint.

For example:

- The compiler reintroduces the dependent header load.
- The intended arithmetic simplification compiles to the same or more micro-ops.
- A new metadata update creates a serial dependency.
- The elimination adds more field reads and writes than it removes.
- A cheap scalar path produces an expensive group-finalization path.
- Bulk coverage deteriorates materially.
- The result cannot remain byte-identical to the oracle.

#### Don't overfocus on the 12-instruction scalar loop

The current scalar loop is already substantially improved. Reducing it to eight instructions might be useful, but only if that reduction matters to total construction cost.

The next important opportunity might instead be hidden in millions of normalizations, repeated descriptor transitions or bulk-group operations.

**Find the next 49-instruction problem, wherever it actually is.**

---

### PHASE V — EXPLAIN FOUR-WORKER COMPLETION

### 8. Analyze the critical path of synchronized construction

The economic objective is not simply lower aggregate instruction count.

Construction-step completion is determined by the last worker to finish a synchronized step.

V2's descriptive four-worker CPU ratio is approximately **1.086**, while its completion ratio is approximately **1.114**.

Inspect the archived per-round, per-worker, per-step and per-block records if accessible.

An existing descriptive decomposition of the INVALID capture suggests:

| Warm W4, C/P    | All steps | All-full steps | Steps containing a short block |
| --------------- | --------- | -------------- | ------------------------------ |
| Thread CPU      | 1.0861    | 1.0712         | 1.1717                         |
| Step completion | 1.1134    | 1.0946         | 1.2129                         |

The average per-step imbalance, calculated as slowest-worker time divided by mean-worker time, was approximately:

- Production: **1.0467**
- Compact30 v2: **1.0734**

Arithmetically, the difference between these imbalance factors can account for the gap between aggregate CPU cost and synchronized completion.

This is **not a causal explanation of why particular workers are slower**.

#### Block-cost variation

The existing analysis also reports an association between v2's relative construction cost and a block's bulk-emitter coverage.

By quartile of bulk coverage on full blocks:

| Mean bulk share | Mean v2/production CPU ratio |
| --------------- | ---------------------------- |
| 70.4%           | 1.178                        |
| 74.5%           | 1.093                        |
| 76.0%           | 1.064                        |
| 78.3%           | 0.985                        |

The reported correlation is approximately −0.74.

Short blocks are also disproportionately expensive relative to production.

**Verify these findings from raw records where possible.** Do not treat correlation as source-level cost attribution.

Investigate:

1. Which blocks and steps account for the largest completion penalties?
2. How often a low-bulk or otherwise expensive block determines the slowest worker.
3. Whether the relevant blocks execute more scalar appends, fragments, finalizations or fixed per-block work.
4. Which instruction eliminations would disproportionately benefit those blocks.
5. Whether proposed savings would reduce step imbalance as well as aggregate CPU time.
6. Which effects depend on the fixed worker-to-block assignment rather than intrinsic block cost.

Distinguish:

- Total worker CPU savings.
- The slowest worker's savings.
- Reduction in per-step imbalance.
- Shortening of dependent instruction chains.
- Memory effects that remain hypotheses.

Each worker's wall time was close to its thread CPU time in the earlier analysis. This argues against off-CPU waiting as the explanation, but **thread CPU time includes memory-stall cycles**. It does not establish that cache or bandwidth effects are absent.

#### Production-control instability

The v2 capture's G1 failure was concentrated in production round 5.

Production warm W4 completion reached approximately 38.32 ms in that round, compared with approximately 31.55–32.40 ms elsewhere. The candidate's round was comparatively normal.

The underlying cause is not established.

Do not exclude round 5 from the formal result or relabel INVALID as FAIL.

Any exclusion-based comparison is a **post hoc diagnostic only**, and must be labeled as such.

---

### PHASE VI — ECONOMIC HEADROOM AND STOPPING RULE

### 9. Target more than the nominal point-estimate gap

The v2 capture's directional point estimates imply:

- CPU cost would need approximately **1.8%** further reduction to reach the nominal 1.06714 ratio.
- Completion would need approximately **8.5%** further reduction to reach the nominal 1.02 ratio.

But the registered PASS requires the **upper 95% confidence bound** to clear each ceiling.

The INVALID capture reported upper bounds of approximately:

- CPU: 1.09517.
- Completion: 1.13693.

As a simple **proportional sensitivity calculation**, moving those upper values to their ceilings would require approximately:

- **2.6%** less CPU cost.
- **10.3%** less completion time.

These are illustrative calculations, **not prospective confidence intervals or reliable predictions**. A revised implementation may have different variance, baseline conditions or worker imbalance.

Nevertheless, they establish the scale of the required opportunity.

Prefer an optimization strategy with credible headroom for a **roughly 10–15% completion-time reduction**, rather than a collection of marginal changes that only just reach the nominal point estimate under optimistic assumptions.

Do not manufacture a predicted speedup from instruction reductions.

#### Assess the maximum plausible total benefit

After accounting for your strongest proposed eliminations:

1. What fraction of current estimated instruction work disappears?
2. How much new work is introduced?
3. Which eliminated operations plausibly lie on the critical execution path?
4. How much of the benefit could affect the slowest blocks?
5. Are the savings additive, overlapping or mutually exclusive?
6. How sensitive is the conclusion to unknown cache and execution effects?
7. Is the total opportunity large enough to justify further engineering work?

Include optimistic, central and conservative **conditional scenarios** if useful, with assumptions explicit.

Do not present them as measured performance forecasts.

### 10. Continue until the meaningful opportunities are exhausted

**Do not stop after finding one promising improvement.**

Whenever you find a concrete elimination:

1. Add its estimated costs and savings to the cumulative ledger.
2. Recalculate the remaining avoidable work.
3. Examine the next significant cost center.
4. Determine whether improvements can coexist.
5. Challenge the resulting combined opportunity.

Continue until the remaining candidates are:

- Too small to materially change the economic result.
- Too speculative to defend.
- Likely to introduce as much work as they remove.
- Blocked by correctness or representation requirements.
- Dependent on compiler behavior that is unlikely to survive compilation.
- Otherwise unsupported by the available evidence.

The purpose is to approach the **practical execution-cost floor**, not to accumulate impressive-looking local instruction reductions.

If the analysis reveals that v2 is already close to that floor, **say so**.

A rigorous NO-GO is valuable research.

---

### PHASE VII — FALSIFIABLE IMPLEMENTATION TARGETS

### 11. Specify the strongest first implementation candidate

Your investigation may discover multiple valuable transformations.

Rank them, but select **one strongest first implementation candidate** for Guardian consideration.

For that candidate, supply enough specificity that a Rust implementer could attempt it without independently reinventing the design.

Include:

- Exact input/output semantics.
- The source routines and data structures affected.
- The required mathematical invariants.
- The expected replacement computations.
- The intended assembly shape.
- Target instruction and memory-operation reductions.
- New operations and exceptional-path work.
- Workspace and allocation consequences.
- Initialization and bounds-safety requirements.
- Expected effects on bulk, fragment, scalar, wide and crossing paths.
- Expected effect on low-bulk and short-block costs.
- Potential interactions with other candidate optimizations.

#### Predefine the untimed rejection conditions

Specify exactly what must be visible after compilation to justify considering the implementation further.

Examples:

- A particular dependent load chain is absent.
- The hot-path instruction count reaches a stated target.
- New per-row loads or stores remain within a stated budget.
- Finalization doesn't recreate the eliminated work.
- Bulk coverage stays within a justified tolerance.
- No additional full-table intermediate buffer appears.
- The same final `rec` and `raw` arrays are produced byte for byte.
- No unsafe read of uninitialized record or header bytes occurs.
- Invalid counts and reused allocations remain correctly handled.

The target must be **falsifiable by inspecting the compiled code and running untimed correctness checks**.

Do not recommend a performance capture merely because the source code looks simpler.

---

### REQUIRED FINAL REPORT

Produce a substantial technical research report with the following sections.

### A. Evidence and provenance

List the exact Rust sources, assembly files, structural records and raw capture files you successfully inspected.

Identify missing evidence and any limitations arising from it.

### B. Baseline machine-code ledger

Give a workload-weighted account of the existing v2 constructor.

Include path frequencies, static instruction sequences, estimated dynamic instructions, memory operations, dependency chains and uncertainty.

Compare with production's builder where useful.

### C. Practical execution lower bound

Separate unavoidable representation work from avoidable implementation overhead.

Identify which paths already approach a reasonable machine-code floor and which retain substantial elimination opportunities.

### D. Cumulative instruction-elimination ledger

For each serious proposed transformation, show:

| Item                    | Required evidence                             |
| ----------------------- | --------------------------------------------- |
| Current path            | Exact source/assembly location                |
| Replacement             | Equivalent computation                        |
| Removed                 | Instructions, memory operations, dependencies |
| Added                   | All new work                                  |
| Frequency               | Structural count or justified estimate        |
| Net opportunity         | Workload-weighted reduction                   |
| Cumulative result       | Updated total and remaining work              |
| Critical-path relevance | Likely effect on slowest workers              |
| Verification            | Exact untimed pass/fail conditions            |
| Uncertainty             | Compiler, cache, safety and model risks       |

Avoid double-counted savings.

Distinguish independently additive transformations from mutually exclusive alternatives.

### E. Critical-path and completion analysis

Explain what the existing records establish about completion-time overhead, worker imbalance and input-dependent construction costs.

Identify which proposed eliminations are relevant to the slowest blocks.

Keep causal claims separate from descriptive arithmetic.

### F. Strongest practical optimization

Present the single strongest first implementation candidate.

Give its concrete algorithm, invariants, intended machine-code structure, expected eliminated work, added work, risks and falsifiable compilation requirements.

### G. Remaining execution-cost floor

After incorporating the strongest credible eliminations, estimate what significant avoidable work would remain.

Explain whether the construction economics appear limited by instructions, serial dependencies, field traffic, cache behavior or insufficient evidence.

Do not claim a global optimum without proof.

### H. GO / NO-GO

Give a clear recommendation:

**GO** — one bounded correctness-and-compilation prototype is justified by a concrete, sufficiently large machine-code cost-reduction opportunity.

**NO-GO** — the remaining plausible savings are too small, uncertain or costly to justify another implementation.

A GO does not authorize implementation or measurement. The Guardian will make that decision separately.

---

### NON-NEGOTIABLE CONSTRAINTS

- Preserve the frozen compact30 record layout.
- Preserve the walker.
- Produce byte-identical `rec` and `raw` arrays against the oracle, including canonical RAW ordering, headers and padding.
- Preserve exact wide-group classification and correct epoch reconstruction.
- Do not introduce a full successor table or F array.
- Do not move construction work outside the timed builder.
- Avoid a second scan of the BWT last column unless an exceptionally compelling elimination argument justifies reconsidering that constraint.
- Do not reinterpret either formally INVALID construction capture.
- Do not weaken or change registered validity or economic thresholds.
- Do not modify the repository, implement Rust variants, run benchmarks, use the i9, or conduct new performance experiments.
- Source inspection, assembly analysis, mathematical derivation, static cost modeling and analysis of **existing** recorded measurements are encouraged.
- Distinguish evidence, bounded estimates and speculation throughout.
- Never fabricate dynamic instruction counts, compiler output, instruction latencies or cache statistics.

**The goal is not to invent constructor v3 for its own sake.**

**The goal is to systematically eliminate unnecessary machine-level work from the complete compact30 construction process, account for the total savings without double-counting, and discover whether there is enough real headroom to meet the four-worker economic targets.**

Keep examining the next significant source of unnecessary work until you can defend either a concrete implementation candidate or a well-supported conclusion that the practical execution-cost floor is already too high.

Take the time and reasoning depth necessary. I want a technical report that can be independently challenged against the frozen assembly and structural census—not a quick collection of optimization suggestions.

## Report 226: the follow-up sent to the Ultra session, as relayed

The user sent the message below to the Ultra session after it had started. It contains:
- the guardian's factual correction (the registered `FULL = 899_000` partition);
- the guardian's and researcher's commentary on running both investigations.

**Anchoring note for whoever compares the two reports.** This message names Astra Max's
design-only NO-GO (report 224), its fused ordinary-group completion candidate, and its
6.6 ns-per-finish requirement. So the Ultra session's conclusions are not fully independent of
report 224 from this point on.

In its progress notes, Ultra reported having found the 900,000-row partition issue itself
before its final report. It also noted, correctly, that the appendix's slowest-to-mean
imbalance is time-weighted (the sum of each step's slowest-worker time divided by the sum of
its mean-worker time), not a simple average over steps.

Kept as relayed:

My ChatGPT (Sol Extra High) guardian chatbot said:

```
No. I don't think Astra Max's NO-GO endangers the Ultra investigation at all. In fact, I'm glad we're running both.

They are answering different questions.

Astra Max asked: Is there one sufficiently credible optimization to justify implementing another constructor?

Its answer was NO-GO. The proposed fused group-completion optimization might remove 40–50 instructions per completion, but Max couldn't justify the roughly 6.6 ns saving per event needed to clear the completion target.

Astra Ultra is asking something more ambitious: How much unnecessary machine-level work remains across the entire constructor, and what happens when we systematically eliminate it?

Ultra could reach the same NO-GO conclusion. But it could also identify several independent savings that collectively matter, even though none would justify a prototype alone.

That distinction is central to what you wanted.

### One thing I would correct immediately

Claude caught a genuine analytical mistake in the workload partition. Tiny blocks matter far less than we thought.

The registered threshold is 899,000 rows, not exactly 900,000. With that definition, all-full steps account for approximately 98.2% of production's completion time, while genuinely short-only steps account for just 0.25%.

That means Ultra should concentrate on removing instructions from the large-block construction paths, especially those prominent on low-bulk-coverage blocks.

Here's the correction I'd send.

Important factual correction to the evidence supplied in my instruction-minimization prompt:

The step-class table in Section 8 used "all-full" to mean four blocks of exactly 900,000 rows. The registered analyzer actually defines `FULL = 899_000`.

Under the registered definition:

- All-full steps account for approximately **98.2%** of production's warm four-worker completion time.
- Those steps contribute approximately **0.109** of the overall **0.113** descriptive completion-ratio excess.
- Genuinely short-only steps account for just **0.25%** of production's completion time and approximately **0.003** of the excess.

Therefore, optimizing tiny-block initialization is not a credible primary route to the economic target.

The aggregate CPU and completion ratios, worker-imbalance decomposition and association between bulk coverage and per-block cost remain unaffected.

Please incorporate this correction into your cumulative execution-cost ledger. Focus on the substantial, frequently executed paths in large blocks.

The objective remains unchanged: systematically account for and eliminate avoidable instructions, dependent operations and memory work across the **whole constructor**. Evaluate cumulative, non-overlapping savings rather than stopping after finding or rejecting one local optimization.

No implementation or benchmarking is authorized.

One practical caution: If Ultra is actively generating its response and sending another message would interrupt it, don't cancel the investigation. Give it this correction as a follow-up afterward and request any necessary revision.

### I would preserve Ultra's independence

There's one part of Claude's proposed message I would not send yet: the previous Astra instance's NO-GO verdict.

Not because it's wrong, but because we deliberately started Ultra to perform an independent, deeper accounting. Telling it another instance has already rejected the strongest candidate risks anchoring its conclusions.

Let Ultra independently discover which costs are removable, whether savings overlap, and how far the cumulative budget can fall.

When both investigations are complete, we can compare them:

- Did Ultra identify the same 40–50 removable instructions?
- Did it discover other substantial opportunities Max overlooked?
- Can those opportunities coexist without reintroducing work elsewhere?
- Does their combined cost argument approach the required completion improvement?

My position remains that the project is not finished simply because Max issued a NO-GO. A NO-GO on one bounded transformation is useful evidence, not proof that we've found the global or practical instruction floor.

And your instinct is still the one I want driving this: keep eliminating unnecessary machine instructions, systematically, until there's nothing substantial left to eliminate.
```

and my researcher chatbot said:

```
No. I think the Ultra investigation is entirely worth continuing, and the Max NO-GO doesn't undermine its purpose. In fact, it gives us a useful hypothesis for Ultra to challenge.

The two investigations are answering different questions.

| Astra Max                                                                                           | Astra Ultra                                                                                                                            |
| --------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Can one specialized ordinary-group completion mechanism plausibly close the remaining economic gap? | How much unnecessary machine-level work remains across the entire constructor, and what is the cumulative opportunity to eliminate it? |
| Conclusion: insufficient evidence to justify that individual optimization.                          | Still investigating the practical execution-cost floor.                                                                                |

Max's conclusion is reasonable within its scope. But it has not proved that v2 is near the minimum possible construction cost.

#### Why I wouldn't accept NO-GO as the final answer yet

Max identified several concrete inefficiencies:

- Approximately 143 instructions around a common ordinary scalar rollover, including `refill` and `finish`.
- A redundant pointer read in the 12-instruction scalar path, executed across roughly 38 million scalar rows.
- Repeated group-level dispatch and descriptor reconstruction.
- Approximately 1.72 million normalizations that read and rewrite record fields.
- Additional bookkeeping and initialization work.

These are real optimization opportunities, even though we don't know their runtime contributions.

Max's estimated requirement of 6.64 ns saved per fast ordinary finish is particularly useful. It tells us how demanding one proposed optimization would be.

But it is not a lower bound on what the constructor can achieve through several compatible transformations.

Ultra was specifically asked to explore those transformations, account for their combined effects without double-counting, and establish which costs are unavoidable.

That is a more fundamental investigation.

#### One thing I'd do differently from Claude's suggestion

I would send Ultra the factual correction, but not tell it that another Astra instance has already recommended NO-GO.

We deliberately started a fresh Ultra session to obtain an independent examination. Introducing another model's conclusion prematurely creates an unnecessary risk of anchoring.

Ultra should arrive at GO or NO-GO from its own cost ledger.

Here is exactly what I'd send:

**Correction to the supplied capture analysis — please incorporate this into your ongoing investigation.**

The step-class partition in Section 8 of my prompt used an incorrect definition of a full block: exactly 900,000 rows.

The registered analyzer actually uses `FULL = 899_000`.

Under that registered definition:

- All-full steps account for approximately 98.2% of production's warm four-worker completion time and approximately 0.109 of the observed 0.113 completion excess.
- Genuinely short-only steps account for approximately 0.25% of production's completion time and approximately 0.003 of the excess.
- Therefore, optimizing tiny-block initialization or fixed per-block overhead cannot explain or eliminate a substantial fraction of the observed completion disadvantage.

The aggregate construction ratios, per-worker imbalance factors and association with bulk-emitter coverage remain unchanged.

**Please use the corrected partition throughout your analysis and prioritize expensive large blocks and critical-path workers.**

One further emphasis: your objective is still the cumulative practical instruction-cost floor, not whether any one local optimization can independently close the remaining gap.

Evaluate compatible eliminations together, subtract their added costs, prevent double-counting and determine whether the combined machine-level opportunity is sufficient.

Do not let an initially disappointing individual optimization end the broader investigation prematurely. Equally, do not manufacture additive savings or a favorable conclusion.

Please continue the original assignment unchanged in all other respects.

#### What I'm most interested in from Ultra

I want to see whether it can establish a credible whole-constructor instruction budget.

For instance, if ordinary-group completion is expensive, descriptor updates are unnecessarily dependent, and finalization repeats work that could be avoided, there may be an opportunity to change several parts of the same computation coherently.

Alternatively, it might establish that these costs cannot be eliminated simultaneously, or that eliminating them increases memory traffic enough to cancel the benefit.

Either conclusion would be valuable.

And there's an important distinction between saying:

“I cannot justify expecting this optimization to save 6.6 ns per event.”

and saying:

“No plausible combination of transformations can remove enough critical-path work to make this constructor viable.”

Max established the former, not the latter.

Ultra has been tasked with investigating the latter.

So my recommendation is straightforward: send the correction, keep Ultra independent, and let it complete the investigation.

We have a genuinely useful NO-GO assessment for one candidate. We do not yet have a demonstrated execution-cost floor for compact30 construction.
```


```
I agree with the guardian here. In fact, our assessments have converged almost completely.

I'd use the guardian's correction verbatim. It's concise, technically specific, and doesn't introduce Astra Max's NO-GO conclusion into the independent Ultra investigation.

The guardian also makes one practical point I should have emphasized: don't interrupt Ultra if it's already generating its report. The correction can be supplied afterward, with a request to revisit any affected analysis.

What I particularly like is that the guardian preserves three distinctions:

- Max's NO-GO is a judgment about one bounded optimization, not a proof that the constructor has reached its practical execution-cost floor.
- Ultra's job is cumulative elimination, including investigating whether several changes can coexist without double-counting savings.
- The corrected block classification changes our priorities, directing attention away from tiny-block overhead and toward the large-block paths that dominate construction completion.

There's one further point I'd keep in mind when Ultra returns.

We shouldn't require Ultra to demonstrate a convincing runtime improvement purely from assembly inspection. That may be impossible without measurement, particularly when memory stalls and multicore contention are involved.

What I want Ultra to establish is whether there's enough demonstrably removable machine-level work to justify another bounded correctness-and-compilation prototype, with precise criteria for rejecting it if the compiler doesn't produce the expected improvement.

That's a demanding but achievable research objective.

And I would not let the 6.6 ns per-finish estimate become an artificial constraint. It's a useful calculation for Max's proposed optimization, not a requirement that every alternative must save time through ordinary finishes.

So I'd leave Ultra to complete its investigation. Send the factual correction afterward if necessary, preserve its independence, and then compare both reports critically.

We have a credible NO-GO for one idea, but we haven't established that the larger instruction-minimization campaign has run out of worthwhile opportunities.
```

## Report 227: the instruction-floor investigation's result, GO for one bounded prototype

The Ultra session's answer to report 225. Its verdict: **GO for one bounded
correctness-and-compilation prototype of epoch-tagged fast finalisation.** It also gives a
cumulative, non-overlapping budget of about **225 million instructions per primary-corpus
pass** across three compatible changes:

| change | conditional net reduction per pass |
|---|--:|
| epoch-tagged fast finish | 66.0 M |
| source and tile bounds checks removed | 87.6 M |
| hot descriptor pointers in separate arrays | 71.6 M |

These are uncompiled targets, not measured savings. Nothing was implemented or run.

**Checked here before filing** (an untimed reading of existing records):
- **Counters:** every structural count the report quotes matches the prototype's primary-corpus
  census (`corpora.jsonl` at `cd14fa9`) exactly. That covers scalar, fragment and bulk rows,
  the 610,521 runs, the finishes by class, the 51,701,707 normalised fields, the 3,654
  snapshots, the 127,249 epoch searches, the 49,633 RAW groups, 48,795 moved and 1,135 cycles.
- **Bounds checks:** the removable checks the report cites are present in the frozen listing
  (`ctor2-build-false.asm` at `cd14fa9`), with the instruction counts it gives:
  - 8 instructions at `33ae8–33b06`: the `rd` overflow check, the slice-end check and the
    second load's bound;
  - 5 at `33b44–33b56`: the run lookahead.

  They come from `rd`'s checked slice-to-array conversion, which the compiler kept although
  the caller has already bounded the tile.
- **Partition and imbalance:** its step-class decomposition under the registered
  `FULL = 899_000` partition, and its time-weighted imbalance factors, agree with the
  correction above.

**Relation to report 224.** The two agree on the facts:
- the 81-instruction fast finish;
- the 62-instruction refill;
- the pointer re-read;
- an achievable saving of about 40–50 instructions per fast finish.

They differ in scope:
- report 224 judged a fused finish on its own against the completion requirement, and found
  it insufficient;
- report 227 bases its GO on the cumulative budget, with the epoch-tagged finish as the first
  step.

The Ultra session saw report 224's verdict partway through (report 226).

Kept as relayed; its heading levels are adjusted, and the words are unchanged.

### Compact30: instruction-elimination investigation

**Recommendation: GO for one bounded correctness-and-compilation prototype of epoch-tagged fast finalization.**

The broader investigation found three substantial, compatible opportunities:

| Transformation | Conditional net reduction per primary-corpus pass |
|---|---:|
| Epoch-tagged fast finalization | **66.04 million instructions** |
| Remove redundant source/tile checks | **87.62 million instructions** |
| Separate hot descriptor pointer arrays | **71.59 million instructions** |
| **Combined, after explicit overhead allowances** | **225.25 million instructions** |

These are **uncompiled machine-code targets**, not measured savings. They do not establish a runtime improvement or an economic PASS.

The existing implementation is **not demonstrated to be near its practical instruction floor**. Substantial removable work exists outside the 12-instruction scalar loop. However, the available census cannot resolve an exact whole-constructor instruction total, and assembly inspection cannot establish how much of the eliminated work translates into synchronized completion time.

The corrected `FULL = 899_000` partition is used throughout. Large-block paths dominate the economic question.

No repository changes, Rust variants, compilation, benchmarks, i9 access, or new performance experiments were performed.

#### A. Evidence and provenance

##### Sources and listings inspected

All requested evidence was accessible.

| Evidence | Frozen location | Inspection scope |
|---|---|---|
| Production builder | [`src/bwt.rs`, e003a32](https://github.com/lmmx/mibz2/blob/e003a32/src/bwt.rs) | One-pass construction, validation, uniform/scalar paths, fallback |
| Constructor v1 | [`ctor.rs`, e003a32](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs) | Online offset construction and canonicalization |
| Constructor v2 | [`ctor2.rs`, 8beffe2](https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs) | Complete construction data flow |
| Frozen representation and oracle | [`c30walk.rs`, e003a32](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk.rs) | Layout, labels, classification, canonical output and padding |
| Prototype report | [Report, cd14fa9](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype.md) | Census definitions, correctness evidence, compilation discussion |
| Main construction assembly | [`ctor2-build-false.asm`](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-build-false.asm) | Scalar, scanning, fragments, bulk, transitions, initialization |
| Finalization assembly | [`ctor2-finish-false.asm`](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-finish-false.asm) | Common and exceptional paths |
| Refill assembly | [`ctor2-refill-false.asm`](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-refill-false.asm) | Successful, initial, exhausted and preset-RAW paths |
| Annotated excerpts | [`excerpts.asm`](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/excerpts.asm) | Cross-check against full listings |
| Raw prototype census | [`corpora.jsonl`](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/corpora.jsonl) | Natural primary-corpus record |
| Registered protocol | [Protocol, 1dc2edf](https://github.com/lmmx/mibz2/blob/1dc2edf/docs/journal/2026-11-03-compact30-construction-protocol-v2.md) | Definitions, validity gates, economic thresholds |
| Capture report | [Report, 603d8c9](https://github.com/lmmx/mibz2/blob/603d8c9/docs/journal/2026-11-03-compact30-construction-v2-capture.md) | Comparison with independent reconstruction |
| Complete archive | [Archive, e035556](https://github.com/lmmx/mibz2/blob/e035556/docs/journal/2026-11-03-compact30-construction-v2-capture/c30-construction-v2-20261103T222928Z.tar.gz) | Raw records and archived listings |
| Registered analyzer | [`bench/c30_construction.py`, fa21f33](https://github.com/lmmx/mibz2/blob/fa21f33/bench/c30_construction.py) | Including `FULL = 899_000` at line 81 |

The downloaded archive’s SHA-256 was:

```text
ebdb74549b39f83f07ae1f7d1c943916bac2ecb0be85868799aa30fe5e3d8af9
```

It exactly matches the supplied hash.

Within that archive, I independently analyzed:

- `c30-construction.jsonl`
- `c30-analysis.json`
- Manifest identity information
- `listing/listing.json`
- Production `listing/bwt-build.asm`
- `listing/ctor-canonicalise.asm`

The primary raw JSONL SHA-256 was:

```text
f25325eb2339b1a1385d598e8dd5a8a6d5e37c110c207ddcff18bae53f2e172b
```

It contains **75,087 records**, including **72,912 build records** and **434 per-block structural counter records**. The warm-W4 analysis selects exactly **17,360 measured builds**: 434 blocks × 20 rounds × two arms.

The archive’s identity report records matching normalized listings for all seven checked functions. I used the existing listings; I did not regenerate assembly.

##### Evidence limitations

The structural records do **not** include:

- Fragment-length histograms.
- Exact scalar-refill counts.
- Exact full-to-full refill counts.
- Bulk-emitter invocation counts and parity.
- Exact nonuniform-tile label-hole split counts.
- RAW reconstruction interval lengths.
- Retired instructions, decoded micro-ops, cache misses, stall cycles or execution-port utilization.

Consequently, the instruction ledger contains exact census-weighted components, bounded components, and an explicitly unresolved remainder. It is not a dynamic instruction trace.

Both construction captures remain **formally INVALID**. Their v1/v2 difference remains descriptive, not a controlled improvement result.

#### B. Baseline machine-code ledger

##### Counting conventions

Here, an “instruction” means an architectural x86 instruction in the inspected path. It does not mean one micro-op or one cycle.

A memory operand in a comparison or arithmetic instruction counts as a read. A memory read-modify-write counts as one read and one write. Explicit memory operands and implicit stack operations are distinguished.

Alignment NOPs, unlisted library internals and unresolved control-path frequencies are retained in the unresolved remainder rather than silently treated as zero.

All corpus totals below describe **one construction of the primary corpus**, not all experimental repetitions.

##### Exact structural reconciliation

The prototype census and the capture’s 434 per-block counters agree:

| Quantity | Count |
|---|---:|
| Blocks | 434 |
| Input rows | 207,006,515 |
| Scalar rows | 38,294,883 |
| Fragment rows | 14,892,632 |
| Bulk rows | 153,819,000 |
| Complete records emitted | 5,127,300 |
| Coalesced runs | 610,521 |
| Fast ordinary finishes | 1,647,416 |
| Non-fast ordinary finishes | 75,997 |
| Wide finishes | 30,949 |
| Crossing portions finished | 41,120 |
| Full normalizations | 1,723,371 |
| Partial normalizations | 42 |
| Fields normalized | 51,701,707 |
| RAW fields reconstructed | 1,485,808 |
| Preset-RAW fields restored | 557,338 |
| Epoch snapshots | 3,654 |
| Non-fast portions requiring epoch searches | 127,249 |
| RAW groups | 49,633 |
| RAW groups moved during canonicalization | 48,795 |
| Canonical permutation cycles | 1,135 |

Important identities:

```text
scalar + fragment + bulk = all rows
run_rows = fragment + bulk
30 × complete_groups = bulk rows
fin_fast + fin_ordinary = norm_full + norm_partial
```

Thus ordinary finishes and normalizations are **the same events viewed through different counters**. They must not be added as independent whole-function costs.

From the recorded block sizes:

```text
T = Σ floor(block_rows / 16) = 12,937,790 full tiles
U = run_rows / 16            = 10,544,477 uniform tiles
S = T − U                   =  2,393,313 nonuniform full tiles
scalar tail rows            =      1,875
```

##### Main-build instruction ledger

Addresses below refer to the frozen prototype listings unless identified as archive addresses.

| Cost center | Existing path and frequency | Instructions, millions | Principal memory/dependency characteristics |
|---|---|---:|---|
| Scalar append | `34350–3437a`, `343e0–34406`; 12/row | **459.539** | Four reads and two writes/row, counting pointer RMW; byte → descriptor address → write pointer → field |
| Uniformity classification and run scanning | `33ae3–33b27`, `33b30–33b6d`, exits | **221.504–222.732** | Exactly 25.876M source-qword reads; bounds and equality branches |
| Nonuniform tile setup and advancement | `33b80–33bc4`, `34340…`, `33a90–33add` | **98.126–114.879** | `41S + 7X`, where `X` is unrecorded split-tile count |
| Fragment filling, excluding surrounding run transitions | `33c9d…`, scalar `33d00–33d17`, vector `34180–341a5` | **43.437–114.277** | Length-dependent scalar/vector mix; histogram unavailable |
| Complete-record emitter core | `341d0–34233`, odd tail `3423a–34269` | **51.273–51.884** | 20 instructions/two records; 11/odd tail; five stores/record |
| Run entry | `33be5–33c4c`; 23/run | **14.042** | Rank reconstruction, whole-run bucket check, descriptor reads and spills |
| Run return and outer availability check | `34330`/`34439`, `34446–34463`, `33ad2…` | **7.937** | Mostly state reload and control |
| Snapshot rank loop | `33910–33948`; 1,792/epoch | **6.548** | Three descriptor reads and one snapshot store/character |
| Descriptor initialization and prefix loops | `333f0–334da` | **0.972** | Excludes the subsequent descriptor copy |
| **Disjoint main-build subtotal** | | **903.377–992.809** | Excludes the components below |

The fragment interval is deliberately broad. Its uncertainty is material, not rounding error.

For a fill of length `m`, the inspected path costs are:

```text
m < 8:                   29 + 7m
m ≥ 8, m divisible by 8: 43 + 9 floor(m/8)
m ≥ 8, other:            44 + 9 floor(m/8) + 7(m mod 8)
```

A run has at most a prefix and suffix fill. Combining that bound with the total fragment rows gives the table’s interval without inventing a length distribution.

##### The scalar loop

The second part-loop is:

```asm
343e0: movzbl (%r15),%esi
343e4: lea    (%rsi,%rsi,4),%rcx
343e8: lea    (%r8,%rcx,8),%r14
343ec: mov    (%r8,%rcx,8),%rax
343f0: cmp    0x8(%r8,%rcx,8),%rax
343f5: je     3440d
343f7: mov    %bx,(%rax)
343fa: addq   $0x2,(%r14)
343fe: inc    %r15
34401: inc    %ebx
34403: cmp    %r12,%r15
34406: jne    343e0
```

Across the corpus this accounts for:

- **459,538,596 instructions**
- **153,179,532 explicit reads**
- **76,589,766 explicit writes**

The four reads are the source byte, write pointer, end pointer, and pointer-RMW read. These are memory operations, not necessarily four cache misses.

V2 removed v1’s expensive header/RAW/span chain. Nevertheless, two address-generation instructions remain avoidable, and the pointer RMW remains a separate load-versus-instruction tradeoff.

##### Fast full finalization is exactly 81 instructions

This path is much more substantial than the 15-instruction normalization body alone:

| Inclusive range | Instructions | Work |
|---|---:|---|
| `61d30–61d93` | 29 | Frame, snapshots metadata, descriptor loads, fast test |
| `61e08–61e17` | 5 | `j0` calculation, RAW dispatch, spill |
| `61f1b–61fb8` | 39 | Endpoint recovery, span check, header, broadcast and normalization |
| `6231b–62329` | 8 | Epilogue |
| **Total** | **81** | Excludes caller’s `call` |

It executes:

- **17 explicit reads**
- **8 explicit writes**
- Six pushes and six pops, additionally

At most 42 fast finishes can be partial, because there are only 42 partial ordinary normalizations in total. Therefore:

```text
1,647,374 ≤ fast full finishes ≤ 1,647,416
```

The 81-instruction path contributes **133.437–133.441M instructions**.

Its most consequential serial chain is:

```text
W
 → snapshots Vec reference
 → Vec length/data
 → latest snapshot element
 → classification
```

The normalizer then requires the first field for its subtraction base.

##### Refill and RAW reconstruction

A successful nonempty ordinary `refill` executes **62 instructions**, excluding the called finish body but including the call instruction. Its total function size of 67 instructions is not the relevant path count.

Not every finish comes through `refill`: runs and the final sweep also finish portions.

Let `P = 1,795,482` be all portions. A run can nonempty-close at most one prefix, and final sweeps close at most `256 × 434` portions. Therefore nonempty scalar refills number at least:

```text
P − runs − 256 × blocks = 1,073,857
```

All successful scalar refills open new portions, so their total is at most `P`.

Accounting for empty initial refills and preset-RAW opening branches gives a wrapper contribution of approximately **66.579–111.402M instructions**.

The caller detours are additional. For example, `3437e–343ac` and `3440d–34437` execute roughly 9–10 instructions around the refill call. These remain in the unresolved remainder below.

The RAW reconstruction loops execute **16 instructions per field** on their non-transition iteration. Across 1,485,808 fields:

```text
16 × 1,485,808 = 23,772,928 instructions
```

Epoch transitions, allocation, dispatch, restoration and record replacement are additional.

##### What the whole-constructor budget establishes

Combining disjoint audited components gives:

```text
Icurrent = K + E

K ∈ [1.127166, 1.261425] billion instructions
```

Here `E` is an explicitly unresolved, nonnegative remainder containing:

| Remainder component | What is known |
|---|---|
| Scalar-refill caller detours | Approximately 9–10 instructions per relevant full-tile detour |
| Run room/close/open/control | Portions and runs are known; exact branch combinations are not |
| Bulk invocation setup | Formula `19 × invocations + 6 × paired invocations`; ≤15.263M instructions for this setup subset |
| Non-fast finish work outside the counted RAW loop | Endpoints, searches, ordinary normalization, dispatch and RAW setup |
| Epoch-search iterations | Portion count known; search-depth distribution absent |
| Final validation and closing sweep | 256 descriptors/block; nonempty distribution not fully classified |
| Allocation, zeroing and copies | Call sites inspected; library internals and exact invocation histories not traced |
| Canonicalization, padding and PARK | Small but nonzero |
| Alignment instructions and remaining scaffolding | Not credited as absent |

**The upper end of `K` is not an upper bound on the entire constructor.**

A conditional `E = 50–100M` would put the whole total around **1.18–1.36B instructions**. That is a sensitivity assumption, not an independently established interval. The absolute elimination budget is more defensible than a precise percentage of total work.

##### Memory work

Some useful logical field counts are exact:

| Operation | Logical bytes per corpus pass |
|---|---:|
| Temporary nonbulk field writes | 106.375 MB |
| Ordinary normalization reads | 103.403 MB |
| Ordinary normalization writes | 103.403 MB |
| RAW reconstruction field reads | 2.972 MB |
| Reconstructed RAW entry writes | 5.943 MB |
| Preset-RAW field restoration | 1.115 MB |

These are **explicit logical accesses**, not measured DRAM traffic.

For the main scalar loop, bulk emitter and fast finish, memory-operation counts are directly available from the assembly. Fragment operations can be bounded: their pattern reads are approximately **3.723–10.134M**, and field stores **1.862–9.340M**, excluding setup metadata accesses.

I do not claim a complete load/store total: doing so would conceal the same transition and histogram uncertainties represented by `E`.

##### Production comparison

Production’s successful one-pass builder writes a complete `u32` table entry:

```text
tt[destination] = L[destination] | source_row << 8
```

It maintains simple bucket cursors and limits. It avoids compact30’s:

- Group classification.
- Epoch recovery.
- Temporary-field normalization.
- RAW promotion and canonicalization.
- Thirty-row record transitions.

Production instead writes a larger table and reads `L[destination]` to supply the low byte. The archived production assembly also contains its own bounds and scalar-path overhead.

This explains structural differences. It does not provide a cycle attribution for the observed C/P ratio.

##### Comet Lake interpretation

Three distinctions matter:

1. Removing two LEAs shortens address preparation but does not remove descriptor loads.
2. Removing predictable compare/branch pairs reduces instruction work, but macro-fusion means instruction savings need not equal micro-op savings.
3. Removing the snapshot dependency chain and generic frame can matter differently from removing independent arithmetic.

Memory RMW instructions can represent more internal work than one register instruction. Conversely, independent loads can overlap. No latency, port occupancy, cache-miss rate or cycles-per-instruction value is inferred here.

#### C. Practical execution lower bound

##### Unavoidable representation and ordering work

For arbitrary accepted input, construction must establish the correct occurrence ordering of each character and emit every defined output byte.

The frozen oracle requires:

```text
φ(i) = 32 floor(i/30) + i mod 30
```

For an ordinary record:

```text
header = first_full_label << 8 | character
field[j] = full_label[j] − first_full_label
```

Classification must use the **full-label span**, with ordinary permitted through 65,533.

For RAW records, the constructor must produce canonical RAW indices, correct entries, `0xFFFF` record fields and zero RAW padding. Final partial-record padding and PARK also remain defined.

For this corpus, the final record and RAW arrays contain **448,002,944 logical bytes**, including PARK records. Their correct materialization is unavoidable. This is not a lower bound on DRAM transfers or CPU time.

##### Avoidable implementation overhead

The representation does not require:

- Stride-40 address multiplication on every scalar append.
- Reconstructing the opening epoch from snapshots when it was known at opening.
- Testing a span already bounded by a same-epoch invariant.
- Saving six callee-saved registers for a simple ordinary normalization.
- Nulling descriptor pointers immediately before overwriting them.
- Dividing the next rank by 30 when the next full group is exactly 64 bytes away.
- Rechecking source slice bounds after proving a complete 16-byte tile exists.
- Reloading an invariant bulk pattern for every pair of records.

These are concrete implementation or code-generation costs.

##### Temporary fields and normalization

Temporary writes followed by normalization are **not an information-theoretic requirement** of compact30. They are the current successful tradeoff.

The straightforward alternative—subtract the base online—looks unfavorable:

```text
2 added scalar instructions × 38,294,883 rows
    = 76.590M added instructions
```

That exceeds the approximately:

```text
18 × 1,723,371 = 31.021M instructions
```

in full-group broadcast plus normalization, before charging online wide handling or new dependencies.

Therefore the current deferred field strategy remains a reasonable practical floor candidate. It is not proven globally optimal.

##### Paths already relatively close to a floor

- **Bulk output:** already emits final records directly and avoids normalization. Remaining instruction/store improvements are smaller than the leading opportunities.
- **SIMD normalization arithmetic:** five subtraction chunks are already bounded work. The generic control surrounding them is considerably less economical.
- **Canonicalization:** necessary ordering is preserved, and only about 49,000 RAW groups participate. It cannot plausibly be the principal solution.
- **Tiny-block fixed work:** expensive per row, but economically small under the corrected schedule partition.

#### D. Cumulative instruction-elimination ledger

##### Core cumulative budget

The following table uses explicit acceptance allowances. These are requirements for a future compilation checkpoint, not observed code generation.

| Step | Gross removable work | Added-work allowance | Conditional net saving | Cumulative saving |
|---|---:|---:|---:|---:|
| Epoch-tagged fast finish | 67.542M | 1.481M slow dispatch + 0.020M epoch/block setup | **66.041M** | **66.041M** |
| Source/tile invariant cleanup | 88.622–88.633M | Up to 1.000M additional setup/control | **≥87.622M** | **≥153.663M** |
| Hot descriptor pointer arrays | 76.590M | Up to 5.000M additional non-row work | **≥71.590M** | **≥225.253M** |

The final residual budget is:

```text
Icore = Icurrent − 225.253M

      = [901.913, 1,036.172]M + E
```

This does not assume that `E` disappears.

These changes remove different instructions:

- Finish changes once-per-portion work.
- Scanner changes source/tile validation.
- Descriptor layout changes scalar address generation.

They can coexist. Their interactions must still be checked in the resulting assembly: changed layouts and register pressure can introduce costs elsewhere.

##### 1. Epoch-tagged fast finalization

**Existing locations:** `ctor2.rs` 343–367 and 397–495; finish ranges listed in section B.

**Equivalent computation:** record the first source epoch when an ordinary portion opens. If that tag equals the active epoch, every received label belongs to the same epoch.

Within one epoch:

```text
0 ≤ temporary label ≤ 65,533
```

Therefore a same-epoch ordinary candidate is necessarily ordinary. Its final-label load and span calculation are unnecessary.

**Removed:**

- Snapshot metadata chain on fast success.
- Last-label read and full-span reconstruction.
- Ordinary-path `j0` calculation.
- Generic descriptor fields not needed by the normalizer.
- Generic frame/spills, if the intended leaf compiles.

**Added:**

- One active-epoch tag in build state.
- One tag load per ordinary opening, replacing the immediate `NO_RAW` value.
- Slow-path dispatch.
- Epoch/block tag setup.

**Frequency:** at least 1,647,374 full fast finishes.

**Target:** ≤40 instructions across the complete fast call boundary, using the existing five-chunk normalizer.

**Memory budget:** fast success targets ≤10 explicit reads and ≤6 explicit writes, with no spills or callee-save stack traffic. After ordinary-opening loads, slow guards and setup allowances, the model still removes approximately **9.16M explicit reads** and **3.27M explicit writes**, plus approximately **9.88M push writes and 9.88M pop reads**.

The stack operations are already included in the instruction saving; they are not another instruction credit.

**Critical-worker relevance:** fast finishes are enriched on the observed slowest large blocks.

**Rejection:** retained snapshot chain, retained generic frame, added per-row work, incorrect opening epoch, or failure of the complete cost budget.

##### 2. Source and tile invariant cleanup

**Existing source:** `rd` at lines 98–100, tile processing at 557–576, scanner at 823–863.

**Existing assembly:**

- Fresh source loads: eight removable instructions at `33ae8–33b06`.
- Run lookahead: five at `33b44–33b56`.
- Nonuniform tiles: five redundant slice checks and two instructions testing an impossible phase state.

The caller already establishes:

```text
i + 16 ≤ epoch_end ≤ L.len()
L.len() ≤ MAX_N
0 ≤ source_phase < 30
```

Precisely bounded unaligned reads can use that proof. The loop-end check remains; malformed-count bucket checks remain.

Let:

- `H` be runs ending with a held nonmatching tile.
- `A = S + R − H` be fresh outer loads.
- `J = U − R + H` be lookahead loads.

Then:

```text
removed = 8A + 5J + 7S
        = 5T + 10S + 3(R − H)
```

At most one run per epoch can end because another full tile is unavailable:

```text
0 ≤ R − H ≤ 3,654
```

Hence:

```text
88,622,080 ≤ removed instructions ≤ 88,633,042
```

**Memory change:** no intended change in source reads or output writes.

**New work:** no algorithmic work is required. The ledger nevertheless reserves 1M instructions for replacement setup/control.

**Critical-worker relevance:** affects all large blocks, with slightly more eliminated tile work on nonuniform input. It offers a weaker direct imbalance argument than the scalar and finish changes.

**Rejection:** surviving equivalent checks, new tail overreads, changed run splitting or coverage, weakened validation, or added work exceeding the allowance.

##### 3. Separate hot descriptor pointer arrays

**Existing locations:** `Desc` at lines 233–240; scalar address LEAs at `34355/34359` and `343e4/343e8`.

The hot state can be laid out as:

```text
write[256]
end[256]
cold descriptors: w0, r0, lim, g, raw_or_epoch_tag
```

Two pointer arrays occupy 4 KiB. A 24-byte cold descriptor plus the two pointer arrays can retain the existing aggregate **10 KiB** descriptor footprint.

Direct scale-eight addressing removes the two LEAs used to form stride-40 addresses.

**Target:** scalar append **12 → 10 instructions**.

```text
2 × 38,294,883 = 76,589,766 instructions removed
```

**Memory change:** no intended per-row change. The write-pointer RMW remains.

**Added work:** initialization, snapshots, refill, run and finish addressing must be recosted. The conservative ledger allows up to 5M additional instructions outside the scalar loop. Any added metadata memory operations must also be reported explicitly; they cannot be hidden behind an unchanged byte footprint.

**Critical-worker relevance:** scalar rows are enriched on slow large blocks.

**Rejection:** >10 common-loop instructions, recurrent base reloads/spills, additional per-row memory accesses, expanded row-sized storage, or non-row overhead consuming the budget.

##### A competing scalar choice: remove the RMW read

The existing:

```asm
addq $2, [write_pointer]
```

can potentially become:

```asm
lea  2(old_pointer), new_pointer
mov  new_pointer, [write_pointer]
```

This reuses the pointer already loaded for the field store.

Its tradeoff is:

| Change across scalar rows | Amount |
|---|---:|
| Additional instructions | **+38.295M** |
| Explicit pointer reads removed | **−38.295M** |
| Pointer stores | Unchanged |

Combined with the descriptor layout change, the target becomes **11 instructions**, rather than 10. The net instruction saving versus baseline is then **38.295M**, not 76.590M.

The repeated-character store-to-load recurrence remains. No shorter latency or micro-op count is established. This is a genuine alternative between instruction and memory-operation budgets, and its savings must not be added to the ten-instruction variant.

##### Further opportunities examined

| Candidate | Concrete opportunity | Weighted scale and added work | Disposition |
|---|---|---|---|
| Interior full-group refill | Advance `w0` by 64, rank by 30 and group by one; avoid null/reset, reciprocal division and bitmap work | At least **991,575** eligible transitions by structural bounds; guards and fallback costs required | Promising secondary candidate; not in core total |
| Four overlapping SSE normalization chunks | Loads/stores at 0, 16, 32, 44 replace five chunks | **5.170M instructions**; one fewer load/store per full normalization | Additive if fast-finish target still uses five chunks |
| Fragment SIMD simplification | Replace two 64-bit loads/adds plus packing with one 128-bit load/add/store | Up to about **5.6M gross** before setup; alias-check removal offers additional bounded work | Smaller, histogram-dependent |
| Bulk pattern/register reuse | Hoist invariant pattern and retain header-bearing final vector | Pair-loop target 20 → 14; approximately **15.4M gross ceiling** before per-call setup and tails | Smaller; favors already efficient bulk paths |
| RAW interval construction | Hoist epoch/character prefix and bounds; optionally vectorize constant-epoch intervals | Current repeated loop contributes **23.773M**; net requires interval lengths and tail accounting | Concrete secondary work; no fixed net credit |
| Metadata initialization/copy removal | Initialize final descriptor/snapshot storage directly | Removes several MB of logical copying/zeroing; library instruction count unavailable | Small; overlaps descriptor-layout work |
| Canonicalization cleanup | Reduce rotation register copies or bookkeeping | Whole mechanism is only a few million instructions | Too small to lead |
| Avoid initial wide-RAW zeroing | Initialize only required padding before writing entries | At most about **0.217M** store-instruction reduction on wide groups | Too small for added unsafe complexity |

###### Interior refill bound

Starting with at least 1,073,857 nonempty scalar refills, conservatively subtract:

- 41,120 old preset-RAW portions.
- 41,120 possible next preset-RAW portions.
- 42 possible next partial ordinary portions.

This leaves:

```text
eligible full-to-full transitions ≥ 991,575
```

For a natural full interior group:

```text
next_w0 = old_w0 + 64
next_r0 = old_r0 + 30
next_g  = old_g + 1
```

If another 30 ranks remain within the same bucket, the next group cannot be crossing or forced RAW. It also cannot already have been wide-finalized.

The relevant existing costs are `6235f–6236a`, `6237f–6239c` and the bitmap/pointer reconstruction following them.

This is meaningful remaining work. However, a fused implementation must count the entire replacement transition. It cannot claim removal of finish loads or frame work already credited to the fast leaf.

###### Overlapping normalization: fewer operations, more bytes

Four 16-byte accesses at offsets:

```text
0, 16, 32, 44
```

cover bytes 0–59. Bytes 44–47 overlap. If all overlapping original values are loaded before stores, both copies subtract the same base and produce identical results.

This removes three instructions per full normalization:

```text
3 × 1,723,371 = 5,170,113
```

It also changes logical reads and writes from **60 to 64 bytes each**. Across the corpus that adds **6.893 MB in each direction**.

It is an instruction and memory-operation reduction, **not a byte-traffic reduction**.

###### RAW reconstruction

The current per-field loop repeatedly forms rank and destination index, checks epoch transitions and slot bounds, and rebuilds the epoch/character prefix.

A constant-epoch interval can hoist:

```text
H = epoch << 24 | character
entry = temporary_label << 8 | H
```

The larger alternative processes contiguous intervals bounded by the next snapshot, with SIMD widening inside sufficiently long intervals.

Repeated snapshots must be skipped correctly. Interval setup, short tails and empty epochs can consume the apparent saving. The aggregate field count alone does not justify booking a 10–20M net reduction.

##### Alternatives rejected or left unsupported

- **Online offset formation:** added scalar work exceeds the normalization instructions it straightforwardly removes.
- **Private scratch for all unfinished fields:** introduces comparable writes/rereads/copies; locality might improve, but instruction elimination is unproven.
- **Post-scan group-order finalization:** can reduce descriptor/call work, but needs pending metadata and changes epoch-recovery/cache behavior. It overlaps the fast-finish saving.
- **Extract scalar bytes from already loaded qwords:** removes byte loads but adds extraction and register-management work; no established net reduction.
- **Coalesce short runs inside scalar tiles:** requires an unrecorded short-run distribution and additional per-row comparison/control.
- **AVX2 normalization:** plausible, but an alternative to SSE normalization, with target-feature and surrounding-code implications. No extra credit is assigned.
- **Scheduling changes:** outside the fixed-constructor instruction budget and cannot be inferred from this assignment’s capture.

#### E. Critical-path and completion analysis

##### Registered descriptive figures independently reproduced

For each round:

```text
CPU = Σ(worker thread CPU durations) / corpus rows

completion =
    Σsteps(maximum build end − minimum build start)
```

The geometric mean of round C/P ratios and the log-scale t interval reproduce:

| Warm W4 measure | Geometric mean C/P | Lower 95% | Upper 95% |
|---|---:|---:|---:|
| CPU | **1.08619319** | 1.07728640 | **1.09517361** |
| Completion | **1.11436908** | 1.09225526 | **1.13693060** |

The capture remains INVALID.

##### Corrected full/short partition

The corpus contains:

- 197 blocks of exactly 900,000 rows.
- 33 near-full blocks of 899,361–899,998 rows.
- 204 genuinely tiny blocks of 1–307 rows, totaling 8,324 rows.

The registered threshold is **899,000**, so the first two categories are full.

The following are ratios of pooled sums across all 20 rounds, not registered geometric means:

| Step class | Steps/round | P completion ms/round | C completion ms/round | Completion C/P |
|---|---:|---:|---:|---:|
| All full, `rows ≥ 899,000` | 57 | 31.642004 | 35.149738 | 1.110857 |
| Mixed full/tiny | 1 | 0.508795 | 0.570458 | 1.121194 |
| Tiny only | 51 | 0.080215 | 0.166383 | 2.074205 |
| **All** | **109** | **32.231014** | **35.886579** | **1.113418** |

The additive decomposition is:

| Step class | Share of P completion | Contribution to overall C/P excess |
|---|---:|---:|
| All full | **98.1725%** | **0.108831** |
| Mixed | 1.5786% | 0.001913 |
| Tiny only | **0.2489%** | **0.002673** |
| Total | 100% | **0.113418** |

Tiny-only steps contribute **2.36% of the total completion penalty**. At the mixed step, the candidate’s slowest worker always handles a large block.

Therefore tiny-block initialization cannot be the primary route to the target.

##### Imbalance: the weighting matters

The quoted values reproduce as:

```text
Σ(step slowest worker wall duration)
─────────────────────────────────────
Σ(step mean active-worker wall duration)
```

They are:

- Production: **1.04670176**
- Candidate: **1.07331477**

They are **time-weighted aggregate factors**, not simple averages of per-step ratios.

The unweighted averages are instead:

- Production: 1.07394717
- Candidate: 1.07068727

Tiny steps dominate the unweighted statistic and reverse its ordering. The weighted definition is the relevant one for explaining aggregate completion arithmetic.

The decomposition describes how aggregate worker work and imbalance combine. It does not identify the source-level cause of the imbalance.

##### Bulk coverage and block cost

Using all 20 rounds, the 230 registered-full blocks show correlation:

```text
bulk share versus ratio of per-arm median CPU times:
r = −0.77275
```

| Bulk quartile, all registered-full blocks | Mean bulk share | Mean C/P CPU ratio |
|---|---:|---:|
| Q1 | 69.13% | 1.2004 |
| Q2 | 74.13% | 1.0954 |
| Q3 | 75.75% | 1.0732 |
| Q4 | 78.16% | 0.9913 |

The originally quoted approximately −0.74 and quartile values reproduce for exactly-900,000-row blocks using round-5-excluded medians. That exclusion is a **post hoc diagnostic**. The association also remains strong with all rounds included and the corrected full-block definition.

Correlation does not distinguish scalar instruction cost, finalization, memory stalls and other correlated characteristics.

##### Which blocks determine completion?

Across **1,140 all-full candidate steps**:

- The slowest worker has the lowest bulk share within its step **85.44%** of the time.
- It has the most scalar rows **75.96%** of the time.
- It has the most full normalizations **85.00%** of the time.

Ties count as matches.

Assignment-normalized quartile exposure is especially informative:

| Bulk quartile | Share of assignments | Share of slowest workers |
|---|---:|---:|
| Lowest | **25.00%** | **40.35%** |
| Highest | **25.44%** | **1.84%** |

Average structural work is also higher on the observed winners:

| Quantity | Average assigned large block | Average slowest large block |
|---|---:|---:|
| Scalar rows | 166,529 | 181,264 |
| Fragment rows | 64,732 | 70,370 |
| Full normalizations | 7,494 | 8,168 |
| Fast finishes | 7,164 | 7,830 |

Fast-finish exposure is **1.0929× greater** on current winners.

This supports the relevance of scalar, fragment and finish eliminations to completion. It does not prove the same percentage of elapsed-time reduction, and the identity of the slowest worker can change.

##### Largest recurring penalties

The table uses the difference between per-arm median step completions over **all 20 rounds**, retaining round 5. It is descriptive, not a registered estimator.

| Step | Candidate’s principal slow block | Median C−P completion |
|---|---:|---:|
| 51 | 284 | **191.50 µs** |
| 55 | 279 | **188.47 µs** |
| 6 | 60 or 56 | **169.07 µs** |
| 32 | 280 | **161.41 µs** |
| 31 | 273 | **139.79 µs** |

Examples:

| Block | Bulk share | Scalar rows | Fragment rows | Full normalizations |
|---|---:|---:|---:|---:|
| 284 | 61.05% | 258,471 | 92,040 | 11,454 |
| 279 | 61.43% | 260,476 | 86,662 | 11,332 |
| 280 | 63.98% | 249,232 | 74,978 | 10,579 |
| 60 | 64.73% | 250,160 | 67,240 | 10,318 |

**53 of 57 all-full steps** have positive median completion penalties. The disadvantage is widespread enough that optimizing only a few exceptional blocks would be insufficient.

The worker-to-block assignment is fixed. These records cannot separate intrinsic block cost from persistent core, worker or order effects.

##### Production round 5

The raw reconstruction confirms:

- Production warm-W4 CPU max/min: **1.07428**
- Production completion max/min: **1.21474**
- Both exceed G1’s unchanged **1.05** ceiling.

Production round-5 completion was **38.319619 ms**; other rounds ranged **31.545436–32.397737 ms**.

The anomaly concentrates on production worker 3. Relative to each block’s median in the other 19 rounds, worker 3 accumulates approximately **6.95 ms** excess CPU time. It affects many large blocks, not one isolated outlier.

CPU and wall durations both rise. The records do not establish the cause.

Post hoc removal of round 5 gives geometric ratios of approximately **1.08986 CPU** and **1.12487 completion**. These are diagnostics only. Round 5 remains in the formal INVALID result.

##### CPU time versus waiting

Aggregate thread-CPU/worker-wall ratios are:

- Production: 0.99848
- Candidate: 0.99866

This argues against large aggregate off-CPU gaps. It does **not** exclude memory-stall cycles or establish that every individual build has negligible off-CPU time.

#### F. Strongest first implementation candidate

##### Selected candidate: epoch-tagged ordinary fast finish

This is one bounded transformation. The scanner and descriptor-layout changes remain separate follow-ons.

Its appeal is not merely the 81→40 instruction target. It removes an existing serial metadata chain and generic frame work from a path disproportionately represented on slow large blocks.

##### Exact semantics

Inputs and outputs remain those of `Ctor2::build`:

- Same accepted inputs and malformed-count rejection.
- Same `rec` and `raw` lengths and bytes.
- Same canonical RAW order.
- Same headers, padding, PARK and forced-RAW behavior.
- Same walker.
- Same bulk, fragment and scalar row classification.

No construction work moves outside the builder.

##### State representation

Reuse the existing descriptor’s `raw: u32` word:

```text
preset RAW:
    raw index, high bit clear

ordinary candidate:
    0x80000000 | first_epoch << 24
```

Maintain:

```text
active_epoch_tag = 0x80000000 | active_epoch << 24
```

in build state.

The descriptor need not grow. Build state gains one small scalar, with any alignment consequence to be checked.

Epochs are in 0–15. Ordinary epoch bits occupy final-header positions 24–27; bit 31 is only the temporary discriminator. RAW indices cannot collide with it.

##### Why opening supplies the correct first epoch

There are two opening routes:

1. `refill` opens immediately before `one` writes.
2. `run` opens only while rows remain, then immediately performs a positive fill.

No epoch transition intervenes. Therefore the opening epoch is the first received row’s epoch.

This invariant must remain explicit. Eagerly opening a descriptor for future use would invalidate it.

##### Important epoch-boundary requirement

**Update the active tag when entering a source epoch, alongside snapshot creation. Do not derive it from `W.y` after source advancement.**

At an exact boundary or EOF, `W.y` may already point into the following epoch while the most recently consumed rows still belong to the preceding active epoch.

##### Fast-path condition

After the existing nonempty-portion discipline:

```text
descriptor.tag == active_epoch_tag
and
descriptor.write − descriptor.w0 == 60 bytes
```

These facts establish:

- Ordinary candidate.
- All rows in the active epoch.
- Thirty initialized fields.
- `w0` is record byte zero.
- Header address is `w0 + 60`.
- Span cannot exceed 65,533.

The header becomes:

```text
(tag & 0x7fffffff) | (first_u16 << 8) | character
```

The existing five-chunk modular subtraction produces the fields.

##### Illustrative machine-code shape

This is an **uncompiled target**, not a verified listing:

```asm
; descriptor addressing
lea     (c,c,4), a
lea     desc(W,a,8), d

; fast guards
mov     tag(d), tag
cmp     active_epoch_tag(W), tag
jne     slow
mov     w0(d), p
mov     write(d), len
sub     p, len
cmp     $60, len
jne     slow

; first field and broadcast
movzwl  (p), u0
movd    u0, xmm0
pshuflw $0, xmm0, xmm0
pshufd  $0x44, xmm0, xmm1

; exact ordinary header
shl     $8, u0
and     $0x7fffffff, tag
or      tag, u0
or      c, u0
mov     u0, 60(p)

; existing five load/subtract/store triples
; offsets 0,16,32,48,56; widths 16,16,16,8,4

ret
```

This illustrates **35 instructions**. The acceptance budget is **≤40**, allowing modest code-generation variation.

The slow finalizer must be separated sufficiently that its register needs do not impose the existing generic frame on the fast route. A tail transfer is desirable. Merely adding a branch inside the large current function may fail this requirement.

##### Slow and exceptional paths

- Old-epoch ordinary portions retain exact full-label classification.
- Their first epoch is available from the tag.
- Their last epoch still requires correct recovery; it must not be assumed current.
- Repeated snapshots for absent characters retain the “greatest snapshot ≤ rank” rule.
- Preset RAW retains its actual RAW index and reconstruction.
- Wide promotion, bitmap updates and canonicalization remain unchanged.
- Partial ordinary groups use bounded partial normalization.

No credit is assigned to eliminating slow-path first-epoch searches. That potential saving remains outside the conservative first-candidate budget.

##### Workspace and initialization

- Reuse the existing descriptor word.
- Retain snapshots and bitmap.
- Add one active-epoch scalar.
- No new allocation proportional to rows or groups.
- No successor/F table.
- No second scan of `L`.

Empty descriptors must remain excluded from finishing. The initial tag value cannot become permission to read fields.

##### Expected path effects

| Path | Intended effect |
|---|---|
| Scalar append | Identical |
| Fragment append | Identical |
| Bulk emitter | Identical |
| Fast full ordinary finish | Specialized leaf |
| Non-fast ordinary finish | Correct fallback; optionally known first epoch |
| Wide | Correct fallback and unchanged final bytes |
| Crossing/forced RAW | Correct fallback |
| Partial last group | Bounded fallback |
| Tiny blocks | Limited benefit |
| Low-bulk large blocks | More exposure to the optimized finish path |

##### Untimed rejection conditions

Reject or return for redesign if any of the following occurs:

1. **Output differs:** any `rec` or `raw` byte, ordering, header, padding or PARK mismatch.
2. **Epoch proof fails:** opening can precede first append across an epoch transition.
3. **Boundary convention fails:** active epoch is taken from an already-advanced source coordinate.
4. **Unsafe access appears:** a full-group load reads uninitialized padding or header bytes.
5. **Classification changes:** exact 65,533/65,534 boundaries or far-wide cases differ.
6. **Repeated snapshots fail:** absent-character epochs recover the wrong full labels.
7. **Malformed counts fail:** bucket overflow can write out of bounds or publish incomplete arrays.
8. **Hot code misses budget:** >40 instructions across guards, wrappers and added caller work.
9. **Memory budget misses:** >10 explicit reads or >6 explicit writes on fast success, or generic stack saves/spills remain.
10. **Removed dependency returns:** snapshots metadata or final-label span classification remains on fast success.
11. **Opening work expands:** tag assignment no longer replaces the existing assignment one-for-one without recosting.
12. **Recurrent row work appears:** scalar or fragment paths acquire tag maintenance.
13. **Coverage changes:** scalar/fragment/bulk counts or epoch run splitting differ on identical inputs.
14. **Workspace expands materially:** a new table-sized intermediate appears.
15. **Cold/setup charges exceed budget:** slow dispatch and epoch/block maintenance invalidate the net ledger.

A source-level simplification alone is insufficient to pass this checkpoint.

#### G. Remaining execution-cost floor

##### Conditional whole-work sensitivity

Using the conservative **225.253M** core saving:

| Assumed current whole-constructor instruction work | Conditional reduction | Residual work |
|---|---:|---:|
| 1.20B | 18.8% | 0.975B |
| 1.35B | 16.7% | 1.125B |
| 1.50B | 15.0% | 1.275B |

These are **instruction-budget sensitivities**, not fitted totals or runtime forecasts. The unresolved remainder determines which denominator is appropriate.

With the independently additive overlapping SSE normalizer, the conservative total could reach approximately **230.42M instructions**, provided its code-generation and ordering conditions hold.

Interior refill, fragment, bulk and RAW improvements offer additional smaller opportunities. Their setup, interval distributions and interactions prevent an honest fixed net sum at this stage.

##### What remains after the core changes

The core portfolio does not remove:

- Most scalar descriptor and field memory operations.
- The repeated-character cursor store-to-load recurrence.
- Required source classification and run coalescing.
- Temporary field writes and ordinary normalization.
- Most fragment filling.
- Final bulk record stores.
- General boundary transitions.
- RAW reconstruction, canonical ordering and final padding.

The scalar body alone would still contribute approximately **382.95M instructions** at the ten-instruction target. Fragment uncertainty remains large. The successful deferred representation still rereads and rewrites roughly 103 MB of ordinary fields per corpus pass.

Thus a substantial reduction in avoidable control work can coexist with a sizeable residual field/metadata workload.

##### Economic headroom

The existing descriptive point estimates require approximately:

- **1.75%** less CPU cost to reach 1.06714.
- **8.47%** less completion time to reach 1.02.

Proportional scaling of the archived upper bounds requires approximately:

- **2.56%** less CPU cost.
- **10.29%** less completion time.

These remain sensitivity calculations, not prospective confidence intervals.

The instruction opportunity is large enough to make a roughly 10–15% completion improvement a worthwhile hypothesis to investigate. It is not enough to assert that improvement.

The outcome could be limited by:

- Work that is already overlapped.
- Memory stalls unaffected by removed checks.
- Register pressure and compiler reshaping.
- Unchanged cursor dependencies.
- Changed worker winners and imbalance.
- Baseline instability and variance in a later valid experiment.

##### Stopping assessment

The investigation reached substantial opportunities beyond ordinary completion and continued through the smaller cost centers.

The remaining proposals now fall into three groups:

1. **Concrete but smaller:** interior refill, fragment SIMD, bulk register reuse, RAW prefix/bounds cleanup.
2. **Overlapping alternatives:** fused finish/refill, delayed record-order finishing, alternative scalar RMW handling, wider SIMD.
3. **Unsupported without more structural evidence:** short-run coalescing, RAW interval vectorization payoff, scratch-locality redesign.

Canonicalization and tiny-block initialization are too small to determine the economics directly. Moving normalization into scalar append currently loses the instruction budget. No defensible global optimum or runtime floor follows.

#### H. GO / NO-GO

**GO — submit epoch-tagged fast finalization as the strongest first bounded correctness-and-compilation candidate.**

The justification is concrete:

- The frozen common finish executes 81 instructions.
- A same-epoch invariant makes part of its computation unnecessary.
- Existing descriptor storage can retain the information needed to remove a serial snapshot lookup.
- A complete, falsifiable ≤40-instruction target is specified.
- Added opening, fallback and setup work is accounted for.
- The opportunity reaches the large, low-bulk blocks that frequently determine completion.
- Independent scanner and descriptor-layout opportunities raise the conservative cumulative budget to approximately **225 million instructions per corpus pass**.

This does **not** establish economic viability, authorize implementation, or recommend a performance capture. The next decision should depend on byte-exact correctness and the actual compiled whole-path ledger.

The evidence supports continued bounded engineering scrutiny. It does not support declaring compact30 v2’s practical execution-cost floor exhausted.

## Report 228: the guardian's review of report 227, and the adversarial audit it commissions

**What report 228 decides:**
- **GO for independent validation of the instruction budget:** one fresh, adversarial review of
  report 227, before any implementation.
- The guardian is provisionally supportive of a bounded prototype strategy. No implementation,
  benchmark, i9 use or protocol change is authorised.
- The v2 capture stays formally INVALID, and the economic thresholds are unchanged.

**The three challenges it records:**
- the 225 M figure depends on compiler behaviour, including the 5 M allowance for the
  descriptor-array change;
- removing bounds checks needs a complete memory-safety proof, short tiles and epoch boundaries
  included;
- the first proposed prototype covers only about 66 M of the 225 M.

The audit's request is in the text below, under "Send this to a fresh Astra Max chat". It goes
with the full text of report 227, which is above in this file.

Kept as relayed; the relay lost its formatting, and the words are unchanged.

Guardian review: Astra Ultra has found a credible way forward

Ultra recommends GO for a bounded prototype

This is a substantially more encouraging result than Max’s earlier NO-GO. More importantly, it’s the kind of result you wanted: a systematic analysis of actual machine instructions, with a cumulative savings budget rather than another collection of speculative algorithms.

Ultra identifies three potentially compatible improvements:

Conditional net instruction savings per primary-corpus construction

Remove redundant source/tile checks

87.62M

Separate hot descriptor pointer arrays

71.59M

Epoch-tagged fast finalization

66.04M

Combined conditional saving

225.25M

These are theoretical instruction-removal targets after stated overhead allowances—not compiled results, retired-instruction measurements or predicted speedups.

That’s potentially around 15–19% of the constructor’s instruction work, depending on the unresolved total. It is enough to make a serious performance hypothesis worth investigating, although not to establish a 10–15% runtime improvement.

Why Max and Ultra disagree

They aren’t necessarily contradicting each other.

Max examined the strongest single optimization it could justify and found insufficient evidence that it would save enough completion time.

Ultra looked for multiple non-overlapping sources of unnecessary machine work. Its most important discovery is that ordinary finalization is not the only remaining opportunity: scanner bounds checks and descriptor addressing could collectively offer greater instruction savings than finalization itself.

So Max’s NO-GO on one change does not refute Ultra’s broader case.

The original v2 capture remains formally INVALID. Neither analysis changes that.

What I’ve checked

I compared Ultra’s proposed transformations with the frozen v2 Rust source and assembly. The opportunities are grounded in real compiled operations:

Opportunity	Existing cost	Proposed target
Fast ordinary finalization	81 instructions on the common full-group path	At most 40
Source/tile processing	Repeated bounds-check and control instructions	Remove redundant checks where bounds are already proven
Scalar descriptor addressing	Two address-generation instructions per scalar row	12 → 10 instructions per iteration

The cumulative arithmetic is internally consistent. Ultra has also accounted for some added work rather than crediting every removed instruction as a free saving.

Its analysis of the slowest workers is especially useful: in 85.44% of the 1,140 registered all-full candidate steps, the slowest worker also had the lowest bulk-emitter coverage in that step. That gives us a concrete reason to target scalar and finalization work rather than polishing already-efficient bulk emission.

Three things I’d challenge before implementation

First, the 225-million figure is conditional on compiler behavior. The descriptor-array change, for example, saves two scalar instructions only if reorganizing the descriptors doesn’t add expensive operations elsewhere. Ultra permits 5 million additional non-row instructions; that allowance needs independent scrutiny.

Second, removing bounds checks must preserve Rust memory safety. The existing scan establishes useful source bounds, but any use of unchecked indexing or raw-pointer reads must have a complete proof, including short tiles and epoch boundaries.

Third, the first proposed prototype addresses only about 66 million of the 225 million instructions. By itself, it doesn’t establish enough headroom to overturn Max’s earlier economic concerns. Ultra’s stronger case is the combined, compatible reductions, not just epoch-tagged finalization.

These are reasons to scrutinize the proposal, not reject it.

Guardian disposition

GO for independent validation of Ultra’s instruction budget.

I’m provisionally supportive of the bounded prototype strategy, but I would follow the review sequence we discussed: one independent adversarial Astra Max assessment before asking Claude to implement anything.

We have abundant reasoning capacity. This is exactly where to spend it.

Send this to a fresh Astra Max chat

Attach Ultra’s complete report, not merely its summary.

Independent adversarial audit: compact30 instruction-elimination budget

Please independently and critically review the attached Astra Ultra report.

This is a machine-code accounting and design-falsification assignment, not another algorithm-ideation exercise.

Ultra proposes three compatible transformations with conditional net savings of approximately:

* 66.04 million instructions: epoch-tagged ordinary fast finalization.
* 87.62 million: source/tile bounds-check elimination.
* 71.59 million: hot descriptor-pointer arrays.

Total: approximately 225.25 million instructions per construction of the primary corpus.

Your objective is to determine whether this cumulative budget is defensible.

Independently inspect the frozen Rust source, archived assembly, structural census and raw capture evidence wherever available. Do not simply endorse or reject Ultra’s conclusions.

Prioritize these questions:

1. Are the path counts, execution frequencies and 225-million-instruction arithmetic correct? Are any operations counted twice, omitted or credited to an execution path where they cannot actually disappear?
2. Are the three transformations genuinely compatible? In particular, could separating descriptor storage increase refill, finish, snapshot or run-handling work beyond Ultra’s 5-million-instruction allowance?
3. Is removing approximately 87.6 million source/tile instructions realistic for rustc 1.97.0? Which checks are demonstrably redundant, what safe bounds invariants justify removing them, and how much of the proposed saving depends on unsafe accesses?
4. Is the epoch-tagged ordinary fast path semantically correct across all epoch boundaries, deferred completion, RAW and forced-RAW configurations, malformed counts, reused arenas and partial groups? Can the complete hot path plausibly compile to 40 instructions or fewer without hidden caller overhead, spills or new dependencies?
5. Are the proposed reductions likely to matter to four-worker completion, especially on low-bulk large blocks? Distinguish reduced architectural instructions from reduced micro-ops, dependency latency and actual cycles.
6. Is Ultra’s choice of epoch-tagged finalization as the FIRST prototype justified, or would testing another transformation first offer stronger instruction savings with less implementation or safety risk?
7. What parts of the practical execution-cost floor remain unaccounted for? Does the unresolved instruction remainder materially undermine the GO recommendation?

Do not generate a fresh collection of algorithmic ideas. Challenge the proposed eliminations and their accounting.

Return a clear verdict: ACCEPT, CONDITIONAL ACCEPT or REJECT.

For each material concern, identify the precise source or assembly location, the quantitative impact on the cumulative budget, and the evidence required to resolve it.

If acceptable, recommend the best order for bounded, untimed correctness-and-compilation prototypes. Define the static rejection conditions before any benchmark.

The constructor-v2 performance capture remains formally INVALID. The economic thresholds remain unchanged. No repository changes, implementation, new timing, i9 use or performance experiment is authorized.

A rigorous negative result is welcome. The goal is to determine whether Ultra has found a defensible path to substantial machine-code elimination, not to validate its optimism.

What this means for the project

We now have a much stronger reason to continue examining v2 than we had yesterday.

Max found one promising change but couldn’t defend its economic upside. Ultra has identified a portfolio of large, specific, potentially compatible instruction reductions and supported it with a detailed workload analysis.

That’s a meaningful advance in our understanding of the constructor, even though no runtime improvement has yet been demonstrated.

I’d ask Claude to archive Ultra’s report as research evidence, without starting implementation. After the independent review, we’ll select the most defensible first compilation target.

We haven’t established the execution-cost floor. We’ve found concrete evidence that substantial avoidable machine-code work may still remain. That’s exactly what we sent Ultra to investigate.

## Report 229: the researcher's review of report 227

The researcher supports a bounded, untimed prototype of the **epoch-tagged fast finaliser
alone**, leaving the scanner and the descriptor layout unchanged. The other two changes would
be later, separate steps.

Its acceptance criteria cover the whole fast call boundary:
- at most 40 instructions on the common success path;
- no snapshot lookup and no full-span classification on that path;
- no generic register-save frame and no new spills;
- no added per-row work;
- a revised ledger that includes all opening, fallback and setup costs.

It adds two audit obligations:
- every reader of the descriptor's `raw` word must handle the proposed epoch-tag encoding,
  with its high-bit discriminator;
- the recorded opening epoch must be the epoch of the portion's first appended row, not of a
  source cursor that has since advanced.

**Sequencing.** This differs from report 228, which asks for an independent adversarial audit
of report 227 *before* any implementation. Report 228 is the current authority: no
implementation is authorised until the guardian decides after that audit.

Kept as relayed; the words are unchanged.

I’ve read the complete Ultra report. My assessment remains GO for a bounded correctness-and-compilation prototype, with no performance capture authorized.

The most consequential result is that Ultra identified three distinct sources of potentially removable work, rather than relying entirely on the ordinary-group finalizer.

Conditional instruction savings per primary-corpus construction

Total: 225.25 million instructions, conditional on meeting the proposed assembly targets and added-work allowances.

That is potentially substantial relative to Ultra’s estimated whole-constructor instruction budget. It also explains why Astra Max’s earlier NO-GO does not settle the question: Max evaluated one optimization, while Ultra identified additional high-frequency work that may be eliminated independently.

My main qualification

I would not yet treat the full 225-million-instruction budget as demonstrated headroom.

The epoch-tagged finalizer has a fairly concrete correctness argument and an explicit target of 81 to at most 40 instructions. The source/tile cleanup is also grounded in identifiable bounds checks.

The descriptor-array proposal is more uncertain. Changing the layout may eliminate two address-generation instructions per scalar row, but it also changes how refill, finish, run handling and snapshots access descriptors. The assumed allowance of five million extra instructions elsewhere needs to survive actual compilation.

More generally, the proposed savings are architectural-instruction counts, not cycle savings. Eliminating a compare and branch, two address calculations, or a serial metadata load can have quite different performance consequences.

One detail I would insist Claude checks

Ultra proposes encoding an ordinary descriptor’s opening epoch in its existing raw field, using the high bit as a discriminator.

That requires a careful audit of every place that currently interprets raw as either NO_RAW or a RAW index. The fast path may be correct while an exceptional fallback silently misinterprets the new encoding.

The other essential invariant is that a descriptor’s recorded opening epoch must correspond to its first appended successor, not the epoch of a subsequently advanced source cursor.

Both obligations are manageable, but neither should be accepted merely because the ordinary fast path produces correct output on the primary corpus.

What I’d authorize next

I’d ask Claude to implement only the epoch-tagged fast finalizer, leaving the scanner and descriptor layout unchanged.

The prototype should establish byte-exact correctness against the frozen oracle, exercise epoch boundaries, wide spans, crossing groups, forced RAW, malformed counts and allocation reuse, and provide annotated release disassembly.

Its machine-code acceptance criteria should include the complete fast call boundary, not merely the normalization body:

* At most 40 instructions on the common fast-success path.
* No snapshot lookup or full-span classification on that path.
* No generic callee-save frame or new spills.
* No additional per-row work in scalar or fragment append.
* All added opening, fallback and setup costs included in a revised instruction ledger.

If those conditions pass, the result would justify considering the other two eliminations as separate, bounded follow-ons. It would not, by itself, demonstrate that the constructor meets the economic target.

The broader significance

Ultra’s most useful contribution is not its GO verdict. It’s that we now have a concrete picture of how much unnecessary control work may remain in v2, and where that work occurs.

The completion analysis is especially relevant: the slowest worker in a full-block step usually has lower bulk coverage and more scalar and finalization work. That gives these optimizations a plausible route to improving synchronized completion, rather than merely reducing instructions in an already efficient path.

The original traversal result remains VALID ADVANCE. Both construction captures remain INVALID. Whole-decoder performance remains unestablished.

My recommendation: proceed with the first untimed prototype, preserve Ultra’s cumulative ledger as a research hypothesis, and require actual compiler output before crediting any of the proposed savings.

## Report 230: the guardian's addendum to the adversarial audit

The guardian keeps the order set in report 228: the independent audit first, then
implementation. It adds the researcher's two correctness obligations (report 229) and the
researcher's acceptance conditions to the audit request.

**Current decision:**
- research direction: GO;
- implementation: awaiting the independent audit;
- performance capture: not authorised.

If the audit passes, the expected instruction is narrow: implement only the epoch-tagged
finalisation, prove byte-exact correctness, and inspect the whole compiled fast path against
the 40-instruction target. The scanner-check and descriptor-array changes stay separate, so
each saving can be seen to survive compilation before they are combined.

Kept as relayed; the words are unchanged.

Claude has handled this correctly. I would keep the independent Astra Max audit before implementation, but make sure it includes the researcher’s two additional correctness obligations.

The researcher and I aren’t disagreeing about the technical direction. We both favor epoch-tagged fast finalization as the first bounded prototype. The difference is simply whether to spend one more reasoning pass checking Ultra’s assumptions before Claude writes code.

Given your available Astra capacity, I think that extra audit is worthwhile. In particular, the proposed reuse of raw: u32 is important enough to examine independently.

Updated instruction for the Astra Max audit

If you haven’t sent the audit request yet, use the complete prompt I gave you previously and append this:

Additional audit requirements from the independent researcher

The researcher has independently reviewed Ultra’s report and provisionally supports the epoch-tagged fast finalizer as the first bounded correctness-and-compilation prototype.

Before endorsing implementation, please examine two additional correctness obligations in the frozen ctor2.rs at 8beffe2.

1. Descriptor raw: u32 encoding

Ultra proposes reusing the existing descriptor raw field to store an ordinary portion’s opening epoch, distinguished from provisional RAW indices by its high bit.

Trace every write and read of this field, including open, finish, write_entries, preset-RAW handling, descriptor transitions and exceptional fallback paths.

Determine whether the proposed encoding preserves all existing invariants. In particular, check that a tagged ordinary descriptor can never be mistaken for a RAW index, that forced-RAW behavior remains correct, and that the generic finalizer correctly handles portions spanning multiple epochs.

Identify any required changes to predicates, dispatch or decoding. Do not assume replacing raw == NO_RAW with a high-bit test is sufficient without checking every use.

2. Opening-epoch correctness

Prove that the stored opening epoch always belongs to the portion’s first appended successor.

Inspect both scalar/refill and run/open paths, including source-epoch transitions, deferred finalization, exact epoch boundaries and end-of-input.

The active epoch must be updated alongside snapshot creation, not inferred from a source coordinate already advanced into the next epoch.

Identify any counterexample or invariant that would make the proposed optimization unsafe or produce incorrect output.

3. Full-path accounting

The researcher’s recommended acceptance conditions are:

* At most 40 instructions across the complete fast-success call boundary.
* No snapshot lookup or full-span classification on that path.
* No generic callee-save frame or new spills.
* No additional per-row scalar or fragment work.
* All opening, fallback, exceptional-path and setup costs included in the revised instruction ledger.

Please determine whether these conditions are realistic and sufficient, and whether any hidden caller-side or descriptor-transition costs have been omitted.

Keep the audit independent. The researcher’s support for the prototype is not evidence that Ultra’s implementation design is correct.

No implementation, repository modification or performance measurement is authorized.

My current decision

Research direction: GO

Implementation: awaiting independent audit

Performance capture: not authorized

There is one point I’d especially like the independent reviewer to challenge: Ultra’s proposed ordinary tag must coexist safely with NO_RAW, existing RAW indices and the exceptional finalization paths. A fast path that works perfectly for common groups but misclassifies a rare wide or crossing group would be unacceptable.

If the audit passes, I expect the next instruction to Claude will be narrowly scoped: implement only epoch-tagged finalization, prove byte-exact correctness, and inspect the complete compiled fast path against the 40-instruction target.

The other two optimizations—scanner-check elimination and descriptor-array restructuring—should remain separate. That lets us see whether each proposed instruction reduction actually survives compilation before combining them.

So I wouldn’t change course or rush into code. We’re one focused independent review away from a well-specified prototype, and the research is progressing in precisely the instruction-minimization direction you wanted.
