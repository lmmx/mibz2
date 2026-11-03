# Compact30: constructor v2, deferred offset formation (reports 205 and 206)

Astra's second constructor design (report 205), answering the request in
[report 203](2026-11-03-compact30-construction-capture-review.md), and the guardian's approval
of one bounded compile-and-correctness prototype of it (report 206). Both are kept as
relayed.

**Status.** Constructor v1 (`examples/c30walk/ctor.rs` at `e003a32`) stays closed by
report 202's engineering STOP and stays frozen. Constructor v2 is authorised as **one new
variant**, research-only, in a separate module, for correctness, an initialisation and
bounds-safety argument, untimed structural counters and a static assembly inspection
(rustc 1.97.0). **No timing**, i9 run, protocol amendment, walker tuning, production
integration or performance claim is authorised. If correctness fails, or the compiled code
keeps the per-row classification machinery, the prototype returns a STOP recommendation
rather than a further variant.

**The design in brief:**
- Incomplete records get temporary low-16-bit successor labels, written straight into
  their final fields, instead of an offset formed per row.
- The identity φ(i) = 65536⌊i/61440⌋ + φ(i mod 61440) lets the high bits be recovered later
  from per-character destination-rank snapshots A_e[c] taken every 61,440 source rows:
  h(r) = max{e : A_e[c] ≤ r}.
- A pending portion is classified and normalised once, when it completes: by modular u16
  subtraction for an ordinary group, and as RAW at completion for a wide or crossing group.
- v1's run coalescing, complete-record emitter and canonicaliser are kept. The workspace is
  about 28 KiB. There is no full successor table, F array or second scan of L.

## Report 205: Astra's design, as relayed

**I recommend one algorithmic change: defer offset formation until an incomplete record is finished, storing successor low bits temporarily in the final record and recovering high bits from sparse bucket-count snapshots.**

Keep the current complete-record emitter, frozen layout, walker and canonical RAW permutation. This targets the expensive scalar and fragment machinery without introducing a full successor table, a second input scan or a new representation.

I consider a 19% constructor CPU reduction credible enough for Guardian review of this design. A 25% completion-time reduction is less certain. Neither is established by the existing capture.

The capture remains **INVALID**, not FAIL. Its consistent four-worker direction nevertheless argues against funding a small adjustment to the existing emitter.

**What the implemented constructor tells us**

My original recommendation placed too much confidence in complete-record coverage. The implementation achieved that coverage; the remaining work still appears expensive enough to defeat the economics.

