claim: crc32-avx2-ring-rescue
by: claude-code
outcome: partial
evidence: evidence 0390e19f crc32-ring-rescue/run-20261128T040117Z.tar.gz (sha256 44fa7413…01d7fb, checked); ready/bench-crc32-ring-rescue at 19902717 (#1298); fix 7a3dd5f6 on #1295; C6 review PASS (2026-11-28-crc32-ring-rescue-c6-review.md, #1292)

**What was asked:** read the first i9 bundle of bench-crc32-ring-rescue against the frozen registration.
**What I found:** no verdict. The run stopped INCOMPLETE (exit 1) about 30 s in, after both builds and before any timed or parity sample: `ab gen` panicked with NotFound in `tests/common/mod.rs::corpora`, which reads the text corpus from the binary's compiled-in `CARGO_MANIFEST_DIR`. `build()` had already removed the base worktree that the base `ab` was built in. Under registration section 4, that is a VOID (a failed prerequisite, not data). Nothing was measured, so C2 is untouched.
**What changed:** `bench/crc32_ring_rescue.py` now keeps the base worktree until the end of `run()` (removed in a `finally`, then `git worktree prune`). The rule, grid, pins and `src` tree (970ee202) are unchanged. A VM smoke of the fixed pipeline passed: both builds, the codegen audit (ring symbol only in head), every input including the wikidata x4 hash, parity in both arms, a 2-round `ab run` and `score()`. Its timings are plumbing checks on a non-i9 machine, not evidence.
**What's next:** the branch is rebased onto master 31c35662, because #1295 conflicted and master's src had moved (#1274, #1294, #1312); BASE and both src pins are updated to that master, and the rule is unchanged. A deliberate repeat because the first attempt was VOID. Move the ready tag to the fixed head once its CI is green, update the queue entry's commit, and read the new bundle.
**Caveats:** `station_balance` wasn't exercised in the smoke; it is jcc_aligned's code path, unchanged.
