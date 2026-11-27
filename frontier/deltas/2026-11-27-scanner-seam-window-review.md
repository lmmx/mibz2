claim: scanner-seam-window
by: claude-code
outcome: blocked
evidence: PR #1031 (reviewed at 93eab093, master merged in at 2c26b2b4), https://github.com/lmmx/mibz2/pull/1031#issuecomment-7210498644, bench/scanner_seam_window.py, examples/ab.rs (record format), bench/two_arm.py `canonical()`

**What was asked:** Do the C6 independent review of #1031 (the seam window, its frozen adoption registration, and the recipe and verdict script). If it passed, queue the i9 run.
**What I found:**
- The registration quotes the claim's `stop_test` verbatim (§3 and §5). Nothing for `bench-scanner-seam-window` exists on `evidence` or in the queue, so it was frozen before data.
- `ChannelSource::view` is correct by case analysis:
  - The buffer offsets stay consistent: `buf` covers `[base, base+len)`, and `next` starts at `base+len`.
  - Swapping after a partial absorb is safe, and `retire` keeps the seam.
  - Empty chunks and eof while a chunk is still waiting are handled.
  - There is no new `unsafe`, and the error-parity tests compare against `crate::decode`.
- CI on 93eab093 was green: every check run plus pre-commit.ci.
- Merging master in was clean. Master's src changes don't overlap this change.
- The recipe's shape follows CLAUDE.md.
- **Blocking:** `bench/scanner_seam_window.py` would VOID the real run in two independent ways. I reproduced both by mutating the test fixture.
  1. It keys records on the raw `arm` field. `ab` writes `mibz2@<absolute path>/build-base/…/ab` there, so every base record reads as unexpected. `two_arm.py` handles this with `canonical()`.
  2. It treats `err is not None` as a failure, but `ab` writes `"err":""` on every success.
  - The unit tests pass only because their fixture uses `mibz2@build-base` and `err: None`, which `ab` never emits.
**What changed:** Merged master into `build/scanner-seam-window`, posted the review comment and added this delta. Queue and tags are untouched, and the integration hold was yielded.
**What's next:**
1. The author or the next integrator makes the two fixes: `ta.canonical(r.get("arm"))`, and a truthiness test on `err`.
2. The fixture must emit what `ab` really writes.
3. Then: green on the new head, the queue entry with `args=build/scanner-seam-window,<head>,100`, and `ready/bench-scanner-seam-window` at that head.
4. Keep #1031 unmerged until the verdict. The registration decides a production default, and the recipe measures the branch against its merge base.
**Caveats:** `cargo test --release --lib` passed on 2c26b2b4 (201 passed, 2 ignored). The fix touches only tooling (the verdict script and its tests), not the registration or its rule, so it needs no re-freeze. A reviewer should still confirm the diff stays within those two files. I didn't dry-run the recipe end to end, so the `occupancy` and `lazy` phase wiring is checked only by reading it.
