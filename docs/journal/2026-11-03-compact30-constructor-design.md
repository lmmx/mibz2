# Compact30: Astra's direct-constructor design (report 188) and the guardian's decision (report 189)

Both reports are kept as relayed. Report 188 answers the brief in
[2026-11-02-compact30-constructor-brief.md](2026-11-02-compact30-constructor-brief.md)
(report 186). Report 189 is the guardian's review of it, which authorises one bounded
compile-and-correctness prototype and nothing beyond it.

## Report 188: Astra's design

**I recommend one algorithm: a single-pass constructor that coalesces equal-byte runs, emits complete ordinary records from a reusable pattern, promotes partial groups when necessary, and finally puts RAW groups into canonical order in place.**

This materially revises my earlier suggestion. With only **6.714% construction overhead available**, a generic per-row promoter is not a convincing starting point. The constructor needs a substantially cheaper path for the runs that production already exploits.

I verified that the attachment matches the brief at `f567817`, inspected both builders at `f1dafa3`, and checked the archived `VALID ADVANCE` analysis. I performed only static arithmetic and identity checks—no implementation or timing.

**1. Construction algorithm**

The useful identity is stronger than simply advancing encoded coordinates:

\[
\phi(i+j)-\phi(i)=j+2\,[\,i\bmod30+j\ge30\,],\qquad 0\le j<30,
\]

and

\[
\phi(i+30)=\phi(i)+32.
\]

Suppose a contiguous equal-byte run in L supplies an entire destination group of 30 rows. Its successors are 30 consecutive source positions. Consequently:

- The group is homogeneous.
- Its span is at most 31, so it cannot be wide.
- Its offsets depend only on the source position modulo 30.
- For successive complete groups in that run, **all thirty offset fields remain identical; only the header changes**, by `32 << 8`.

