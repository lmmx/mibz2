# Frozen compact30 v3 closeout: archival verification
Repository: lmmx/mibz2
Date: 2026-11-04

Disposition

PASS for the archived v3-specific correctness/safety closeout and the conservative
historical instruction interval [60,049,330, 60,345,544] versus B2. The formal
compiler-gate STOP remains: fast leaf 44 instructions including ret, 45 with call,
12 explicit reads, against the registered 40/41 and 10. This is an achieved static
result; it is not a timing result or automatic qualification for integration.

CONDITIONAL for charter section 7 inclusion. A completed, independent section 6.2
review must accept the safety evidence without unresolved material findings; the
future combined listing must independently yield at least 50 M finalizer savings
at its conservative lower end, and its leaf must not exceed 44 instructions (or
the qualified prototype 3 count). No combined listing or completed-sequence
closeout index was supplied for this review. This report is not that section 6.2
review and does not authorize integration, optional SSE2 work, or a new prototype.

Scope and provenance

Rules inspected: requested REVIEWER at b302c5db47f4104093ed5f408dcbe5a59720ac7c,
plus active charter and protocol at 2f15e513bdb9808a2c7504e3f01b59ebce65f221.
The initial local materialization was the current REVIEWER; it was compared with
the requested b302c5d version before the constructor source inspection. It
adds report-256 branch ownership, knowledge-base and canonical-ledger paragraphs;
these do not change the verification method or gate results. No repository
source was changed. No constructor was run or compiled; no timing/perf work,
posts, commits, or branch writes were done. Scratch scripts only read evidence.

Pinned identities:
- B2 source: 8beffe28bec9b539517b442350d8a3fda62164c8,
  examples/c30walk/ctor2.rs blob 168c2bac9ad0d8da4d9f82788f009d32f419608e.
- B2 listings: cd14fa90e45c39d7d42b4eb71efa6e3a9b28f78f,
  docs/journal/2026-11-03-compact30-constructor-v2-prototype/.
- B3 source: dbf5139e12ae60b5f5bff40059b88d65ddccea41,
  examples/c30walk/ctor3.rs blob 01f91419904a605b721dc1742ba8087fe79f74cf.
- Closeout: 46c322213e5780e4be2a05ebcc57348836657720,
  journal blob 38c9e7a739eec93f6d96e1916be3be49af7c5d7f.
- Requested REVIEWER blob: f5941d00db4c212ef3dab61230a9a4038e4708f1.
- Active charter blob: 31ec5719ca24cd3204b34595401c68ffd31bb57f.
- Active protocol blob: 01d5391874e756ff90bbd9a822913b201cd9dece.

Both frozen local source files hash exactly to the GitHub blobs at their frozen
pins. Twenty-five source, policy, listing and evidence blobs are checked by
replay.py; full repository paths, full-commit URLs, optional cache paths, Git blob
hashes and SHA-256s are in evidence-manifest.json. Its SHA-256 is:
18e8de1ca5a97cf81ef489ce7e3e24a8f01f42272d1dd05707d24c0689868e85

Review cursor: repository lmmx/mibz2; checkpoint ctor3-closeout; closeout commit
46c322213e5780e4be2a05ebcc57348836657720; evidence digest above; scope archival
checkpoint verification only. No experiment PR was identified for this historical
closeout; PR #145 is the channel, not an assertion of a v3 experiment PR.

Correctness, safety and mutation evidence

The source harness compares all rec bytes and all raw bytes against C30::build
(ctor3.rs:1052), preserving RAW order, headers, padding and PARK as part of exact
array equality. check_block (1199) exercises counted and uncounted Ctor3, v2
counter agreement, production/oracle walker outputs, transition counters and
marker restoration. Empty synthetic cases check PARK separately (1452).

The raw closeout archives attest 11 corpora across four configurations, the
adversarial suite, 53,960 full synthetic cases with 99,840 array comparisons,
149,760 walks and 99,840 restorations, and the 542-case quick suite. The full
synthetic archive includes RAW discovery/forced groups, epoch and bucket
boundaries, changing-size reuse, exact 65,533/65,534 spans, and malformed counts:
3,995 rejected; seven accepted because the altered counts equal the input's true
histogram. The rejection harness checks that both destination arrays are empty.

The Miri console at closeout lines 27-32 independently matches the source's
registered selection (ctor3.rs:1533): 127 small/forced cases + four tiny tag/RAW
dispatch cases + two exact spans + one epoch-1 partial = 134. All completed;
exit 0. The 262 array comparisons and restorations and 393 walks equal 2/2/3
times the 131 nonempty cases; the other three are empty-block PARK checks.
Counters show leaf 6,162 and slow 189 = old-epoch 8 + partial 105 + preset RAW76,
with one wide promotion. There is no partial-run or timeout PASS here.

