# bench-supplied-codes: registration

*24 November 2026. Written and frozen before any data, on `bench/supplied-codes`. The recipe is
`just bench-supplied-codes` (`.just/bench.just`), run as `just submit bench-supplied-codes`. The
verdict is `bench/supplied_codes.py`'s, tested by `bench/test_supplied_codes.py`. The
workstation conventions are bench-emit-cost's (#725) and bench-emit-cost-sq's (#792): one new
timestamped directory per attempt with its console log, the machine's records,
`bench/bundle.sh`, and nothing in the home directory. The experiment is the supplied-state
diagnostic from the outside review's round 2.*

**Diagnostic only.** This run decides no production default and no merge. ADVANCE means only that a
capability compiling retained records into output-order word codes is worth designing an
acquisition for, under its own registration.

## The question

Suppose Open's emitter were handed its existing word codes already in marked-cycle output order,
so it no longer walks the retained cycle to discover that order. How much faster is the decoder?
The answer bounds the payoff of any future "compile retained records into output-order word
codes" capability, before anyone builds the acquisition.

## The arms

There is one binary, `examples/supplied_codes.rs`, built with the `supplied-codes` research
feature (`src/supplied_codes.rs`, `src/open/supplied.rs`, and a hook in `Kernel::try_run`). The
arm is switched at run time between decodes (`mibz2::supplied_codes::set_mode`) and read once a
block, at its start. Without the feature nothing of this is compiled.

- **Arm A (`open`):** the current open kernel, unchanged. It runs the construction, then the
  current emitter: two-ended `emit_bidirectional` on the blocks the kernel already emits from both
  ends (`QUEUE && !wide && m ≥ 65,536`), and the forward `emit` on the others.
- **Arm B (`supplied`):** the same construction, charged as usual. Then:
  - **The supply.** The block's marked-cycle code sequence q_0..q_{L-1} (L is the actual cycle
    length) is looked up by the block's key. It is copied, one u32 a code, into the first L
    words of the existing 4m-byte packed-edge buffer (`edges`). From the consumer's entry the
    code stream replaces the edge buffer. The q_i are the existing dictionary codes, the same
    ones arm A's walk reads.
  - **The consumer.** It reads the codes sequentially by index, from the front and the back on
    a two-ended block, or from the front alone on a forward block, matching arm A's choice of
    emitter on that block.
  - **The appends stay unchanged:** the `lens` lookup, the unconditional 16-byte `inline16`
    store at the front, its shifted twin at the back, and the long words' slices from
    `spelling`. These are real output stores into the same `out`.
  - **The short-cycle behaviour is kept:** the clamp to n, the meeting test (`f == b` is the
    walk's `front == back`, `f > b` its `front == next`), and the `copy_within` and `repeat` of
    a cycle shorter than the block.
  - **Nothing else is supplied:** no decoded output, no offsets, no new word boundaries, no new
    dictionary.
- **The oracle (outside the clock).** Before the rounds, one decode in `record` mode walks each
  open block's marked cycle once, from the marked row (by the XOR links on a two-ended block,
  by the forward links and `extra`'s codes on the others). It records the codes in output order
  into an in-memory store keyed by the block.
  - *The key:* rows n, row p, retained rows m, the marked row's rank, and the run records' count,
    first and last. The construction knows all of these in O(1).
  - *Collisions:* a key recorded twice with different codes is a collision. The harness stops,
    and the run is VOID.
  - *The corpus's repeated copies* share their keys and codes. The store holds the distinct
    blocks once.
  - *The lookup* takes a read lock and a hash probe. It is charged to arm B's delivery.

## The receipts (two, from one run)

1. **Primary:** the whole-decoder wall time, with the delivery of the supplied codes charged.
   These are the decodes with nothing conditioned, *cond 0*. Arm B's lookup and copy into
   `edges` are inside the clock. The offline discovery is not.
2. **Diagnostic:** the matched consumer (emit-only) timing, from separate decodes in the same
   run, *cond 1*.
   - *Oracle substitution outside the clock:* arm B's lookup and copy are timed by their own
     clock read (`deliver_ns`), outside the consumer's (`consume_ns`).
   - *A common cache-conditioning policy for both arms:* after the delivery and before the
     consumer's clock, both arms read once, sequentially, every buffer either consumer may read:
     `edges[..m]`, `extra`, `inline16`, `lens` and `spelling`. That is one load a 64-byte line.
     The cond-0 decodes have no conditioning, so arm A there is exactly the kernel as shipped.
   - *The figure:* sum(consume_ns) / sum(m) over the open blocks of a decode, B over A.

**Per block, in every arm and mode,** the harness takes five clock reads (construct | deliver |
condition | consume) and records:
- n, m and the cycle's length L;
- whether the block was two-ended, refused (it went to `LazyCells`) or supplied;
- the worker kernel's scratch, every buffer's capacity;
- the block's key id.

**Byte-exactness, checked each round:**
- *The check decode.* At the end of every round, outside the clock, one decode in `check` mode
  at 10 workers runs, on every open block:
  - arm A's walk;
  - the block's own cycle, walked again and compared with the stored codes;
  - arm B's consumer on the stored codes.

  Any difference in the codes or the bytes counts as a mismatch.
- *The CRCs.* Arm B's bytes go to the output, where the decoder checks each block's CRC and
  the stream's. Every timed decode in both arms is CRC-checked the same way and must write
  838,800,784 bytes.
- *Why not compare inside the timed decodes:* the comparison would put work on the clock. The
  check decode compares every block, every round, outside it instead.

**Peak scratch per arm:**
- *The kernel:* the largest worker kernel's scratch in each arm, from the block records. Arm B
  adds nothing to the kernel, because the codes go into `edges`.
- *The store:* its bytes, from the harness's `meta` line. These are the supplied codes, held for
  the run.
- *The process:* its peak RSS (`/usr/bin/time -v`, where installed). One process runs both
  arms, so this is a single figure.

## The protocol (fixed now, before any data)

- **Where:** native, on Louis's workstation (the i9-10900K), unpinned, as the existing recipes
  run. Production's lazy workers (`BzReader::parallel_workers`, lazy: `OpenCells`, the window
  `SLOTS_PER_THREAD` a worker).
