# Revision 4: the analyser-only G5 repair (reports 178, 179)

`equivalence.py` compares the repaired analyser with the pinned one at `c0f8d38` on the
regression suite's synthetic fixture:

    git show c0f8d38:bench/c30_traversal.py > /tmp/c30_traversal.c0f8d38.py
    python3 docs/journal/2026-11-02-compact30-traversal-protocol/revision-4/equivalence.py \
        "$PWD/bench" /tmp/c30_traversal.c0f8d38.py

`equivalence.out` is its output:
- **On complete, valid calibration:** 50 fixture variants give results identical to the
  pinned analyser's, apart from the added `S9_calibration` entry. The variants cover
  ADVANCE, STOP, INCONCLUSIVE, gate failures and a dry run.
- **Report 178's case:** one CPU's empty-window median is far above the others'.
  - Complete, both analysers fail G5 (INVALID).
  - With that CPU's entries deleted from both batches, the pinned analyser passed G5 and
    gave ADVANCE. The repaired one rejects the run at S9, with no figures.
