# External research report 156: the researcher's theoretical review of compact navigation (candidate 3): a 30-halfword, four-byte-header record with relabelled rows, a worst-case storage bound, and a bounded-staging direct constructor (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with reports 153–155.
- **Source:** the independent researcher of reports 138–155.
- **In reply to:** the maintainer's instruction to treat each candidate in turn as report 154
  did. That instruction for this candidate was not relayed separately.

**Evidence cutoff:** 2 November 2026. Review of PR #130 at `9d22ac9` and its recorded
disassembly, plus arithmetic and invariant checks; no compilation of a decoder and no
benchmark. The proposed layout is unimplemented.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
## 3. Compact navigation: there is a stronger theoretical candidate

**The useful next advance is not another replacement for `shrx`. It is a small layout change that removes the packed-field extraction problem.**

I can sharpen my earlier 28-entry proposal into **30 directly loadable 16-bit offsets in each 64-byte record**, with the header reduced to four bytes. Relabelling rows makes both the selected offset and its header directly addressable, **without division or a directory lookup during navigation**.

There is also a useful new bound: for a full 900,000-row block, this proposed representation requires **at most 2.412 MB of logical table storage for any byte alphabet**, against 3.6 MB for the shipped table. With at most 98 used byte values—the current corpus’s recorded upper limit for full blocks—the bound improves to **2.109 MB**.

Those are **derived storage bounds, not measurements or speed predictions**. The proposed layout is unimplemented. The existing `k` and `j` remain the only compiled packed-state candidates, and neither has an i9 timing result.

I reviewed PR #130 at `9d22ac9164d0bba46a4942cf4c8001e40d75eca4`, including its implementation and recorded disassembly. I also ran arithmetic and invariant checks, but did not compile a decoder or run a benchmark.

## 1. What the existing `k` and `j` have genuinely accomplished

The recorded ordinary-step counts are:

| Implementation | Instructions | Data loads | Stores |
|---|---:|---:|---:|
| Shipped reference `o` | 16 | 3 | 2 |
| BMI2 FOR-14 baseline `b` | 37 | 5 | 3 |
| Packed-state `k` | **31** | **4** | **2** |
| Packed-state plus multiplication `j` | **27** | **4** | **2** |

These are compiled-code observations, not performance estimates. The new arms remove an actual lane-state load/store pair, shrink each lane from 40 to 32 bytes, and add no recorded spills or dependent table lookups. Their correctness checks passed. 

The packed-state identity is sound:

\[
q=\mathrm{row}+(\mathrm{carry}\ll20),
\]

and, on an ordinary record,

\[
q_{\mathrm{next}}
=
\underbrace{\mathrm{base}+(F[\mathrm{row}]\ll20)}_{\text{header low word}}
+
\mathrm{offset}.
\]

Because `base + offset` is a valid row below \(2^{20}\), no carry enters the character field. The generated code uses a memory-source addition to form the next token. This was a worthwhile algebraic simplification, not a cosmetic instruction-selection change.  

**But most remaining extraction work is still imposed by the 14-bit layout.**

For every ordinary step, the walker must derive:

\[
i=q\mathbin{\&}31,\quad
a=14i,\quad
p=a\gg3,\quad
s=a\mathbin{\&}7,
\]

then load at byte offset \(p\), shift by \(s\), mask to 14 bits, and combine the result with the header. 

Two shortcuts do not work:

* The shift-count mask is not redundant merely because `SHRX` masks its count. A 32-bit `SHRX` masks to **five bits**, whereas this extraction needs the remainder modulo eight. For slot 1, the product is 14, but the required shift is 6—not 14. [Intel](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-2b-manual.pdf)
* A two-byte load cannot replace the existing four-byte load for all slots: a 14-bit field starting four or six bits into the first byte extends beyond those two bytes. The current four-byte loads already remain within their 64-byte records, so there is no inherent cache-line split to eliminate. 

### `k` versus `j` is still an empirical question

