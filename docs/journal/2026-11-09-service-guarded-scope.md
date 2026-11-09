# 2026-11-09: The service core on current master, behind a narrow default (DRAFT)

**Status: DRAFT. Untimed groundwork, not a registration.** Nothing here is a performance
measurement, and no timing on the shared 4-core machine is evidence. This entry scopes the
guarded integration the Optimiser's REVIEW asks for (#145, comment 6739420751; report
`state/records/reviews/inbox-6739270804/report.md` at 79f6ece9) and records what this branch
(`perf/service-guarded`) integrates, what its tests show, and what remains before a
registration can be frozen.

Labels:
- **checked:** by a run on the shared machine, at this branch's head (correctness only);
- **CI:** to be read from this PR's own CI run, not claimed here;
- **not shown:** no evidence on this branch.

## 1. What is integrated

**The service core's runtime, ported unchanged in its logic.** The source is the B5 branch
`safety/service-core-b5` at fc013927, whose runtime equals #258's measured 3e201246 plus
test-only wait counters (D057). Its diff from their common base dd939518 to fc013927 for
`src/parallel.rs`, `src/coop.rs` (`invert_team`, its Miri test) and `src/trace.rs` (the
service marks) applied cleanly to master a773853f; `tests/service.rs` came across whole.
Master's changes to `src/parallel.rs` since dd939518 are #275's test-only fault hooks and
`min_coop`, and `src/read.rs` has #278's feeder guard; neither touched the code the service
core uses beyond `publish` (its test-only record of publications) and `Shared` (its
test-only `faults` field, now set in the service core's constructor too).

**Changes to the ported runtime** (each a change to the measured code, so D057's "changed
runtime" applies; none is believed to matter for speed, and none is measured):
1. `HubState::counts` (`ServiceCounts`: prepared, pooled, finished): three plain integer
   increments, each under the hub lock at a point where the lock is already held
   (`prepare`'s queueing, `team_worker`'s hand-off, and `finish`'s retirement, through
   `Hub::retire_counted`). No new lock acquisition and no atomic.
2. `team_worker`'s pool threshold reads `min_coop(shared)` (master's #275 function) in place
   of the constant `MIN_COOP`. Outside tests it returns `MIN_COOP`. Under test it lets a
   tiny block reach the pooled path.
3. `StreamDecoder` gains `hub: Option<Arc<Hub>>` (one more Arc reference on the hub, held
   by the decoder, dropped with it after `join`).
4. `BzReader::parallel_service` (the research constructor, as on the research branch) and
   `BzReader::parallel_decoder` (hidden, for tests). With #278's guard, a source panic under
   the service core cancels the decode as it does for every other parallel reader (§4).

**Not ported:** the research branches' `examples/ab.rs` arms, `examples/occupancy.rs`, the
B5 fuzz overlay (`examples/fuzz.rs`'s service candidates and `tests/common`'s `Serviced`).
Master's fuzz harness has diverged (#273's pipeline candidates), and the overlay needs a
re-port, not a mechanical merge (§5).

## 2. The default, and its predicate

`StreamDecoder::new` builds the service core at `SERVICE_PLAN` (two teams of three, two
service workers: 8 decoding threads) when, and only when, `service_target(threads, k,
&Machine::here())` holds and no plan is forced with `with_plan`. Everything else is the
existing dispatch, unchanged (`Plan::for_threads`, `window`, `defaulted`). `Plan` has no
new variant: `Plan::for_threads` still answers three teams of three on the i9 at 10, which
is the fallback, and external matches on `Plan` don't break.

**The predicate (`parallel::service_target`), every condition required:**

| condition | value | why |
|---|---|---|
| threads asked for | exactly 10 (`SERVICE_THREADS`) | the one budget measured; not "10 or more" |
| kernels | `avx2 && clmul` | the kernels the run used (`Kernels::detect` on the i9); scalar or mixed kernels keep the existing plan |
| platform | x86-64 Linux (`cfg!`) | the only topology readers |
| CPU identity | `GenuineIntel`, family 6, model 165 (`/proc/cpuinfo`'s first processor) | Comet Lake-S, the i9-10900K's CPUID; *not* inherited from the 10-core/20 MiB rule, which any CPU of that shape meets |
| physical cores in affinity | 10 | as the existing reader counts them |
| logical CPUs in affinity, online | 20 | the measured SMT shape, all of it this process's |
| L3 | 20 MiB (cpu0's index3) | as the existing reader reads it |
| cgroup v2 CPU quota | none, or at least 10 CPUs (tightest `cpu.max` from the process's cgroup up) | a quota below the budget is not the measured machine; an unparseable `cpu.max` fails the predicate |

Any fact `Machine::here` cannot read is None, and None fails the predicate. The CPU's
identity is read once (`OnceLock`); the others each call, as `physical_cores` already was.
Model 165 is shared by other Comet Lake-S parts (i7/i5-10xxx), but the core, logical-CPU
and L3 counts together exclude every one of them except the 10-core i9 parts (the
i9-10900/10900F/10900KF/10900T/10850K and the 10-core Xeon W-1290 parts share all three, as
far as their published specifications go: not checked on hardware). That is an explicit and documented reach of
the predicate, narrower than today's, but still more than one SKU: a registration should
either accept it or add a model-name check.

**Lookahead and window.** The service constructor's lookahead is `teams`, so 2 (L2). The
window is `window(SERVICE_PLAN.as_plan(), threads, slots)`, the same scaling a plan of two
teams gets: at the 40 slots `BzReader::parallel` and `decode_parallel` give at 10 threads,
8 blocks (W8). A caller's own window scales the same way, at least a block a team: 1→2,
10→2, 20→4, 40→8, 80→16, 200→40.

**Test hooks, all hidden:** `with_machine(Machine, f)` substitutes the machine's facts on
this thread (the plan policy reads them too, so the fallback is the real fallback on that
machine); `observe_builds(f)` records each decoder `StreamDecoder::new` built on this thread
during `f`, and the service core's counts once `f` returned; `StreamDecoder::is_service`,
`window`, `service_counts`; `BzReader::parallel_decoder`.

## 3. Reach (checked)

All at this branch's head on the shared machine (an Intel family 6 model 173 with AVX2 and
CLMUL: not the target, so the target's facts come from `with_machine`, and every decoder
then runs for real on 4 vCPUs).

`src/parallel.rs`, `plan_tests`:
- `the_service_target`: the predicate true at the target and false for each of: 0, 1, 4, 8,
  9, 11, 12, 16, 20, 64 and `usize::MAX` threads; scalar, AVX2-only and CLMUL-only kernels;
  9 or 11 cores, 19 or 10 logical CPUs, an L3 a byte short or 16 MiB, models 166 and 158,
  family 15, AuthenticAMD, quotas of 9.5 and 2 CPUs, an unknown quota, and each fact unread.
  True again at quotas of 10, 12.5 and 20. `cpu.max` parsing, including malformed lines.
- `the_service_default`: `StreamDecoder::new(10, 40, ..)` on the target is the service core,
  8 threads, window 8; custom windows as above; every other budget (1–20) is exactly what
  `Plan::for_threads` says on that machine, with its pipelining and window; 10 threads on
  scalar kernels is the pipelined three teams of three; an AMD CPU of the same shape is the
  pipelined three teams of three, window 12; a small machine, 10 workers; three forced plans
  win on the target. A decode through it: four streams, prepared 4, finished 4.

`tests/service_default.rs` (natural dispatch through the public entry points; each build
observed):
- `the_entry_points_reach_the_service_core`: `StreamDecoder::new(10, 40)`,
  `BzReader::parallel(.., 10)`, `BzReader::parallel_with(.., 10, 40, ..)` and
  `decode_parallel(.., 10)`, on three multi-block inputs (text at levels 9 and 3, records at
  level 6): libbzip2's bytes; the service core, 8 threads, window 8; prepared and finished
  both equal to the block count, and at least all but one block inverted on a team's pool.
- `custom_windows_scale_to_the_teams`: `new` and `parallel_with` at 1, 10, 20, 40, 80 and
  200 slots.
- `the_existing_plan_otherwise`: 9, 11 and 20 threads on the target, 10 threads on a same-
  shape Intel model 151 and on a 4-core machine, through all four entry points: not the
  service core, the existing plan's pipelining and threads, libbzip2's bytes; and a forced
  plan through all four on the target.
- `errors_through_the_entry_points`: three blocks' stored CRCs flipped and two truncations,
  through all four entry points on the target: the plain parallel decoder's error (kind,
  block, bit), and its bytes before (the three streaming entry points).

`tests/reader_panic.rs`: `a_panicking_source` adds the explicit service core and
`BzReader::parallel(.., 10)` on the target (asserted to be the service core), at the four
panic positions, including after the feeder's first 4 MiB write.

`tests/service.rs` (ported) adds svc2's own shape (2, 3, 2, 8) to its shapes; the other
seven are the research branch's.

## 4. Reconciling with #278 and M12

**#278 (the feeder guard).** The guard calls `StreamDecoder::cancel`, which the service core
shares with every decoder: it closes the input, sets `cancelled` and wakes the reader and
scanner. The service core then drains as D045 describes: the scanner sees the closed input
and ends, the hub's feeder sees the closed jobs channel and closes the hub, and every stage
sees `cancelled` (prepare and team retire, finish's `take_out` returns None). The reader's
next read returns the guard's error. **Checked:** `a_panicking_source` on both service
readers (§3). **Not shown:** the D045 caveat stands: cancellation reaches the hub only
through that scanner→feeder close, not a direct notify, so its progress depends on the
scanner not being stuck in a source.

**M12 (#275's outer-unwind recovery).** The pipelined team has an outer region: the leader's
closure runs outside its stages' catches while helpers walk, holding up to three blocks
(the held prior, the current, the staged next), and M12 is about retiring all of them when
that region unwinds. The service core's shape is different, and the mapping is:

| service-core state | buffers held | on a stage's panic (ported, unchanged) |
|---|---|---|
| preparing (a service worker) | a front, filling | caught; the partial front is *discarded*, a new one returned (never reused); `Internal` at the block's place and bit |
| prepared, queued | a front | — (no code runs on it) |
| in a team's run | a front (read only) + a back (filling) + the team's T-vector | caught, after `Pool::run` has waited for every helper; the front returned (unchanged), the back and the T-vector replaced; `Internal`; the pool kept |
| walked, queued | a back | — |
| finishing (a service worker) | a back + the worker's assembly buffer + an output buffer | caught; the back and the assembly buffer replaced; `Internal` |

There is no held prior and no staged next: no service-core thread holds more than one block,
and every stage's code that touches a block's buffers runs inside its catch. So the M12
states (current; current + next; current + held prior) have no direct counterpart, and the
partial-front rule is "discard, never reuse", as in the pipelined `stage`.

**The gap (not shown, a B5 obligation before registration):** code *outside* those catches.
In each service-core thread it is the hub bookkeeping (taking a task under the lock, the
pushes to `prepared`/`walked`, `publish`, `retire`), and in the team's loop, the stretch
between `take_prepared` and the catch. None of it should panic but on a bug (an `expect`
whose precondition is checked just before, a `live` underflow from a double retirement) or
an allocation failure (an abort, outside any promise). If it did, the thread would end
holding its block: that block is never retired, `live` never reaches zero, the reader waits
at that place, and `join` (and `Drop`) waits for the other threads, which wait for the
hub's end. That is a hang, not a wrong output, but it is the analogue of the stranding M12
guards against. Two designs, for the Guardian:
- **Outer guard per thread, retire and fail:** each service worker and team holds a drop
  guard naming the block it holds (place, bit, which buffers); on unwind it publishes
  `Internal` at that place (once: the guard is disarmed at the hand-off under the hub lock),
  retires it with fresh buffers, and, if it was the last service worker or team, cancels
  the decode so nothing waits on a thread that is gone. Tests in the shape of
  `unwind_tests`: a cfg(test) injection at each outside-catch point, a watchdog, exactly-once
  publication, and the M12-style mutant (the guard's retirement removed) killed.
- **Catch the whole iteration and continue,** as `pipelined_worker` does. Needs the same
  "held" bookkeeping, so it is not simpler, and a thread that keeps going after an
  unexplained panic in bookkeeping is harder to argue for.

Neither is implemented here: it changes runtime code D057 reviewed, and it should be one
reviewed change with its own tests rather than folded into the port.

## 5. Safety evidence: shown and not

**Shown (checked, at this head):**
- `cargo test` (every library and integration test; §8 lists the result), including the
  ported `tests/service.rs` (corpora, every level, multi-stream, malformed CRC, primary
  index, truncation, move-to-front errors, injected panics in a helper, a team's first
  thread, preparing and finishing, cancellation and a dropped reader, all under a watchdog,
  byte for byte and error for error against the plain decoder) at its eight shapes.
- Miri, locally (nightly 2026-10-26): the four `parallel::service_tests::miri_*` fixtures
  and `coop::tests::miri_team_run_and_its_panic` (§8).
- The Miri workflow's selection check now fails if those fixtures are dropped by a rename.

**Carried over, by scope only:** D045/D053/D057's ownership and memory-ordering argument,
the wake and retirement fixtures, the targeted mutation records and the repaired fuzz
reach are for 3e201246/fc013927's runtime. The ported runtime is that code with §1's four
changes. Whether that reuse is accepted is the Guardian's call; this entry claims no more
than the source identity and the passing tests.

**Not shown:**
- **The fuzz route.** The B5 fuzz overlay is not ported. Its re-port onto master's
  `examples/fuzz.rs` (beside #273's pipeline candidates) should select the default through
  `with_machine` + `BzReader::parallel(.., 10)` and `decode_parallel(.., 10)`, as well as
  the explicit shapes, and count reach with D057's fenced collector.
- **The outer-unwind gap** (§4), and its mutant.
- **Mutation testing** of anything new here (the predicate, the window scaling, the
  counters). The predicate's tests flip each condition alone, which is a mutation-shaped
  check of each clause, but no mutant was run.
- **AVX2 under Miri**, full-size Miri, or every interleaving: as before, the Miri fixtures
  are tiny blocks on the scalar kernels.
- **Feature-off code removal:** the service core is compiled in every build (as on the
  research branches); with no predicate true it is dead code at run time, not removed.
- **The predicate on a real i9:** no test here ran on the target. The facts' readers are
  checked on this machine only (`the_policy_here`); `cpu_id`, `logical_cpus` and
  `cpu_quota` need a read-back on the workstation (a printout of `Machine::here()` and
  `service_target(10, detect)` belongs in the registration's preflight).

## 6. Memory accounting, for a later registration (design; nothing measured)

The figure to register is **whole-process peak RSS** (`/usr/bin/time`'s maximum resident
set, or `getrusage(RUSAGE_SELF).ru_maxrss` read by the harness after the decoder's drop),
for the exact switched default against the exact current default, through the same public
entry point, on the same input and consumer. Not a sum over decoding threads: the scanner,
the hub's feeder (`mibz2-svc-feed`), the reader's feeder (`mibz2-read`), the caller and the
pool helpers are all in the process, and the eight "decoding threads" are two leaders, four
helpers and two service workers.

What the svc2 default holds, as a model to check the measurement against (block-size
figures for a full 900 kB block; a high-expansion block's output is up to ~46 MB):

| item | count at svc2 | each, about |
|---|---|---|
| front sets (`ll` + counts + checkpoints) | teams + L + services = 2 + 2 + 2 = 6 | 0.96 MB |
| back sets (segments, bitmap, starts; or `direct`) | 2 × teams = 4 | ~1.0–1.1 MB |
| T-vectors | one a team: 2 | 3.6 MB |
| assembly buffers | one a service worker: 2 | up to the block's pre-RLE1 length |
| window: symbols + decoded output, retained | W = 8 slots | 1.8 MB symbols at most; the output's size (JSON ~1 MB; ≤ ~46 MB) |
| the reader's free list and current output | ≤ W | as above |
| input: queued chunks + the scanner's window | 2 × the writer's chunk + ~2–4 MB | |
| thread stacks | 8 + scanner + hub feeder (+ reader feeder) | virtual; RSS as touched |

Against the current default at 10 (three pipelined teams of three, W12): three T-vectors
and twelve window slots, against svc2's two and eight, so one T-vector and four slots less;
the front/back sets on each side need the pipelined default's own counts, read from its
source at the registration's pin (not tallied here). #258's corpus medians (about 64 MB
against 74 MB) agree in direction, for the old runtime on that corpus only, and are not a
worst-case bound.

How a registration would measure it, untimed or alongside the timed comparison:
1. **Peak RSS per run**, both arms, every run, recorded beside wall and CPU (no new timed
   work). Report the paired difference with an interval, as the CI bench does.
2. **A high-water census in an untimed build** (`trace` feature): the service marks already
   carry sizes (`Prepared`/`Walked` queue lengths; `RleEnd`'s output length and capacity),
   and the window marks the scanner's buffer. A census pass reports the maximum live fronts,
   backs, prepared queue (bounded by L + K − 1 = 3, D045), walked queue, and retained output
   capacities. Untimed: the marks perturb the schedule.
3. **Stress inputs, untimed:** high-expansion blocks (runs-only, so outputs near 46 MB),
   small and mixed blocks, a slow consumer (the reader sleeping between reads, so the window
   fills), cancellation mid-stream. Peak RSS of each against the current default. This is
   where "W12 admits four extra blocks" and "L4 adds fronts" would show up, if those shapes
   were ever proposed; at svc2 (L2/W8) they are not in play.
4. **Thread census:** the live thread count of the process during the decode (procfs
   `/proc/self/task`, as D057's collector does), both arms, so "one fewer decoding thread"
   is not confused with one fewer process thread (svc2 adds the hub feeder; the pipelined
   default has none).

## 7. The corrected diagnostic's hooks (#293, Researcher REVIEW 6739649705)

The Researcher asks that #293's service-starvation diagnostic be corrected and ported to
this guarded integration rather than run on the old runtime. It is **not** ported here.
This integration keeps every hand-off in one place, so the hooks fit at these points, each
named with the lock held:

| event | where | lock held |
|---|---|---|
| `Issued` | `scanner`, after `issued += 1` | `shared.state` |
| `Enqueued` | `scanner`, after `jobs.send` | none (channel) |
| `FeedRecv` | `feed`, after `rx.recv()` returns | none |
| `JobIn` | `feed`, after `jobs.push_back` | hub |
| task chosen (`Prepare`/`Finish`), with the choice's inputs: P (prepared), A (preparing), C (finishing), J (jobs), free fronts and backs, and whether a preparation was eligible when a finish was chosen | `service_worker`'s selection loop, at each `break` | hub |
| preparation queued; walked queued; block taken by a team | `prepare`, `team_worker`, `take_prepared`, at the push/pop | hub |
| wait entered / woken (per thread, with the predicate that held at entry and at wake) | before and after each `hub.wait` | hub |
| retire (and the kind: finished, error, cancel) | `Hub::retire_counted` | hub |
| `SlotWait` / `SlotGot`, with `issued − taken` at both | `scanner`'s slot wait | `shared.state` |
| `Taken` (credit released), with `issued − taken` after | `StreamDecoder::read` | `shared.state` |

What the port needs, by the review's defects:
1. **Ordering (R1).** Take the sequence number and the timestamp together under the lock
   the event already holds (the hub's or `shared.state`'s), so each lock's events are
   totally ordered in both; merge across the two locks only where a causal edge exists
   (`Enqueued` → `FeedRecv` → `JobIn`; `publish` → `Taken`), and refuse, rather than clamp,
   duration attribution where the merged timestamps go backwards. Check that each wait's
   attributed parts sum to its endpoints and the scanner's partition to the span. There are
   only two locks on these paths, so no global logging lock is needed; that keeps the
   diagnostic's perturbation to the per-event cost.
2. **Credit (R2).** Record `issued − taken` with `SlotWait`, `SlotGot` and `Taken`, so
   "window full" is a current-occupancy fact (`issued − taken >= slots`), and the interval
   from a `Taken` that frees credit to the scanner's `SlotGot` is labelled wake/hand-off.
3. **Finish priority (R3).** The task-chosen event records whether a preparation was
   eligible at that choice (jobs and a free front, P below L), so a finish-over-prep decision
   is a recorded fact per wait, not inferred from occupancy. Split A>0 from A=K.
4. **Admission (R4).** Bind the analysed TSV by digest to the run's validation output and
   its plan (svc2, L2, W8 from `observe_builds`' record), refuse mixed plans, and treat an
   incomplete log as failed evidence. `ServiceCounts` gives an independent per-run check of
   prepared/finished against the log's counts.

## 8. Test results at this head (checked)

On the shared machine (Intel family 6 model 173, 4 vCPUs, AVX2 and CLMUL), stable rustc
1.99.1, at this branch's source (the commit that adds this entry; this entry's own text
changed nothing compiled):
- `cargo test` (all targets): library 64 passed, 1 ignored (as on master); cooperative 4,
  corpus 2, decode 14, in_place 7, mtf_checkpoints 3, mtf_kernels 4, pipelined 12,
  reader_panic 1, service 10, service_default 4, doc-tests 1; zero failed. 3 min 45 s.
- `cargo clippy --all-targets --all-features -- -D warnings`: clean. `cargo fmt --check`:
  clean.
- `cargo +nightly miri test --lib -- service_tests::miri_ coop::tests::miri_team_run_and_its_panic`
  (rustc 1.101.0-nightly 16d1ffac1 2026-10-26, `MIRIFLAGS` unset): 5 passed, zero failed,
  224 s of Miri's clock. The full `-- miri_` set CI runs was not run locally; read it from
  this PR's Miri job.

Wall times above are the test runs' own, not measurements of the decoder.

## 9. What remains before a registration

In order, roughly:
1. **The outer-unwind guard** (§4), its injection tests and its mutant: the one runtime
   change this integration still needs. Small in code (a guard and a few hand-off points),
   but it is unsafe-adjacent hub bookkeeping and needs its own Guardian review.
2. **The B5 fuzz overlay re-ported** onto master's harness, through the default entry points
   and the explicit shapes, with D057's fenced reach counts; a bounded run.
3. **The Guardian's scope decision** on reusing D045/D053/D057's evidence for the ported
   runtime with §1's changes, and an exact-head CI (native, Miri) read-back.
4. **The predicate's read-back on the i9** (`Machine::here()` printed, the predicate true at
   10 and false at 9/11), as part of the registration's preflight.
5. **The CI bench** on this PR: the predicate is false on CI, so the bench prices only the
   off-path cost (the `Machine::here` reads at construction; nothing per block). A null
   there is not a win (the Optimiser's §4).
6. **The registration itself** (Researcher assesses, Coordinator registers): the guarded
   switched default against the current default at 10 on the i9, CPU and peak RSS (§6),
   frozen; then Louis's C5 run.
7. Optionally, the corrected diagnostic (§7), separately registered.

An honest estimate: items 1–3 are one to two focused passes of implementation and review
work; item 6 cannot start before them.
