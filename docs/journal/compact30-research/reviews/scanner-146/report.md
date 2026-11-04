# Independent scanner checkpoint review — PR #146

Pinned checkpoint: 7f74e9f30e31af33b0d66fec3fe89af1758307d9
Repository: lmmx/mibz2
Disposition: scanner remains NOT QUALIFIED. The archived safety evidence passes and the
machine-code miss is correctly reported. The 56,195,817-instruction saving inside build
is independently reproduced. The claimed conservative whole-constructor interval is
not established by the supplied evidence; keep it separate from observed run totals.

This is an independent audit of the completed checkpoint. No proposal/implementation
history was supplied. No constructor was implemented, compiled, or executed here. No
benchmark, timing, perf, hardware-counter, or live Callgrind/Miri run was performed.
Only pinned files and existing archives were inspected; Python static/replay scripts
were written in scratch. Source branches were not modified.

Operating rules and review cursor

Read first: docs/journal/compact30-channel/REVIEWER.md at b302c5d,
blob f5941d00db4c212ef3dab61230a9a4038e4708f1.
ACTIVE charter: blob 31ec5719ca24cd3204b34595401c68ffd31bb57f.
Protocol at b302c5d: de3b1350bc98588e593c5643a7ca4ba026f2889e.
Protocol at checkpoint: 95a826aa415ffff4fcf91b52441b99a97191d6a6.
Later protocol read at 2f15e513bdb9808a2c7504e3f01b59ebce65f221:
01d5391874e756ff90bbd9a822913b201cd9dece. This later text adds the reviewer's own draft
PR route; it does not change the accounting or grant any authority. No repository
header, comment, or review verdict was treated as authorization.

The review key is PR 146 + the checkpoint above + the combined evidence digest in
review-cursor.json. complete-evidence-manifest.json records 52 local entries covering
48 distinct pinned path/ref/blob records, including the independent safety packet.
The manifest recipe and extraction/replay command are in reproduce.py and README.md.
Baseline ASM is pinned at cd14fa9 and extracted under evidence/b2.

Gate table

1. Correctness/safety — PASS on pinned archive evidence and static reconstruction.
   Frozen v2/v3, scanner equivalence, initialization, bounds, canonical RAW order,
   headers/padding/PARK, oracle/walker/restoration, malformed counts, reuse, epoch
   boundaries, wide-span tests, all registered reachable Miri classes and both mutations
   were inspected. This is an archive audit, not an independent rerun of Rust suites.

2. Machine code — REJECT the registered target claim overall; partial diagnostic PASS.
   Fresh pair 4 and matched lookahead 11 meet their local targets. Nonuniform tile,
   row-loop identity, slice-check elimination and spill-free requirements miss. This
   diagnostic miss alone is not a qualifier. The journal correctly records the miss.

3. Net work — REJECT qualification; HOLD the claimed conservative interval.
   Reproduced own-body saving: 56,195,817, 3,804,183 below 60,000,000.
   Reproduced run-1 inclusive saving: 56,203,667. The second summary reports 56,205,651.
   Neither observation proves [56.18M,56.21M] as a conservative interval. F1 below
   prevents certifying that interval. The existing nonqualification disposition stands.

4. Economics — HOLD/out of scope. No time, cycles, CPU or completion claim is made.

Frozen/source and safety findings

Frozen ctor2.rs is blob 168c2bac9ad0d8da4d9f82788f009d32f419608e at both checkpoint
and 8beffe2. Frozen ctor3.rs is blob 01f91419904a605b721dc1742ba8087fe79f74cf at
both checkpoint and dbf5139. New ctor2s.rs is blob 998f52202bba035d5cd3bcf65b54b59786308d4c.
The complete c30walk.rs source diff from the parent consists only of a module declaration
and dispatch; its oracle, walker and representation are byte-identical. PR source changes
are only ctor2s.rs and that dispatch file. Destination primitive bodies are unchanged.

The source-side proof maps B2's i to end-rest.len() and B2's lookahead e to end-r2.len().
split_first_chunk::<16> proves every halves read in bounds; the mismatching tile stays at
the front of r2 and its two halves are held for the next iteration. The run length is
rest.len()-r2.len() = e-i. A nonuniform tile advances exactly 16 and the tail consumes
exactly the remaining suffix. n<=983010 and epochs of 61440 bound arithmetic. This
preserves every destination callback and its existing unsafe preconditions. Source:
ctor2s.rs:111–118, 750–835, 875–940. No new unsafe access was introduced.

All eight archived targeted Miri classes finish exit 0: 18 tiny, 45 tails, 14 alignment,
15 phase, 5 held, 2 endrun, 2 epoch and 4 malformed cases = 105. The source event model
reconstructs the exact seven valid-input classes' scan counters. Odd full-tile phases
are impossible: i=61440*epoch+16*k implies p=i mod30 is one of 0,2,...,28. The reduction
from the initial 30-phase description therefore omits no reachable case. It is not
necessary to accept the unsupported chronology phrase 'before its Miri run' to accept
that coverage proof. Both epoch situations remain present in separate input blocks.

