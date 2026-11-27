claim: segmented-fallback-build
by: claude-code
outcome: done
evidence: PR #1080 head 64686ead5fb2e49794d2da5b5c7f624aa37bcbd4; review https://github.com/lmmx/mibz2/pull/1080#issuecomment-7211406768

**What was asked:** an independent B5 review of #1080's new `unsafe` (`bwt::build_lf`, `reverse_copy_scalar`/`reverse_copy_avx2`, `runs::fill_avx2_trusted` under the `Runs::trusted` invariant).
**What I found:** B5 approve-equivalent, with no memory-safety blockers.
- **`trusted`:** it is cleared on every writer of the fields `fill` reads (`Default`, `load_shared`, `records_mut`, and entry to both fronts), and set only on the fronts' success returns.
- **`shared_runs`:** `front`/`front_scalar` reset it, so a stale `Arc` is never read under a trusted flag.
- **`fill_avx2_trusted`:** its bounds need only Σlen = `r.len` = `n`, giving stores at `w < n`, `w + 8 <= n + 8`. Its `set_len` and link bound need each byte's records to sum to `counts[c]`. Both fronts guarantee both, because each emitted record length is added to the same count cell.
- **`build_lf`:** its accesses stay in `0..n` whatever `counts` holds. Its final `ftab == lim` check forces `counts == occurrences` exactly (even on a 32-bit target, where the `usize` sum could wrap), so every link is below `n` and the `set_len` follows a full sequential write.
- **Reverse assembly:** bounds-checked destination slices, disjoint borrows, AVX2 entered only through `Kernels`.
- **Runs:**
  - `cargo test --release --lib`: 197 passed, 2 ignored.
  - clippy `-D warnings`: clean.
  - Miri on the three named tests: clean, both with default flags and with `-C target-feature=+avx2,+bmi1,+bmi2,+lzcnt,+popcnt`.
**What changed:** this delta, and a review comment on #1080. No code. I did not merge #1080: antigravity's hold runs to 13:18 UTC.
**What's next:**
- **CI Miri:** `miri.yml` runs without `+avx2`, so CI Miri never executes the AVX2 kernels (`is_x86_feature_detected!` is false under Miri). It is worth a small CI claim to add a `+avx2` Miri step.
- **Doc:** `TVector`'s doc ("built only by `build`/`build_fused`") is stale.
- **Integrator:** check PERFORMANCE-POLICY §2 against the bench. Wikidata at 4t shows peak RSS +5.4% (not significant), close to the stop test's 1.05 bar.
**Caveats:** the review covers that exact head only. CI `miri` was still running when the review was posted.
