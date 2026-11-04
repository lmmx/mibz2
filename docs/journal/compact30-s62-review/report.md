# Compact30 §6.2 independent pre-integration review

Review cursor: `lmmx/mibz2`, §6.2 index at
`18ca5d4cb9fcaeaf5cee7e7f1db2ebb1fd72080e`, index blob
`27062e3971bd83981cf2725506e84e4f5430e082`. Reviewer role: `s62-reviewer`.

## Disposition

The archived safety evidence supports all three items. The two isolated instruction
intervals reproduce numerically, but their gate-3 PASS claims do not yet meet the
charter's explicit counter requirement. There is also a demonstrable overlap omitted
by the journals' “none” conclusion. Resolve S62-F01 and S62-F02 before integration.

| Item | Gate 1 / safety | Gate 3 | Review consequence |
|---|---|---|---|
| 1. Scanner, final U2 variant | **PASS** on the pinned evidence | **HOLD** under §5.3(b); numerical interval reproduced | S62-F01 |
| 2. Descriptor arrays | **PASS** on the pinned evidence | **HOLD** under §5.3(b); numerical interval reproduced | S62-F01 |
| 3. Frozen v3 finaliser | **PASS for §7(a)**; no unresolved material safety finding | Not a gate-3 review of v3 | §7(b), §7(c), and the combined build remain separate obligations |

This report grants no authority, approves no integration, and does not decide v3's
eligibility. Its historical compiler STOP remains unchanged. Gate 2 remains diagnostic;
the finaliser listing still contains 44 instructions, 12 explicit reads, and 6 writes.

## Independence and input boundary

This review began in a fresh context with the supplied brief as its only
project-specific input. The index was the first project document fetched. No PR
discussion, comments, reviews, reassessment journal, Optimizer ledger or proposals,
Guardian decisions, or pre-existing `review/*` work was read. Journals were treated as
claims, not as established results. The earlier Researcher review suggested the U1 and
U2 mutant controls used in item 1; this reviewer had no involvement in that work.

The parent supplied operational metadata about the launch and the connector's commit
identity, and requested a progress update. None supplied scientific evidence or a
prior verdict. [launch-record.md](launch-record.md) records the configured isolation
and the observed boundary; [brief.txt](brief.txt) preserves the received brief.

The governing blobs were verified: charter
`31ec5719ca24cd3204b34595401c68ffd31bb57f` and review contract
`2839405784ec4e63d2f4cc7bd406e5dff4dd1029`. The index does not pin `PROTOCOL.md`;
it was not fetched, and no protocol hash is asserted. No candidate was compiled, run,
timed or modified. No benchmark, perf capture, i9 access, protocol change, or merge
was performed.

## Pinned evidence and reproduction

| Evidence | Commit |
|---|---|
| Index and governing documents | `18ca5d4cb9fcaeaf5cee7e7f1db2ebb1fd72080e` |
| B2 source | `8beffe28bec9b539517b442350d8a3fda62164c8` |
| B2 listings | `cd14fa90e45c39d7d42b4eb71efa6e3a9b28f78f` |
| Scanner source and trial evidence | `49cb46b65070059af49e906fa8d98693b808bda2` |
| Descriptor source and evidence | `29dd071f8787a28560a7a081fd7ac0ef13991466` |
| B3 source | `dbf5139e12ae60b5f5bff40059b88d65ddccea41` |
| v3 closeout evidence | `46c322213e5780e4be2a05ebcc57348836657720` |

[manifest.json](manifest.json) lists the exact commit, path and Git blob hash of all
132 downloaded files. No unpinned transitive reference was followed. The safe scanner
predecessor was not needed to decide the final U2 variant.

From a checkout containing these commit objects, run:

```sh
python3 fetch_evidence.py --repo /path/to/mibz2
python3 replay.py --output replay-results.json
python3 audit_figures.py --replay replay-results.json --output figures.json
```

