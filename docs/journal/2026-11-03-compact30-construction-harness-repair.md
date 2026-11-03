# Compact30: the construction harness's validation repair (report 197), and its revalidation

One bounded validation repair of the construction-cost harness
([2026-11-03-compact30-construction-harness.md](2026-11-03-compact30-construction-harness.md)),
as reports 195–197 decide it
([2026-11-03-compact30-harness-review.md](2026-11-03-compact30-harness-review.md)). The
untimed VM revalidation follows.

- **No construction timing** was done, and nothing here is a performance figure. The dry
  runs read clocks because the machinery does, but their numbers are not measurements; they
  are kept out of this note, except as evidence that the machinery ran.
- **No i9 or workstation run.** The workstation commands (§7) are **NOT AUTHORISED**.
- **Unchanged:**
  - the constructor at `e003a32`: `examples/c30walk/ctor.rs` byte for byte, blob `73b3179`;
  - the walker: `examples/c30walk.rs` itself is not touched by this repair;
  - the record format;
  - the timed boundaries, the paired design, the 20 rounds, the statistical rules and both
    thresholds (warm W4 b4 upper ≤ 1.06714 and e4 upper ≤ 1.02 for PASS);
  - no cache flushing.

## 1. The pins

- **The revised harness:** **`14d9ecb`**. It is the code: the `src/` accessor, the
  collector, the analysis and its tests, and the evidence tests.
- **The recipe:** pinned to `14d9ecb` (`pin=14d9ecb016a1b6d05748d795e30802e8aae31e06` in
  `.just/bench.just`). It is committed after it, with protocol draft 4, this note and its
  records.
- **The toolchain:** rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6), `CARGO_ENCODED_RUSTFLAGS=`,
  `--locked`, the release profile, baseline x86-64. The recipe verified it in every run.

## 2. The source changes (`e264eed` → `14d9ecb`)

### `src/bwt.rs`: the only production change, in full

```diff
@@ -231,6 +231,15 @@ impl TVector {
     pub fn is_empty(&self) -> bool {
         self.0.is_empty()
     }
+    /// The table's entries, read-only: for harnesses that check a built table. There is no
+    /// mutable access, since the walks rely on every link being below the length.
+    pub fn as_slice(&self) -> &[u32] {
+        &self.0
+    }
+    /// The table's allocated capacity, in entries
+    pub fn capacity(&self) -> usize {
+        self.0.capacity()
+    }
 }
```

Nothing else in `src/` changed (`git diff e264eed 14d9ecb -- src/` is this hunk).

### `examples/c30walk/cbuild.rs` (the collector)

- **A1: no reinterpretation.**
  - `tv`, `tv_mut` and their compile-time layout assertions are removed.
  - P's checks use `TVector::as_slice`: G6's comparison with the reference, G7's
    counting-scatter definition, the reference copy, and the restoration check after the
    walk.
  - P's capacity reading uses `TVector::capacity`, and the `malloc_usable_size` pointer comes
    from `as_slice().as_ptr()`.
- **The P fault injection** corrupts a copy of the expected values after the completion
  barrier, never production's table:
  - in G6 (`output:w4:P:2`), the reference copy compared with the table;
  - in G7 (`output:w1:P:0`), the counting-scatter definition's expected entry for L-position 0.

  C's injection still corrupts C's own table, which the harness owns.
- **G9: the dry run's evidence.**
  - A capture now requires `--dry-run-evidence DIR`. `--peak-rss` is gone, and a dry run
    refuses an evidence argument.
  - `verify_evidence` runs before the sizing pass and checks, with no number taken on trust:
    - the dry run's `MANIFEST.json`: exit status 0, no tracked changes, this binary's sha256
      (`/proc/self/exe`, hashed by the collector), this corpus's sha256 (hashed by the
      collector), and its records' and analysis's checksums;
    - its `c30-analysis.json`: a dry run's, every schedule check and G3, G4, G6–G10 true, no
      failures, and its peak equal to the records';
    - its `c30-construction.jsonl`: a completed (postflight present), uninjected dry run with
      no failure record, of the same corpus name and sha256, rows, binary, CPU selection and
      machine identity;
    - the peak must be at least 2 bytes a row, which the decoded inputs alone occupy.
  - Any failure writes a `failure` record (`what: evidence`) and exits 3, before any meta
    record, decode or build.
  - `required` = max(estimate, verified peak); the margin is unchanged.
  - `--verify-evidence-only` stops after the verification and the memory preflight, building
    nothing.
  - The supporting code: a SHA-256, a strict JSON reader, and the machine identity
    (hostname, `/etc/machine-id`, kernel release, CPU model, online CPUs, `MemTotal`).
  - The meta record now carries `corpus_sha256`, `binary_sha256` and `machine`.
