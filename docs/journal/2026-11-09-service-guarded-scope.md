# 2026-11-09: The service core on current master, behind a narrow default (DRAFT)

**Status: DRAFT. Untimed groundwork, not a registration.** Nothing here is a performance
measurement, and no timing on the shared 4-core machine is evidence. No performance
verdict of #258's runtime (3e201246) transfers to this one: its runtime, dispatch and
wrappers have changed. Timing, qualification and production admission are HELD by the
Guardian's D067 (#145, comment 6740148831; `state/decisions.md#d067` on team-state/guardian
at d882543d); this revision makes the repairs D067 asks for and re-shows their evidence.

This entry scopes the guarded integration the Optimiser's REVIEW asks for (#145, comment
6739420751; report `state/records/reviews/inbox-6739270804/report.md` at 79f6ece9).

**Revisions:**
- 3c16b730: the first draft (port, predicate, reach tests, scoping);
- this revision (D067): guard-and-fail outer unwind (§4), fail-closed discovery (§2), the
  completion fence (§3), the service fuzz mode with fenced attribution (§5), and the
  targeted mutants (§5).

Labels:
- **checked:** by a run on the shared machine, at the pinned source (correctness only);
- **CI:** to be read from this PR's own CI run, not claimed here;
- **not shown:** no evidence on this branch.

## 1. What is integrated

**The service core's runtime.** The source is the B5 branch `safety/service-core-b5` at
fc013927, whose runtime equals #258's measured 3e201246 plus test-only wait counters (D057).
Its diff from their common base dd939518 for `src/parallel.rs`, `src/coop.rs` (`invert_team`,
its Miri test) and `src/trace.rs` (the service marks) applied cleanly to master a773853f;
`tests/service.rs` came across whole. **`src/coop.rs` is byte-identical to fc013927's**
(checked: `git diff fc013927 -- src/coop.rs` is empty), so the pool's two completed runs and
its helper-completion fence are the reviewed code.

**Changes to the ported runtime** (each a change to the measured code; none measured):
1. `ServiceCounts` (prepared, pooled, finished): plain increments under the hub lock where it
   is already held, read through `Hub::counts`.
2. `team_worker`'s pool threshold reads `min_coop(shared)` (master's #275 function; outside
   tests, `MIN_COOP`).
3. `StreamDecoder` holds the hub (`hub: Option<Arc<Hub>>`).
4. `BzReader::parallel_decoder` (hidden, for tests); `parallel_service` as on fc013927; #278's
   `CancelOnUnwind` from master.
5. **The outer guard (`Outer`), `Hub::fail` and the terminal state (D067; §4):** a guard per
   hub thread (feeder, service workers, teams), the hold it carries (`Outer::held`, set and
   cleared under the hub lock at each acquisition, hand-off and retirement), `HubState::failed`
   in the wait predicates and in `done`, a retirement on failure at the two hand-offs
   (`retired_on_failure`), the feeder's dropping of jobs after a failure, and the reader's
   terminal error (`State::terminal`, `State::stopped`).
6. **The completion witness (§3):** `HubState::exited`, counted as each hub thread's guard
   drops (one lock acquisition per thread exit), and `ServiceCounts::ended`.

The per-block cost of 5 and 6: one `Cell` store at each acquisition, hand-off and retirement
(each inside a critical section the code already had), and one `failed` check in each wait
loop and hand-off. Nothing per block takes a new lock.

## 2. The default, its predicate and its discovery

`StreamDecoder::new` builds the service core at `SERVICE_PLAN` (two teams of three, two
service workers: 8 decoding threads) when, and only when, no plan is forced with `with_plan`
and `service_target(threads, k, &Machine::here())` holds. The machine is read only at the
predicate's budget with its kernels (the cheap conditions are tested first). Everything else
is the existing dispatch, unchanged. `Plan` has no new variant.

**This is an automatic default on the supported class, not a dormant option** (D067 §5): on a
machine where the predicate holds, `new(10, ..)`, `BzReader::parallel(.., 10)` and
`decode_parallel(.., 10)` run svc2. CI's machines fail the predicate, so CI leaves it
unexercised there; its bench prices only construction and discovery off-path.

