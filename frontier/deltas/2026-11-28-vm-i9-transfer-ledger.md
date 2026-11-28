claim: vm-i9-transfer-ledger
by: antigravity
outcome: done
evidence: docs/notebook/2026-11-28-vm-i9-transfer-ledger.md; origin/evidence 9de7b6e0, 9dacc085, c0b2341c, d6bc8ca2, 40bb1509, 95edea2a, a6e427b1, 5182d546, ccfa473f, e98948b6, 1950f77f, 77236472, c8489a45, 16d5e409; CI run 46131501937; PR #1328

**What was asked:** tabulate every comparable same-station, same-corpus `(VM/CI predicted, physical i9 measured)` lever pair from primary sources, report measured/predicted transfer medians and ranges by station (`S`, `W`) and build flags (`base`, `JCC-32B`), price upcoming local pre-builds, and update `frontier/DECODER.md` only if a per-station median lies outside `0.3–0.7×`.

**What I found (`10` primary algorithmic i9 pairs on `wikidata` + `v0.5.18` cohort + `#1328` CI-EPYC reference + `2` sign-reversing failure modes):**
- **Station `S` (serial scanner, algorithmic, `n = 4`):** transfers nearly **1:1** — **`base` median `1.05×` (range `0.99–1.13×`)**, **`JCC-32B` median `1.01×` (`0.99–1.03×`)** (`#1034` `1.05×`, `#1117` `1.13×`/`1.03×` & CI `1.00×`, `#1031` `in_maint` `1.05×`, `#1144` `in_maint` `0.99×`; `#1144` `9T` wall vs `S+in_maint` `0.67–0.68×`). **Outside `0.3–0.7×`.**
- **Station `W` (parallel worker, algorithmic, `n = 6`):** pooled **`base` median `0.74×` (`0.20–1.50×`)**, **`JCC-32B` median `1.09×` (`0.20–1.97×`, `n = 4`)** — cleanly **bimodal by mechanism**:
  - *Compute / ALU (`n = 3`):* **median `0.45×` (`0.20–0.47×`)** (`CRC32 ring` `0.20×`, `#1274` `0.21×` 1T `base` / `0.45×` 6T `JCC`, `#1053` 1W `0.47×`; `v0.5.18` batch `0.36–0.41×`; `#1328` CI-EPYC vs VM `base` `0.50×` / `JCC` `1.31×`).
  - *Branchy scalar pre-scan (`n = 1`):* `#1246` `within_cap` `1.00×` (`base` 8T) / `0.80×` (`nat` 8T).
  - *MLP / latency-hiding (`n = 2`):* **median `1.37×` (`base`, `1.24–1.50×`) / `1.85×` (`JCC`, `1.73–1.97×`)** (`#1217`, `#1312` 4-lane walk).
- **Sign-reversing failure modes:** `#1053` `+4.19 MiB` 8W L3 cliff (`−0.28×` vs `+0.47×` at 1W) and `#1119` Haswell `JCC-32B` flag (`−0.19×` `W` / `−0.14×` `S` on Comet Lake-S stepping 5).
- **CI bridge:** `#1245` open-route 1T EPYC (`−3.6%` on `v0.5.18`) predicts physical i9 `8T/10T` (`−3.3..−4.0%`) at `~1.0×` (`0.92–1.10×`).
- **Derived pre-build pricing (`JCC-32B`-anchored):** `default-a5a6a-fast-build` `−2.6..−3.8%` `W`; `u8-syms-handover-build` `−8.5..−10.2%` `S` + `−1.2..−1.6%` `W`; `L6a` `−0.8..−1.3%` `W`; `mtf-fold+F8` `−4.0..−4.6%` post-`lanes` `W`; `scanner-adaptive-seam-composed` `−9.95 ms` (`~−5.8%` on 8W `lanes`, already measured on i9); `#1343` `−5.7..−6.4%` `S`.

**What changed:** added `docs/notebook/2026-11-28-vm-i9-transfer-ledger.md`, updated `frontier/DECODER.md`'s pricing advice (triggered by `done_when` because `S` `1.05×` and pooled `W` `0.74×` lie outside `0.3–0.7×`), marked `vm-i9-transfer-ledger` `done`.

**What's next:** use this mechanism-calibrated prior (and CI `#1245` open-route 1T checks) when pricing future worker vs scanner i9 recipes.
