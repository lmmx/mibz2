# `scanner-a2-fast-i9-run` physical i9-10900K verdict: PASS (`9dacc085`)

- **Claim:** `scanner-a2-fast-i9-run` (adoption run for PR #1117, `scanner-a2-fast-build`)
- **Registration:** `docs/journal/2026-11-27-scanner-a2-fast/registration.md` (frozen pre-data at `40a16165`, C6 reviewed, queued in #1243)
- **Evidence bundle:** `origin/evidence` commit `9dacc085ff11beb2455ee1ad6d4d7cd1cc918261`, `scanner-a2-fast/run-20261127T233936Z.tar.gz`
- **Verified SHA-256:** `c42ecb37705312cdc7715b9eff3b8872dc3bbe2dd955936b4d4420c61aed212b` (exact match with `.sha256` sidecar)
- **Measured commits:** head `689e55f7e4307167fa3fd0c744e16a3f85528490` (#1117 merged onto master) vs base `2af67a675b39d9523e96686c98c72c984f799868` (second parent); `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, LLVM 23.1.1, seed `1117`, scanner pinned CPU `0`, worker mask `1..9,11..19`.
- **Verdict:** **`PASS`** (`findings: []`) — all four frozen adoption criteria and all correctness checks passed.

## 1. Correctness & Codegen Verification (§2)

- **Parity (`parity.jsonl`):** 28/28 checks (head & base × 7 corpora × `k=1, 10`, including `wikidata` x4 = 838,800,784 B / 1,724 blocks, `wikidata-single` = 209,700,196 B / 431 blocks, `text`, `records`, `random`, `runs`, `onebyte`) bit-identical SHA-256 against `libbz2`, with `bmi2: true` on all 28 checks.
- **Timed sample integrity (`runs.jsonl`):** 300/300 processes (15 cells × 10 paired rounds × 2 arms) passed exact byte-count and scanned-block checks.
- **BMI2 disassembly (`bmi2-base.asm` vs `bmi2-head.asm`):**
  - `bmi2-base.asm`: `22` `shlx`, `8` `shrx`, `0` `vzeroupper`
  - `bmi2-head.asm`: `22` `shlx`, `8` `shrx`, `0` `vzeroupper`
- **Binaries (`binaries.json`):** `head-real` `7d442a67…`, `head-phases` `d80a5242…`, `base-real` `e6abf1a8…`, `base-phases` `ae88fbd8…`.

## 2. Full Measured Results (10 paired rounds, head / base)

| Cell | n | Base median (ms) | Head median (ms) | Ratio (head/base) | 95% CI | Spreads | Peak RSS ratio (base → head KiB) |
|---|--:|--:|--:|--:|---|---|---|
| **`phases/wikidata/1`** | 10 | 142.774 | 136.306 | **0.9548 (-4.52%)** | **[0.9514, 0.9582]** | **disjoint** (`142.42–143.42` vs `135.63–137.24`) | — |
| `phases/text/1` | 10 | 7.703 | 7.852 | 0.9968 | [0.9736, 1.0206] | overlap | — |
| `phases/records/1` | 10 | 21.087 | 21.231 | 1.0044 | [0.9675, 1.0428] | overlap | — |
| `phases/random/1` | 10 | 17.242 | 17.032 | 0.9996 | [0.9656, 1.0348] | overlap | — |
| `phases/runs/1` | 10 | 0.177 | 0.167 | 0.9560 | [0.8975, 1.0183] | overlap | — |
| `phases/onebyte/1` | 10 | 0.017 | 0.016 | 0.9924 | [0.9252, 1.0645] | overlap | — |
| `real/wikidata/1` | 10 | 1341.301 | 1334.891 | 0.9930 | [0.9825, 1.0036] | overlap | 1.0007 (80,784 → 80,840) |
| `real/wikidata/8` | 10 | 194.302 | 193.967 | 0.9999 | [0.9952, 1.0047] | overlap | 1.0055 (106,362 → 106,948) |
| `real/wikidata/9` | 10 | 189.437 | 189.566 | 1.0041 | [0.9823, 1.0264] | overlap | 1.0068 (113,334 → 114,110) |
| `real/wikidata/10` | 10 | 195.150 | 196.533 | 1.0058 | [0.9848, 1.0273] | overlap | 1.0065 (115,802 → 116,552) |
| `real/text/8` | 10 | 19.407 | 19.366 | 0.9967 | [0.9791, 1.0146] | overlap | 1.0042 (81,112 → 81,454) |
| `real/records/8` | 10 | 34.716 | 35.130 | 1.0076 | [1.0016, 1.0137] | overlap | 1.0053 (81,064 → 81,496) |
| `real/random/8` | 10 | 30.474 | 30.552 | 1.0041 | [0.9900, 1.0185] | overlap | 1.0006 (76,120 → 76,168) |
| `real/runs/8` | 10 | 9.049 | 9.157 | 1.0077 | [0.9745, 1.0421] | overlap | 1.0043 (18,640 → 18,720) |
| `real/onebyte/8` | 10 | 12.418 | 12.777 | 1.0288 | [1.0038, 1.0544] | overlap (`11.69–13.19` vs `12.32–13.31`) | 1.0028 (28,480 → 28,560) |

## 3. Frozen Gate Check (§4)

1. **`S'` cut ≥ 3.0% on `wikidata ×4`:** **PASS** — ratio `0.9548` (`-4.52%`, `-6.47 ms` median), 95% CI upper `0.9582 <= 0.970`, disjoint ranges (`137.24 ms < 142.42 ms`).
2. **Guard serial scan (no > 2.0% regression beyond noise):** **PASS** — all five guards `0.9560..1.0044`, every 95% CI includes `1.000`.
3. **Peak RSS ≤ 1.02× at every real cell:** **PASS** — max `1.0068×` (`real/wikidata/9`).
4. **No whole-decoder loss on `wikidata ×4`:** **PASS** — `k=1` `0.9930`, `k=8` `0.9999`, `k=9` `1.0041`, `k=10` `1.0058` (all 95% CIs straddle `1.000`; `wikidata` remains worker-bound at `k=8`, `k* = 9.46`, so the `-6.5 ms` scanner cut unlocks headroom as worker cuts land).

## 4. Disposition

- **Verdict:** **PASS** (adoption evidence satisfied per registration §4; stands in for `PERFORMANCE-POLICY.md` §2 rule 1).
- **Next:** Integrate PR #1117 (`scanner-a2-fast-build`) after independent review and exact-head CI verification.
