# External research report 172: the researcher's independent review of the compact30 prototype: conditional go; the ordinary path is 21 instructions, not 19; one validation gap in the harness (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with two attached files. Both are kept
  byte for byte in `2026-11-02-external-report-172/`:
  - `ordinary-path-recount.txt`;
  - `review-calculations.json`.
- **Source:** the independent researcher.
- **In reply to:** the guardian's handoff in report 171: PR #134 at `85b87fb`, its journal,
  code and machine code.

**Evidence cutoff:** 2 November 2026. A source and machine-code review of the pinned
artifacts. The researcher did not rebuild, rerun the suites, or benchmark.

**The coordinator confirms both findings against the pinned listing and source:**
- **The ordinary step is 21 instructions.** It is 8 (15f5b–15f78) plus 13 (15f30–15f59).
  The journal's "19" was a miscount of its own listing, and the PR body and report 171
  repeated it.
- **The harness gap is real.** Each block runs `walk::<true>` and then `walk::<false>` on
  the same scratch. The marker-restoration assertion is under `if COUNT`.

The corrections await the guardian's response, since the code is frozen at `85b87fb`
(report 171).

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
## Recommendation: conditional go, with two corrections before the performance freeze

**Compact30 has earned preparation of one separately registered traversal comparison. I found no semantic defect or architectural execution obstacle in the reviewed paths. But the ordinary path contains 21 instructions—not 19—and the correctness harness has one specific coverage gap to close before freezing the experiment.**

The gap is that the audited, uncounted walker always runs **after** the counted walker on the same scratch space. That gives it already-grown segment buffers. The recorded growth counts therefore do not demonstrate cold-start growth coverage for the actual uncounted implementation. Marker restoration is also explicitly checked only for the counted version.

Neither finding calls for another layout or optimization round. **Correct the reporting and strengthen that one validation sequence; leave the traversal algorithm unchanged.**

I reviewed the source and recorded release disassembly at **`85b87fb4f21bf52fe9f327dcb2a371ddeddc983d`**, the harness and primary transition records, and Astra’s report 165 and the relevant BLAKE3 guidance. I did not rebuild the executable, rerun the correctness suites, or run a benchmark.

## 1. The instruction count needs correcting: 21, not 19

Following the ordinary control-flow path in `room-false.asm` gives:

| Executed portion | Address range | Instructions |
|---|---|---:|
| Lane-index update, row load, field/header loads, classification | `15f5b`–`15f78` | **8** |
| Successor formation, state update, prefetch, output and loop control | `15f30`–`15f59` | **13** |
| **Total ordinary full-loop visit** | | **21** |

The branch at `15f78` jumps back to `15f30`; both portions execute. This is the same convention used for the archived 16-instruction control: include the ordinary lane step and its lane-loop bookkeeping, but not exceptional paths or per-round/per-pass setup.  

The other reported counts are correct:

| Property | Independently checked result |
|---|---:|
| Machine instructions | **21** |
| Data loads | **4** |
| Stores | **2** |
| Successor prefetches | **1** |

The journal’s own displayed listing contains those 21 instructions, despite labelling it 19. Instruction fusion or move elimination does not change how many machine instructions are in that executed sequence. 

### Why the result differs from Astra’s 19-instruction sketch

Astra’s proposed sequence writes the output byte before shifting the header register. The compiled sequence instead preserves the header in `esi`, copies it to `ecx`, and forms and prefetches the successor before performing output bookkeeping. It also contains the redundant self-move:

```asm
15f30  mov ecx,esi     ; retain esi's low byte for output
...
15f3a  mov ecx,ecx     ; redundant zero-extension
```

Those are the two additional instructions relative to the 19-instruction sketch. The first serves the chosen scheduling and register-use arrangement; the second is redundant.  

**This remains within Astra’s stated 19–21 expectation.** It changes the headline gap against the archived control from three instructions to **five**, not the substantive conclusion that packed extraction and carried-byte state have disappeared.

I would correct the journal and comparison chart. I would **not** make reaching 19 instructions a new acceptance condition or alter the candidate before its first performance comparison.

## 2. The revised byte-emission semantics survive the transitions

The central invariant is:

> **For an active, non-parked lane, `row` identifies the next F byte not yet emitted. For a parked lane, `pend` identifies the RAW entry that supplies that same pending byte and its successor.**

That invariant is consistent with the builder, setup, ordinary path, slow paths and tail.

### Initialization and segment ownership

The oracle builder constructs encoded successors by stable counting scatter and stores the corresponding current-row F byte. `starts()` explicitly inserts `φ(orig_ptr)`. Before marking any starts, `room()` saves every start’s original field and its packed `(successor << 8) | F` value. `take()` emits the saved low byte and sets the lane to the saved successor.  

