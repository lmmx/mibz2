claim: crc32-avx2-ring-rescue
by: claude-code
outcome: dropped
evidence: evidence 40bb1509 crc32-ring-rescue/run-20261128T082026Z.tar.gz (sha256 140f368926615dafcc46382efa304ee6817217a952e466d1d9321ff23ec6c644, checked); recipe 30886324 (PR #1295) against base 31c35662; registration docs/journal/2026-11-28-crc32-ring-rescue/registration.md; CI note #1331; first attempt VOID 0390e19f

**What was asked:** read the i9 adoption run of the zero-table AVX2 ring CRC-32 against its frozen rule, and apply §4.
**What I found:**
- **Verdict: STOP.** The run is valid: i9-10900K, the pinned trees and toolchain, the ring kernel only in the head build, and the same bytes in every cell.
- Rule 1 fails: wikidata x4 at 1 worker, wall 1.0025 [0.9915, 1.0137] against the required upper bound of 0.985.
- Rules 2–4 hold. Wikidata/4, /8 and /10 are 0.998, 0.998 and 1.014, all intervals containing 1, and text is flat. The only secondary moves beyond noise are both faster: runs/8 0.980 [0.965, 0.995] and pattern/1 0.990. Peak RSS is within the calibrated bands everywhere, the largest being random/8 at 1.014.
- Diagnostic (1T `station_balance`): the wikidata worker station is −1.0% [−1.5, −0.5] and the scanner −0.3%. The CRC saving is real, but it is about a fifth of the ~5% predicted from VM station prices and #1179's isolated 0.52 → 0.29 c/B. At about 1% of worker time it is below this run's ~1% 1T wall interval, so no paid gain is visible.
- CI on AMD at this base showed primary losses: wikidata/1 +1.0% and text/1 +0.9% (#1331). §4 would have blocked a universal default even on an i9 PASS.
**What changed:**
- `frontier/dead-ends.tsv`: a row scoped to this kernel under this rule.
- `frontier/surface.tsv`: row R annotated with the result.
- The claim is marked dropped.
- #1295 is closed unmerged, the queue entry is removed (separate queue-only PR) and the hold is released.
**What's next:** with the measured 1% worker saving, a standalone CRC kernel swap has no credible lever for a whole-decoder gain at the bars used here, on either Intel or AMD. Do not re-run this arm. The CRC cost remains reachable only inside a fusion that removes a pass (forward-rle1-crc, segmented-fused-postwalk), where the zero-table combine helpers from #1179 may still pay. That is evidence for those claims, not a new one.
**Caveats:** one 16-round run. The prediction error comes from pricing W from a VM and an isolated kernel; the station figure here is the first in-decoder i9 measurement of the saving.
