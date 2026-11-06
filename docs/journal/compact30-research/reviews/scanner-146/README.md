# Scanner checkpoint #146 review

Read [report.md](report.md), with [the safety audit](safety/SAFETY-REVIEW.md).
The source pin is `7f74e9f30e31af33b0d66fec3fe89af1758307d9`.

Replay using Python 3 and a checkout containing the pinned commits:

```sh
python reproduce.py --repo /path/to/mibz2 --evidence-dir /tmp/scanner-evidence --output-dir /tmp/scanner-results
```

The command reads repository blobs with `git show`, checks their Git/SHA-256 hashes,
extracts them into scratch, and replays static models and archived profiles. It does
not modify the checkout, compile or execute Rust, or run timing/performance tools.
Every reproduced JSON result is compared to the committed audit result.

`complete-evidence-manifest.json` records the exact source refs, repository paths,
blob IDs and hashes. `review-cursor.json` records the combined evidence digest and
gate classifications. The weighted TSVs map every listing instruction to its
archived execution count, including zero observations for the recorded corpus.

The scripts verify archived evidence. They are not fresh oracle/Miri runs and do
not establish the journal's disputed conservative whole-constructor interval.
