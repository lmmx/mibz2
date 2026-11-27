claim: scanner-coalesced-u8-decode
by: claude-code
outcome: partial
evidence: PR #1034 (producer head d8b47e0c, merged with master efb3dd9d); CI bench run 45921211004; local Miri (nightly 2026-11-25)

**What was asked:** independent code and B5 review of #1034, and its integration under the registration's adoption path.

**What I found:**
- **Overrun parity is exact.** Each symbol is at most 20 bits, so `take` groups consume at most `take * 1000` bits from a start bit at or below `r.pos * 8`; with `r.pos + take*125 + 16 <= len` the run cannot end past the input, so skipping the per-group `overran` check there cannot change a result. Near EOF, runs fall back to one group with the old check, after the group's own errors, as before. The 16 B margin is conservative (mutant M3, dropping it from `safe_m`, is equivalent).
- **Run decode is exact.** Hot loop: `ck <= stop_ck <= 1000`, batch `chunk[ck..ck+18]` fits 1042, at most 10 symbols per pass, never past `target`. Tail: refilled on entry, at most 9 steps of <= 10 bits from >= 56 live bits; pairs never contain EOB, 256 or 257, so EOB and errors fall where the 50-symbol decoder put them, and a pair straddling a group boundary inside a run decodes with the same table.
- **LUT packing is exact.** Fast entries hold symbols <= 255 only; 256 (alpha 258) is `SYM256` in the cnt-0 slow path, EOB matched first (alpha 257 gives eob 256 = `EOB_FAST`). The template clone's `wrapping_add((s1 - s0) << 8)` stays inside SYM1 because s1 <= 255. LEN1 bits 24..28 overlap the tag bits, which are read only when cnt == 0.
- **B5.** The one `unsafe` is a call to a `#[target_feature(enable = "bmi2")]` fn behind `is_x86_feature_detected!("bmi2")`; its body is safe Rust, its arguments carry no SIMD vector types (no ABI mismatch), and `deny(unsafe_op_in_unsafe_fn)` holds. Sound.
- **Miri:** `miri.yml` doesn't trigger on `src/block.rs`/`huff.rs`/`bits.rs`, so CI never ran it. I ran `block::tests::miri_decode_runs_scalar_exact` under Miri myself: passes, and passes again with `-C target-feature=+bmi2`, which executes `decode_runs_bmi2`.
- **Mutation testing** (`block::`/`huff::`/`bits::` tests, with a control run): every non-equivalent mutant was killed (margin 125 to 100, no overran check, 256 to 255, 256 as fast entry, tail overshoot). Three survivors are provably equivalent: M3 above, `live -= e & 15`, and gating pairs at 1 selector.
- **Tests on master merged in:** lib 198/198, `tests/decode.rs` 14/14, `tests/service.rs` 10/10. CI bench at d8b47e0: same bytes everywhere; wikidata 1T -1.9% and 4T -2.2% wall beyond noise; no primary loss; secondary losses under 1%; the scanner stage -14.5%. It passes PERFORMANCE-POLICY §2.

**What changed:** merged master into the PR (52 commits; none touch the three owned files). Opened `scanner-coalesced-u8-i9-run` (idea) for the i9 kill test, which still has no recipe.

**What's next:** the registration's rule is "adopt / keep merged" on PASS and "revert" on STOP, so merging now is its adoption path. The i9 run waits for a recipe, `scanner-coalesced-u8-i9-run`. Suggested: add `src/block.rs`, `src/huff.rs` and `src/bits.rs` to `miri.yml`'s paths (a protected path, so outside this review).

**Caveats:** the local Miri run covers one deterministic test, not the 500-case parity test. There is no i9 evidence yet.
