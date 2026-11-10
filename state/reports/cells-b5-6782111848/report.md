# PR365 independent B5 review: long-word writer coverage required

Inbox **6782111848**; reviewed head **57f51bbc0ca62ac3ee1cc148a1010842eea6222a**.
Verdict: **REVISE / B5 HOLD, one targeted coverage correction (F01).** No demonstrated
undefined behavior or byte mismatch. The source argument supports the design, but the
advertised Miri coverage omits the new multi-pass long-word writer. This is reachable
under ordinary admission, not merely an unreachable forced-test branch.

## Independence and evidence

I did not implement PR365 or its tests. I previously researched its mathematical context
and wrote the blog manuscript and explanatory figures. That involvement is disclosed;
this is an independent implementation review, not a blind review of the underlying idea.
Authority remains SPEC/SPEC.RESEARCHER, MODE, RULES and PROTOCOL at
`a2da49638ac54d7763de1c167bd054f67ffdc763`. No newer role authority is assumed.

Read the exact-head kernel, trait/default integration, run accessor, differential helpers,
fuzz dispatch, benchmark adapter, workflow changes, SAFETY, VALIDATION, qualification
HANDOFF, mutation script and archived outcomes. Verified the kernel's SHA-256
`c11be9107e462737ceab4a8ab4291a91ba0f2bd389010fa6295c3c545a9350be` against the original
source manifest; independently checked the archived Miri log and both mutation-outcome
digests. Details and current check readback are in [provenance.json](provenance.json).

Independent replay: [replay.py](replay.py), [replay.json](replay.json), `python replay.py`.
This is a checked-index Python model of the slot transitions and writer passes, with a
separate stable-sort Psi oracle and a separately expanded grammar/rotation. It checks
22,172 cases: all ternary columns through length 7 at every primary, two-run size edges,
seeded arbitrary columns through 256 bytes, and two targeted repeated-factor witnesses.
All pass. The original Miri fixtures and a production-admission witness are additional
checks. This does **not** execute Rust raw pointers, model provenance, or replace Miri.
Rust/Cargo are unavailable on this runtime's PATH; no native Rust, new Miri, timing or
full decoder run is claimed. No implementation source was changed.

## F01 — blocking: Miri never enters the long-rule writer

At this head `miri_cells_exact_fit_and_reuse` selects five columns and primary rows
`0,n/2,n-1`. Their selected-cycle lengths, independently computed from Psi, are:

| Column | Cycle lengths at those rows |
|---|---|
| `[1,0]` | 2, 2, 2 |
| `bbaaa` | 5, 5, 5 |
| `abababa` | 1, 3, 3 |
| 17 zeroes | 1, 1, 1 |
| `(i % 3), i=0..64` | 1, 12, 12 |

The exact-budget repeats cannot change the cycle. The other two cells Miri tests either
refuse before writing or inspect slots without invoking the writer. The exhaustive native
unit test stops at n=10; the remaining fallback/bound unit test reaches only short cycles
or refusal. Larger byte length is not evidence of larger **selected period**.

Every unmarked factor on the marked spine is shorter than its period. Consequently no
Miri fixture seeds an unmarked rule longer than SMALL=16. `pos` may be allocated for
marked rules, but all its entries remain NIL: the substantive descending propagation and
ascending expansion bodies are skipped, and `copy_long` is never called. See
`src/cells.rs:663-742`, especially 681-720 and 737-742. SAFETY's position-array row saying
“covered by Miri” overstates this reach. The green Miri job is real: final-head job
130128163701 passed all 37 selected library tests, including the three cells tests; it
does not resolve this omission.

Compact witnesses for the implementer's targeted correction:

1. Column `1^16 0^15`, primary 16, forced budget 31: model period 31, 17 cells,
   seven unmarked-rule expansions. This exercises position seeding and propagation.
2. BWT column of `(a^9 b^9)^3 c`, primary **0 and 3 in that column**. Construct the column
   by sorting its source word's cyclic rotations, taking each preceding byte; the exact
   55-byte column hex is in replay.json. At p=0 it reaches the final marked-spine recopy;
   at p=3 it reaches a long-factor copy in ascending expansion. Model cell counts 37/38.
   These primary rows deliberately test rotations, not just the source word's primary.
3. Repeat **each byte of that column** ten times, p=0: n=550, eight capped records,
   seven marked-split letters, floor 6, B=88, model 57 cells and period 55. Thus density
   (80<550), floor and budget all pass. It reaches the final long-word recopy and then
   short-cycle repetition under production admission. This is not a default-flip request.

