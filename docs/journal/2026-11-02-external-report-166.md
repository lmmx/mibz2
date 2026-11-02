# External research report 166: the guardian approves Astra's Option 2: one isolated compile-and-correctness prototype of refined compact30 (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** report 165, Astra's compact30 review.

**Evidence cutoff:** 2 November 2026. The guardian checked production's walker, the
compact30 reference and the constructor source. It ran nothing.

**This authorises:**
- one experimental compact30 walker, with current-row F emission, `(B << 8) | F` headers
  and plain encoded-row lane state;
- oracle-built records;
- correctness, transition counts, a storage account and annotated disassembly.

**It does not authorise:** timing (including indicative VM timing), production integration,
the direct constructor, a second variant, or any i9 run.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
**This is the architectural contribution we were asking Astra for: it has improved the design, not merely approved it. I agree with Option 2—one isolated, compile-and-correctness prototype of its refined compact30 walker.**

The improvement is concrete: **stop carrying a byte between navigation steps by changing which row supplies each output byte.** That simplifies the lane state and addressing. Astra also identifies what becomes worse, rather than assuming that every simplification produces a speedup.

I checked the relevant production walker, compact30 reference and constructor source. The byte-orientation argument is consistent with them.

## 1. The important invention: remove the need for carried-byte state

The existing production walk begins at the successor of the primary row, then emits last-column bytes. The reference’s counting scatter establishes that the first-column byte at a row equals the last-column byte at that row’s successor:

\[
F[r]=L[\Psi(r)].
\]

The production code explicitly selects `r0 = Ψ(orig_ptr)` before starting its segments; the reference constructs Ψ and F with the corresponding stable scatter.  

Consequently, these output sequences are identical:

\[
\begin{aligned}
\text{Existing convention:}&\quad L[\Psi(p)],L[\Psi^2(p)],\ldots\\
\text{Astra’s convention:}&\quad F[p],F[\Psi(p)],\ldots
\end{aligned}
\]

**That identity lets the new walker emit the current record’s byte rather than preserve a byte from the preceding lookup.**

The progression is worth recognizing:

| Design | Treatment of the output byte |
|---|---|
| Original FOR-14 | Separate carried-byte state |
| `k`/`j` and initial compact30 | Byte packed into the row token |
| Astra’s refined compact30 | No carried byte; emit F from the current record |

This is not another compiler hint. It changes the state transition so that some work is unnecessary.

It does **not** remove another load/store pair beyond what `k`/`j` already achieved. Its additional opportunity is simpler addresses and fewer operations separating the row from the byte.

## 2. The trade-off is real—and sufficiently specific to test

Astra’s new ordinary header is:

\[
H=(B\ll8)\;|\;F.
\]

The byte is directly available in the low byte, and the successor becomes:

\[
\text{next}=(H\gg8)+\text{offset}.
\]

Compared with the original packed-token proposal, this removes token masks but adds a shift between loading the header and forming the successor. It also makes output wait for the header load, whereas the carried-byte version already had that byte available.

**So the design is not proven to have the shortest dependency chain. It has a credible reduction in execution work, with an explicit dependency trade-off.**

That is enough to justify compiling it. It is not enough to predict its speed.

The proposed **19–21 instructions, four data loads and two stores** should remain a code-generation hypothesis. The meaningful questions are whether the compiler actually delivers the simpler accesses, whether the header and offset addresses remain independent, and whether the surrounding segmented loop introduces hidden work.

I would not reject a sensible 22-instruction result merely because the sketch said 19. Conversely, a 19-instruction listing would not excuse extra spills or a newly serialized lookup.

## 3. The storage advantage survives the refinement

The revised byte placement does not change which groups fit the ordinary format: the classification still depends on character-bucket membership and encoded successor span. Record and RAW-entry sizes also remain unchanged. That is visible in the existing census’s classification logic. 

Thus the census’s **logical storage counts** can transfer to the revised encoding, subject to confirming that the prototype adds no hidden metadata. The timing implications cannot.

Astra’s two implementation warnings are particularly valuable.

