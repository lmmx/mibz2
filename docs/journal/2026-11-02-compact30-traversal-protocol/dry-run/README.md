# The recipe's untimed dry run on the VM

`just bench-c30-traversal <corpora> 20 dry-run` at the pin `b12b49b`, on the VM's four
CPUs, rounds −1, 0 and 1.

- `console.log`: the recipe's console (the corpus check, the toolchain, the tests, G3, the
  CPUs, the gates and measurement-path checks).
- `c30-analysis.json`: the analysis in dry mode: gates, path checks and the allocation; no
  timing figure.
- `room-false.diff`: G3's diff against the reviewed listing (empty: identical).
- `MANIFEST.json`: the archive's manifest (the pin, the corpus and binary hashes, the
  command, the exit status, every file's checksum).

The walk records (`c30-traversal.jsonl`) are not kept here: their windows are VM
timestamps, and no VM timing is reported. Its checksum is in the manifest.
