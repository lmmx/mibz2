# The unrolled room walk's full-decoder trial: amendment 2 (the end snapshot's thread lifecycle; before the replacement CI run)

**Status:**
- Authorised by the maintainer after exploratory CI run 41235849514.
- That run stays recorded as it happened (`docs/walk-unroll-decoder-records` `1e64f6d`):
  - r1 (EPYC 7763) aborted in the timed trial, and its partial rows aren't a result;
  - r2 (EPYC 9V74) aborted in the self-test;
  - r3 (Xeon 6975P-C) completed, including its `records` regression (w1 0.928
    [0.924, 0.933]), which stays part of the evidence.
- **Unchanged:** the kernels, corpora, thresholds, schedule, the stop rules and amendment
  1 (the W4 (c) acceptance).

**The defect and the change:** the same as the P74-C decoder trial's amendment 2 (harness
v1.2, `research/p74c-decoder` `b406a3b`). The patch is identical in both harnesses:

- **`listed_record`** classifies each thread in `/proc/self/task` one of three ways:
  - **alive:** its record, from the same two reads as before;
  - **`EXITED:<tid>`:** its directory is gone;
  - **`UNRESOLVED:<tid>`:** the directory is still there, but there is no readable CPU
    list after about 10 ms of retries.
- **`assert_confined`** accepts `EXITED:` only in the end snapshot, taken after the joins,
  and fails on `UNRESOLVED:`, on an empty record and on a mask outside the allocation.
  The spawn snapshot stays live-only.
- The summariser re-checks confinement by the same rules, and reports how many exited
  records end snapshots held.
- **The normal case's reads are unchanged,** including the spawn snapshot's inside the
  timed interval.

**Validation:**
- the targeted `lifecycle` tests (rustc 1.97.0);
- the pre-timing checks;
- then the one authorised **replacement exploratory CI run**: three jobs at w = 1, 2,
  recorded as the second exploratory validation, not as a rerun that replaces 41235849514.

Its purpose is the AMD decoder observations the first run didn't produce. No kernel
tuning, and no threshold changes.

**The frozen hashes:** `frozen.json`'s `sources_sha256` for `examples/walk_decoder.rs`
and `bench/walk_decoder.py` are updated, and this amendment is added to `amendments`.