**The predicate (`parallel::service_target`), every condition required:**

| condition | value |
|---|---|
| threads asked for | exactly 10 (`SERVICE_THREADS`) |
| kernels | `avx2 && clmul` |
| platform | x86-64 Linux |
| topology snapshot (`Machine::target`, `target_topology`) | every allowed online CPU `GenuineIntel`, family 6, model 165, a 20 MiB L3; 20 of them on 10 physical cores |
| CPU quota (`Machine::quota`, `cpu_quota`) | established: none, or at least 10 CPUs |

**The class it admits** (D067 §5): family 6 model 165 with these counts is a
microarchitecture and topology class (the 10-core Comet Lake-S parts: by published
specifications the i9-10900K/KF/F/T, i9-10900, i9-10850K and 10-core Xeon W-1290 parts; not
checked on hardware), not the one i9-10900K #258 measured. **Its other members are
unmeasured, and one i9 result does not qualify them.** There is no brand-string gate: a name
would prove neither power, clock nor quota equivalence, and D067 does not ask for one. A
registration must either align the default's reach with the admitted evidence or obtain the
coverage or authority for the class; and its preflight must read the actual target back
(§9, item 4).

**Discovery fails closed (D067 §2).** The predicate reads only two facts, each established or
refused:
- **`target_topology`** (one coherent snapshot): the CPUs in `Cpus_allowed_list` that
  `/sys/devices/system/cpu/present` and `online` list; CPUs those lists exclude are
  positively offline or absent and are left out, not guessed. Each counted CPU's
  `topology/core_id`, `physical_package_id`, `cache/index3/level` (must be 3) and `size`, and
  its `/proc/cpuinfo` record (matched by `processor`) must all be read and parsed; any read
  error, a CPU with no record, or any difference among them (vendor, family, model, L3 size)
  is None. The plan policy's own readers (`physical_cores`, `l3_bytes`) are unchanged and
  are not the predicate's.
- **`cpu_quota`** (cgroup v2 only): the one `0::` entry of `/proc/self/cgroup`, the one
  `cgroup2` mount of `/proc/self/mountinfo` (its root and mount point), and every directory
  from the process's cgroup up to the mount point, each `cpu.max` parsed strictly (one line,
  `max PERIOD` or `QUOTA PERIOD`, positive integers), the tightest kept. A level's `cpu.max`
  may be absent (the controller not enabled there) only if its directory is there. The mount
  point must be the true root (no `cgroup.type`, which every non-root cgroup has); a mount
  below it (a cgroup namespace, most containers) hides ancestors whose limits can't be read,
  so it is `Unknown`. Everything else (membership unreadable, a level unreadable, an I/O
  error, an unparsable file, no v2 entry or cgroup v1 alone, several entries or mounts, a
  relocated mount, an escaped path) is `Unknown`, and `Unknown` makes the predicate false.
  `Quota::default()` is `Unknown`.

**Lookahead and window.** Lookahead `teams` (L2). Window
`window(SERVICE_PLAN.as_plan(), threads, slots)`: 8 at the wrappers' 40 slots (W8); a
caller's own scales the same way, at least a block a team (1→2, 10→2, 20→4, 40→8, 80→16,
200→40).

**Hidden test hooks:** `with_machine` (substitutes `Machine::here` on this thread, so both
the plan policy and the predicate see that machine), `observe_builds`, `is_service`, `window`,
`service_counts`, `BzReader::parallel_decoder`.

## 3. Reach, and the completion fence (checked)

**The fence (D067 §3).** CI's occupancy job 129157302740 failed `the_service_default` with
`(prepared, finished) = (4, 3)` against `(4, 4)`: `finish` publishes the last block and wakes
the reader before `retire_counted` counts it, so the reader's EOF can come first. EOF is not
the workers' completion, and nor is dropping a `BzReader` (its feeder may hold the decoder's
last reference). The repair keeps the equality and fences instead:
- every hub thread's guard counts its exit under the hub lock, after the thread's last
  retirement; `ServiceCounts::ended` is true only when all have exited;