- **Memory sources.** `MemAvailable`, `SwapTotal`, `SwapFree`, `pswpin` and `pswpout` are
  required. A missing or unparseable one is recorded as null with `readable: false`, and the
  preflight fails. `/proc/pressure/memory` is recorded as `absent`, `read` or `unreadable`;
  unreadable fails the preflight and the postflight. Swap deltas are null when either reading
  is missing.
- **G7: the poison's actual coverage.**
  - In round −1's W1 warm pass, C's `check0` record now holds `poisoned_capacity`, `needed`
    and `poison_survived`. Survival means the poisoned allocation is still the arena after
    the build, with enough capacity. There is no unconditional `poisoned: true`.
  - A new **untimed poison pass**, after the counter pass, builds every block with
    `build::<false>` into one arena preallocated for the largest block and poisoned. It
    records `poisoncheck` per block, and stops the run unless the poison survived and the
    arrays are exact. The measured arms are not preallocated or altered.
- **Capacities.** C's figure is labelled `cap_scope: rec+raw`. The bitmap is private to the
  accepted constructor, so it is not observed; its requested size, `bitmap_req_bytes` =
  8·⌈G/64⌉, is recorded beside it. P's is `tt`.
- **`cbuild-rss`** records add provenance: the corpus sha256, the pid, the start time and
  `capacity_scope`, with unreadable values as null.

### `bench/c30_construction.py` (the analysis)

- **G2,** per draft 4: every all-full W4 step's own skew ≤ 0.02, in every arm, regime and
  measured round. `start_skew` reports the per-step maximum, the count over the threshold,
  the steps checked, and the row-weighted aggregate (draft 3's measure, now deciding
  nothing).
- **G9** recomputes from the records:
  - before the run: readable, required = max(estimate, verified peak), the margin, swap,
    pressure (absent or zero; unreadable fails);
  - after the run: readable, swap, pressure, deltas exactly zero (null fails).

  A capture must carry `dry_run_evidence`. The analyser re-reads the evidence directory
  (`--dry-run-evidence`) and checks:
  - that its files are the ones the collector verified (by checksum);
  - the manifest, the analysis's gates and schedule, and that the run completed with no
    failure;
  - that its meta matches the capture's (corpus, sha256s, CPUs, machine, rows, block rows);
  - the peak's agreement across the three files, and the 2-bytes-a-row floor.

  A dry run must carry no evidence. The reasons are listed in `memory.problems`.
- **G7** checks:
  - for P: definition, walk and restored;
  - for C: exact, walk and restored, with a well-formed coverage claim (a survival claim needs
    capacity ≥ needed > 0);
  - every `poisoncheck`: survived, exact, capacity ≥ needed.

  `poison_coverage` reports the counts. S10 requires one `poisoncheck` per block, and S1 the
  three identity fields.
- **The RSS records** are validated before any summary: six records, three per arm from
  distinct processes, the capture's corpus name and sha256, block count and CPUs, the reset
  accepted, nonnegative readings, a positive baseline and capacity, and the peak after the
  pass at least the reset value. Otherwise `peak_rss_processes` is
  `{"unavailable": true, "problems": [...]}`; with none supplied it is unavailable too.
- **A7:** `block_economics.mixed_steps` reports the steps mixing full and short blocks, with
  their own e4.
- **Capacities** carry their scope, and the bitmap's requested size beside C's.

### `.just/bench.just` (the recipe)

- **Modes:**
  - `dry-run`;
  - `verify-evidence <dry-run.tar.gz>`;
  - `capture <dry-run.tar.gz>`;
  - the injections `fail-output`, `fail-output-p` (G6), `fail-output-p0` (G7), `fail-pin`,
    `fail-reject` and `fail-preflight`.
