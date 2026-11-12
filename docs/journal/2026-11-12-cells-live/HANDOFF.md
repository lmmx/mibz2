# Matched cells live-path diagnostic: preparation handoff

Assignment [363/6819644553](https://github.com/lmmx/mibz2/issues/363#issuecomment-6819644553).
Started2026-11-12 09:43UTC, bounded to11:43UTC. Implementation and untimed local
preparation delivered; **no execution/readiness or production qualification**.
[Plan](PLAN.md) frozen before observations at
`3b43af956d553905ea744f6cb6e1c785a0f8297a`, published in
[145/6820313763](https://github.com/lmmx/mibz2/pull/145#issuecomment-6820313763).
No candidate timing, workstation/i9 job, corpus census or real-block probe ran.

## What changed

One isolated branch, `diagnostic/cells-live-path`, descending from the unchanged
#367 head `a08277e1c36e276cfea7b3116d99ea5fd93f0e84`. The new child crate
[bench/cells-gap/live](../../../bench/cells-gap/live/README.md) preserves367's old
crate, corpus reconstruction/oracle tools and recorded evidence. Its workflow now
runs only the new untimed preparation; it does not automatically replay the old
all-corpus census. The only Just addition is a separate recipe. Production root
src/examples/tests/Cargo files, existing benchmark/Miri workflows and workstation
queue remain byte-identical to367. No365/366/368 branch or other role state changed.

Generated diagnostic source is pinned to corrected365
`d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2`, not367's historical57f51b dependency.
All53 original source files are hashed; original cells.rs SHA256 is
`42e377d8a980c94d95a4b162b0f7ba6466ff6b3844cbc8b10fce933fd25692b2`.
The [source correspondence](evidence/source-correspondence.json) and
[additive instrumentation patch](evidence/instrumentation.patch) expose every hook.
The frozen reader adapter and ab output hash are reused exactly. Four separate
release builds select baseline/Cells and compile hooks out/in; no runtime route
selection or parallel/team experiment was added.

The exact design remains Wikidata/pattern/onebyte, one thread/allowed pinned CPU,
ten balanced paired rounds,0.25s settle,120 measured decodes plus9 untimed/oracle
checks in a later authorised run. Untimed fixtures do not consume or masquerade as
that future corpus budget. Prospective CPU must be chosen explicitly in the
Coordinator-approved recipe and recorded before results; source/compiler/input/
executable identities are checked. No three-round alternative is reachable.

The driver reconciles exclusive phases against inclusive reader and process wall,
keeps inclusive inversion separate, clusters ratios by complete decode and rejects
quantitative attribution when the off/on conclusion changes or phases fail. Real
bytes/CRC/oracle checks and all refused/abandoned induction work remain charged.
See the README for measured fields and explicitly missing counters. Memory counts
are capacities/requests/addressed copies, not physical traffic or a complete heap
inventory. No numerical performance gap is claimed closed.

## Actual validation

- Five native harness tests: original reader parity/error cases, actual admitted/
  density/floor/budget reach, CRCs, explicit57-cell long-factor/short-cycle witness,
  and logger scope/error cleanup. All untimed clock counts are zero.
- Seven corrected kernel tests pass, including exhaustive small columns, exact-fit
  reuse, admission/invalid inputs, bounds/fallback and both F01 writer tests.
- Ten Python harness tests pass with **real Just1.38.0, no skips**. Fixed129-job
  wiring is simulated with fake values:126 decoder children+3 oracles,120 settles,
  zero real candidate decodes/clocks. Failure, incomplete/duplicate records,
  malformed JSON, CRC/phase errors, changed off/on conclusion and exit propagation
  are checked. A `wait4` test uses fake clock/resource values, not measurements.
- Four Rust1.99.1 release builds. Real CLI dry run:16 tiny valid fixture decodes,
  full byte/SHA parity, plus12 expected failure children (CRC, truncation, missing
  input); identical errors, zero clocks, zero performance measurements.
- **Miri:2/2 unchanged F01 long-writer tests pass** with diagnostic source hooks;
  **1/1 active untimed logger contract passes**. These are scoped checks, not a
  rerun of365's full39-test review, fuzz/mutation programme or independent B5.
- Scoped Clippy for new crate, Ruff check/format, owned Rust formatting, TOML,
  Python syntax, shell syntax, actual Just dry-run and diff whitespace pass.
- All53 original-source hashes, additive-only patch, corrected kernel identity,
  protected production paths and final binary/harness identities checked.

Logs and source/binary/compiler digests are in [evidence](evidence/). SHA256.json
covers archived evidence. CI is reported separately at the exact PR head in the
completion REVIEW/state snapshot; pending jobs are never treated as green.

Failures retained: initial generated manifest omitted declared examples/bench
files and failed Cargo parsing; generation now copies all frozen examples/benches.
Initial Miri setup could not create a sysroot under the read-only default cache;
using the existing writable XDG cache produced the actual passes. Initial full
Clippy hit unchanged parallel.rs `let_and_return`; new-crate `--no-deps` scope
then found unchanged helper `manual_repeat_n`. The latter is allowed only on the
imported frozen helper module. New code passes `-D warnings`; no production source
was reformatted to silence inherited findings. Initial new TOML ordering was fixed. A clean-build metadata pass found that this
Python embeds `_bz2` as a built-in with no file path; the recorder now records
module origin and interpreter SHA, with a separate extension SHA only when present.
The failed metadata log is retained; it was not a decoder failure.
Historical367 legacy pre-commit failure remains historical, not relabelled; its PR
also records default onebyte4t+1.0%[+0.0,+1.9] in run42925377667. Its three old Python
files still fail current Ruff formatting; they are unchanged and never replayed here.

## Safety argument and independent review boundary

The patch contains **only insertions** plus a separately compiled safe module;
its generator asserts no original line is deleted. No unsafe operation, pointer
arithmetic, allocation bound, admission predicate, copy length, fallback call or
F01 test is replaced. Hooks copy scalar values and query Vec length/capacity; they
do not expose references/pointers into kernel buffers. Independent allocations for
TLS records cannot move a live Rust Vec allocation. No record is held borrowed
across a decoder call. RefCell borrows are released inside each hook; guards hold
only stack depth and an active flag. Ordinary Result::Err returns close scopes in
LIFO order, and an explicit front stop is idempotent. Existing tests exercise
Result::Err and scope destruction, not an injected Rust panic; panic-unwind
recovery remains untested. No new panic coverage or B5 certification is claimed. Logging may add cost or allocation failure,
which is part of diagnostic overhead, but does not change a successful decoder's
memory-access obligations. Off hooks are removed by feature cfg.

Existing short-copy disjointness, long-copy overlap semantics, marked-spine recopy,
position initialisation, repetition/slack, canaries and budget/fallback invariants
are unchanged. The original long-factor reach/regression tests and independent
Researcher F01 acceptance remain pinned to365. Miri checks here are author evidence;
Codex does not certify independent B5 or close any reviewer finding. Independent
review must inspect the generated patch, phase nesting, original reader/hash
correspondence, scheduling, refusal accounting, missing-counter limits, off/on
interpretation and failure preservation before Coordinator admission.

## Reconciliation and remaining decisions

The current live programme can locate bounded costs and common-work dilution. It
cannot settle production-versus-research warm transport directly. PLAN explicitly
proposes, but does not implement/admit, one extraction plus20 measured inversion
file-passes and20 untimed warm passes with matched records, host/compiler/affinity/
reuse and no minima. Research source remains
`c79ff7d384072e5e95eda7e513227f2aa06ff3d5`; layouts/admission still differ. Coordinator
must approve that separate finite budget/adapter before execution. Warm port loses
-> price writer/port; warm wins but live inversion does not -> conditioning;
live inversion wins but total does not -> remaining stages/reader. Unmeasured
branches stay open; no additional programme is silently included.

**Next required action:** Coordinator reconcile this preparation with Optimiser
6782313693,367 and372; arrange one independent harness review and decide separate
integration/readiness. PROTOCOL9 checklist remains open: frozen exact final recipe
check, exact-head green Actions+legacy, deliberate integration disposition, ready
tag/Run signal and master queue entry. No queue/tag/signal was created. Input
identities are frozen, but full three-corpus oracle/prechecks run only during the
future authorised programme, not to enlarge preparation evidence.

Preserve365 DEFER,371 REVISE METADATA, stopped treap/deferred summaries and372's
no-backend-screen-clearance. #366's separate diagnostic/integration is untouched.
Direct coord.py audit/verify remains blocked by proxy tunnel403; route145 works,
connector readback confirms publication. Cursor completion records delivery of
this preparation only. GitHub comments cannot restart this Codex session; a manual
launch remains required, and no event-triggered execution is assumed.

## CI correction after initial publication

At d2db35fca04b17ed766b29fc1e7932d49f7f70cc, untimed run43199851877 /
job131120099184 passed source preparation, all10 Python/Just tests, five native
harness tests and two native F01 tests. Step7 failed before any release build:
`FileNotFoundError: target/live-binaries` because a fresh checkout had no root
`target/`. **Class A, harness setup bug.** `build.py` now creates missing parent
directories, still refusing an existing destination. A regression checks both
properties; the Python suite is now11/11 with real Just and no skips. The experiment,
Rust source/binaries, admission/copies, corpus budget and performance rules did not
change. Exact failed excerpt and correction test log are retained. New-head CI
must confirm the real fresh-checkout path; no manual rerun was requested.

Legacy pre-commit at the initial head is also failure. Its external results page
is blocked by the local proxy (403) and unavailable through the web connector;
its cause is not asserted from the badge. Remaining check dispositions are in the
completion REVIEW/state snapshot. This cannot support an all-green/readiness claim.


## Targeted R1 correction after independent review

Dispatch [363/6821674655](https://github.com/lmmx/mibz2/issues/363#issuecomment-6821674655),
answering [Researcher6821626797](https://github.com/lmmx/mibz2/pull/145#issuecomment-6821626797)
at reviewed head7b53ded9eb139fdd69b4c29565e7373ac95e90d4. One bounded correction;
Coordinator assesses the result without a second full review round.

`checked_child` now flushes command/job/status and already-collected process
wall/user/system/total CPU/RSS to its reserved `.resources` sidecar before exit,
JSON or decoder-error rejection. Spawn failure records explicit null child/resource
fields. Raw stdout/stderr and FAILED survive. Resource collection is unchanged;
sidecar I/O occurs after the second clock read. Successful return tuples and the
paired estimator are unchanged. Phase reconciliation now precedes successful-row
append; failure rows cannot enter paired estimates. Sidecar validation describes
child output validation only; later checks may still fail the attempt.

Evidence limits corrected: the existing Rust error tests are Result::Err tests,
not injected panic recovery. `pos_initialized_bytes` is logical initialized extent,
not repeated resize/fill stores or physical traffic. Bundles omit files>=50MiB,
including full Wikidata precheck byte files; length/SHA and logs remain bundled.
Matching off/on conclusion categories do not establish effect-size equivalence
or causal off-build savings (off0.9/on0.45 can both be faster). Instrumented-path
interpretation, source/input identities, prospective PLAN,120+9 budget and the
unadmitted warm20+20 proposal are unchanged. No hooks or Rust source changed.

Focused regression and source-protection evidence is appended under [r1](r1/);
all97 prior archived digests and historical367 files remain immutable. Exact
changed-head Actions and legacy dispositions travel in the completion REVIEW and
own state report. Inherited Occupancy accounting, prior legacy Class D and macOS
ARM cancellation/Class D remain unwaived until reconciled on the changed head.
No workstation execution, readiness signal or production qualification follows.
