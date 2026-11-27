claim: segmented-fallback-build
by: antigravity
outcome: done
evidence: PR #1080 (merge 0e51f0cadab9bf285ec941c51706d50383371138, reviewed head 64686ead5fb2e49794d2da5b5c7f624aa37bcbd4), B5 review #1092, CI bench run 45948941328

**What was asked:** Close `segmented-fallback-build` after PR `#1080` passed independent B5 review (`#1092`), full CI (including `miri`), and `PERFORMANCE-POLICY.md` §2 paired `bench`, and merged into `master`.
**What I found:**
- Independent B5 review (`#1092`) approved exact head `64686ead5fb2e49794d2da5b5c7f624aa37bcbd4`, verifying all new `unsafe` blocks (`Runs::trusted`/`len` + `fill_avx2_trusted`, `build_lf`, `reverse_copy_scalar`/`reverse_copy_avx2`/`assemble_reversed_with`) cold and under Miri with both default flags and `+avx2,+bmi1,+bmi2,+lzcnt,+popcnt`.
- Official CI paired `bench` (10 rounds, Xeon 6975P-C, `21/21` `same bytes: yes`, zero new memory allocations) cleared `PERFORMANCE-POLICY.md` §2 cleanly:
  - Primary `text`: **-2.7% (`-5.2%, -0.1%`) faster** wall at 1t, **-4.3% (`-7.8%, -0.6%`) faster** at 2t, `-2.9%` at 4t; RSS `-0.8%..-1.5%`.
  - Primary `wikidata`: **-1.7% (`-1.8%, -1.5%`) faster** wall at 1t, **-1.6% (`-2.7%, -0.5%`) faster** at 2t, `-1.1%` at 4t; CPU/B lower at every thread count (`1.86->1.83`, `2.15->2.12`, `3.34->3.31` ns/B); `4t` RSS `+5.4%` non-significant ready-queue jitter (0 B new buffers).
  - Secondary corpora: `random` **-5.3% (1t) / -6.2% (4t) faster**, `records` **-6.0% (4t) faster**, `pattern` **-1.4% (1t) faster** (`+2.9%` noisy at 4t `<= 5%`), `runs`/`onebyte` flat.
- Merged `#1080` at `0e51f0cadab9bf285ec941c51706d50383371138` (`coord.py head-sha` verified before and after) and released `hold/segmented-fallback-build`.
**What changed:** Set `frontier/claims/segmented-fallback-build.toml` to `status = "done"` with `reviewed_by` recorded; added this closing delta.
**What's next:** `src/bwt.rs` and `src/runs.rs` are free of `hold/segmented-fallback-build`. Non-blocking follow-ups from B5 review `#1092`: add a `+avx2` Miri job step in `.github/workflows/miri.yml` and update `TVector`'s doc comment in `src/bwt.rs`.
**Caveats:** CI paired `bench` runs on 2 physical / 4 logical cores; workstation-scale multi-core behavior is unaffected on Wikidata Open blocks (`0` Open blocks route to `Segmented`).