- **Evidence handling:** for `capture` and `verify-evidence` the archive must exist and end
  `.tar.gz`. It is copied into the run's directory (`dry-run-evidence/`) with its sha256
  (`archive.sha256`), so it is archived with the run, then unpacked. The collector and the
  analyser are given the unpacked directory.

## 3. Protocol draft 4

`docs/journal/2026-11-03-compact30-construction-protocol.md` is now draft 4:
- **The header** records the amendments, made before any construction timing.
- **§2:** production's table is read through the read-only accessors, with no mutable access
  and the injection on a copy.
- **§3:** the capture consumes a verified dry run, with the checks above, a refusal on
  direct invocation too, and the archive kept. Unreadable memory sources fail and never
  become zero or absent.
- **§5:**
  - the mixed step is reported on its own (A7);
  - C's capacity is labelled rec + raw, with the bitmap's requested size;
  - the six RSS records are validated before any summary.
- **§6:**
  - S1 requires the identity fields; S10 the poison pass's records;
  - **G2 is per step** (the aggregate reported, not deciding);
  - G7 records the actual coverage and adds the universal poison pass (a′);
  - G9 requires the verified evidence and readable sources;
  - the dry run checks G6–G10 (A4).
- **§9:** what is built, the VM checks (with the evidence tests and `verify-evidence`), the
  recipe's steps, and the workstation sequence.
- **§10:** the decisions of reports 195 and 197 (A1–A8, G2, G9, G7, RSS records, memory
  sources), and what stays unchanged.

## 4. The tests

- **`python3 -I bench/test_c30_construction.py`: 94 tests passed.** That is the original 61,
  adapted to the new records, plus 33:
  - **G2:**
    - the 57-step fixture passes cleanly;
    - **one all-full step at 33.3% skew among 56 clean steps is INVALID**, at a PASS base and
      at a FAIL base. The per-step maximum is 0.3333 and the aggregate stays under 2%, and
      every other gate passes;
    - a step at 1.96% passes.
  - **G9 evidence (analysis):** each of these is INVALID, with G9 false:
    - no evidence files;
    - no recorded evidence;
    - a zero peak;
    - **a consistent forgery of a 1-byte peak**;
    - required ≠ max(estimate, peak);
    - the analysis's peak disagreeing with the records';
    - a stale binary;
    - another corpus, machine or CPU selection;
    - an injected dry run;
    - a failed dry-run gate or schedule check;
    - a failure record in the dry run;
    - a nonzero exit status;
    - tracked changes;
    - evidence edited after verification;
    - a dry run carrying evidence.
  - **Memory sources:** unreadable PSI before or after, `readable: false` before or after, a
    null `pswpout` delta and a null swap reading are each INVALID. Absent PSI is accepted
    and recorded.
  - **G7:** each of these is INVALID:
    - a survival claim with no capacity;
    - the old unqualified `poisoned` flag;
    - a poison-pass block whose poison didn't survive;
    - a missing poison pass (rejected before figures).

    The coverage counts are reported.
  - **RSS:**
    - six valid records are summarised;
    - each of these makes the diagnostic unavailable and leaves the decision unchanged: a
      missing record, a duplicate process, a failed reset, another corpus, other CPUs,
      negative growth, a wrong arm count, a null reading;
    - no records means unavailable.
  - **A7:** the mixed step is reported.
- **`python3 -I bench/test_c30_build_listing.py`: 6 tests passed** (unchanged).
- **`bench/test_c30_construction_evidence.py`,** on the recipe's own binary with the VM dry
  run's directory (`evidence-tests.log`): **all 22 cases behaved as registered.**
  - **The positive control** verifies the real evidence: it records the peak, sets required
    = max(estimate, peak), and builds nothing.
  - **21 rejections,** each exit 3 with a `failure` record (`what: evidence`) and no meta or
    build record:
    - no evidence, a missing directory, a dry run given evidence;
    - a consistently resealed 1-byte peak (refused by the 2-bytes-a-row floor);
    - a resealed disagreeing peak;
    - records edited without resealing (checksum);
    - a stale binary in the manifest;
    - a resealed different recorded binary;
    - a resealed other machine;
    - other CPUs (the capture run with the CPUs reversed);
    - a resealed failed gate;
    - a resealed failed schedule;
    - a resealed injected run;
    - a resealed non-dry-run;
    - a resealed incomplete run (no postflight);
    - a resealed failure record;
    - a nonzero exit;
    - tracked changes;
    - a missing analysis;
    - another corpus hash;
    - a resealed malformed analysis.
