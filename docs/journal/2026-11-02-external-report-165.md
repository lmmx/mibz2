# External research report 165: Astra's architectural review of compact30: Option 2, one compile-and-correctness prototype, refined to current-row F emission with F in the header's low byte (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** Astra, the existing Codex conversation, at Max reasoning with no local commands.
- **In reply to:** the two-part prompt in reports 160 and 161.

**Evidence cutoff:** 2 November 2026. A reading of the compact30 reference and raw census
(`1fbc0a0`); the o/h/b/k/j listings and their implementation (`9d22ac9`); production
`bwt.rs` (`13e9854`); report 156; and the amended walker-ladder interpretation. No code was
changed or run.

**Two of its corrections are already applied** to the census journal (`4871b65`):
- the row total is 707,949,378, not "about 690 M";
- the scalar reference's `Vec<u16>` records are not guaranteed 64-byte aligned.

**Its prototype handoff is not yet authorised.**

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
**Choose Option 2: build one isolated, compile-and-correctness prototype of compact30, with a refinement: emit the current row’s F byte, keep only the encoded row in lane state, and put F in the header’s low byte.** Keep the 30 offsets, coordinate transformation, two sentinels and RAW parking scheme.

This preserves the census’s storage economics while removing two address masks from the proposed packed-token path. It also has a concrete disadvantage: successor formation gains a shift after the header load. That trade deserves a bounded prototype; it does not yet justify a performance prediction or a direct-constructor implementation.

Measuring k/j first would answer whether their remaining packed-field extraction is affordable. It would not settle whether direct halfword addressing is affordable. Neither outcome is a necessary prerequisite for this cheaper, compile-only decision.

I inspected the compact30 reference and raw census, the o/h/b/k/j listings and supporting implementation, production `bwt.rs`, report 156’s construction proposal, and the amended hardware-ceiling interpretation. The closed negative experiments remain closed.

**The architectural improvement starts with changing byte ownership.**

The stable counting scatter establishes

\[
F[r]=L[\Psi(r)].
\]

Production’s output, starting from `orig_ptr = p`, is

\[
L[\Psi(p)],\ L[\Psi^2(p)],\ldots
\]

Therefore exactly the same output is obtained by starting at **p**, emitting **F of the current row**, and then advancing through Ψ:

\[
F[p],\ F[\Psi(p)],\ldots
\]

This works on periodic and multi-cycle permutations; it does not require Ψ to have one cycle. The coordinate transformation remains

\[
\Psi'=\phi\circ\Psi\circ\phi^{-1}.
\]

The fundamental ordinary-step inputs are consequently just:

- The current encoded row.
- The output destination.
- The existing segment and buffer state.

