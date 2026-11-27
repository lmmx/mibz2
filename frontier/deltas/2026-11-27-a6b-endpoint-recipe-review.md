claim: none
by: claude-code
outcome: done
evidence: #1085 at 02e492a7 (review https://github.com/lmmx/mibz2/pull/1085#issuecomment-7211520101); #1053 at faa9df70 (corroboration https://github.com/lmmx/mibz2/pull/1053#issuecomment-7211521208); registration docs/journal/2026-11-27-a6b-endpoint-build/registration.md

**What was asked:** a cold independent review of the A6b endpoint path to the workstation. That meant #1053's C6 registration and code (already reviewed and merged as 07cf47a when I started), and `a6b-endpoint-run`'s i9 recipe, #1085.
**What I found:** #1085 passes, with no blockers.
- **The decision rule:** its gates map one-to-one onto registration §3. They are 1W at ≤0.97 with ≥39 ms and non-overlapping median intervals, 8W real-scanner `wkr_busy` at ≤0.965 with ≥6.5 ms/w, and ≤+1.0% wall at each of wikidata 8 and 10 and text 1 and 8. `S ideal` runs unconditionally. No threshold moved.
- **The bundle:** it follows the one-tarball rules: a fresh `mkdir`'d directory, a tee'd console log, an EXIT trap that always writes `outcome.txt` and runs `bench/bundle.sh`.
- **The dry run:** I built the recipe's four binaries from faa9df70 as `prepare` does. I rebuilt the Wikidata sample, whose hash matched `bench/wikidata.json`, and ran its own `run()` for 2 rounds on a 4-CPU VM, 32 samples in all.
  - Every digest matched the libbzip2 reference at k = 1/8/10.
  - Open ran on 932 of 1,724 Wikidata blocks and on 2 of 47 text blocks.
  - `S` replayed every block, at a residual of 0.0026.
  - A corrupted block CRC is classified as `ByteMismatch`, which is STOP/KILL.
- **#1053's own suites at faa9df70:** default 40/40, feature 43/43, feature+mtf-fold 1/1. On re-reading, the u16 node ids (≤ 3ns+2 ≤ 65,535) and the 20-bit packing hold.
**What changed:** this delta, plus review comments on #1085 and #1053. No claim, tag or queue entry changed.
**What's next:** once #1085's `rust` job is green, `a6b-endpoint-run`'s holder (codex, live) merges it, sets `ready/bench-a6b-endpoint` and adds the queue entry. There is no prior a6b bundle, tag or queue entry, so the run is not a repeat. Expect about 25–30 min in all. Non-blocking notes on #1085:
- the ≤+1% bounds are point estimates;
- a failed scan-residual check makes the run INCOMPLETE;
- a panic without a CRC message is INCOMPLETE rather than STOP.
**Caveats:** VM timings decide nothing. Hardware evidence alone decides adoption.
