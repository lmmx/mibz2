claim: scanner-adaptive-seam-window
by: claude-code
outcome: partial
evidence: #1144 at 0fda23812aecd8e170a88cb226933cf0fe318630 (base 07e4fadd5d50); ready/bench-scanner-adaptive-seam; bench/workstation-queue.txt

**What was asked:** finish integration of #1144 (adaptive per-block lookahead and one-shot seam window). Its frozen stop test needs an i9 run before any merge.

**What I found:**
- **Code (independent C6 review, author was another agent):** sound.
  - The seam invariants hold: `buf` always ends at `next_base + at`; the subtractions cannot underflow; every loop pass makes progress; EOF is reached only once `next` is drained.
  - `stopped` mirrors `cancelled`: both are set together in `cancel_locked`.
  - The retry growth reproduces master's `Truncated` and `BlockTooLarge` behaviour.
  - No new `unsafe`. Lib tests pass, 62/62, after merging master.
- **CI at the old head:** green, Miri included. The paired bench has no primary gain beyond noise on 4 CI CPUs. Wikidata at 2T: wall −1.3% (−3.1, +0.6), RSS −5.8%. Runs at 2T: +2.3%, which is secondary and within 5%. So PERFORMANCE-POLICY §2 cannot adopt the change on CI alone, and the registered i9 run is the path to merge.
- **Registration:** accepted as frozen. The fifteenth pass found that the trimmed i9 plan is worker-bound at *pinned* 8T/10T. This gate is *unpinned*, and the run is VOID unless the trace shows 8 workers / 32 slots. That is the plan #1118 measured, where #1031's 3.2 ms cut came out FASTER at 10T. The note is in registration §8.
- **Recipe:** there was none, so I ported #1031's harness, made it shallow-safe, and added the in_maint gate and the 1.05× RSS bound.
  - Sources are fetched by exact SHA into a private repository, and the merge base is checked against fetched history.
  - There are 17 unit tests, one of them a detached `--depth 1 --no-local` runner.
- **Cold review of the recipe (a separate reviewer agent):** CHANGES REQUIRED at first. The traced `occupancy` exited 1 on `ledger check: VIOLATED` for Open blocks, which would have VOIDed every run after the timed phases. I fixed it by porting #1031's trace-only Open-route ledger attribution, and also unset `CARGO_TARGET_DIR`. On re-review: PASS.
- **Full DRY run here (wiring only):** every phase rc 0. The trace shows base 12 spans and head 34, in_maint −97.8%, window capacity 0.27×.

**What changed:**
- #1144 branch: recipe `bench-scanner-adaptive-seam`; `bench/scanner_adaptive_seam{,_source,_trace,_identity}.py`, tests and two trace patches; registration §8.
- Under the standing workstation authorisation: `ready/bench-scanner-adaptive-seam` set at 0fda2381 and the queue entry added. Nothing for this recipe was queued, tagged or archived before.
- Claim marked `blocked` on the bundle.
- **Ordering slip:** I sent the ready dispatch alongside `verify` instead of after it. `verify` failed because the lease still recorded the old head e8b5859. A ready tag does nothing without a queue row, so I re-pinned the reservation (yield-review, then take-review at 0fda2381), and `verify` passed before this queue row was written.
- **Concurrent rewrite:** while I held the integration, another agent rebased the branch with a force-push, which dropped my recipe commits, and added #1208's genuine fix. That fix isolates the trace test, which flaked under parallel test threads, in `tests/adaptive_seam_trace.rs`. I kept the fix and cherry-picked the recipe on top. Nothing else changed: the base source at 07e4fadd is identical to c5c34377's, so the DRY run stands.

**What's next:**
- **Reading the bundle:** take the integration and check its SHA-256 on `evidence`. Then read `verdict.txt` and `in-maint.txt`.
- **ADOPTS:** merge #1144 under the CLAUDE.md gates and update the `S` row in `surface.tsv`.
- **NOT ADOPTED:** add a scoped `dead-ends.tsv` row and close #1144. The fifteenth pass says the lever should still pay off Wikidata (text) and at 1–2T, so weigh a rescue scoped to those before shelving.
- **When #1144 merges:** the claim file conflicts trivially with this PR's `blocked` status.

**Caveats:** no i9 data yet. On a 4-CPU VM, unpinned runs are not evidence.
