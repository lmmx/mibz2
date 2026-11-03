# Refined compact30: one experimental walker, compiled and checked (no timing)

- **Authorised by:**
  - external report 166, the guardian approving Astra's Option 2 (report 165);
  - report 168, an explicit authorisation beyond report 158's census scope;
  - report 167, three audits drawn from the BLAKE3 study.
- **Scope:**
  - **one design:** current-row F emission, plain encoded-row lane state, `(B << 8) | F`
    headers;
  - **oracle-built records** and production's 32-lane room walk with L1 prefetch;
  - **no** timing (not even indicative), benchmark, direct constructor, second variant or
    production integration.
- **Unchanged:** the census (PR #132), the frozen FOR-14 result and its `k`/`j` arms, and
  the walker-ladder decisions.
- **Branch and code:** `research/compact30-walker`, from the census branch;
  `examples/c30walk.rs`.
- **Records:** `2026-11-02-compact30-walker/`.

> **Correction (report 172, accepted in report 173): the ordinary step is 21 instructions,
> not 19.** The listing in §5 always held 21: 8 at `15f5b`–`15f78` and 13 at
> `15f30`–`15f59`. The first wording miscounted the second part as 11. The four loads, two
> stores and one prefetch were right. This corrects a count, not the binary, and it is not
> a performance result. Against the shipped step's 16, the gap is 5 instructions, not 3.
> The 21 is still within Astra's 19–21 expectation, and nothing is being changed to reach
> 19.
>
> The two instructions beyond Astra's sketch (report 172) are:
> - `mov ecx,esi`, which keeps the header for the byte store while the successor is formed
>   and prefetched first;
> - the redundant `mov ecx,ecx`.
>
> **The strengthened validation of the audited walk** (reports 172–173) is in §8.

## 1. What was built

**The representation** (`Rec`, `RawG`):
- **Records:** `#[repr(C, align(64))]`, one per 30 rows in F order: 30 `u16` fields at
  bytes 0–59 and a `u32` header at byte 60.
- **Compile-time assertions:** size 64, alignment 64, `f` at 0, `h` at 60, `RawG` 128
  bytes.
- **Runtime assertions:** the record and RAW arrays are 64-byte aligned after every build.
- **Labels:** φ(r) = 32⌊r/30⌋ + r mod 30. Row x's field is at byte 2x and its record's
  header at 2(x & !31) + 60.
- **An ordinary record:** H = (B << 8) | F, with offsets ≤ 65,533.
- **A RAW record:** H = RAW_BIT (bit 28) | group index, every field 0xFFFF. The RAW group
  holds 32 `u32` entries, 30 used, each (successor << 8) | F[row].
- **0xFFFE** marks a segment start during a walk.
- **Parking:** one record of 0xFFFF fields after the last, at label PARK = 32G. Its header
  is RAW_BIT | 0xFFFFF, a dummy index never dereferenced (§3c).

**Emission (report 165):**
- F[r] = L[Ψ(r)], so F[p], F[Ψ(p)], … from p = orig_ptr equals production's
  L[Ψ(p)], L[Ψ²(p)], ….
- A lane at row x emits F[x] and moves to Ψ(x); it carries no byte.
- The first segment starts at φ(orig_ptr).

**The walk** (`C30::room`) is production's room walk as the FOR-14 `k` arm copies it:
- 511 random starts plus the first, and with `extra` the adversarial ones (§2);
- room passes over 32 lanes, replacement at a start, termination midway through a round,
  the tail and assembly.
- **A lane** is `{row, pend, seg, p, end}`, 32 bytes, checked at compile time.

**The step:**
- **Ordinary:** load the field and header (both addressed from x). If the field is below
  0xFFFE, write H's low byte, set row = (H >> 8) + field, and prefetch the successor's
  record line.
- **Slow,** tested in this order:
  1. **START:** the segment ends without emitting the marked row.
  2. **PARK:** resume. Write the pending entry's low byte and move to `entry >> 8`.
  3. **RAW:** save the entry index, prefetch the entry, park, and emit nothing this turn.
- **A start's own step** comes from `take`, using the start's saved (successor << 8) | F.
- **Markers** stay set until the tail has ended, and are then restored from the saved
  fields.
- **Short blocks** (under 2^16 rows) use the plain walk over the same records.

**What the traversal reads:** only the candidate's records, its RAW groups, and declared
segment and lane metadata:
- starts;
- saved fields;
- the starts' (successor << 8) | F, computed from the records before marking;
- the segment buffers;
- the lane array and output-base array.

Ψ, F and the last column exist only inside the oracle builder and are dropped when it
returns. The walk never receives `ll`.

**Two instances:**
- `room::<false>` is the audited walk, disassembled below.
- `room::<true>` is the same code with transition counters.

Every check runs both against production (`bwt::Segmented`).

## 2. Correctness

**The checks:** every block of every input under four configurations, permille of
otherwise-ordinary groups forced RAW, and adversarial starts:

| config | forced RAW (‰) | adversarial starts |
|---|--:|---|
| natural | 0 | no |
| natural + extra | 0 | yes |
| forced 5% | 50 | yes |
| forced all | 1000 | yes |

- **Both walks** must reproduce production's inverse-BWT output byte for byte.
- **In the counting walk,** the records after the walk must equal the records before it,
  so every marker is restored.
- **At natural storage** (0‰), the RAW-count bound R ≤ min(G, σ⌊M/65534⌋ + σ − 1) must hold
  for every block. The forced runs test the exceptional paths separately from the bound.

**The adversarial starts:**
- fixed rows (0, 1, n−1, n/2, n/2+1, n/3, n/3+1, and 40 consecutive from n/5);
- eight chain-consecutive triples, giving one-byte segments;
- the first rows of up to 16 RAW groups, giving RAW starts;
- up to 16 rows whose successor is RAW.

**The inputs:**
- **The established 11 corpora,** hash-checked against `corpora.sha256` (11 of 11 OK).
- **25 adversarial inputs at levels 1 and 9:**
  - random data of 1, 2, 3, 29, 30, 31, 59, 60, 61 and 1,000 bytes (short and
    partial-record blocks);
  - 65,535, 65,536, 65,537, 99,999, 300,000 and 899,990 bytes (the segmented-walk boundary,
    full blocks);
  - zeros;
  - periodic data: `ab`, `abc`, period 7, period 30, period 31;
  - every byte value;
  - runs;
  - two-letter text.

**The results:**

| run | build | outcome |
|---|---|---|
| 11 corpora × 4 configs (`corpora.*`) | release | **44 of 44 exact**, both walks |
| 25 inputs × 2 levels × 4 configs (`adversarial.*`) | release | **200 of 200 exact**, both walks |
| the same adversarial set | debug (assertions on) | **200 of 200 exact** |
| 8 corpora, including the primary `wikidata-20261026` (all 434 blocks), × 4 configs | debug (assertions on) | **32 of 32 exact** |

- **Every transition was exercised** (§4): ordinary and RAW starts, RAW arrival,
  waiting and resumption, one-byte segments, buffer growth in the full loop and the tail,
  mid-pass replacement, tail entry and termination midway through a round.
- **Periodic and multi-cycle inputs:** segments cover only the cycles that hold starts, and
  assembly repeats them. In the adversarial set that shows as fewer emitted bytes than
  rows: 3.15 M fewer over the periodic inputs, each still exact.
- **Debug and release agree** on outputs and transition counts. The exceptions are the
  grow counts and `reserved_bytes`, which record capacity a worker's buffers retain from
  earlier blocks, so they depend on which blocks each thread handled.
- **The records match the committed code.** After clippy's fix (a type alias in the
  harness), the walker's instructions were unchanged at the same addresses, and the
  release suites and the debug adversarial suite were run again on the final build. The
  debug corpus run predates that fix; the code it ran differs only by the alias.

