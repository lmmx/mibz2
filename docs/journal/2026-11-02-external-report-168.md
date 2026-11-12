# External research report 168: the guardian's explicit authorisation of the one refined compact30 prototype, beyond report 158's census scope (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, while the prototype was being built.
- **Source:** the guardian.
- **In reply to:** the coordinator's note that report 158 authorised only the census, and
  its load-port remark on report 165.

**Evidence cutoff:** 2 November 2026.

**What it adds to report 166:**
- **The traversal uses only the candidate representation** and its declared segment and lane
  metadata. Full Ψ, the T-vector and original-column data may support construction and
  independent checking only.
- **The 2.5-cycles-a-row load-resource estimate** is a simplified lower-bound model, not a
  bottleneck diagnosis.

**It does not authorise:** timing, a benchmark run or production integration. Work stops for
review after compilation and correctness validation.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
**Yes—give Claude explicit authorization to build the one refined compact30 prototype now.** Its pause is appropriate: report 158 authorized the census, not an optimized walker. We should preserve that historical boundary and issue a new authorization rather than reinterpret the old one.

This is **not another architectural-review round**. Astra has made its recommendation, we have accepted the direction, and the next useful evidence is compiled code and correctness.

One qualification to Claude’s analysis: **the 2.5-cycles-per-row load-resource estimate is a simplified lower-bound model, not a bottleneck diagnosis.** Comparing it with the shipped walker’s 6.9 cycles does not establish how much additional load traffic can be accommodated. That should remain a documented assumption—not become another reason to predict success or reject the prototype.

## Send Claude this

> **I authorize the bounded compact30 prototype recommended in Astra’s report 165.**
>
> This is a new authorization beyond report 158’s census-only scope. Keep the census, frozen FOR-14 results and walker-ladder decisions unchanged.
>
> **Implement one experimental design:** current-row F emission, plain encoded-row lane state, thirty directly addressed `u16` offsets per explicitly aligned 64-byte record, and ordinary headers containing `(encoded_base << 8) | F`. Retain the coordinate transformation, two sentinels and RAW parking scheme. The mandatory first segment starts at `φ(orig_ptr)`, following Astra’s emission convention.
>
> **Keep the implementation isolated.** Use oracle-built records and the existing 32-lane room-pass organization with the existing L1-prefetch policy. Do not change production defaults, dispatch, admission, assembly semantics or the closed experiments. Do not implement a direct constructor, a second optimized compact30 variant, a lane-count sweep or a general assembly rewrite.
>
> The traversal must use only the candidate representation and its declared segment/lane metadata. Full Ψ, T-vector and original-column data may support record construction and independent checking, but must not silently supply information needed by the candidate’s traversal.
>
> **Establish physical layout and access safety.** Assert the record’s 64-byte size and alignment, the header’s byte-60 offset, and valid field addresses. Check actual readable extents for ordinary, RAW, partial and parking records. An eager header load must be valid even when the field selects the slow path. The parking header’s dummy RAW index must never be dereferenced.
>
> **Validate the new byte-ownership convention throughout the segmented walker.** Compare exact reconstruction with production across the established eleven corpora and adversarial short, periodic, multi-cycle and partial-record cases. Explicitly exercise ordinary starts, RAW starts, parking/resumption, one-byte segments, buffer growth, mid-pass replacement and tail entry. Preserve markers until all segments finish and restore them afterward. Test forced RAW separately from the natural-storage-bound checks.
>
> Record untimed transition counts for RAW arrivals, resumptions, waiting turns, segment transitions and emitted bytes. Do not infer dynamic exception frequency from the census’s stored RAW-row percentage.
>
> **Return actual release disassembly and compiler provenance.** Record the source commit, `rustc -Vv`, LLVM version, target features and build flags. Use the established baseline x86-64 release configuration; do not introduce BMI2 or other target changes.
>
> Audit the ordinary path for four data loads, two stores, one successor prefetch, independently addressable field/header loads, and the absence of packed extraction, an F-directory lookup, carried-byte state traffic and token-address masks. Show the header-to-successor dependency and distinguish required lane-array accesses from additional spills. Treat Astra’s 19–21 instructions as an expectation to check, **not an arbitrary acceptance threshold**.
>
> Use the BLAKE3 material only to sharpen this dependency, state-storage and physical-access audit. Do not begin a separate BLAKE3 investigation or transplant its prefetch distances.
>
> **Stopping boundary:** if correctness remains unresolved or the realistic implementation introduces substantial unexpected state traffic, dependent metadata access or extraction overhead, report the obstacle rather than starting an open-ended rescue campaign.
>
> Return the implementation, correctness evidence, transition counts, storage/liveness account and annotated machine code. Preserve the direct-construction proposal as a design note only.
>
> **No performance timing—including indicative VM timing—no benchmark run, and no production integration are authorized. Stop for review after compilation and correctness validation.**

The BLAKE3 study fits exactly where Claude proposes: understanding which addresses are independently available and ensuring aggressive accesses remain valid. Its guide explicitly distinguishes address availability from computation dependencies and emphasizes that correct output alone does not prove safe memory access. Pasted text Pasted text

**Nothing for you to run locally.** Claude’s next deliverable should tell us whether Astra’s simpler state transition survives contact with the compiler and the real segment machinery. That is the remaining prerequisite before deciding whether this candidate earns a performance experiment.
`````