Run these commands in this report's directory. The first script reads only manifest
paths with `git show`; it does not fetch, check out or modify branches. The replay
parser is independently written, reads the Callgrind `positions:` and `events:`
headers, resolves compressed function names and addresses, separates call-edge costs
from own costs, and checks profiled constructor addresses against the listings.
It replays all eight raw profiles. The second audit derives the tables below, verifies
every individual targeted console, and evaluates small integer models of the safety
arguments. The archived implementer parsers and their generated summaries are not
inputs to these scripts. [figures.json](figures.json) is the resulting audit output.

Both same-binary B2 build listings contain 1,431 instructions and match the fixed
`cd14fa9` B2 listing after relocation addresses are normalised. Frozen source checks
match at all three component pins:

| Source | Blob |
|---|---|
| `ctor2.rs` | `168c2bac9ad0d8da4d9f82788f009d32f419608e` |
| `ctor3.rs` | `01f91419904a605b721dc1742ba8087fe79f74cf` |

## Findings

### S62-F01 — Required changed-path counters are missing

**Severity: high. Material: yes. Affects items 1 and 2, gate 3.**

Charter §5.3(b) requires: “Where no existing counter covers it, a counter is added to
the counted instantiation only, and the uncounted listing is checked unchanged.” The
frozen sources do not contain counters for several changed paths. Profile execution
frequencies exist, but that is not the required counted-instantiation evidence.

The scanner's §10.4 explicitly identifies the first- and second-segment refill detours,
vector-fill entry, executed alignment nop, whole-group branch, and smaller classes as
having no counter. The descriptor journal §7(3) admits the same deviation for fill,
open, refill detours, sweep, initialisation, and small classes. Source anchors are the
counter fields at `ctor2su.rs:167` and `ctor2d.rs:152`, the scanner's `tile16` at
`ctor2su.rs:653`, and the descriptor's `refill` at `ctor2d.rs:433`.

The raw profiles independently reproduce these uncountered signed terms:

| Item | Added instructions charged from profile-only frequencies | Removed instructions withheld at the conservative end |
|---|---:|---:|
| Scanner | 4,359,203 | 17,203 |
| Descriptor | 1,173,383 | 3,279,857 |

The conservative arithmetic is internally consistent. It does **not** satisfy the
separate mandatory evidence requirement. Numerical slack is not permission to amend
§5.3(b). This changes the claimed gate-3 status from PASS to HOLD; it is not a claim
that either measured saving falls below its minimum.

**Remedy:** add counted-only counters for every changed path lacking coverage, archive
the primary-corpus counts, establish that the uncounted whole functions and callees
are unchanged, and replay the ledger. An alternative accounting method would require
an authorised charter decision; this review supplies none.

### S62-F02 — The claimed empty overlap is false

**Severity: medium. Material: yes for §6.3's overlap and floor.**

The headline scanner checks and the descriptor's two per-row LEAs are disjoint.
They are not the entirety of either isolated ledger's credited removals. Both also
credit a nonuniform-tile class changing from 38 to 37 instructions.

There is an exact shared B2 instruction:

| B2 anchor | Instruction | Executions |
|---|---|---:|
| `ctor2-build-false.asm`, `0x343b5`, build offset `+0x12b5` | `mov 0x8(%rsp),%r13` | 2,393,313 |

This reload restores `ll.len()` in the nonuniform full tile's second-slice setup.
The same offset is `0x3ea85` in the scanner profile's B2 listing and `0x3f2c5` in the
descriptor profile's B2 listing. Each raw profile gives exactly S executions there,
matching the scanner counter and the descriptor's `outer_iterations - runs`.

The descriptor removes that reload while retaining the length in `%r10`: its
successor sequence at `ctor2d-build-false.asm +0x13e0..+0x13ef` has the range comparison
and branches with no intervening reload. The scanner removes the checked second-slice
setup altogether: `ctor2su-build-false.asm +0xa80..+0xa89` has the remaining segment
test and a reload of different scan state. That reload is not `ll.len()` and is
already charged as added scanner work.

