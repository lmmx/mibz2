# External research report 139: the guardian on the researcher's review: one compile-and-correctness check of the packed lane token, and the ladder's measurement repaired, before any capture (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01, with reports 136–139 together.
- **Source:** the guardian.
- **In reply to:** report 138.

**Evidence cutoff:** 1 November 2026. Instructions: no i9 performance capture, direct builder or new format is authorized.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
**This is a substantive contribution—and it gives us a better next step than either abandoning FOR-14 outright or launching another benchmark immediately.**

The researcher has identified two distinct things:

**A concrete way to remove work from FOR-14’s ordinary path**, rather than merely choosing a different instruction for the same work.

**A measurement defect in the walker ladder that must be fixed before its hardware-counter results are interpreted.** I checked the cited Rust and Python sources, and that defect is real.  

My recommendation is **one bounded, compile-and-correctness-only rescue check, alongside repair of the ladder’s measurement boundary. No new i9 performance run yet.**

## 1. The strongest idea is packing the lane state—not another BMI2 switch

The current implementation keeps the next row and the carried output byte as separate fields:

```text
Current lane state

    row ─────────────► load / update / store
    carried byte ────► load / update / store
```

The researcher proposes putting both into one 32-bit word:

```text
Proposed lane state

     31       28 27              20 19                       0
    ┌───────────┬──────────────────┬──────────────────────────┐
    │   unused  │ carried byte     │ row number               │
    │   4 bits  │ 8 bits           │ 20 bits                  │
    └───────────┴──────────────────┴──────────────────────────┘

    state = row | (carried_byte << 20)
```

This is a change to **the walker’s runtime state**, not a new navigation-table format. Its intended benefit is to remove one lane-state load and one lane-state store per ordinary step. The researcher explicitly labels that reduction as uncompiled. Pasted text(20261101-233722) Pasted text(20261101-233722)

### The useful algebra

The ordinary record header already contains:

\[
h_{\text{low32}}=\text{base}+(F[j]\ll20).
\]

The next row is:

\[
\Psi(j)=\text{base}+\text{offset}.
\]

Provided that sum remains below \(2^{20}\), adding the offset cannot carry into the byte field. Therefore:

\[
\boxed{\text{next state}=h_{\text{low32}}+\text{offset}}
\]

One addition produces **both the successor row and its carried byte in the required packed form**. The actual record-construction code uses the header and raw-entry layouts on which this identity depends. 

That is more promising than hoping LLVM will keep 32 separate byte values in registers. It changes what state the program needs to load and store.

| Ordinary-path operation | Current FOR-14 | Researcher’s proposed state packing |
|---|---:|---:|
| Data loads | 5 | 4 |
| Stores | 3 | 2 |
| Separate carried-byte field | Yes | No |

These are the report’s current-versus-proposed counts, **not a measured performance result**. Pasted text

**This corrects something I accepted too readily earlier:** the carried-byte load/store pair is not necessarily an unavoidable cost of FOR-14. It is partly a consequence of the current lane representation.

The difficult correctness cases remain important: entering and leaving RAW parking, preserving the carried byte, masking it out of addresses, segment initialization and tail processing. The researcher has supplied explicit invariants for those cases, including checking that the parking row fits within the 20-bit row field. Pasted text(20261101-233722)

## 2. The 28-instruction target is worth compiling—but it is not a speed forecast

The second proposal replaces the current multiply-by-14 expansion with an explicit immediate multiply. Together with state packing and simpler record addressing, the researcher derives a possible reduction from **37 to 28 static instructions**. It also identifies the trade-off: fewer operations may come with a longer address-calculation dependency. Pasted text(20261101-233722)

I would test these in two steps:

**First, pack the lane state and simplify addressing.** Check whether the extra load/store actually disappears and whether the slow paths remain correct.

**Then, separately inspect the immediate-multiply version.** Do not obscure which improvement comes from state packing and which comes from exchanging arithmetic throughput for dependency latency.

