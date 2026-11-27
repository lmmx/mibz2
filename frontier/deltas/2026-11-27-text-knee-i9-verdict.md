claim: text-knee-i9
by: claude-code
outcome: done
evidence: origin/evidence 5e3f28cc text-knee/run-20261127T172728Z.tar.gz sha256 786084e2...ab7d4 (matches archived .sha256); recipe af7ff482 (#1139); docs/notebook/2026-11-27-text-knee-i9.md

**What was asked:** read the i9 bundle for the frozen text-knee probe and record its verdict.

**What I found:** **CONFIRMED** under the frozen rule: panel median knee k = 5 (repo_json 4, src_rs 5, docs_md 8) <= 7, wikidata x4 knee k = 9 >= 8.
- Checked: provenance head af7ff482, rustc 1.99.1, i9-10900K, registration byte-identical, 252/252 timed rows exact.
- i9 1T k* = W/S: text 3.43-4.19, wikidata 9.55 (r = 0.36-0.44, lower than the VM's 0.58-0.85).
- Walls are bimodal: every thread is ~1.45x slower in the slow mode, which is `i9-clock-modes` mode (b). The verdict holds within each mode (fast: text flat over k = 4..10, wikidata falls to k = 9; slow: text 5-7, wikidata 9).
- Text's median rise past k = 7 (src_rs 203 -> 256 ms at 8) is mostly mode mix; docs_md's knee of 8 is too.

**What changed:**
- `frontier/DECODER.md` gains the text row; the stop_test requires it.
- `surface.tsv` S row: i9 knee and k* figures.
- Notebook entry `docs/notebook/2026-11-27-text-knee-i9.md`.
- Queue entry `bench-text-knee` removed now that its bundle has been read.
- Claim marked done. No new claim: the bimodality belongs to `i9-clock-modes` (#1201). I first opened a scanner-placement claim, then withdrew it before merge after a sibling's regrouping (#1209), which I re-checked on the bundle.

**What's next:**
- Price scanner levers at k = 8 on text, worker levers on wikidata only.
- `i9-clock-modes` should say whether high k on text drives mode (b). That decides whether "extra workers cost wall" holds for users or only for this harness.
- `ready/bench-text-knee` can be cleared; leaving it is harmless because the runner runs a tag once per commit.

**Caveats:**
- Single-corpus knees and absolute walls are sensitive to the mode mix; the verdict uses the registered medians of 9 and stands, within-mode as well.
- Only three text corpora, all from this repository.
- No fresh hold: master had the claim `blocked` (my mark while awaiting the run), so `claim` refused. This PR is records only, plus routine queue bookkeeping under the standing workstation authorisation, and nobody else could take a blocked claim.
