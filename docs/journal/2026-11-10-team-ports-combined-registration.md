# 2026-11-10: master against the three stacked team ports, one combined diagnostic (DRAFT registration)

**Status: DRAFT, revision 2. Not frozen, nothing built or run for it.** Revision 2 answers the
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
- Does the stack's team actually engage? Each helper shares the build and the walk in each
  of its team's blocks, as master's walk-stats instrument (#338) records it.

**The information objective:**
- ΔDd, with D089's coupled-residual interval;
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
| checker | `bench/walk_stats_check.py` **with the per-T-block roster check of #341** (head `8b62e14d3e8e0bfd4b07ae8503ca89cbcbba5e9c`; the Researcher's counterexample closed). The freeze needs #341 merged or its blob pinned by SHA-1; master's `8f540ef11` checker admits a T block that no helper built, so it is not usable here |

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
- **Every later step still runs,** and its outputs are archived as observations.
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

## 5. The reported interval (D089)

**ΔDd** = Dd(A2) − Dd(A1), per decoded byte, reported with **[ΔDd − U(A1), ΔDd + U(A2)]**.
- This is D089's coupled-residual bound. Each arm's U may hold decoder work independently.
- It holds only if the Dd and H attribution is correct.
- The difference in P is reported beside it, and is not a gate.
- Every stage's difference is reported. No stage is predicted to the third decimal.

## 6. Self-cost conformance: reported, not gated (revision 2)

Revision 1 gated M1–M5 on exact historical self costs. The Researcher's review rejects that
without a shown proof.
- **Textually unchanged Rust does not prove identical machine code.**
- **A self cost is schedule-independent only where paths and work partitions are.**
- **M3's agreement of `share` + `new` to 13 Ir does not establish `share` alone.**

**None of them is a gate now.** The same figures are reported as **conformance
observations**:

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

**The checker:** `python3 bench/walk_stats_check.py FILE --walker traverse --helpers 2
--classes wm-classes.txt` (#341's version).
- Its **full output** is archived verbatim, with its **original classification**, ENGAGED
  or NOT ENGAGED, and exit code.
- NOT ENGAGED is never relabelled ENGAGED.

**This diagnostic's acceptance rule,** different from the checker's and stated as such, needs
the Guardian's concurrence before the freeze. For each of E1 and E2:
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

**What E cannot show:**
- **E1 and E2 are different binaries** (the walk-stats feature) **and a different schedule**
  (native, 11 threads on 4 CPUs) from A1 and A2. No observation from them establishes
  engagement in the counted binaries.
- **They are evidence about the same source and plan only.**

**On failure:** INVALID, or (a), (c) or (d) failing, makes the diagnostic
**FAIL-MEASUREMENT**. The counts are still reported as observations.

## 9. Output identity

All six arms write 209,700,196 bytes with hash `a8830caee5c511a6`. This is required.

## 10. Verdict rule

**VALID** only if all of these hold:
- §4's validity;
- every step inside its deadline (§3), with complete artefacts;
- W1 to W7 and the call conditions;
- E1 and E2 structurally VALID with (a), (c) and (d), as in §8. (b) and the checker's own
  ENGAGED/NOT ENGAGED are reported, not required;
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
- **Per E arm:** the JSONL records, the checker's full output and exit code, the child's
  JSON, and the walk-stats binary's SHA-256.
- **The class map:** `wm-classes.txt`, its tool, and the SHA-256 of both.
- **The count:** `count.py`, its full output (the complete H and U inventories), and the
  verdict with every condition.

## 12. Review

- **The Researcher** reviews this draft, the arms, witnesses, allowlist, acceptance rule and verdict rule above
  all, before any freeze.
- **The Guardian** may HOLD.
- **B5** for ports 2 and 3 is separate and outstanding. This diagnostic does not discharge
  it.

## 13. Revision 2 (the Researcher's REVIEW 6763089509 item 3)

| point | change |
|---|---|
| the checker admits a T block no helper built (`probes.py`) | fixed on master's checker in #341 (per-T-block roster, negative tests); this draft requires that checker (§2, §8) |
| ⅔ balance on 4 CPUs | (b) reported, not required. Structural VALID plus (a), (c), (d) required. The checker's own output and classification are kept verbatim. The Guardian's concurrence is needed before the freeze (§8) |
| exact M gates without a codegen or path proof; M3 rests on share + new | M1–M5 dropped as gates. Reported as conformance observations C1–C5, with no tolerance and stated meaning, and the condition for restoring a gate stated (§6) |
| budget a forecast | per-step deadlines, partial-output treatment, no rerun (§3) |
| inventories | complete H and U inventories (§4), and a complete artefact inventory with toolchain and site maps (§11a) |