Compared with production’s [actual `build_one_pass`](https://github.com/lmmx/mibz2/blob/e003a32/src/bwt.rs#L253), the strongest evidence is in the compiled scalar path:

- Along the ordinary scalar iteration at `23c99–23d1d`, followed by `23c60–23c93`, I count **49 instructions**, including source-coordinate advancement and the inner-loop update. This is a static path count, not a timing attribution.
- It updates the encoded destination cursor, derives the group and slot, loads the record header, tests RAW, computes an offset, tests for promotion, writes the field, and updates and stores the source coordinate and phase.
- Production’s scalar scatter does substantially less work. Its equal-byte path is also already efficient; compact30 is competing against an optimized constructor.

The fragment writer adds header inspection, classification, variable-length vector-loop setup and scalar tails. The archived structural census records approximately **1.08 million fragment calls for 14.82 million fragment rows**—about 13.7 rows per call. Those setup costs deserve attention, although their CPU share is unmeasured.

The complete emitter’s five stores and pattern reloads are real, but a weak primary target. Removing one store per complete record eliminates only approximately

\[
0.7434/30=0.0248
\]

store instructions per input row. That is not a convincing explanation for recovering the whole gap.

Likewise, the census records 30,949 wide promotions and 476,120 reconstructed prefix fields across 207 million rows. Promotion and RAW permutation remain costs, but these frequencies do not make them the leading explanation by themselves.

These observations come from the [frozen constructor](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs), [matched assembly](https://github.com/lmmx/mibz2/blob/4adabf1/docs/journal/2026-11-03-compact30-construction-harness/listings/ctor-build-false.asm) and [structural census](https://github.com/lmmx/mibz2/blob/4adabf1/docs/journal/2026-11-03-compact30-constructor-prototype/corpora.jsonl). **They support targeting repeated per-row work; they do not establish its measured cost share.**

The alternative I recommend removes that repeated work rather than making its branches slightly cheaper.

### The construction identity

For source position \(i\), let

\[
y=\phi(i)=32\lfloor i/30\rfloor+i\bmod30.
\]

The useful additional identity is

\[
\phi(i)=65536\left\lfloor\frac{i}{61440}\right\rfloor
       +\phi(i\bmod61440).
\]

Thus:

- The successor’s high 16 bits change only every **61,440 source rows**.
- Its low 16 bits always lie in `0..=65533`.
- A 900,000-row block occupies only **15 such epochs**.

For an input byte \(c=L[i]\), stable counting scatter assigns destination row

\[
r=C[c]+\operatorname{occ}_c(i),
\]

where \(C[c]\) is the bucket start and \(\operatorname{occ}_c(i)\) counts earlier occurrences.

At the beginning of each epoch \(e\), save, for every character,

\[
A_e[c]=C[c]+\#\{i<61440e:L[i]=c\}.
\]

These are snapshots of the **next destination ranks**, obtainable from the constructor’s current cursors.

For any already received destination row \(r\) in bucket \(c\), its successor’s epoch is exactly

\[
h(r)=\max\{e:A_e[c]\le r\}.
\]

Therefore, if its stored temporary field is \(u_r=y\bmod65536\),

\[
y=65536h(r)+u_r.
\]

Repeated snapshot values are important: choosing the **greatest** qualifying epoch correctly handles a character absent for several epochs. Low bits alone would not suffice.

This recovers full successors using a few thousand counts—not a successor table.

### Recommended construction steps

**1. Prepare the same arenas and known RAW groups.**

Use the existing count validation, RAW reserve bound and crossing-group detection. Preallocate crossing RAW groups as today, with zeroed RAW storage.

Maintain one descriptor per character containing:

- A pointer to its next temporary field.
- The end pointer of its current group or character-owned portion of a crossing group.
- The portion’s starting destination rank.
- Its bucket limit and group classification.

An ordinary interior portion has 30 entries. A crossing portion ends at the relevant bucket boundary.

**2. Preserve run detection and complete-record emission.**

Keep the production-derived 16-equal-byte test and current run coalescing.

When a run supplies complete homogeneous groups, emit the existing final records directly. **Do not subsequently normalize those records.** The measured 74.34% bulk coverage is worth preserving.

Split processing at epoch boundaries so the snapshots describe the exact source prefix. Since 61,440 is divisible by both 16 and 30, this does not disrupt the uniformity-test alignment. It introduces at most a small number of extra boundary fragments per block.

**3. Append low labels for everything else.**

For scalar rows and incomplete run fragments, write the low 16 bits of \(\phi(i)\) directly into the destination record’s field slots.

The ordinary scalar operation becomes conceptually:

```text
d = descriptor[c]

if d.write == d.end:
    finish_previous_portion(d)
    prepare_next_portion_or_reject_bucket_overflow(d)

*d.write = low16(phi(i))
d.write += 1
```

There is no ordinary per-row:

- Record-header load.
- RAW test.
- Base subtraction.
- Span test.
- Encoded destination-cursor increment that skips header labels.

The descriptor boundary check also protects bucket limits. Empty or exhausted buckets reject before the field write.

Source labels should be generated per input tile. Within a nonuniform 16-byte tile, split at the possible source-coordinate hole and use simple increments inside each part. Update the source phase once per tile, rather than repeating the current phase calculation and two state stores for every row.

Fragments use the same temporary-field convention. They no longer need separate ordinary, already-RAW and promotion cases while appending.

**4. Finish each portion once.**

For a homogeneous group, recover its first and last full successor labels from the saved epoch counts.

There is a cheap common case: if the latest epoch snapshot satisfies \(A_e[c]\le r_{\text{first}}\), all its received rows belong to that epoch. Their span is automatically at most 65,533.

For an ordinary group, write:

\[
H=(y_{\text{first}}\ll8)\;|\;c
\]

and normalize its fields with 16-bit modular subtraction:

\[
f[j]=(u_j-u_0)\bmod65536.
\]

After establishing that the full span is at most 65,533, these are exactly the oracle’s offsets—even when the group crosses a 16-bit wrap. SSE2 `psubw` implements the subtraction without widening or packing.

For a wide group, allocate its RAW block **at completion**, reconstruct the full labels and write

\[
\text{raw}[j]=(\phi(i_j)\ll8)\;|\;c.
\]

Process RAW entries in contiguous destination intervals belonging to each epoch. Do not search the snapshot history separately for every row.

For a crossing group, finish each character’s portion into its preallocated RAW block, then restore only that portion’s temporary record fields to `0xFFFF`. Different characters own disjoint slots.

**5. Preserve canonicalization and final initialization.**

Retain the existing bitmap-ranking and in-place RAW permutation. Discovery order can change; final RAW order must still be increasing destination-group order.

Initialize partial-record padding, RAW padding and the parking record exactly as today. Publish the arrays only after all temporary fields have been finalized.

### Why the output can be byte-identical

The [frozen oracle](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk.rs#L236) stores \(\phi(i)\) and \(F[r]=L[i]=c\) at the stable-scatter destination. The proposed append preserves precisely that destination order.

Consequently:

- Ordinary headers use the same **first successor**, not merely an equivalent alternative base.
- Normalized fields equal the same full successor differences.
- Crossing and wide classification use the same criteria.
- RAW entries recover the same full encoded successor and byte.
- Existing canonicalization supplies the same RAW indices and array order.

Several implementation obligations are consequential:

- A scalar portion may finish when the **next** occurrence arrives, or at end of input. Its last successor must be recovered from its destination rank and snapshots; the current source position is not necessarily that successor.
- Snapshot ranks must include completed-but-not-yet-finalized portions.
- Crossing portions must never normalize or restore another character’s fields.
- Snapshot history must handle duplicate counts and multiple empty epochs.
- Ordinary normalization must not read an uninitialized header. If overlapping vector loads/stores cover the 60 field bytes, load overlapping inputs before modifying either copy; otherwise some fields could be subtracted twice.
- Span boundaries 65,533/65,534, partial groups, forced RAW, invalid counts and reusable poisoned storage all remain mandatory oracle checks.

These are constructor obligations. The walker and final representation remain unchanged.

### Costs and memory

This design replaces online classification with **one completion operation per pending group**. It also avoids a 32-bit staging buffer: the temporary labels occupy the final `rec` fields.

A straightforward bounded workspace is:

| Workspace | Maximum approximate size |
|---|---:|
| Sixteen epochs × 256 × `u32` snapshots | 16 KiB |
| 256 descriptors at 32 bytes | 8 KiB |
| Existing bitmap at 900,000 rows | 3.7 KiB |
| RAW permutation carry | 128 bytes |

That is approximately **28 KiB**, excluding small stack temporaries, instrumentation and the shared pattern table. Most historical snapshots are cold; the active descriptors and current snapshot are the frequently accessed metadata.

The existing table reservations remain unchanged: approximately 2,108,096 bytes under the cited 98-symbol bound, or 2,411,456 bytes under the 256-symbol bound at 900,000 rows. Input storage is unchanged. These are capacity calculations, **not RSS measurements**.

The price is an additional read and rewrite of the pending ordinary fields. At the existing 25.66% non-bulk row share, that is roughly **1.0–1.1 extra bytes of explicit field loads/stores per input row**, depending on the vector sequence, before other bookkeeping. It is not an estimate of DRAM traffic.

Other risks remain:

- Finalization may revisit a line that has left L1.
- RAW reconstruction can cost more per exceptional group than the present promotion path.
- Descriptor access and completion calls can introduce spills.
- Short blocks pay proportionally more setup.
- Fragment loop setup does not disappear entirely.

This is why I prefer this design over staging full 32-bit successors, but do not regard it as free compression.

### Can it close the gap?

Using the directional ratios from the [INVALID capture](https://github.com/lmmx/mibz2/blob/4adabf1/docs/journal/2026-11-03-compact30-construction-capture.md):

\[
1-\frac{1.06714}{1.31583}\approx18.9\%
\]

constructor CPU reduction is needed, and

\[
1-\frac{1.02}{1.36409}\approx25.2\%
\]

completion reduction is needed.

The second requirement is materially harder. If CPU and completion improved proportionally, merely reaching the CPU ceiling would leave completion around **1.106× production**. Meeting the completion ceiling would instead bring CPU to approximately **0.984× production**. Proportional scaling is an assumption, not a prediction.

A useful conditional screen is:

| If the affected work becomes twice as fast… | Required present cost share |
|---|---:|
| To save 18.9% constructor CPU | 37.8% |
| To save 25.2% completion time | 50.4% of the relevant critical-path time |

The 49-instruction scalar iteration makes substantial cost concentration in the irregular rows plausible. It does not prove either share. Finalization and snapshot overhead must also be deducted from the saving.

**My judgment:** 19% CPU improvement is a credible objective for this algorithm because it removes both instructions and the dependent header-load chain from a substantial minority of rows. A simultaneous 25% completion improvement remains a demanding, uncertain objective. The evidence does not justify forecasting that both gates will pass.

### The bounded investment I would recommend

Submit **this single constructor algorithm** for Guardian review, with the current complete emitter and canonicalizer retained initially. I would not invest in five-store-to-four-store tuning first.

A subsequent correctness-and-compilation prototype would earn further consideration only if:

- `rec` and `raw` are byte-identical across the existing corpus and adversarial checks, including canonical RAW order.
- The ordinary scalar append loses the header/RAW/span dependency chain and reduces its complete iteration substantially—roughly halving the current instruction path is a reasonable structural screen.
- Normalization is vectorized per ordinary group, rather than becoming another expensive scalar pass.
- Bulk coverage remains essentially intact, and workspace stays bounded without a full-table temporary or another scan of \(L\).

If compilation reintroduces repeated classification, heavy per-row phase bookkeeping or scalar normalization, **stop this direction rather than funding a timing capture on its mathematical elegance**.

The strongest reason to proceed is specific: compact30’s frozen coordinates permit cheap temporary halfword appends and exact recovery from sparse occurrence counts. That removes work the current implementation demonstrably performs. Whether it removes enough execution time remains the question a bounded prototype must first make credible.

## Report 206: the guardian's decision, as relayed

I approve Astra's new algorithm for one bounded compile-and-correctness prototype. Not for benchmarking yet.

This is the kind of architectural reconsideration we wanted. Astra hasn't proposed making the existing constructor slightly faster; it has identified repeated work that the constructor may not need to perform at all.

The previous STOP decision still applies to constructor version 1. This is a new design, so it deserves a separate assessment rather than reopening the invalid capture.

### Why this proposal is worth pursuing

The first constructor achieved its intended bulk coverage but still spent too much time constructing tables.

Primary corpus: how constructor v1 processes rows

Bulk 74.34%

Fragments 7.16%

Scalar 18.50%

Astra proposes retaining the established bulk emitter while simplifying the other 25.66% of rows.

The key change is to postpone expensive offset calculations until a record is complete.

| Constructor v1                              | Astra's proposed v2                            |
| ------------------------------------------- | ---------------------------------------------- |
| Recover header for individual rows          | Append temporary 16-bit successor values       |
| Test RAW status during writes               | Classify once at group completion              |
| Calculate offsets repeatedly                | Normalize completed ordinary groups            |
| Promote groups while filling                | Classify wide groups when complete             |
| Maintain encoded destination cursor per row | Maintain a character-specific write descriptor |

This is a credible way to remove significant work from the irregular path.

#### I checked the central mathematical idea

Astra's identity is correct:

\\[ \phi(i+61440)=\phi(i)+65536. \\]

That lets a constructor store a successor's low 16 bits, then recover the high bits using sparse snapshots of character-bucket counts.

The greatest-qualifying-epoch rule is particularly important. It correctly handles characters absent across several consecutive epochs.

I also checked that reconstruction and modular offset subtraction work in an independent small model across random, sparse, run-heavy and epoch-gap inputs. That supports the underlying arithmetic, although it does not establish implementation correctness.

### What still worries me

The idea is sound, but it adds a new deferred-finalization mechanism. Its correctness depends on several precise invariants:

- Every character's pending portion must be finalized exactly once, including portions that never receive another occurrence.
- Already completed bulk records must never be normalized a second time.
- Crossing-group portions belonging to different characters must not overwrite each other's temporary fields.
- Snapshot lookup must choose the greatest qualifying epoch, including ties.
- Every temporary field must be converted to its final representation before `rec.set_len`.
- The ordinary-group span test must distinguish 65,533 from 65,534 exactly.

Those are manageable implementation obligations, not reasons to reject the design.

The economic uncertainty is more serious. The previous invalid capture suggests v2 needs roughly 19% less construction CPU and 25% less completion time than v1. Astra gives a plausible mechanism for reducing CPU instructions, but not measured evidence that enough critical-path time will disappear.

So the next checkpoint should answer a narrow question:

Can this algorithm be implemented correctly while compiling into a materially simpler scalar path and an efficient group-finalization path?

If not, we stop before another expensive measurement.

### Send Claude this

Send the complete Astra response you just received alongside this message, so Claude has the exact specification.

Astra has returned a new architectural proposal for compact30 construction.

**Guardian disposition: DESIGN APPROVED for one bounded compile-and-correctness prototype, with no timing.**

This is constructor v2, a separate research direction. Report 202's STOP remains valid for constructor v1. The previous construction capture remains formally INVALID, and the closed compact30 traversal result remains VALID ADVANCE.

Please archive Astra's complete new response as the next design report, preserving its mathematical identities, algorithm, uncertainties and explicit restrictions.

I authorize implementing **one new constructor variant** based on Astra's deferred-offset proposal.

The required algorithm:

- Preserve the frozen compact30 `Rec`/`RawG` representation, walker, original constructor and oracle.
- Preserve v1's run coalescing and direct complete-record emitter.
- Replace online per-row offset formation for incomplete records with temporary low-16-bit successor writes.
- Capture per-character destination-rank snapshots at source-position epochs of 61,440 rows.
- Recover each successor's high bits from the greatest epoch whose character snapshot does not exceed the destination rank.
- Classify and normalize ordinary groups only when their pending portion is complete.
- Construct crossing and wide RAW groups exactly as specified, including mixed-character portions and canonical RAW ordering.
- Preserve byte-for-byte equality with the oracle, including padding, headers, RAW indices and PARK.

Keep this implementation separate from frozen v1, preferably in a new research-only module or branch. Do not alter `ctor.rs` v1, the walker, production's builder, or the existing benchmark harness.

**Correctness gate:** Reuse the established corpus, adversarial and synthetic suites. Add focused cases for epoch boundaries, absent characters across repeated snapshots, pending portions left open until end of input, crossing-group ownership, ordinary/wide span boundaries, bulk-to-partial transitions, malformed counts, forced RAW and reuse after changing sizes.

Require complete byte equality of `rec` and `raw` against `C30::build` and unchanged walker output. Provide an explicit initialization and bounds-safety argument. Never publish records containing temporary labels.

**Compilation/architecture gate:** Build with rustc 1.97.0 and inspect the actual release assembly.

Show whether:

- the ordinary scalar append loses the repeated header-read, RAW-test and span-test dependency chain;
- source-coordinate updates are amortized over input tiles;
- ordinary group normalization compiles to bounded vectorized work, rather than a substantial new scalar loop;
- the existing bulk emitter and its coverage remain intact;
- workspace is bounded without a full successor table, F array or second scan of L.

Report static instruction-path observations and untimed structural counters. Do not present instruction counts as cycle measurements or forecasts of speedup.

If the implementation fails correctness or compilation retains the expensive per-row classification machinery, return a STOP recommendation rather than silently developing another variant.

**Not authorized:** any construction timing, i9 run, benchmark-protocol amendment, walker tuning, production integration or end-to-end performance claim.

Return the implementation pin, correctness evidence, initialization proof, structural counters and assembly observations for Guardian review before any performance experiment is considered.

### My decision in context

Compact30 traversal

VALID ADVANCE

Constructor v1

STOP

Constructor v2 design

Prototype approved

This is a meaningful opportunity to improve construction without changing the part of compact30 that already succeeded.

Only Claude needs the implementation handoff. Astra has completed this design assignment, and the researcher needn't be involved until there is new code to review.

The critical discipline is to prove the new algorithm and inspect its compiled structure before committing to another i9 capture.
## Report 207: the researcher's review of the v2 design, as relayed

Kept as received. The researcher's checks are of the mathematics, done with an independent
counting-scatter model, not tests of a Rust constructor. Two of its requirements became
explicit prototype obligations in report 208:
- SIMD finalisation must stay inside a record's field bytes 0–59, because the header starts
  at byte 60.
- A group must be classified as wide or ordinary from its fully reconstructed labels before
  any modular subtraction. The exact boundary cases are source positions 3 → 61,440
  (ordinary) and 2 → 61,440 (RAW).

### My assessment of Astra's second design

This is a substantially better research proposal than simply tuning the first constructor. I think it merits consideration for one bounded correctness-and-compilation prototype, although I would not yet authorize another performance capture.

The distinction is that Astra is proposing to eliminate work, rather than make the existing work a little cheaper.

The first constructor calculates the final representation as it scans each byte. That forces it to repeatedly inspect record headers, calculate offsets and decide whether a group has become too wide.

Astra's new idea is to write a simpler intermediate value into the final record, then finish each group once its contents are known.

#### The mathematical idea checks out

The key identity is:

\\[ \phi(i)=65536\left\lfloor\frac{i}{61440}\right\rfloor+\phi(i\bmod61440). \\]

Because 61,440 is exactly \\(30\times2048\\), the encoded successor label advances by exactly 65,536 at each boundary.

Consequently, storing its low 16 bits loses only one piece of information: which 61,440-row epoch it came from.

Astra proposes recovering that epoch from snapshots of each character's destination cursor. This works because occurrences of the same character are scattered into their destination bucket in their original order.

I independently checked the reconstruction against a counting-scatter oracle on several synthetic patterns, including the maximum permitted block size of 983,010 rows, absent characters across multiple epochs, random alphabets and runs crossing epoch boundaries. The reconstructed successor labels matched throughout.

These are mathematical/model checks, not tests of an implemented Rust constructor.

An important detail is that Astra correctly specifies the greatest qualifying epoch. When a character is absent for several epochs, its cursor snapshot repeats. Choosing the earliest matching snapshot would reconstruct the wrong successor.

So the central idea is not hand-waving. It exploits a genuine arithmetic property of compact30's frozen representation.

#### The two technical risks I'd take most seriously

1\. The temporary fields must be completely finalized before the record becomes readable.

The existing [`Rec`](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk.rs?utm_source=chatgpt.com#L73-L84)[ layout](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk.rs?utm_source=chatgpt.com#L73-L84) has 30 `u16` fields occupying 60 bytes, followed by a four-byte header.

Astra mentions overlapping SIMD loads, but I'd require a more explicit rule here: a straightforward fourth 16-byte load starting at byte 48 would also read the header, which may still be uninitialized. A corresponding store would overwrite it.

The implementation must either handle the final 12 bytes separately or use an overlapping load/store confined to bytes 0–59, with the necessary inputs loaded before overlapping stores modify them.

This isn't merely a vectorization detail. Given the constructor's unsafe arena initialization, it's a memory-safety obligation.

There is also a useful property: the temporary low-16-bit labels never equal `0xFFFE` or `0xFFFF`, because they range only from 0 to 65,533. Those two reserved values therefore cannot be confused with a legitimate temporary label.

2\. Wide-group detection must use full recovered labels, not modular subtraction.

I checked an exact boundary case, with an ordinary group's first source row at position 3 and last at 61,440:

| First source position | Last position | Encoded span | Classification |
| --------------------- | ------------- | ------------ | -------------- |
| 3                     | 61,440        | 65,533       | Ordinary       |
| 2                     | 61,440        | 65,534       | RAW            |

Both cross a 16-bit epoch boundary.

The modular subtraction gives the correct final field offsets only after the constructor has established that the full span is at most 65,533. If it first subtracts and then classifies by the truncated result, it could misclassify a much wider group.

Astra's proposed order gets this right. The implementation and tests must preserve it.

#### The performance case is plausible, not proven

The source confirms Astra's structural figures:

### 74.34%

Complete-record rows retained

### 18.50%

Scalar rows targeted

### 7.16%

Fragment rows targeted

I also inspected the cited assembly around `23c99–23d1d`. It does contain the repeated cursor arithmetic, header load, RAW classification and offset computation that Astra identifies. So the proposed optimization targets a real dependency chain in the generated code.

But there's an important tradeoff.

The new algorithm postpones classification and normalization; it does not make those operations disappear altogether.

For the roughly 25.66% of rows not handled by the bulk emitter, temporary labels must subsequently be read and rewritten. That's approximately another 1.03 bytes of explicit record reads and writes per input row, before any other finalization overhead. It isn't necessarily DRAM traffic, but it is additional work and can revisit cache lines that have been evicted.

That is especially relevant because the harder requirement is not the 18.9% CPU reduction but the 25.2% completion-time reduction under four-worker contention.

So I broadly agree with Astra's judgment: the instruction-count saving is credible, but the four-worker completion target is significantly less certain.

#### What I would ask the guardian to authorize

I'd support one bounded, untimed correctness-and-compilation prototype, with four conditions:

1\. Preserve the existing compact30 record format, walker, bulk emitter and final RAW ordering. Verify complete byte-for-byte equivalence with the oracle, including forced RAW, malformed counts, reused allocations, partial records and both wide-offset boundaries.

2\. Require a source-level initialization proof for temporary fields and headers. Include targeted tests above 61,440 rows, where the new epoch logic actually operates. The previous 942-case Miri quick suite is insufficient coverage of this design.

3\. Inspect the compiled ordinary scalar append path. It should genuinely eliminate the per-row header load, RAW test, offset subtraction and span check. If those dependencies return in another form, the architectural rationale weakens considerably.

4\. Inspect finalization as carefully as the append path: vectorized subtraction must not read uninitialized header bytes or normalize overlapping fields twice. The finalizer must not become another expensive scalar walk.

No timing yet. First establish that the proposed savings survive compilation without introducing a costly finalization path.

#### Verdict

I would advance Astra's second design to one correctness-and-compilation prototype, not to a benchmark.

The mathematical insight is valid, the proposed workspace is small and bounded, and the design attacks the specific repeated work visible in the first constructor's machine code.

That is materially different from trying another five-store-to-four-store optimization.

But I would not describe the performance target as likely to be met. This architecture trades immediate per-row computation for deferred record processing, and that trade may not pay under four-worker contention.

The right next question is therefore very specific:

Can we turn the expensive scalar scatter into a genuinely short temporary-label append, without paying the saved cost back when the groups are finalized?

That's worth one carefully bounded prototype. It is not yet a reason to reopen the i9 measurement campaign.

## Report 208: the guardian's addendum, as relayed

The researcher's review strengthens the case for constructor v2. I agree with their assessment, and my earlier authorization stands: one correctness-and-compilation prototype, with no benchmarking.

The important point is that Astra and the researcher have independently arrived at the same conclusion: the epoch-snapshot arithmetic is sound, and the proposed design targets genuine repeated work in constructor v1. Neither claims the required performance improvement is guaranteed.

I also checked the frozen `Rec` layout on GitHub. The header begins at byte 60, so the researcher's SIMD warning is valid: a 16-byte load beginning at byte 48 would read four header bytes, potentially before initialization.

#### Two details to emphasize to Claude

The original prototype brief already covers most of the researcher's concerns. I'd send Claude just this short addendum, rather than another complete specification.

The independent researcher has reviewed Astra's constructor-v2 design and supports the Guardian's existing authorization for one untimed correctness-and-compilation prototype.

Two implementation-critical requirements deserve explicit emphasis:

**1. SIMD finalization must never read an uninitialized header or process fields twice.** In the frozen 64-byte `Rec`, the 30 `u16` fields occupy bytes 0–59 and the header starts at byte 60. A 16-byte load at byte 48 crosses into the header. Use bounded finalization, or overlapping field-only loads with all overlapping source values captured before modifying them. Partial records must never read uninitialized fields or normalize padding.

**2. Wide-group classification must use fully reconstructed successor labels before modular subtraction.** Include these exact epoch-crossing boundary tests:

- Source positions 3 → 61,440: encoded span 65,533, ordinary.
- Source positions 2 → 61,440: encoded span 65,534, RAW.

The researcher also independently verified the greatest-qualifying-epoch rule, including duplicate snapshots and absent characters across epochs, up to the maximum supported block size.

Retain the original compile-only structural gates: a genuinely shorter scalar append path, bounded vectorized finalization, preserved bulk emission and exact oracle byte equality.

No new algorithm, timing, i9 work or additional experimental variants are authorized. These are clarifications to the already-approved v2 prototype.

### Current status

| Work                    | Status                             |
| ----------------------- | ---------------------------------- |
| Compact30 traversal     | VALID ADVANCE                      |
| Direct constructor v1   | STOP                               |
| Astra constructor v2    | Approved for one untimed prototype |
| New performance capture | Not authorized                     |

No further message to Astra or the researcher is needed. Claude can implement the already-approved design with these two safety requirements made explicit.

The next meaningful decision will be based on v2's actual correctness results and compiled scalar/finalization paths—not another theoretical performance prediction.
