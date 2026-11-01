# External research report 138: the new researcher's review: a packed lane token for FOR-14, and a measurement-boundary defect in the walker ladder (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01, with reports 136–139 together.
- **Source:** a new independent researcher session (ChatGPT). It refers to an attached PDF memo (`for14_independent_research_memo.pdf`) that was not relayed; only this summary was.
- **In reply to:** FOR-14 stage 1 at 9175a11 and the walker ladder at e46506b.

**Evidence cutoff:** 1 November 2026, after the BMI2 code-generation test and before the FOR-14 i9 confirmation's result reached it. Source, recorded-disassembly and literature review: nothing modified, compiled or run. Its instruction latencies are published Skylake/Coffee Lake figures (Agner Fog, uops.info), not i9 measurements.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
**The BMI2 result closes the `shrx`-only rescue, but it does not exhaust the removable cost in FOR-14.** I found a specific same-format change that can eliminate one lane load and one lane store per ordinary step. Combined with simpler record addressing and a deliberately chosen multiply instruction, it gives a plausible **28-instruction assembly target**, compared with the current 37. That is an uncompiled target—not a speed prediction—and it still needs to earn its way back into consideration.

**PR #131 has a more immediate problem: its PMU measurements include repeated output hashing that its internal timer excludes.** That must be corrected before the ladder can support its proposed conclusions about execution cost or bookkeeping.