Removing the final recopy in the independent model leaves output unwritten on witness 2.
The current three Miri fixtures cannot expose that removal. The existing five hand
mutants test count, marker, predecessor, rotation and slot capacity; none tests the
long-word position/recopy mechanism. Their recorded 5/5 result remains valid for those
targets, and is not rewritten as a failure.

**Required correction:** add small Rust Miri fixtures with asserted reach of long-rule
seeding/propagation/expansion, long-factor copy and the final recopy; use poisoned or
complemented output and include reuse with a smaller block. Confirm the production
admission witness through the actual kernel, not only this model. Add a writer-specific
mutation (for example omitting the final recopy) which compiles and is caught by the new
test, then archive a completed Miri run at the corrected source digest. If instrumentation
shows these proposed witnesses differ in Rust, retain that result and supply equivalent
small witnesses. No unrelated rewrite or expanded performance experiment is requested.
F01 remains open until the independent reviewer accepts that correction.

## Source argument and other findings

- **Isolation/fallback:** the bwt.rs diff is an export only; lib.rs adds the private
  module and counts accessor, not a default change. All Cells state is boxed. Records
  refusal delegates to the existing Segmented trait body. Ordinary column/in-place
  production routes delegate too; cooperative code is unchanged. No admission refusal
  escapes as `false` except the original fallback's broken-tiling result.
- **Safe boundary:** n<2^24, R<=n, B<=n; lengths 1..127 tile n and primary<n before
  unsafe construction. Density uses capped records and precedes traversal/allocation.
  The budget counter is checked before every cell, including equal-length cells.
  Output is not touched until successful induction and marked-period validation.
- **F02, nonblocking terminology/design discrepancy:** the implemented middle gate is
  `letters-1`, after coalescing runs and splitting out primary. It does not compute the
  union-of-cuts cell count r2 from the mathematical design. For column `1^51 0^50`, p=0,
  the marked orders have lengths `[1,50,50]` and `[50,1,50]`: three letters but four
  initial cells from cuts {0,1,50,51,101}. Thus its floor is 2, not 3. SAFETY accurately
  calls it `letters-1`; channel shorthand “r2-1 gate” should not imply the stronger
  implemented filter. This is weaker performance admission, not a memory-safety hole:
  the hard B-cell cap and exact fallback still apply. On disconnected columns even the
  letter floor is conservative for recovering one cycle; that limitation is disclosed.
- **Allocation re-derived:** byte regions total 48R+80B+4528+4G, with G=64 in debug/Miri;
  positions add at most 4B. The stated 48R+84B+4528+4G bound is correct. Reuse uses
  maxima across history, not the current small block; both fallback and cells allocations
  can remain resident. Vec growth releases its predecessor first. No whole-process
  13n memory claim or default-adoption RSS conclusion follows.
- **Induction invariant:** each live interval represents disjoint return paths; the
  weighted sum of interval width times word length cannot exceed n after closed cycles
  are removed. The marked row starts in a width-one interval. A positive cell containing
  it therefore has width one: the mark cannot split into two live factors. Word lengths
  remain <=n, so summing a single MARK bit cannot carry into the packed pointer. Each
  cell consumes positive measure; an unmarked closure discards positive measure. The
  marked closure is retained and returned before stepping beyond both order sentinels.
  Each cell needs at most one fresh slot per side and one rule; counted capacity suffices.
- **Speculation:** FAR*SLOT=288 is below PAD=4096, so prefetched *field reads* stay in
  initialized storage. Stale offsets produce wrapping hint pointers only, with no Rust
  dereference. Miri intentionally omits the x86 hint; it does not validate the intrinsic.
  `copy16` reads the value before storing, so overlap is permitted. Arena advancement is
  <=16 per cell with 32 trailing bytes; rule stores have the extra speculative slot.
  Canaries detect region crossings after the fact, not a substitute for these bounds.
- **Writer proof:** cutting immediately after the marked unit leaf cannot split an
  unmarked sibling subtree. Those subtrees partition the remaining cycle. Seeds are
  unmarked, so descendants cannot contain MARK; marked rules stay NIL in the plain
  passes. Rules refer to earlier children; descending propagation reaches children
  before ascending expansion reads their first occurrence. Equal source/destination
  skips already materialized child bytes; other long copies read completed output,
  using overlap-permitting `ptr::copy`. n+32 is initialized before obtaining output
  pointers; repetition uses checked safe copy_within with 1<=period<=n. This argument
  is plausible and the model agrees; the missing raw-pointer test reach is F01.
