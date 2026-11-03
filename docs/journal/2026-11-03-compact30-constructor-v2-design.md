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