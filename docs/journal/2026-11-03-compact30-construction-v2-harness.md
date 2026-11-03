# Compact30: constructor v2's construction-cost protocol and harness, validated untimed on the VM

**What this is.** Report 211 authorised one bounded v2 construction-cost protocol and harness
adaptation, with untimed VM validation only
([the audit and report 211](2026-11-03-compact30-constructor-v2-audit.md), "Send Claude this").
This entry records:
- the protocol, [2026-11-03-compact30-construction-protocol-v2.md](2026-11-03-compact30-construction-protocol-v2.md);
- the harness's v2 mode, pinned at **`fa21f33`**;
- its tests, the machine-code comparisons, and the proof that draft 4's mode is unchanged;
- the VM's dry run, evidence run and injected failures;
- one proposed workstation command, **not run**.

**Nothing was timed.** No construction timing, benchmark, i9 or workstation run, constructor
tuning, walker change or production integration was done. The dry runs' timestamps are VM
artefacts of an untimed check. **They are invalid for inference, and no figure is drawn from
them.** Every timing figure is suppressed in a dry run by construction (§6 of the protocol).

**Frozen, and unchanged** (each `git diff` empty):
- `examples/c30walk/ctor2.rs` against `8beffe2`, and `examples/c30walk/ctor.rs` against `e003a32`;
- the walker and `C30::build` (in `examples/c30walk.rs`, whose only change is four dispatch
  lines and two usage lines), and `src/` against `14d9ecb`;
- draft 4, `docs/journal/2026-11-03-compact30-construction-protocol.md`, against `e4ffc8d`;
- v1's capture, its analysis and its journals;
- v1's recipe, `bench-c30-construction` (pinned to `14d9ecb`): the v2 recipe is added after it,
  with no line of it changed.

## 1. The commits

| commit | what |
|---|---|
| `5305bd5` | the harness's v2 mode, its tests and the frozen v2 reference listings (the first pin) |
| **`fa21f33`** | **the pin.** One fix found by the first VM validation (§6.4): a v2 run that fails before writing its meta record is reported as draft 4 reports it |
| this entry's commit | the recipe `bench-c30-construction-v2` pinned to `fa21f33`, the protocol, this journal and its evidence |

