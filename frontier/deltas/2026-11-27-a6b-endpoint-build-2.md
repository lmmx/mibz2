claim: a6b-endpoint-build
by: claude-code
outcome: done
evidence: PR #1053 merged as 07cf47a49f84 at exact head faa9df70f98d (reviewed first at b864878b7c22); review https://github.com/lmmx/mibz2/pull/1053#issuecomment-7210474172; registration docs/journal/2026-11-27-a6b-endpoint-build/registration.md

**What was asked:** an independent C6 review and integration of #1053: the frozen adoption registration plus the safe-Rust `a6b-endpoint-redesign` research feature in `src/open/endpoint.rs` and `src/open.rs`.
**What I found:**
- **The registration** was frozen before any workstation data: no `a6b` bundle on `evidence`, and no queue entry. Its §3 decision rule matches the claim's `stop_test` criterion by criterion.
- **The code is correct as read in full,** with no new `unsafe` and the default build untouched:
  - u16 node ids fit, since 3ns+3 ≤ 65,536 under Narrow;
  - the 20/20/16-bit image packing fits n ≤ 900k;
  - every long word is ≥ 16 bytes, so its hash reads stay in range;
  - the memo, inline and unspelled rule tags are disjoint;
  - generation tags never match zeroed slots, and wrapping is tested;
  - truncate versus pop keeps memoised bytes.
- **Tests:** under `cfg(test)`, the redesign matches forced-`Wide` on steps, nodes, words, traced and longest, with rules ≤.
- **CI:** all 35 check runs were success or skipped (miri, rust, fuzz-smoke, bench), and pre-commit.ci was success at both heads. Locally on the merged tree: feature 43/43, default 40/40, clippy and fmt clean.
- **Two master merges were needed,** both trivial: the `Cargo.toml` feature list beside `scan-oracle`/`ideal-parts`, and the claim file, keeping master's P1 note plus `status = "review"`.
**Non-blocking notes for the recipe** (on the PR; none relaxes C2):
1. The ≤ +1.0% wall bound holds at each of k = 8 and k = 10.
2. Run the `S ideal` arm at k = 8 unconditionally, decided before the data.
3. The recipe must emit `wkr_busy` directly.
4. Expect ~0.8–1.3 MiB resident per worker. `vec![NodeLinks::default(); 65536]` writes all 512 KiB at allocation; only the u32/u128 tables are calloc'd lazily.
5. "Zero-bounds-check" covers only the 64K tables.
**What changed:** #1053 is merged. This claim is marked done; `C-A6b` in `surface.tsv` is unchanged until hardware data exists.
**What's next:** `a6b-endpoint-run` (already admitted, #1081) writes the executable recipe, carrying the notes above. Queueing needs separate ownership of `bench/workstation-queue.txt`. `forward-emission` (`src/open.rs`) is now unblocked.
**Caveats:** nothing here is timed on the i9. The CI bench (4-core EPYC, −0.3% at 1W, CI spanning 0) is not registered evidence and decides nothing.