## 3. The physical-access audit (report 167)

**a. The layout.**
- Size, alignment and the header's offset are asserted at compile time.
- The arrays' alignment is asserted at runtime after each build.
- Array stride is the element size, 64, and no padding exists to differ from it.

**b. Readable extents** (debug assertions in the walk, on for every debug run above):
- Before each ordinary or tail step's loads, the row's field is in bounds:
  2x + 2 ≤ 64 × records.
- The row's header is in bounds: 2(x & !31) + 64 ≤ 64 × records.
- The row is not a header position: x & 31 < 30.
- Every RAW index computed, and every pending index read on resumption, is below
  32 × RAW groups.

**c. The eager header load is valid on every record kind.**
- The header is loaded before the branch (§5). Its address is the current record's last
  4 bytes, so it is inside the arena for ordinary, RAW, partial and parking records alike.
- **On a RAW row,** the header is the RAW header, and the slow path uses it for the index.
- **On PARK,** the header is the parking record's dummy. The slow path tests PARK before
  any RAW index is formed, and a debug assertion confirms a RAW index is never formed
  from PARK.
- **The resumed entry's index** is the one saved at the RAW row (`pend`), never the parking
  header's.
- **The partial last record's unused fields** are 0xFFFF. No successor label points at
  them: the builder asserts every stored successor is a live row below n and not a header
  position.