- **Worker counts:** 1 and 10. 10 workers is the primary utility configuration. 1 worker is
  diagnostic.
- **The corpus:** `corpora/wikidata.bz2`, checked by its SHA-256. It is the corpus the census
  and the emit-cost recipes use, with four copies end to end (838,800,784 decoded bytes), read
  into memory once.
  - *Distinct blocks:* results are reported per distinct block, by key id. The repeated copies
    are repeated observations, not independent ones.
  - *The decision:* it is taken on the whole decode of the four copies.
- **Rounds:** 20. Each round decodes the corpus once in each cell of {1, 10} workers × {open,
  supplied} × {cond 0, cond 1}, with 2 s of sleep before each decode. A check decode ends the
  round.
- **The order:** the arms are paired, and order-balanced across rounds.
  - *Within each (count, cond) pair, the two arms* run `open` first in even rounds and
    `supplied` first in odd ones, 10 times each. (With two arms, "all arm orders" is both
    orders.)
  - *Within each count, the two pairs* run cond 0 first when ⌊r/2⌋ is even, cond 1 first
    otherwise, 10 times each.
  - *The worker counts* are rotated every four rounds.
  - `examples/supplied_codes.rs`' `cells` gives the order, and its test checks the balance.
- **The pairing:** by round, at the same worker count and cond.

## The decision rule (frozen now, before any data)

Quoting the commission verbatim:

> **Decision rule:** compute the paired ratio B/A of whole-decoder wall time at 10 workers with the repo's usual 95% interval procedure. **ADVANCE** (the capability is worth building an acquisition for) iff the interval's upper bound < 0.90. **STOP** (close this representation/consumer) iff the lower bound > 0.95. Otherwise **INCONCLUSIVE**, reported as such, no rerun to chase it. Never lower the threshold or select blocks. The diagnostic emit-only ratio is reported but decides nothing. This is a diagnostic: it decides no production default.

