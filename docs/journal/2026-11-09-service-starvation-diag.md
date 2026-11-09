# The service core's starvation diagnostic (untimed, opt-in)

**DRAFT.** 9 November 2026. Branch `research/service-starvation-diag`, from
`research/service-lookahead` at `8a0fc8e0`. This is the diagnostic the Researcher recommended in
REVIEW 6736071231 on #145. Its report is `state/reports/service-starvation-6735725456/report.md`
and `witnesses.py`, at `d6f52bdd`.

Nothing here is timing evidence. The local shares below come from this 4-core shared machine,
which runs svc2's eight decode threads oversubscribed. They are indicative only, and they are
not the i9.

## The question

In #271's workstation run, every window-8 trace showed teams waiting on "prep starvation"
(`2026-11-09-service-lookahead-result.md`; the correction is PR #289). But `TeamIdleStart`
labels a whole wait by the state it began in. If P = 0 at entry, the whole wait counts as prep,
even when a prepared block arrives and the team then waits for a back set. So the label can't
say what the team was actually waiting on. The report names two mechanisms that survive without
saturated average workers:

1. **Finish-first starvation.** With P = 1, Q = 2 and J > 0, both service decisions choose a
   finish, because rule 2 comes before rule 3. A team then takes the last prepared block, and the
   other team waits while both services are inside non-preemptive finishes.
2. **Window-credit hold.** The scanner admits while issued − taken < W. A ready result releases
   no credit; only the reader's ordered take does. So results that are ready behind an earlier
   missing block hold credits, the jobs run dry, and the teams wait.

## What is logged

The Cargo feature `svc-diag` (`src/diag.rs`) is off by default. Every hook in `src/parallel.rs`
is inside `diag!`, which compiles to nothing without the feature, so no timed build contains it.
The hooks don't change what the decoder does: they take no lock of their own and alter no
scheduling decision. The hub's extra counters (A, C and U) exist only with the feature.

Each event has a process-wide sequence number, taken inside the lock whose state the event
reports, so the merged order agrees with each lock's own order. It also has a monotonic
timestamp, the decoder's id, the block's place, two arguments, and a snapshot.

- **Hub transitions,** under the hub's lock, carry the hub's snapshot: J, P, Q, A (preparing),
  C (finishing), U (walking), the free front and back sets, the live blocks and whether the jobs
  are closed. The events are:
  - the feeder's `JobIn` and `Closed`;
  - each service decision, `SvcPrepare` (rule 1, P empty, or rule 3, below the threshold, with
    the block's symbol count) or `SvcFinish`;
  - each service wait and recheck, with why it can't prepare (no job, no front, or the
    threshold), and its exit;
  - `Prepared`, with rows and the front's capacity;
  - each team's wait (with `TeamIdleStart`'s entry label), recheck, `TeamGot`, `Walked` and exit;
  - each retire, credited to the task the thread had under way (`PrepRetire`, `FinishRetire`,
    `WalkRetire`). A retire from no task is logged as `RetireUnowned`, an accounting defect.
