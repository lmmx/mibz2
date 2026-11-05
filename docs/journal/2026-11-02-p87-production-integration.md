# P87's production candidate: the integration cleanup (no timing)

- **Authorised by:** external report 158, task 2. It covers correctness and accounting only:
  - no change to any decoding algorithm;
  - no production timing;
  - no merge.
- **The candidate:** `0c121c5`. `Segmented` inverts in place over `ll`, and the worker never
  allocates `rle`.
- **This entry:** the ledger corrected, the contracts written down, and the differential
  checks added. The production-validation design remains a separate decision, with its
  no-harm requirements unchanged.

## 1. The ledger failure, reproduced and fixed

**The failure.** CI's test job (run 41352595637, job 124985814761) failed
`trace::tests::ledger_matches_a_workers_buffers`, with `left: 0, right: 140004`. It is
reproduced locally on `0c121c5` with the same values.

**The cause.**
- The test expected the worker's `rle` to grow as the separate-buffer path grew it:
  2 × 70,002 rows.
- The candidate never allocates `rle`, which is P87's intent. So the expectation was stale,
  not the decoder wrong.

**The fix** keeps every exact assertion, and adds one for the other kernel kind:
- **`trace::Settings` gains `in_place`.** It is true for `Segmented`, the default and the
  kernel the ledger describes. With it set:
  - the ledger's `rle` reserves 0 (at most 0, live 0);
  - `ll`'s note says it then holds the walk's output until RLE1 has read it.
  - With it false, as for `Plain`, the old `rle` row is unchanged: n reserved, 2n − 1 at most.
- **`ledger_matches_a_workers_buffers` (Segmented)** now asserts all of:
  - `rle`'s capacity is exactly 0;
  - the ledger's entry is exactly 0;
  - the capacity is within the entry's bounds.

  The other assertions are unchanged.
- **New: `ledger_matches_a_separate_buffer_workers_buffers` (Plain).** It runs the same two
  blocks through a kernel that inverts into `rle`, with `in_place` false. It asserts:
  - `rle` grew exactly by the growth rule (`rle == 2 × rows[0]`, its length rows[1]);
  - that is within the ledger's bounds;
  - `ll` and `tt` are as before;
  - every buffer is in the ledger.
- **`ledger_at_level_9`** checks the `rle` row both ways: 0 in place; 900,000 reserved and
  1,799,999 at most otherwise.
- **`docs/ledger.md`:** the `rle` and `ll` rows and the -9 totals are updated from
  `occupancy --ledger`. For example, at four threads: reserved ≥ 68.9 MB, against 72.5 MB
  before. These are ledger arithmetic, not RSS.

## 2. The contracts (`InverseBwt::invert_in_place`, src/bwt.rs)

**The preconditions are `invert`'s:** `counts` counts `ll`, and `orig_ptr` is below
`ll.len()`.

**On success** it returns true, and then:
- `ll` holds exactly the bytes `invert` would have put in its `out`, so its length is the
  original `ll.len()`;
- `ll`'s buffer and capacity are unchanged;
- the kernel's buffers are as `invert` leaves them.

**On fallback** it returns false and changes nothing: not `ll`'s contents, length or buffer,
and not the kernel. A kernel does one or the other for every block, never by the block.

**Why `Segmented` meets it.** There are two arguments, and they are complementary, not
interchangeable (report 158):
- **The algorithm's.** The build is `ll`'s last read: it copies every byte and link the walk
  needs into the table. The walk then reads only the table and its segment scratch. Both
  writers (`walk`, `SegScratch::assemble`) clear their output and write exactly the table's
  n rows, and n = `ll.len()` fits the capacity MTF reserved.
- **The borrow checker's.** It rules out conflicting live references in safe code: the
  build's shared borrow of `ll` ends before the walk's mutable one begins. That constrains
  the code. It does not by itself show the table holds everything the walk needs. That
  part is the algorithm's argument above.

## 3. The differential checks (tests/in_place.rs)

- **`segmented_in_place_equals_segmented_into_its_own_buffer`** holds the algorithm fixed
  and changes only the output's storage. Block by block, on every sample shape at levels 1
  and 9, with one reused kernel on each side:
  - `Segmented::invert` into a buffer of its own;
  - `Segmented::invert_in_place` over `ll`, straight from MTF.

  Every block must succeed with identical bytes and length, and `ll` must keep its pointer
  and capacity. The samples include short blocks (the plain walk) and full ones, periodic
  data (`abab`, period 3), zeros, runs and random bytes.
- **`a_kernel_that_falls_back_changes_nothing`:**
  - `Plain` and `Cooperative` return false;
  - `ll`'s bytes, pointer and capacity are unchanged;
  - the following `invert` matches a fresh kernel's.
- **`a_scratch_reused_after_an_error_decodes_exactly`:** one `Segmented` scratch decodes a
  bad block (a wrong CRC, then a primary index past the block), and then a good stream,
  exactly.

**Run here** (development VM, not timed):
- `cargo test`: every suite passes;
- `cargo test --features trace --lib --test in_place`: passes;
- `cargo test --features trace --example occupancy`: passes;
- `cargo fmt --all --check`;
- `cargo clippy --all-targets --all-features -D warnings`.

## 4. CI's stage table: unexplained between-build sensitivity

CI's bench report for `0c121c5` against `13e9854` has two kinds of figures:
- **The whole-decoder figures,** which exercise the in-place path. At four threads on
  Wikidata: wall −1.9% [−3.2, −0.7], CPU −2.3% [−3.4, −1.3]. That is four logical CPUs on
  two SMT cores.
- **The stage table,** which comes from `examples/phases.rs`. That harness still builds into
  separate vectors and never calls `invert_in_place`. Yet it shows "T-vector build"
  −1.4% [−1.7, −1.2] and "RLE1" +3.2% [+2.3, +4.2] between the two builds.

**How to read the stage table:**
- These are unexplained between-build sensitivities. They are not attribution evidence for
  P87's mechanism.
- They are not evidence that code layout caused them, or that layout explains the
  whole-decoder result.
- A layout control, such as a deliberately perturbed base build, belongs in the later
  validation proposal, if anywhere. One perturbation would not bound layout effects, so this
  does not open a benchmark matrix.

## 5. What this changes

**Nothing measured.** The decoding path is unchanged, and only tests, the trace ledger and
documentation changed.

**Still pending:**
- the production-validation design, a separate decision;
- the i9.
