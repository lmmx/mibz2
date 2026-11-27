claim: mtf-fold-run
by: claude-code
outcome: partial
evidence: PR #1041 (head e33e54b5bc979c8018d733d01465169eb0e8554b, squash a1869b5a24cf), PR #1048, frontier/claims/mtf-fold-l1-batch-redesign.toml, docs/journal/2026-11-24-algorithm-level/mtf-fold/registration.md §3

**What was asked:** Independent (B5) review and integration of PR #1041 (zero-copy scan oracle + frozen `bench-mtf-fold` recipe), then queue the i9 run.

**What I found:**
- Recipe vs registration §3: arms none/F/S/FS at k = 1, 6, 8, 9, 10, SM/SC at k = 8; 12 blocks, 3 traced; scanner pinned; 100 ms pre-spin; `ab gen` text at k = 1, 8. K1 (F/none@1 upper > 0.975), K2 (FS/S@8 upper > 0.975), K3 (F/none@6,8 lower > 1.01), K4 (median RSS F/none > 1.05 at any k) are verbatim. All five §3.5 VOID conditions are enforced (non-i9, pin failure, non-identical check pass, `folded == 0` or `bailed > 0` in F/FS, S scan residual > 5%, undefined judged bound). It adds `majflt > 0` as VOID: stricter only, fixed before data.
- Recipe shape (CLAUDE.md): a new `mkdir` run dir, `console.log` tee, `bench/bundle.sh` in the EXIT trap, nothing written to `$HOME`.
- B5: the existing `unsafe` kernels (`runs::front_avx2`, `fold::front_avx2`, `mtf_avx2`, `mtf_checkpoints_avx2`, `fill_avx2`) now take a separate `&[u16]`. Their safety arguments rest only on the slice's own length, `b.max_len` checks and clamped indices, so they are sound for any slice. The new `unsafe` is the `sched_{set,get}affinity` FFI on a 128-byte (`cpu_set_t`) mask, reached only when a scanner CPU is set, and not under Miri tests. `src/ideal/frozen.rs` is safe Rust. Miri was green on the head.
- Default build: every hook is `cfg`-gated. The one unconditional change, `open.rs`'s `walk`/`construct_and_record` refactor, is semantically identical; the bench job was green.
- CI: 31/31 check runs and pre-commit.ci green on the exact head. head-sha checked before and after the merge; no AI attribution.
- Non-blocking defect (research only): under `scan-oracle` Replay, `mtf_checkpoints`' scalar (non-AVX2) fallback still reads `b.syms` (empty). On a non-AVX2 host the check-pass VOID would catch it; the i9 takes the AVX2 path. Fix whenever `src/mtf.rs` is next touched.

**Decision: not queued.** #1048's dissection measured #1010's fold 1.61% *slower* at k = 1 (7/7 repeats) and 4.33% slower native. K1 needs a 2.5% speedup proven, so an i9 run of this implementation would very likely spend a slot on a near-certain KILL. `mtf-fold-l1-batch-redesign`'s frozen stop test also requires a local gate before any i9 run. I set no ready tag and added no queue entry (none existed). The recipe is now on master, ready for whichever fold implementation passes that gate. This choice is reversible: anyone may queue it under the 9 Nov standing authorisation if they judge otherwise, and should say why.

**What changed:** PR #1041 merged. `mtf-fold-run` is `blocked` (was `review`), with its reservation released (`finish-review`).

**What's next:** `segmented-fallback-build` (src/runs.rs) is unblocked. Reopen `mtf-fold-run` once `mtf-fold-l1-batch-redesign` passes its local gate; then set `ready/bench-mtf-fold` and add a queue entry at that commit.

**Caveats:** #1048's numbers come from a VM, not the i9. The decision not to queue rests on predicted, not measured, i9 behaviour.
