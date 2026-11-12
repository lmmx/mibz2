# MTF Rust 1.100 compatibility correction

Assignment: [#363 comment6835658086](https://github.com/lmmx/mibz2/issues/363#issuecomment-6835658086).
PR: https://github.com/lmmx/mibz2/pull/375.
Parent: ec1b84eb60c8f011da4fa41d6d7d4a839f91bf24.
Exact corrected head: **853b4ac9eea20195a2dbc5b5374ab5fd2a2e8117**.
Master/base: 1f2738c5448fc123a71fd201921e4101e664b0c6.
Branch: maintenance/packet-census-clippy-1100.

## Bounded legacy retrieval (first, before implementation)

2026-11-12 21:21:46–21:22:07 UTC (21 seconds, within ten-minute ceiling).
Exact #373 head ed6f424f238851ff8296e7a48c2b96dfb8332e7e.
Result URL: https://results.pre-commit.ci/run/github/1435546475/1794504358.Y40p1JvNSESWR89bgRGktA.

Direct curl through the configured proxy returned exit56 / CONNECT403; the web reader reported the URL inaccessible. GitHub's exact-head legacy status still says failure at that URL. These are access observations, not hosted failure output. **Class D retained**: no decisive cause evidence, no inferred infrastructure failure, no connection to the Rust lint. No rerun, policy change or further retrieval campaign. Original headers/log, web response and GitHub status saved.

## Minimal correction and preserved invariants

One appended focused commit changes only examples/mtf_two_level.rs:273–274:
`for k in 0..K { m.bases[k] = N - 16 * K + 16 * k; }`
becomes
`for (k, base) in m.bases.iter_mut().enumerate() { *base = N - 16 * K + 16 * k; }`.

The array has exactly K elements; enumeration produces exactly k=0..K in increasing order. No take(), truncation or suppression. For K=0 neither version evaluates the expression. For each visited k, usize arithmetic, multiplication/subtraction/addition order and overflow behavior are identical. The previous indexed write was in bounds by construction; the iterator yields the same location. No hoisting of N−16*K or new precondition.

Current callers are Moving<1024,14> (bases800,816,…,1008) and Moving<4096,16> (bases3840,3856,…,4080), with224 and256 input bytes respectively. All16-byte sublists fit their arrays. Initial byte placement, take(), repack(), pointer loads/stores, AVX2 dispatch, assertions and scalar oracle are byte-for-byte unchanged. This preserves the existing unsafe assumptions; it is not a new independent safety certification.

Exact replacement assertion and changed-file check in validation.json establish the two-line-only edit relative to the parent. The accepted packet correction remains unchanged. No production, recipe, dependency, pin, article, gate or benchmark edit. Entire PR now has two files,4 insertions/4 deletions.

## Commands and results

Rust environment:
```
CARGO_HOME=/workspace/.mibz2-tools/cargo
RUSTUP_HOME=/workspace/.mibz2-tools/rustup
PATH=$CARGO_HOME/bin:$PATH
CARGO_BUILD_JOBS=4
CARGO_TARGET_DIR=/tmp/mibz2-clippy-1100-target
```
rustc1.100.0 (0ef1bc386 2026-11-09); full version saved.

1. Before correction, `cargo +1.100.0 clippy --all-targets --all-features -- -D warnings` →101: actual needless_range_loop at mtf_two_level.rs:273. Retained clippy-before.log.
2. `cargo +1.100.0 fmt --all --check` →0.
3. Full Clippy command above after correction →0. No further lint finding.
4. `cargo +1.100.0 run --profile fuzz --example mtf_two_level -- synth 0` →0. Existing synthetic scalar-oracle checks,17 cases ×8 checked arms =136 comparisons (result, output bytes and counts); both Moving variants exercise34 comparisons. Twelve fixed indices (1,5,15,16,31,32,40,63,64,100,200,255), four uniform ranges and one mixed distribution;899,999 byte symbols plus EOB each. Debug assertions/overflow checks enabled. Zero repetitions means the TSC closure and timing body are never invoked; stdout's inherited cycle heading and u64::MAX-derived sentinel columns are **not measurements** and must not be interpreted as such. Skeleton is unchecked and never invoked at zero repetitions. No corpus timing.
5. Additional `cargo +1.100.0 test --all-features --test mtf_kernels --test mtf_checkpoints` →101 at link: local linker cannot find -lpython3.12. No test execution/pass claimed for this attempt; full log retained. No source/configuration workaround.
6. `cargo +1.100.0 test --test mtf_kernels --test mtf_checkpoints` →0: all7 existing tests passed (3 checkpoints,4 kernels), including row edges, random streams, corrupt runs/limits and real-block equivalence. These test production kernels; the example's Moving variants are covered by step4.
7. `git diff --check` →0; worktree clean after commit. No tests added for the trivial refactor.

Prior packet evidence remains immutable at d2ba8d734ea0ef66a09ef98ec4d4586b43492617/state/reports/packet-clippy-6832249633. Its exhaustive validator was not repeated.

## Exact-head CI and handoff

At 21:26 UTC: Test43310214911 and packaging CI43310214895 in progress; legacy pre-commit success at the new exact head. Synthetic integration SHA6f7d50e540411442a86a6f6949105281ac57ce27. Later snapshot, if any, accompanies this report. No manual retries or local-all-green inference.

Scoped implementation and required local validation complete within the60-minute ceiling. Optional all-feature test execution limitation remains explicit; ordinary CI must be reconciled. No benchmark/workstation measurement, queue, Run, merge, release, default/gate change or B5 self-certification.

Coordinator next action: reconcile corrected exact-head375 Actions/legacy and decide integration; retain #373 legacy ClassD until actual hosted output becomes available, and route its remaining dependencies deliberately. CPU/topology and absolute corpus bindings remain PENDING for a separately authorised actual-i9 preflight;120 measured+9untimed design unchanged and warm20+20 unadmitted. No successor self-dispatch. A Coordinator GitHub message alone cannot guarantee restarting this Codex session; manual launch remains required.

coord.py audit attempted and returned proxy403. Route resolves REVIEW→145. No audit/verify pass claim; verify's hardcoded Coordinator sender limitation remains. Completion receipt uses actual connector read-back and coord.py check.


Later exact-head snapshot (ci-later.json): all packaging build jobs succeeded, including both macOS targets; release/publish jobs skipped as expected on a PR. Test workflow Rust Format and Clippy steps succeeded; Rust tests, fuzz-smoke and submit still running. Python3.10–3.15 and typing passed. Legacy remains success. No failure or cancellation observed on this updated head at this snapshot; overall Test not yet complete.