- **Malformed/partial output:** the integration helper compares `(output, exact Error)`
  with Segmented, including computed CRC, block and bit. Its libbzip2 side compares
  successful bytes and accepts both-error outcomes; it does not compare libbzip2 partial
  output or exact error identity. Forced fuzz opens admission but keeps structural
  validation, alternates scalar/detected kernels and asserts admissions, all three
  production refusal classes, short cycles and zero forced refusals. These are route
  counts, not assertions that every unsafe writer branch executed. Larger structured
  native cases are useful evidence, not a replacement for the missing Miri branch.

## Gate status and next action

Exact PR head still 57f51bbc at review. Miri, bench, Rust/fuzz, walk-stats, Python,
Linux/Windows builds and legacy pre-commit read success. **Both macOS jobs read cancelled**
(130128069688 x86_64, 130128069626 aarch64), not queued or passed. I did not rerun them,
diagnose their cancellation, download/recompute benchmark archives, or run timing.
The performance gate remains comment 6775049636: unchanged default arms level everywhere;
experimental figures reported only. A green bench is not certification of future default
adoption. Earlier benchmark attempts and later correction remain separate evidence.

Expected bounded safe experimental integration; source isolation/bounds are supported,
but the independent review found an untested long-word writer rather than evidence
closing all B5 obligations. The consequential next step is the small F01 coverage
correction, whose value is direct: it makes a reachable previously invisible writer
fault observable. No new speed experiment, merger or default selection is warranted here.

## Editorial reconciliation — no new production

Compared QUEUED 6769376567 and 6771749702 with reviews 6769522898/6771799134 and the
published post's source at master **2635f04e9155dd31388b24fd3f7248f87b3ddcda**,
`docs/site/blog/2026-11-10-rauzy-veech-reconstruction.md`. No live-browser/render audit
or fresh editorial production is claimed.

| Request | Observed incorporation | Remaining disposition |
|---|---|---|
| 6769376567 earlier progression | §3 retains 1.225; §5 states the scoped 1.220 floor; §4 distinguishes established Lyndon/Cartesian identity from an open cheaper construction | Earlier 1.327/7.7% comparison remains omitted pending a traceable same-run source; explicitly deferred, not implemented |
| Relay raster/chart material | Site uses redrawn inline figures and formal explanations rather than embedding the supplied synthesis raster | Reference material superseded by site-native treatment; no claim that the original image or each original chart was ported |
| 6771749702 item 1, corrected headline figure | §2 now shows 1.677 vs 2.049 and 0.367/0.937/0.347 stages, with later 0.82–0.86 variability | Incorporated in published source after the earlier figures handoff left it outstanding |
| Item 2, steps times price | Opening and §2 caption give about 0.13 steps/byte times 7.7 ns/step ≈1 ns/byte | Numerical causal explanation incorporated; a dedicated visual multiplication remains deferred |
| Item 3, wins/losses vs density | Text retains corrected Wikidata/prose/uniform/repetitive ratios and the short-run explanation | Dedicated r/n bands or comparison diagram not present; deferred |
| Item 4, a step removes multiple rows | Opening now works a 6-row/4-row interval step in prose; return-ticket figure still uses aab plus a generic batch | Larger fully worked multi-row figure remains deferred; prose example is not that figure |

Six prior causal metaphor figures are present in the source. The published record must
not be described as fully implementing every QUEUED visual suggestion. Deferred items
above are preserved, without reopening them as production work in this checkpoint.

## Operational record

Migration check passed before receipt: old branch head 1283b969e9e295e268257cd1b6be3ad7da29e6dd,
manifest SHA-256 9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0.
Receipt commit 00dae54082759d81f28ca1e602ced9b264024e1f. Pinned coord.py audit executed
against fresh connector snapshots of #145 and all four pinned role inboxes: five resolved
historical entries and one old misroute, LOG6757065549 in #162 (already finished in this
cursor). No current assignment misroute; no claim of a globally clean pinned audit.
The later master's resolved list is not substituted for this pinned result.
Route REVIEW/coordinator returns #145. Only own state and one REVIEW145 are written;
inbox, implementation, defaults, automation and other roles' state remain untouched.
