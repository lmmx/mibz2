# PR373: bounded TOML formatting correction

Assignment [#363 comment6839520708](https://github.com/lmmx/mibz2/issues/363#issuecomment-6839520708).
Operator evidence [6839448187](https://github.com/lmmx/mibz2/pull/145#issuecomment-6839448187).
Started2026-11-13 08:21–08:22UTC; implementation and local validation completed before08:28UTC, within60-minute bound.
Expected/observed starting head **ed6f424f238851ff8296e7a48c2b96dfb8332e7e**.
Appended corrected head **8329f0c702a1c5598e39e7f2413db55a12ce8884** on existing diagnostic/cells-live-path; no history rewrite/master import/merge.

## Exact correction and actual failure reproduction

Two changed lines only:
- bench/cells-gap/Cargo.toml: serde inline table becomes features=["derive"], version="1".
- bench/cells-gap/live/Cargo.toml: bzip2 inline table becomes features=["static"], version="0.4".

No value, dependency, feature, profile, build flag, lockfile, production or unsafe source change. Parsed before/after TOML documents compare equal via Python tomllib. All1572 other tracked files equal the starting head.

The repository pins lmmx/just-pre-commit v0.6.6. Its tag resolves to **3c1ff3e062dbdc776210dfc182dfa1db1611e2da**, bundling **Taplo0.10.0** and **Just1.40.0**. With the unchanged repository taplo.toml, that actual Taplo reproduces both reported manifest failures, exit1. It proposes exactly the two key reorderings. The provider's original output was operator-supplied; the reproduction here is independent and retained. ClassA repository formatting, not a Clippy/runtime failure.

Prior local Taplo0.9.3 passes the two targeted files and therefore did not reproduce this provider failure. An initial unrestricted invocation in the previously prepared worktree instead flags ignored live/.source/Cargo.toml. Both observations are retained; no frozen/generated file was reformatted. Final whole-repository pc-ci runs in a clean detached checkout of the starting head with just the two corrected manifests copied in, matching the hosted source layout without ignored preparation/build artifacts. This explains the earlier local/provider discrepancy without changing configuration.

Bundled Taplo SHA256:68ef7a7375390ef941c7930801a28438bbb65dd4777007fd925061f7cf50bc0f.
Bundled Just SHA256:38a1faf025ab566f40f67974c7dce27f1476762ae25b1d3ce78464c5c4cd2a7a.

## Source/evidence preservation

Old/new SHA256:
| live file | old | new |
|---|---|---|
| bench/cells-gap/Cargo.toml | cb54232a25e3cc5d4c71dfe7b5ed1d9defbbf09af7c286912a0967a92a108381 | 214b617d5528cad6a47dfa87181fd84b49eda516a37c8c49e7b812e494205dd9 |
| bench/cells-gap/live/Cargo.toml | 9538427f5e1b642c6b5c5d1ad90320ea0df8437aa5c46f759a4cbc27f42d6417 | 80e90ecff4ca78952069ac89420e9578c998fcc63ce79292fae754987afbc3f9 |

Historical SOURCE-SHA256SUMS deliberately retains the original parent-crate manifest digest. Its seven records still verify at original367 commit a08277e1c36e276cfea7b3116d99ea5fd93f0e84. This follow-up changes the live formatter presentation, so the old source manifest is not represented as a digest of today's working copy. No archived source/evidence/digest was overwritten or silently rebaselined.

The live harness build recorder will record its new manifest hash for any future separately authorised build. Previous build records remain historical. Measured decoder pin **d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2** and all53 original hashes verified against Git. Existing prepared source/overlay verified in place, including edits, generator/trace/hash/patch digests and additive-only patch. No preparation tree or old build evidence regenerated.

Prospective PLAN matches freeze3b43af956d553905ea744f6cb6e1c785a0f8297a byte-for-byte. All97 preparation and12 R1 archive hashes verified. Driver, R1 tests, schedules, estimator, source hooks, Rust source, configurations, historical records, lockfiles and recipes unchanged. The archived R1 check_protection.py is a historical comparison to its pre-accounting base, not a gate for this later two-file correction; it was not weakened or rewritten. New scoped verify.py separately checks this correction and preserves the old records.

No file literally named MANIFEST is present in the relevant preparation tree; the applicable records read/checked are PLAN, evidence/source-correspondence.json, evidence/verified-build.json, both SHA256.json archives, historical SOURCE-SHA256SUMS, prepare.py and build.py.

## Commands and exits

Tools/environment: hook bin directory first in PATH; writable CARGO_HOME=/workspace/.mibz2-tools/cargo, RUSTUP_HOME=/workspace/.mibz2-tools/rustup. pc-ci uses Rust1.100.0; existing native diagnostic test uses frozen Rust1.99.1. Just used for Python wiring tests is the actual pinned hook's1.40.0, rather than the diagnostic CI's separately pinned1.38.0; normal preparation CI retains1.38.0, unchanged.

- Taplo0.9.3 targeted format --check:0; initial broad worktree check:1 on ignored generated source. Retained, not a reproduction of the hosted two-file failure.
- Bundled Taplo0.10.0 format --check [two manifests]:1 before correction, exact two failures.
- Bundled taplo format [two manifests]:0; only2 insertions/2 deletions.
- Clean checkout taplo lint:0.
- Clean checkout taplo format --check:0.
- Clean checkout just pc-ci:0 (Taplo lint/format, whitespace1358files, cargo-machete, Rust1.100 cargo fmt --check --all). No suppression/configuration edits.
- python3 verify.py:0; parsed identity, tracked-byte preservation, frozen/archive/prepared-source checks above.
- CELLS_LIVE_REQUIRE_JUST=1 python3 bench/cells-gap/live/test_driver.py:0,13tests,0skips; real Just success/failure wiring with stubbed children.
- cargo +1.99.1 test --locked --manifest-path bench/cells-gap/live/Cargo.toml --features live-trace,cells-reader:0,5untimed tests; actual reader routes/bytes/CRC/errors, long-factor writer trace and scope cleanup. Existing nested-profile warning retained; no profile adjustment.
- git diff --check:0; committed worktree clean.

No corpus replay, performance measurement, four-release-build campaign, Miri rerun or broader Rust campaign needed for identical parsed manifests and unchanged source. Existing independent review/R1 acceptance is not reopened.

## Current-master and CI correspondence

Current master8138e014c53f8c61cf028cbea78d8d9ae134c865 contains375. No master merge or cherry-pick was made into373. Read-only git merge-tree succeeds; it retains both375 example blobs unchanged. GitHub's synthetic merge **078f37a5ef27240ee9161cfc70c134a95110939a** has parents current master8138e014 and corrected head8329f0c, and its tree **05e2a56c3381600ef975cbd351b19d973cb0f123** exactly equals that local prospective merge. Thus current-master compatibility is checked without modifying unrelated source or importing master.

Corrected-head hosted legacy **SUCCESS**, result https://results.pre-commit.ci/run/github/1435546475/1794558438.dZzzsTpVR12DA6C9_jbU1g.
Normal PR CI started from this genuine correction: Test43393348451, packaging43393348433, preparation43393348575, Occupancy43393349525, Benchmark43393348524. No manual rerun or workflow dispatch. Full-PR path filtering starts the ordinary benchmark/Occupancy workflows even though this appended diff is only TOML formatting; no local/workstation measurement was started. Latest snapshot accompanies the report; running checks are not passes.

Old cancelled macOS x86_64 job131361540210 remains a separate incident. Operator output names acquisition failure and internal server error correlation a612f7bc-7191-407c-9513-cbe60dd27773, but four reported annotations versus five supplied lines, and x86_64 job versus arm64 capacity notices remain unresolved. No invented notice mapping/final ClassB classification; no project steps ran and it was not a pass. Corrected-head macOS jobs have started; their outcome must be checked independently.

## Handoff

Scoped assignment complete; Coordinator reconcile corrected-head Actions and macOS outcomes, then decide separate integration/readiness. No merge, release, queue/tag/Run, gate/default/toolchain/configuration change.120measured+9untimed/oracle preserved; warm20+20 unadmitted; CPU/topology/corpus and argument-free recipe bindings remain pending separately authorised actual-i9 preflight. No self-dispatched successor. Direct coord.py audit remains proxy403; no audit/verify-pass claim. REVIEW uses connector readback and unchanged routing check; manual Codex launch remains required.


Closing08:29UTC CI snapshot: packaging43393348433 SUCCESS, including macOS x86_64 job131778321286 and arm64 job131778321437; Cells preparation43393348575/job131778289728 SUCCESS (unchanged frozen CI uses Just1.38.0). Hosted legacy already SUCCESS. No remaining macOS packaging check failure on this corrected head; old annotation/count/architecture discrepancies remain historical, not retrospectively resolved. Test43393348451, Occupancy43393349525 and Benchmark43393348524 still in progress; no all-green or integration/readiness claim.
