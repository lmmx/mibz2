# Public sequential Cells integration — PR382

Author: Codex; owner-directed pickup6868355345, concrete review request6868508410,
Coordinator reconciliation6868441092/6868553543. Implementation head
`a9c7716aaf664a7898f3d8f37c955e09d5204a23`, base
`6dec95ba6aaf59d5134e5ab119fd52433e2032ee`.

The minimal public type selections and reviewed kernel import are complete. No
merge/release/workstation action. Existing source correspondence, exact entry-point
map, safety argument, allocation bounds and seven-family i9 evidence are in the
PR's `docs/journal/2026-11-14-cells-default/INTEGRATION.md`.

## Completed local evidence

Rust1.100.0 (0ef1bc386, LLVM23.1.5), custom writable Cargo/Rustup homes.
No performance measurements locally. Correctness tests are untimed qualifications;
their harness elapsed times are not decoder performance evidence.

- `cargo fmt --all --check`: pass.
- `cargo clippy --all-targets --all-features -- -D warnings`: pass.
- `taskset -c 0-3 cargo test`: 109 library +70 integration +1 doctest passed;
  one existing census and the external-input test ignored in this command.
  Local four-CPU affinity matches the managed CPU quota and avoids exercising an
  unrelated five-CPU plan fixture. This is not an i9 binding.
- `cargo test --example ab`: all27 harness/machine/resource tests pass.
- All7 compressed files SHA-256 and byte sizes checked against the immutable
  #380 pins; `source-inputs.json` contains identities. Existing text generated from
  pinned57f51bbc0ca62ac3ee1cc148a1010842eea6222a, not new master.
- `CELLS_PINNED_INPUTS=/workspace/mibz2-selective/cells-corpora cargo test --test
  cells_default public_defaults_pinned_corpus_parity -- --ignored --nocapture`:
  all7 pass, including209,700,196 Wikidata decoded bytes. Full byte comparisons:
  libbzip2, explicit Segmented, public decode, public BzReader, scanned decode_blocks.
- `cargo +nightly miri test --lib -- miri_cells`: all6 pass, including F01 long
  writer reach and selective live-state/mark/15/16/17/reuse. Miri0.1.0 ac8310ba64.
- Existing selective mutation runner applied to an isolated copy of this exact
  head: positive baseline passes; all3 mutants compile and fail expected selected
  payload/descriptor assertions. `mutations/results.json` records the actual
  documented kernel hash1221a9a5f0993daee73d8352fc559b11557ebd4929f16de72bf8eeb50e5f0b58.
  These are live-state kills, not claims of separate output-oracle kills.

The mutation runner is copied alongside its raw logs. Reproduce with its `ROOT`
set to the checked-out PR head before calling `main(Path(fresh_destination))`;
Cargo/Rustup environment must select1.100.0. Existing #380 F01/invariant mutation
campaign remains independent prior evidence; no claim to have repeated it here.

Failures retained: the first precommit fixture incorrectly attempted Spec.clone;
Spec has no Clone implementation. Replaced with slice::from_ref before the only
production commit; subsequent full Clippy/native suite passes. First local Miri
attempt failed before tests because its default cache was read-only (raw
`miri-cache-readonly.log`). Setting `XDG_CACHE_HOME=/workspace/.mibz2-tools/cache`
uses the existing writable cache; no safety flags disabled.

## Pending at this checkpoint

New full-public-route Miri test, forced/default fuzz smoke, final exact-head CI and
ordinary paired PR benchmark are still running. Guardian integration checkpoint
161/6868538108 is independently routed; no completion inferred.

The ordinary benchmark's one-thread `mibz2` arm calls actual BzReader::new;
multiworker arms call unchanged BzReader::parallel. Its separate phases executable
still measures the existing Segmented construction/walk and is not a Cells phase
attribution. Do not interpret that diagnostic as the new default's stage costs.
Whole-reader wall/CPU/RSS decides. No new timing arm, filtering or rerun requested.

## Disposition

Reviewable implementation, not merge-qualified. Carry Researcher6857443323's
native B5 scope and request Guardian's caller/lifetime/fallback integration review.
Keep failed #380 ordinary4t evidence; no claim to resolve its cause from unchanged
parallel source alone. Queued three-build attribution was parked by Coordinator.

Current6775049636 default gate requires other corpora/thread counts level and RSS
within+10%; PERFORMANCE-POLICY also requires owner sign-off for secondary losses
over10%. The previously validated i9 Wiki benefit does not erase pattern+43.5%
wall, Wiki+31.1%RSS or pattern+15.1%RSS. A prospective, narrowly scoped adoption
decision is needed, not a retroactive PASS. Actual-head CI may add blockers.
Separate registered public-default target confirmation before release remains a
Coordinator/owner decision. No run/queue signal or release is part of this task.

Audit via fresh connector comments and unchanged coord.py: six resolved historic
misroutes, none unresolved. REVIEW6868508410 readback exact, misrouted=None.
coord.py verify hard-codes FROM coordinator and flags Codex publications; this
limitation is recorded, not reported as a passing verify or used to impersonate
Coordinator. GitHub comments do not launch a manually activated Codex session.
