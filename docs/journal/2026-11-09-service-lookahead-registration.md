# The service core's admission threshold: registration

9 November 2026. Registered before any run. **The run is held under Guardian D045** (REVIEW
6730103867): service-core timing waits for targeted B5 evidence, which a separate agent is
producing. This registration, its freeze and its untimed dry run go ahead. The run is not
queued until D045's HOLD on service-core timing closes.

It follows up the service core at two teams (`2026-11-09-service-2x3-result.md` on master,
bundle `service-2x3/20261109T010708Z`, VALID). That run judged svc2@10 (two teams of three plus
two service threads, 8 decode threads) against today's default at 10 (`mibz2@10`, the pipelined
3 × 3 with park and small):
- speed 1.045 [1.017, 1.073], a SMALL GAIN;
- CPU 1.146 [1.123, 1.170], the default's CPU over svc2@10's. So svc2@10 spends 0.873 of the
  default's CPU: 12.7% less, about 11.0–14.5% less over the interval;
- peak RSS 64 MB against 74 MB.

Its trace of svc2@10 was NOT SATURATED (service utilisation max 0.70) but showed PREP
STARVATION: the teams were idle 16.5% of the time waiting for a prepared block
(`team.idle_prep_share` 0.165, against the 0.10 threshold). The registered reading was that
this is consistent with the prepared queue's lookahead binding, and not shown to be it.

## Why the admission threshold, not the window, is the judged question

The first version of this registration (frozen at 4587b54b, never run) varied the **window**:
8, 12 and 16 blocks in flight. That was the wrong bound for the question.

In `src/parallel.rs` (`StreamDecoder::service_with_fault`) the window and the prepared queue's
lookahead are separate:
- **the window** (`slots`) is the blocks in flight, from the scanner's issue to the reader's
  take. It bounds the jobs the service workers can prepare from;
- **the lookahead** was `let lookahead = teams;`, two at 2 × 3, fixed and not a parameter.

So the window arms couldn't test the lookahead, and a NULL would have left it untested. The
earlier trace also argues against the window: jobs waited 0.158 ms (median) in the hub before a
service worker took them, so the scanner's supply was usually there while the teams starved.

This version makes the lookahead a parameter (minimal and opt-in, below) and judges it. The
window is kept as one reported arm, and a joint arm crosses the two.

## What the lookahead is: an admission threshold

The hub's `lookahead` is **an admission threshold, not a cap on the queue** (Guardian D045). A
service worker, in its registered order:
1. prepares if the prepared queue is empty (and a job and a front set are free);
2. else finishes the earliest walked block, if any;
3. else prepares if fewer than `lookahead` blocks are prepared;
4. else waits, or ends.

Preparations under way are not counted. With K service threads, up to K can start while the
count P is below the threshold L, so P can reach L + K − 1. At two teams and two service
threads, L = 2 already admits three prepared blocks. The earlier trace's prepared backlog max
was 3.

So this registration names the parameter as the admission threshold L, and doesn't treat it as
a queue length. Each trace records the prepared count P that the arm actually reached: its max,
and its median over every preparation's push (the count each `Prepared` mark carries).

## The code change (minimal, opt-in)

- **`src/parallel.rs`:**
  - `StreamDecoder::service_admission(teams, k, services, slots, admission, kernels)`, which is
    `#[doc(hidden)]`.
  - `service_with_fault` and it both call one private `service_full`, with `None` and
    `Some(admission)` respectively.
  - The hub's threshold is `admission_for(teams, asked)`: `teams` when nothing is asked, as
    before, and at least 1 otherwise. The front sets keep their sizing, `teams + lookahead +
    services`, from it.
  - **The default path is unchanged.** `service` and `service_with_fault` pass `None`, so
    `lookahead = teams` exactly as before.
  - The unit test `service_admission_default_unchanged` checks that `admission_for(teams, None)
    == teams`. It also checks that thresholds 1–4 at 2 × 3 + 2 decode byte for byte as
    `service` does.
- **`src/read.rs`:** `BzReader::parallel_service_admission`, also `#[doc(hidden)]`, which
  forwards to it.