**Alignment must be enforced.** The scalar reference stores its records in `Vec<u16>`; that does not establish a 64-byte-aligned arena. The optimized prototype should use an explicitly specified layout and check record size, alignment and header offset. Rust provides `repr(C)` and alignment modifiers for these guarantees.  [Rust Documentation](https://doc.rust-lang.org/reference/type-layout.html)

**Scalar marker restoration is not segmented-walker validation.** The reference substitutes saved fields and continues along one chain. Production’s room walker has segment termination, buffer growth, mid-pass replacement and tail transitions. Those need testing under the new emission convention.  

Those are implementation obligations, not objections requiring another architectural review.

## 4. The constructor idea is useful—but should stay behind the walker

The proposed “write offsets directly, promote a group to RAW only when necessary” constructor is a meaningful alternative to per-character staging.

It gives us a plausible future route that avoids building a full conventional table first. But its classification, header accesses and prefix rewriting must compete with the shipped constructor, which already has a sixteen-equal-byte fast path. The current constructor also performs the `ll[s]` byte read Astra identifies. 

**Preserve that constructor design, but do not implement it now.**

First establish that the refined navigation operation remains cheap inside the real segmented machinery. An expensive walker would make a constructor project premature; a promising walker would give us a concrete construction budget.

## 5. Measuring `k`/`j` first is not a prerequisite

I agree with Astra on the ordering.

A timing result for `k` or `j` would tell us something useful about their packed-field extraction and state-management trade-off. But neither success nor failure determines whether a directly addressed halfword lookup is affordable.

The next step being proposed is only **compilation, correctness and disassembly**. We do not need another i9 experiment before taking it.

Keep `k` and `j` intact as existing candidates and comparisons. Do not build two optimized compact30 variants simultaneously. The original packed-token compact30 remains a documented alternative, not an automatic additional arm.

## What to send Claude

> **Proceed with Astra’s Option 2: one isolated compile-and-correctness prototype of refined compact30.**
>
> Archive Astra’s memorandum verbatim, then implement the design as a separately identified experimental candidate—not a change to production defaults or any frozen experiment.
>
> **Use the refined emission convention.** Start the mandatory first segment at `φ(orig_ptr)`, emit the current row’s F byte, and retain only the encoded row in lane state. Ordinary headers hold `(base << 8) | F`; RAW entries hold `(encoded_successor << 8) | F[row]`. Preserve the 30 halfword fields, coordinate transformation, two sentinels and RAW parking.
>
> **Keep the implementation bounded.** Retain the existing 32-lane room-pass organization and L1 prefetch policy. Build records from an oracle for this stage. Do not implement the direct constructor, a second optimized emission convention, a format sweep or a scheduling rewrite.
>
> **Enforce the physical layout.** Use explicitly aligned records, with assertions for 64-byte size and alignment, the header at byte 60, and valid field addresses. Document candidate storage separately from oracle-only data and validation metadata.
>
> **Validate the changed byte ownership thoroughly.** Compare exact reconstruction against production across the established eleven corpora and adversarial short, periodic, multi-cycle, partial-record and marker cases. Exercise ordinary starts, RAW starts, parking/resumption, one-byte segments, buffer growth, mid-pass replacement and tail entry. Preserve markers until segmentation is complete. Run forced-RAW tests separately from natural-storage-bound checks.
>
> Record RAW arrivals, resumptions, waiting turns and emitted bytes in untimed validation. Do not treat the stored RAW-row fraction as the measured dynamic exception rate.
>
> **Audit the actual release code.** Record the source pin, `rustc -Vv`, LLVM version, target features and flags. Use the existing baseline x86-64 settings; this design does not need BMI2. Provide annotated ordinary and slow paths, including load/store counts, spills, address dependencies and successor-prefetch timing.
>
> The intended ordinary signature is four data loads, two stores, one prefetch, independently addressable field/header loads, and no packed extraction, F-directory lookup, carried-byte state traffic or token-address masks. Treat 19–21 instructions as a hypothesis, not an arbitrary pass gate.
>
> If correctness is unresolved or the realistic loop reintroduces substantial state traffic, dependent metadata access or `k`/`j`-scale extraction work, stop and report the obstacle rather than beginning another optimization campaign.
>
> Preserve the direct-promotion constructor as a design note and capacity/liveness account only. Also correct the census total to **707,949,378 rows**; that agrees with the sum of the recorded per-corpus counts.
>
> **Return the code, correctness evidence, transition counts, storage account and disassembly for review. No performance timing, including indicative VM timing, and no production integration at this stage.**

## My overall judgment

**Astra has earned the next bounded implementation step.** It found a semantic simplification, developed it into an executable proposal, identified the trade-off, and supplied concrete correctness transitions.

This is also a good example of why the earlier failures were useful: the new design directly addresses packed extraction and carried-state costs, rather than merely making the table smaller again.

The BLAKE3 study can remain useful background, but it should not delay this prototype or expand it into a general assembly rewrite.

**Nothing needs running on your workstation now. Claude’s next result should be compiled code and correctness evidence—not another speed estimate.**
`````
