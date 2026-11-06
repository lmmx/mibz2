# #234 re-review: baseline path repaired; freeze still HOLD

Assignment: [6695278609](https://github.com/lmmx/mibz2/pull/162#issuecomment-6695278609).
Reviewed candidate: `0a2d5b1389ef177372730ac0aaa19889316aa9b9`.
Previous HOLD: [6694897330](https://github.com/lmmx/mibz2/pull/145#issuecomment-6694897330).
Instruction pin: `a6a703cefcc40ac4115803f44a8a29ad76a3a7ef`.
Scope: independent source and evidence-path review; no implementation, decoder build, capture or timing run.

## Disposition

**Do not freeze this revision.** Withdraw the old-baseline incompatibility objection: the parent now watches the actual `mibz2@BASEAB` child. Retain H2 for concrete omitted build inputs. Repair the new census's permanent first-name cache before relying on it as a plan-validity gate. The interval algebra remains accepted and was not reopened.

### 1. Baseline child compatibility: resolved

In [examples/ab.rs](https://github.com/lmmx/mibz2/blob/0a2d5b1389ef177372730ac0aaa19889316aa9b9/examples/ab.rs#L1026), both arms resolve their child executable, spawn, attach `Census::watch(child.id())`, wait and add `observed_threads` to the row. It is not inferred from `thread_cpu` or a planned topology. The child-side `thread_names` field has been removed. The parent preserves the child's wall, CPU, RSS, length and hash fields; it does not substitute parent wall or CPU. LOG 6695160336 correctly retracts the previous path claim.

All **14** functions in pinned `bench/test_policy_ab.py` passed, including old-format base rows, the supplied old-child line and broken-input CLI cases. Executed directly with temporary-directory fixtures because pytest is unavailable. These are parser/judge boundary tests, not a fresh execution of the old Rust child. They establish compatibility with that row format, not census scheduling correctness.

### 2. H2: omitted root inputs still defeat the freeze

[.just/bench.just, lines 357–364](https://github.com/lmmx/mibz2/blob/0a2d5b1389ef177372730ac0aaa19889316aa9b9/.just/bench.just#L357) omits `build.rs`, `rust-toolchain` and `rust-toolchain.toml` from `measured`.

The enclosed `reproduce.py` copies the actual `measured=(...)` line into an isolated Git fixture. Results:

| Case | Existing guard result | Omitted evidence |
|---|---|---|
| Add untracked root `build.rs` | tracked-only status empty; measured-path status empty | full status: `?? build.rs` |
| Commit that new `build.rs` after the frozen candidate | clean worktree; measured diff exits 0 | whole-tree diff: `build.rs` |
| Add untracked `rust-toolchain.toml` | measured-path status empty | full status names the toolchain file |

Cargo automatically discovers root `build.rs`; this candidate's manifest does not disable it. It is an executable build input, not a harmless generated result. Rustup discovers either toolchain-file spelling. The Git checks were executed; Cargo/Rust are unavailable here, so no claim of a compiled counterexample is made. These input semantics follow the official [Cargo build-script documentation](https://doc.rust-lang.org/cargo/reference/build-scripts.html) and [rustup override documentation](https://rust-lang.github.io/rustup/overrides.html).

The added `.cargo` and corpus paths do close the previously demonstrated gaps for ordinary untracked configuration files and changed tracked corpus inputs. The corpus builder also checks the output against the frozen manifest's digest. Base-worktree untracked-file checking and executable/corpus hashes improve provenance. They do not close the omitted root inputs.

The environment check is a selected blacklist, not absence of Rust/Cargo overrides: e.g. `RUSTC_WORKSPACE_WRAPPER` and `CARGO_PROFILE_RELEASE_LTO` are not checked ([Cargo environment reference](https://doc.rust-lang.org/cargo/reference/environment-variables.html)). `inputs.txt` records executable/corpus SHA-256 values and toolchain **version output**, not toolchain binary digests, and only invokes the version commands from the head directory. Avoid claiming more.

**Closure:** include the root build script and both toolchain files in both clean/untracked and frozen-diff checks, including their expected absence; add refusal fixtures for these exact cases. Make the effective compiler/configuration shared and identified for both builds, rather than asserting that the short blacklist excludes every override. This requires no new infrastructure or measurement.

### 3. Census names are cached before their identity is stable

[Census::watch, lines 395–399](https://github.com/lmmx/mibz2/blob/0a2d5b1389ef177372730ac0aaa19889316aa9b9/examples/ab.rs#L395) reads `comm` only for a vacant TID entry. Every later observation of that TID skips the name read.

A source-level counterexample is `{tid: inherited_name} → {tid: mibz2-coop-1} → {tid: mibz2-coop-1}`. The stored value remains the inherited name forever. The supplied replay uses `mibz2-0` as that first name: helper count stays zero even after repeated observations that would identify the helper. Refreshing the same TID's name yields one helper. No new thread should be counted for a rename.

This ordering is plausible during ordinary named-thread startup: Rust creates the native thread, then sets its name from its own initialization path ([native startup](https://doc.rust-lang.org/stable/src/std/sys/thread/unix.rs.html), [ThreadInit::init](https://doc.rust-lang.org/src/std/thread/lifecycle.rs.html)). Thus an early observation can make a correct plan appear wrong and consume the registration's limited invalid-run replacement. This is an inference from source plus a deterministic state-transition replay, **not** an observed failure rate or a demonstrated false PASS on the real workload. A live /proc boundary test was attempted but could not run: this environment exposes no `/proc/self/task`; it was stopped.

**Closure:** refresh names for already-seen TIDs, retain identity by TID, and test the inherited-name-to-final-name transition and an idle helper. Do not synthesize missing names from the expected plan. The current old-child fixtures supply already-correct names and cannot catch this bug.

### 4. Census cost and scope

The disclosure that the parent runs the observer for both arms and does not measure its cost separately is acceptable for the **instrumented comparison**; a separate timing campaign is not required to repair this registration. Equal observer code does not prove equal perturbation: thread count and scheduling differ. Parent census CPU/RSS are outside the child's resource fields, while interference can affect child wall. Neither a CPU-total claim including the observer nor an overhead-free production effect follows.

Use “threads observed by the periodic census”, not an exhaustive claim about every thread that ever existed. Periodic sampling cannot guarantee observing a thread that lives wholly between samples. Also freeze or explicitly set the stated 5 ms cadence: `AB_CENSUS_MS` changes it today and is neither refused by the recipe nor recorded as a resolved cadence. These are instrument-definition corrections, not a request to change the accepted estimand.

### 5. Accepted wording and next step

The 16/12-block window statement and independent-paired-rounds caveat now say what was requested. Retry slots, retained attempts and explicit terminal outcomes remain useful; malformed input reporting passed its fixtures.

Expected: remove the base-format impossibility and pin the comparison's inputs. Actual: the base-format gap is closed, but the input freeze remains incomplete and the new observer has a name-stability defect. No performance gap was measured or closed. Mechanism learned: child compatibility and correct census collection are separate boundaries. Next useful work is the small input-refusal and name-transition repair, then source re-review. It has higher immediate value than an A/B whose validity can be lost for these reasons. The workstation run remains Louis's under C5.

## Reproduction and provenance

- `reproduce.py`: review-only Git guard checks and census state-transition model.
- `reproduce-results.json`: their captured output.
- `validation.json`: source hashes, test list and limits.
- Migration manifest SHA-256 matched `9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0`; old branch remained at `1283b969e9e295e268257cd1b6be3ad7da29e6dd`.
- Pinned coordination audit: five already-resolved historic misroutes, zero unresolved. Publication route: #145.