This is the required convention:

\[
F[\Psi^t(p)]=L[\Psi^{t+1}(p)].
\]

There is no accidental initialization at `φ(Ψ(orig_ptr))`, no remaining L-based segment seeding, and no carried-byte field that needs preserving.

Saving all start values **before marking any of them** is important. A start whose successor is another start is still decoded from the unmodified records during setup.

### Transition audit

| Transition | What the source does | Assessment |
|---|---|---|
| Ordinary segment start | Emits the saved F byte and advances once | Correct |
| RAW segment start | Uses the saved RAW-derived byte and successor directly | Correct; it intentionally needs no parking turn |
| Ordinary step | Emits `H as u8`; advances to `(H >> 8) + offset` | Correct |
| Encounter START | Finishes without emitting the marked row | Correct; that byte belongs to the next segment |
| Encounter unmarked RAW | Saves its entry index, prefetches it, parks and emits nothing | Correct |
| Resume PARK | Reads the saved entry, emits its low byte, advances to `entry >> 8` | Correct |
| One-byte segment | `take()` emits its sole byte; the next START encounter finishes it | Correct |
| Tail termination | Finishes the lane and processes the lane swapped into its position without skipping it | Correct |

These are source-level conclusions from the actual transition code, not deductions from the scalar census reference. The compiled RAW resumption also visibly loads `pend` and the RAW entry, then joins the common row-store, prefetch and byte-store sequence.   

### START and PARK precedence are correct

The slow path checks:

1. START.
2. PARK.
3. Otherwise, RAW.

Thus a marked RAW row terminates its predecessor’s segment rather than entering RAW processing. A parked lane resumes from `pend` before the dummy parking header could be interpreted as a RAW-group index. The release listing preserves that order. 

The eager header load on PARK is therefore harmless **because its address is valid**, not merely because its value is subsequently unused.

### Periodic and multiple-cycle inputs

Each selected segment starts on a cycle containing at least that start, so it eventually reaches a marked row. Assembly starts from the segment containing `φ(orig_ptr)` and follows the recorded links until it has produced exactly \(n\) bytes, repeating that cycle when necessary.

This does not require traversal to emit every table row exactly once. The journal correctly reports periodic cases where the sum of traversed segment bytes is smaller than the eventual assembled output. The prototype’s full segmented comparison is stronger evidence here than the earlier scalar-reference checks, although it is still finite testing rather than exhaustive verification.  

## 3. The room-pass pointer arithmetic is consistent

This is the most delicate source-level part of the review.

At pass offset \(o\), define a lane’s next output position as:

\[
P_k=\mathrm{base}[k]+o.
\]

The code maintains that position through all three kinds of action.

**Ordinary or resumed byte.** It writes at \(P_k\). Advancing the global offset advances that lane’s next output position by one.

**RAW arrival with no byte.** It decrements `base[k]`. Advancing the global offset then leaves the effective output position unchanged:

\[
(\mathrm{base}[k]-1)+(o+1)=P_k.
\]

**Mid-pass replacement.** `take()` has already written the new segment’s first byte and returned its next free position \(p\). Setting

\[
\mathrm{base}[k]=p-(o+1)
\]

makes the next turn write exactly at \(p\). It neither repeats nor skips the first byte. 

### Termination midway through a round

When the final queued segment has been assigned and a lane encounters START, the loop reconstructs:

- Earlier lanes’ pointers using `off + 1`.
- Later lanes’ pointers using `off`.
- The finishing lane’s pointer using `off`, because it emitted nothing.

The earlier-lane adjustment remains correct even when an earlier lane parked during that round: its base has already been decremented. It also remains correct when an earlier lane was replaced: its base was already rebased to the new segment. 

### Capacity and growth

At each pass’s start, `room` is bounded by every active lane’s remaining capacity and by `cap − 1`. A replacement receives at least `cap` capacity and consumes one byte in `take()`, so it has enough room for the remainder of that pass. Waiting turns only reduce the number of writes.

Growth happens before capturing the new pass bases, or in the tail where the lane pointers are updated immediately. The outer segment-buffer vector is not resized during traversal. These facts support the raw-pointer lifetime and bounds argument.  

