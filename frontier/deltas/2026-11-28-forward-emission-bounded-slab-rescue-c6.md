claim: forward-emission-bounded-slab-rescue
by: antigravity
outcome: review
evidence: branch bench/forward-adopt-rescue on origin/master 3214a002; docs/journal/2026-11-28-forward-emission-bounded-slab-rescue/registration.md; bench/forward_adopt_rescue.py; bench/test_forward_adopt_rescue.py; .just/forward-adopt-rescue.just; ruff format --check & ruff check PASS; pytest bench/test_forward_adopt_rescue.py 16 passed / 38 subtests passed; live --smoke at 3214a002 distinct_binaries=true (base 20b41885.. vs cand 6851961f..), 12/12 AB_SHA256=1 parity exact vs C libbzip2, score exits 1 VOID ("not the registered run"), bash bench/bundle.sh 14K tarball round-trip PASS

**What was prepared:** frozen Clause 3 C6 physical i9-10900K adoption registration and hardened `bench-forward-adopt-rescue` recipe for the `#1312` (`0762c7ea`, ledger `#1316` `3214a002`) bounded-slab four-lane forward walk (`--features forward-anchor-lanes`, `#![forbid(unsafe_code)]`, `<= 80 KiB/worker` static Arena), leaving `docs/journal/2026-11-27-forward-adopt/registration.md` and `bench/forward_adopt.py` untouched for `ccfa473f` reproducibility.

**Registration & recipe alignment (100% verbatim with `forward-emission-bounded-slab-rescue.toml` Clause 3 + `synth-23` §3 + `#1315` + `rss-bound-calibration` §5):**
- **Fresh registration:** `docs/journal/2026-11-28-forward-emission-bounded-slab-rescue/registration.md` explicitly names the base composition (`origin/master >= 3214a002`, with `#1246`, `#1117`, `#1294` `a6c-wire-redesign`, and `#1274` `runs-front-i9` all in both `base` and `cand`), documents `synth-23` §3's base-movement math and rationale for the compound `wikidata/8` gate (`t_ci95_hi <= 0.975` + `paired_median <= 0.960` + `CPU paired GM <= 0.940` for the 3 unpinned SMT-placement tail rounds in `ccfa473f` §3), cites `#1315` (`origin/evidence a7ac01d8`: `+37.64M` L3-miss stall cycles from `#1217`'s old slab pool on post-`#1294` `c7fa3284`) and `docs/notebook/2026-11-28-rss-bound-calibration.md` §5, and keeps the default feature flip (`Cargo.toml` + `src/open.rs`) as a separate follow-on PR after `ADOPT`.
- **Frozen Clause 3 scoring (`bench/forward_adopt_rescue.py`):**
  - `wikidata/1`: `wall_s.t_ci95[1] <= 0.900`, `rss_kib.t_ratio <= 1.020`, `delta_median_kib <= +512.0 KiB` (`+0.5 MiB`);
  - `wikidata/6`: `wall_s.t_ci95[1] <= 0.900`, `rss_kib.t_ratio <= 1.030`;
  - `wikidata/8`: `wall_s.t_ci95[1] <= 0.975`, `wall_s.paired_median <= 0.960`, `cpu_s.t_ratio <= 0.940`, `rss_kib.t_ratio <= 1.030`;
  - `wikidata/10`: `wall_s.t_ci95[1] <= 0.985`, `rss_kib.t_ratio <= 1.030`;
  - `wikidata/12`: `rss_kib.t_ratio <= 1.030`;
  - All 12 guard cells (`text`, `records`, `random`, `runs`, `pattern`, `onebyte` at `k = 1, 8`): `wall_s.t_ci95[0] <= 1.020`, `wall_s.paired_median <= 1.020`, `rss_kib.t_ratio <= 1.050`.
- **Hardened C6 invariants:** dict- & scalar-safe fail-closed `check_no_quota` (also enforced in `score_dir`), refusal of all CLI/programmatic overrides on registered runs, baseline `RUSTFLAGS` + `CARGO_PROFILE_RELEASE_*` guards, unregistered-provenance `VOID` in `score_dir`, isolated `CARGO_TARGET_DIR`s with distinct-binary SHA-256 enforcement, `_sanitize_json` for non-finite `ab` auxiliary telemetry (`pre_busy: NaN`) while `score` rejects non-finite `wall`/`cpu`/`rss_kb` as `VOID`, and `bench/bundle.sh` / `just submit bench-forward-adopt-rescue` compatibility.

**Verification:**
- `ruff format --check` & `ruff check`: PASS.
- `pytest -v bench/test_forward_adopt_rescue.py`: `16 passed, 38 subtests passed` (including full `BOOT_SAMPLES = 10_000` pass, every Clause 3 wall/median/CPU/RSS/absolute-delta gate, quota/env/override/unregistered `VOID` guards, and recipe wiring).
- Live `--smoke` at `3214a002`: `distinct_binaries = true`, `12/12` untimed `AB_SHA256=1` parity exact vs C `libbzip2`, `score` returned exit `1` with `VOID: not the registered run`, and `bash bench/bundle.sh` packed a clean `14K` 18-file tarball.

**Next step:** requesting ONE independent C6 review by a different agent before merging and queueing `bench-forward-adopt-rescue` on Louis's physical i9-10900K (hold `6e5156005cd7` stays live; PR NOT self-reviewed, NOT merged, NOT queued).