**d. Operand widths** (from the disassembly, §5):
- the field is a 16-bit zero-extending load (`movzx … WORD PTR`);
- the header is a 32-bit load;
- the RAW entry is a 32-bit load;
- the output is a byte store.

No wider load touches the records.

**What this does not establish:** the assertions cover every access this code makes, but
they are not a guard-page test of the release binary. Guard pages, and Miri on a small
case, remain options if wanted.

## 4. Transition counts (untimed)

The full per-input table is `corpora-summary.md`. On the primary corpus,
`wikidata-20261026` (434 blocks, 230 segmented, 207,006,515 rows):

| config | RAW arrivals (per emitted byte) | resumptions | waiting turns: full / tail | starts hit | replacements | grows: full / tail | one-byte segments | RAW starts |
|---|--:|--:|---|--:|--:|---|--:|--:|
| natural | 1,476,765 (0.00713) | 1,476,765 | 1,387,335 / 89,430 | 117,558 | 110,198 | 1,797 / 420 | 64 | 915 |
| natural + extra starts | 1,469,072 (0.00710) | 1,469,072 | 1,396,940 / 72,132 | 144,067 | 136,707 | 2,086 / 420 | 3,796 | 8,608 |
| 5% forced RAW | 11,590,343 (0.0560) | 11,590,343 | 10,992,257 / 598,086 | 143,968 | 136,608 | 2,052 / 401 | 3,766 | 14,472 |
| all forced RAW | 206,854,383 (0.9993) | 206,854,383 | 196,007,532 / 10,846,851 | 143,808 | 136,448 | 2,493 / 464 | 3,778 | 143,808 |

- **Every run balances:** arrivals = resumptions = waiting turns (full + tail).
- **Grows vary between runs.** A segment buffer grows when it outruns the capacity its
  worker's buffer already had from earlier blocks, so the grow counts depend on which
  blocks each of the three threads took. Two full corpus runs agree on every other count,
  and on every output byte.
- **On this corpus,** the walker's measured RAW arrivals at natural storage are 0.713% of
  emitted bytes, close to the census's stored 0.714% for full blocks. That is a count for
  this walker on this corpus, with its starts. It is not a general dynamic exception rate,
  as report 159 cautions.
- **Each RAW arrival costs that lane one waiting turn** before its byte. The lane's output
  base moves back one byte in the full loop, the existing room-loop compensation.

## 5. The machine code (release, baseline x86-64)

**Provenance** (`provenance.txt`):
- rustc 1.97.0 (`2d8144b78`), LLVM 22.1.6, target `x86_64-unknown-linux-gnu`;
- default CPU and features only (fxsr, sse, sse2);
- no `RUSTFLAGS`, no `.cargo/config`;
- the repository's `[profile.release]` (`codegen-units = 1`, `lto = "fat"`,
  `debug = "line-tables-only"`);
- `cargo build --release --example c30walk`, binary SHA-256 `713c079e…`;
- `room::<false>` at 0x14ed0; the full listing is `room-false.asm`.
- **No BMI2 or other extension instruction** appears in it (0 matches for shrx, shlx,
  sarx, bzhi, pdep, pext, rorx, andn and mulx).

**The ordinary step in the full room loop** (observed, `room-false.asm` lines 690–713).
Its registers:
- r10: the lane;
- r14: the records;
- rax: the pass offset;
- rdx/r9: the lane index;
- `[rsp+rdx*8+0x48]`: the output-base array;
- ebp: PARK.