Miri log identifies nightly rustc 1.101.0 16d1ffac17ad8b739e32483ac2e65fe7e1804977
and Miri 0.1.0; the journal records Stacked Borrows, unpoisoned arenas. This
targeted run does not cover all bucket offsets, bulk-to-partial transitions,
epochs beyond 1, forced RAW permille 50, malformed counts, large reuse or poisoned
arenas. Those must not be described as newly Miri-verified; their evidence is the
unchanged baseline plus release-suite checks. Miri is evidence, not a proof.

The mutation diff changes precisely the two slow ordinary tests from raw&ORD to
raw==u32::MAX (source lines 565 and 583). Its console reports the intended
finish_slow<true> raw-index assertion and exits 101. The first nonempty quick
case has tag 0x80000000 interpreted as a RAW index. The repaired source passes
the quick suite. This exposes the targeted dispatch fault; it does not claim a
Miri UB report from the mutant.

The release argument is consistent with the inspected source:
- close (445) finishes only a nonempty descriptor. open (414) initializes raw,
  w0, write and end before appending; there is no eager opening over a snapshot.
- snapshot (386) sets active_tag before rows in the new epoch. Tags are unique
  over at most 16 epochs and distinct from UNOPENED; ordinary tags have bit31,
  preset RAW indices do not. Reuse constructs fresh local descriptors (890).
- Equal tags at finish imply all appended rows came from the current epoch.
  Their low labels are increasing in [0,65533], hence span <=65533, and the
  header (e<<24)|(u0<<8)|c is exactly the frozen ordinary header.
- write-w0==60 means 30 initialized u16 fields, forcing group offset zero.
  The +60 header store is aligned and inside that same 64-byte Rec. Raw-derived
  provenance is retained; reserve precedes scatter, and record allocation does
  not move during scatter. The leaf's five field stores cover exactly 60 bytes.
- Other cases use the unchanged finish logic with the repaired high-bit
  discriminator. Unpublished temporary records, canonicalization, padding and
  PARK remain the baseline's.

Original and rebuilt corpora, adversarial, full synthetic and quick synthetic
consoles compare exactly after removing only appended 'exit 0' lines. The source
and archive hashes are independently verified. The claimed identical rebuild
listings are recorded in closeout section 9.3; this review did not compile the
binary again and does not independently certify executable identity.

Instruction replay and ledger

The archived release toolchain is rustc 1.97.0 (2d8144b78), LLVM22.1.6, locked
release, empty encoded rustflags. Listing paths embed the same rustc source ID.
No different local compiler was substituted.

replay.py follows the B2 successful fast/full trace through addresses
61d30..61d93, 61e08..61e17, 61f1b..61fb8, 6231b..62329, obtaining 81 instructions,
17 explicit reads, 8 explicit writes and six push/six pop instructions.
B3 81090..8114b contains 44 instructions, 12 reads, 6 writes, no call, stack
access or save frame. Its slow exits occur after instruction5 and instruction12.
The five field stores are 16+16+16+8+4 bytes; header is one four-byte store.
No snapshot access or span check occurs on successful leaf execution.

Whole archived sizes are build1432, leaf44, slow378, refill67. The B2 whole build
is1431 and refill67. Whole-function size differences are not dynamic savings.
All three caller routes remain instruction-count neutral; scalar/run/final-sweep
boundaries cost46/47/48 including existing setup, call and leaf. See the attached
finalizer-costs.txt for full-build comparison and wording corrections.

The raw corpus JSONL has44 configuration records. All satisfy the finish
partition identities. The selected primary record is wikidata-20261026.bz2,
permille0, extra_starts=false:434 blocks,207,006,515 rows.

Item                                      Event count    Instruction contribution
B2 fast/full81 minus B3 leaf44              1,647,375        +60,952,875
5-instruction tag-failure prefix              148,066           -740,330
12-instruction same-epoch partial prefix           41               -492
4-instruction active-tag update/epoch            3,654            -14,616
ordinary opening immediate->load           1,754,362                  0
Center                                                       60,197,437
Historical conservative uncertainty                            +/-148,107
Historical interval                                       [60,049,330,60,345,544]

The denominator is eligible full fast finishes1,647,375, not all fin_fast
1,647,416. Old-epoch ordinary106,946 plus preset RAW41,120 make the148,066 tag
exits; partial41 brings slow entries to148,107. Ordinary openings1,754,362 are
instruction-neutral but add a memory read each. No call instruction saving or
double credit for eliminated push/pop is claimed. The conditional66.041 M
proposal figure is not used. Integration overlap is not inferable by summing
this isolated result with scanner or descriptor results.

