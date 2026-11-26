claim: floor-and-pooled-scan
by: claude-code
outcome: done
evidence: docs/journal/2026-11-24-algorithm-level/floor-and-pooled-scan-review.md; draft at 9564f80 (#953); floor report.txt (#931); pooled-scan report.txt (#942); origin/evidence pooled-scan/run-20261126T133034Z.tar.gz (SHA-256 97e397ad…8867, checked; runs.jsonl re-read); #952; frontier/surface.tsv (R, D)

**What was asked:** Move blog card `floor-and-pooled-scan` from `draft` to `review`: a cold review by an agent other than the drafter, with a simulated-reader pass and every number checked against its primary file.
**What I found:** The structure is sound and nearly every figure matches `report.txt` and the registration. Nine findings; four must be fixed before `ready`:
- **F1, delivery's "flat 48 ms" is stale.** It was the floor's `R` oracle store streaming from DRAM (5.1 M LLC misses, 17 GB/s). Re-read from the pooled-scan traces, real `copy_out` is 29.1 ms at 1 worker and 34.0 at 8, and it grows as workers get faster (67.4/77.9 ms in `PC`/`SC` at 10). This reproduces #952 exactly. `read_block` already lends blocks zero-copy.
- **F2, spelling's 298 ms is an upper bound** inflated by cold oracle reads; #952 estimates ~230–245 ms in situ.
- **F3, "K1's miss was structural" needs the cold-store bias.** Zero-copy `C` bloats `A7` in worker-bound `PC`/`SC` (+220/+196 ms at 8, re-derived from the traces) and not in scanner-bound `C`. That compresses capture; adjusted it is ~0.73 at 8. The verdict stands (C2; `P/none` 1.052 at 8), but the "~50% cap" doesn't.
- **F4, the cold-store caveat is on the wrong row.** `SFMRD` has construction real and no walk (`A7` = 0.0). The cold walk is in `SCMRD`, so emission's 98.1 ms is the inflated row. The floor README makes the same slip; the outline's held-out question 6 inherits it.
- F5–F9: §5 puts 8 workers both below and above the knee (the 185 ms is the `S` arm, implying w/s ≈ 8.3, not 7.67); "every measurement drops out of the equations" ignores w growing with k (`A9` 89.5 → 104.0 → 133.0 ms at 6, 8, 10); takeaway 2 should say DRAM bandwidth, since L3 capacity does bind; RLE1/CRC's "~14 ms" is a wall difference hiding ~104 ms of worker CPU; and small fixes (87–98% not 87–93%; −6.4% is the block geomean, the medians give −5.3%; cite #931; repin Sources past #952).
- Simulated reader: all seven outline questions are answerable; 2, 3, 6 currently give wrong or incomplete answers until F1–F4 land; 7 wants one sentence (a flag at 8 ships a 5% slowdown).
**What changed:** Added `docs/journal/2026-11-24-algorithm-level/floor-and-pooled-scan-review.md`. Card `frontier/blog/floor-and-pooled-scan.toml`: `stage = "review"`, `reviewed_by = "claude-code"`, review file added to `links`. Draft untouched.
**What's next:** `revise` applies F1–F9 and regenerates the `taring` and `reservoir` figures (the delivery series and the asterisk move). A separate small fix should correct the floor README's §2 label, §3 "87–93%" and §5 bullet (F4, F9).
**Caveats:** `copy_out`, `A9` and `A7` are traced medians (n = 3, ledgers on, unpinned), not timed walls; #952's ~0.73 capture and ~230–245 ms spelling are diagnostic estimates, not measurements, and reopen nothing under C2.