Thus **O has a demonstrated lower bound of 2,393,313 instructions** from this
instruction alone. This is a lower bound, not an assertion that the full intersection
has been exhausted. The journals' “instruction-level intersection ... none” and “the
credited sets don't intersect” statements are false. Using this demonstrated overlap
instead of zero changes the scanner-plus-descriptor translated floor from 100,000,000
to 97,606,687 before any other demonstrated overlap. That can change the combined
gate's outcome; it must be explicit in §6.3.

**Remedy:** correct the overlap record and carry an instruction-address intersection
into the combined ledger. Recount the combined constructor from scratch. In particular,
the following shared paths cannot be added as independent savings or costs:

| B2 path | B2 → scanner → descriptor instruction counts | Reason |
|---|---|---|
| Outer scan iteration, `+0x9d2..+0xa27` | 13 → 11 → 17 | Shared control, pointer/base reloads and register allocation |
| Nonuniform full-tile class | 38 → 37 → 37 | Includes the shared length reload above |
| First-segment refill detour | 10 → 12 → 10 | Shared argument setup, call, base rematerialisation and reloads |
| Second-segment refill detour, B2 `+0x130d..+0x1337` | 9 → 11 → 8 | Shared refill call and reloads of descriptors, length and scan state |
| Run-entry class | 32 → 31 → 35 | Shared descriptor addressing and live scan state |

The scanner still contains both descriptor LEAs in each scalar copy. The descriptor
still contains the fresh/lookahead checks. Those specific headline removals remain
disjoint. Neither fact establishes zero whole-ledger overlap.

### S62-F03 — Stores increase on constructor paths

**Severity: low. Material: no. Reported under §5.3(g).**

Raw profiles reproduce the increased writes already acknowledged by the implementer:

| Own constructor code, including changed callees | Δ reads | Δ writes |
|---|---:|---:|
| Scanner | −505,885 | +3,391,432 |
| Descriptor | −1,096,093 | +5,004,156 |

The descriptor's `refill` alone adds 1,299,478 reads and 5,197,912 writes: four
additional writes for each of 1,299,478 refills. The final scalar append loops retain
their memory-operation count; the increase is in conditional refill and surrounding
paths. Descriptor inclusive write totals partly hide this increase behind the changed
initialisation copy. No timing consequence is inferred. Preserve this finding in the
combined review and do not equate an instruction saving with a store reduction.

### S62-F04 — Caller boundary needs an explicit accounting note

**Severity: low. Material: no at the observed bound.**

The uncounted constructor-inclusive totals exclude `c30walk::main`. In both pairs of
profiles, that caller's own cost rises by **868 instructions = 2 × 434 blocks**:
4,499,105,219 → 4,499,106,087 for the scanner harness and
4,499,110,446 → 4,499,111,314 for the descriptor harness. The branch selection in
`ctor2su.rs:2237` / `ctor2d.rs:2037` is harness code, so this is not evidence of an
868-instruction constructor regression. However, the index has no caller listing
with which to partition dispatch from call setup.

**Remedy:** document the caller boundary and archive its call-site listing when
claiming the complete §5.3(a) caller audit. Conservatively charging the entire observed
caller delta gives scanner [84,042,350, 84,059,553] and descriptor
[54,549,438, 62,276,927], still above their respective minima. This finite sensitivity
does not change a gate outcome and is not used to manufacture a further HOLD.

## Gate 1: source arguments and completed checks

### Scanner U2

`scatter` maintains `rest` as a suffix of `ll[i0..end]`. On the sole `tile16` call
(`ctor2su.rs:984`), `split_first_chunk::<16>` has established `rest.len() >= 16`, and
the index is `end - rest.len()`. Therefore `i + 16 <= end <= ll.len()`. Held lookahead
does not consume its mismatching tile; the next iteration obtains the same chunk from
the still-unconsumed suffix. Advancing either `after` or `r2` preserves the suffix
invariant. Tail processing uses the checked `tile`.