- each test joins the decoder (`StreamDecoder::join`, or the decoder's drop inside
  `decode_parallel`) before reading the counts, and asserts `ended`.

Counts are operational snapshots: a block failing in preparing or a team's run retires
without finishing, so `finished` is not "every block retired" on error paths.

**Reach tests** (on this machine, not the target; the target's facts come from
`with_machine`, and every decoder then runs for real on 4 vCPUs):
- `plan_tests::the_service_target`: each condition flipped alone (budgets, kernels, each
  topology fact, the snapshot absent, quotas 9.5/2/unknown, the default `Machine`).
- `plan_tests::discovery_fails_closed_quota`: 18 fixtures through `cpu_quota` and the
  predicate, among them unreadable membership, an unreadable restrictive parent, an I/O
  error on a parent beneath an unlimited leaf, a restrictive parent beneath an unlimited
  leaf (8 CPUs: false), inherited quotas of 10 (true) and 12 under 15, no and a relocated v2
  mount, two mounts, cgroup v1 alone, a hidden ancestor, an unreadable root `cgroup.type`;
  and strict `cpu.max` parsing (`max garbage`, `inf`, `1e6`, extra fields and more refused).
- `plan_tests::discovery_fails_closed_topology`: a 20-CPU fixture reads as the target; an
  unreadable or missing core id, package, L3, an L3 of another size, a non-level-3 index3,
  unreadable `online`/`present`/status/cpuinfo, unparsable affinity, a CPU of another model,
  a CPU with no cpuinfo record: None. Positively offline (19 CPUs) and masked (10 CPUs)
  snapshots read as smaller machines; allowed-but-absent CPUs are left out.
- `plan_tests::the_service_default`: dispatch, custom windows, fallbacks, forced plans, and a
  decode through it with the fence (`ended`, then 4/4).
- `tests/service_default.rs`, through the four public entry points by natural dispatch:
  libbzip2's bytes, svc2 at 8 threads and window 8, `ended`, prepared = finished = blocks,
  pooled ≥ blocks − 1; custom windows; the existing plan at 9, 11 and 20 threads, on a
  same-shape Intel model 151, with an unknown quota, on a 4-core machine, and when forced;
  errors (kind, block, bit, bytes before) as the plain decoder's.
- `tests/reader_panic.rs`: #278's guard with the explicit service reader and the on-target
  default, at four panic positions.

## 4. The outer unwind: guard-and-fail (D067 §1)