[for14_independent_research_memo.pdf](https://chatgpt.com/library?file_id=libfile_4bc014a942688191b41a5492c0580422&account_id=personal)

The memo contains the complete review, pinned references, implementation invariants and stopping criteria. The substantive findings follow below. I performed source, recorded-disassembly and literature review; I did not modify the repository, compile a candidate or run a capture.

## 1. Evidence and current hot-path reconstruction

I reviewed FOR-14 at [`9175a112`](https://github.com/lmmx/mibz2/blob/9175a1122aac2b8031de01336353e2971984b55a/docs/journal/2026-11-01-for14-stage1.md) and the prepared ladder at [`e46506be`](https://github.com/lmmx/mibz2/blob/e46506be6871076781734a6abf6c821c604dff80/docs/journal/2026-11-01-walker-ladder-protocol.md). The original FOR-14 confirmation remains separately pinned to `7d7962fce96b4009a30ee85fa935a0a20bfa0607`.

The current source performs this ordinary lookup:

```text
slot   = row & 31
at     = 14 * slot
record = records + 64 * (row >> 5)

header = load_u64(record + 56)
offset = (load_unaligned_u32(record + (at >> 3))
          >> (at & 7)) & 0x3fff

if offset != 0x3fff:
    byte       = lane.b
    lane.b     = (header >> 20) as u8
    lane.row   = (header & 0xfffff) + offset
    prefetch(record_for(lane.row))
    store_output(byte)
```

The header and packed-word loads are independently addressed from the current row. A RAW row, marked segment start or parked lane enters the sentinel path. RAW navigation takes an extra lane turn while its separately prefetched entry becomes available. These are properties of the [actual implementation](https://github.com/lmmx/mibz2/blob/9175a1122aac2b8031de01336353e2971984b55a/examples/coop/compact.rs), rather than assumptions about generic packed integers.

Here is my decomposition of the [recorded BMI2 ordinary path](https://github.com/lmmx/mibz2/blob/9175a1122aac2b8031de01336353e2971984b55a/docs/journal/2026-11-01-for14-stage1/disassembly-bmi2.md):

| Component | Static instructions |
|---|---:|
| Lane-loop bookkeeping | 6 |
| Load lane row | 1 |
| Form current record address | 4 |
| Load header | 1 |
| Slot calculation, packed address and extraction | 12 |
| Sentinel compare and branch | 2 |
| Carried byte and successor-state update | 7 |
| Next-record mask and prefetch | 2 |
| Output-base load and output store | 2 |
| **Total** | **37** |

The memory operations are:

| Path | Data loads | Stores |
|---|---:|---:|
| Shipped `o` | 3 | 2 |
| Current FOR-14 | 5 | 3 |
| Proposed packed lane state | **4** | **2** |

The last row is a derivation, not an observed binary. The existing counts come from the [archived disassembly](https://github.com/lmmx/mibz2/blob/9175a1122aac2b8031de01336353e2971984b55a/docs/journal/2026-11-01-for14-stage1/disassembly.md).

Two distinctions matter.

**The carried-byte traffic is explicit state traffic, not a demonstrated spill.** The load from `lane+8` and store back to `lane+8` directly implement `LaneH.b`. Asking LLVM to “keep it in a register” across 32 dynamically indexed lanes does not remove the underlying state requirement. The separate `base[k]` load is also intentional.

**The sentinel compare/branch is not automatically two additional instructions relative to production.** Production already has a start-flag test and branch. The dominant difference is the complete address/extraction/state-update sequence, not the mere existence of a conditional branch.

The existing unaligned load also has favorable geometry. For slots 0–31:

\[
a=\lfloor14i/8\rfloor\leq54,\qquad s=(14i)\bmod8\in\{0,2,4,6\}.
\]

A four-byte read ends no later than byte 57, inside the initialized record. There is no inherent cache-line split to eliminate. A two-byte read would be insufficient when the shift is four or six.

## 2. What the i9 instruction set actually offers

The relevant evidence is published Skylake/Coffee Lake instruction measurements, used as the appropriate family model for Comet Lake. These are not new measurements on your i9.

| Instruction | Executed µops | Relevant latency | Reciprocal throughput | Ports |
|---|---:|---:|---:|---|
| `SHRX r,r,r` | 1 | 1 cycle | 0.5 cycles | 0 or 6 |
| `SHRX r,m,r` | 2 | 6 cycles address-to-result in the measured 64-bit form | 0.5 cycles in the loop test | 0/6 + 2/3 |
| Legacy `SHR r,CL` | 3 in Agner’s Skylake table | 2 cycles | 2 cycles | 0/6 |
| `IMUL r32,r32,14` | 1 | 3 cycles | 1 cycle | 1 |
| `BEXTR r,r,r` | 2 | 2 cycles | 0.5 cycles | Integer ALUs |
| `BZHI r32,r32,r32` | 1 | 1 cycle | 0.5 cycles | 1 or 5 |
| `PEXT r32,r32,r32` | 1 | 3 cycles | 1 cycle | 1 |
| `MOVZX r32,m16` | 1 | 5 cycles indexed address-to-result | 0.5 cycles | 2 or 3 |

Agner supplies the legacy-shift, immediate-multiply, BEXTR and generic SHRX figures; uops.info supplies the exact memory-SHRX, PEXT, BZHI and narrow-load measurements. [agner.org](https://www.agner.org/optimize/instruction_tables.pdf?utm_source=chatgpt.com)

### Why the BMI2 result is unsurprising

The folded memory-source `shrx` still executes a load µop and a shift µop. The measured form also occupies **two retirement slots**. Folding the load into one assembly instruction therefore does not make the operation one micro-op. It can remove the legacy variable shift’s extra work and `CL` constraint, but cannot compute the record address, multiply the slot, recover the byte or update lane state. [uops.info](https://uops.info/html-instr/SHRX_R64_M64_R64.html?utm_source=chatgpt.com)

That explains why **38→37 static instructions can represent a useful local improvement without constituting an architectural rescue**.

### Why I would not pursue PEXT, BZHI or BEXTR here

For this contiguous field:

- **PEXT** requires a shifted mask and has a three-cycle data dependence. Its memory operand is the **mask**, not the packed-data operand, so it cannot replace the current memory-source SHRX with one instruction that loads and extracts the payload. [cdrdv2-public.intel.com](https://cdrdv2-public.intel.com/774492/325383-sdm-vol-2abcd.pdf?utm_source=chatgpt.com)
- **BZHI** needs a register containing 14. The existing immediate `AND 0x3fff` already performs the fixed-width operation cheaply. [uops.info](https://uops.info/html-instr/BZHI_R32_R32_R32.html?utm_source=chatgpt.com)
- **BEXTR**, a BMI1 instruction, needs its start/length control assembled and uses two ALU µops before accounting for a data load. It does not eliminate the packed-address calculation. [agner.org](https://www.agner.org/optimize/instruction_tables.pdf?utm_source=chatgpt.com)

A 32-entry extraction-parameter table is also not free: it introduces a load before the packed-data address becomes available. L1 residency removes neither the dependence nor load-port demand.

The practical compression literature reinforces the distinction between this lookup and sequential decompression. SIMD-BP128 amortizes decoding across blocks; its authors explicitly discuss the resulting random-access granularity. SDSL’s integer-vector implementation separately specializes 16-bit access to a direct typed load. These are relevant precedents for simplifying the access primitive, not evidence that a fast block unpacker transfers to an unpredictable successor walk. [arxiv.org](https://arxiv.org/pdf/1209.2137?utm_source=chatgpt.com) [SDSL implementation](https://github.com/simongog/sdsl-lite/blob/master/include/sdsl/int_vector.hpp).

## 3. Three ranked candidates

### 1. Pack the row and carried byte into one lane word

**This is the strongest remaining same-format opportunity.**

Define the lane’s hot state as:

\[
q=\mathrm{row}\;|\;(\mathrm{carry}\ll20).
\]

The current header’s low 32 bits already contain precisely the corresponding fields:

\[
h_{32}=\mathrm{base}+(F[j]\ll20).
\]

On the ordinary path, `offset != 0x3fff` establishes that the record is FOR rather than RAW, and that this slot is not marked. The RAW bit and reserved bits in the low header word are zero. Furthermore:

\[
\mathrm{base}+\mathrm{offset}=\Psi(j)<2^{20}.
\]

Consequently:

\[
\boxed{q_{\mathrm{next}}=h_{32}+\mathrm{offset}}
\]

There is **no carry into the byte field**, so the ordinary path needs neither a packing OR nor the present header-base mask. This follows directly from the [current header and RAW-entry encodings](https://github.com/lmmx/mibz2/blob/9175a1122aac2b8031de01336353e2971984b55a/examples/coop/compact.rs).

A precise semantic prescription is:

```text
q     = lane.state
group = q & 0x000fffe0
slot  = q & 31

header = load_u64(records + 2*group + 56)
offset = extract14(records + 2*group, slot)

if offset != 0x3fff:
    next = (header as u32) + offset
    byte = (q >> 20) as u8

    lane.state = next
    prefetch(records + 2*(next & 0x000fffe0))
    store_output(byte)
```

The group mask also exposes a cheaper address expression:

\[
64(\mathrm{row}\gg5)
=2(\mathrm{row}\mathbin{\&}\texttt{0x000fffe0}).
\]

That can replace the present `shr 5; shl 6` sequence with a mask and scaled addressing.

#### Required slow-path changes

The ordinary algebra is simple; these invariants are where a careless implementation would fail:

| Operation | Required behavior |
|---|---|
| Initialize a segment | Set `state = successor | (F[start] << 20)`. Retain the existing first output byte `ll[start]`. |
| Enter RAW wait | Set `state = (state & 0x0ff00000) | PARK`, preserving the old carried byte. |
| Resume RAW | Emit the old token’s byte; assign the raw entry directly as the next token. RAW entries already have the required encoding. |
| Finish a segment | Pass `state & 0xfffff` to row-identity operations such as `seg_of`. |
| Test a start-mask bit | `state & 31` remains the slot. |
| Form any record/prefetch address | Clear the carried-byte bits; `& ~31` alone is insufficient. |
| Process the tail | Apply the same token transitions as in the full-lane loop. |

Also assert that **`PARK < 2²⁰`**. The actual bzip2 maximum guarantees this, but the generic condition `n < 2²⁰` does not: rounding a number just below the limit up to a multiple of 32 can reach the limit.

**Expected effect, uncompiled:** eliminate one common-path load and one common-path store; reduce the seven-instruction state update to approximately three; save another instruction in record addressing. Removing `b` should also reduce the current 40-byte lane layout to 32 bytes.

This has a different mechanism from the failed unrolling trial. It removes algorithmic state traffic across all 32 lanes; it does not depend on keeping all lane states in registers.

### 2. Replace the five-instruction multiply-by-14 expansion with one IMUL

The [current BMI2 assembly](https://github.com/lmmx/mibz2/blob/9175a1122aac2b8031de01336353e2971984b55a/docs/journal/2026-11-01-for14-stage1/disassembly-bmi2.md) computes `14*slot` as:

```text
copy slot
double it             // 2*slot
copy slot
shift left by four    // 16*slot
subtract              // 14*slot
```

The desired alternative is exactly:

```asm
imul at32, slot32, 14
```

This saves **four static instructions**, but not four executed ALU µops. If the two copies are eliminated, the original sequence is three arithmetic µops. The add and shift can execute in parallel, followed by the subtraction.

Immediate IMUL reduces that to one µop, but changes the product dependence from approximately two cycles to three and places it on port 1. **It trades throughput demand for slightly longer address latency.** That is why its value must be assessed alongside the packed-state change, rather than inferred from instruction count alone. [agner.org](https://www.agner.org/optimize/instruction_tables.pdf?utm_source=chatgpt.com)

The Rust source already spells this as multiplication. Changing it to `.wrapping_mul(14)` is not a reliable way to request IMUL; LLVM may produce the same strength-reduced sequence. A future authorized check should first inspect the compiler result, then use at most a tiny explicit multiply if needed. Such inline assembly must not claim `preserves_flags`.

The combined hand count is:

| Change from current BMI2 path | Static reduction |
|---|---:|
| Packed lane-state update | 4 |
| Record-address simplification | 1 |
| Immediate multiply | 4 |
| **Total** | **9** |
| **Candidate target** | **37 − 9 = 28** |

The corresponding component-level back-end saving is closer to **seven executed µops under favorable move elimination**. The exact function-wide count still depends on allocation, fusion and addressing.

This is a credible assembly target. It is not evidence that the resulting walker clears `W1 ≥ 0.774`.

### 3. Use 28 sixteen-bit fields per cache line, with encoded row coordinates

**This has the largest structural extraction saving, but it is a new format and remains a paper proposal under the current authorization.**

Naively widening the existing fields requires:

\[
32\times2+8=72\text{ bytes}.
\]

A 64-byte-aligned record rounded to 128 bytes loses the base footprint advantage. Packing 72-byte records tightly loses the guarantee that header and selected field share one line.

A more coherent neighboring layout is:

- 28 sixteen-bit offsets: 56 bytes.
- The existing eight-byte header.
- Four reserved halfword positions corresponding to the header.

The apparent problem is division by 28 on every lookup. That can be moved entirely out of navigation by relabeling rows at construction:

\[
\phi(j)=32\left\lfloor\frac j{28}\right\rfloor+(j\bmod28).
\]

Every successor and segment coordinate uses this encoded label. Its low five bits are always 0–27.

For an **untagged encoded row** \(x\):

```text
field_address  = records + 2*x
header_address = records + 2*(x & ~31) + 56
offset         = load_u16(field_address)
```

There is no hot-path division, multiply-by-14, variable shift or field mask. With a packed carry token, first clear the carried-byte bits when forming these addresses.

A direct `MOVZX r32,m16` is one load µop. Header and field remain independent loads of one cache line. [uops.info](https://uops.info/html-instr/MOVZX_R32_M16.html?utm_source=chatgpt.com)

The capacity arithmetic works for a full bzip2 block:

| Quantity | Derived value |
|---|---:|
| Records for 900,000 real rows | 32,143 |
| Largest real encoded label | 1,028,567 |
| Parking label | 1,028,576 |
| Twenty-bit limit | 1,048,576 |
| Base storage | 2.2857 bytes per real row |

If RAW groups retain a padded 128-byte area, total storage is approximately:

\[
\frac{64}{28}+\frac{128}{28}p
\]

bytes per real row, where \(p\) is the **new** RAW-group fraction.

If \(p\) happened to remain 1.9%, that would be about **2.373 bytes/row, or 2.135 MB per full block**: roughly 14% larger than current FOR-14, while still about 41% smaller than T. That exception fraction is illustrative, not measured.

The obligations are substantive:

- Encode every ordinary and RAW successor as \(\phi(\Psi(j))\).
- Store differences in the **encoded-label domain**.
- Transform starts and endpoints consistently.
- Ensure no link targets a header slot.
- Recompute the census because grouping and spans change.
- Budget the direct builder’s coordinate maintenance.
- Preserve the byte orientation.
- Enforce a production memory cap: padded all-RAW storage reaches **6.857 bytes/row**.

I would retain this as one specific reserve design. I would not automatically implement it after a failed FOR-14 optimization.

## 4. Lowest-cost decisive check and compiler controls

**Do not repeat the completed target-feature-only experiment.** If further code-generation work is authorized, the smallest useful comparison is the packed-state/address change, followed by the single-multiply composition, with disassembly and existing correctness checks only.

The decisive questions are:

1. Does the extra lane load/store actually disappear?
2. Does the binary approach the approximately 28-instruction target with substantial µop savings?
3. Does it avoid helper calls, new spills or additional dependent lookups?
4. Are RAW waiting, starts, one-byte segments, short blocks and tail behavior still exact?

A cosmetic instruction reduction should fail this gate.

The target controls need care:

- `+bmi2`, `x86-64-v3` and `target-cpu=skylake` are not equivalent. Current LLVM associates `x86-64-v3` with `HaswellModel`, while `skylake` uses `SkylakeClientModel` and different feature/tuning lists. [LLVM processor definitions](https://github.com/llvm/llvm-project/blob/main/llvm/lib/Target/X86/X86.td).
- `target-cpu=native` depends on the machine performing compilation. Record the actual compiler and target settings. [The rustc book](https://doc.rust-lang.org/rustc/codegen-options/index.html?utm_source=chatgpt.com)
- The existing whole-function BMI2 annotation is appropriate. Rust does not inline a target-feature function into an incompatible caller; annotating only a tiny per-row helper risks introducing a call boundary. [The Rust Reference](https://doc.rust-lang.org/reference/attributes/codegen.html?utm_source=chatgpt.com)
- There is a concrete harness trap: [`bench/coop.sh`](https://github.com/lmmx/mibz2/blob/9175a1122aac2b8031de01336353e2971984b55a/bench/coop.sh) overwrites `RUSTFLAGS`. With `--no-libsais`, its flags variable is empty. Exporting a CPU target before invoking that script does not establish that rustc receives it.

Preserve `rustc -Vv`, source commit, feature flags, release/LTO settings and binary hash. Preserve the frozen original separately: adding the BMI2 function already changed `h`’s code allocation in the newer binary, as the [journal records](https://github.com/lmmx/mibz2/blob/9175a1122aac2b8031de01336353e2971984b55a/docs/journal/2026-11-01-for14-stage1.md).

**Any later timing of changed machine code needs its own frozen candidate identity and recorded decision procedure.** Passing this compile-only assessment would not authorize that timing.

## 5. Critical assessment of PR #131

The ladder is useful preparation. Its present measurement and interpretation rules are not ready to support the intended causal conclusions.

### A. Blocking defect: PMU counts include repeated validation

In [`run()`](https://github.com/lmmx/mibz2/blob/e46506be6871076781734a6abf6c821c604dff80/examples/walker_ladder.rs), D/P/S execute:

```text
start timer
run kernel
stop timer
hash entire output
```

A–C instead validate returned scalars.

The [Python driver](https://github.com/lmmx/mibz2/blob/e46506be6871076781734a6abf6c821c604dff80/bench/walker_ladder.py) runs `perf stat` around the whole process and calculates:

\[
\frac{\mathrm{counts}(\mathrm{REPS})-\mathrm{counts}(0)}
{\mathrm{REPS}\times n}.
\]

But the repeated output hashes scale with `REPS`:

\[
\mathrm{counts}(\mathrm{REPS})-\mathrm{counts}(0)
\approx
\mathrm{REPS}\times(\mathrm{walk}+\mathrm{validation}),
\]

plus other differences.

**The zero-repetition baseline does not remove them.**

Consequences:

- Internal `ns/row` excludes hashing.
- PMU cycles, instructions and µops include hashing.
- D/P/S receive substantial validation work absent from A–C.
- `summarise()` preferentially uses those PMU cycles for D1–D3.

This could manufacture an apparent output-store or production-overhead penalty. More repetitions will not fix it.

The correction is to make the counter interval and timer interval describe the same workload, with validation outside both. Counter-control overhead must be accounted for. The timer-only VM figures do not have this particular PMU contamination, although validation still changes the cache state between repetitions.

### B. The rungs measure net contrasts, not additive costs

The actual source changes more than the labels suggest:

| Contrast | Actual change |
|---|---|
| A→B | Adds software prefetch |
| B→C | Replaces counter termination with flagged termination and removes remaining-count maintenance |
| C→D | Removes ordinary-row byte summation; adds stores and destination-pointer maintenance |
| D→P | Changes destination layout, lane management, room passes, buffers, growth and assembly |

A–C also use a shared `sum` across lanes. That is an artificial recurrence absent from the production common path. Its significance needs assembly inspection; the compiler may reassociate some work.

Therefore, “stores cost 27%” should become “the C→D kernel contrast costs 27% in this VM run.” Likewise, D→P is not an isolated bookkeeping tax. These distinctions follow from the [source](https://github.com/lmmx/mibz2/blob/e46506be6871076781734a6abf6c821c604dff80/examples/walker_ladder.rs).

There is no recorded disassembly in this PR establishing which arrays become registers or memory at 32 lanes. A is already a segmented, scheduled, counter-terminated walker. It is not a demonstrated register-only hardware minimum.

Separately timing S also does not make `P−S` an exact traversal cost: repeated assembly alone has a different cache context from assembly immediately following the walk.

### C. D3 overinterprets issued µops

The current rule is effectively:

\[
\frac{\texttt{UOPS\_ISSUED.ANY}}{4\,\mathrm{cycles}}\geq0.70
\Longrightarrow \text{execution-bound}.
\]

The complement is labeled “not execution-bound.”

Neither implication follows.

Intel defines `UOPS_ISSUED.ANY` as µops issued from the RAT to the reservation station. It is not retired useful work and not a universal measure of execution-port saturation. High issued work can include speculation. Conversely, a kernel can saturate one execution port or follow a limiting dependency chain while remaining far below four issued µops per cycle. [Intel event definitions](https://github.com/intel/perfmon/blob/main/SKL/events/skylake_core.json).

D3 should report **issue/allocation pressure**, with an appropriately qualified capacity budget. Classification needs the assembly, retired work, speculation, front-end delivery and memory/store evidence. Intel’s own top-down model distinguishes these categories. Replacing issued µops with retirement slots would measure occupied retirement capacity, but still would not prove the cause of elapsed cycles. [Intel metric definitions](https://github.com/intel/perfmon/blob/main/SKL/metrics/skylake_metrics.json).

### D. Demand-only MLP cannot establish total useful overlap

Intel’s `PENDING/PENDING_CYCLES` ratio describes demand-associated fill-buffer occupancy during cycles when that population is nonempty. A buffer allocated by prefetch enters this population when demand encounters it—not throughout its earlier prefetch lifetime. Better prefetching can therefore reduce observed demand MLP while improving throughput. [cdrdv2-public.intel.com](https://cdrdv2-public.intel.com/671378/335279-performance-monitoring-events-guide.pdf?utm_source=chatgpt.com)

The other events need equally careful interpretation:

- The mapped Skylake **`FB_FULL`** event counts requests needing an unavailable fill buffer. Dividing its raw count by cycles gives an event rate, not automatically a fraction of cycles full.
- **`SW_PREFETCH_ACCESS.T0`** counts executed prefetch instructions.
- **`LOAD_HIT_PRE.SW_PF`** observes demand dispatches encountering an in-flight software-prefetch fill buffer; it does not count every useful prefetch. [Intel event definitions](https://github.com/intel/perfmon/blob/main/SKL/events/skylake_core.json).

A demand-MLP plateau plus increasing fill-buffer pressure is evidence **consistent with** a capacity constraint. It is not an exclusive diagnosis. Doubling an initially negligible rate is also weak evidence.

The protocol’s fallback to “MLP alone” when `FB_FULL` is unavailable should be removed. The implementation currently skips D4 without all quantities, so prose and analysis already disagree.

### E. Four-core transfer and validity gates need tightening

The driver launches four processes without a post-setup barrier or recorded overlap of their timed windows. P also hashes output between walks while A/B do not, changing neighbor duty cycles and traffic. Four-core counter records retain setup totals and discard running percentages. These properties are visible in the [driver](https://github.com/lmmx/mibz2/blob/e46506be6871076781734a6abf6c821c604dff80/bench/walker_ladder.py).

Before D5 is interpreted, establish synchronized measurement after setup and record the common overlap. Apply counter-validity checks to every value entering analysis, including baselines and four-core measurements.

Three smaller corrections are also warranted:

- A supported event reporting zero is not automatically unavailable.
- D2 selects its best bare reference across lane counts but attributes the gap using only 32-lane transitions. Those comparisons must match.
- D5’s prose says “15 points”; its code tests a 15% relative ratio of ratios. Choose one definition before freezing.

## 6. What a corrected ladder can establish

Report 132’s underlying method remains valuable. The literature supports distinguishing fewer memory operations, more overlap and less execution work:

- Kärkkäinen–Kempa–Puglisi explicitly separate reducing BWT misses from overlapping their cost. [cs.helsinki.fi](https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf?utm_source=chatgpt.com)
- AMAC studies irregular lookup scheduling and saturation on a Xeon X5670; its numerical resource limits are not i9 measurements. [homepages.inf.ed.ac.uk](https://homepages.inf.ed.ac.uk/bgrot/pubs/AMAC_VLDB16.pdf?utm_source=chatgpt.com)
- Beamer’s pointer-chasing study varies instructions between misses to expose instruction-window limits on Ivy Bridge. Its parameters should not be transplanted into this software-prefetched walker. [scottbeamer.net](https://scottbeamer.net/pubs/beamer-thesis.pdf?utm_source=chatgpt.com)

A corrected ladder can answer:

- Is there a large demonstrated gap between a reduced-semantics reference and production?
- Which kernel contrast introduces that gap?
- Does prefetch materially change the result?
- Does the conclusion survive controlled four-core contention?

It cannot automatically assign each difference to one instruction class, or turn one repeated block into a whole-decoder result.

The BMI2-only test has already preceded the ladder. There is no reason to repeat it. A narrowly authorized compile-only review can settle the two concrete source simplifications without waiting for PMU data; the corrected ladder supplies the broader architectural evidence.

## 7. Economics and stopping conditions

The conditional planning arithmetic remains sound:

\[
W_1\geq\frac{1.236}{1.847}=0.6692
\]

for traversal-plus-assembly break-even, and

\[
W_1\geq\frac{1.236}{1.847-0.25}=0.7740
\]

for saving 0.25 CPU-s/GB, under the retained phase model, unchanged construction cost and the assumption that compact four-worker CPU cost is no lower than its one-worker cost. [Reports 131](https://github.com/lmmx/mibz2/blob/27e425dd5919ba02cdb57558d0a08fc8e7795cca/docs/journal/2026-11-01-external-report-131.md) and [135](https://github.com/lmmx/mibz2/blob/27e425dd5919ba02cdb57558d0a08fc8e7795cca/docs/journal/2026-11-01-external-report-135.md).

I would qualify one argument in the Stage 1 interpretation: **`v` taking 28 instructions and achieving 0.721× does not establish that every 28-instruction compact path must fail.** Its byte-directory dependencies and seven loads differ from the proposed same-line lookup with four loads. The two-point instruction fit is a warning, not a transferable performance law.

My stopping rule is:

1. **Keep direct construction stopped now.**
2. If one authorized same-format check fails to remove the extra lane load/store and produce substantial actual µop savings without new dependencies, stop this rescue.
3. If it succeeds, require a separately identified, frozen candidate before any timing.
4. Under the existing economic assumptions, a valid candidate whose `W1` upper bound is below 0.774 fails the 0.25-saving objective; below 0.669 it fails even the optimistic traversal break-even condition.
5. Do not automatically advance to the neighboring format.

No static count proves that FOR-14 is intrinsically impossible. The defensible engineering conclusion is that the best specifically justified candidate fails the agreed budget and therefore does not deserve further work.

## 8. Three questions for Astra and Claude

1. **Can the compiled common path use `header_low32 + offset` as the complete next lane token?** Return the actual load/store reduction and dependency graph, including carry preservation while parked, masked addresses and tail behavior.

2. **Can PR #131 make its PMU and timing intervals describe identical work?** Remove repeated validation from both intervals, document the existing rung assembly, establish four-core overlap, and narrow D3/D4 to claims their event populations support.

3. **Which measured resource makes compact navigation affordable or unaffordable?** Distinguish allocation/retirement demand, store pressure, address latency and miss overlap. Use that evidence to judge the same-format candidate and any later layout decision, rather than treating a match to `v`’s instruction count as decisive.

**My research judgment is to retain exactly one bounded same-format hypothesis: exploit the header as packed lane state, then remove unnecessary address arithmetic. The immediate diagnostic priority is to repair PR #131’s measurement boundary.** The 38→37 result is a valid negative result; it should close the failed instruction-selection hypothesis without being stretched into proof that the remaining explicit state traffic is unavoidable.
`````