**Files changed at the pin, against `14d9ecb` (v1's harness pin):**
- `examples/c30walk/cbuild.rs`: the collector, generic over the candidate (`Candidate`, for
  `Ctor` and `Ctor2`), with `c2_build`, `main_v2` and `rss_v2`;
- `examples/c30walk.rs`: `cbuild-v2` and `cbuild-rss-v2` dispatched, and their usage lines;
- `bench/c30_construction.py`: `analyse_v2` and `--protocol v2`;
- `bench/c30_build_listing.py`: `--protocol v2` and `FUNCS_V2`;
- `bench/test_c30_construction_v2.py` (new);
- `bench/test_c30_construction_evidence.py`: an optional subcommand argument, and v2's
  protocol cases;
- `docs/journal/2026-11-03-compact30-construction-v2-harness/listings/` (new, the frozen
  references).

## 2. The protocol's changes against draft 4

The protocol states them in its §A with their justifications, quoting report 211. In short:

| | draft 4 | v2 |
|---|---|---|
| C | `Ctor::build::<false>` | `Ctor2::build::<false>` at `8beffe2`, natural configuration |
| C's window | v1's work | all of v2's: descriptor and snapshot initialisation, temporary writes, deferred finishing and normalisation, RAW work, canonical ordering, allocation, padding and publication |
| G1 | P's W1 cost, W4 cost and W4 completion, max ÷ min ≤ 1.05, **both regimes** | P's **warm W4** cost and **warm W4** completion only; W1, cold and every C ratio reported |
| G2 | **every** all-full W4 step ≤ 2%, every arm, regime, round | **each arm's warm** all-full steps over the 20 rounds: aggregate ≤ 2%; at most ⌊N/1000⌋ over 2% (N = 1,140, so **1 per arm**); none over 10%; cold reported |
| identifier | none | `protocol: c30-construction-v2`, `candidate: ctor2` in the meta and RSS records; a capture accepts only a v2 dry run's evidence |
| G3 | v1's listings | v2's frozen listings; P against v1's reviewed listings, unchanged |
| counters | v1's | v2's `Stats2`, and a v2 slow-path table |
| estimate | — | adds v2's 16 KiB of snapshots per worker's candidate arena (64 KiB on the primary corpus) |
| recipe | `bench-c30-construction` | `bench-c30-construction-v2`, its own pin and worktree |

**The economic limits, the deciding measures, the statistics and every other safeguard are
draft 4's.** PASS needs b4+ ≤ 1.06714 and e4+ ≤ 1.02.

**Why G1 and G2 change (the protocol's §A.1),** citing v1's archived record and computing
nothing new:
- v1's capture failed G1 only in P's one-worker cost: max ÷ min 1.2274 warm, 1.2232 cold.
  Both arms' W4 measures were within 1.022–1.028.
- v1's capture failed G2 on a single step of 4,560, at 4.125% (round 7, warm, P, step 1).
  The next largest step in that pass was 0.11%, and the largest aggregate 0.09%.
- **The rules are prospective, for v2 only.** v1's capture stays INVALID. The analysis doesn't
  apply v2's gates to v1's records: it refuses them (§3.3), and no "v1 under the new rules"
  figure exists anywhere.

## 3. The harness

### 3.1 The collector (`cbuild-v2`, `cbuild-rss-v2`)

- **One schedule, two candidates.** `Arenas<K>`, `Worker<K>`, `pass_w1`, `pass_w4`, `run` and
  `rss_with` are generic over `K: Candidate`. `Candidate` gives:
  - the timed call (`#[inline(always)]`, calling the arm's `#[inline(never)]` wrapper and
    nothing else);
  - the untimed `build::<false>` for the poison pass;
  - the untimed counted build and its counter fields;
  - the protocol identifier, the name and the scratch size.
- **v1's instantiation** (`cbuild`, `cbuild-rss`, `K = Ctor`):
  - `PROTOCOL = None`, so no field is added to any record;
  - `SCRATCH = 0`, so the estimate is unchanged;
  - `verify_evidence` checks exactly draft 4's conditions, in their order.
- **v2's** (`K = Ctor2`): C's window holds `c2_build`, which calls `Ctor2::build::<false>(ll,
  counts, 0, rec, raw)`.
  - In the cold regime the `Ctor2` is fresh (`Arenas::default()` in the preparation), so its
    bitmap and snapshot allocations happen inside the window.
  - The meta record adds `protocol`, `candidate` and `candidate_scratch_bytes` (16,384).
  - Each RSS record adds `protocol`.
  - The counter pass runs `Ctor2::build::<true>` from zeroed `Stats2` and records its fields
    per block.
  - The poison pass runs `Ctor2::build::<false>`.
  - `verify_evidence` additionally requires the dry run's records and analysis to carry the
    identifier. It checks this after every draft 4 condition.
- **The records' other kinds and fields are draft 4's,** so the analysis's schedule checks
  apply unchanged.

### 3.2 The analysis (`c30_construction.py --protocol v2`)

**`analyse_v2` runs draft 4's `analyse`, unchanged, on v2's records,** then applies v2's
differences:
1. **S1_protocol:**
   - A meta record without v2's identifier and candidate: INVALID, before any figure.
   - No meta record at all: draft 4's own "no meta record" verdict, with S1_protocol false.
2. **G1:** draft 4's six G1 keys are replaced by `G1_warm_w4_cpu_ns_row` and
   `G1_warm_w4_completion_ns` (P, ≤ 1.05). `stability` reports all twelve ratios (both arms,
   both regimes, W1 cost, W4 cost, W4 completion), with the rounds of each extreme and
   whether each gates.
3. **G2:** draft 4's `G2_simultaneous_start` is replaced by three gates:
   - `G2_aggregate`;
   - `G2_outlier_allowance`;
   - `G2_max_step`.

   They are computed from the build records by draft 4's per-step formula
   (`step_skews`, `g2_summary`). `start_skew` reports, per arm:
   - the warm (gated) and cold (reported) N;
   - the aggregate;
   - the maximum;
   - the outlier count;
   - the allowance;
   - every outlier's round, step, arm and skew;
   - the count over 10%;
   - the largest per-round aggregate.
4. **The evidence:** a capture's dry-run evidence must carry the identifier in its analysis
   and its meta record, or G9 fails. RSS records without it make the RSS diagnostic
   unavailable.
5. **v2's slow-path table** (`slow_paths_v2_by_one_row_decile`, reported only). Draft 4's
   table is computed too, reading v2's wide finishes as its promotions.
6. **Draft 4's decision table**, unchanged, on the recomputed validity.

The default mode is draft 4's `analyse`, as reviewed. Its `main()` refuses records carrying a
protocol identifier (exit 2, "analyse them with --protocol v2").

### 3.3 Draft 4's mode is unchanged

Of draft 4's analysis code, two lines changed: the usage docstring's argument line, and the
call in `main()`, which now dispatches on `--protocol`. Every function of draft 4's analysis
is textually unchanged. Of the collector, v1's path changed only by being made generic: the
record format strings are the same text, with v2's fields appended as an empty string for v1.

**Proof by replay** (the analyser at the pin, in its default mode, on v1's archives as they are
in this repository):

| archive | replayed against | result |
|---|---|---|
| v1's capture `c30-construction-20261103T172906Z` (its records, six RSS files and dry-run evidence) | its archived `c30-analysis.json`, and `c30-analysis-replayed.json` | **byte-identical** (sha256 `52d40743…5205c7a5`); the analyser's console identical to the archived console's analysis lines |
| v1's i9 dry run `c30-construction-20261103T172121Z-dry-run` | its archived `c30-analysis.json` | **byte-identical**; console identical |

**Further checks of draft 4's mode:**
- **Its 94 tests pass unchanged,** and its 6 listing tests.
- **G3 in v1's mode, on the v2 harness binary,** against v1's reviewed listings: `yes`, all
  seven functions identical, all checks true (`machine-code/v1-mode-on-v2-binary/`).
- **Record format:** a v1-mode `cbuild --dry-run` of the new binary, on the small `text.bz2`
  corpus, has the same record kinds as the archived i9 dry run, each with exactly the same keys
  in the same order. So do the nested preflight parts, the calibration entries and the RSS
  record (`v1-mode-format.txt`).

**Known asymmetry, kept deliberately:** in v1's mode the collector doesn't check a dry run's
protocol identifier, as draft 4 doesn't. A v1-mode capture of the new binary would therefore
accept a v2 dry run of the same binary. No v1 capture is authorised, and v1's recipe builds
`14d9ecb`, not this binary. v2's mode refuses a v1-mode dry run.

### 3.4 The listing check (`c30_build_listing.py --protocol v2`) and the recipe

- **The listing check compares `FUNCS_V2`** with `listings/` and checks four things:
  - `p_build` only transfers to `mibz2::bwt::build`;
  - `c2_build` only calls `Ctor2::build::<false>`;
  - `bwt::build` inlines both passes;
  - every `c30walk` function the four checked constructor functions call is itself checked.

  Library generics such as `RawVec<RawG>::grow_one` aren't c30walk functions.
- **The recipe** (`just bench-c30-construction-v2`) is draft 4's recipe with:
  - its own pin and worktree (`target/c30-construction-v2-pin`);
  - output under `target/c30-construction-v2-<stamp>[-mode]`;
  - v2's collector and RSS mode;
  - v2's analysis and listing mode;
  - v2's tests run beside draft 4's.

  Its modes, stops and archive are draft 4's.

## 4. The tests

| suite | result |
|---|---|
| `test_c30_construction.py` (draft 4's analysis, unchanged) | **94 passed** |
| `test_c30_construction_v2.py`, part 1: draft 4's suite under v2 | **89 passed**, its 5 draft-4-G2 tests replaced by part 2 |
| `test_c30_construction_v2.py`, part 2: v2's rules | **30 passed** |
| `test_c30_build_listing.py` | **6 passed** |
| `test_c30_construction_evidence.py … cbuild-v2` on the pin's binary (§6.3) | **passed**: the positive control and 24 rejections (21 of draft 4's cases, 3 of v2's) |

**Part 1** reruns every draft 4 test with v2 records and `analyse_v2`, so these all stay
INVALID under v2, never PASS, FAIL or INCONCLUSIVE:
- every schedule defect (a missing, duplicated or extra build; lists not the snake; shifted
  workers; wrong CPUs; a swapped arm order; barrier and ordering violations; an unknown
  record; no meta);
- wrong denominators (rows doubled or rewritten, a wrong full-block count);
- missing or malformed calibration, and too large a clock overhead;
- a correctness failure (an inexact build, a failed or missing round −1 check, the one-pass
  precondition, missing counter and poison records, a poison claimed without capacity);
- an identity failure;
- every memory gate (margin, swap, pressure, unreadable sources, the evidence's forgeries,
  staleness, other binary, corpus, machine or CPUs);
- a major fault;
- too few rounds, a dry-run label on 20 rounds, a capture with injection;
- and the decision table at its boundaries.

**Part 2, the G1/G2 regression table.** The fixture is 228 full blocks and 2 short, so 57
all-full steps a pass, N = 1,140 per arm warm, and an allowance of 1, as on the primary corpus.
A step's skew is set exactly: its worker 3 starts later and ends where it did.

| case | the records | v2's outcome | failing gates |
|---|---|---|---|
| the base | no stagger | **VALID, PASS** | — |
| isolated moderate interruptions | warm P one step 5%, warm C one step 8%; cold P 6%, cold C 3% and 7% | **VALID, PASS**; outliers located (3, 10, P) and (7, 20, C); cold's two reported | — |
| a 3% step | one warm P step | **VALID, PASS** | — |
| widespread staggered starts | every step 9.1% | **INVALID** | aggregate, allowance, max |
| widespread, at a FAIL base | the same, C at 1.2× | **INVALID** (never FAIL) | aggregate, allowance, max |
| outliers above the allowance | three warm P steps at 3% (aggregate < 0.1%) | **INVALID** | allowance only |
| the allowance at its limit | one warm P step at 5% | **VALID** | — |
| one over the allowance | two warm P steps at 5% | **INVALID** | allowance only |
| the allowance is per arm | one at 5% in P and one in C | **VALID** | — |
| exactly 2% | three warm P steps at 2.000% | **VALID**, no outlier | — |
| a single 33.3% step | one warm P step (worker 3 half a build late) | **INVALID** | max only |
| a single 33.3% step in C, at a FAIL base | one warm C step | **INVALID** (never FAIL) | max only |
| just above 10% | one warm P step at 10.01% | **INVALID** | max only |
| exactly 10% | one warm P step at 10.000% | **VALID** | — |
| the aggregate binds alone | every warm P step at 1.9993% and one at 10% | **INVALID**: aggregate 2.006%, one outlier, max 10% | aggregate only |
| the aggregate's definition | five C steps staggered | equals Σ skews ÷ N, and the mean of the per-round aggregates | — |
| cold staggering alone | a 33.3% cold P step and a 25% cold C step in every round | **VALID, PASS**; reported | — |
| W1 instability alone | P's W1 CPU 0.8× in rounds 6, 8, 12, 14, 16, both regimes (max ÷ min 1.25, as v1's capture's shape) | **VALID, PASS**; reported, `gates: false` | — |
| cold instability alone | P's cold W4 CPU 0.85× in two rounds | **VALID, PASS**; reported | — |
| C's instability | C's warm W4 CPU 0.9× in one round | **VALID**; reported | — |
| P's warm W4 cost unstable | P's warm W4 CPU 0.9× in round 4 (1.11) | **INVALID**, min round 4 | G1 warm W4 cost only |
| P's warm W4 cost just inside | 0.955× (1.047) | **VALID** | — |
| P's warm W4 completion unstable | P's worker 0 at 1.1× the step mean in every round but 6 (completion −12% in round 6; cost 3%) | **INVALID** | G1 warm W4 completion only |
| v1's records under v2 | draft 4's fixture, no identifier | **INVALID**, `{S1_protocol: false}` only, no figure | S1_protocol |
| another candidate | `candidate: ctor` | **INVALID**, no figure | S1_protocol |
| v2's records under draft 4's mode | a v2 dry run's records | draft 4's `main()` exits 2; v2's mode accepts them | — |
| a failure before the meta record | the preflight's failure | **INVALID: no meta record** | S1_meta, S1_protocol |
| a v1 dry run's evidence | the evidence's analysis or meta without the identifier, or another candidate | **INVALID** | G9 only |
| RSS records without the identifier | one of six | RSS unavailable; decision unaffected | — |
| a v2 dry run | rounds −1, 0, 1 | G1, G2 and figures suppressed; protocol recorded | — |
| v2's slow-path table | the base | reported | — |

## 5. The machine code (static observations only; nothing is inferred about runtime)

The harness binary at the pin, built by the recipe: rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6),
no rustflags, `--locked`, release (codegen-units 1, fat LTO), sha256 ``ea2662dc…52441937``.

| function | size | instructions | compared with | result |
|---|--:|--:|---|---|
| `<c30walk::ctor2::Ctor2>::build::<false>` | 0x18e5 | 1,431 | the audited prototype listing (`8beffe2`) | **identical after normalisation**; frozen as `listings/ctor2-build-false.asm` |
| `<c30walk::ctor2::W<false>>::finish` | 0x5fa | 379 | the prototype listing | **identical**; frozen |
| `<c30walk::ctor2::W<false>>::refill` | 0xec | 67 | the prototype listing | **identical**; frozen |
| `c30walk::ctor::canonicalise` (v1's, shared) | 0x305 | 173 | v1's reviewed listing | **identical** (the reference is v1's file, copied unchanged) |
| `c30walk::cbuild::c2_build` | 0x15 | 8 | — (new) | recorded and frozen |
| `c30walk::cbuild::p_build` | 0x5 | 1 | v1's reviewed listing | **identical** (v1's file) |
| `mibz2::bwt::build` (one and two pass inlined) | 0xe97 | 814 | v1's reviewed listing | **identical** (v1's file); callees `reserve`'s slow path, `panic_bounds_check`, `slice_index_fail`; no `build_one_pass` or `build_two_pass` symbol |

- **Production's builder is identical to its reviewed listing.** The recipe's G3 printed
  `yes` in every run (§6).
- **v2's constructor is the audited code.** Against the prototype's listings, the three
  functions differ only in absolute addresses, call targets' addresses and RIP displacements:
  199, 74 and 8 instructions' raw text, and no instruction after normalisation
  (`machine-code/v2-against-prototype.txt`).
  - **Inlining into this binary changed nothing.** `build::<false>`, `finish` and `refill` are
    `#[inline(never)]` boundaries, and their own inlined callees (the scan, tiles, runs, the
    emitter, snapshots, epoch search) compiled to the same instructions.
  - The only new code on C's path is `c2_build`: it moves `rec` and `raw` into place, zeroes
    `permille` and calls `build::<false>`. That is the same eight-instruction shape as v1's
    `c_build`, against P's one `jmp`, the wrapper overhead difference draft 4 documented (A2).
- **Its out-of-line callees:**
  - `build::<false>` calls `refill` (4 sites), `finish` (2), `canonicalise` (1), and library
    code: `RawVec` growth, `reserve`, `handle_error`, the bounds-check panics, and glibc's
    memset, memcpy and free.
  - `finish` calls `RawVec<RawG>::grow_one` and bounds panics; `refill` calls `finish`.
  - All three c30walk callees are checked (`v2_constructor_callees_all_checked: true`).
- **The scalar append** (`build::<false>` at offsets +0x1250 to +0x127a and +0x12e0 to +0x1306,
  the prototype's `34350`–`3437a` and `343e0`–`34406`): 12 instructions a row. They are the
  byte load, the descriptor's index (`lea` ×2), `write` load, `end` compare, the branch to
  `refill`, the label store, `write += 2` (`addq` to memory), the increments and the loop
  branch. There is no header load, RAW test, offset or span arithmetic.
- **Per-tile source updates:** the label is an incremented register within a tile; y and p
  advance once per tile; there is no `div`.
- **The finaliser (`finish`) touches only its record's bytes 0–59 when normalising:**
  - The full-group path (`finish`+0x244 to +0x283):
    - `movdqu` loads and stores at 0x00, 0x10 and 0x20 of the record's first field;
    - `movq` at 0x30 (bytes 48–55);
    - `movd` at 0x38 (bytes 56–59);
    - `psubw` of the broadcast u₀ in each.

    So each of bytes 0–59 is read once and written once.
  - The header is stored once, before the normaliser, by a scalar `mov %esi,0x3c(%r8,%rcx,1)`
    (+0x21e), and nothing vector-stores at or above 0x3c.
  - The partial path's loops (+0x470 and +0x4d0) run over the portion's own fields, `(%rdi,%rdx,2)`
    from its first field for its length (< 30), in 16-lane, 4-lane and scalar steps.
  - The wide-promotion path writes 0xFFFF fields with `movdqa` at 0x00, 0x10 and 0x20 and an
    overlapping `movdqu` at 0x2c (ending at byte 59), then the RAW header at 0x3c by a scalar
    store.
  - The 128-byte `movdqa` zeroing at +0x2b1 is a new RAW group's (`raw`), not a record.
- **The bulk emitter** in `build::<false>` is the prototype's loop (its `341d0`–`34233`),
  unchanged.
- **The workspace:**
  - the descriptors (256 × 40 bytes) live in `build::<false>`'s frame;
  - the snapshots (16 × 1 KiB) live on the heap in `Ctor2`, reserved inside the window on
    first use;
  - the bitmap, ⌈G/64⌉ words, also on the heap.

## 6. The VM validation (behaviour only, untimed)

The VM has 4 cores, so the recipe used CPUs 0–3 (its "first four" case). It has 15 GB of RAM,
no swap and PSI present. Every run used the full primary corpus, and nothing was shrunk. Every
run at the pin printed in the recipe:
- 94, 89 + 30 and 6 tests passed;
- G3 `yes`;
- `barrier-test` passed.

### 6.1 `dry-run` (recipe exit 0)

| what | behaviour |
|---|---|
| pin and toolchain | `fa21f33`, clean; toolchain verified |
| identifier | meta record `protocol: c30-construction-v2`, `candidate: ctor2`, `candidate_scratch_bytes: 16384`; analysis `protocol` recorded; S1_protocol true |
| schedule | S0–S10 true for all 10,416 build records (3 rounds × 4 passes × 2 arms × 434) |
| correctness | G6 on every build; G7 on all 434 blocks for both arms; G8 on all 434 |
| poison | the poison pass covered **434 of 434** blocks; round −1's warm pass **432 of 434** (the first block, whose arena starts empty, and one growing block), recorded as such |
| memory | estimate 1,855,391,770 bytes (v1's 1,855,326,234 plus 65,536 for v2's snapshots); preflight and postflight passed, every source read; peak RSS 1,771,724,800 bytes (0.955 of the estimate); no major fault (G10) |
| RSS | six records validated, each carrying the identifier |
| analysis | S0–S10 and S1_protocol true; G3, G4, G6–G10 true; every timing figure, G1, G2, G5 and the decision suppressed |
| archive | `c30-construction-v2-20261103T214131Z-dry-run.tar.gz`, sha256 `dc092dcb…34e86819`, naming commit `fa21f33` with no tracked changes; kept in this entry's folder |

**v2's structural counters on the primary corpus** (the untimed counter pass, `build::<true>`,
a property of the input and the algorithm, not a timing):

| counter | total |
|---|--:|
| rows: bulk (complete records), fragment, one-row | 153,819,000; 14,892,632; 38,294,883 |
| runs, run rows; complete groups | 610,521, 168,711,632; 5,127,300 |
| finishes: fast, non-fast ordinary, wide, crossing, forced | 1,647,416; 75,997; 30,949; 41,120; 0 |
| normalisations: full groups, partial groups, fields | 1,723,371; 42; 51,701,707 |
| RAW fields read and entries written; fields restored | 1,485,808 each; 557,338 |
| epochs; epoch searches; runs split at an epoch; multi-epoch portions | 3,654; 127,249; 2,638; 117,029 |
| RAW groups, moved, cycles | 49,633; 48,795; 1,135 |
| largest ordinary span; smallest wide span | 65,529; 65,535 |

The 1,723,371 full-group normalisations match the audit's "about 1.72 M".

### 6.2 `verify-evidence` (recipe exit 0)

The dry run's archive was copied into the run with its sha256 and unpacked. The collector
verified it, including v2's identifier, recorded `verified_peak` and required = max(estimate,
peak), wrote `evidence_verified`, and built nothing.
- The verified peak was 1,771,724,800 bytes, so required = the estimate, 1,855,391,770.
- The run's archive is `c30-construction-v2-20261103T214255Z-verify-evidence.tar.gz`, sha256
  `ff59a662…3f9bf737` (the full checksums are in §6.5).

### 6.3 The collector's evidence rejections (`test_c30_construction_evidence.py … cbuild-v2`)

On the pin's binary, directly, with the dry run's directory. Every case stopped at the evidence check with a `failure` record whose `what` is
`evidence`, before any meta record, decode or build (`evidence-tests.log`).
- **The positive control** verified the unmodified evidence: the peak was recorded,
  required = max(estimate, peak), and nothing was built.
- **Draft 4's 21 rejections:**
  - no evidence argument, a missing directory, or a dry run given evidence;
  - a tiny peak, or a peak disagreeing with the analysis (both resealed);
  - edited records (checksum);
  - a stale binary, or another recorded binary (resealed);
  - another machine (resealed), or other CPUs;
  - a failed gate, a failed schedule, an injected dry run, not a dry run, an incomplete dry
    run, or a failure record (all resealed);
  - a nonzero exit, tracked changes, no analysis, another corpus hash, or malformed JSON
    (resealed).
- **v2's 3 rejections:**
  - the dry run's meta record without v2's identifier (a v1-mode dry run; resealed);
  - its analysis without the identifier (resealed);
  - another protocol's identifier (resealed).

### 6.4 The injected failures

In each:
- G3 printed `yes`;
- the collector exited 3 with its failure record written;
- the analysis ran and declared the run INVALID with no figure;
- the recipe archived the run and exited 3.

| mode | where it stopped | the failure record | the analysis |
|---|---|---|---|
| `fail-preflight` | after the sizing pass, before any decode or meta record | `preflight` … (injected) | INVALID: no meta record |
| `fail-output` | round −1, W4 warm, C, worker 2, block 7 | `output`: compact30's arrays differ from the oracle image | INVALID: not the registered schedule; no figures |
| `fail-output-p` (G6) | round −1, W4 warm, P, worker 2, block 7 | `output`: production's table differs from its reference. **The real G6 comparison rejected the corrupted expected copy**; the table was untouched | INVALID, no figures |
| `fail-output-p0` (G7) | round −1, W1 warm, P, block 0 | `output`: P's table: definition false, walk true, restored true. **The real G7 definition check rejected the corrupted expected entry** | INVALID, no figures |
| `fail-pin` | round −1, W4 warm, worker 1 | `pin`: could not pin to CPU 1023 | INVALID, no figures |
| `fail-reject` | round −1, W4 warm, C, worker 3, block 9 | `rejection`: the constructor rejected the block (`Ctor2::build` returned false on counts that are not L's own) | INVALID, no figures |

**The recipe's refusals:** `capture` with no archive argument, and with a nonexistent
archive, each exit 2 before anything runs. ("capture needs a v2 dry run's archive …", "no dry-run archive at …").

**What the first validation found** (at `5305bd5`, superseded): the same runs, with the same
outcomes, except that `fail-preflight`'s analysis said "INVALID: the records are not protocol
v2's". The run fails before writing its meta record, so the cause is a failure without a meta
record, not a protocol mismatch. `fa21f33` reports it as draft 4 does ("INVALID: no meta
record", with its failure record), and a test covers it. The verdict was INVALID either way.
Those runs' archives were discarded. Everything in this entry is from the pin.

### 6.5 Kept (`2026-11-03-compact30-construction-v2-harness/`)

- `listings/`: the frozen references (in the pin's tree);
- `machine-code/`: the v2-against-prototype comparison and v1's mode on the v2 binary;
- `v1-unchanged/`: the replay log and the v1-mode record-format comparison;
- `vm-*`: each run's console, analysis, manifest, barrier test, toolchain identity and
  listing results; the dry run's six RSS records; the verify-evidence run's records
  (preflight and `evidence_verified`, no timing); each failure's record;
- `c30-construction-v2-20261103T214131Z-dry-run.tar.gz`: the dry run's archive, the evidence
  the verify-evidence run consumed. Its records hold VM timestamps of an untimed run: **they
  are not measurements, and are invalid for inference**;
- `evidence-tests.log`.

The other runs' record files and tarballs aren't kept, as in v1's harness journals; their
files' checksums are in each manifest. The archives' own sha256s:
- dry-run `dc092dcb968fbcbdcebb2842161a45ced82d34632439696376617eeb34e86819`;
- verify-evidence `ff59a6626e371ce610806ef8c30bc74a4b8ad4c2d3ea50472809e5433f9bf737`;
- fail-output `0747aad7b299afd50a05036fa2f61433b085163d067d3dfacc50d5e4462af204`;
- fail-output-p `09601cb9ff082c1067d9f56e887dbfb4c69a856736f88685736e73a57330c66c`;
- fail-output-p0 `6e4f8486ad7ac3b55d27af5191541f27930346f310d1478f4ec50c04e905b270`;
- fail-pin `a57ad7247eb43a841ea136adf362e03cd4651712cdcf0487aa035a848df82adb`;
- fail-reject `2e30b9b6e47bb81c381b56020ad891817c94ebb43ddebdfb4cee6331c8d66c44`;
- fail-preflight `2d376e157584a7f0f13b40390e0c2d4af7fd74716fc9e1c5bc46350e2fd64b77`.

## 7. The proposed workstation command (for the guardian's review; NOT RUN, NOT AUTHORISED)

On the i9, from `/home/louis/dev/mibz2`, at this entry's commit on
`research/compact30-constructor` (the recipe pinned to `fa21f33`), with the primary corpus in
`target/walk-decoder/work/corpora`. **The one proposed command** is the registered v2 dry run,
whose archive is the evidence a capture needs:

```
just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 dry-run
```

If that passes and a capture is separately approved, it would be followed by:

```
just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 capture target/c30-construction-v2-<stamp>-dry-run.tar.gz
```

An optional untimed `… 20 verify-evidence <that archive>` between them confirms the archive
would be accepted, building nothing. **None of these is authorised by this entry.**

## 8. Uncertainties and negative findings

- **The figure stage of v2's analysis** (G1, G2, the ratios, G5, the decision) is tested only
  on synthetic records. A dry run suppresses it, as draft 4's does.
- **v2's G2 is looser than draft 4's.** One interruption per arm up to 10% is tolerated. It
  can't be tightened after data is seen, and it isn't applied to v1.
- **The allowance depends on N:** ⌊N/1000⌋ is 1 for N = 1,140, and 0 for any corpus with fewer
  than 1,000 all-full steps an arm. That is only the primary corpus here.
- **The aggregate pools the 20 rounds, per arm,** as report 211 words it ("across the 20
  rounds"). A single bad round could therefore stay under a pooled 2% where a per-round
  aggregate would not. The per-round maxima are reported beside it. The other two G2
  conditions bound any one step.
- **The collector's v1 mode is not literally the reviewed binary.** It is the same source
  made generic, its timed functions identical to v1's reviewed listings, and its record format
  identical. v1's recipe still builds `14d9ecb`. In v1's mode the collector doesn't check a
  dry run's protocol (§3.3).
- **One failure-path message in v1's mode changed text:** the counter pass's internal
  `assert!` now names `ok`, not the call, so a panic there would record a different message.
  It occurs only on a correctness failure.
- **v2's bitmap and snapshots aren't observed,** only requested; the descriptors are on the
  stack. The memory figures cover `rec` + `raw`, as v1's did.
- **The second-to-fifth core choice, a real memory shortage and an unreadable PSI file** are
  not exercised on the VM (four cores, ample memory).
- **Clippy wasn't run** (not installed for 1.97.0). The Rust is rustfmt-clean (rustfmt
  `2d8144b788`), and the Python is ruff-clean.
- **No Miri or sanitiser runs in the harness.** The constructor's memory-safety evidence is the
  audit's and the prototype's (report 211 accepted the partial Miri run).