**The stage catches stay.** A panic inside a stage (preparing, a team's run, finishing) is
caught there and becomes that block's `Internal` error, with the ordered prefix before it; a
partial front is discarded, never reused; a team's back set and table are replaced only after
`Pool::run` has waited for every helper. These are fc013927's, unchanged.

**What the guard covers:** everything outside those catches. Each hub thread makes an
`Outer` first, so it drops last: after every lock guard the thread held, and (in a team)
after its pool, whose drop joins its helpers. Its hold (`held`) is set when the thread takes
a block (`take_job`, `take_prepared`, a walked block's selection) and cleared when it hands
the block on (queued) or retires it, each in the same critical section as the queue's change
or `live`'s. Between publication and retirement the hold stays set, so an unwind there
retires the block once and never publishes it again.

**On an unwind (`Hub::fail`), in order:**
1. **the terminal error first:** `State::terminal` (`Internal`, naming the thread) and
   `cancelled` are set, and the ready blocks dropped, under the shared lock, before any wake;
2. **the hub fails:** `failed` set; the held block retired; every queued block (prepared,
   walked) retired and its buffers dropped, never reused; the jobs not yet taken dropped. A
   block another thread holds in a stage is that thread's to retire when its stage ends; at
   its hand-off it sees `failed` and retires it instead of queueing it
   (`retired_on_failure`);
3. **both domains woken:** the shared condition (the reader, the scanner, which then ends and
   closes the jobs, and the writers) and the hub's (every waiter sees `done`).

Counts saturate rather than panic during the unwind; the two locks are taken one after the
other, never nested; nothing in it calls a trace hook. Losing the last service worker, the
last team, the feeder, or a thread holding no block still ends the decode: the terminal
predicate (`failed` in `done` and in each wait loop) does not wait for a consumer that is gone.

**The contract (documented on `Hub::fail` and below):** an outer unwind is a terminal
failure. The reader keeps the bytes it already had and then gets the terminal `Internal`
error (or, if the failure came after it had read the whole stream, the whole stream and EOF).
**The full ordered prefix before the failure is not promised:** the failure clears `ready`,
as a cancel does. A cancel racing the failure may give `Cancelled` instead. Abort (an
out-of-memory one among them) and a forever-blocked external `Read` remain outside any
promise.

**Tests** (`src/parallel/service_unwind_tests.rs`; cfg(test) injections, `outer_fault`, fired
once):
- all nine outer points (a service worker holding nothing; a job just taken; before and after
  the prepared hand-off; a team before its pool is chosen; after the walked hand-off; a
  walked block just taken; between publish and retire; the feeder) at block 1, on one team
  and one service worker (the faulting thread the last of its kind) and on svc2's shape, on
  tiny blocks (each team's first thread alone) and full blocks (a team's pool, asserted
  reached across the full-block suite as a whole, not in each injection's run: this shows
  representative pooled execution, not that every fault fired while helpers were active,
  D069);
- a parked-state handshake (`HubState::parked`, counted under the hub lock around each wait):
  the fault fires only once every other hub thread is parked and the jobs are closed, so only
  `Hub::fail`'s wake can end them (a service worker's loss with two teams parked; the last
  team's loss with its service worker parked);
- a writer blocked on a one-block window while the reader waits, a cancel racing the fault,
  the decoder dropped rather than joined;
- each checks, under a 60-second watchdog: the fault fired; the hub failed; every hub thread
  ended; `live` 0 and as many retirements as blocks taken; no block published twice (the
  publication history read after the join or drop, so every publication before the threads
  ended; at 6642faf4 it was read before the join, D069); the
  bytes a prefix of the stream; the result EOF with the whole stream, or the terminal error
  (or `Cancelled` with the racing cancel).
- Miri (`miri_outer_after_job`, `miri_outer_after_publish`, `miri_outer_parked`): tiny blocks,
  scalar kernels, the last-of-its-kind and parked shapes.

**No reuse before helper completion:** no outer point lies inside a team's pool run (the run
is wholly inside the stage catch), the guard never touches a buffer, `Hub::fail` drops only
queued buffers (no thread is using them), and a team's guard drops after its pool. This is
argued from the source, not separately mutated.

## 5. Safety evidence: shown and not

**Targeted mutants (checked; `2026-11-09-service-guarded-scope/mutants/`: the script, each
patch and log headed by the mutated file's SHA-256, and `summary.txt`).** Each applied alone,
the outer-unwind suite run natively, the file restored byte for byte:

| mutant | obligation | result | detected by |
|---|---|---|---|
| G1 guard removed (`Outer`'s failure never runs: M12's analogue) | outer unwind fails the decode | KILLED, 7/7 tests fail | the watchdog: hangs at a held block (`AfterJob`, `AfterPublish`, `AfterPrepared` with a blocked writer), at `SvcIdle` and in every parked case |
| G2 hub wake removed from `Hub::fail` | both domains woken | KILLED, 4 fail | the watchdog: the parked handshakes (both shapes, and Miri's fixture natively), `BeforePrepared` and `AfterFinishTake` |
| G3 terminal predicate removed (`failed` out of `done` and the wait loops) | no wait for a consumer that is gone | KILLED, 3 fail | the watchdog: the feeder's loss (jobs never closed), `AfterPrepared` at svc2 |
| G4 double retirement (the guard retires whether or not it holds a block) | each block retired once | KILLED, 3 fail | `blocks taken, retired` (`SvcIdle`, a blocked writer) |
| G5 hold released at publication, not retirement | the publish→retire gap | KILLED, 4 fail | `a live block lost` (`AfterPublish`, both shapes, Miri's fixture natively, the drop case) |
| restored | | 7/7 pass | |

**The fuzz route, ported (checked).** `examples/fuzz.rs` MODE `service`
(`tests/common::service_candidates`): svc2 as the default through `StreamDecoder::new` (two
write/read sizes), `BzReader::parallel` and `decode_parallel` on the target's facts, and four
explicit shapes (one service worker in a one-block window; three teams of three in 12; a team
of four with two service workers; two teams of two on the scalar kernels). Inputs and edits
are the pipeline cases' (`pipeline_corpus`, aimed at later blocks' headers and the stream's
end); each case is held to the plain parallel decoder (bytes before any error; the error's
kind, block and bit) and to libbzip2. **Attribution is completion-fenced:** counts are read
only after the decoder is joined and `ended` holds (the run panics otherwise, and a panic is
a failure). Per candidate, cases with a block pooled, finished, with more blocks prepared
(finished) than the decoder has front (back) sets (so those sets were reused), and with an
error. Seed 2, 120 s: **2,724 cases, 0 failures** (`fuzz-service-seed2.log`); each candidate
reached pooled, finished, front reuse and back reuse in many cases (the log gives the
counts). These are new-head results, not old counts relabelled. CI's fuzz job now runs
`fuzz 30 1 .. service`. Not ported from the old overlay: per-case cancellation
(`cancel_after`); `tests/service.rs` and the outer-unwind tests cover cancellation.

**Native and Miri (checked; §8).** All native suites; Miri on the repaired source for the
service hub fixtures, the outer-unwind fixtures, the team run and the M12 fixtures.

**Reused by obligation (D067 §4):** D045's ownership and memory-ordering derivation, D053's
controlled wake/retirement evidence and H3/O3 arguments, D057's repaired fuzz/Miri evidence,
for the unchanged obligations and premises they covered (`src/coop.rs` byte-identical; the
stage catches and buffer movements unchanged). Not re-run: the inherited mutants.

**Not shown:**
- AVX2 or full-size Miri (not required by D067), and every interleaving;
- a mutation of the "no reuse before helper completion" argument (argued from the source);
- the predicate on a real i9 (§9, item 4);
- the service core is compiled into every build (dead at run time off the class, not
  removed);
- the CI results of this revision (to be read from the PR's checks).

## 6. Memory accounting, for a later registration (design; nothing measured)

The figure to register is **whole-process peak RSS**, for the exact switched default against
the exact current default, through the same public entry point, on the same input and
consumer. **Each arm and run in its own process** (`/usr/bin/time`'s maximum resident set, or
`ru_maxrss` read at that process's exit): `ru_maxrss` is a process's high-water mark and
never resets, so reading it after one decoder's drop in a process that ran another says
nothing about the second (D067 §5). Not a sum over decoding threads: the scanner, the hub's
feeder, the reader's feeder, the caller and the pool helpers are all in the process.

A working model to check the measurement against (not a proved bound; a full 900 kB block;
a high-expansion block's output is up to ~46 MB):

| item | count at svc2 | each, about |
|---|---|---|
| front sets | teams + L + services = 6 | 0.96 MB |
| back sets | 2 × teams = 4 | ~1.0–1.1 MB |
| T-vectors | 2 | 3.6 MB |
| assembly buffers | 2 | up to the block's pre-RLE1 length |
| window slots (symbols + output) | W = 8 | ≤ 1.8 MB symbols; the output's size |
| input: queued chunks + the scanner's window | | 2 × the writer's chunk + ~2–4 MB |
| thread stacks | 8 + scanner + hub feeder (+ reader feeder) | virtual; RSS as touched |

Against the current default at 10 (three pipelined teams of three, W12): one T-vector and
four window slots less; the pipelined default's front and back counts are to be read from its
source at the registration's pin. Untimed extras: a `trace` high-water census (prepared queue
bounded by L + K − 1 = 3, D045), stress inputs (high-expansion, small and mixed blocks, a slow
consumer, cancellation), and a `/proc/self/task` thread census for both arms.

## 7. The corrected diagnostic's hooks (#293, Researcher REVIEW 6739649705)

Not ported here. The hooks fit at these points, each with the lock it holds:

| event | where | lock |
|---|---|---|
| `Issued` | `scanner`, after `issued += 1` | `shared.state` |
| `Enqueued` | `scanner`, after `jobs.send` | none (channel) |
| `FeedRecv` | `feed`, after `rx.recv()` | none |
| `JobIn` | `feed`, after `jobs.push_back` | hub |
| task chosen (with P, A, C, J, free fronts and backs, and whether a preparation was eligible when a finish was chosen) | `service_worker`'s selection, at each `break` | hub |
| prepared queued; walked queued; taken by a team | `prepare`, `team_worker`, `take_prepared` | hub |
| wait entered and woken (with the predicate at each) | around each `hub.wait` (where `HubState::parked` now counts, under test) | hub |
| retire (finished, error, cancel, failure) | `Hub::retire_counted`, `retired_on_failure`, `Hub::fail` | hub |
| `SlotWait` / `SlotGot` with `issued − taken` | `scanner`'s slot wait | `shared.state` |
| `Taken` with `issued − taken` after | `StreamDecoder::read` | `shared.state` |

The port needs: (R1) sequence and timestamp taken together under the event's own lock,
cross-lock merges only along causal edges, refusal (never clamping) where timestamps go
backwards, and conservation checks; (R2) current occupancy with the slot events, the hand-off
from `Taken` to `SlotGot` labelled separately; (R3) per-choice eligibility recorded, A>0 split
from A=K; (R4) the TSV bound by digest to the run's validation and plan (from
`observe_builds`), mixed plans and incomplete logs refused. `ServiceCounts`, read after the
completion fence (`ended`), gives an independent per-run check of prepared and finished.

## 8. Test results at the pinned source (checked)

On the shared machine (Intel family 6 model 173, 4 vCPUs, AVX2 and CLMUL), at f747fe7a's
source (`src/`, `tests/`, `examples/` as committed there; this revision's later commits
change only this entry and its records):
- `cargo test` (stable rustc 1.99.1, all targets): library 73 passed, 1 ignored (as on
  master); cooperative 4, corpus 2, decode 14, in_place 7, mtf_checkpoints 3, mtf_kernels 4,
  pipelined 12, reader_panic 1, service 10, service_default 4, doc-tests 1; zero failed.
- The occupancy job's command, `cargo test --release --features trace --lib --example
  occupancy`: 73 passed, 1 ignored; 3 passed. One local pass of a race is not evidence the
  race is gone; the fence (§3) is the repair, and CI's run of this head is the read-back.
- `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --check`: clean.
- Miri (`cargo +nightly miri test --lib -- service_tests::miri_ service_unwind_tests::miri_
  coop::tests::miri_team_run_and_its_panic unwind_tests::miri_`; nightly 1.101.0 16d1ffac1
  2026-10-26, `MIRIFLAGS` unset): **11 passed, 0 failed** (the four service hub fixtures, the
  three outer-unwind fixtures, the team run, the three M12 fixtures), 311 s of Miri's clock
  (`2026-11-09-service-guarded-scope/miri.log`).
- The outer-unwind mutants: §5's table (`mutants/summary.txt`).
- The service fuzz: seed 2, 120 s, 2,724 cases, 0 failures (§5).

Wall times are the runs' own, not measurements of the decoder.

## 9. What remains before a registration

1. **The Guardian's review of this revision** against D067's closure conditions, and an
   exact-head CI read-back (native, the occupancy job, Miri, fuzz).
2. **The adoption question D067 §5 raises:** the default's automatic reach on the supported
   class against the admitted evidence, and how CI's unchanged acceptance rule applies when
   CI can't exercise svc2. These are Louis's (RULES D/C6), not settled here.
3. **The registration** (DRAFT: `2026-11-09-service-guarded-registration.md`, its recipe
   `just bench-service-guarded`, judge `bench/service_guarded.py`, preflight
   `examples/svc_target.rs`; D069 said to draft it; Researcher assesses, Coordinator registers): the exact switched
   default against the current default at 10 on the i9, CPU and per-process peak RSS (§6),
   frozen; then Louis's C5 run.
4. **An actual-target preflight in that registration:** `Machine::here()` printed on the
   workstation, `service_target(10, Kernels::detect(), ..)` true there and false at 9 and 11,
   the quota and snapshot as read, before any timed run.
5. Optionally, the corrected diagnostic (§7), separately registered.
