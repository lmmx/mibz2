# Reproduce the untimed review checks

This directory contains reviewer tests and results, not a harness correction.

Place `independent_checks.py` and `coord.py` in a fresh directory. In its `input/`
subdirectory materialize from lmmx/mibz2 commit
`7b53ded9eb139fdd69b4c29565e7373ac95e90d4` these exact directories:

- `bench/cells-gap/live/`
- `docs/journal/2026-11-12-cells-live/`

Run `python independent_checks.py`. It executes no decoder, real measurement clock,
settling or corpus replay. Child measurement results are entirely mocked. The
archived fixture replay checks JSON identities/accounting, not full decoded bytes.
For the author's Python tests, run `python input/bench/cells-gap/live/test_driver.py`;
real Just wiring requires the full pinned checkout and Just. Here Just was absent.

The source-correspondence reproduction used all53 files named by archived
`source-correspondence.json`, fetched at corrected365
`d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2`. `prepare.git` was replaced only for this
review with a byte-preserving reader of those files: `ls-tree` returned their
paths and `show PIN:path` their fetched contents. `prepare.generate()` then ran
unchanged. Its emitted source.json equals archived prepared-source.json in full;
its patch equals prepared-instrumentation.patch byte-for-byte. All shared fields
also equal source-correspondence.json. `source-check.json` records the results.
Running the unmodified generator in a clean pinned Git checkout containing the
corrected source object reproduces the same comparison. No compiler is needed.

`source-protection.json` records independent Git blob comparisons, including
27 historical367 harness/evidence files. PLAN was compared byte-for-byte to the
prospective commit. `coord.py` is the unchanged routing implementation from
the task's pinned spec commit. Audit used fresh connector discussion snapshots
for145/160/161/162/222; only the audit findings are retained here. Without an
`audit-input.json` mapping those issue numbers to original comment objects, the
script reports audit as not replayed and still checks REVIEW routing. Do not
infer a current audit from this historical snapshot.