The report is right to distinguish assembly instructions from processor work. For example, published measurements of memory-source `SHRX` on Skylake and Coffee Lake show two executed micro-operations and two retirement slots despite its being one assembly instruction. That helps explain why merely folding a load and shift did not deliver the hoped-for rescue. [uops.info](https://uops.info/html-instr/SHRX_R64_M64_R64.html)

### The measured gap is still large

Your completed i9 run measured the frozen FOR-14 walk at:

| Configuration | FOR-14 / conventional control |
|---|---:|
| One core | **0.475×** |
| Four cores | **0.688×** |

Both calibration checks passed. Those results remain the baseline against which any successor must be judged. for14-analysis for14-analysis

From the four-core ratio, the tested implementation would need roughly **45% more walk throughput just to reach parity**:

\[
1/0.688-1\approx45\%.
\]

Reducing 37 instructions to 28 does not establish that improvement. But removing actual memory operations gives the proposal a more specific mechanism than “fewer instructions ought to be faster.”

Equally, the old `v` arm having 28 instructions is not a universal objection to every 28-instruction implementation. The researcher points out that its byte-directory dependencies and seven loads differ from the proposed same-line lookup with four loads. That comparison needs a dependency-and-resource analysis, not just a count. Pasted text

**My judgment: this earns a small code-generation test, not another timing authorization.**

## 3. The ladder’s hashing defect is the immediate blocker

This is the most important measurement finding in the memo.

The pinned Rust code does this for the output-producing rungs:

```text
Start internal timer
    Run the kernel
Stop internal timer
    Hash the output
Repeat
```

But the Python driver places `perf stat` around the **whole process**. Consequently, its counters include the repeated hashes that the internal timer excludes. I confirmed both parts in the cited code.  

Subtracting a zero-repetition setup run does not fix it:

\[
\begin{aligned}
\text{counts}(N)&\approx\text{setup}+N(\text{kernel}+\text{hash}),\\
\text{counts}(0)&\approx\text{setup},\\
\frac{\text{counts}(N)-\text{counts}(0)}{N}
&\approx\text{kernel}+\text{hash}.
\end{aligned}
\]

The consequence is especially troublesome because some rungs hash a complete output while others check returned scalars. The benchmark could attribute validation work to output stores or production-walker overhead. The researcher identifies precisely this problem. Pasted text(20261101-233722)

**I missed this measurement-boundary mismatch in my earlier review. It needs to join the multiplexing and four-core validity fixes before the ladder is frozen.**

The correction should make the timer and counters describe the same workload, with validation outside both measurement scopes. Linux perf provides explicit counter enable/disable controls, although any chosen implementation must account for control overhead and synchronization rather than assuming perfectly free boundaries. [Kernel Git Repositories](https://kernel.googlesource.com/pub/scm/linux/kernel/git/stable/linux-stable/%2B/master/tools/perf/Documentation/perf-stat.txt)

This does **not** invalidate the completed FOR-14 confirmation. It is a defect in the separate, not-yet-captured walker-ladder experiment.

### The ladder also needs more modest interpretation

The researcher’s distinction between **kernel contrasts** and **isolated costs** is important.

For example, C→D does not simply add stores: it also removes byte summation and changes destination handling. D→P changes buffer organization, management and assembly. A difference between those kernels is useful evidence, but it is not automatically “the cost of stores” or “the cost of bookkeeping.” Pasted text(20261101-233722)

The same applies to the proposed issue-width and memory-parallelism rules. They should report supporting evidence about resource pressure, not turn one counter ratio into an exclusive bottleneck diagnosis. Pasted text(20261101-233722)

I would have Claude address the complete set of measurement objections in this memo once, rather than repairing them one at a time across further i9 captures.

## 4. Keep the 28-entry, 16-bit layout as a reserve idea

The third proposal is genuinely different: use 28 simple 16-bit fields plus the header, and relabel rows so navigation does not perform division by 28 on every step. Its proposed encoded coordinates make the field address directly computable. Pasted text

That is an interesting architectural trade: **spend some of the compression benefit to make individual accesses much simpler**.

But it changes the format, coordinate system, exception census and eventual builder. Its approximately 2.135 MB estimate assumes an illustrative exception rate, not a newly measured one. Pasted text

I would preserve it, **not authorize it now**. The state-packing proposal lets us test a concrete improvement without opening another representation project.

## What I would send Claude

Send the complete researcher memo with this bounded instruction:

> The new researcher has identified a specific same-format optimization and a blocking measurement-boundary defect. Please preserve the memo as a separately attributed report.
>
> **First, repair the walker ladder before any timed capture.** Confirm and fix the mismatch whereby the internal timer excludes repeated output hashing but process-wide PMU counts include it. Validation must remain outside both measurement scopes, with counter-control overhead and cache preparation documented. Address the memo’s four-core synchronization, counter-validity and rung-comparability concerns together with the already identified multiplexing issues. Add tests for these failure modes and revise D3/D4 so their conclusions do not exceed what the counters measure.
>
> **In parallel, one bounded same-format compile-and-correctness check is justified.** Implement the proposed packed lane token, `row | (carry << 20)`, and the corresponding masked record addressing. Verify `next_state = header_low32 + offset` under the ordinary-path invariants, including RAW parking, segment initialization, address masking and tails. Report actual lane size, load/store counts, instructions and dependency structure.
>
> Inspect the explicit multiply-by-14 alternative separately, so its latency/throughput trade-off is not confused with the state-traffic reduction. Do not use fewer static instructions alone as the acceptance criterion.
>
> Keep the original `7d7962f` result and its STOP verdict unchanged. Any new code must have a separate identity. Do not build a direct constructor, implement the 28-entry format, alter production defaults or run another i9 performance capture.
>
> Return the repaired measurement design and the actual compiled same-format result. Stop there for review.

**This is the kind of contribution we wanted from the fresh researcher:** a concrete algebraic simplification that removes explicit state traffic, plus a source-level check that prevents the next experiment from measuring the wrong work.

The appropriate response is neither renewed optimism that FOR-14 must succeed nor a blanket rejection because its first implementation lost. It is **one precise implementation check, a properly repaired diagnostic, and no further workstation run until those are ready.**
`````