This produces exactly the frozen representation. It does not change its interpretation or layout. [Frozen oracle builder](https://github.com/lmmx/mibz2/blob/f1dafa3/examples/c30walk.rs#L226).

Prepare a shared, read-only table of thirty 64-byte patterns. Pattern `p` contains:

```text
fields[j] = j + 2 * (p + j >= 30), j = 0..29
header = 0
```

The table occupies 1,920 bytes.

Preserve production's initial 16-byte uniformity test. When it succeeds, coalesce subsequent 16-byte chunks containing the **same** byte. Retain the first failing chunk's loaded words for the next iteration rather than reading them again. Nonuniform chunks retain a scalar scatter path. This preserves the original fast-path coverage while exploiting longer runs. [Production constructor](https://github.com/lmmx/mibz2/blob/f1dafa3/src/bwt.rs#L253).

Conceptually:

```text
validate block size and counts
allocate record arena, bounded RAW arena and zeroed RAW bitmap
initialize encoded bucket cursors and encoded bucket limits
pre-create the distinct bucket-crossing RAW groups

scan L in the production 16-byte units:
    if the unit is uniform:
        coalesce consecutive uniform units containing the same byte
        check that the whole run fits its destination bucket
        emit_run(byte, length, source_label, source_phase)
    else:
        scatter its bytes individually, checking bucket limits

handle the remaining input bytes
require every bucket cursor to equal its limit
initialize unused fields of the final ordinary record
write PARK
canonicalize RAW ordering
publish the completed arrays
```

`emit_run` divides its destination range into:

1. A prefix completing an existing partial group.
2. Zero or more complete groups.
3. A suffix beginning a new partial group.

The prefix and suffix use the general fragment writer described below. Complete groups use the pattern emitter:

```text
pattern = patterns[source_phase]
header = (source_label << 8) | byte

for each complete group:
    write pattern with this header
    header += 32 << 8
    advance destination by one Rec

source_label += 32 * number_of_complete_groups
```

For natural construction, a complete group inside the checked run needs **no header read, RAW test or span check**.

On baseline x86-64, the intended implementation holds the pattern in four XMM registers. The fourth contains six offsets and the header. Each iteration performs four aligned 16-byte stores; a packed dword addition increments only the header lane by 8,192. This is an explicit code-generation target, not an observed listing.

For partial groups, emit exactly the required fields, using bounded vector/scalar stores. Never overstore into a header or another group.

**Coordinate maintenance does not require division per row.** Keep encoded destination cursor `x`; its group and slot are `x >> 5` and `x & 31`. For a run of length `m`:

\[
x_{\rm end}=x+m+2\left\lfloor\frac{(x\bmod32)+m}{30}\right\rfloor.
\]

Compare that endpoint with the encoded bucket limit. The division is once per coalesced run; the single-row path uses an increment and a conditional skip over the two holes.

For source labels, maintain `Y = φ(i)` and `p = i mod 30`. A fragment of at most thirty rows advances them with one wrap adjustment. A complete thirty-row group adds 32 to Y and leaves p unchanged. Within a nonuniform input tile, its labels are `Y + patterns[p].fields[j]`, so the scalar fallback also needs no division per source row.

**2. Allocations and peak memory**

Use the proven natural RAW bound:

\[
R_{\max}=\min\left(G,\sigma\left\lfloor\phi(n-1)/65534\right\rfloor+\sigma-1\right).
\]

Reserve that capacity once before scattering. Allocate and initialize an actual `RawG` only when a group becomes RAW; do not zero the entire reserved RAW capacity.

| Storage | Requested size at 900,000 rows |
|---|---:|
| `rec`, including PARK | 1,920,064 bytes |
| RAW capacity, general 256-symbol bound | 491,392 bytes |
| Encoded cursors and limits | 2,048 bytes |
| RAW-group bitmap | 3,752 bytes |
| One permutation carry record | 128 bytes |
| Shared static patterns | 1,920 bytes, shared across workers |

The requested table capacities plus working scratch total **2,417,384 bytes per worker**, before a few scalars, alignment and allocator overhead. With the common 900,000-byte L buffer, that becomes approximately **3.317 MB**, versus approximately **4.502 MB** for production's T-vector, its cursor arrays and L.

For σ≤98, table capacity plus the same scratch is **2,114,024 bytes**.

These are allocation requirements, not RSS measurements. Actual capacities, allocator rounding and retained storage must be reported.

Important allocation rules:

- Reserve ordinary records as uninitialized storage. Do not fill the entire arena with `0xFFFF` before overwriting it.
- Initialize every returned field before publishing the array.
- Reuse sufficient capacities across blocks.
- When an allocation must grow, its previous table is disposable: release the insufficient allocation before allocating its replacement, avoiding simultaneous old/new arenas.
- Do not retain an oracle successor table or F array.

Array equality requires identical lengths and element bytes. Spare capacity need not equal the oracle's capacity and must be accounted for separately.

**3. Memory traffic**

The principal saving is concrete: a complete ordinary group requires **64 output bytes for thirty rows**, rather than production's 120 bytes. The input scan also supplies F directly, eliminating production's additional `ll[s]` byte reads.

Let:

- `R` be the actual number of RAW groups;
- `T` be their number of valid rows;
- `W` be the number of promoted wide groups;
- `P` be the total number of already-written fields reconstructed during promotion, with `P ≤ 29W`.

The traffic accounting is:

| Operation | Logical traffic |
|---|---|
| Final representation | `64(G+1) + 128R` bytes written |
| Previously written ordinary prefixes later promoted | Additional `2P` bytes written and `2P` read |
| Zero-initialized RAW groups subsequently filled | Additional `4T` bytes written |
| Header replacement during promotion | Additional `4W` bytes written |
| RAW canonicalization | At most `128R` array bytes read and `128R` written, plus carry-buffer traffic |
| Canonical indices and temporary tags | A few words per RAW group |
| Bitmap | One small clearing pass, updates per RAW group, one final word scan |

There are also cursor accesses, header reads on the general path, and reads of the small static patterns. Complete-record batches can keep their pattern in registers across all their records; they do not require sixty pattern bytes to be loaded again for every group.

At the brief's density of roughly 240 RAW groups per million rows, the RAW-array permutation contributes at most approximately **0.0614 bytes/row read plus written**, with up to a similar amount of carry-buffer traffic. RAW ordering is therefore unlikely to dominate by byte volume, although its CPU cost still belongs inside the construction boundary.

The writes retain the 256 bucket streams. Coalesced runs turn parts of those streams into contiguous, complete-record writes. Partial groups remain partially written cache lines.

These are program-level byte counts. They are **not DRAM-traffic predictions**: temporal stores still incur write allocation and eventual writeback according to cache state. I recommend ordinary cached stores, with no new non-temporal-store policy.

**4. Promotion, crossing groups and exact RAW ordering**

Crossing groups are known before reading L. For every internal boundary between nonempty character buckets, mark its group RAW if the boundary is not a multiple of thirty. Deduplicate boundaries falling in the same group.

Creating a RAW group:

1. Append a zeroed `RawG`.
2. Set all thirty record fields to `0xFFFF`.
3. Set the record header to `RAW_BIT | provisional_index`.
4. Set its bit in the RAW bitmap.

Each subsequent scatter writes:

```text
raw[provisional_index][destination_slot] =
    (source_label << 8) | current_byte
```

This naturally handles crossing groups containing several different F bytes.

For an ordinary partial group, its first destination row establishes the header base. Later fragments read that base and inspect their **last** source label. All labels in the fragment increase, so one endpoint check covers every field.

If that last label exceeds `base + 65533`, promote **before writing the fragment**:

```text
save the old base and F byte
append a zeroed RawG
for each of the k previously populated fields:
    raw[k] = ((old_base + old_offset[k]) << 8) | old_F
replace all thirty fields with 0xFFFF
install provisional RAW header and set bitmap bit
write the incoming fragment as RAW entries
```

At most 29 old fields need reconstructing. No overflowing offset is ever truncated into u16.

A useful clarification to the brief: **wide status can be established before the final row arrives**. Once a monotone prefix exceeds the limit, the completed group must be wide. Ordinary status requires completion; wide status does not.

Discovery-order allocation does not satisfy the oracle's ordering. Canonicalize after all scatter writes:

```text
rank = 0
for g in increasing order, enumerated from RAW bitmap:
    old = rec[g].header's provisional index
    raw[old][31] = rank
    rec[g].header = RAW_BIT | rank
    rank += 1

permute raw in place so each record reaches its stored destination rank
clear raw[*][31] back to zero
```

Word 31 is always padding in the frozen representation. Its use here is temporary constructor scratch; it is restored before any array is returned or walked. Word 30 remains zero throughout.

Use cycle rotation with one `RawG` carry buffer. The destination tag moves with its record. A record whose tag equals its current index is already placed; completed cycles therefore need no separate visited array.

This gives the exact oracle ordering without:

- Another pass over L.
- An n-entry successor array.
- Sorting or repeatedly inserting 128-byte records.
- A second RAW arena.
- Scanning every 64-byte ordinary record to discover exceptions.

The bitmap scan costs `O(G/64 + R)`.

**5. CPU-cost risks and the 1.06714× budget**

The archived conservative budget is:

\[
1.847(1-0.8431259318)-0.25
=0.0397464\ {\rm CPU\!-\!s/GB}
\]

of additional construction cost. Dividing by 0.592 gives **b ≤ 1.067139…**, consistent with the brief's **1.06714**. My earlier discussion of tolerating 10–20% slower construction does not apply to this measured result. [Frozen analysis](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4/c30-analysis.json).

The proposed algorithm has a plausible route to that budget because its complete-record path removes work:

- No destination-byte lookup in L.
- No per-row base recovery or span check.
- No per-record RAW test in naturally ordinary complete-run groups.
- Fewer output bytes and fewer stores.
- One cursor/limit operation per coalesced run rather than per sixteen rows.

But I cannot responsibly predict that it meets 1.06714.

The most important missing quantity is **the fraction of rows emitted as complete records inside coalesced runs**, not simply the fraction using production's sixteen-equal-byte path.

For example, depending on destination alignment:

| Coalesced run length | Rows emitted through the complete-record path |
|---:|---:|
| 16 | 0 |
| 32 | 0–30 |
| 64 | 30–60 |
| 128 | 90–120 |
| 256 | 210–240 |

Thus "most rows use the sixteen-byte optimization" does not establish that the main saving is large.

The risks I expect to dominate are:

1. **Too little complete-record coverage.** Short uniform runs and mixed tiles retain header access, encoded-cursor arithmetic and fragment handling.
2. **Poor partial-write code generation.** Turning those fragments into scalar u16 stores and branch-heavy loops could consume the entire allowance.
3. **Expensive scalar fallback.** Removing `ll[s]` replaces one load, but does not automatically pay for all classification and offset arithmetic.
4. **Run-scanner overhead.** Failed lookahead must be reused; cursor and source-state updates must actually be amortized.
5. **Accidental initialization passes.** Zeroing all records, or all reserved RAW capacity, would add work absent from this design.

RAW promotion and canonicalization should be secondary on the cited corpus density, but must remain measured costs.

**My judgment is conditional:** this algorithm is plausible enough for review because it provides a substantially cheaper bulk operation. If complete-record coverage is low, the principal reason to believe it can meet the budget disappears. I would not then defend the generic promotion path merely because the table is smaller.

**6. Correctness and byte-identity plan**

The proof obligations correspond directly to the frozen oracle:

- Encoded cursors enumerate exactly the original counting-scatter destination slots.
- Within a homogeneous group, the first occurrence supplies exactly the oracle's base.
- Endpoint checking is equivalent to checking every ordinary offset because successors increase.
- Complete-run patterns are the exact differences `φ(i+j)−φ(i)`.
- RAW entries use the same successor label and F byte as the oracle.
- Canonicalization assigns RAW indices in increasing group order.
- Partial-record fields, RAW padding and PARK have precisely the prescribed values.

I statically checked the pattern identity for every phase and slot, and the thirty-row advance identity across the supported range. Those checks establish arithmetic identities, not implementation correctness.

The eventual correctness work should compare **array lengths and every byte** against `C30::build`, before walking. Include the eleven hash-checked corpora and the existing adversarial set, plus direct L/counts cases covering:

- Every partial-record length and every internal bucket-boundary offset.
- Source phases 0–29 and all destination slots.
- Equal-run lengths around 16, 30, 32, 60 and longer runs.
- Spans exactly 65,533 and 65,534.
- Promotion after different prefix lengths, including 29 fields.
- A fragment whose endpoint first exceeds the limit partway through a run.
- RAW discovery order different from group order, including nontrivial permutation cycles.
- One-symbol and 900,000-row blocks.
- Repeated construction with changing sizes, to expose stale fields or padding.

The pinned oracle also supports forced RAW groups. Preserve its exact `mix` predicate for the test configurations. Pre-mark selected groups RAW; route affected complete groups through the RAW writer. Array equality does not depend on whether a forced group would later prove wide. If classification counters must also match, preserve the oracle's crossing-before-wide-before-forced precedence.

Forced-RAW tests need a larger allocation bound—up to G RAW groups—and must not be confused with the natural-construction memory bound or timing path.

Finally, run the **unchanged frozen walker**, compare its output, and compare the arrays again after marker restoration. Decoded-output equality alone is insufficient.

Publish initialized `rec` only after all bucket cursors equal their limits and all padding is written. Malformed counts must fail before an out-of-bounds write; this design does not change production's fallback semantics.

**7. What the later construction-cost comparison needs**

The later registration should measure one constructor against the pinned `build_one_pass` on identical L/counts inputs, with matching compiler and target settings.

The timed candidate boundary must include:

- Cursor and bitmap initialization.
- Crossing-group setup.
- Input scanning and all record writes.
- Promotion.
- RAW ordering and header patching.
- Padding, PARK and final validation.

No oracle preprocessing, sorting, copying or allocation work may disappear outside the accounting boundary.

Report:

- Paired **total CPU-cost ratios**, especially with four physical cores, rather than averages of per-block speed ratios.
- One-worker results separately.
- Reused-capacity behavior and cold/growth costs.
- Actual capacities and peak RSS, without live oracle arrays contaminating the candidate's memory measurement.
- Untimed counts of coalesced runs, complete-record rows, partial-fragment rows, scalar rows, promotions, reconstructed prefix fields and moved RAW groups.
- Generated code for the complete-record and partial-fragment paths.

The economic screen is **b ≤ 1.06714**, with uncertainty handled conservatively in the separately registered comparison. Passing it would not establish the decoder saving: the frozen traversal used a copied oracle image, and a direct constructor leaves a different cache state. That issue remains for subsequent integration assessment, without revising the closed traversal result.

**The algorithm to take forward is this run-coalescing constructor with complete-record emission and sparse RAW canonicalization. Its decisive uncertainty is whether enough rows reach the cheap complete-record path to pay for the general path within the 6.714% allowance.**

## Report 189: the guardian's decision

Astra has delivered a credible constructor design. I approve moving to one bounded compile-and-correctness prototype, but not to performance benchmarking yet.

The most important improvement over Astra's earlier proposal is that it no longer treats construction as a generic succession of per-row updates. It exploits the structure of bzip2's last column: long equal-byte runs can fill complete 30-row records using a reusable pattern.

That's the right algorithmic direction given our tight 1.06714× construction-cost ceiling.

The design also preserves the two things we cannot compromise: the validated compact30 record format and byte-for-byte equivalence with the oracle builder, including RAW-group ordering.

There is, however, one implementation decision I want resolved carefully before granting Claude carte blanche.

### What I would require from the prototype

The key uncertainty is how many rows actually reach the cheap complete-record path.

Production already handles sixteen equal bytes efficiently. Astra's proposed advantage depends on coalescing enough of those chunks to fill whole 30-row records. A constructor could be beautifully designed yet lose if most work falls into partial fragments.

So the prototype must collect untimed structural counters before we invest in benchmarking:

| Counter | Why it matters |
|---|---|
| Rows emitted through complete-record patterns | Determines how much work reaches the proposed fast path |
| Rows written through partial fragments | Estimates the less efficient path's importance |
| Rows handled by scalar fallback | Exposes the cost of nonuniform input |
| Number and lengths of coalesced runs | Tests whether run coalescing pays off |
| Wide promotions and reconstructed fields | Quantifies exceptional construction work |
| RAW groups moved during canonicalization | Quantifies final ordering work |

I would also insist on two correctness properties.

Memory initialization must be explicit. Astra suggests allocating uninitialized records to avoid a redundant clearing pass. That's reasonable for performance, but Claude must prove every field, header and padding byte is initialized before exposing the representation. No reliance on an uninitialized `Vec<Rec>` length or speculative reads.

RAW canonicalization must be tested independently. The proposed in-place permutation uses word 31 of each RAW group as temporary metadata. This can work because that word is padding, but the cycle algorithm must demonstrably restore every padding word to zero and preserve the exact oracle order.

Neither point demands a redesign. They are implementation obligations.

### My decision

- **Approved:** one compile-and-correctness constructor prototype.
- **Not authorized:** construction timing, i9 work, walker modifications or production integration.

Claude may implement Astra's specified design, including its run coalescing, complete-record patterns, bounded partial-fragment writer, early wide promotion and in-place RAW ordering.

I would not require Claude to reproduce Astra's proposed four-XMM-register sequence exactly. That's a code-generation hypothesis, not part of the format's correctness contract. Inspect the actual generated code and report whether the intended amortized fast path exists.

### The instruction

Astra's direct-constructor design passes the guardian's architectural review. I authorize one bounded compile-and-correctness prototype based on that design.

Implement the run-coalescing compact30 constructor with reusable complete-record patterns, partial-fragment handling, early wide-group promotion and canonical RAW ordering. Preserve the frozen Rec/RawG layout and the unchanged walker. Do not add another constructor variant.

Correctness is the first gate. Require byte-identical `rec` and `raw` arrays against `C30::build`, including header indices, partial-record initialization, RAW padding and PARK. Cover the 11 established corpora, adversarial inputs, forced RAW, span boundaries at 65,533/65,534, all source phases, destination offsets, promotion prefix lengths and nontrivial RAW permutation cycles. Check unchanged walker output and restoration.

Implement uninitialized allocation only where its complete-initialization proof is clear. Otherwise use a safe initialization approach and explicitly report its additional cost. Do not introduce undefined behaviour to pursue the construction budget.

Add untimed structural counters: complete-record rows, fragment rows, scalar-fallback rows, coalesced-run lengths, wide promotions, reconstructed fields, RAW permutation moves and actual capacities. These are diagnostic counts, not performance measurements.

Compile using the reviewed toolchain and inspect the real complete-record and fragment machine code. Report whether the intended bulk fast path survives compilation, without optimizing the walker or inventing another construction strategy.

Keep this research-only, separate from production. No i9 benchmarking or constructor performance capture is authorized. Return the source pin, correctness evidence, structural counters, memory-safety argument and assembly observations for review.

The existing VALID ADVANCE traversal result is closed and must remain unchanged. A later construction-cost experiment requires its own separately approved protocol.

The researcher's blog review can proceed independently. It doesn't need to block this technical work, and the constructor prototype should not introduce performance claims into the already published account.

One final perspective: the important milestone is not merely that compact30 won at four workers. We now have a specific, implementable explanation of how its compact representation might be built at competitive cost. The next evidence should tell us whether that algorithm's cheap path is common enough—and its slow paths controlled enough—to justify paying for another hardware experiment.
