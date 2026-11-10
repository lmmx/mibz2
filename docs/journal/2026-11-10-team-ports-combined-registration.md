# 2026-11-10: master against the three stacked team ports, one combined diagnostic (DRAFT registration)

**Status: DRAFT, revision 3. Not frozen, nothing built or run for it.** Revision 3 answers
Guardian D091 (#145 comment 6763469482; `state/decisions.md` at `5a86f4bd`, section D091: G1
concurs that (b) is reported, G2 HOLDs the freeze on the checker-exit contradiction) and the
Researcher's REVISE (6763607539, item 2; `state/reports/diagnostic-revisions-6763474717/report.md`
at `ee1b1f80`), as listed in §14. Revision 2 answered the
Researcher's REVIEW 6763089509 item 3 (REVISE; `03-combined.md` and `probes.py` at
`07b453d1`), as listed in §13. This is the one
combined diagnostic Guardian D089 admits (#145 comment 6761524241; `state/decisions.md#d089`
at `3f2b0a49`): a distinct question of integrated cost and engagement. The Researcher reviews
it next. It is frozen only at a commit the Researcher's PROCEED names. It runs once, after
that, with no retry.

## 0. What this diagnostic cannot do

- **It cannot turn any of the three failed registrations into a PASS:**
  - port 1 (#335): condition 3 is UNMET as registered and stays FAIL-MEASUREMENT (D085);
  - port 2 (#337): condition 6 is UNMET (D088), and its `share` band of 5.16–5.25 was missed
    (5.403);
  - port 3 (#339): V1 and V2 are UNMET (D089).

  Those verdicts stand. Nothing here re-reads their data or substitutes an endpoint into them.
- **It cannot separate each port's contribution.** It compares master with the whole stack.
  The per-port figures stay the failed attempts' diagnostic observations, and nothing here
  apportions the combined difference among them.
- **It cannot qualify production.** It decides nothing about time, adoption, merge or a gate.
  CI's A/B never runs a team (#330 §7).
- **It cannot claim a whole-decoder figure beyond its attribution.** The reported interval
  (§5) holds only if the decoder and harness attribution is right.

## 1. The question (C1, a diagnostic: an information objective, no performance minimum)

**What it asks:**
- On all of `wikidata.bz2`, how many instructions per decoded byte does the stacked team path
  execute in the decoder, against master's, under the frozen partition?
- Is the stack's team present, with call integrity? Each helper is in the build and calls
  the walker in each of its team's blocks, as master's walk-stats instrument (#338) records
  it. This is presence, not useful work (§8, "The claim limit").

**The information objective:**
- ΔDd, with the independent per-arm residual bound (§5);
- the instruction witnesses (§7), each met or not, and the self-cost conformance observations
  (§6), each reported;
- the engagement predicates (§8), each met or not, in both arms.

**Expectation (model, not a bar):**
- **Master's team decoder** is about 36.43 Ir/D (#330 at `ae44497e9`). Master's `src/mtf.rs`
  and `FusedBuild::share` are textually unchanged since then.
- **The stack's** is about 32.28 (#339's counted candidate, `count.txt`).
- **ΔDd** is about −4.1.

No outcome of this diagnostic is a pass or a fail of the ports. It is a validity verdict
(VALID or FAIL-MEASUREMENT) plus the reported figures.

## 2. Pins

| item | pin |
|---|---|
| master | `8f540ef1190689f59a279f081d8405f46669fba0` |
| stack (candidate) | the source tree of `f76f0e3cdeec9260179eacd5e1a111a5a434bec5` (#339's head after the B5 reviewer's test, carrying #335, #337 and master `8f540ef11`). Its non-test sources equal those of `7323196d556e7bca6ee3ffe2cbf80df5a3575973` (#339's counted kernel): since then only `docs/` and `src/mtf/tests.rs` (`#[cfg(test)]`) changed. Before any build, the freeze commit must pass `git diff --quiet f76f0e3cd -- src examples Cargo.toml Cargo.lock` and `git diff --quiet 7323196d5 -- examples Cargo.toml Cargo.lock $(git ls-files src | grep -v '^src/mtf/tests.rs$')` |
| toolchain | rustc 1.99.1 (`58cf8f9c3`), cargo 1.99.1 |
| builds | `cargo build --release --example ab` (counted arms), and `cargo build --release --example ab --features walk-stats` (engagement arms), each in a fresh checkout of its pin at one fixed path per pin, `target/` removed first. Every binary's SHA-256 is recorded before any run |
| input | `wikidata.bz2`, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 431 blocks, D = 209,700,196 decoded bytes |
| block-class map | `wm-classes.txt`, made **before any arm** from the input alone. Each block is `T` if its MTF length is at least `MIN_COOP` (`16 × CHECKPOINT` = 262,144 rows, `src/bwt.rs`), else `S`. It is made by a ≤ 40-line example using master's public API (scanner and `mtf::mtf`), committed with its SHA-256. Expected 233 `T` and 198 `S` (#330); a different count is reported, not corrected |
| parser | `bench/team_gap_count.py` at the freeze commit. `load` refuses raw-and-gzip pairs and repeated threads. `STAGES` and `LIBRARY` are as at `be23a2be0` |
| checker | `bench/walk_stats_check.py` **with the per-T-block roster check of #341** (head `8b62e14d3e8e0bfd4b07ae8503ca89cbcbba5e9c`, merged as `88768ac15924b08ceb039169690c942542791f7e`; the Researcher's counterexample closed), **pinned as git blob `eb7c392d2d766e870de3e14a31b5fe940787dd3f`** (D091 §1). The merge does not put it in the decoder checkouts: master's pin `8f540ef11` carries the older checker, which admits a T block that no helper built, and is not usable here. The run copies the blob out with `git cat-file -p eb7c392d2d766e870de3e14a31b5fe940787dd3f` into its own directory, checks `git hash-object` of the copy equals the pin before the first E arm, and runs only that copy. A mismatch is a failed checker step (§3): no other checker is substituted. No decoder source pin widens |

## 3. Arms and budget

Every arm runs once, in this order, sequentially, with no other build or valgrind job on the
machine.

| arm | binary | command | kind |
|---|---|---|---|
| A1 | master | `valgrind --tool=callgrind --dump-instr=yes --separate-threads=yes … ab child mibz2-pipe3-both 9 wikidata.bz2` | count |
| A2 | stack | the same | count |
| A3 | master | `… ab child mibz2 1` | count (the one-worker reference) |
| A4 | stack | `… ab child mibz2 1` | count (G3: the plain path unchanged) |
| E1 | master, walk-stats | `AB_WALK_STATS=e1.jsonl ab child mibz2-pipe3-both 9` (native) | engagement |
| E2 | stack, walk-stats | `AB_WALK_STATS=e2.jsonl …` (native) | engagement |

**Budget (a forecast, not a bound):** 4 builds of about 40 s, 4 callgrind runs of 40 to 60 s and
2 native runs of seconds, about 1 MB of profiles.

**Deadlines (frozen, enforced):** each step runs in its own process group under `timeout -s
TERM -k 30 <limit>`:

| step | limit |
|---|--:|
| each build | 900 s |
| each callgrind arm (A1–A4) | 900 s |
| each native walk-stats arm (E1, E2) | 300 s |
| `count.py` and the checker, each | 600 s |

- **A step that times out, is killed or exits non-zero:**
  - its partial output is archived as it stands and marked partial;
  - it is never read as a measurement;
  - the diagnostic is **FAIL-MEASUREMENT**, naming the step.
- **The one exception: the checker's exit 1** (revision 3; D091 §2). The pinned checker exits
  0 for ENGAGED, 1 for a structurally VALID record set that is NOT ENGAGED, and 2 for
  INVALID. Exit 1 is therefore a completed verdict, not necessarily a failed process. A
  checker step is adjudicated by §8a, and only by §8a:
  - a complete, parsed verdict with exit 0 or 1 is a completed step, never marked partial
    merely for its exit status;
  - everything else (exit 2, any other exit, a signal, a timeout, an output that does not
    parse completely, or an output that contradicts its exit) is a failed step under the
    rule above.

  No other step has an exception. No exit status is ever discarded (`|| true` or the like),
  and the original exit code, stdout and stderr are archived verbatim either way.
- **Every later step still runs,** and its outputs are archived as observations, within
  these limits:
  - it never overrides B6: on an unresolved undefined-behaviour, invariant or safety
    failure, the affected work stops;
  - a step whose input is missing, partial or stale (an incomplete build, a timed-out
    profile, an earlier attempt's output) does not run on it; it is recorded as unavailable,
    naming the input;
  - only independent steps on complete inputs continue, as observations, one heavy job at a
    time.
- **No step is rerun,** individually or as a whole.

**Archiving:** profiles are written to a directory that only ever holds raw files, gzipped
without `-k` and moved into the archive. Every figure is computed from the archive.

**Resource conditions** (recorded, never gating; D088 §3): before each arm, `pgrep -c rustc`,
`pgrep -c valgrind`, `/proc/loadavg`, `nproc` and `df -h /`; after it, ab's `runq_s` and
`sampler_s`.

## 4. The partition (frozen; D088 §2, D089)

Over each counted arm's profiles (each thread once), with process total **P**:

- **Decoder Dd:** the stages scanner, MTF, build, walk, assembly, RLE1, CRC and coordination,
  by `STAGES` and `LIBRARY`.
  - A non-library function's self Ir goes to its stage.
  - A library routine's inclusive Ir goes to its non-library caller's stage, arc by arc. So
    allocator and runtime arcs stay in Dd's inclusive accounting.
  - Membership is by function, on any thread.
- **Harness H:** stage `harness`, restricted to:
  - functions whose names start with `ab::` or `<ab::`, with their library arcs;
  - **the allowlist**, exactly these four instantiations of std or core over `ab`'s types,
    which #339's archive shows (`count-d089.txt`), and their library arcs:
    1. `core::ptr::drop_glue::<ab::Threads>`;
    2. `core::ptr::drop_glue::<<ab::Threads>::watch::{closure#1}::{closure#0}>`;
    3. `std::sys::backtrace::__rust_begin_short_backtrace::<<ab::Threads>::watch::{closure#1}::{closure#0}, (f64, core::option::Option<alloc::string::String>)>`;
    4. `<std::thread::lifecycle::spawn_unchecked<<ab::Threads>::watch::{closure#1}::{closure#0}, (f64, core::option::Option<alloc::string::String>)>::{closure#1} as core::ops::function::FnOnce<()>>::call_once::{shim:vtable#0}`.

    **Why the allowlist is justified:**
    - each is generic library code monomorphised for, and reached only through,
      `ab::Threads::watch`, the harness's sampler (`examples/ab.rs`);
    - none is instantiated over a `mibz2` type;
    - together they were 1,383 and 1,319 Ir in #339's team arms.
  - Any other harness-stage function is charged to **U**, not H, and listed. This is not an
    invalidity.
- **Unattributed U = P − Dd − H.** It is reported, never subtracted as harness.
- **Complete inventories** (each arm, printed in full, never truncated):
  - every function and library arc in H, with its Ir and threads;
  - every function and arc charged to U, with its Ir and threads.

  Ledger closure alone cannot detect a classifier that mislabels or double-counts, so these
  lists are read as well as summed.
- **Validity (each required):**
  - the integer ledger `Dd + H + U = P`, with `U ≥ 0`;
  - every thread once;
  - objdump exits 0 on every range read.

## 5. The independent per-arm residual bound (D089, D091 §4)

**ΔDd** = Dd(A2) − Dd(A1), per decoded byte, reported with **[ΔDd − U(A1), ΔDd + U(A2)]**.
- This is the **independent per-arm residual bound** (renamed in revision 3; D089 rejected
  the narrower coupled-composition rule, so "coupled-residual" was the wrong name). Each
  arm's U may hold decoder work independently: if 0 ≤ x₁ ≤ U(A1) and 0 ≤ x₂ ≤ U(A2), the
  decoder difference ΔDd + x₂ − x₁ lies in the bound.
- Every term is divided by the same denominator, D = 209,700,196 decoded bytes.
- It holds only if the Dd and H attribution is correct.
- The difference in P is reported beside it, and is not a gate.
- Every stage's difference is reported. No stage is predicted to the third decimal.

## 6. Self-cost conformance: reported, not gated (revision 2)

Revision 1 gated M1–M5 on exact historical self costs. The Researcher's review rejects that
without a shown proof.
- **Textually unchanged Rust does not prove identical machine code.**
- **A self cost is schedule-independent only where paths and work partitions are.**
- **M3's agreement of `share` + `new` to 13 Ir does not establish `share` alone.**

**None of them is a gate now.** C1–C5 stay descriptive in revision 3 (D091 §4). What stays
binding is unchanged: W1–W7 and the call conditions (§7), the attribution and its validity
(§4), output identity (§9), one attempt (§3, §10) and the complete inventories (§4, §11a).
The same figures are reported as **conformance observations**:

| observation | arm | function (self Ir) | historical value | its source |
|---|---|---|---|---|
| C1 | A1 | `<mibz2::coop::FusedBuild>::share` | 1,874,014,575 | #335's old arm (`27ed84df4`) |
| C2 | A1 | `mibz2::mtf::mtf_checkpoints_avx2` | 835,326,277 | #337's and #339's control arms |
| C3 | A2 | `<mibz2::coop::FusedBuild>::share` | 1,132,900,032 | #337's port-2 arm |
| C4 | A2 | `mibz2::mtf::mtf_checkpoints_reg_avx2` | 779,679,144 | #339's candidate arm |
| C5 | A3, A4 | `mibz2::bwt::build`, `mibz2::mtf::mtf_avx2` | 1,082,059,629 and 773,179,796 | #337's and #339's one-worker arms |

- **Each is reported as equal or not, with its difference.**
- **A difference is a failure to reproduce that code-path count,** not evidence for or against
  a port's mechanism or the integrated figure.
- **No tolerance is set,** before or after the data.
- **Allocator and runtime arcs** are excluded from these observations and stay, in full, in
  Dd (§4).
- **What would turn an observation back into a gate:** before freezing, each function's
  listing in the pinned binary must be shown identical, modulo addresses, to the historical
  binary's listing, with both binaries rebuilt bit-identically, and its path and work partition
  shown independent of the schedule. Revision 2 does not make that showing, so it registers
  none of them as gates.

## 7. Instruction witnesses (the binary's SHA-256, address, objdump text, executions summed over threads)

An instruction not found, or an address objdump cannot decode, is **unmet**, never zero.

**In A2, `FusedBuild::share`:**
- **W1:** `pmovmskb` executes exactly 13,106,081 times, and `rol` executes 0 times.
- **W2:** a group of four non-stack `movdqu %xmm,(…)` stores executes 10,740,083 times each,
  and is followed by the source line `ftab[c] = (s + 16)`.
- **W3:** a group of four executes 4,536,329 times each, followed by
  `ftab[c] = (s + len)`.

**In A2, `mtf_checkpoints_reg_avx2`:**
- **W4:** its one-byte non-stack stores sum to exactly 15,148,085.
- **W5:** its non-stack `vmovdqu %ymm` stores come in equal pairs whose first stores sum to
  4,031,045.

**In A1:**
- **W6:** `share` has `rol` ×13,106,081 and no `pmovmskb`.
- **W7:** `mtf_checkpoints_avx2`'s non-stack byte stores sum to 15,148,085, and it has no
  non-stack `vmovdqu %ymm` store.

**Labels:** the source-line labels of W2 and W3 are location evidence, not a control-flow
proof (D088 §4). The binary and the instruction-to-site map are recorded.

**Calls:**
- A2's `mtf_checkpoints_reg_avx2` and A1's `mtf_checkpoints_avx2` are each called 431 times
  by `mibz2::parallel::stage`.
- `mtf_avx2` has 0 Ir in A1 and A2.

## 8. Engagement (E1 and E2: master's walk-stats instrument, #338, with #341's checker)

**The plan:** E1 and E2 run the **same forced team plan as the counted arms**,
`mibz2-pipe3-both 9`, natively, with walk-stats builds. The team is not shrunk to make
balance easier.

**The checker:** `python3 walk_stats_check.py FILE --walker traverse --helpers 2
--classes wm-classes.txt`, run from the pinned blob's copy (§2), never with `--reference`.
- Its **full stdout and stderr** are archived verbatim, with its **original classification**,
  ENGAGED or NOT ENGAGED, and its **original exit code**.
- NOT ENGAGED is never relabelled ENGAGED, and is never hidden: `--reference` (which
  suppresses the outcome) is not used.

**This diagnostic's acceptance rule,** different from the checker's and stated as such. The
Guardian concurred prospectively that (b) may be reported rather than gated (D091 G1). For
each of E1 and E2:
- **Structural validity (required):** the checker's VALID. That includes #341's per-T-block
  roster: in every T block, `built` is exactly helpers 1 and 2, with share records from both;
  every record reconciles with entered, joined and conservation; and the terminal record is
  complete.
- **Presence and call integrity (required):** predicates **(a)**, **(c)** and **(d)** hold.
  - (a): each helper's call presence equals its share presence in every T block. With the
    roster, every helper is present in every T block.
  - (c): a leader called `traverse`.
  - (d): every call is `traverse`.

  Zero-work helper calls are reported, in (a)'s detail.
- **Balance (reported, not required):** predicate **(b)**, each team's largest helper share
  of walked bytes against ⅔.
  - **Why it does not gate:** on 4 CPUs, 11 threads' scheduling can fail it without a broken
    port.
  - **Not retrospective:** no earlier registration's (b) is relaxed. It still gates where it
    was registered.
  - **The limit of that reason** (D091 §1): oversubscription can produce imbalance without a
    broken port, but that does not show an observed imbalance is caused by scheduling, or
    harmless for performance. (b) is dropped as a requirement for the scoped information
    objective, not because balance is unimportant.
  - **What is reported:** each team's actual helper byte totals and ratios, and every zero
    case. A team whose helpers walked zero bytes in total has an **undefined** ratio, reported
    as such with (b) failed; no balance fraction is made up for it.

**The claim limit** (D091 §3; the Researcher's second fixture). A complete record set in which
both helpers are present in every T block, but walk **zero bytes**, is also structurally
VALID with (a), (c) and (d) true (and (b) false, exit 1). This rule accepts it. So E1 and E2
can show only **structural validity plus presence and call integrity**:
- **reported:** presence, call integrity and the zero-work helper call counts;
- **not claimed:** that any helper did useful walk work, that the team ran balanced, or any
  parallel speedup.

No positive-byte minimum is added: it would answer a different question.

**What E cannot show:**
- **E1 and E2 are different binaries** (the walk-stats feature) **and a different schedule**
  (native, 11 threads on 4 CPUs) from A1 and A2. No observation from them establishes
  engagement in the counted binaries.
- **They are evidence about the same source and plan only.**

**On failure:** INVALID, (a), (c) or (d) failing, or a checker step that §8a does not accept,
makes the diagnostic **FAIL-MEASUREMENT**. The counts are still reported as observations.

## 8a. Adjudicating a checker step (revision 3; D091 §2)

The checker's exit status alone never decides. **Exit 1 alone never proves a balance-only
failure:** an uncaught Python exception also exits 1. Each of E1's and E2's checker steps is
read in this order, and the first rule that applies decides it.

1. **Execution failure, rejected:**
   - the step timed out (§3) or was killed by a signal;
   - its exit code is anything but 0 or 1. Exit 2 is the checker's INVALID: a completed
     verdict, but one that fails structural validity, so it is rejected too.
2. **A complete, parsed verdict, or rejected.** Rejected unless all of these hold:
   - stderr is empty;
   - stdout is exactly: `VALID`; then one line each for (a), (b), (c) and (d), in that order,
     each `  (x) holds: …` or `  (x) fails: …`; then `ENGAGED` or `NOT ENGAGED`; and nothing
     else, with no `reference` line;
   - the outcome line agrees with the predicates: ENGAGED exactly when all four hold;
   - the exit code agrees with the outcome: 0 for ENGAGED, 1 for NOT ENGAGED.

   A truncated or partial output, a traceback, a crash with a non-zero exit and no verdict, an
   INVALID listing, or any other shape is rejected under this rule.
3. **The engagement component, checked explicitly.** Only for a complete, parsed verdict:
   accepted exactly when it is structurally VALID (its first line) and (a), (c) and (d) each
   read `holds`. (b)'s line, the outcome and the exit code are reported as the checker gave
   them, whatever they are.

So a complete verdict with exit 1 is accepted only when its own predicate lines explain it:
(b) alone fails. Exit 1 with (a), (c) or (d) failing is a completed verdict that is
rejected. The adjudicator's decision for each step (the rule that decided it, and why) is
archived beside the checker's raw output.

## 9. Output identity

All six arms write 209,700,196 bytes with hash `a8830caee5c511a6`. This is required.

## 10. Verdict rule

**VALID** only if all of these hold:
- §4's validity;
- every step inside its deadline (§3), with complete artefacts; each checker step is judged
  by §8a, not by its exit status alone;
- W1 to W7 and the call conditions;
- E1 and E2 structurally VALID with (a), (c) and (d), each accepted by §8a. (b), the
  checker's own ENGAGED/NOT ENGAGED and its exit code are reported, not required;
- §9.

Then the result reports ΔDd with §5's interval, every stage, P, U and the engagement
predicates. Otherwise it is **FAIL-MEASUREMENT**: each unmet condition is named, and the
figures are reported as observations.

- **One attempt. No retry**, and no re-run of a single arm.
- **Every artefact is archived:** profiles, JSONs, records, binaries' hashes, resources and
  the class map.

## 11. G3 (dispatch paths of the stack against master)

| path | changes? | evidence here |
|---|---|---|
| pipelined team: helpers' `FusedBuild::share` (ports 1 and 2), the front's `mtf_checkpoints` (port 3) | **yes** | A1/A2, W1–W7, E1/E2; C1–C4 reported |
| ordinary teams (`build_fused`, `decode_block` with checkpoints) | **yes**, the same functions | not counted; the unit tests only |
| plain worker `bwt::build`, `mtf::mtf` / `mtf_avx2` | **no** | A3/A4; C5 reported |
| team solo paths (`parallel::solo`) | **no** | read |

## 11a. The artefact inventory (complete; all archived)

- **Builds:**
  - each binary's SHA-256;
  - `rustc -vV`, `cargo -vV`;
  - the build's environment: `RUSTFLAGS` and `CARGO_*`, recorded empty or as set;
  - `Cargo.toml`'s profile at the pin;
  - the `git diff --quiet` checks' outputs.
- **Per callgrind arm:**
  - every per-thread profile, `PREFIX.PID-NN.gz` (12 for team arms, 1 for one-worker
    arms), with its SHA-256, and the empty `PREFIX.PID` file listed;
  - the child's JSON, valgrind's log and the resources record.
- **Per binary:** objdump listings of `FusedBuild::share`, `mtf_checkpoints_avx2` or
  `mtf_checkpoints_reg_avx2`, and `mtf_avx2`, as the site maps of W1–W7 (each address
  executed and its text).
- **Per E arm:** the JSONL records, the checker's full stdout and stderr and its original
  exit code, §8a's decision, the child's JSON, and the walk-stats binary's SHA-256.
- **The checker:** the copy that ran, and its `git hash-object` (the pin
  `eb7c392d2d766e870de3e14a31b5fe940787dd3f`).
- **The class map:** `wm-classes.txt`, its tool, and the SHA-256 of both.
- **The count:** `count.py`, its full output (the complete H and U inventories), and the
  verdict with every condition.

## 12. Review

- **The Researcher** reviews this draft, the arms, witnesses, allowlist, acceptance rule and verdict rule above
  all, before any freeze.
- **The Guardian** may HOLD.
- **B5** for ports 2 and 3 is separate and outstanding. This diagnostic does not discharge
  it.

## 12a. The adjudication dry run (untimed, synthetic records only)

Before the freeze, §8a's adjudicator runs on synthetic walk-stats records against the pinned
checker blob, with no decoder build or run and no timing. Each case's checker output, exit
code and §8a decision are archived. The cases (the D091 and Researcher fixtures are one T
block, both helpers in the built set and share records, entered mask 7, the leader 100 walked
bytes, exact conservation and terminal count):

| case | checker | §8a |
|---|---|---|
| balanced, helpers 100/100 | VALID, all hold, ENGAGED, exit 0 | accepted |
| **balance-only, helpers 1/99** | VALID, (a)/(c)/(d) hold, (b) fails, NOT ENGAGED, **exit 1** | **accepted** for engagement; NOT ENGAGED and exit 1 reported |
| zero-work, helpers 0/0 | VALID, (a)/(c)/(d) hold, (b) fails (ratio n/a), NOT ENGAGED, exit 1 | accepted, with the claim limit (§8): presence only, 2 zero-work calls |
| a required predicate fails ((a), (c) or (d)) | VALID, NOT ENGAGED, exit 1 | rejected (rule 3) |
| INVALID (for example, the terminal record missing) | INVALID, exit 2 | rejected (rule 1) |
| a timeout, or a kill by signal | anything | rejected (rule 1) |
| a truncated or partial report (stdout cut after the predicates, no outcome line) | exit 0 or 1 | rejected (rule 2) |
| a crash: non-zero exit, a traceback, no verdict | exit 1 | rejected (rule 2) |
| a verdict whose exit disagrees with its outcome | ENGAGED with exit 1, or NOT ENGAGED with exit 0 | rejected (rule 2) |

The checker's side of five rows was reproduced in revision 3 against the pinned blob, on
synthetic records only: balanced, exit 0; 1/99, exit 1; 0/0, exit 1 with "2 zero-work helper
calls"; one helper calling `traverse_staged` with 50/50 bytes, VALID with (d) failing, exit 1;
and the 1/99 file without its terminal record, INVALID, exit 2. D091 and the Researcher
report the same for their fixtures. The pinned blob's 20 committed tests pass. The
adjudicator is now written and this dry run archived (§14a).

## 13. Revision 2 (the Researcher's REVIEW 6763089509 item 3)

| point | change |
|---|---|
| the checker admits a T block no helper built (`probes.py`) | fixed on master's checker in #341 (per-T-block roster, negative tests); this draft requires that checker (§2, §8) |
| ⅔ balance on 4 CPUs | (b) reported, not required. Structural VALID plus (a), (c), (d) required. The checker's own output and classification are kept verbatim. The Guardian's concurrence is needed before the freeze (§8) |
| exact M gates without a codegen or path proof; M3 rests on share + new | M1–M5 dropped as gates. Reported as conformance observations C1–C5, with no tolerance and stated meaning, and the condition for restoring a gate stated (§6) |
| budget a forecast | per-step deadlines, partial-output treatment, no rerun (§3) |
| inventories | complete H and U inventories (§4), and a complete artefact inventory with toolchain and site maps (§11a) |

## 14. Revision 3 (Guardian D091, 6763469482; the Researcher's REVISE, 6763607539 item 2)

| review point | change |
|---|---|
| D091 §2, Researcher item 2: §3 rejects every non-zero exit, but §8 and §10 accept a balance-only failure, which the pinned checker reports as a complete NOT ENGAGED verdict with **exit 1** (fixture: helpers 1/99) | §3's one exception for the checker's exit 1, decided only by the new §8a: a complete, parsed verdict is told apart from a timeout, signal, crash, INVALID, partial or contradictory output; the original exit, stdout and stderr are kept verbatim; the engagement component is accepted only after checking the verdict is VALID and that (a), (c) and (d) read `holds`. No `|| true`, no `--reference`, no relabelling (§3, §8, §8a, §10, §11a) |
| D091 §2, Researcher item 2: add the balance-only witness (and balanced, required-predicate failure, invalid and truncated cases) to the untimed adjudication check | §12a lists the dry run's cases, with the 1/99 fixture accepted for engagement, a timeout or partial output rejected, and a crash with a non-zero exit and no verdict rejected; the checker's side reproduced on synthetic records |
| D091 §1, Researcher item 2: pin the checker blob; its merge does not install it in the older decoder checkout | §2 pins blob `eb7c392d2d766e870de3e14a31b5fe940787dd3f`, copied out and hash-checked before the E arms; no decoder pin widens |
| D091 §3, Researcher item 2: a complete 0/0 fixture also satisfies VALID + (a)/(c)/(d) | §8's claim limit: presence, call integrity and zero-work counts are reported; no useful helper work, balance or speedup is claimed; undefined ratios reported as undefined; no positive-byte gate. §1's question narrowed to presence |
| D091 §1: the limit of the oversubscription reason for not gating (b) | stated in §8; actual helper byte totals, ratios and zero cases reported |
| D091 §4: "coupled-residual" is the wrong name | §1 and §5 now say **independent per-arm residual bound**, with its derivation and one denominator |
| D091 §4, Researcher item 2: "every later step runs" | §3: never overrides B6, never consumes a missing, partial or stale input (an incomplete build or stale output); only independent steps on complete inputs continue, as observations |
| D091 §4: C1–C5 descriptive; the rest binding | §6 states C1–C5 stay descriptive, and W1–W7, attribution, output identity, one attempt and the complete inventories stay binding |
| D091 G1: concurrence on (b) reported, not gated | §8 cites it in place of "needs the Guardian's concurrence" |

**No new gate** is added, and nothing is built, timed or run on the decoder.

**Unresolved:**
- **The adjudicator** is now written, tested and dry-run (§14a). The rest of the harness
  (builds, A arms, `count.py`, the recipe) is still unwritten, so §3's deadlines are enforced
  in code only for the checker step.
- **The freeze still needs** the Researcher's PROCEED and D089's other pre-data conditions
  (D091 §2 does not assert they are met), and B5 for ports 2 and 3 stays separate.

## 14a. The §8a adjudicator and the §12a dry run (implementation; the Researcher's PROCEED 6764231210 item 2, Guardian D094 §3)

Commit `e9f8833d97fc6590e52c9ebef42a239869ba9f88`. Untimed; no decoder built or run; nothing
frozen.

| file | what it is |
|---|---|
| `bench/team_ports_adjudicate.py` | §8a. `checker --out DIR` copies blob `eb7c392d2d766e870de3e14a31b5fe940787dd3f` out with `git cat-file -p` and checks `git hash-object` of the copy; `run --dir DIR --step E1\|E2 --records … --classes …` re-checks the copy's hash, runs §8's command exactly (`python3 walk_stats_check.py FILE --walker traverse --helpers 2 --classes …`, never `--reference`) under `timeout -s TERM -k 30 600` in its own process group, archives `checker-exit.txt`, `checker-stdout.txt` and `checker-stderr.txt` verbatim, and writes `decision.json`; `judge` adjudicates an archived outcome |
| `bench/test_team_ports_adjudicate.py` | the tests: 11 test methods, with subtests over every rule and every dry-run case. In CI: `test.yml`, job `submit`, which first fetches #341's merge commit `88768ac15` (the shallow checkout lacks the blob) |
| `bench/team_ports_dry_run.py` | the §12a dry run: builds every case, runs it and writes `summary.json` |
| `docs/journal/2026-11-10-team-ports-combined/dry-run/` | the archive: `checker.json` (the copy's `git hash-object`, `eb7c392d…`, and SHA-256 `27eaa104…`), the copy that ran, `summary.json`, and per case `cases/<case>/` with its records, the raw exit, stdout and stderr, and `decision.json` |

**The decision record** (`decision.json`, schema `team-ports-combined/checker-step/1`): the
command, the status (exit, timed out, signal, elapsed), the stdout and stderr sizes and SHA-256,
the checker's own outcome and exit (never relabelled), the decision (accepted or rejected), the
rule that decided it (1, 2 or 3) and its reasons, the parsed verdict, what §8 reports ((b)'s
line, undefined-ratio teams, zero-work helper calls) and, when accepted, the claim limit. The
checker's PYTHON* environment variables and `python3`'s path are recorded with it.

**Readings this implementation fixes, each fail-closed:**
- stdout must be UTF-8, end with a newline, and hold exactly six lines with no carriage return
  or blank line; a predicate line must read `  (x) holds: ` or `  (x) fails: ` with a non-empty
  detail, its letter in order;
- the deadline: `timeout`'s 124 (TERM) or 137 (KILL after 30 s), with the elapsed time at
  least the deadline, is recorded as a timeout; any other non-0/1 code, and a negative one (a
  signal re-raised by `timeout`), is rejected under rule 1 anyway;
- §2's identity check comes before §8a: a copy whose `git hash-object` is not the pin is not
  run, no checker is substituted, and the step is rejected as `rule 0` (`run` exits 2).

**Tests:** `python3 bench/test_team_ports_adjudicate.py`: 11 tests, all pass (Python 3.13.16).
They cover rule 1 on 2, 3, 124, 137, −9, −15 and a timeout or signal with good output; rule 2 on
21 shapes (stderr, truncation, blank line, CRLF, non-UTF-8, INVALID, wrong letter, empty detail,
duplicate, missing line, reference line, outcome against predicates, exit against outcome); rule
3 on each of (a), (c) and (d); the exact command; the identity check (a missing blob, a tampered
copy not run); and the whole dry run, expected against actual for every case.

**The dry run** (`summary.json`, `all_match: true`): 35 cases. *Real* cases run the pinned
checker on a synthetic one-T-block ledger through `run`'s own path. The timeout is a FIFO no one
writes, with a 2 s deadline in place of 600 s; the signals are sent to the blocked checker
process (the dry run's fault injection, never the production path); the crash is the checker's
own `UnicodeDecodeError` traceback on non-UTF-8 records, exit 1. *Simulated* cases are outcomes
a correct checker run does not produce, built from the real cases' archived stdout. The
Researcher's controls (report at `8d3d9004`, §2) and the Guardian's eight (D094 §3,
`review_probes.py`) are included as given.

| case | §12a row | kind | checker exit | checker outcome | expected | actual |
|---|---|---|--:|---|---|---|
| `balanced` | balanced, helpers 100/100 | real | 0 | ENGAGED | accepted, rule 3 | accepted, rule 3 |
| `balance-only-1-99` | balance-only, helpers 1/99 | real | 1 | NOT ENGAGED | accepted, rule 3 | accepted, rule 3 |
| `zero-work-0-0` | zero-work, helpers 0/0 | real | 1 | NOT ENGAGED | accepted, rule 3 | accepted, rule 3 |
| `a-fails` | a required predicate fails | real | 1 | NOT ENGAGED | rejected, rule 3 | rejected, rule 3 |
| `c-fails` | a required predicate fails | real | 1 | NOT ENGAGED | rejected, rule 3 | rejected, rule 3 |
| `d-fails` | a required predicate fails | real | 1 | NOT ENGAGED | rejected, rule 3 | rejected, rule 3 |
| `invalid-no-terminal` | INVALID | real | 2 | — | rejected, rule 1 | rejected, rule 1 |
| `timeout` | a timeout | real | 124 (timed out) | — | rejected, rule 1 | rejected, rule 1 |
| `signal-kill` | a kill by signal | real | -9 (signal 9) | — | rejected, rule 1 | rejected, rule 1 |
| `signal-term` | a kill by signal | real | -15 (signal 15) | — | rejected, rule 1 | rejected, rule 1 |
| `crash-traceback-exit1` | a crash: non-zero exit, a traceback, no verdict | real | 1 | — | rejected, rule 2 | rejected, rule 2 |
| `truncated-no-outcome-exit0` | a truncated report | simulated | 0 | — | rejected, rule 2 | rejected, rule 2 |
| `truncated-no-outcome-exit1` | a truncated report | simulated | 1 | — | rejected, rule 2 | rejected, rule 2 |
| `truncated-mid-line` | a truncated report | simulated | 1 | — | rejected, rule 2 | rejected, rule 2 |
| `truncated-empty` | a truncated report | simulated | 0 | — | rejected, rule 2 | rejected, rule 2 |
| `crash-no-output-exit1` | a crash: non-zero exit, a traceback, no verdict | simulated | 1 | — | rejected, rule 2 | rejected, rule 2 |
| `crash-partial-verdict-exit1` | a crash: non-zero exit, a traceback, no verdict | simulated | 1 | — | rejected, rule 2 | rejected, rule 2 |
| `crash-exit3` | a crash: non-zero exit, a traceback, no verdict | simulated | 3 | — | rejected, rule 1 | rejected, rule 1 |
| `engaged-exit1` | a verdict whose exit disagrees with its outcome | simulated | 1 | — | rejected, rule 2 | rejected, rule 2 |
| `not-engaged-exit0` | a verdict whose exit disagrees with its outcome | simulated | 0 | — | rejected, rule 2 | rejected, rule 2 |
| `outcome-contradicts-predicates` | a verdict whose exit disagrees with its outcome | simulated | 0 | — | rejected, rule 2 | rejected, rule 2 |
| `stderr-warning` | non-empty stderr | simulated | 0 | — | rejected, rule 2 | rejected, rule 2 |
| `duplicate-line` | duplicate output | simulated | 1 | — | rejected, rule 2 | rejected, rule 2 |
| `reordered-lines` | reordered output | simulated | 1 | — | rejected, rule 2 | rejected, rule 2 |
| `reference-line` | extra output | simulated | 0 | — | rejected, rule 2 | rejected, rule 2 |
| `invalid-listing-exit1` | INVALID | simulated | 1 | — | rejected, rule 2 | rejected, rule 2 |
| `invalid-listing-exit0` | INVALID | simulated | 0 | — | rejected, rule 2 | rejected, rule 2 |
| `D094-truncated` | D094 control | simulated | 0 | — | rejected, rule 2 | rejected, rule 2 |
| `D094-crash` | D094 control | simulated | 1 | — | rejected, rule 2 | rejected, rule 2 |
| `D094-timeout` | D094 control | simulated | 0 (timed out) | — | rejected, rule 1 | rejected, rule 1 |
| `D094-signal` | D094 control | simulated | -9 (signal 9) | — | rejected, rule 1 | rejected, rule 1 |
| `D094-exit-disagrees-engaged` | D094 control | simulated | 1 | — | rejected, rule 2 | rejected, rule 2 |
| `D094-exit-disagrees-not-engaged` | D094 control | simulated | 0 | — | rejected, rule 2 | rejected, rule 2 |
| `D094-stderr-warning` | D094 control | simulated | 0 | — | rejected, rule 2 | rejected, rule 2 |
| `D094-extra-stdout` | D094 control | simulated | 0 | — | rejected, rule 2 | rejected, rule 2 |

Every actual decision equals its expected one. The 0/0 case is accepted with 2 zero-work
helper calls and an undefined ratio reported, under the claim limit (§8). Rejections by rule 3
keep the checker's own NOT ENGAGED and exit 1 in the record; nothing is relabelled.

**Still before the freeze:** the rest of the harness (§14, unresolved), the Researcher's
review of this implementation, D089's other pre-data conditions, and B5 for ports 2 and 3.
