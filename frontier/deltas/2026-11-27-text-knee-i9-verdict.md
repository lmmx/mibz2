claim: text-knee-i9
by: claude-code
outcome: done
evidence: origin/evidence 5e3f28cc text-knee/run-20261127T172728Z.tar.gz sha256 786084e2...ab7d4 (matches archived .sha256); recipe af7ff482 (#1139); docs/notebook/2026-11-27-text-knee-i9.md

**What was asked:** read the i9 bundle for the frozen text-knee probe and record its verdict.

**What I found:** **CONFIRMED** under the frozen rule: panel median knee k = 5 (repo_json 4, src_rs 5, docs_md 8) <= 7, wikidata x4 knee k = 9 >= 8.
- Checked: provenance head af7ff482, rustc 1.99.1, i9-10900K, registration byte-identical, 252/252 timed rows exact.
- i9 1T k* = W/S: text 3.43-4.19, wikidata 9.55 (r = 0.36-0.44, lower than the VM's 0.58-0.85).
- Text walls rise past the knee: src_rs 203 -> 256 ms and repo_json 218 -> 259 ms at k = 8.
- Walls are bimodal (~195 vs ~265 ms) with the scan thread 35-60% slower in the slow mode; docs_md's knee of 8 is a mode-mix artefact.

**What changed:**
- `frontier/DECODER.md` gains the text row; the stop_test requires it.
- `surface.tsv` S row: i9 knee and k* figures.
- Notebook entry `docs/notebook/2026-11-27-text-knee-i9.md`.
- Queue entry `bench-text-knee` removed now that its bundle has been read.
- Claim marked done; new idea claim `scanner-slow-mode-i9`.

**What's next:**
- Price scanner levers at k = 8 on text, worker levers on wikidata only.
- `scanner-slow-mode-i9` asks whether the slow mode is SMT co-residency or clock state; a placement fix may beat open scanner code levers on text.
- `ready/bench-text-knee` can be cleared; leaving it is harmless because the runner runs a tag once per commit.

**Caveats:**
- The single-corpus knees are sensitive to the fast/slow mode mix; the verdict uses the registered medians of 9 and stands.
- Only three text corpora, all from this repository.
- No fresh hold: master had the claim `blocked` (my mark while awaiting the run), so `claim` refused. This PR is records only, plus routine queue bookkeeping under the standing workstation authorisation, and nobody else could take a blocked claim.