The intermediate bases may temporarily precede their allocations. Using `wrapping_sub` and `wrapping_add` for this is permitted: the eventual dereference must return to the originating allocation and be in bounds. Rust’s documentation explicitly distinguishes this from immediately in-bounds `add`/`sub` arithmetic. [Rust Documentation](https://doc.rust-lang.org/core/primitive.pointer.html?utm_source=chatgpt.com)

**I found no off-by-one defect in the waiting, replacement, mid-round-exit or tail logic.**

### Record and RAW access bounds

The layout supplies a compact argument independent of the test outcomes:

- A live encoded row is \(x=32g+s\), with \(0\le s<30\).
- Its two-byte field occupies record bytes \(2s\) through \(2s+1\), ending no later than byte 59.
- Its four-byte header occupies bytes 60–63.
- PARK has its own fully initialized record.
- A RAW entry occupies four bytes within its 128-byte group; the thirty live entries end at byte 119.

The builder checks successor labels, and the source uses the matching 16-bit and 32-bit loads. The release ordinary and resumption listings confirm those widths. No wide vector read crosses a logical record boundary.   

Guard-page or Miri checks would provide additional evidence, but I have not found an unexplained memory-access obligation that makes them a prerequisite to evaluating this prototype.

## 4. One validation repair is needed: exercise the audited version independently

The harness’s actual order for each block is:

```text
build records
run walk::<true>       // counted
compare output
run walk::<false>      // audited, uncounted
compare output
```

Both calls use the same `C30`, including its segment buffers. 

### Why that leaves a gap

The counted traversal has already expanded buffers to accommodate the same segments before the audited traversal begins. Consequently, the reported full-loop and tail growth counts belong to `room::<true>`; they do not establish that `room::<false>` exercised its corresponding allocation and pointer-update paths.

The source also places the before/after record comparison under `if COUNT`. After the uncounted call, the next block rebuilds the records. Therefore the current harness does not directly establish either:

- Full marker restoration by the uncounted version.
- Repeated uncounted traversal of the same records without rebuilding them.

The restoration loop is present in the uncounted assembly at `16c70`–`16cd2`, and I found no defect in it. **This is a coverage omission, not a demonstrated restoration failure.**  

### The narrow repair

For the existing correctness inputs, validate the audited version **first on fresh scratch**, including cases requiring growth. Check its output and the complete record array afterward. Then run it again **without rebuilding**, checking output and restoration again. Keep the counted version’s transition audit separate.

That addresses cold growth, restoration and repeated-use coverage without changing the format, ordinary loop or exceptional transitions.

I would complete this before freezing a repeated traversal benchmark. There is no reason to use it as an opening for another implementation variant.

### What the existing tests do establish

The harness genuinely compares both implementations byte-for-byte with production’s inverse-BWT output. Its generator includes short and partial-record inputs, segmented-boundary sizes, periodic inputs, all byte values, forced RAW groups and adversarial starts. The journal records 244 release configurations and additional debug coverage.  

That is substantial evidence. It should be described as **inverse-BWT output equivalence across those cases**, not as exhaustive verification, production integration testing, or an independently rerun correctness result from this review.

## 5. The compiled dependency structure delivers the intended simplification

The ordinary dataflow is:

```text
lane pointer
    │
    └─ load current encoded row x
            │
            ├─ field address: records + 2*x
            │       └─ load offset ────────────────────┐
            │                                          │
            └─ header address: records + 2*(x & ~31)+60 │
                    └─ load H ── copy/shift ────────────┤
                             │                          └─ add → successor
                             │                                    ├─ store lane row
                             │                                    └─ self-move → prefetch
                             └─ low byte ────────────────┐
                                                        └─ output store
lane index ── load output base ──────────────────────────┘
```

This follows the registers and addresses in the recorded listing. 

### The two table loads are genuinely independently addressed

The header address depends on the current row and its group mask, **not on the field’s value**. The field address depends directly on that same row. The header load can therefore proceed while the field load is outstanding, subject to the processor’s scheduling and resources.

The fact that `movzx` appears first in the listing does not serialize the two memory operations.

They remain **two load operations**, even when their cache-line fetch is shared. Nothing in this review establishes their hit rate, latency or degree of overlap on the i9.

### Output bookkeeping is off the successor-prefetch dependency

Neither the output-base load nor the byte store supplies an input to successor formation. The successor comes from the shifted header and offset.

The state store also does not feed the prefetch through a memory reload: the prefetch uses the successor already in a register. Its appearance after the state store does not imply that it waits for that store to complete. 

Conversely, the output byte now depends on the header arriving. That is the intentional trade-off Astra identified, not a hidden carried-byte dependency.

### Other lanes can expose independent work

The lane pointer advances by a fixed 32-byte stride, and the lane index advances independently of the table results. The next lane’s row does not depend on the current lane’s successor.

Reusing architectural register names does not itself create a true dependency between those independent lane computations. Branch prediction and execution resources still affect how far the processor can progress; the listing establishes available independence, not achieved memory-level parallelism.

### No unexpected ordinary-path state traffic

The four data loads are precisely:

- Lane row.
- Offset.
- Header.
- Output base.

The two stores update the lane row and output byte. The stack-resident output-base array is declared room-pass state, not evidence of an accidental scalar spill.

There are allocations, stack operations and more state traffic in setup, growth and the tail. The tail also reloads its output pointer after the byte store. Those costs are real, but they are outside the 21-instruction ordinary full-loop path and are documented in the source and listing. 

### The self-move is a real blemish, not an architectural obstacle

At `15f3a`, `mov ecx,ecx` is redundant: ordinary successor formation already wrote `ecx`, and the resumption path also arrives after a 32-bit shift.

Published Coffee Lake measurements distinguish same-register moves from eliminable register-to-register moves: the same-register case executes work and has a one-cycle dependency in that measurement. Applying this to Comet Lake is a family-based inference, not a measurement of this candidate. [Uops](https://uops.info/html-instr/MOV_89_R32_R32.html)

The instruction lies on the prefetch path, so it should not be dismissed as necessarily free. But its presence is not a reason to postpone evaluation for another optimization round. The current candidate is coherent enough to test as it stands.

## 6. The transition records are consistent—but exception frequency is not exception cost

I recalculated the primary natural-storage record:

| Check | Result |
|---|---:|
| RAW arrivals | 1,476,765 |
| Resumptions | 1,476,765 |
| Full-loop waits + tail waits | 1,387,335 + 89,430 = **1,476,765** |
| Segments finished | **117,558**, matching segments created |
| Initial segments + replacements | \(32\times230+110,198=\mathbf{117,558}\) |
| RAW arrivals per emitted byte | **0.7133906%** |
| Logical storage per row | **2.1641973 bytes** |

The storage formula also reconciles exactly with the record’s group and block counts. Against four logical bytes per row, this is a **45.895% reduction in navigation storage**. 

Two qualifications matter.

**A RAW arrival produces two slow-path visits:** the arrival/parking visit and the resumption visit. It adds one non-emitting turn before the byte is produced. Thus “0.713% RAW arrivals” is not “0.713% overhead” or even the complete fraction of slow-path dispatches. START handling adds further exceptional visits.

**This pin uses 128-byte padded RAW groups.** Its storage accounting is:

\[
64(G+1)+128R.
\]

It does not implement the later dense-RAW suggestion, and none of that suggestion’s storage claims should be attributed to this prototype. The `reserved_bytes` aggregate also must not be read as peak RSS: the harness sums retained capacities across block visits.  

The remaining performance risks are therefore concrete: an additional table load, header-to-successor arithmetic, RAW parking/resumption, branch behaviour, setup and tail costs. Their presence does not establish that the candidate loses; their small-looking static counts do not establish that it wins.

## 7. The BLAKE3 lesson is already reflected in this path

The relevant lesson from report 163 is to expose independently known addresses and keep unrelated work off the next-request path—not to import streaming prefetch distances into a successor chase. 

Compact30 already satisfies the useful parts:

- Current field and header addresses are independently derived.
- The header is loaded eagerly, with a valid address even on exceptional records.
- The successor prefetch does not depend on the output-base load or byte store.
- Other lanes have independently available state addresses.

Beyond the already documented self-move, **I did not identify a concrete missed dependency simplification that needs fixing before measurement**.

Batching more loads across lanes would require retaining their rows, headers, offsets and transition outcomes. It could change register pressure and exceptional-path handling. The present listing does not demonstrate a serialization defect that justifies that extra machinery.

## Final disposition

**The implementation meets the architectural intent of the compile prototype. It merits one separately registered traversal comparison, conditional on the narrow validation repair and corrected reporting.**

Before the performance freeze, I would require:

1. **Correct the ordinary-path count to 21**, including the chart and comparisons.
2. **Validate `room::<false>` on fresh scratch and on repeated walks without rebuilding**, with output and marker-restoration checks after each call.
3. **Record the final source, compiler and binary identity**, and verify that the audited ordinary path remains the one being measured.

The future comparison must include the candidate’s setup, marking/restoration, exceptional paths, tail and assembly at a clearly matched boundary—not just its ordinary loop. Oracle construction must remain excluded and explicitly labelled; any traversal gain must then earn a separate construction budget.

**There is no basis here for another architectural redesign, and no basis yet for a speed claim. The useful correction is “a validated 21-instruction candidate with one test-harness gap,” rather than “a proven-fast 19-instruction walker.”**

The ordinary-path-recount.txt[numbered instruction recount](sandbox:/mnt/data/compact30_review_final/ordinary-path-recount.txt) and review-calculations.json[primary-record arithmetic checks](sandbox:/mnt/data/compact30_review_final/review-calculations.json) preserve the independently checked details.
`````
