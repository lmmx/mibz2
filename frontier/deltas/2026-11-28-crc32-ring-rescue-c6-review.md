claim: none
by: claude-code
outcome: done
evidence: PR #1295 at 19902717 (branch build/crc32-avx2-ring-rescue): docs/journal/2026-11-28-crc32-ring-rescue/registration.md, bench/crc32_ring_rescue.py, bench/test_crc32_ring_rescue.py (6/6 pass), .just/crc32-ring-rescue.just; `git diff --stat 0179ca9d 19902717`; examples/ab.rs `ab env` (`cpu_quota` is `null` when unlimited); frontier/deltas/2026-11-27-crc32-avx2-ring-barrett-build-2.md

**What was asked:** the one independent C6 review of `crc32-avx2-ring-rescue`'s adoption registration. The author is a different claude-code session, and I have no stake in the kernel. I read the code cold. I took no hold and wrote nothing to the author's branch.
**Verdict: PASS.** The ready tag and queue entry may go at the PR head once its CI is green.
- *(a) The rule is sound and frozen before data.* `score()` implements §3 clause for clause:
  - rule 1: `wikidata/1` upper bound ≤ 0.985;
  - rule 2: a primary cell STOPs on lower bound > 1 at any count;
  - rule 3: a secondary cell STOPs only when point > 1.020 and lower bound > 1;
  - rule 4: at k = 1, STOP only if GM > 1.005 and the median delta is > 256 KiB; at k ≥ 4, primary cells ≤ 1.030 and secondary cells at k = 8 ≤ 1.050.

  All thresholds are module constants with no CLI or `just` overrides (`run` takes only the output directory). The grid is complete, with no duplicates, and an unknown cell or round VOIDs. Every timed row must match parity's size and hash, which are bound to libbz2's SHA-256 on both arms. The C2 note and the predecessor's evidence exclusion are explicit.
- *(b) The 1.5% bar is right.* At 1T the saving is 0.23 c/B against roughly S + W ≈ 5–5.6 c/B, so about 4% is predicted. Note that the 0.52 → 0.29 stage figure is from a local VM, not the i9, so it is a projection there. The bar demands about a third of that, beyond noise. With 16 paired rounds and the clock-modes probe's 1T layout variance (CPI up to ~3–13% in a minority of runs, concentrated early), the half-width is about 1.5%. So a true effect of 0.96 passes, and a stage gain the whole decoder does not pay for (≤ ~1%) STOPs. A 2 s settle avoids the HWP slow mode.
- *(c) Yes, only `src/crc.rs` differs.* `0179ca9d` is an ancestor of the head. From it to the head, the only build input that changes is `src/crc.rs`; the rest is bench, just, journal and frontier files. The `src` trees match the pins (`970ee202` / `8b818058`). `sources()` refuses on any change to `Cargo.toml`, `Cargo.lock`, `examples/ab.rs` or `examples/station_balance.rs`. The `nm` audit requires `avx2_ring8_update` in the head's `ab` only. The ring dispatches at runtime on AVX2 + PCLMULQDQ (128-bit VEX), which Comet Lake has, so the i9 exercises it.
- *The quota check is not the forward-adopt VOID bug.* `run()` reads plain `ab env`, whose `cpu_quota` is `null` when unlimited (the `--facts` shape that broke `forward_adopt` is a dict).
**Notes for the verdict reader (no change required):**
- `--pin` pins only the 1T cells; k ≥ 4 runs are unpinned, as ab does.
- Rule 2 over six primary cells has about a 14% family-wise chance of a false STOP when the true effect is null. That is PERFORMANCE-POLICY §2's standing rule, so a STOP at k = 8/10 alone with a rule-1 PASS should be read as such and recorded, not relaxed.
- `rust-toolchain.toml` / `.cargo/` are not compared. `RUSTUP_TOOLCHAIN=1.99.1` and the `rustc -Vv` check cover the toolchain, and today they are identical.
**What changed:** this delta only.
**What's next:** the author merges #1295's prerequisites, sets `ready/bench-crc32-ring-rescue` at the PR head, and adds the queue entry under the standing authorisation, after checking the queue and `evidence`. Then they read the bundle.
