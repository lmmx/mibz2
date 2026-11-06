# #234 re-review: prior fixes accepted; two H2 boundaries remain

Assignment [6695535463](https://github.com/lmmx/mibz2/pull/162#issuecomment-6695535463), candidate `523ceaa922e1c479f90aad113891dc8ce98ebf0a`. Previous review [6695493108](https://github.com/lmmx/mibz2/pull/145#issuecomment-6695493108). Instructions pinned at `a6a703cefcc40ac4115803f44a8a29ad76a3a7ef`.

**Recommend retaining the freeze HOLD on this exact revision.** The previously named root-input, environment-variable and census-cache fixes are accepted. The revised build/provenance path still does not guarantee the two executables and configurations it claims to preserve.

## Closed findings

- The measured-path list now includes `build.rs`, `rust-toolchain` and `rust-toolchain.toml`. Independent Git fixtures confirmed refusal of each as an untracked addition and as a committed addition after the candidate.
- The environment filter rejects the demonstrated `RUSTC_WORKSPACE_WRAPPER`, `CARGO_PROFILE_RELEASE_LTO` and `RUSTUP_TOOLCHAIN` overrides, plus both sampler-interval variables. The intended installation/network exceptions remain allowed. This closes the prior selected-blacklist counterexamples and fixes the named cadence override.
- [`census_saw`](https://github.com/lmmx/mibz2/blob/523ceaa922e1c479f90aad113891dc8ce98ebf0a/examples/ab.rs#L369) now replaces the value for an existing TID. The caller rereads every available `comm` on every pass, with no CPU threshold. The new test covers the first `ab` label becoming `mibz2-coop-1`; this resolves the permanent-name-cache defect, including an idle helper observed again. Rust tests were inspected, not executed here.
- The actual base-child watcher, child-metric preservation, 16/12 window statement, model caveat and instrumented-comparison scope remain accepted. “Observed threads” is the right qualification; periodic sampling is not a lifetime-complete trace.
- The judge and its Python test file are byte-identical to the preceding reviewed revision. All 14 Python test functions passed again, directly invoked with temporary-directory fixtures. No interval-algebra re-review.

## H2a: the base copy is made after the head overwrites its source

[Recipe lines 406–411](https://github.com/lmmx/mibz2/blob/523ceaa922e1c479f90aad113891dc8ce98ebf0a/.just/bench.just#L406) do:

1. build base and retain its reported pathname;
2. build head and retain its reported pathname;
3. copy the base pathname;
4. copy the head pathname.

A shared Cargo `build.target-dir` can give both builds the same example-executable path. That configuration is allowed through the global Cargo config; rejecting the environment variable does not reject the config setting. Cargo documents configurable output placement in [build.target-dir](https://doc.rust-lang.org/cargo/reference/config.html#buildtarget-dir).

After step 2 the shared `ab` path contains the head build. Both subsequent copies preserve that head, not the earlier baseline. The registration's “neither build can overwrite the other's” claim is therefore false at this pin.

**Independent replay:** extracted the recipe's `built()`, toolchain check and build/copy sequence; supplied simulated successful Cargo artifact messages and distinct BASE/HEAD payloads at one shared output path. Exit 0; `bin/ab-base = HEAD ab`, `bin/ab-head = HEAD ab`. The original process-substitution form could not run because this sandbox lacks `/dev/fd`; the replay materializes those successful path lists in temporary files without changing build/copy order. This is a shell/order counterexample, not an actual Cargo compilation.

On the target machine this would normally fail the baseline-plan validity check at changed cells, rather than falsely qualify the policy. It still wastes the bounded run and violates baseline identity.

**Closure:** copy the base executable immediately after its successful build, before starting head, or enforce distinct output directories. Test this exact shared-path case and verify the saved base bytes remain BASE. No timing needed.

## H2b: a base-only ancestor configuration escapes both clean checks and the record

The config-recording loop at [line 428](https://github.com/lmmx/mibz2/blob/523ceaa922e1c479f90aad113891dc8ce98ebf0a/.just/bench.just#L428) hashes only `$CARGO_HOME/config` and `config.toml`. Cargo also searches the current directory and all ancestors, with nearer settings overriding more distant ones ([Cargo configuration hierarchy](https://doc.rust-lang.org/cargo/reference/config.html#hierarchical-structure)).

Concrete allowed path:

`<head>/target/three-team-policy/.cargo/config.toml`

The baseline checkout is below that directory at `build-<base>`; the head checkout is above it. A `[build]` setting there, such as `rustflags = ["-C", "opt-level=0"]`, applies only to the base invocation. It is inside the head's ignored `target/`, outside the base worktree, and outside the two recorded CARGO_HOME paths. Both standalone `rustc -vV` / `cargo -V` values can remain equal: compiler options are not compiler version identity.

**Independent fixture:** created an isolated Git repository, its detached nested base worktree, and this config file. Head tracked-only status, head measured-path status and base full untracked status were all empty. The config's effect follows Cargo's documented ancestor search; no compilation or measured speed change is claimed.

**Closure:** before either build, inspect the actual config search chains for both checkout locations, including the intermediate ancestors of the nested base worktree and the installation config. Refuse unintended arm-specific build settings, or make both effective configurations explicitly controlled and recorded. Hashing only the home files cannot substantiate “configuration outside the repository applies to both builds alike.” This is a concrete asymmetric configuration path, not a demand for new hermetic infrastructure.

## Decision and evidence limits

Expected: close the earlier input/census gaps so the registration can freeze. Actual: those specific defects are closed, but artifact-copy order and config discovery leave H2 open. No measured performance gap has been closed; no decoder build, corpus run, timing or new statistical rule was introduced.

The next useful step is the two small harness repairs and their boundary checks. Re-review can then focus on those changes. The accepted interval rule stays fixed; Louis retains the workstation run under C5. The observer remains part of the instrumented comparison, with its previously stated unmeasured perturbation limit.

Evidence: `replay.py`, `replay-results.json`, the exact recipe in `inputs/bench.just`, and `validation.json`. Cargo/rustc and a live /proc thread boundary are unavailable here. State migration manifest and old-branch SHA matched; coordination audit found zero unresolved misroutes (five historical resolved items). No implementation or authority change.