`j` replaces the multiply-by-14 expansion with one multiplication. The recorded model trades a roughly two-cycle arithmetic dependency for a three-cycle multiplication, while reducing instruction and execution work. Some of the removed register copies may already be eliminated by the processor, so four fewer instructions do not mean four fewer executed arithmetic operations.  [Agner Fog](https://agner.org/optimize/instruction_tables.pdf)

With 32 interleaved chains, either side of that trade could matter. I would not select `k` solely for its shorter dependency, or `j` solely for its shorter listing.

Also, the header and payload addresses remain independent. The memory-source addition appearing later in the listing does **not** turn the header into a pointer lookup dependent on the extracted offset. We should preserve that distinction when modelling the path. 

## 2. One correction to the optimistic compact-table story

The frozen implementation produced these per-row times:

| Walk | One block | Four blocks | Increase |
|---|---:|---:|---:|
| Control `o` | 1.275 ns | 1.991 ns | 0.716 ns |
| Frozen FOR-14 `h` | 2.676 ns | 2.899 ns | 0.223 ns |

The smaller increase for `h` is real. Its much worse absolute four-core performance is also real. 

However, **we cannot treat the difference between those increases as a fixed memory saving that will remain unchanged when navigation becomes faster**.

The old compact walker also processes rows more slowly. Even with the same requests per row, that reduces requests issued per second. Less request pressure can itself reduce contention. Faster `k`, `j`, or a different layout could generate more pressure and therefore exhibit a different concurrency penalty.

This does not negate the representation’s potential. It prevents this unjustified projection:

> “Subtract the new instruction saving from `h`, and retain all of its favourable four-core behaviour.”

Against the recorded four-core medians, reaching parity would require reducing `h`’s time by approximately **0.908 ns/row, or 31.3%**—equivalent to about **45.6% higher throughput**. That arithmetic describes the required improvement, not what `k` or `j` will deliver.

The frozen negative result therefore remains intact. A promising successor needs a substantially cheaper access primitive, not just an appealing storage ratio.

## 3. A better neighbouring layout: 30 halfword fields and a four-byte header

My earlier proposal kept the eight-byte header and fitted 28 sixteen-bit fields beside it. There is a further simplification.

**The full start bitmap is unnecessary if the field itself distinguishes a segment start from a RAW entry.**

Use two reserved values:

| Sixteen-bit field | Meaning |
|---|---|
| `0 … 65533` | Ordinary offset |
| `65534` (`0xFFFE`) | Segment start |
| `65535` (`0xFFFF`) | RAW entry or parking record |

The ordinary path needs one comparison:

```text
offset < 65534  → ordinary step
otherwise      → slow path
```

On the slow path, the two values distinguish a start from RAW/parking. The current row distinguishes a parked lane from an ordinary RAW row.

That removes the 32-bit start bitmap from the header. The remaining information fits in four bytes:

| Header bits | Ordinary record | RAW record |
|---|---|---|
| 0–19 | Encoded successor base | RAW-group index |
| 20–27 | The record’s `F` byte | Unused |
| 28 | Clear | RAW flag |
| 29–31 | Reserved | Reserved |

The resulting record is exactly:

\[
30\times2+4=\boxed{64\text{ bytes}}.
\]

This is a **new proposed encoding**, not a change that can silently be applied to the frozen arm. Its distinction is specific: direct halfword access, with marker information carried by two reserved field values.

### Why this does not add an ordinary-path format branch

Both ordinary offsets and the exceptional codes are read from the same field. The common path tests the field once, just as current FOR-14 tests its sentinel once.

Marking a start temporarily replaces that row’s field with `0xFFFE`, saving the old value for restoration. This works for both ordinary and RAW records. A RAW start no longer needs a separate bitmap bit to distinguish it from the record’s other RAW rows.

The extra distinction is handled only after the ordinary comparison fails.

## 4. Relabelling removes division by 30 from navigation

A literal implementation of 30 rows per record might calculate `row / 30` on every step. That would undermine the design.

Instead, label original row \(r\) by:

\[
\boxed{\phi(r)=32\left\lfloor\frac r{30}\right\rfloor+(r\bmod30)}.
\]

Thus original rows 0–29 receive labels 0–29; rows 30–59 receive labels 32–61. The two unused labels at the end of each 32-label group correspond to the four header bytes.

For an encoded row \(x=\phi(r)\):

\[
\begin{aligned}
\text{field address}&=\text{records}+2x,\\
\text{header address}&=\text{records}+2(x\mathbin{\&}\sim31)+60.
\end{aligned}
\]

**Both addresses follow directly from the encoded row.** There is no division, multiplication by 14, byte-offset split, variable shift, or field mask.

Every successor is stored in encoded coordinates, so navigation remains in those coordinates. It does not convert back after every step.

For a full block:

| Quantity | Derived value |
|---|---:|
| Real rows | 900,000 |
| Records | 30,000 |
| Largest live label, \(\phi(899999)\) | **959,997** |
| Parking label | **960,000** |
| Twenty-bit limit | **1,048,576** |

The current twenty-bit row field therefore still suffices, including parking.

### The packed-token addition survives

For each ordinary group, store:

\[
B=\min_r\phi(\Psi(r)),
\]

and the field:

\[
o_r=\phi(\Psi(r))-B.
\]

The header’s low word is:

\[
H=B+(F[r]\ll20).
\]

Then:

\[
\boxed{H+o_r=\phi(\Psi(r))+(F[r]\ll20).}
\]

This is the same useful packed-state identity as `k`, but in the relabelled coordinate system.

The essential requirement is that offsets are **differences of encoded labels**. Keeping old offsets and merely relabelling the base would be incorrect.

### The ordinary lookup becomes

Illustrative semantics—not an implemented kernel:

```text
x = q & ROW_MASK
offset = load_u16(records + 2*x)

if offset < START_CODE:
    header = load_u32(records + 2*(x & ~31) + 60)
    next_q = header + offset
    emit the old q's carried byte
    store next_q
    prefetch next_q's record
else:
    handle start, RAW, or parked lane
```

The offset occupies bytes 0–59; the header occupies bytes 60–63. Both loads remain in the same aligned cache line.

A zero-extending 16-bit memory load is one load µop in the published Skylake-family measurements. That is the primitive replacing the packed-field extraction, rather than an attempt to make the shift instruction perform unrelated addressing work. [Uops](https://uops.info/html-instr/MOVZX_R32_M16.html)

**A hand-written ordinary loop can be sketched at around 20 instructions with the current loop structure.** That is an assembly target only—not compiled evidence. The firmer result is the removal of named operations: slot multiplication, packed byte addressing, variable shift and extraction mask.

It still needs two table loads. It does not turn compact navigation into the shipped table’s single-load operation, and it does not remove more lane traffic than `k` already removed.

## 5. Correctness follows from a coordinate transformation, with explicit obligations

Relabelling changes the navigation function to:

\[
\Psi'=\phi\circ\Psi\circ\phi^{-1}.
\]

Because \(\phi\) is one-to-one on real rows, this preserves the permutation’s trajectories and cycles. It does not require every block to have one cycle.

The byte orientation remains the existing invariant:

\[
L[\Psi(r)]=F[r].
\]

At encoded row \(\phi(r)\), emit the carried \(L[r]\), then carry \(F[r]\) into the next token. The reviewed implementation already uses this orientation. 

The implementation obligations would be:

* Choose segment starts in original row coordinates, then encode them. Seed the first byte from `L[start]`; do not use an encoded label to index `L`.
* Store encoded successors in both ordinary and RAW records.
* Never create a successor or segment start pointing into the two header positions.
* Preserve the carried byte while parked, and test the START code before interpreting RAW/parking.
* Keep final assembly’s length as the original \(n\), not the encoded address span.

The inverse coordinate formula,

\[
\phi^{-1}(x)=30(x\gg5)+(x\mathbin{\&}31),
\]

is available where setup needs an original row. It is not required in the ordinary navigation loop.

This settles the mathematical transformation. It does not substitute for correctness testing of code that has not been written.

## 6. The important new result: a worst-case storage bound

The earlier 28-entry sketch depended on an unknown new RAW fraction. We can do better than assuming the old 1.9% will persist.

### The property we exploit

Within each first-column character bucket, \(\Psi(r)\) is increasing: it lists that character’s occurrences in `L` in their original order. The stable counting-scatter construction establishes this property. Applying the increasing function \(\phi\) preserves it. 

Let:

\[
G=\left\lceil n/30\right\rceil,\qquad
M=\phi(n-1),\qquad
S=65534.
\]

A group becomes RAW for one of two reasons:

**It crosses a character-bucket boundary.**  
With \(\sigma\) used characters there are only \(\sigma-1\) boundaries, so there are at most \(\sigma-1\) such groups.

**It is homogeneous, but its encoded successor span is too wide.**  
Within one character bucket, different groups’ successor ranges are ordered and disjoint. Their spans sum to at most \(M\). Each too-wide group contributes at least \(S\), so there can be at most:

\[
\left\lfloor M/S\right\rfloor
\]

too-wide groups per bucket.

Therefore the total RAW-group count satisfies:

\[
\boxed{
R\leq
\min\!\left(
G,\;
\sigma\left\lfloor\frac{M}{65534}\right\rfloor+\sigma-1
\right).
}
\]

This is a conservative bound, not an estimate of the average case.

### Apply it to a full block

For \(n=900000\):

\[
G=30000,\quad M=959997,\quad
\left\lfloor M/65534\right\rfloor=14.
\]

Hence:

\[
R\leq15\sigma-1.
\]

Keep the current convenient RAW organization: **128 bytes per exceptional group**, holding 32 padded four-byte entries, of which 30 are used. Including one parking record:

\[
\text{logical table bytes}=64(G+1)+128R.
\]

This gives:

| Full-block assumption | Maximum RAW groups | Maximum logical table size | Smaller than 3.6 MB |
|---|---:|---:|---:|
| At most **98** used byte values | 1,469 | **2,108,096 bytes** | **41.4%** |
| Any byte alphabet: at most **256** values | 3,839 | **2,411,456 bytes** | **33.0%** |

The current census reports 80–98 used byte values in its full blocks. That supplies the premise for the first row; the bound itself is my derivation. 

For context, with no RAW groups this proposed table would occupy about **1.920 MB**, versus the existing FOR-14 census’s approximately **1.868 MB including its exceptions**. The wider fields need not destroy most of the compactness benefit. Their actual corpus size remains unmeasured. 

### What this bound does—and does not—buy us

It removes a substantial uncertainty:

> **We can retain a meaningful table-size advantage without assuming that the new grouping inherits the old exception rate.**

But it does not establish cheap exceptions. The general bound still permits appreciable RAW use, and periodic inputs can make dynamic lookup frequencies differ from storage frequencies.

It also bounds **logical representation bytes**, not total decoder RSS, allocator rounding, vector spare capacity, construction scratch or output buffers.

Finally, the bound assumes that every eligible group uses the ordinary format. Artificially forcing groups to RAW for correctness testing intentionally bypasses it. Marking segment starts does not add RAW groups.

## 7. Direct construction is possible without first making a full T-vector

This is another question that can be narrowed theoretically.

The current prototype builds its records from a full-table oracle outside timing. That proves the representation’s correctness, but does not provide a production constructor or a measurement of construction cost. 

A direct constructor can instead exploit the same monotonicity.

A sequential scan of `L` supplies each character’s occurrence positions in increasing order. The existing counts give that character’s destination interval in `F`. Consequently, the constructor can maintain a small pending group for each character.

For the proposed layout:

1. Identify the groups crossing character-bucket boundaries from the counts; assign those RAW destinations.
2. Scan `L`, appending each encoded occurrence position to its character’s pending group.
3. On completion of a homogeneous group, its first and last successors determine the span. Emit either the ordinary record or a RAW record.
4. Finalize partial groups and validate the bucket endpoints.

Thirty `u32` values for each possible byte value require at most:

\[
30\times4\times256
=
\boxed{30{,}720\text{ bytes}}
\]

of value staging, plus counters and metadata.

Thus **a full \(n\)-entry T-vector is not mathematically required as an intermediate**. Nor is a comparison sort required: the occurrence streams are already ordered.

That settles feasibility, not economics.

The constructor still pays for staging stores and reads, coordinate maintenance, classification, final stores, boundary handling and validation. Fewer final table bytes do not imply less total work or less total memory traffic. The shipped builder is already optimized; the proposed constructor must beat an actual budget, not an unoptimized scatter model.

I would therefore keep direct construction behind the navigation decision. The useful conclusion now is that there is a bounded-memory construction route—not that we should start implementing it.

## 8. The economic requirement remains demanding

A compact table that merely reaches traversal parity may still lose after construction.

The project’s retained planning model is:

\[
\text{CPU saving}
\approx
0.592(1-b)+1.847(1-w),
\]

where \(b\) and \(w\) are candidate/control **cost ratios** for construction and traversal-plus-assembly. The coefficients are approximate phase costs from the earlier reconstruction, not universal constants or a fresh measurement. 

For a saving of 0.25 CPU-s per decoded GB:

| Construction cost | Required traversal-time reduction | Equivalent traversal throughput |
|---|---:|---:|
| Unchanged | **13.5%** | **1.157×** control |
| 10% higher | **16.7%** | **1.201×** control |
| 20% higher | **19.9%** | **1.249×** control |

These are conditional calculations under unchanged remaining costs.

This is why I would not promote either of these statements into a decision:

> “The new table is guaranteed smaller.”

> “The new loop should be about 20 instructions.”

Both are relevant. Neither demonstrates that the constructor and traversal together improve the decoder.

Likewise, the earlier one-worker thresholds were conditional on the phase model, unchanged construction and an assumption about concurrent cost. They should remain attached to those assumptions, rather than becoming universal rejection rules for every possible representation.

## 9. What this changes in the compact-navigation assessment

There are now two distinct pieces of work, with different levels of maturity:

| Item | What is established | What remains |
|---|---|---|
| **Existing FOR-14 `k`/`j`** | Compiled work reduction and recorded correctness checks | Their performance, including the multiplication trade-off |
| **Proposed 30-entry halfword layout** | Direct addressing, packed-token algebra, marker scheme and conservative storage bound | Independent design review, implementation correctness, code generation, corpus behaviour and performance |
| **Direct construction** | A bounded-staging route exists without a full T-vector intermediate | Actual construction cost and integration |
| **Whole-decoder benefit** | An explicit conditional cost budget | A production result satisfying it |

**My preferred theoretical direction is now the 30-entry layout, rather than more instruction-selection work on the 14-bit extractor.**

That is not because `k` or `j` have failed—they have not been timed. It is because the new layout removes the operation sequence that those candidates must still execute, while retaining a provable storage advantage.

I would keep this as **one concrete neighbouring design**, not the start of a FOR-width sweep. Its review questions are specific: verify the two-sentinel scheme, the encoded-coordinate invariants, and the RAW-count bound; then determine whether a future compiled step actually realizes the predicted simplification. Any implementation or timing would need its own identity and authorization. The frozen `h` result remains unchanged.

### Verification performed here

The accompanying checks covered all 900,000 live labels of a maximum-size block, address and inverse-coordinate identities, 100,000 packed-token additions, and the RAW-count argument on 980 synthetic monotone-permutation cases with several thresholds. They also checked the numerical storage bound across every block size from 65,536 through 900,000 rows.

Those checks supplement the proofs; they are not a corpus census, decoder validation or benchmark.

The layout-proof.md[layout and proof note](sandbox:/mnt/data/compact_navigation_review/layout-proof.md), compact-math-checks.json[arithmetic results](sandbox:/mnt/data/compact_navigation_review/compact-math-checks.json), and check_compact_math.py[reproduction script](sandbox:/mnt/data/compact_navigation_review/check_compact_math.py) preserve the details.

**The substantive advance is this: we can exchange a modest amount of compression for a much simpler ordinary lookup, without surrendering the table-size advantage or adding a navigation directory. That strengthens compact navigation as a research candidate—but its performance case still has to be earned.**
`````