At `ctor2su.rs:664–666`, `n1 = min(16, 30.wrapping_sub(p))` is at most 16 even outside
the valid phase domain. Hence `[i+n1, i+16)` is ordered, bounded, and cannot overflow;
the live immutable byte slice provides provenance and initialisation. Empty U2 ranges
at the end of a full tile are permitted. The first segment indexes the 16-byte window
safely. Correct output additionally needs `p < 30`, preserved by the existing advance
rules; memory safety of U2 does not depend on asserting that phase in release mode.

All reachable full-tile phases are even: full tiles begin at multiples of 16 and an
epoch has 61,440 rows, divisible by both 16 and 30. The 15-case phase sweep covers
`0,2,...,28`. The earlier literal “every p” wording does not leave reachable odd
phases untested. Tails cover every length 1–15 and never call U2.

All nine registered Miri classes completed with exit 0: tiny 18, tails 45, align 14,
phase 15, held 5, endrun 2, epoch 2, malformed 4, controls 1: **106 cases**.
MU1 reports the reversed U2 range at mutated line 667; MU2 reports the U1 endpoint at
line 666 in variant a, where U1 actually exists; MU3 reports U2's endpoint at line 666.
All three are caught both with library UB checks and with `-Zub-checks=no`, at their
intended operations. MU2 does not claim coverage of a U1 operation in the final source,
which contains only U2. The variant-a diff verifies that distinction. Release mutants
M1 and M2 fail on `small n 32 sigma 3` and `small n 16 sigma 1` respectively.

### Descriptor split

The persistent state is a fieldwise bijection: `wr[c]`, `en[c]`, and
`cold[c].{w0,r0,lim,g,raw}`. `desc(c)` assembles a read-only temporary; it is not written
back as competing authoritative state. At initialisation all three pointers are null.
`open` sets all three for the same group; `close` computes rank before nulling all
three; `one` either obtains room through `refill` or appends within existing room;
`fill` receives `min(room,m)` from `run`; bulk completion changes rank while the
descriptor is empty. Snapshot and final checks read the same logical rank as B2.
No new raw-pointer arithmetic or unsafe access is introduced by the split. Indices
come from a byte or a bounded 256-element loop. Source anchors are `ctor2d.rs:311–326`,
`:392–445`, `:590–647`, and `:697–733`.

All nine registered Miri classes completed with exit 0: tiny 18, slots 60, allbytes 1,
crossing 120, runs 6, epoch 1, pending 4, malformed 4, reuse 7: **221 cases**.
M1's shortened end is caught by the finish invariant assertion; M2's use of `en`
instead of `wr` in rank is caught by the epoch case's RAW-length mismatch. Both exit 101.

### v3 epoch tags, §7(a)

`snapshot` assigns the tag before any row in the new epoch. `refill → one` and
`run → open → fill` are the only opening routes; each open is followed by a row in that
same epoch. The source's 16 distinct tags cannot equal a RAW index or `UNOPENED`.
Only a nonempty portion reaches finish. Thus matching the active tag identifies an
ordinary portion whose rows all belong to the current epoch; its label span is at
most 65,533. The exact 65,533 and 65,534 boundary examples agree with `phi(i)=i+2⌊i/30⌋`.

Thirty written fields imply a group-aligned `w0`, with all 60 field bytes initialised.
The header store at byte 60 remains in the same reserved record allocation and is
aligned. The source uses raw pointers derived from the record allocation, never
reallocates that allocation during scatter, and publishes length after finalisation.
The tag, low label and byte occupy disjoint header fields. The slow dispatcher tests
the high bit at `ctor3.rs:583`, so tagged partial or old-epoch ordinary portions cannot
be mistaken for RAW indices. These arguments follow `ctor3.rs:387–509` and `:544–659`,
not the success assertions alone.

The targeted Miri source selects `(41 × 3 + 4) + 4 + 2 + 1 = 134` cases; the complete
console confirms 134, with **6,162 leaf finishes and 189 slow entries** split into
8 old-epoch ordinary, 105 same-epoch partial, and 76 preset RAW. The tag-dispatch
mutant changes both high-bit tests back to equality with `u32::MAX`; the one-row
ordinary portion then fails the RAW-index assertion and exits 101. Its failure is
the intended classification fault, not an unrelated crash.