A carried byte is unnecessary under this emission convention. This follows directly from the reference’s counting scatter and production’s initial successor selection. [Reference implementation](https://github.com/lmmx/mibz2/blob/1fbc0a0b609ed9584fdafcaf9668308ccbd0358d/examples/census30.rs), [production walker](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/src/bwt.rs#L829).

For the recommended layout:

| Item | Encoding |
|---|---|
| Ordinary record | Thirty u16 offsets, followed by `H = (B << 8) \| F` |
| Ordinary successor | `(H >> 8) + offset` |
| Ordinary output byte | Low byte of H |
| RAW entry | `(encoded_successor << 8) \| F[row]` |
| RAW header | Retain the RAW flag and external-group index |
| Lane state | Plain encoded row; `pend`, segment and output pointers as before |

B remains the encoded successor base. Ordinary eligibility and RAW classification are unchanged, so **the census’s logical sizes transfer exactly to this representation**. Performance does not.

I made a further refinement after removing the carry: keeping F in bits 20–27 would still require separating the base and byte with masking and shifting. Moving F to bits 0–7 makes the byte directly usable by an x86 byte store; one shift extracts the base.

I also considered carrying a cached header in lane state. That either adds state traffic or moves the next-header load behind successor formation, creating another dependent lookup. It does not eliminate the information requirement cheaply.

The original packed-token compact30 remains the one close alternative: it forms the next token with a single header-plus-offset addition and has the output byte ready earlier. Those advantages could outweigh its extra masks. I would not implement both optimized variants initially.

**The ordinary executable path is straightforward, but its advantage is limited and specific.**

```text
x = lane.row                         // plain encoded row
o = load_u16(records + 2*x)
H = load_u32(records + 2*(x & ~31) + 60)

if o >= 0xFFFE:
    handle_start_raw_or_parking(x, o, H)
else:
    emit low_byte(H)
    next = (H >> 8) + o
    lane.row = next
    prefetch(records + 2*next)
```

Both table addresses derive independently from x. Load H before the exceptional branch if LLVM permits: its address is valid for ordinary, RAW and parking records.

A plausible sequence retaining the observed loop bookkeeping is below. This is **proposed assembly, not a compiled result**. Here `r9` points to the lane, `r10` is its index, `r12` is the record arena, `r15` is the pass’s output-base array, and `rdx` is the pass offset.

```asm
.loop:
    lea         r11, [r10+1]
    mov         eax, [r9]                 ; encoded row
    mov         ecx, eax
    and         ecx, -32
    movzx       edi, word [r12+rax*2]      ; selected offset
    mov         esi, dword [r12+rcx*2+60]  ; independent header load
    cmp         edi, 65534
    jae         .slow

    mov         rbp, [r15+r10*8]
    mov         byte [rbp+rdx], sil        ; F[current row]
    shr         esi, 8
    add         esi, edi                  ; encoded successor
    mov         [r9], esi
    prefetcht0  [r12+rsi*2]                ; no successor mask

    lea         rcx, [r11-1]
    add         r9, 32
    mov         r10, r11
    cmp         rcx, 31
    jb          .loop
```

That sketch has 19 instructions. Around **19–21** is a reasonable compilation expectation with the existing loop shape, not a performance gate by itself.

| Ordinary step | Instructions | Data loads | Stores |
|---|---:|---:|---:|
| Observed o | 16 | 3 | 2 |
| Observed h | 38 | 5 | 3 |
| Observed b | 37 | 5 | 3 |
| Observed k | 31 | 4 | 2 |
| Observed j | 27 | 4 | 2 |
| Proposed refinement | Approximately 19–21, uncompiled | 4 | 2 |

The four loads are lane row, output base, offset and header. The stores are lane row and output byte. There is one ordinary successor prefetch.

The named extraction costs genuinely disappear. In j, the sequence at `5814c–58164` computes the slot, multiplies by fourteen, derives byte and bit positions, executes a memory-source `shrx`, and masks the extracted value. Compact30 replaces that sequence with an indexed `movzx`. Unlike b, this changes the representation rather than merely selecting BMI2 instructions. [Observed listings](https://github.com/lmmx/mibz2/blob/9d22ac9164d0bba46a4942cf4c8001e40d75eca4/docs/journal/2026-11-01-for14-stage1/disassembly-kj.md).

The additional refinement removes the packed token’s field-address mask and successor-prefetch mask. It **does not remove another state load/store pair**: k/j already achieved that.

The dependency structure is:

```text
                           ┌─ offset load ──────────┐
current encoded row ──────┤                       ├─ ADD ─ successor
                           └─ group mask ─ H load ─ SHR
                                              └──── output byte
```

The important counterargument is visible here:

- Relative to packed-token compact30, the header-to-successor path gains one shift.
- The field address becomes available without a token mask.
- The header-limited path to the next prefetch may be approximately unchanged: the new header shift replaces the old post-add prefetch mask.
- Output now depends on H arriving, instead of an already-loaded carried byte.

Thus this is primarily a reduction in execution work and live values. It is **not a claim that every pointer-chasing dependency becomes shorter**.

On the i9’s Skylake-derived core, the halfword load is an ordinary load operation, not a cheap substitute for a load. Two loads from one line still consume two load operations and load-queue entries. Four data loads already impose a two-cycle load-port floor, before prefetch and store-address competition. Indexed output stores also compete for address-generation resources; two stores consume store-data bandwidth. Compare/branch fusion and register-move elimination affect µop counts, so the compiled sequence must be audited in both fused and executed domains. [MOVZX measurements for Coffee Lake/Skylake](https://uops.info/html-instr/MOVZX_R32_M16.html#CFL), [Intel optimization manual](https://cdrdv2-public.intel.com/671488/248966-Software-Optimization-Manual-V1-048.pdf).

Keep the existing 32-lane room organization. The rows and cursors remain explicit array state, mostly served from L1; they cannot all reside in scalar registers. No generic unrolling or scheduling rewrite is part of this proposal.

**The encoding proof holds; the segmented implementation still has substantial correctness obligations.**

The RAW bound is sound. Within one character bucket, Ψ is strictly increasing, and φ preserves order. Successor intervals of distinct homogeneous groups are therefore ordered and disjoint. Their spans sum to at most M. Every wide group contributes at least 65,534, giving at most `floor(M/65534)` per bucket. Mixed groups number at most `σ−1`, since each contains a distinct bucket boundary.

Consequently,

\[
R\leq\min\left(G,\sigma\left\lfloor M/65534\right\rfloor+\sigma-1\right).
\]

At 900,000 rows, `G=30,000`, `M=959,997`, and parking is `960,000`. All fit twenty bits. The stated bounds—2,108,096 bytes for σ≤98 and 2,411,456 for σ≤256—are correct, including parking and padded RAW groups. The proposed header also fits: a twenty-bit B shifted by eight leaves bit 28 available as the RAW flag. [Proof and construction proposal](https://github.com/lmmx/mibz2/blob/f136dbadad5502f532cde6e7da4ca2fdb71405ce/docs/journal/2026-11-02-external-report-156.md).

I recalculated the archived JSONL’s storage formulas and bounds; they agree. A minor reporting correction: its 1,513 blocks total **707,949,378 rows**, rather than approximately 690 million. [Raw census](https://github.com/lmmx/mibz2/blob/1fbc0a0b609ed9584fdafcaf9668308ccbd0358d/docs/journal/2026-11-02-compact30-census/census30.jsonl).

Two implementation qualifications matter:

- The scalar reference’s `Vec<u16>` does **not guarantee 64-byte alignment**. Use an explicitly aligned 64-byte record type in the prototype. Otherwise the claimed single-line lookup need not hold.
- The marked scalar walk substitutes saved fields locally and continues. It does not exercise segmented termination, parking, buffer growth or mid-pass replacement. Its successful round trips do not discharge those obligations.

The refined walker should maintain this invariant: **a normal lane’s row identifies the next F byte not yet emitted**. The transitions are:

- **Taking a segment:** before marking starts, save its original field and `(successor << 8) | F[start]`. `take` writes that F byte and sets the lane to the saved successor. The mandatory first segment starts at `φ(orig_ptr)`, not `φ(Ψ(orig_ptr))`.
- **Encountering START:** finish the current segment without emitting the marked row’s byte. Preserve all markers until every segment has finished; restore saved fields afterward. START must win over RAW interpretation.
- **Encountering an unmarked RAW row:** calculate and save its external entry index, prefetch that entry, set the row to PARK, and emit nothing. Preserve the existing room-loop compensation for a waiting turn.
- **Resuming PARK:** load the pending RAW entry, emit its low byte, set the row to `entry >> 8`, and prefetch the successor record. No carried byte needs preserving.

This keeps the current 32-byte lane structure: two u32 fields plus segment and output pointers. Parking must be recognized before attempting to dereference the parking header’s dummy RAW index.

One-byte segments, RAW starts, partial records, tail entry with a full buffer, and termination midway through a 32-lane round all need explicit tests. Unused field positions and header positions must never become successors. No original-row conversion is needed in the hot path.

The journal’s statement that every row is visited exactly once also needs qualification. Segmented traversal can cover selected cycles, while assembly repeats the required cycle for periodic data. Stored RAW fraction therefore does not establish dynamic RAW frequency. Count RAW arrivals, resumptions, waiting turns and emitted bytes in untimed correctness runs. [Census journal](https://github.com/lmmx/mibz2/blob/1fbc0a0b609ed9584fdafcaf9668308ccbd0358d/docs/journal/2026-11-02-compact30-census.md).

**A direct constructor is feasible, and there is a concrete way to avoid the proposed staging array.**

Report 156’s per-character staging approach is valid. A neighboring approach can write ordinary records directly and promote them only when necessary:

1. Use the counts to identify bucket-crossing groups and assign their RAW storage.
2. Scan L once. Maintain each character’s destination cursor in encoded coordinates; advance by one, skipping the two header positions. Maintain the source’s encoded coordinate similarly.
3. For a homogeneous group’s first occurrence, install its base and F byte. Write subsequent offsets directly into the record while they fit.
4. If an offset exceeds 65,533, allocate a RAW group, reconstruct its already-written prefix from the saved base and offsets, replace those fields with RAW sentinels, and direct subsequent entries to RAW storage.

Monotonicity makes promotion permanent and bounds the converted prefix to at most 29 entries per promoted group. Crossing groups are RAW from the outset. Validate bucket endpoints as production does.

This needs no full T-vector, no sort, and no thirty-value staging buffer per character. It replaces staging stores and reads with header accesses, classification and occasional rewriting. That is a real trade, not free construction.

The byte is already available as the scanned character `c`. Compared with shipped construction, this avoids the dependent `ll[s]` byte load. **That saving is available to either compact30 emission convention**, not uniquely to my refinement. The refinement additionally eliminates L-based segment seeding. Production’s existing sixteen-equal-byte fast path remains a serious competitor and must be included in any later constructor comparison. [Shipped constructor](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/src/bwt.rs#L253).

Preallocating RAW capacity from the proven bound avoids transient reallocations. At the general full-block bound, records plus RAW storage are 2.411 MB; adding the 0.9 MB last column gives approximately 3.311 MB before shared scratch, output buffers and allocator effects. That is a credible bounded-memory route, **not an RSS measurement**. Retaining the old T-vector’s capacity alongside it would invalidate that accounting.

Do not use h’s archived “build” time to estimate this constructor: that phase copied precomputed records.

**The memory-system case remains plausible under several explanations, but not all.**

The amended ladder shows that substantial concurrency cost survives without output, software prefetch or the production queue. That supports investigating representation changes. It does not establish that LLC capacity is the bottleneck, or that a smaller table will recover a particular fraction of the penalty. [Amended ladder record](https://github.com/lmmx/mibz2/blob/356282babbf4f69d4d60b7e26babd73b05ea3524/docs/journal/2026-11-01-walker-ladder-protocol.md).

This design is favored when smaller navigation storage improves reuse, reduces translation pressure, or reduces construction’s dirty-line footprint. It is disadvantaged when execution ports, load-queue capacity or instruction-window occupancy dominate: it adds a table load relative to production.

If uncore service time dominates even for an unchanged stream of cache hits, compaction may do little. If intermediate output traffic dominates interference, that traffic remains. A smaller four-versus-one penalty could also arise because the candidate generates requests more slowly—exactly why absolute cost against production matters.

The historical compact-u failure remains a strong warning. This design removes its directory lookup and packed extraction, but retains additional execution work. Improved cache statistics would not constitute success.

**The economic screen should govern further investment.**

Using the supplied planning model,

\[
S\approx0.592(1-b)+1.847(1-w),
\]

the requirements for saving 0.25 CPU-s/GB are:

| Constructor cost ratio b | Required traversal-plus-assembly cost ratio w |
|---:|---:|
| 1.00 | ≤0.865 |
| 1.10 | ≤0.833 |
| 1.20 | ≤0.801 |

These are budget constraints, not forecasts. In particular, w must include assembly and exceptional work, normalized to the same decoded corpus. A lookup-only cycles/row result cannot be substituted directly.

The approximation can break when construction changes traversal cache state, phase overlap or downstream contention. Those interactions must eventually be charged end to end.

A useful gain remains credible because the candidate retains roughly half-sized navigation storage while removing named costs responsible for previous failures. **There is no current evidence that it delivers 1.5–2× complete decoding.** Even the 0.25 CPU-s/GB screen is demanding and represents only about 7% of the present total CPU cost.

**The next implementation should answer one bounded question: does this representation retain its cheap path inside the real segmented walker?**

The handoff to Claude should be:

> Implement one experimental compact30 walker with current-row F emission, low-byte F headers and plain encoded-row lane state. Use oracle-built records; retain 32 lanes, the existing room-pass structure, L1 prefetch and RAW parking. Keep production dispatch unchanged. Do not implement the direct constructor or run timing experiments.
>
> Check exact reconstruction against production across the eleven corpora and adversarial short, periodic, multi-cycle, partial-record and marker cases. Exercise forced RAW separately from natural-storage-bound checks. Record untimed RAW/wait/tail transition counts.
>
> Produce annotated release disassembly with `rustc -Vv`, LLVM version, target features and build flags. The expected ordinary signature is four data loads, two stores, one prefetch, independently addressable field/header loads, and no packed extraction, F-directory lookup, carried-byte state traffic or token-address masks.

Use the existing baseline x86-64 release settings; this design needs no BMI2. The checked listings document release/BMI2 distinctions, but I did not find an exact rustc/LLVM identifier in those disassembly records. Cargo’s MSRV is not compiler provenance.

The early stopping rules are:

- **Correctness:** any unresolved orientation, marker-restoration, parking or buffer-accounting failure blocks timing.
- **Code generation:** if the actual loop returns to k/j-scale work, introduces additional ordinary state/spill traffic, or serializes one table address behind the other table load, do not commission a timing trial of that binary.
- **Storage:** no hidden per-row metadata, and an explicit capacity/liveness account supporting a direct construction route without retaining a full temporary T-vector.
- **Later economics:** if a separately authorized traversal-plus-assembly test cannot reach the required w for a defensible constructor budget, stop. Better scaling or cache counters do not override that result.

Successful compilation and correctness would earn a separately frozen one-/four-core traversal experiment. It would not authorize production integration or establish constructor economics.

Three precise questions for Claude:

1. Does the compiled 32-lane loop preserve the four-load/two-store signature, and where exactly does its header-dependent recurrence differ from packed-token compact30?
2. Do transition counts and adversarial tests establish correct byte ownership at ordinary starts, RAW starts, parking resumption and repeated short cycles?
3. Can the capacity/liveness accounting support direct construction without a retained full T-vector, including RAW promotion and reusable buffer capacity?

**Build this one bounded prototype next.** The falsifier is a real segmented implementation whose extra execution or state costs restore the historical compact-format tax; the opportunity is a compact table whose ordinary path is finally close enough to production for its memory benefits to matter.
`````