```asm
15f5b  lea    r9,[rdx+0x1]                 ; next lane index
15f5f  mov    r11d,DWORD PTR [r10]         ; LOAD 1: lane row x
15f62  movzx  edi,WORD PTR [r14+r11*2]     ; LOAD 2: field     (address from x)
15f67  mov    ecx,r11d
15f6a  and    ecx,0xffffffe0               ; x & !31: the record, for its header
15f6d  mov    esi,DWORD PTR [r14+rcx*2+0x3c] ; LOAD 3: header (address from x; eager)
15f72  cmp    edi,0xfffe
15f78  jb     15f30                        ; ordinary → 15f30 (taken)
       ...
15f30  mov    ecx,esi
15f32  shr    ecx,0x8                      ; B = H >> 8
15f35  add    ecx,edi                      ; successor = B + field
15f37  mov    DWORD PTR [r10],ecx          ; STORE 1: lane row
15f3a  mov    ecx,ecx                      ; zero-extension (see below)
15f3c  prefetcht0 BYTE PTR [r14+rcx*2]     ; PREFETCH: successor's record line, no mask
15f41  mov    rcx,QWORD PTR [rsp+rdx*8+0x48] ; LOAD 4: this lane's output base
15f46  mov    BYTE PTR [rcx+rax*1],sil     ; STORE 2: H's low byte, F[x]
15f4a  lea    rcx,[r9-0x1]
15f4e  add    r10,0x20                     ; next lane
15f52  mov    rdx,r9
15f55  cmp    rcx,0x1f
15f59  jae    15f00                        ; end of the 32 lanes → the next offset
```

**21 instructions** (8 at `15f5b`–`15f78`, 13 at `15f30`–`15f59`), **four data loads,
two stores and one prefetch.** The first wording said 19, a miscount corrected after
report 172. Astra's expectation was 19–21, an expectation to check, not a threshold. Its
19-instruction sketch wrote the byte before shifting the header. The compiled code
instead copies the header (`mov ecx,esi`), forms and prefetches the successor first, and
carries the redundant `mov ecx,ecx`.

**Absent:** packed-field extraction, an F-directory lookup, carried-byte state and token
masks. The only mask is `and ecx,-32`, which forms the header's address. It is part of the
design and is independent of the field load.

| ordinary step | instructions | data loads | stores | source |
|---|--:|--:|--:|---|
| `o`, the shipped entry (reference) | 16 | 3 | 2 | FOR-14 `disassembly-kj.md` |
| `k`, packed token | 31 | 4 | 2 | same |
| `j`, packed token with `imul` | 27 | 4 | 2 | same |
| **refined compact30** | **21** | **4** | **2** | this listing (first reported as 19) |

**The dependency audit** (report 167). This is static analysis of the listing, not a
measurement.

| value | available after | depends on |
|---|---|---|
| lane row x | LOAD 1 | the lane pointer (r10), which advances by 32 bytes and is independent of any table load |
| field address | x | x alone |
| header address | x, plus one `and` | x alone, **not the field** |
| field | LOAD 2 | x |
| header | LOAD 3 | x |
| successor | `shr` then `add` | header (via `shr`) and field (via `add`) |
| prefetch address | successor, plus `mov ecx,ecx` | the successor |
| output byte | header's low byte | header, and the output base (LOAD 4, independent) |

- **The two table loads are independently addressed from x.** Neither waits on the other's
  result.
- **The header-limited path to the next prefetch** is: and → header load → shr → add →
  mov → prefetch.
- **The field path** is: field load → add.
- **Against the packed-token compact30:**
  - the header path gains the `shr`, as Astra said;
  - the prefetch needs no row mask;
  - the field address needs no token mask.
- **One redundant instruction is on the prefetch path:** `mov ecx,ecx` re-zero-extends a
  value that `add ecx,edi` already zero-extended. It appears because the resumption path
  (which forms the successor with its own `shr`) merges at 15f37. On Skylake-family cores
  a move of a register to itself is not eliminated, so it is probably one cycle of latency
  before the prefetch. This is recorded, not rescued: removing it is a code-shape change
  for any later, separately authorised step.
- **The byte store** depends on the header's arrival and the output base, not on the
  successor.

**The state audit** (report 167):
- **Every memory access other than the table loads** belongs to the design's declared
  state:
  - the lane row (LOAD 1 and STORE 1);
  - the output-base array (LOAD 4), which is the room walk's `base[k]` and present in
    `o`, `k` and `j` alike.
