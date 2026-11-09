# The service core at two teams: the registered run's result

9 November 2026. This is the run registered in `2026-11-08-service-2x3-registration.md`
(#258), frozen at 7aeb5bf9 after the revision for the Researcher's HOLD 6729358962.

## The run

- **Bundle:** `service-2x3/20261109T010708Z.tar.gz` on `evidence` at 4fccc02b.
  - Its SHA-256 is 8f21fb1f551d3845b0ec25e429fa4565222329c9d1ac5800130f9631b197aa7f, which
    matches the notice on #145 and was checked again for this entry.
  - It was run at 3e201246 with `just submit`, its frozen inputs at 7aeb5bf9 (`frozen.txt`).
  - The machine was the i9-10900K: 1 socket, 10 cores, a 20 MiB L3, so `mibz2` is the 0.2.6
    pipelined default.
- **VALID:** every check passed.
  - The noise control N1 (team3-twin@10 against team3@10) came out at 0.997 [0.971, 1.023].
  - 20 rounds in every cell, each cell's plan as registered, and no other cell.
  - Every run gave the same bytes as the reference (`a8830caee5c511a6`), the registered
    209,700,196 bytes, from the registered corpus (82f65ace…).
  - The trace was complete and as registered: 6 rounds, 431 blocks, at the run's commit.

Speed is the reference's wall over the arm's. CPU is the reference's CPU over the arm's, so
above 1 the arm spends less.

## The registered comparisons (judged)

| | arm | against | decode threads | speed | outcome | CPU | outcome |
|---|---|---|---|---|---|---|---|
| P1 | svc2@10 | mibz2@10 (the default) | 8 / 9 | 1.045 [1.017, 1.073] | SMALL GAIN | 1.146 [1.123, 1.170] | CPU SAVING |
| C1 | svc2@10 | team2x3@10 (plain 2 × 3) | 8 / 6 | 1.402 [1.368, 1.437] | GAIN | 0.941 [0.924, 0.959] | CPU COST |
| P2 | pipe2x3@10 | mibz2@10 | 6 / 9 | 0.866 [0.851, 0.881] | REGRESSION | 1.214 [1.196, 1.233] | CPU SAVING |

- **P1's CPU:** the ratio 1.146 [1.123, 1.170] is the default's CPU over svc2@10's. So
  svc2@10 spends 1/1.146 = 0.873 of the default's CPU: 12.7% less, with an interval of about
  11.0–14.5% less (1/1.123 and 1/1.170). Put the other way, the default spends 14.6% more.
- **C1's extra-thread label:** BEYOND ITS THREADS (its lower bound 1.368 is above 8/6 = 1.333).
  As registered, this label is reported, not required. C1's extra threads are the service
  threads themselves.
- **The findings, each its own:**
  - P1: SVC2@10 BEATS THE DEFAULT (SMALL GAIN; CPU SAVING).
  - C1: THE SERVICE BUNDLE BEATS THE PLAIN 2 × 3. It measures the bundle (the two service
    threads, the serial work they take, the feeding, the hub's synchronisation and hand-offs),
    and no one mechanism.
  - P2: TWO PIPELINED TEAMS DON'T BEAT THE DEFAULT.
- **Overall, by the registered order:** FOR SVC2@10 (step 6: P1 a speed gain without a CPU
  cost, and C1 a gain; S1 shows svc2@10 faster than pipe2x3@10, as the revision requires).

**The decomposition** (geometric means over the 20 common paired rounds; it is exact on the
means, and the intervals' endpoints don't multiply):

| | P1 | S1 | P2 | S1 × P2 | C1 | S2 | C1 × S2 |
|---|---|---|---|---|---|---|---|
| speed | 1.0448 | 1.2066 | 0.8659 | 1.0448 | 1.4021 | 0.7451 | 1.0448 |
| CPU | 1.1459 | 0.9438 | 1.2141 | 1.1459 | 0.9413 | 1.2174 | 1.1459 |

## Reported, never judged

| | arm | against | decode threads | speed | CPU |
|---|---|---|---|---|---|
| S1 | svc2@10 | pipe2x3@10 | 8 / 6 | 1.207 [1.171, 1.243] | 0.944 [0.921, 0.967] |
| S2 | team2x3@10 | mibz2@10 | 6 / 9 | 0.745 [0.733, 0.757] | 1.217 [1.199, 1.236] |
| S3 | pipe2x3@10 | team2x3@10 | 6 / 6 | 1.162 [1.148, 1.177] | 0.997 [0.984, 1.010] |
| S4 | team2x3@10 | team3@10 | 6 / 9 | 0.801 [0.784, 0.819] | 1.173 [1.144, 1.203] |
| S5 | svc2@10 | team3@10 | 8 / 9 | 1.124 [1.086, 1.162] | 1.104 [1.071, 1.138] |
| R1 | mibz2@10 | team3@10 | 9 / 9 | 1.076 [1.060, 1.092] | 0.964 [0.948, 0.980] |

**Medians:**

| cell | threads (leaders + helpers + service) | window | wall s | CPU s | peak RSS MB |
|---|---|---|---|---|---|
| mibz2@10 (the 0.2.6 default) | 3 + 6 + 0 | 12 | 0.144 | 1.026 | 74 |
| svc2@10 | 2 + 4 + 2 | 8 | 0.137 | 0.895 | 64 |
| team2x3@10 | 2 + 4 + 0 | 8 | 0.193 | 0.847 | 52 |
| pipe2x3@10 | 2 + 4 + 0 | 8 | 0.166 | 0.850 | 57 |
| team3@10 | 3 + 6 + 0 | 12 | 0.153 | 0.985 | 67 |
| team3-twin@10 | 3 + 6 + 0 | 12 | 0.154 | 0.983 | 67 |

The model before the run predicted the plain 2 × 3 at 0.180–0.188 s, C1 near 1.33, and P2
at 0.82–0.85. The plain 2 × 3 came out slower (0.193 s), C1 higher (1.40), and P2 a little
better (0.866). Every prediction was in the right direction.

## The trace

`occupancy-t10-svc2`, svc2 at 10 threads, window 8, medians over 6 complete rounds:

| figure | value |
|---|---|
| service utilisation (each; max) | 0.70, 0.67; max 0.70 |
| `team.idle_prep_share` (teams idle waiting for a prepared block) | 0.165 |
| `team.idle_back_share` (teams idle waiting for a back set) | 0.015 |
| prepared-queue wait, median / p90 | 0.139 / 0.594 ms |
| finishing-queue wait, median / p90 | 0.056 / 0.381 ms |
| a job's wait in the hub, median / p90 | 0.158 / 0.654 ms |
| backlog, prepared / finishing (mean; max) | 0.76 / 0.48 (3; 4) |
| the teams' run wait | 87.3 ms |

- **The registered counterexample:** NOT SATURATED at 2 × 3 + 2 (utilisation max 0.70,
  against the 0.90 threshold), with PREP STARVATION (0.165, against 0.10) and no finish
  backlog.
- **The registered reading:** prep starvation without saturation is consistent with the
  prepared queue's lookahead binding. It is not shown to be it.
- **Two facts bear on that reading,** neither tested here:
  - In `src/parallel.rs` the prepared queue's lookahead is `teams`, two blocks at 2 × 3,
    fixed. It doesn't move with the window (`slots`), which bounds the blocks in flight.
  - Jobs waited 0.158 ms (median) in the hub before a service worker took them. Supply from
    the scanner was there at least some of the time the teams starved. That points away
    from the window, and towards how the service workers order their work (finishing
    before preparing beyond an empty queue) or the lookahead itself.

## What it shows, and what it doesn't

**Established, for this hardware and workload:**
- svc2@10 is faster than the 0.2.6 default at 10 threads: 1.045 [1.017, 1.073]. That
  replicates the service core's reported S4 (1.045 [1.023, 1.068]) in a run registered to
  judge it.
- It does so on fewer decode threads (8 against 9), 12.7% less CPU (about 11.0–14.5% less),
  and 10 MB less peak RSS.
- Two teams without the service don't do it, whether plain (S2 0.745) or pipelined (P2 0.866).

**Not established:**
- **The cause.** C1 is the service bundle against the plain 2 × 3: the two service threads,
  the serial work moved onto them, the feeding, and the hub's hand-offs, together. Which of
  them earns the 1.40 isn't separated.
- **Whether the lookahead binds.** The trace is consistent with it, and doesn't show it.
- **Beyond the i9-10900K and the wikidata sample.** Nothing here measures another topology,
  another corpus, or CI's runners.

**Nothing is adopted by this run.** The service core stays an opt-in prototype on
`research/service-core` (#251), not wired into the default. Guardian D042's scoped
production-safety HOLD on the composed pipeline is untouched; this run can't discharge it.

## Next

- **The follow-up the trace points to,** registered and frozen before any run:
  `research/service-lookahead`, svc2@10 at windows 8, 12 and 16
  (`2026-11-09-service-lookahead-registration.md`).
- **A B5 review of the service core** (Guardian, staged), before any default.
- **The production-qualification plan,** below.

## Qualifying svc2 for production on the ten-core topology: the plan

No code. This is what a default change to svc2 would need, in order. Each step can stop it.

1. **A Plan rule.** The default today is chosen by `plan_with(threads, phys, l3)`. On the
   ten-core topology (10 physical cores, a 20 MiB L3) at 10 or more threads it is
   `PIPELINED_PLAN`, 3 × 3, which `StreamDecoder::new` runs pipelined with park and small.
   The rule's shape:
   - A new plan variant, `Plan::Service { teams: 2, k: 3, services: 2 }`, which
     `StreamDecoder::new` builds as the service core.
   - It is selected exactly where the measurement was made: the ten-core topology at a
     budget of exactly 10. That is not "10 or more", since 12 was measured only as svc2@12,
     a trade (`2026-11-08-service-core-result.md`, C4).
   - Its window as measured, 8 (`SLOTS_PER_THREAD` a team), unless the lookahead run changes
     it.
   - Every other machine and budget keeps today's plan, as `PIPELINED_PLAN` does now.
   - Unit tests at `plan_with`'s level, as for `PIPELINED_PLAN`: the rule fires only there,
     and no plan exceeds the threads asked for. `Plan::threads()` would count the service
     threads.
2. **CI's paired bench (RULES C3, C5).** Adoption into master goes through CI's A/B, as
   always. CI's runners aren't the ten-core topology, so the rule never fires there.
   - CI shows the change is inert off the topology: no regression at 1, 2 and nproc, and
     parity.
   - It can't show the gain. That needs the workstation.
3. **A workstation confirmation.** A registered, frozen recipe that runs the production
   build's default (`BzReader::parallel` at 10 threads, the rule wired in) against the 0.2.6
   release, not the research arm. It checks that the gain survives the production path,
   the default's own window and the release build, with N1 as its noise control.
4. **A B5 review of the service core** (Guardian, staged). The service core brings new
   concurrency: the hub's two queues, the front and back sets' bounds, the service workers'
   priority, cancellation, the panic path with its replaced buffers, and the end-of-stream
   flush. Its own `unsafe` code, if any, needs a written safety argument. The review is
   adversarial, by an agent other than the implementer, and the Guardian may HOLD it.
5. **The D042 fuzz evidence.** Guardian D042 holds the composed pipeline's production
   qualification pending reached-path fuzz and mutation evidence. A default that runs the
   service core needs the same for its own paths: fuzzing that reaches the service workers'
   stages, the hub's hand-offs, small blocks and errors, with coverage showing they are
   reached, and mutation testing of its checks. Matching hashes and green CI don't
   substitute for it.

**The rule's risks:**
- **Topology detection.** The rule fires on any CPU that reads as 10 physical cores and a
  20 MiB L3, and only the i9-10900K has been measured.
  - Another CPU with that topology gets svc2 unmeasured, as it gets 3 × 3 today.
  - `physical_cores` and `l3_bytes` read Linux's topology alone. A misread (cgroup limits,
    SMT, a VM that reports its host) picks the wrong plan.
  - The rule should key on the threads the process may run on, not the machine's count.
- **Fewer decode threads.** svc2 decodes on 8 threads where the default uses 9, and leaves
  its service threads' work serial within each block.
  - It depends on the service threads not saturating. Under co-running load, or on a
    corpus with more small blocks or a different mix of stages, the two service threads
    may saturate first, where the default's three teams would degrade more gently.
  - The service core at one service thread saturated (svc1, 96% busy). The margin at two is
    the trace's 0.70.
  - A caller who asks for 10 threads gets 8 decoding, plus the service feeder. That is
    within the budget, but the thread count reported by the decoder changes.
- **The cause is unseparated** (C1 is a bundle). A rule built on it is a rule for the plan
  as measured, not for any one mechanism, so it shouldn't be extended to other budgets by
  argument.