All four rebuilt release-suite consoles equal their original consoles after removing
the appended `exit 0` footer where present. The finaliser's separate rebuilt machine
listing is not archived; identical machine code on rebuild remains the implementer's
closeout claim. This review verifies the pinned source, archived leaf, mutation diff,
coverage and identical suite transcripts; it did not compile B3 or claim an independent
binary reproduction. No material safety counterexample was found.

### Release suites

The harnesses compare counted and uncounted arrays byte for byte with the oracle,
compare B2 counters, check walks, and recheck arrays after marker restoration.
The logs and JSON records give:

| Item | Corpus records | Adversarial records | Full synthetic cases | Full synthetic array / walk / restoration checks |
|---|---:|---:|---:|---|
| Scanner | 44 | 200 | 54,030 | 99,968 / 149,952 / 99,968 |
| Descriptor | 44 | 200 | 54,145 | 100,194 / 150,291 / 100,194 |
| v3 | 44 | 200 | 53,960 | 99,840 / 149,760 / 99,840 |

For each item the corpus records sum to 12,104 array checks, 18,156 walk checks and
12,104 restorations; adversarial records sum to 1,056, 1,584 and 1,056. Malformed,
reuse, RAW-order, padding and PARK checks are in the pinned harnesses; targeted
malformed cases check both builds' empty arrays separately. Miri coverage is the
registered selection, not a claim that all release cases ran under Miri.

## Gate 3: independently replayed arithmetic

One primary pass has 434 blocks and 207,006,515 rows. Scanner counters give
A=2,396,084, J=10,541,706, H=607,750, Q=2,771, S=2,393,313, X=1,116,556,
R=610,521, epochs=3,654, tails=230. These satisfy A=S+Q, R=H+Q,
J+Q=run_rows/16, and scalar_rows=16S+1,875=38,294,883.

| Signed candidate-minus-B2 term | Scanner | Descriptor |
|---|---:|---:|
| Counter-supported terms, including scalar rows and refill where counted | −88,402,421 | −56,102,571 |
| Profile-only positive classes | +4,359,203 | +1,173,383 |
| Profile-only negative classes | −17,203 | −3,279,857 |
| Own constructor sum | **−84,060,421** | **−58,209,045** |
| Changed library initialisation, observed net | 0 | −4,068,750 |
| Inclusive constructor sum | **−84,060,421** | **−62,277,795** |

Scanner `build::<false>` own work is 1,026,601,670 → 942,541,249; inclusive work is
1,294,420,653 → 1,210,360,232. Its refill, finish and canonicalise own and inclusive
costs match. All 11 library call-site aggregates match in Ir, Dr and Dw in both runs.

Descriptor build own work is 1,026,601,670 → 959,332,822. Refill grows by 9,059,803:
6×1,299,478 + 1×1,262,935. Its four scalar copies save
2×(1,769+29,357,924+8,935,084+106)=76,589,766. Finish and canonicalise are equal.
Ten shared library sites match. The removed initialisation copy costs 4,447,632;
the three added initialisation calls cost 58,156+262,570+58,156=378,882.
The conservative end charges all three added calls and gives no credit for the
removed copy, avoiding reliance on the `rep movsb` per-byte counting convention.
Equality here means **aggregate per call site**, not an unavailable per-invocation
trace or a general theorem about every allocator state.

| Replayed saving interval | Minimum | Lower-end margin |
|---|---:|---:|
| Scanner: [84,043,218, 84,060,421] | 60,000,000 | 24,043,218 |
| Descriptor: [54,550,306, 62,277,795] | 40,000,000 | 14,550,306 |

Every changed frequency class, including small positive terms and executed nops, is
retained in `figures.json`. No observed callee delta is silently discarded. These are
reproduced numerical intervals in the archived instruction-count convention. Their
formal gate status remains HOLD for S62-F01. They are neither timing predictions nor
a combined-build ledger.