- **No spills or reloads** appear in the ordinary step. The loop keeps the lane pointer,
  records base, pass offset and lane index in registers.
- **No carried-byte load or store** exists.

**The slow paths** (observed):
- **RAW arrival** (16060–1608c), 11 instructions:
  - the RAW index is formed as (H & 0xFFFFF) × 32 | (x & 31), using `shl`, `and`, `and`
    and `or`;
  - `pend` is stored;
  - the RAW entry is prefetched;
  - row = PARK is stored;
  - `base[k]` is decremented in memory (`dec QWORD PTR [rsp+rdx*8+0x48]`, the
    compensation);
  - then the loop control.
- **Resumption** (15f7c–15f93): compare with PARK, load `pend`, load the RAW entry, then
  `shr`. It joins the ordinary path's store, prefetch and byte write at 15f37, writing the
  entry's low byte.
- **START** (15fa0 onwards): the segment's length is set, then a binary search for the next
  segment and a replacement. This is production's own logic.

**The tail loop** (16a4a–16ac1), observed:
- Its step has the same field and header loads and the same successor formation.
- The tail's lanes live in a `Vec`, as in production's and `k`'s copies, so the lane's
  `p` and `end` are loaded and `p` stored on every step. `p` is also reloaded once after
  the byte store: the compiler can't prove the byte store doesn't alias it.
- This is inherited room-walk structure and applies only to the tail's steps. It is not
  new state traffic from the representation.

**The load-resource figure, as an assumption** (report 168). The ordinary step issues four
loads and one prefetch. On a two-load-port core, that gives a simplified lower bound of
about 2.5 cycles a row from load issue alone, against about 2 for the shipped step's three
loads and one prefetch. This is a documented assumption, not a bottleneck diagnosis.
Nothing here measures how much load traffic the walk can absorb.

## 6. The storage and liveness account

**Candidate storage, per block:**

| | bytes |
|---|---|
| records | 64(G + 1), with G = ⌈n/30⌉ |
| RAW groups | 128R |
| **total** | 64(G + 1) + 128R logical bytes |

- The arrays are reserved exactly per block.
- A worker's capacity is a high-water mark across blocks, as production's buffers are.
- **On the primary corpus** at natural storage: 2.1642 logical bytes a row, identical to
  the census. **The census's sizes carry over to this encoding exactly,** because group
  classification is unchanged by the byte's placement.
- **No per-row metadata beyond the records:** fields, headers and RAW entries only.

**Walk metadata, per walk** (declared, not per row):
- the starts, about 512 to 700 `u32` with extras;
- their saved fields (`u16`) and (successor << 8) | F (`u32`);
- the segment buffers and their links, as in production;
- 32 lanes of 32 bytes;
- the 32-entry output-base array.

**Oracle-only, during construction:**
- Ψ as labels (4n bytes), F (n) and a class byte per group (G);
- 5n + G bytes in all, dropped before the walk.
- These stand in for a direct constructor.

**Validation-only:** `room::<true>` keeps a copy of the records to prove every marker is
restored.

