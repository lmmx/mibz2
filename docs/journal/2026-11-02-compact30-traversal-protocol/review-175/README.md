# Report 175's review bundle

`review-evidence.zip` is the researcher's bundle, unchanged (also kept, unpacked, with
report 175 on the reports branch). Its `review_gate_checks.py` runs the analysis at
`b12b49b` (checked by Git blob hash) on five malformed synthetic record sets, all of which
that analysis accepted with ADVANCE. Run here unchanged, it reproduced its
`gate-review-results.json` byte for byte.

The five cases are rebuilt on the current fixture as rejection tests in
`bench/test_c30_traversal.py` (`test_r175_*`): each is now rejected by the schedule checks
before any ratio is computed.
