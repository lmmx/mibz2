# PR #365: benchmark isolation and qualification

This continues [slice-one validation](../2026-11-10-cells-slice-one/VALIDATION.md).
The kernel, fallback, default and cooperative paths are unchanged. Independent B5
review remains required. The controlling gate remains [#145 comment 6775049636](https://github.com/lmmx/mibz2/pull/145#issuecomment-6775049636):
default arms level everywhere, correctness/B5/Miri pass; experimental figures reported,
not judged. No merge, release or default flip is authorised by this record.

## Original head and single permitted rerun

Head `b24d9baa167e01d57961b82a9a3c6f4501849190`, base
`edaa984c4f57614f25bb897e808ab12c59c48d8f`. All six Actions workflows and legacy
pre-commit.ci passed. CI success is **not** a performance-gate pass.

[Benchmark run 42883082689](https://github.com/lmmx/mibz2/actions/runs/42883082689):
Rust 1.99.1 (`58cf8f9c3`, 2026-10-13), AMD EPYC 7763, four logical CPUs on two
physical cores; ten paired rounds, unchanged corpus/thread grid and judge.

| default wall comparison | attempt 1 | attempt 2 |
|---|---|---|
| Wikidata 4t | +1.9% (+0.0%, +3.9%), slower | −0.6% (−2.2%, +1.0%), level |
| text 2t | +4.4% (+0.2%, +8.9%), slower | −1.4% (−5.0%, +2.3%), level |
| pattern 4t | +3.3% (+0.4%, +6.2%), slower | +4.1% (−2.6%, +11.2%), level |
| random 4t | level | +2.3% (+0.7%, +3.9%), slower |
| runs 4t | level | +2.6% (+0.5%, +4.6%), slower |

Intervals are the existing report's 95% intervals; rounded zero endpoints retain
its verdict. Every default-arm output hash agrees. The two primary losses did not
reproduce. The original pattern verdict is **not erased**, and the additional second
attempt flags are retained. This head is not declared default-neutral or qualified.
There will be no further rerun of this head.

The one primary rerun was announced before execution in [REVIEW 6778679718](https://github.com/lmmx/mibz2/pull/145#issuecomment-6778679718),
under PERFORMANCE-POLICY §2. The result and correction rationale were posted in
[REVIEW 6778891459](https://github.com/lmmx/mibz2/pull/145#issuecomment-6778891459).
The acting monitor acknowledged the initial report in [6778716882](https://github.com/lmmx/mibz2/pull/145#issuecomment-6778716882);
it has not dispatched or completed independent safety review.

Both overwritten sticky-report versions and full decoded job logs are archived in
`evidence/`. Job IDs: **130064200845**, **130113923685**. GitHub artifact IDs and
server-reported SHA-256 digests:

- Attempt 1: **13794754212**, `e6675ef4ee11d4d5e0f5752ced5a959424a5028922c68572c00cf41d86dbe50e`.
- Attempt 2: **13802041847**, `a9201085b4e296e5676005883340855c677b958197cc49403f91911c6b27572a`.

The artifact download host is blocked in this environment. These digests are metadata,
**not independently verified archive digests**. The local SHA256SUMS covers the fetched
logs, report snapshots and other named local evidence; it does not claim receipt of the
raw JSONL archive. Console rows are rounded and were not used to recompute gate statistics.
Experimental summaries/precise ratios still need artifact access; their rounded console
rows show no basis for claiming an end-to-end cells speedup. The prior i9 inversion-stage
ratios remain stage evidence only.

## Diagnosis and targeted correction

Matched Rust 1.99.1 release builds at the same source path establish a concrete
linking effect. With cells linked into the default `ab` executable, LTO changes
`drain<BzReader<File>>` from 2,688 to 2,524 instructions and moves default worker code.
The worker decoder's normalized instructions and all 27 common `phases` functions
are unchanged. Of 252 common mibz2-associated `ab` functions, 250 match; the other
difference is sequential-reader drop glue (111 to 69 instructions).

Restoring **only master's `examples/ab.rs`**, keeping the entire head library,
restores 252/252 normalized functions and the **entire `.text` section byte-for-byte**.
The correction's ordinary build has that same `.text`:

| build | `.text` SHA-256 |
|---|---|
| base; head library + base harness; corrected ordinary harness | `bb5f540c839a0fe9bd0000371862c9e48ac6629f9d642ae7860f7eb69b35865f` |
| original head harness | `79bb263df3946923c4acd25cf2fbfe8b8589afb34e72c6d7c5854561a4beb588` |

This demonstrates executable coupling despite unchanged routing. It does **not**
prove that coupling caused the non-reproduced +1.9% timing result. No additional
shared-machine timing is offered as performance evidence.

The correction gates only the harness module/dispatch on `mibz2_cells_bench`. Ordinary
`ab` keeps master's dispatch and `parallel()` code; the public `bwt::Cells` type
remains available without any feature flag. CI copies the ordinary binary before
building the cells executable. Its experimental producer uses
`bin/cells-ab run ... --arms mibz2@bin/ab,mibz2-cells --threads 1`, ensuring the default
child is the ordinary executable and cells records correctly say `parallel=false`.
The default head/base schedule, gate, sample counts and statistics remain unchanged.
CI also archives informational `.text` digests; equality is diagnostic, not a new gate.

Reproduction of the code-generation check: archive base and head into the **same**
scratch source directory in turn, use a shared scratch `CARGO_TARGET_DIR`, and run
`cargo +1.99.1 clean --release -p mibz2` before each archived-source build (archive
mtimes otherwise permit stale Cargo results). Build with
`cargo +1.99.1 build --release --locked --example ab --example phases`, save each
binary outside that source directory, then repeat with only `examples/ab.rs` restored
from base, and with the corrected source. Extract without rewriting the executable:
`objcopy --dump-section .text=OUTPUT BINARY /dev/null`; compare with `cmp`/`sha256sum`.
The normalized diagnostic used Pathfinder's `allcmp.py.txt` from PR #313; its address
masking is weaker evidence than exact section equality. The decisive comparison is
the actual `.text` bytes, not symbol normalization. No binaries are committed.

## Validation and remaining handoff

Correction-local validation (Rust 1.90.0) passes:

- `cargo test --example ab`: 27 ordinary tests.
- `RUSTFLAGS='--cfg mibz2_cells_bench' cargo test --example ab ab_cells::tests`: one
  adapter differential test, now explicitly covered in the Test workflow.
- `cargo clippy --example ab --all-features -- -D warnings`, with and without the cfg.
- `cargo fmt --all --check`.
- Separate-executable producer/summary smoke check: two rounds, four successful
  records, matching output hashes, cells marked single-threaded. Timing excluded.
- Both corrected release harness variants build with CI's Rust 1.99.1.

Source digests bind this correction's benchmark files and the unchanged kernel to
the archived local evidence. The existing Miri/fuzz/mutation evidence is retained
unchanged in the original slice-one directory.

Next: the corrected head receives its own unchanged CI performance gate. Do not
relax the gate or treat binary identity as a substitute for its result. Record its
exact SHA, Actions and legacy status, benchmark artifact identity and verdict in
#145. Independent adversarial B5 review is requested, still outstanding: review
the original safety argument, four-region layout/canaries, disconnected cycles and
marked words, speculative writes/prefetch, writer positions/overlap, malformed-input
parity, and actual fuzz/Miri/mutation reach. The implementer cannot certify this review.

The acting monitor reported reviewer routing inactive while #364 remains unmerged.
An authorised Coordinator must arrange the independent review and decide the gate
from the corrected head's evidence. No research or cooperative/team source was changed.

## Isolated-harness CI result

Head `989f782719ec69420713f94ed0b79cc5bedc8709`, [run 42899341988](https://github.com/lmmx/mibz2/actions/runs/42899341988),
job **130120814367**, Intel Xeon Platinum 8370C (a different runner from the two
original attempts), Rust 1.99.1. Every default output hash agrees. Wikidata is level
at 1/2/4 threads; 4t is **+0.0% (−2.6%, +2.7%)**. Text 1t is flagged
**+2.4% (+0.2%, +4.6%) slower**. Every other default wall row is level or faster.
Random 2t CPU is +0.9% (+0.2%, +1.6%) slower; default RSS stays within +3.1%.
Thus the slice-one performance gate is **still open**, not ACCEPT.

Artifact **13801838309**, reported digest
`2653899fef924a6618e3e1d5210624cf3aa305e76a6ca5b57c0979a844c793e2`.
Its bytes remain unavailable locally; the report and complete job log are archived
as `benchmark-isolated.*`. Test CI passes, including 27 ordinary harness tests and
the enabled adapter test. CI cells fuzz: **27,701 cases**, **14,539 forced admissions**,
**10,254 forced short cycles**, zero forced refusals (`fuzz-isolated.log`).

CI's `.text` digests differ: base `ee20eb0052bfec3654dfe308e30e7562b769f9cde1f25a479a93b3d4ee38c6d4`,
head `827a77a3d92101166cbab22f9f15c3c3a8b2a8136b6bc9bcc3e76fb349bdb551`.
The local exact-equality experiment controlled the source path; CI builds base in
`.base-src`. Rebuilding **unchanged master** locally in that extra subdirectory alone
changes its `.text` digest from `bb5f540c…` to
`72c68d1dcca0a0d1cb429023204e2c6690ebcf19748dc197314a7322ea3e701a`, while 252/252
common functions remain normalized-identical. The harness embeds `CARGO_MANIFEST_DIR`
for corpus generation. Thus whole-section equality across different paths is not
expected; the local controlled comparison remains valid, but **CI byte identity is
not claimed**. No benchmark-path/remapping change was made to seek a timing result.

## Formatting and summary visibility follow-up

The isolated-harness head is `989f782719ec69420713f94ed0b79cc5bedc8709`. Its legacy
pre-commit status failed. The external log host is blocked here, so that remote
failure's exact step is unverified (class D), not assumed to be infrastructure.
Running the configured Taplo check locally reproduced a concrete formatting failure
in the added `Cargo.toml` cfg array; the formatter's line wrapping fixes it. Parsed
TOML before/after is equal. Taplo lint/check on the changed manifest and the configured
whitespace check on the correction's files pass; logs are in `evidence/`.

The follow-up also tees `cells-summary.md` to the job log as well as the Actions
summary, making the reported experimental figures readable when artifact downloads
are blocked. It changes no producer command, data, timing, statistic or gate. Decoder
and harness source are byte-identical to `989f782`; the synthetic text corpus reads
only src/tests/examples/README, so these formatting/logging changes do not alter it.
The completed isolated-harness benchmark is retained above. The follow-up's required
exact-head CI measurement will count as the **one permitted primary-loss follow-up**
for this unchanged decoder/harness: if text 1t is slower again, that loss stands. There
will be no additional rerun or code-layout perturbation to seek a passing outcome.
Any secondary flag must likewise be retained under the original fixed slice-one gate.
