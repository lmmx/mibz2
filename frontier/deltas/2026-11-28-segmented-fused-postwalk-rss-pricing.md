claim: segmented-fused-postwalk
by: claude-code
outcome: partial
evidence: PR #1183 head 3c80d181 (src = blocked exact head 56049aeb) vs merge base 3d8fc666, local 20-round paired `ab run` + A/A control; docs/notebook/2026-11-28-segmented-fused-postwalk-rss-pricing.md

**What was asked:** take the parked integration of #1183 (refused: the PR head's card is `blocked`), so instead price the B5 review's next step, "a revision that removes the extra peak RSS", before anyone spends a CI cycle on it. No hold taken; smooth-zone measurement only.
**What I found:**
- None of the review's three candidate mechanisms reaches `pattern`, the failing cell. Its blocks never expand under RLE1, so `grow_out` never runs. The 2T/4T path is `decode_owned`, with pooled `n + n/16 + 64` buffers and no `s.ll` reserve.
- Locally, `pattern` 4T peak RSS head/base was 1.002 [0.963, 1.044] over 20 paired rounds. The A/A control (base against itself) was 0.991 [0.960, 1.023]. Both arms span 36-44 MB: this is the bimodal queued-output regime of the RSS calibration census, §2.4. `pattern` 2T: 1.022 [0.984, 1.061] against A/A 0.982 [0.951, 1.015].
- Where RSS is deterministic (1T), the head is lower: `pattern` 0.956, `onebyte` 0.992. The only deterministic increase is `onebyte` 2T/4T, +1.4% [+1.2, +1.6] (about 0.4 MB, probably the one copying regrow of a 51x-expanding block), which is inside 1.02.
**What changed:** notebook entry `docs/notebook/2026-11-28-segmented-fused-postwalk-rss-pricing.md`; this delta. No claim, hold, verdict or surface row changed (C2: BLOCKED on 56049aeb stands).
**What's next:**
- Don't queue an RSS revision of #1183 for `pattern`. It has nothing to remove, and under the frozen literal 1.02 point bound across 14 multi-thread cells, a single run fails by chance 17 times in 27.
- The eligible route is the one `crc32-avx2-ring-rescue` and `runs-front-i9-rescue` use: an i9 paired run of #1183's code under a memory rule calibrated before its data (calibration §5), with CI for no-regression only.
- Continue on #1183 itself, by `take-review` once its card is `open`/`review` again. Do not close it: the parked lease's `pr_head` (e12e0aa1) no longer matches the head (3c80d181), so a close would strand three src paths until `parked-hold-moved-head` lands.
**Caveats:** this is a 4-vCPU VM, not CI's runner and not the i9. The +1.4% `onebyte` cause was not instrumented. The timing columns were not read for any decision.