- **Window transitions,** under the shared state's lock, carry issued, taken, the ready count
  with its least and greatest places, the end and W. The events are:
  - the scanner's slot wait, recheck and slot got;
  - `Issued` (with the block's symbol count) and `ScanEnd`;
  - each `Publish`;
  - the reader's wait, recheck, `Taken` and end;
  - `Cancel`.
- **Lock-free events:** the scanner's input waits, its `Enqueued` after the send, and the
  feeder's `FeedRecv`.

Buffer ownership can be checked at every hub event through two identities:
F = free_front + A + P + U and 2T = free_back + U + Q + C. Each thread writes to its own
buffer, and the buffers are merged by sequence number once the decoder is dropped.

`diag::check` validates a clean decode's log:
- one `Config` event;
- sequence numbers unique, and increasing within each thread;
- both identities hold at every hub event, and there is no unowned retire;
- every place passes through every stage exactly once, in causal order. `Enqueued` only has to
  follow `Issued`, because the feeder can receive before the scanner records it, and
  `FinishRetire` only has to follow `Publish`, because the reader can take first;
- each thread ends once, the hub is empty at the end, and the window is drained.

## The attribution rules (bench/svc_diag.py)

The log is replayed in sequence order. The state is constant between two consecutive events, so
every team wait is cut at every event, and each interval is classified by the first rule that
holds:

1. `startup`: the team hasn't yet taken its first block.
2. `drain`: the jobs are closed and none is left (J = 0).
3. P > 0: `other:handoff` if a back set is free (the team is being woken), `other:no_back` if
   not (finishing holds the backs).
4. P = 0 and A > 0: `prep_in_progress`.
5. P = 0, A = 0, J > 0 and C = K: **`finish_first` (mechanism 1).**
6. P = 0, A = 0, J > 0 and C < K: `other:service_handoff`.
7. P = 0, A = 0, J = 0:
   - `other:feeder_lag` if a block has been issued but isn't yet in the hub's queue;
   - else, with the scanner waiting for a credit:
     - **`window_credit` (mechanism 2)** when a later result is ready and the reader's next block
       is missing;
     - `other:reader_lag` when the reader's next block is itself ready;
     - `other:window_in_flight` when nothing is ready;
   - otherwise `other:scan_or_input`.

The report gives:
- each class's share of total team wait time, and its share with startup and drain removed;
- counts of intervals and of waits that touch each class;
- the four deficit states (P = 0 or not, B = 0 or not) across all waits, beside the old entry
  label;
- for `window_credit`, the stage of the reader's next block (the earliest missing one);
- the service decisions, and how many finishes were chosen while a below-threshold preparation
  was eligible (the decision mechanism 1 needs);
- the scanner's time by state, and the defects.

Counts establish occurrence, not recovered wall time. An interval ends at the next event of any
thread, so interval counts depend on how busy the log is. The count of waits is the steadier one.

## The witnesses (bench/test_svc_diag.py)

The tests replay the report's abstract witnesses as event logs. A small simulator builds each
log with the source's priority (witnesses.py's `choose`) and checks both identities at every
snapshot:
- **Finish priority:** from P = 1, J = 1, Q = 2, both decisions are finishes. A team takes the
  last block, and the other team's wait is attributed `finish_first` (5 units) and then
  `prep_in_progress` (3 units). Two finishes are counted over an eligible prep.
- **Nine units:** the entry label says prep for all 9 units. The deficit states give 1 unit of
  P = 0, B = 0 and 8 units of P > 0, B = 0, attributed as 1 unit `prep_in_progress` and 8 units
  `other:no_back`.
- **Equal marginals:** in both cases each service is busy 7 units in 10. The selected starvation
  unit is `finish_first` in case a and `other:scan_or_input` in case b.
- **Front availability:** across the 5,040 states, no eligible preparation lacks a front, and at
  least two fronts are free.
- **The report's deterministic checks:**
  - a full window with later results ready while the earliest missing block is finishing gives
    `window_credit`, with that block's stage recorded as finishing;
  - feeder lag;
  - startup and drain are kept separate.
- **Fixed-split bound:** this is arithmetic, not a classification, and the tests check it
  (400 pairs).

On the Rust side:
- the event log's unit tests cover merge order against a lock's order, no lost events across
  four threads and two decoders, and `check` failing on each dropped, duplicated or swapped
  event and on a lost front;
- `parallel.rs` has a test that the hooks compile to nothing without the feature;
- `tests/svc_diag.rs` runs a small svc2 decode. Its log is complete, its bytes match the default
  decoder's and the input, and with a panic injected into a finish, a walk or a preparation,
  every buffer is still accounted for.

## Local untimed shares (indicative, not the i9)

This ran on this agent's 4-core shared machine:
- the bench's Wikidata corpus, its SHA-256 checked against `bench/wikidata.json` (`82f65ace…`);
- `examples/svc_diag` at svc2 (2 × 3 + 2, W = 8, L = 2), a release build with `svc-diag`;
- 3 runs, each 431 blocks and 209,700,196 bytes, identical to the default decoder's, with every
  log checked complete.

