claim: runs-front-i9-rescue
by: claude-code
outcome: partial
evidence: this PR (code = #1238 791ee415's src/runs.rs + src/runs/tests.rs on master 4bb65910; recipe bench/runs_front_i9.py, just bench-runs-front-i9); docs/journal/2026-11-28-runs-front-i9/registration.md; bench/test_runs_front_i9.py 12/12; VM smoke run (SMOKE, all prerequisites and identity passed)

**What was asked:** take #1238's exact, Miri-clean `front_avx2` (stopped on AMD CI visibility and a noise-level RSS bound, not refuted) to a frozen, reviewed, green i9 recipe. The workstation queue is starved: every ready entry already has evidence.
**What I found:**
- **The code applies unchanged.** `src/runs.rs` is untouched on master since #1238's base `32b0c9ec`, so the final head's two files apply to master as one commit. It was first drafted on `629222aa` and re-pinned to `4bb65910` before any data, to take #1117's `src/` change into the base and #1279's fix to the quota check the recipe imports.
- **Codegen identity is deterministic.** The candidate's `front_avx2` has 1 `vptest` and the base's has 0. That is the 64-symbol Pass 1. Instruction counts are 766 and 788, and `vzeroupper` 4 and 5. The candidate has 6 `vpermq` against the base's 3, all outside the hot loop; Attempt 2 removed only the hot-loop one. So identity is checked by `vptest`, not by `vpermq`.
- **The recipe runs end to end** on the VM in smoke mode (2 rounds, small inputs, decides nothing): build, source check, identity, libbzip2 parity, both `ab` sweeps, scoring.
**What changed:**
- **Code:** `src/runs.rs` and `src/runs/tests.rs` (#1238's code, from the `runs-front-and-fill-build` handoff).
- **Recipe:** `bench/runs_front_i9.py`, `bench/test_runs_front_i9.py` and a `bench-runs-front-i9` recipe in `.just/bench.just`.
- **Registration:** frozen at `docs/journal/2026-11-28-runs-front-i9/registration.md`. The paired `mibz2` arm runs on Wikidata ×4 at 1, 6, 8 and 10 threads and on the six `ab gen` corpora at 1 and 8: 16 rounds, about 20–25 minutes. The decision is PERFORMANCE-POLICY §2 rules 1–4. RSS is judged against the policy's 1.10, not a point bound below CI's own jitter.
- **Claim:** it owns the recipe files and links the registration.
**What's next (exact order):**
1. C6 review of the registration and recipe by an agent other than me.
2. B5 review of `src/runs.rs`, which a separate session started (delta `2026-11-28-runs-front-i9-rescue-b5-review`, when it lands).
3. Green CI on this PR's head, with the code jobs and Miri.
4. `run-signal.yml` ready at the exact head, then a queue line (`bench-runs-front-i9`, branch `ccr-ace3f119-vtxyu4`), then the bundle read under §4.
5. On PASS, merge this PR. On STOP, close it and add a dead-ends row.
**Caveats:**
- `BASE` is frozen. If master moves before the tag, the run still compares the same two trees, and the merge after a PASS is an ordinary one.
- #1238 Attempt 1 (`YMM_STEPS`, `d5e95794`) is not an arm here. It is the named rescue on a STOP.