- **The harness:**
  - `examples/ab.rs` adds `mibz2-svc2-a3`, `-a4`, `-w12` and `-a4-w12`. The threshold-2 arms
    (`mibz2-svc2`, `-w12`) call `parallel_service` itself, the unchanged path.
  - `examples/occupancy.rs` adds `--admission N` (with `--svc`), records `admission` in its
    settings, and adds `t.svc.backlog.prep_median`.

The service core stays an opt-in research prototype. Nothing reaches the default.

## The cells

One `ab run` at THREADS = 10, interleaved by round, 20 rounds, with the arms' order rotated as
ab does:

| cell | plan | decode threads | admission threshold L | window | P's ceiling (L + K − 1) |
|---|---|---|---|---|---|
| `mibz2@10` | 3 × 3 pipelined, park and small (the 0.2.6 default) | 9 | – | 12 | – |
| `mibz2-svc2@10` | 2 × 3 + 2 service (as run in #258) | 8 | 2 | 8 | 3 |
| `mibz2-svc2-a3@10` | the same | 8 | 3 | 8 | 4 |
| `mibz2-svc2-a4@10` | the same | 8 | 4 | 8 | 5 |
| `mibz2-svc2-w12@10` | the same, the window alone | 8 | 2 | 12 | 3 |
| `mibz2-svc2-a4-w12@10` | the same, both (the joint arm) | 8 | 4 | 12 | 5 |
| `mibz2-team3@10` | 3 × 3 plain | 9 | – | 12 | – |
| `mibz2-team3-twin@10` | `mibz2-team3` under another name | 9 | – | 12 | – |

- **The w16 arm is dropped.** It added 20 runs and answered only the window's question, which
  is now a reported arm.
- **The other threads:** every arm runs the scanner, the reader's feeder and the consumer; svc2
  adds `mibz2-svc-feed`. None is counted in the budget, as before.
- **The default:** `mibz2` is the pipelined 3 × 3 only on the ten-core topology (10 physical
  cores, a 20 MiB L3). The report checks `lscpu.txt` for it.
- **The traces:** one occupancy pass of `--svc 2` at 10 threads for each svc2 arm, 6 rounds
  each, with its `t.svc.*` figures. Occupancy gives two teams a window of `slots × 2 / 10`, so
  the passes run `--slots 40` or `60`. They add `--admission 3` or `4` only where L isn't 2, so
  the L = 2 passes run the unchanged path. The report checks each pass's recorded slots and
  admission.

## The model (before any run)

Extrapolations, not measurements of these cells:
- svc2@10 at L = 2 ran 0.137 s (median); the default 0.144 s.
- The teams' 16.5% prep idle is the most a higher threshold could recover, so at most about
  1.20 for LA3 or LA4.
- **Against a large effect:**
  - Rule 3 comes after finishing. A higher threshold admits more preparations only when no
    walked block is waiting.
  - The service workers were busy 70% of the time. While both finish, no threshold prepares
    anything.
  - P already reached 3 at L = 2.
- **For some effect:** in the 30% the service workers were idle, a higher threshold would have
  let them prepare ahead.
- **The model's expectation:** LA3 and LA4 a SMALL GAIN or a NULL; P's median rising with L;
  prep starvation falling but not below 0.10. The window alone (W12) a NULL, from the jobs'
  wait. A small peak RSS cost for the extra front sets.

## The registered comparisons

Speed is the reference's wall over the arm's. CPU is the reference's CPU over the arm's. Above
1, the arm is faster, or spends less. Each comparison is paired by round, with the geometric
mean and its 95% t-interval over the 20 registered rounds. **Speed and CPU are co-primary**,
each with its own outcome.

**Judged, fixed now** (all 8 decode threads at window 8, so the extra-thread rule doesn't
apply):
- **LA3:** threshold 3 (`mibz2-svc2-a3@10`) against threshold 2 (`mibz2-svc2@10`);
- **LA4:** threshold 4 (`mibz2-svc2-a4@10`) against threshold 2.

**Reported, never judged:**
- **W12:** the window alone, `mibz2-svc2-w12@10` (L 2, window 12) against `mibz2-svc2@10`;
- **J:** the joint arm, `mibz2-svc2-a4-w12@10` (L 4, window 12) against `mibz2-svc2@10`;
- **J4:** the joint arm against `mibz2-svc2-a4@10`, the window given L = 4. Read with W12, it
  says whether the window binds once the threshold is raised;
- **D-a2w8, D-a3w8, D-a4w8, D-a2w12, D-a4w12:** each svc2 arm against `mibz2@10`. D-a2w8
  replicates bench-service-2x3's P1;
- **the best svc2 arm against the default:** the D- with the highest geometric-mean speed. It
  is chosen after the run from five, so its interval is optimistic, and it decides nothing;
- **R1:** `mibz2@10` against team3@10, as before;
- **per arm, from its trace:** P max and median (with its ceiling L + K − 1),
  `team.idle_prep_share`, saturation, prep starvation and finish backlog, with the queue waits
  and backlogs.

## The outcome map (numeric)

As the service core's and the service-2x3 registrations, unchanged. On a ratio's 95% interval
[lo, hi], speed and CPU each:

| outcome | condition | CPU's wording |
|---|---|---|
| GAIN | lo > 1.02 | CPU SAVING |
| SMALL GAIN | 1 < lo ≤ 1.02 | SMALL CPU SAVING |
| NULL | lo ≥ 0.97, hi ≤ 1.03, and 1 inside | NULL |
| REGRESSION | hi < 1 | CPU COST |
| INCONCLUSIVE | anything else | INCONCLUSIVE |

NULL is bounded uncertainty (within ±3%, a small effect is not excluded). CPU INCONCLUSIVE is no
detected CPU cost, not a CPU cost shown to be absent.

## The reading (registered, computed mechanically by the report)

In this order:
1. **A gain at either threshold and a regression at neither:**
   - **1a. THE ADMISSION THRESHOLD BINDS SVC2@10 AT 2:** no gaining contrast has a CPU COST;
   - **1b. A HIGHER ADMISSION THRESHOLD IS FASTER FOR A CPU COST:** a gaining contrast has a CPU
     COST. That is for Louis to weigh.
2. **INCONCLUSIVE (MIXED):** a gain at one threshold and a regression at the other.
3. **INCONCLUSIVE:** either is INCONCLUSIVE, and neither is a gain.
4. **A HIGHER ADMISSION THRESHOLD IS SLOWER:** both REGRESSION.
5. **NO GAIN DETECTED FROM A HIGHER ADMISSION THRESHOLD:** otherwise (both NULL, or one NULL and
   one REGRESSION).

Every reading says that the threshold admits preparations and doesn't cap the queue, which can
hold up to L + 1 at two service threads, and points to each trace's prepared count.

**The traces beside it** (reported, never judged; the service core's thresholds unchanged: PREP
STARVATION at `team.idle_prep_share` ≥ 0.10; FINISH BACKLOG at `queue.finish.median_ms` ≥ 0.5
or `team.idle_back_share` ≥ 0.05; SATURATED at utilisation max ≥ 0.90 with either):
- **PREP STARVATION AT THRESHOLD 2, RELIEVED AT A HIGHER THRESHOLD:** with reading 1,
  starvation at L 2 (window 8) and not at L 3 or 4. Consistent with the threshold binding the
  teams' supply at 2, not shown to be it.
- **PREP STARVATION PERSISTS AT EVERY THRESHOLD:** with reading 4 or 5, starvation at L 2, 3
  and 4 (window 8). Not relieved by admitting more preparations. The service workers' order of
  work (finishing first) remains a candidate, untested here.
- Otherwise the arms with starvation are listed, and not read.
- Any arm SATURATED is named.

**Whatever the reading,** nothing is adopted by this run. A change of svc2's threshold, like
svc2 itself, is qualified in the usual way (the plan in the service-2x3 result: a Plan rule,
CI's paired bench under RULES C3 and C5, a workstation confirmation, the Guardian's B5 review
of the service core, and fuzz evidence to D042's standard).

## Validity (the report checks each)

The run is VALID only if all of these hold, with bench/service_2x3.py's strict checks,
imported unchanged:
- the registered 20 rounds;
- a clean tree; the frozen inputs unchanged (no `NOT-FROZEN.txt`); not a dry run; `commit.txt`
  the checkout's head;
- the ten-core topology in `lscpu.txt` (1 socket, 10 cores, a 20 MiB L3);
- every run's record usable: a hex output digest, the registered sample's 209,700,196 decoded
  bytes, finite positive wall, CPU and peak RSS, the corpus `wikidata`;
- every run the same bytes as the reference `mibz2@10`'s, none failed;
- `corpus.sha256` the registered corpus's digest (82f65ace…, `bench/wikidata.json`);
- every registered cell's rounds 0–19 once each, every run with its registered plan (team
  leaders, helpers and service threads counted from ab's per-thread CPU); no other cell;
- **each of the five traces** complete and as registered:
  - one result at 10 threads, the plan "the service core, mibz2-svc2's plan", 2 service
    threads;
  - its slots the registered ones for its window (40 or 60, recorded as numbers);
  - its admission the registered one (3 or 4, or the default's "teams" where L is 2);
  - its commit the run's, parity true, the registered output size;
  - every metric the report reads finite and in range, P's max and median included;
  - and saturation resolved in each;
- **the noise control N1:** team3-twin@10 against team3@10 within 1 ± 0.03, the paired
  geometric-mean ratio of wall over the 20 rounds, both bounds of its 95% t-interval. N1 and its
  method are copied unchanged. That method is reserved to Louis (RULES C6).

**An INVALID run judges nothing.** What follows one is not decided here. Any rerun is
registered before it runs, and a question about N1's method goes to Louis (C6).

## Size and run time

- **Twenty rounds**, as for the service core and service-2x3. In service-2x3, P1's interval was
  about ±2.7% (speed) at 20 rounds. LA3 and LA4 compare one plan at two thresholds, so their sd
  should be no larger than P1's, where the plans differed.
- **Run time:**
  - service-2x3's runs took about 2.17 s each. Here there are 8 cells × 20 rounds = 160 runs,
    about 5.8 minutes, against about 4.3 for service-2x3's 120.
  - Add five traces of 6 rounds instead of one, and the build check and corpus.
  - That is **about 8 minutes once built**, or about 12 from a cold build.

## How it is frozen

`bench/service-lookahead.frozen` names the commit that all of these are frozen at: this
registration, the code change, the harness, the recipe (`just bench-service-lookahead`) and the
report (`bench/service_lookahead.py`, which imports `bench/service_2x3.py`'s and
`bench/service_core.py`'s predicates unchanged).
- The recipe refuses to run if any measured input differs from it.
- The measured inputs are `src`, the Cargo files and toolchain, `examples`, `tests/common`, the
  three reports and their helpers (`bench/team_size.py`, `bench/wikidata.py`),
  `bench/wikidata.json`, `bench/bundle.sh`, `.just/bench.just` and this file.
- The frozen commit is the commit before that file's last change, since a registration can't
  name its own commit's SHA.
- It supersedes 4587b54b, the window version, under which nothing timed ran.

The report calls nothing VALID but the registered run.

## What this design does not separate

- **The threshold's two effects:** a higher L both admits more preparations and adds front
  sets. The front sets are sized from L. The trace's P says how far the queue actually ran
  ahead.
- **The service workers' order of work:** finishing before preparing beyond an empty queue is
  not varied.
- **The window's other effects** (W12, J, J4): a wider window also lets the reader's ready
  blocks and the scanner run further ahead, holding more memory. Peak RSS is reported; the L3
  footprint is not measured, and RSS is not cache residency.

## Safety

- **D045:** Guardian REVIEW 6730103867 HOLDs further service-core timing and production
  admission, pending targeted B5 evidence:
  - reached-path service fuzz and differential evidence (the fuzz harness never selects the
    service decoder);
  - exact mutation records for wake-up, retirement and buffer return, helper-completion
    ownership and ordered publication;
  - a controlled schedule or handshake for the wake regression.

  This run is held under it. The code change here adds a parameter to the hub's existing
  threshold. It changes no ownership, wake-up or retirement path, and adds a test that the
  default path is unchanged. That is not new B5 evidence.
- **D042's** scoped production-safety HOLD on the composed pipeline stands.
- **The panic scope:** the service core's stage catches don't cover a panic in a user's
  `Read`, as D045 notes.
- Matching output hashes and green CI are not safety evidence.

## Local runs

Local runs on the shared 4-core machine are correctness checks only, never timing evidence. The
recipe's DRY run (`just bench-service-lookahead 20 1`) is untimed, and the report never calls it
VALID.
