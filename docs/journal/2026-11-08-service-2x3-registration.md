# The service core at two teams: registration

8 November 2026. Registered before any run. It follows up the service core's registered run
(`2026-11-08-service-core-result.md`, bundle `service-core/20261108T103216Z`, VALID). That run
reported, unjudged, its cell S4:
- svc2@10 (two teams of three plus two service threads, 8 decode threads) against today's
  default at 10 (`mibz2@10`, the pipelined 3 × 3 with park and small, 9 decode threads);
- speed 1.045 [1.023, 1.068], CPU 1.138 [1.118, 1.158] (12% less), peak RSS 64 MB against
  74 MB.

That run had no plain 2 × 3 cell. So it can't tell whether svc2@10's advantage comes from the
service threads or from running two teams rather than three. This registration asks both
questions, and judges them.

The prototype is the one on `research/service-core` (#251), unchanged: `src/` is as it was
at 9ef3c17d. The only code added is two arms of the harness (`examples/ab.rs`), which build
existing decoders at a fixed plan: no decoder changes.

**How it is frozen.** `bench/service-2x3.frozen` names the commit that all of these are frozen
at: this registration, the harness, the recipe (`just bench-service-2x3`) and the report
(`bench/service_2x3.py`, which imports `bench/service_core.py`'s predicates unchanged). The
recipe refuses to run if any measured input differs from it. The measured inputs are `src`,
the Cargo files and toolchain, `examples`, `tests/common` (the harness's `#[path]` module),
the two reports and their helpers, `bench/wikidata.*`, `bench/bundle.sh`, `.just/bench.just`
and this file. The frozen commit is the commit before that file's last change, since a
registration can't name its own commit's SHA. The report calls nothing VALID but the
registered run.

## The questions

- **(a) The product question:** at a budget of 10 decode threads, is svc2@10 faster than
  today's default, and at what CPU cost?
- **(b) The causal question:** at the same two teams of three, do the two service threads
  earn their keep? That is svc2@10 against the plain `Plan::Teams { teams: 2, k: 3 }`, with
  the same window, at the same budget.
- **(c) The alternative:** does today's decoder gain from two teams alone? That is the
  pipelined 2 × 3 with park and small (the default's team, at two teams) against the
  default.

**Building (c) needed no decoder change.** The pipelined team with the default's levers is
`BzReader::parallel_pipelined_levers` with `parallel::PIPELINED_LEVERS`, the same call
`StreamDecoder::new` makes at `PIPELINED_PLAN`. The existing arm `mibz2-pipe3-both` already
runs it as 2 × 3 at 6 to 8 threads, as `mibz2-team3` runs the plain 2 × 3. But an ab run's
arms share one THREADS, and those arms size their teams from it. So both arms are added under
fixed names, independent of THREADS, so they interleave with the other cells at the budget of
10:
- `mibz2-team2x3`: `parallel_planned(Plan::Teams { teams: 2, k: 3 })`;
- `mibz2-pipe2x3`: `parallel_pipelined_levers(2, 3, .., PIPELINED_LEVERS)`.

Both use the teamK arms' window, `SLOTS_PER_THREAD` (4) blocks a team, so 8, the same window
`mibz2-svc2` keeps at 10. They are exactly `mibz2-team3@8` and `mibz2-pipe3-both@8`.

## The cells

One `ab run` at THREADS = 10, interleaved by round, 20 rounds, with the arms' order rotated
as ab does:

| cell | plan | decode threads | other threads | tables at once | window |
|---|---|---|---|---|---|
| `mibz2@10` | 3 × 3 pipelined, park and small (the 0.2.6 default) | 9 | 3 | 3 | 12 |
| `mibz2-svc2@10` | 2 × 3 + 2 service | 8 | 4 | 2 | 8 |
| `mibz2-team2x3@10` | 2 × 3 plain | 6 | 3 | 2 | 8 |
| `mibz2-pipe2x3@10` | 2 × 3 pipelined, park and small | 6 | 3 | 2 | 8 |
| `mibz2-team3@10` | 3 × 3 plain | 9 | 3 | 3 | 12 |
| `mibz2-team3-twin@10` | `mibz2-team3` under another name | 9 | 3 | 3 | 12 |

- **The other threads:** every arm runs the scanner, the reader's feeder and the consumer.
  svc2 adds `mibz2-svc-feed`. None is counted in the budget, the same way for every arm, as
  in the service core's registration.
- **The default:** `mibz2` is the pipelined 3 × 3 with park and small only on the ten-core
  topology (10 physical cores, a 20 MiB L3). The report checks `lscpu.txt` for it.
- **The window:** the default's is scaled to its three teams (12). The 2 × 3 cells keep 8.
  So in P1 and P2 the window differs with the plan, as the default's policy would size it.
  In C1 it is the same.
- **The trace:** an occupancy pass of `--svc 2` at 10 threads, 6 rounds, with its `t.svc.*`
  figures, as before.

## The model (before any run)

These are extrapolations from earlier measurements, not measurements of these cells:
- the sample decodes to 209.7 MB;
- the service core's run, as medians: svc2@10 0.136 s, `mibz2@10` 0.143 s, team3@10 0.152 s;
- E1's per-team rate at two teams on the i9: 558–583 MB/s.

**What they predict:**
- **The plain 2 × 3:** 1.12–1.17 GB/s, so a wall of about 0.180–0.188 s.
  - C1 about 1.32–1.38, near the threads' ratio 8/6 = 1.333.
  - S2 (the plain 2 × 3 against the default) about 0.76–0.79.
- **The pipelined 2 × 3:** if the pipeline gains at two teams what it gained at three (R1,
  about 1.07), about 1.20–1.25 GB/s, a wall of about 0.168–0.175 s. So P2 about 0.82–0.85, a
  REGRESSION.
- **P1:** the earlier S4, 1.045, if it replicates.

So the model expects:
- the service threads to earn their keep (C1);
- two teams alone not to beat the default (P2);
- svc2@10's gain over the default to come from the service threads, not from two teams.

**The traces' warning:** svc2@10's trace in that run showed the service 70% busy, with no
finish backlog, but with PREP STARVATION (the teams idle 15% of the time waiting for a
prepared block). The prepared queue, not the service's capacity, may bind.

## The registered comparisons

Speed is the reference's wall over the arm's. CPU is the reference's CPU over the arm's.
Above 1, the arm is faster, or spends less. Each comparison is paired by round, with the
geometric mean and its 95% t-interval over the 20 registered rounds. **Speed and CPU are
co-primary**, each with its own outcome.

**Judged:**
- **P1 (a):** svc2@10 against `mibz2@10`, the product question (8 decode threads against 9);
- **C1 (b):** svc2@10 against team2x3@10, the causal question (8 against 6);
- **P2 (c):** pipe2x3@10 against `mibz2@10`, two teams under today's decoder (6 against 9).

**Reported, never judged:**
- **S1:** svc2@10 against pipe2x3@10 (the two candidate products head to head);
- **S2:** team2x3@10 against `mibz2@10`;
- **S3:** pipe2x3@10 against team2x3@10 (the pipeline's gain at two teams);
- **S4:** team2x3@10 against team3@10 (two plain teams against three);
- **S5:** svc2@10 against team3@10 (the earlier S3, replicated);
- **R1:** `mibz2@10` against team3@10 (the levers' result and the earlier R1, replicated).

## The outcome map (numeric)

As the service core's registration, unchanged. On a ratio's 95% interval [lo, hi], speed and
CPU each:

| outcome | condition | CPU's wording |
|---|---|---|
| GAIN | lo > 1.02 | CPU SAVING |
| SMALL GAIN | 1 < lo ≤ 1.02 | SMALL CPU SAVING |
| NULL | lo ≥ 0.97, hi ≤ 1.03, and 1 inside | NULL |
| REGRESSION | hi < 1 | CPU COST |
| INCONCLUSIVE | anything else (the interval spans gain and harm, or is too wide for NULL) | INCONCLUSIVE |

**The extra-thread rule (as registered for the service core):**
- A speed gain where the arm runs more decode threads than its reference gets a label. It is
  BEYOND ITS THREADS only if its lower bound is above the threads' ratio.
- **Only C1 qualifies:** 8 against 6, ratio 1.333.
- C1's extra threads are the service threads themselves, the intervention under test, as K's
  were in the service core's D1. So C1's label is reported, not required: a C1 gain is read
  as the service threads' effect at an equal budget, either way.
- P1 and P2 run fewer decode threads than the default, so no label applies.

## The readings (registered, computed mechanically by the report)

*Revised 9 November 2026 (see "Revision for HOLD 6729358962" below): the order and the
judged family below are unchanged; the words the report prints for them are narrowed there.*

**The product (P1):**
- **SVC2@10 BEATS THE DEFAULT:** a speed GAIN or SMALL GAIN, with CPU not a CPU COST;
- **A TRADE:** a speed gain with a CPU COST;
- **NOT REPLICATED:** NULL or REGRESSION;
- **INCONCLUSIVE** otherwise.

**The cause (C1):**
- **THE SERVICE THREADS EARN THEIR KEEP:** a speed GAIN;
- **A SMALL EFFECT OF THE SERVICE THREADS:** a SMALL GAIN;
- **THE SERVICE THREADS DON'T EARN THEIR KEEP:** NULL or REGRESSION;
- **INCONCLUSIVE** otherwise.

C1's CPU outcome is reported beside it: the service threads' CPU against the plain teams'.

**Two teams under today's decoder (P2):**
- **TWO PIPELINED TEAMS BEAT THE DEFAULT:** a speed GAIN or SMALL GAIN;
- **TWO PIPELINED TEAMS DON'T BEAT THE DEFAULT:** NULL or REGRESSION;
- **INCONCLUSIVE** otherwise.

**Overall, in this order:**
1. **NOT REPLICATED:** P1 NULL or REGRESSION. svc2@10 doesn't beat the default; nothing is
   adopted.
2. **NONE OF THE FOUR:** P1 inconclusive.
3. **TWO TEAMS, NOT THE SERVICE THREADS:** P1 a speed gain, and C1 NULL or REGRESSION. The
   service core's advantage comes from the plan of two teams. P2 and S1 then say whether
   today's decoder at two teams is the simpler way to get it.
4. **NONE OF THE FOUR:** C1 inconclusive.
5. **A TRADE:** P1 a speed gain for a CPU COST, and C1 a gain. That is for Louis to weigh.
6. **FOR SVC2@10:** P1 a speed gain with CPU not a CPU COST, and C1 a gain. The service
   threads cause it.

**Whatever the readings,** nothing is adopted by this run. A production change is qualified
in the usual way, through CI's paired bench (RULES C3, C5).

## The registered counterexample, service saturation

From svc2@10's trace, as medians over its complete rounds, with the service core's
registration's thresholds unchanged:
- **service utilisation:** each service thread's busy time over the run's span;
- **PREP STARVATION:** `team.idle_prep_share` at 0.10 or more;
- **FINISH BACKLOG:** `queue.finish.median_ms` at 0.5 ms or more, or `team.idle_back_share`
  at 0.05 or more;
- **SATURATED:** utilisation max at 0.90 or more, and PREP STARVATION or FINISH BACKLOG.

**Reading it with C1** (the flag unchanged; its capacity reading is a hypothesis, see the
revision below):
- **SATURATION BINDS:** SATURATED, and C1 NULL or REGRESSION. Two service threads can't feed
  two teams. The null is the service's capacity, not the idea.
- **SATURATED, A GAIN NONETHELESS:** SATURATED, and C1 a gain. More service capacity might
  add more.
- **SATURATED, C1 INCONCLUSIVE.**
- **NOT SATURATED at 2 × 3 + 2:** neither. PREP STARVATION without saturation, as in the
  earlier trace, points at the prepared queue's lookahead, not the service's capacity. It is
  reported, not judged.
- **UNRESOLVED:** the trace is missing or incomplete (and then the run is INVALID).

The trace's queue waits, backlogs, the service's stages and the teams' run wait are reported
as in the service core's report.

## Validity (the report checks each)

The run is VALID only if all of these hold (and, since the revision below, the usable-evidence,
workload and trace-configuration checks it adds):
- the registered 20 rounds;
- a clean tree;
- the frozen inputs unchanged (no `NOT-FROZEN.txt`);
- not a dry run;
- `commit.txt` the checkout's head;
- every run the same output bytes, none failed;
- every registered cell's rounds 0–19 once each, every run with its registered plan (team
  leaders, helpers and service threads counted from ab's per-thread CPU);
- no other cell;
- the registered trace complete, with at least one complete round;
- **the ten-core topology** in `lscpu.txt` (1 socket, 10 cores, a 20 MiB L3), so `mibz2` is
  the pipelined default;
- **the noise control N1:** team3-twin@10 against team3@10 (the same decoder under two
  names, in one interleaved run) within 1 ± 0.03.

N1 and its method are copied unchanged from the service core's registration: the paired
geometric-mean ratio of wall over the 20 rounds, both bounds of its 95% t-interval within
[0.97, 1.03]. That method is reserved to Louis (RULES C6), and nothing here changes it.

**An INVALID run judges nothing.** What follows an INVALID run is not decided here. Any rerun
is registered before it runs, and a question about N1's method goes to Louis (C6).

## Size and run time

- **Twenty rounds**, as registered for the service core. The harness's arithmetic, on that
  run's paired rounds at 10 threads, shows that fewer rounds don't meet the same precision:
  - **P1's cells:** the sd of the paired log-ratio was 0.046 (speed) and 0.038 (CPU). That
    gives intervals of ±2.2% and ±1.8% at 20 rounds, and ±3.3% and ±2.7% at 10.
  - **N1:** its sd was 0.032, an interval of ±1.5% at 20 rounds and ±2.3% at 10. N1 needs both
    bounds within ±3%, so a shorter run would leave a twin's chance offset less room. The
    first service-core attempt was INVALID on N1 at 1.018 [1.000, 1.035].
- **Run time:** the service core's registered run took 2.17 s a run (220 runs in 7 min 58 s,
  from its bundle's file times).
  - Here that is 6 cells × 20 rounds = 120 runs, about 4.3 minutes.
  - Add about 45 s of build check and corpus, and a few seconds for the trace.
  - That is **about 6 minutes once built**, or about 10 minutes from a cold build, well under
    the 45 minutes allowed.

## What this design does not separate

- **C1 compares two things at once,** where the serial work runs and the two extra threads,
  plus the hub's synchronisation and hand-offs. The extra threads are the intervention, so
  C1 answers "do the service threads earn their keep" at an equal budget. It doesn't answer
  whether the same two threads would help the plain teams as plain workers.
- **P1 and P2 compare plans as the budget gives them:** tables at once, decode threads and
  the window all change together.
- **What is not measured:** the L3 footprint of the queued front and back sets. Peak RSS is
  reported, and RSS is not cache residency.

## Local runs

Local runs on the shared 4-core machine are correctness checks only, never timing evidence.
The recipe's DRY run (`just bench-service-2x3 20 1`) is untimed, and the report never calls
it VALID.

## Revision for HOLD 6729358962 (9 November 2026)

Before any run, the Researcher's REVIEW 6729358962 held the recipe from the workstation queue
(report and probe: `state/reports/service-2x3-6729243943/` at e6411029 on
`team-state/researcher`). This revision closes its three findings. **Nothing judged changes:**
the judged family (P1, C1, P2), the outcome map's thresholds (GAIN lo > 1.02, NULL within
[0.97, 1.03], REGRESSION hi < 1), the extra-thread rule, N1 and its method (C6), the
saturation thresholds and the registered order of the overall reading are as above. The
changes are the report's validity checks (evidence it must have, not new statistics) and
the words it prints. The recipe is unchanged; the freeze moves to the revised commit.

| finding | fix (`bench/service_2x3.py`; `bench/service_core.py` unchanged) | fixture (`bench/test_service_2x3.py`) |
|---|---|---|
| H1: all-null output, empty hash with zero output passed VALID | every run's record must be usable: `hash` a hex digest, `out` the registered sample's 209,700,196 decoded bytes (`bench/wikidata.json`), every hash equal to the reference `mibz2@10`'s | `test_null_output`, `test_empty_hash_zero_output`, `test_hashes_differ` |
| H1: deleted `hash`, `out` or `wall` raised KeyError | an unusable record is INVALID with its reason, never an exception | `test_deleted_hash`, `test_deleted_size`, `test_deleted_wall` |
| H1: negative wall and CPU passed | wall, CPU and peak RSS finite and positive | `test_negative_wall_and_cpu`, `test_nonfinite_wall` |
| H1: a wrong corpus name, or no corpus digest, passed | every run's `corpus` is `wikidata`, and `corpus.sha256` is the registered corpus's digest (82f65ace…, `bench/wikidata.json`); so a `CORPUS=` override can't pass as the registered workload unless it is the same bytes | `test_wrong_corpus`, `test_absent_corpus_digest`, `test_other_corpus_digest` |
| H1: a trace of only `complete_rounds: 1` was VALID with saturation UNRESOLVED | the trace must carry every metric the report reads, finite and in range, and saturation must be resolved, as "UNRESOLVED (and then the run is INVALID)" above always said | `test_trace_complete_rounds_only`, `test_trace_metric_out_of_range` |
| H1: the trace's configuration was not checked | the trace's own record must be the registered one: one result at 10 threads, the plan "the service core, mibz2-svc2's plan", 2 service threads, its `commit` the run's, parity true, the registered output size | `test_trace_wrong_configuration`, `test_trace_wrong_plan`, `test_trace_other_commit`, `test_trace_no_parity` |
| H1: the rejection controls | kept | `test_one_round`, `test_duplicate_round`, `test_wrong_threads`, `test_unregistered_arm`, `test_dry`, `test_not_frozen` |
| H2: P2 never entered the overall reading; illustrative walls (default 1, plain2 1.10, pipe2 0.90, svc2 0.95) printed "FOR SVC2@10 … the service threads cause it" while svc2 loses to pipe2 | P1, C1, P2 and S1 are each printed as a finding of its own (S1 reported, never judged). The overall reading follows the registered order, but prints FOR SVC2@10 only where S1 shows svc2@10 faster than pipe2x3@10; otherwise it prints "REGISTERED STEP 6 MET, BUT SVC2@10 NOT SHOWN BETTER THAN THE SIMPLER PIPE2X3@10 …, for Louis to weigh". No reading names the service threads as the cause | `test_illustrative_walls`, `test_for_svc2_only_when_s1_shows_it`, `test_without_s1_never_for_svc2` |
| H2: C1 read as the service threads' effect | C1 is the service **bundle** against the plain 2 × 3 at the same teams and window: the two service threads, the serial work they take, the feeding, the hub's synchronisation and hand-offs, together. Its words: THE SERVICE BUNDLE BEATS THE PLAIN 2 × 3, A SMALL GAIN OF THE SERVICE BUNDLE, NO SERVICE-BUNDLE GAIN DETECTED, THE SERVICE BUNDLE IS SLOWER. Step 3 reads "P1 A GAIN WITHOUT A DETECTED SERVICE-BUNDLE GAIN", the plan's and the service's roles unresolved | `test_illustrative_walls`, `test_null_is_bounded` |
| H2: the decomposition | on the common paired rounds' geometric means, P1 = S1 × P2 = C1 × S2 (exact round by round, so on the log-scale means), printed for speed and CPU; a decomposition of comparisons, not of causes; the intervals' endpoints are never multiplied | `test_illustrative_walls` |
| H2: NULL and CPU INCONCLUSIVE | NULL is printed as bounded uncertainty ("within ±3%, a small effect is not excluded"), CPU INCONCLUSIVE as "no CPU cost detected, not shown to cost nothing" | `test_null_is_bounded`, `test_without_s1_never_for_svc2` |
| H3: "SATURATION BINDS: the null is the service's capacity, not the idea" | the numeric flag SATURATION BINDS is kept (`saturation_flag` in the JSON); its capacity reading is printed as a hypothesis the figures are consistent with, with load imbalance between the service threads and queue or scheduling effects still possible. Prep starvation without saturation is "consistent with the prepared queue's lookahead binding, not shown to be it" | `test_saturation_is_a_hypothesis`, `test_saturated_trace_flag_kept` |

**What this does not close.** The report's VALID is a statement about its predicates on the
attempt's directory. It checks `commit.txt` against the checkout and the absence of
`NOT-FROZEN.txt`; the recipe makes the frozen-tree comparison. So the bundle's admission
still rests on the recipe, the head, the frozen commit and the corpus digest together, as
before. Guardian D042's scoped production-safety HOLD on the composed pipeline stands; this
run can't discharge it, and nothing here is a production qualification.