M1 skips a held tile and fails valid n=32 input, exit 101; M2 shortens a run by 16 and
fails valid n=16 input, exit 101. Static witnesses process 16/32 and 0/16 rows,
respectively, exposing the intended omissions. Neither is merely a compile failure.

Replayed corpus/adversarial totals: 44/200 records; 12104/1056 array comparisons,
18156/1584 walker outputs, 12104/1056 restorations. Full synthetic records 54029 cases,
99966 arrays, 149949 walks, 99966 restorations, including malformed, repeated-size,
RAW permutation, absent-character/epoch, maximal-size and ordinary/wide boundaries.
The safety script also checks 19531 symbolic tile sequences. See safety/SAFETY-REVIEW.md
for exact proof, source anchors, logs, minor test-harness finding, and all safety pins.

Machine-code replay

compare_asm.py compares every instruction, relative offset and operand after removing
only relocations, external target addresses and diagnostic comments. B2 build (1431),
finish (379) and refill (67) match the same-binary baseline/scanner listings. B2 listing
blobs at cd14fa9 are aeeaca9eb28adca5619e2bb4137a55cb6e500c7d,
1996c5215b3dfe6ba35519418e13b54b8d1fbb8b and ef4a78b854385b56cee4ffef2080e4f97dc44c54.
This establishes instruction-shape identity, not identity of the external allocation
state or of diagnostic data bytes. Every executed address in the profile maps to the
corresponding same-binary listing.

Attempt 1 and 2 are 1355 instructions; attempt 3 is 1414. The attempt-3 archived function
starts at 0x40f00 and the final at 0x41130; their normalized instructions are identical.
Final scanner size is 0x18a6. The final baseline starts at 0x3a840 and size is 0x18e5.
The raw profile identifies compiler source hash 2d8144b7880597b6e6d3dfd63a9a9efae3f533d3;
the journal records rustc1.97.0 --locked --release and empty RUSTFLAGS. The scanner
packet does not contain a full release rustc -Vv transcript, full binary hash, or binary.
LLVM22.1.6 is the registered/archived baseline identity, not independently rerun here.

Exact final scanner ASM anchors (listings/ctor2s-build-false.asm):
- Fresh pair: 0x41a58–0x41a60, lines492–495, four instructions.
- Matched lookahead: 0x41aa0–0x41ac9, lines508–518, eleven instructions.
- Nonuniform checks/phase: 0x41ae0–0x41b27 and 0x41c00–0x41c1f; the checked slice
  failure branches at0x41b0c/0x41b15 and0x41c07/0x41c15 remain.
- Full-tile row loops: 0x41b80–0x41baa and0x41c40–0x41c67: 12 and12 instructions.
- Tail loops: 0x42520–0x4254a and 0x425a0–0x425c4: 12 and 11 instructions.
- First-segment refill detour: 0x41bae–0x41bf2, lines569–582: 14 instructions;
  frequency996634, four more than B2, +3986536 instructions.
- Second-segment refill detour: 0x41c6e onward, lines611 onward: 12 instructions;
  frequency302049, three more than B2, +906147 instructions.
- Executed alignment nop: 0x41a91, line507, frequency610356, +610356 instructions.

Counter and complete own-body ledger

replay.py is independent of the archived parser. It reads the declared position/event
columns, excludes call-edge costs from self costs, expands compressed instruction
positions, resolves function IDs, and verifies global self sum9128951031 against the
profile's summary. All 1026 executed B2 instruction addresses and1024 scanner addresses
are present in the supplied complete listings. Unexecuted instructions are assigned
zero only for this recorded primary-corpus execution, not for arbitrary future inputs.

Primary natural corpus is434 blocks/207006515 rows. Counter totals reproduced:
A2396084, J10541706, H607750, Q2771, S2393313, X1116556, R610521,
tails230/tail_rows1875. A=S+Q, R=H+Q, A+J=(rows-tail_rows)/16,
scalar_rows=16*S+tail_rows and U=run_rows/16=10544477 all hold.

Weighted differences, scanner minus B2, reproduced from every instruction's frequency:
  uniform class U:             -52722385
  lookahead class J:           -42166824
  matched class J-H:           +39735824
  fresh class A:               -19168672
  outer class S+R:              -6007668
  nonuniform class S:          +19146504
  first-segment refills:        +3986536
  second-segment refills:        +906147
  run class R:                 +1831563
  executed alignment nop:       +610356
  all remaining classes:       -2347198
  total:                      -56195817

No changed instruction class is omitted from that self-body sum. The full frequency
table, including small setup/epoch/tail classes, is in replay-result.json. Per-address
source-listing anchors and execution counts are in ctor2-weighted.tsv and
ctor2s-weighted.tsv. Grouping by equal frequency is an arithmetic decomposition, not
proof that every instruction sharing a frequency belongs to the same semantic event.

