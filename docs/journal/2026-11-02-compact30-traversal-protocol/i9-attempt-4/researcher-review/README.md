# The researcher's independent review of the fourth i9 capture (relayed verbatim)

A research input, not a finding of this repository. The three files are kept exactly as
the maintainer relayed them on 2 November 2026 (report 186):

- `independent_review.md`: the review;
- `independent_stats.json`: its recalculated statistics;
- `recheck.py`: the script that produced them, from the capture's raw records.

`recheck.py` reads the unpacked archive from `/mnt/data/c30_archive_audit/` and writes to
`/mnt/data/compact30_final_audit/`, the researcher's own paths, and needs SciPy. To rerun
it here, point `root` and the output path at an unpacked copy of
`../c30-traversal-20261102T200732Z.tar.gz`.
