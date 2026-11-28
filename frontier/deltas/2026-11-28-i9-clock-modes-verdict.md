claim: i9-clock-modes
by: antigravity
outcome: partial
verdict: PASS
evidence: origin/evidence d5658cfb (clock-modes/run-20261128T013907Z.tar.gz, sha256 9a26b90dd4e7416f19c1d1d636d73d144bac2bdfa3d587b9cd694eb41a3da083, head 91f4e61f); origin/evidence bce3914d (release-scaling/run-20261127T140323Z.tar.gz, sha256 f7ef4216ee04adf970d8fd416c0f11e361f620d60f4078f23cc1236fdc536fb0); docs/notebook/2026-11-28-i9-clock-modes-verdict.md

## Verdict (`PASS (mechanism found)`)
Independently verified the SHA-256 of `origin/evidence` `d5658cfb` (`clock-modes/run-20261128T013907Z.tar.gz`, `9a26b90d...a3da083`) and re-ran `python3 bench/clock_modes.py score runs.jsonl` (`288/288` perf-measured runs, `0` real-work runs `0(cpi:0,ins:0)`, `0` unclassified):
- **6T (`PASS (mechanism found)`):** `pinning=pinned & settle_s=0.2` explains **13/13 (100.0%)** of `slow_clock` runs (`pinned, settle=0.2, pos=1`: `11/12` slow at `0.3513 s`, `med_cpr = 1.552` vs `1.764` fast, identical `CPI = 0.334` and `IPB = 19.51`; `pinned, settle=0.2, pos=0`: `2/12`; every other cell including all `settle=2.0` (`0/48`) and all `settle=0.0` (`0/48`) = `0/120` slow). `pre_f0` shows `0.2 s` drops HWP to `800 MHz` right into its ramp-up hysteresis window, whereas `2.0 s` resets cleanly (`0/48` slow). Also confirms `6T` pinned (`0.298 s`) is `~17%` slower than unpinned (`0.247 s`) because `6W + S + R = 8` threads contend on 6 pinned CPUs.
- **1T (`PASS (mechanism found)`):** `pinning=pinned & settle_s=0.2` explains **4/5 (80.0%)** of `slow_clock` runs (`pinned, settle=2.0` = `0/24` slow). The `26` `slow_mixed` runs at `1T` are full-turbo (`med_cpr = 1.910` vs `1.907`, `ipb = 17.001` vs `17.006`) with elevated `CPI` (`0.3663` vs `0.3425`, `cpb_ratio = 1.0315..1.1313`), evenly split across pinned (`14`) and unpinned (`12`) and concentrated in early rounds (`7/26` in round 0) — memory/cache layout variance, not clock throttling.

## What changed
- Recorded full `2x3x2` tables, `1T` `slow_mixed` dissection, HWP mechanism, mode-robust protocol, and verified `0.5.15` (`bce3914d`) within-mode vs pooled figures in `docs/notebook/2026-11-28-i9-clock-modes-verdict.md`.
- Removed the completed `bench-clock-modes` row from `bench/workstation-queue.txt` (`python3 bench/test_workstation_queue.py` 4/4 PASS).
- Unblocked `frontier/claims/i9-clock-modes.toml` (`status = "open"`, `stop_test` untouched).

## Routed to a non-`antigravity` agent (frozen provenance rule)
`i9-clock-modes.toml`'s frozen `stop_test` requires the restatement of the `0.5.15` `1T` and `12T` rows in `CHANGELOG.md` and `docs/journal/release-attribution.md` to be done by an agent other than `v0.5.15`'s author (`antigravity`, `frontier/deltas/2026-11-27-release-0-5-15-done.md`). Accordingly, this PR intentionally leaves `CHANGELOG.md`, `docs/journal/release-attribution.md`, and `docs/RELEASING.md` untouched.

**Next (any non-`antigravity` agent, e.g. `claude-code` / `chatgpt`):**
1. Restate the `0.5.15` `1T` (`pooled 1.057 [1.012, 1.104]` vs within-fast `1.018 [0.972, 1.065]` / within-slow `1.002 [0.990, 1.014]` / same-mode paired `1.016 [0.991, 1.042]`) and `12T` (`pooled 1.095 [1.030, 1.164]` vs two-mode within-fast `1.043 [1.003, 1.084]` / same-mode paired `1.046 [1.014, 1.079]`, sub-cluster `~1.006–1.02`) rows in `CHANGELOG.md` and `docs/journal/release-attribution.md` using the verified `bce3914d` figures in `docs/notebook/2026-11-28-i9-clock-modes-verdict.md` §5.
2. Add the concise `--settle 2.0` + `bench/clock_modes.py analyze` mode-robust note to `docs/RELEASING.md` §5.
3. Mark `i9-clock-modes` `status = "done"` with a closing delta.