- **Format and lint:** the Python passes `ruff check` and is `ruff format`ted; `cbuild.rs`
  is rustfmt-clean. Clippy wasn't run (not installed for 1.97.0 here).

## 5. The machine code (G3) after the repair

Built from `14d9ecb` with rustc 1.97.0, in the recipe's own worktree, and compared by
`bench/c30_build_listing.py` with the same references as before (normalised: addresses
dropped, internal branches as offsets, external targets by symbol, RIP displacements masked;
**not a bitwise identity**):

| function | size | instructions | reference | result |
|---|--:|--:|---|---|
| `mibz2::bwt::build` (P, `build_one_pass` and `build_two_pass` inlined) | 0xe97 | 814 | the reviewed pin `e003a32`'s binary | **identical** |
| `c30walk::cbuild::p_build` | 0x5 | 1 | the harness's recorded listing | identical (`jmp <mibz2::bwt::build>`) |
| `<c30walk::ctor::Ctor>::build::<false>` (C) | 0x1b5e | 1,538 | `e003a32`'s binary | **identical** |
| `<c30walk::ctor::W<false>>::promote` | 0x21e | 135 | `e003a32`'s binary | identical |
| `<c30walk::ctor::W<false>>::complete_checked` | 0x3be | 241 | `e003a32`'s binary | identical |
| `c30walk::ctor::canonicalise` | 0x305 | 173 | `e003a32`'s binary | identical |
| `c30walk::cbuild::c_build` | 0x15 | 8 | the harness's recorded listing | identical |

- **The accessor didn't change production's builder:** `bwt::build` is identical, its
  out-of-line callees are the same three (`reserve`'s slow path, `panic_bounds_check`,
  `slice_index_fail`), and no `build_one_pass` or `build_two_pass` symbol exists.
- **No substantive difference was found** in either builder's timed path.
- G3 printed `yes` in the recipe's build in all eight runs; the per-run results are in each
  `vm-*/listing/`.

## 6. The VM dry run, the evidence run and the injected failures (behaviour only)

The VM has 4 cores, so the recipe used CPUs 0–3 (its "first four" case). It has 15 GB of
RAM, no swap, and PSI present. The full primary corpus ran in the registered dry-run scope;
nothing was shrunk.

**`dry-run`, recipe exit 0:**

| what | behaviour |
|---|---|
| pin and toolchain | pin `14d9ecb`, clean; toolchain verified; 94 and 6 tests passed inside the recipe; G3 `yes` |
| placement and barriers | `barrier-test` passed. S4 and S7 passed for all 10,416 build records (3 rounds × 4 passes × 2 arms × 434) |
| regimes | both regimes ran, warm and cold, with their faults recorded |
| correctness | G6 on every build. G7 on all 434 blocks for both arms. G8 on all 434 |
| poison coverage | the poison pass covered **434 of 434** blocks. In round −1's warm pass the poison survived in **432 of 434**, recorded as such: the first block, whose arena starts empty, and one block needing more capacity than any before it |
| memory | the preflight and postflight passed, every source read. No major fault (G10). No evidence carried, as a dry run must. The recorded peak RSS was below the computed estimate |
| RSS records | all six validated (three distinct processes per arm, corpus and CPUs matching, resets accepted) and summarised |
| analysis | S0–S10 true; G3, G4 and G6–G10 true; every timing figure and the decision suppressed |
| archive | `c30-construction-20261103T165932Z-dry-run.tar.gz`, sha256 `daf168b2…`, naming commit `14d9ecb` with no tracked changes |

