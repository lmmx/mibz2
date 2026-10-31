# P74-C full-decoder trial: amendment 2, harness v1.2 (the end snapshot's thread lifecycle; before any confirmatory timing)

**Status:**
- Authorised by the maintainer, and committed before the i9 run.
- Harness v1.0 (pin `535296c`) and v1.1 (pin `e3f0940`, head `3870c12`, amendment 1)
  stay in history, with all their records.
- **Unchanged:** the P74-C kernel, selection policy, corpora, schedule, thresholds and
  gates. The timed path's normal case is also unchanged (below).

**The defect (measured):**
- **Where it surfaced:** in the unrolled-walk trial's exploratory CI run 41235849514,
  whose harness is this one ported. Two jobs aborted in `assert_confined` on a child's
  **end** snapshot: `thread |||| outside the allocation`.
- **The mechanism:**
  - The end snapshot lists `/proc/self/task` just after `dec.join()`.
  - A joined decoder thread can stay listed until the kernel releases it. In that run's
    completed job this happened in 79 of 1,386 children.
  - When the snapshot reads such a task mid-release, its `status` is gone, so the record
    is empty, and the check reads an empty CPU list as outside the allocation.
- This harness, v1.1, has the same code. Its CI validation (41198956443) didn't hit it, but
  the about 8,200 children of the i9 run could.

**The change (harness v1.2):** every thread listed by a snapshot is classified one of
three ways (`listed_record`):

| Case | Record | Accepted where |
|---|---|---|
| alive: `status` readable, with a CPU list | `name\|allowed\|psr\|vcs\|nvcs`, as before | its CPUs must be within the allocation, in every snapshot |
| exited: its `/proc` directory is gone (released during the read) | `EXITED:<tid>` | the end snapshot only |
| unresolved: the directory is still there, but no readable CPU list after about 10 ms of retries | `UNRESOLVED:<tid>` | nowhere: it always fails |

- **The spawn snapshot is unchanged in strictness.** Every decoder thread is alive and
  blocked when it is taken, so an `EXITED:` or `UNRESOLVED:` entry there fails, as does any
  live thread with an empty or out-of-allocation CPU list.
- **The summariser** (`bench/p74c_decoder.py summary`) re-checks confinement from the raw
  rows by the same rules. It reports the number of exited records in end snapshots.
- **The cost inside the timed interval is unchanged** (the spawn snapshot): a live thread is
  still read with the same two reads, `status` then `stat`. Only an unreadable task takes
  the new path (a directory check, then bounded retries).

**Validation (before any confirmatory timing):**
1. **Targeted tests** (`cargo test --example p74c_decoder lifecycle`, rustc 1.90.0):
   - a vanished directory is `EXITED`;
   - a directory without status, or with an empty CPU list, is `UNRESOLVED`, and a
     readable one gives its record;
   - the check fails on `EXITED` in a live-only snapshot, on `UNRESOLVED`, on an empty
     record and on an out-of-allocation mask, and passes on `EXITED` in the end snapshot;
   - **the race reproduced:** 3,000 snapshots taken right after joining three threads gave
     50–66 `EXITED` records a run, and no `UNRESOLVED` or empty record.

     Before the fix, those would have been empty records, which fail.
2. A local dry run up to timing.
3. CI integrity: three jobs with `P74C_STOP_BEFORE_TIMING=1`. Each runs the synthetic
   self-test's full trial path (children, end snapshots, the summary), and none is timed.

**The frozen hashes:** `frozen.json`'s `sources_sha256` for `examples/p74c_decoder.rs`
and `bench/p74c_decoder.py` are updated. No corpus hash changes.
