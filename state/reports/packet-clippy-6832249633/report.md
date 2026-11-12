# Packet-census Rust 1.100 compatibility correction

Assignment: #363 comment6832249633. PR: https://github.com/lmmx/mibz2/pull/375.
Base: `1f2738c5448fc123a71fd201921e4101e664b0c6` (current master, including merged #374).
Exact validated/committed head: `ec1b84eb60c8f011da4fa41d6d7d4a839f91bf24`.
Branch: `maintenance/packet-census-clippy-1100`.

## Correction and equivalence

Only examples/packet_census.rs:583–584 changes: `for j in 0..pk.k` / `pk.path[j]` becomes `for &id in &pk.path[..pk.k]` / `id`.

Packet.path is [u32;16]. paths() writes path[k−1] with checked indexing before invoking the callback, with k from1..=kmax; all11 MODELS have kmax≤16. Every emitted packet therefore has1≤k≤16. Both loops visit exactly those k cell IDs in the same order. The comparison with c.id[y], break 'reps, opsi advancement for c.sb steps, whole counter and maximality assertion are unchanged. Slicing remains bounds-checked: k>16 panics instead of silently shortening the path as `.take(k)` would. This check occurs at slice construction; malformed packets are already excluded by the checked constructor, and no claim of identical panic timing for unreachable malformed packets is made.

One file,2 insertions/2 deletions. All other tracked blobs match base, including the article, source archives, src/, bench/, Just recipes, gates/workflows and lockfiles. No new unsafe code, lint allowance, toolchain pin-back, algorithm change or performance claim.

## Commands and results

Environment for Rust commands:
```
CARGO_HOME=/workspace/.mibz2-tools/cargo
RUSTUP_HOME=/workspace/.mibz2-tools/rustup
CARGO_BUILD_JOBS=4
CARGO_TARGET_DIR=/tmp/mibz2-clippy-1100-target
PATH=$CARGO_HOME/bin:$PATH
```
Toolchain: rustc1.100.0 (0ef1bc386 2026-11-09), installed with rustfmt and Clippy. Full version/install logs retained.

1. On unchanged base: `cargo +1.100.0 clippy --all-targets --all-features -- -D warnings` →101, exact assigned needless_range_loop at packet_census.rs:583. `clippy-before.log`.
2. After correction: `cargo +1.100.0 fmt --all --check` →0.
3. Required full Clippy command again →101, now at unchanged examples/mtf_two_level.rs:273, needless_range_loop indexing m.bases. `clippy-after.log`. Separate ClassA project-lint finding; no full-Clippy pass claim.
4. `cargo +1.100.0 clippy --example packet_census --all-features -- -D warnings` →0.
5. `cargo +1.100.0 build --profile fuzz --example packet_census` →0, default profile with debug assertions/overflow checks enabled; no profile overrides.
6. `/tmp/mibz2-clippy-1100-target/fuzz/examples/packet_census validate` →0, empty stderr. Existing exhaustive oracle validator:29,523 ternary columns through length9 plus120 periodic-family columns; every origin, five marker sets and all packet models. Counts:29,643 columns;254,907 walk checks;11,652,415 states;182,754,446 packets;89,711,145 composed paths;18,402,243 repeated packets;509,814 cell-row checks.
7. `cmp /tmp/mibz2-clippy-6832249633/validate.jsonl docs/journal/2026-11-09-packet-census/validate.jsonl` →0. All three output lines match archive byte-for-byte. SHA256:050734632c8d800033c4b81781a49497020515fd10fec2f7ced7ae0562f3f334.
8. `git diff --check` →0; only the authorised source file differs; committed worktree clean.

Not run locally: broad cargo test suite, fuzz campaigns, other harness tests, real-corpus census, benchmarks, workstation measurements or queue operations. Existing exhaustive validator directly exercises this changed maximality loop. No additional tests mirroring the implementation were added.

## Remaining blockers and routing

Scoped implementation and packet validation complete. Full-project Clippy remains red at mtf_two_level.rs:273, whose blob is unchanged from base (validation.json). Dispatch explicitly says to report further scope separately: no modification to that example. Coordinator must decide the smallest separate compatibility correction; this finding does not justify rerunning unchanged benchmarks or waiving checks. Ordinary exact-head Actions and legacy status must be reconciled before integration.

Hand control back to the sole Coordinator: integrate/route this maintenance result deliberately; prioritise obtaining the decisive hosted pre-commit output on #373 at ed6f424f238851ff8296e7a48c2b96dfb8332e7e and reconciling remaining integration dependencies before dispatching the next research task. This packet lint does not explain that previously unclassified hosted failure. No self-dispatched successor or reopening of R1/blog/integration.

Current-master coord.py audit was attempted and blocked by proxy403. No audit-pass claim. Route for a Codex REVIEW is145; verify's hardcoded Coordinator sender limitation persists. Use exact connector read-back plus unchanged header/misroute checks; do not claim verify passed.

Exact-head CI observation: Test43303537738 and packaging CI43303537632 in progress; hosted pre-commit success. No all-green claim or rerun.

Later CI observation: Test43303537738 rust job131480699603 completed failure. Actual downloaded log confirms Rust1.100.0 and the same mtf_two_level.rs:273 needless_range_loop (exit101); source head ec1b84e merged synthetically onto1f2738c. ClassA project lint, not setup. Format passed; downstream Tests, Harness tests, guarded service producer/judge, scan fault/finder tests and alternatives-lock check were skipped. Python3.10–3.15 and typing jobs passed at inspection; other jobs still active. Decisive log excerpt retained; no unchanged rerun.