**The repository's usual 95% interval procedure** is `bench/two_arm.py`'s `step`: each round's
paired ratio wall_B / wall_A over the cond-0 decodes at 10 workers, and its geometric mean with
the 95% Student's t interval on the logs (19 degrees of freedom). Every open block counts. There
is no trimming and no selection.

The verdict script prints the decision as one `DECISION:` line: ADVANCE, STOP or INCONCLUSIVE.

**VOID** means no verdict. It is declared when:
- *The machine or the files are wrong:*
  - the machine is not the i9-10900K;
  - the records are missing or malformed.
- *The store is unusable:* its `meta` line is missing, the store is empty, or it has a
  collision.
- *A decode is wrong:*
  - any decode is missing, duplicated or unexpected;
  - a decode's output is not 838,800,784 bytes;
  - a decode has no open block.
- *A round's check fails:* the check is missing or has a mismatch, no block was checked, or its
  output is wrong.
- *The blocks disagree:*
  - a block is recorded under another arm or cond than its decode's;
  - an arm-A block is supplied, or an arm-B open block isn't;
  - the blocks' (n, m, refused) multisets differ between decodes;
  - the two-ended counts differ between decodes.

**Reported, deciding nothing:**
- the 1-worker ratio;
- the diagnostic consumer ratio at both counts;
- arm B's emit with the delivery charged, (deliver + consume) / arm A's consume, cond 0;
- the cond-1 wall ratio;
- each arm's construct, deliver, condition and consume per retained row;
- the distinct blocks' own consumer ratios (min, p10, median, p90, max; and two-ended only);
- the marked cycles shorter than their retained rows;
- the peak scratch, the store's bytes and the process's peak RSS.

**Not changed after the data:** the arms, the key, the conditioning read, the grid and its
order, the block set, the lines 0.90 and 0.95, the judged count, and the interval procedure.

## What it does not decide

- **No c₁ or event-count claim.** It measures the time of a consumer that is handed the order.
  It doesn't measure how many events or dependent steps any acquisition of that order would
  need.
- **No acquisition algorithm.** How the codes would be put into output order (from retained
  records, at construction or otherwise) is outside this run. The offline discovery here is the
  walk itself, run outside the clock.
- **No global E2 conclusion.** The run bounds this representation and consumer on this corpus
  and machine. ADVANCE says that an acquisition is worth designing. STOP closes this
  representation and consumer, not the wider line.
- **No production default.** A capability built after ADVANCE goes through ordinary adoption
  under PERFORMANCE-POLICY.md, with its own registration.

## Expected runtime

About 15 minutes with the build:
- the build, about 2–3 minutes;
- the corpus check and the copies, seconds;
- the oracle pass, about a second;
- each round about 30 s: eight timed decodes (four at 1 worker of about 2 s each, four at 10
  workers of about 0.2–0.3 s), a check decode, and nine 2 s settles. Twenty rounds take about
  10 minutes.

The run's memory is the corpus (about 4 × its compressed size) and the store, about 4 bytes a
retained row of the distinct open blocks (printed by the oracle pass). That is well within the
workstation's memory.

## Implementation choices, for review before the ready tag

- **Forward blocks are supplied too.** The commission's "front/back indices" describes the
  two-ended blocks. On the open kernel's forward blocks (m < 65,536, or codes too wide to pack)
  arm B reads its codes from the front alone, as arm A's forward `emit` walks. Every open block
  is then in both arms' timing, and none is selected.
- **The key is O(1) and checked twice:** at recording, against collisions, and every round, by
  the check decode's byte comparison and code comparison.
- **The diagnostic's conditioned decodes are separate decodes in the same run.** If the
  conditioning ran in the primary decodes, arm A there would no longer be the kernel unchanged.
- **Byte-exactness is checked by the check decode and the CRCs,** not by a comparison inside the
  timed decodes, for the reason given above.