**`verify-evidence c30-construction-20261103T165932Z-dry-run.tar.gz`, recipe exit 0:**
- The archive was copied into the run with its sha256 (`daf168b2…`) and unpacked.
- The collector verified it, recorded `verified_peak` (the dry run's VmHWM) and required =
  max(estimate, peak), wrote `evidence_verified`, and built nothing.
- The run was archived: `5a65fbc9…`, holding the dry run's archive and its checksum.

**The injected failures,** each on the full corpus. In each, G3 printed `yes`, the collector
exited with status 3 and its failure record was written, the analysis ran and declared the
run INVALID, and the recipe archived it and exited 3.

| mode | where it stopped | the failure record |
|---|---|---|
| `fail-preflight` | after the sizing pass, before any decode | `preflight` … (injected) |
| `fail-output` | round −1, W4 warm, C, worker 2 | `output`: compact30's arrays differ from the oracle image |
| `fail-output-p` (G6) | round −1, W4 warm, P, worker 2 | `output`: production's table differs from its reference. **The real G6 comparison rejected the corrupted expected copy**; the table was untouched |
| `fail-output-p0` (G7) | round −1, W1 warm, P, block 0 | `output`: P's table: definition false, walk true, restored true. **The real G7 definition check rejected the corrupted expected entry**; the walk of the untouched table passed |
| `fail-pin` | round −1, W4 warm, worker 1 | `pin`: could not pin to CPU 1023 |
| `fail-reject` | round −1, W4 warm, C, worker 3 | `rejection`: the constructor rejected the block |

The archives' sha256s:
- fail-preflight `43431134…`;
- fail-output `06415a92…`;
- fail-output-p `b85f3ad5…`;
- fail-output-p0 `7478825d…`;
- fail-pin `84921156…`;
- fail-reject `11646b30…`.

**The recipe's refusals:** `capture` with no archive argument, and with a nonexistent archive,
each exit 2 before doing anything.

**Kept** (`2026-11-03-compact30-construction-harness-repair/`):
- each run's console, analysis JSON, manifest, barrier test, toolchain identity and listing
  results;
- the dry run's six RSS records;
- the verify-evidence run's records (preflight and `evidence_verified` only, with no timing)
  and the evidence archive's checksum;
- `evidence-tests.log`.

**Not kept:** the dry run's and the injections' record files and the tarballs, which hold the
VM's timestamps, as in the traversal protocol. Their checksums are in each manifest.

## 7. The workstation commands (NOT AUTHORISED)

From a checkout of `research/compact30-constructor` at the commit holding the recipe, with
the primary corpus in `target/walk-decoder/work/corpora`:

```
# 1. the registered dry run on the workstation (untimed): its archive is the evidence
just bench-c30-construction target/walk-decoder/work/corpora 20 dry-run
# 2. the capture, consuming that archive (same machine, binary, corpus and CPUs, or refused)
just bench-c30-construction target/walk-decoder/work/corpora 20 capture target/c30-construction-<stamp>-dry-run.tar.gz
```

An optional untimed check between them, `… 20 verify-evidence target/c30-construction-<stamp>-dry-run.tar.gz`,
confirms the archive would be accepted, without building anything.

**Neither step is authorised.** This repair is for the guardian's review.

## 8. Negative and uncertain findings

- **The evidence can't be made forgery-proof.** A forger with write access can still fabricate
  a consistent dry run whose peak is above 2 bytes a row and below the truth. The checks catch
  inconsistency, staleness, a different binary, corpus, machine or CPU selection, failed
  gates, and implausibly small values, not a careful falsification of plausible numbers.
- **The machine identity is self-reported** (hostname, machine-id, CPU model, online CPUs,
  memory, kernel). A kernel upgrade between the dry run and the capture makes the capture
  refuse, as intended for "a machine since changed", but it would also require a new dry run.
- **The poison in round −1's warm pass** still can't cover the first block or a growing build.
  This is now recorded per block, not claimed; the separate poison pass gives universal
  coverage for `build::<false>`, outside the measured arms.
- **C's bitmap capacity is not observed** (private to the accepted `ctor.rs`). Only its
  requested size is recorded.
- **G2 is now stricter than the traversal experiment's gate.** One descheduled worker on one
  step makes a capture INVALID, as report 197 accepts.
- **The analysis's figure stage** (ratios, G1, G2 on real records, G5) is still tested only on
  synthetic records. A dry run suppresses it.
- **Not exercised here:**
  - a capture (not authorised);
  - the second-to-fifth core choice (the VM has four cores);
  - a real memory shortage;
  - an unreadable PSI file on a real system (only the synthetic fixtures);
  - clippy.
- **The A2 wrapper overhead difference** (C's eight instructions against P's one jump) is
  unchanged and documented.