Nonmaterial findings and remedies

F1, LOW, journal section5.1 caller-equivalence wording. B2 final-sweep mov
%r14,%rsi (346ea) becomes B3 mov %r14d,%esi (3c4da). Both execute one register
move; no count or memory effect. Replace 'instruction-for-instruction identical'
with 'instruction-count equivalent, with argument-width narrowing'. Formal
compiler STOP and budget do not change.

F2, LOW, journal section6 slow-path per-event explanation. The scalar residual
normalizer loses a jmp per residual field, not merely one per slow entry.
B2 62312/62316/62319/6230d/62310 takes five instructions per residual field;
B3 8106d/81071/81074/81077 takes four. There are at most three such residual
fields per partial finish, and norm_partial=42, giving at most126 favourable
instructions. The snapshot-reserve setup also saves two register moves per
invocation, bounded by868 over434 builds. An empty build can instead add three
setup instructions; conservatively charging all434 gives at most1,302. Thus
even [60,196,135,60,198,431] contains all these enumerated effects and lies well
inside the historical interval. Remedy: clarify the bound's derivation in a
new checkpoint note; do not silently replace the authoritative interval.

F3, LOW, journal section6 exact memory formulas omit setup terms. For434 nonempty
primary blocks, setup adds two reads and one store per block; epoch maintenance
adds one store per epoch. Corrected changes are -6,185,349 explicit reads and
-3,290,662 explicit writes, with -19,768,500 push/pop instructions already inside
the instruction saving. The widened initialization store also writes four extra
bytes per block without another store instruction. Coarse reported totals
(-6.19 M/-3.29 M) are unchanged. Remedy: include these setup terms in exact memory
reporting; this is not a net-instruction or gate defect. Full anchors and the
empty-block caveat are recorded in finalizer-costs.txt.

Gate record and limitations

- Archived safety/correctness closeout: PASS for its registered v3 coverage.
- Historical compiler gate: REJECT/STOP unchanged; 44>40 and12>10.
- Historical isolated structural interval: PASS as an archived achieved result,
  with nonmaterial report corrections in the supplemental cost analysis.
- Full charter section6.2 pre-integration review: HOLD pending the completed
  sequence's single pinned evidence index and independent adversarial review.
- Section7 future inclusion: CONDITIONAL on section6.2 acceptance, combined
  contribution lower end>=50 M and combined leaf<=44 (or prototype3's count).
- Runtime economics: no claim. Both prior construction captures stay INVALID,
  traversal stays VALID ADVANCE, and instruction counts are not cycles.

Replaying this standalone archive review

From the repository root, with the recorded commits available locally:

```sh
python docs/journal/compact30-research/reviews/v3/replay.py --repo .
```

The script accepts matching working-tree files, and otherwise uses read-only
`git show <full-commit>:<path>` to obtain the exact pinned version. A shallow
checkout may need the commits listed in the manifest to be fetched beforehand.
It never fetches, compiles, executes the constructor or modifies source files.

Downloaded evidence may instead be supplied using the manifest's `path` or
`cache_path` layout. `--repo` and `--evidence-dir` can be combined:

```sh
python replay.py --repo /path/to/mibz2 --evidence-dir /path/to/evidence
python replay.py --evidence-dir /path/to/all-pinned-evidence
```

The second form needs all 25 inputs under that evidence directory. Each input is
checked against both its Git blob hash and SHA-256 before any calculation. The
manifest links each input at a full immutable commit; source files and archived
consoles are not duplicated in this publication package.

Results are printed as JSON. Use `--output /path/to/new-results.json` to save them,
then compare with the included `replay-results.json`. Assertions must remain
enabled: Python `-O` is rejected. The replay verifies archived evidence only;
it remains distinct from charter section 6.2 pre-integration acceptance.

Publication portability validation used the unchanged pinned local sources plus
downloaded closeout evidence. The output exactly matches the archived numeric
results; only the manifest digest changes because URLs now use full commits and
cache paths are explicit. The earlier scratch-manifest digest was
bd571f2950b85352a06f37d0170319e9bc69acb5d97141c14a8983965abc6854.

Archives, not fresh corpus/Miri runs, were replayed. Corpus JSONL supplies
aggregated counts rather than a new independent per-block collection. Exact
rebuild identity is the coordinator's archived statement with matching suite
consoles. No final combined build exists in the evidence reviewed. A clean
sequence-closeout review must inspect its own complete evidence index anew.
