claim: forward-emission-run
by: antigravity
outcome: done
evidence: origin/evidence e98948b6 (forward-lanes/run-20261127T223754Z.tar.gz, sha256 db53fe2ca106b69a4b85e63bfb223e6072b5e02017e70720306e8f482de00fff); docs/notebook/2026-11-27-forward-emission-i9-verdict.md; docs/journal/2026-11-27-forward-emission/registration-disjoint-slabs.md

**What was asked:** Read the archived physical i9-10900K bundle for `bench-forward-lanes` (`#1168`, re-pinned to `#1217`'s `#![forbid(unsafe_code)]` disjoint-slab four-lane walk on `v0.5.17` `01acd74d`), verify its SHA-256 and exactness gates, apply the frozen §3 kill-test verdict rule honestly under C2, clear the queue entry, update `frontier/surface.tsv`, and file the P1 adoption successor under `#1224`.

**What I verified:**
- **Bundle integrity & validity:** `origin/evidence` commit `e98948b68fbd613f5f4b84d9ffb571df57819316`, `forward-lanes/run-20261127T223754Z.tar.gz`, SHA-256 `db53fe2ca106b69a4b85e63bfb223e6072b5e02017e70720306e8f482de00fff` verified exact; `commits.txt` head `022e94dc`, `base` & `cand` both at `v0.5.17` (`01acd74d`, default vs `+forward-anchor-lanes`); `env-facts.json` confirms Louis's 10-core/20-CPU i9-10900K (`unlimited` cgroup v2 quota, `throttling: false`); `exactness_ok: true` with 154/154 `runs.jsonl` records SHA-256 exact and `0` scanner-pin failures; scan-oracle residual `0.27%` (`base`) / `0.30%` (`cand`) `<< 5%`.
- **All 5 cells (`n = 15` paired blocks):**
  - **`wikidata/1` (forced open):** `1.353 s -> 1.113 s` (**`0.8282x`**, boot 95% CI `[0.8160, 0.8337]`, **`-17.18%`**, `-230.6 ms`, 100% disjoint; RSS `1.0119x` / `+0.94 MiB`). `r1_hi_le_0.950`: **PASS**.
  - **`wikidata/8` (real scan):** `0.1959 s -> 0.1844 s` (**`4.55 GB/s`**, **`0.9434x`**, boot 95% CI `[0.9377, 0.9483]`, **`-5.66%`**, `-11.08 ms`, 100% disjoint; RSS `106,388 -> 112,920 KiB`, `1.0609x` / `+6.33 MiB` paired median). `r8_hi_le_1.010`: **PASS**.
  - **`wikidata/8/none` (traced real):** wall `0.9427x [0.9266, 0.9537]` (`-5.73%`), **`wkr_busy/w`** `196.45 -> 169.61 ms` (**`0.8936x [0.8813, 0.8992]`**, **`-10.64%`**; RSS `1.0580x`).
  - **`wikidata/8/S` (ideal scan):** `0.1906 s -> 0.1683 s` (**`4.99 GB/s`**, **`0.8793x`**, boot 95% CI `[0.8570, 0.8954]`, **`-12.07%`**, `-22.79 ms`, 100% disjoint; `wkr_busy/w` `0.8789x [0.8549, 0.8950]`, `-12.11%`; RSS `1.0643x`). `r8s_hi_le_0.975`: **PASS**.
  - **`text/1` (guard):** paired median **`0.9964x`** (`-0.36%`, 9/15 paired blocks faster; `t_ratio = 0.9934x`, `t_ci95 = [0.9767, 1.0104]`; boot 95% CI `[0.9892, 1.0166]`; RSS `1.0000x [1.0000, 1.0043]`). `text_hi_le_1.010`: **unmet** (`1.0166 > 1.010`).

**Verdict (C2, unrelaxed):** **`RESCUE`** (`3/4` PASS clauses crushed with large disjoint margins, `0/3` KILL clauses triggered, `text_hi_le_1.010` missed by `0.0066` solely on 15-round bootstrap noise around a `-0.36%` faster median and `1.000x` RSS).

**What changed:**
- Added `docs/notebook/2026-11-27-forward-emission-i9-verdict.md` with the complete 5-cell table and per-block `text/1` audit.
- Updated `frontier/surface.tsv` (`C-A6c`, `E-walk`, `E-spelling`) with the physical i9 `e98948b6` numbers and `RESCUE` verdict.
- Removed the completed `bench-forward-lanes` entry from `bench/workstation-queue.txt` (`pytest bench/test_workstation_queue.py` green).
- Marked `frontier/claims/forward-emission-run.toml` `status = "done"`.
- Filed P1 successor claim `frontier/claims/forward-emission-adopt.toml` (`status = "idea"`, `priority = 1`) per `#1224` (`result-to-release`) with a pre-data C6 physical i9 full-guard adoption stop test to promote `--features forward-anchor-lanes` to the default `OpenCells` walk.