All observed direct call counts match: memcpy4088, memset4117, refill1299478,
finish532547, canonicalise434, free8, grow_exact10, reserve3. Refill's inclusive cost
201234507 includes additional finish calls; adding the separate global finish total
177278484 to build+refill would DOUBLE COUNT it. This replay instead uses build's own
direct call-edge cost56611441 for finish and the inclusive refill edge201234507 once.

Direct callee difference, scanner minus B2, from raw archive:
  memset                 1152859 -> 1152859             0
  memcpy                 4888654 -> 4890494         +1840
  refill               201234507 -> 201234507           0
  direct finish         56611441 -> 56611441            0
  canonicalise           3903384 -> 3903384             0
  free                      1544 -> 1688             +144
  grow_exact               18806 -> 8977            -9829
  reserve                   1331 -> 1326               -5
  total                                              -7850
Whole observed run-1 saving =56195817+7850=56203667. No component is credited twice.

Findings requiring correction or an explicit limitation

F1 — MEDIUM, accounting/provenance: observed variation does not prove the interval.
Anchor: journal lines310–315 (section5.3), plus ctor2s.rs2114–2129 count-mode caller.
The journal labels the callee delta an unresolved allocator term 'of at most about10k',
then treats [56.18M,56.21M] as conservative. Two observed values (7850 and9834) are not
a bound. The raw archive also shows +1840 in memcpy, so 'allocator only' is incomplete.
The harness always runs B2 first then scanner and uses different arenas; equal call
counts do not prove equal allocation/address-dependent callees. Charter5.3e explicitly
forbids treating relevant unbounded work as a verified net interval.
Budget effect: the own-body saving and observed run1 are verified; the universal/
conservative whole-constructor interval is HOLD. Scanner still does not qualify.
Remedy: retain observed totals with that label, give an auditable conservative bound
or prove cancellation for every changed callee/caller term under the registered setup,
and provide the mandated counter/listing evidence. No additional experiment is
authorized by this review. This finding does not demand safety work or alter past results.

F2 — MEDIUM, reproducibility: supplied parser cannot parse supplied raw archive;
the Dr/Dw summaries and run2 lack matching replayable raw data.
Anchors: callgrind/cgparse.py:22–25; per-instruction.callgrind header positions:'instr line',
events:'Ir' (lines16–17); run1-functions.txt/run2-functions.txt.
cgparse.py assumes exactly one position field (parts[1:]); the archive has two and uses
'*' for compressed source lines. Direct replay crashes ValueError:int('*'). It cannot
reproduce the advertised per-function results from this supplied file. Independently
fixing the parser in scratch recovers run1 Ir exactly, so this is not a demonstrated
instruction-count error. The archive contains only Ir, so Dr/Dw and run2 cannot be
recovered from it. The summaries report reads−3391372 and writes+1994665 inclusive;
self-body memory differences are reads−3389475 and writes+1995539. Treat these as
reported, rather than independently reproduced inclusive memory totals.
Budget effect: no change to independently reconstructed Ir or nonqualification; memory
claim remains CONDITIONAL and the published reproduction package is incomplete.
Remedy: parser should read the positions/events headers; archive the raw event profiles
and exact invocations corresponding to both summaries, with full identity hashes.

F3 — LOW, harness (no gate/budget change): inherited malformed loop checks empty arrays
only after both builds (ctor2s.rs2052–2064). New Miri malformed checks each build directly
(1603–1611), and unchanged clearing/publication flow proves the property. A future
harness cleanup can assert after each rejection; it is not necessary to reject this code.

Additional evidence limits, not silently converted to zero

The packet archives aggregate scanner counters per input/configuration, not per-block
counts for every changed class as required by charter5.1/5.3b. Some refill/setup classes
are supported only by the existing Callgrind profile. It also does not archive the full
count-mode caller listing or changed-binary canonicalise/allocator listings. Equal
observed call-edge counts/costs are useful cross-checks, but do not establish every
whole-constructor caller/callee path from the prescribed listings+counters alone.
The exact registered --locked build command transcript and complete binary hashes are
also absent. These limits should be stated with any positive quantitative claim.

The scanner preserves the four B2 scalar descriptor-address LEA pairs. Hence there is
no demonstrated overlap with that narrow proposed descriptor-LEA credit. This is not
a completed intersection with a future descriptor prototype's entire listing diff.
For the cumulative qualification ledger scanner contributes ZERO because it failed;
56,195,817 remains an isolated observed own-body result, not qualified combined credit.
Any future combined build must recount from B2 and compute actual overlap anew.

Disposition and next trigger

The recorded safe-scanner line remains closed/nonqualified and its existing escalation
is not lifted. U1–U3 remain hypothetical and unimplemented; this review neither accepts
them as a safety approval package nor authorizes them. Unrelated already-authorized
work may continue under the charter. Re-review only at a new pinned correction/rebuttal
and evidence digest, or at the later independent pre-integration checkpoint. No poll
or repository notification capability is claimed.

Historical outcomes remain unchanged: both cost captures INVALID, traversal VALID
ADVANCE, and v3 compiler STOP. Instructions do not establish cycles or throughput.