Here svc2's 8 decode threads, plus the scanner, the feeder and the reader, share 4 cores, so
descheduling dominates. These figures say nothing about the i9's schedule.

| class | share of team wait | without startup/drain | waits touching it (of 321) |
|---|---|---|---|
| finish_first (mechanism 1) | 0.000 | 0.001 | 3 |
| window_credit (mechanism 2) | 0.030 | 0.033 | 25 |
| prep_in_progress | 0.291 | 0.319 | 259 |
| startup | 0.073 | | 6 |
| drain | 0.015 | | 7 |
| other | 0.591 | 0.648 | 308 |

- **"Other", by detail:** handoff 0.222, feeder_lag 0.127, service_handoff 0.067,
  scan_or_input 0.060, reader_lag 0.059, no_back 0.052, window_in_flight 0.004.
- **Deficit states:** P0B0 0.089, P0B1 0.636, P1B0 0.052, P1B1 0.223.
- **Entry label:** prep 0.898, back 0.102.
- **window_credit, the reader's next block's stage:** finishing 0.53, walked 0.39, walking 0.08.
- **Decisions over 3 × 431 blocks:**
  - 538 empty-P preparations, 755 below-threshold preparations, and 1,293 finishes;
  - 560 of the finishes were chosen over an eligible below-threshold preparation;
  - service waits: 746 for no job, 640 at the threshold;
  - scanner slot waits: 573.
- **Time shares:**
  - teams waiting: 0.104 of the teams' time;
  - service workers preparing or finishing: 0.24 to 0.30 each;
  - the scanner waiting for a slot: 0.78 of the span.
- **Defects:** none. Both identities held at every hub event, there was no unowned retire, and no
  eligible preparation lacked a front.

Read only as an existence check: both mechanisms occur here, and both are rare. Mechanism 1
(3 waits) is far rarer than the decision it needs, which happened 560 times. Mechanism 2 touched
25 waits. Here a team's wait is mostly a positive-ready handoff and feeder lag, which is what
oversubscription would produce. On this machine the window is full most of the time, behind
team walks, not preparation. The entry label would have called 0.898 of the wait prep, while
P = 0 holds for 0.725.

## What a frozen workstation trace would need (Louis's C5 run)

Nothing is proposed for running yet. A diagnostic recipe would first be registered, reviewed and
frozen, and run only through RULES C5 with `just submit`. It would need:

- **the build:** `cargo build --release --features svc-diag --example svc_diag`, at a frozen
  commit, on the i9. It is never a timed arm and never shares a run with timed arms, since the
  feature adds a sequence counter to every logged transition.
- **the plan:** svc2@10 at W = 8, L = 2, the a3, a4 and w12 controls, and the a4-w12 joint arm.
  Teams 2 × 3 and services 2 come from `bench/service_lookahead.py`'s table, on the registered
  corpus (`82f65ace…`), with each run's bytes checked against the default decoder's.
- **the rounds:** several logs per arm (the six traced rounds of #271 would be the precedent),
  each passing `diag::check`, and the analyser's JSON beside each TSV, in one bundle.
- **a pre-registered branching decision,** as the report sets out:
  - recurrent `finish_first` with eligible jobs → the in-flight-aware prep reserve;
  - gaps while both services prepare → prep cost and locality;
  - `window_credit`, or J = 0 with a full window → the retirement/credit path;
  - P > 0, B = 0 → finish/back pressure;
  - P > 0, B > 0 → handoff.
- **its limits, stated in advance:**
  - this is a description, not a judged comparison;
  - counts establish occurrence, and interval times are indicative of exposure, not recovered
    wall time;
  - shares of overlapping waits aren't added, and none is subtracted from the run's wall time.

## Not done

- No front identity, last owner or capacity-growth tracking per front set. The report asked for
  these to price front-buffer cost. `Prepared` carries the front's `ll` capacity and rows, and
  `SvcPrepare` the symbol count, but no per-front id.
- No per-block CPU or schedstat. Time-bearing data on the i9 would need an authorised trace.
- No notify/wake accounting beyond the rechecks logged after each wake.