**For a future direct constructor** (report 165's design, preserved here as a note only):
- Peak live storage is the last column (n bytes) plus the records and the RAW area
  preallocated to the bound: at most 2,411,456 bytes for a full block of any alphabet.
- About 3.3 MB in all, before shared scratch and allocator effects. That is a bound, not an
  RSS measurement.
- **No full T-vector is needed:**
  - scan L once with per-byte destination cursors in encoded coordinates;
  - write offsets directly;
  - promote a group to RAW on its first offset over 65,533, rewriting at most its 29
    earlier entries.
- Not implemented.

## 7. What this establishes, and what it doesn't

**It establishes:**
- **The refined design compiles to the intended ordinary step.** 21 instructions, with four
  loads, two stores and one prefetch. The field and header are addressed independently
  from the row. There is no extraction and no carried-byte state, and no spills.
- **The new byte ownership is correct** through production's segmented machinery: starts,
  RAW starts, parking and resumption, one-byte segments, growth, replacement, tail and
  mid-round termination. Exact on the 11 corpora and the adversarial set.
- **Every access stays inside the candidate's own storage,** checked in debug builds over
  the adversarial set and 8 corpora including the primary.

**It does not establish:**
- **Speed,** on any machine. Nothing was timed.
- **Cache or memory behaviour.**
- **Construction cost,** since the records are built by an oracle.
- **The production-shaped integration.**
- **Whether the redundant `mov ecx,ecx` or the `shr` matter.**

**The stopping boundary** (report 168) was not reached:
- correctness is resolved;
- the compiled path adds no unexpected state traffic, dependent metadata access or
  extraction.

The next decision, whether this candidate earns a separately frozen performance
experiment, is the guardian's.

## 8. The audited walk validated on its own (reports 172–173)

**The gap the researcher found** (report 172), confirmed:
- In `check`, every block's counted walk ran before the audited `room::<false>`, on the
  same scratch. So the audited walk always started with buffers the counted walk had
  already grown.
- The complete record-restoration check ran only after the counted walk.
- Neither shows a failure. Both left the audited implementation's own cold-growth and
  restoration paths untested.

**The repair** is a new harness mode, `audit` and `audit-adversarial` (`audit()` in
`examples/c30walk.rs`). The walk, the format and the loop are unchanged, and so is `check`.
Its original results in this directory are kept as they were.

**For every block of every input, under the same four configurations:**
1. **A fresh `C30`** (fresh records and fresh scratch) builds the block and snapshots its
   records.
2. **`walk::<false>` runs cold.** Its output is compared with production's, and its records
   with the snapshot.
3. **`walk::<false>` runs again** on the same records, without rebuilding, and both checks
   repeat.
4. **The counted walk** (`walk::<true>`, the transition audit) runs on a separate,
   per-thread instance. It never shares scratch with the audited one.

**Growth in the audited walk is read from its own scratch afterwards,** with no counter
in the walk:
- A fresh segment buffer is reserved exactly `cap` = 2n/ns + 64 bytes (an empty `Vec`'s
  reserve is exact), so a buffer above `cap` after the cold walk was grown by that walk.
- In the repeated walk, a buffer above its own capacity before the walk was grown again.
- The counted walk's growth totals are not used.

**The results** (`2026-11-02-compact30-walker/validation-2/`, `summary.md`):

| run | build | configs | blocks (segmented) | outputs = restorations checked | segments the audited cold walk grew (blocks) | grown again when repeated |
|---|---|--:|--:|--:|---|--:|
| 11 corpora | release | 44 | 6,052 (3,584) | 12,104 | 275,763 (all 3,584) | 0 |
| adversarial, levels 1 and 9 | release | 200 | 528 (372) | 1,056 | 17,639 (288) | 0 |
| 8 corpora, including the primary | debug | 32 | 2,344 (1,516) | 4,688 | 111,243 (all 1,516) | 0 |
| adversarial, levels 1 and 9 | debug | 200 | 528 (372) | 1,056 | 17,639 (288) | 0 |

**What these show:**
- **Every check was exact,** and every record array was restored after every audited walk,
  cold and repeated.
- **Debug builds** ran the physical-access assertions of §3 throughout.
- **On the corpora,** every segmented block's audited cold walk grew segment buffers
  itself, so its growth path was exercised.
- **The repeated walks grew none.** They reuse the same starts, so the capacity the cold
  walk left suffices. That is the reuse case.
- **In the adversarial set,** 84 segmented blocks grew nothing. These are periodic inputs
  (`ab`, `abc` and others), whose segments stay under `cap`. Their outputs and
  restorations are still checked.

**The final identity** (`validation-2/provenance-final.txt`):
- rustc 1.97.0 (`2d8144b78`), LLVM 22.1.6;
- baseline x86-64 (fxsr, sse, sse2), no `RUSTFLAGS`, the repository's release profile;
- binary SHA-256 `db7feaac…`.

**The harness change left the audited walk unchanged.** Against the reviewed binary:
- the same 1,565-line listing;
- every instruction identical in order;
- only RIP-relative displacements differ (66 lines, `asm.diff`), because the function
  moved from 0x14ed0 to 0x15aa0.

The ordinary path is now at `16b2b`–`16b48` and `16b00`–`16b29`: the same 8 + 13 = 21
instructions, four loads, two stores and one prefetch. The exceptional paths are the same
instructions at the shifted addresses.

**The next step:** the draft traversal-comparison protocol is in
`2026-11-02-compact30-traversal-protocol.md`, for review. Nothing has been timed.
