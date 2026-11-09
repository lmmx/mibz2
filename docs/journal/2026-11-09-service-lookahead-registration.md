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

Extrapolations and hypotheses, not measurements of these cells (revised 9 November, see the
revision section):
- svc2@10 at L = 2 ran 0.137 s (median); the default 0.144 s.
- **The 0.165 prep idle is a modelled share, not removable wall time.** It is the teams' summed
  idle waiting for a prepared block, over teams × the trace's span. Under a balanced-
  utilisation model, removing all of it would give about 1/(1 − 0.165) ≈ 1.20. That is a
  model, not a speed ceiling: which worker is critical, overlap and changed costs can each
  break it.
- **Against a large effect:**
  - Rule 3 comes after finishing. A higher threshold admits more preparations only when no
    walked block is waiting.
  - The service workers were busy 70% of the time. While both finish, no threshold prepares
    anything.
  - P already reached 3 at L = 2.
- **For some effect:** a higher threshold lets an idle service worker prepare ahead. The 30%
  service idle doesn't show such chances existed, since a job and a free front had to be there
  at the same moment, and the jobs' 0.158 ms median wait doesn't show jobs were there while the
  teams starved. Both are hypotheses.
- **Fronts aren't a second admission bottleneck** (the Researcher's bound, REVIEW 6730484783).
  - Let F = T + L + K be the front sets, P the prepared blocks, A the other workers' fronts
    held for a preparation, and U the teams' fronts.
  - At an idle worker's decision under rule 3, P ≤ L − 1, A ≤ K − 1 and U ≤ T. So the free
    fronts are F − P − A − U ≥ 2. Rule 1, with P = 0, gives the same.
  - So a front is never what stops an eligible preparation, under the source's conservation of
    fronts: a team returns its front when the walk ends, and walked blocks keep only their
    back sets.
  - A 5,040-state arithmetic check agrees. It checks the inequality, not Rust ownership.
  - The extra fronts can still change allocation, reuse and cache costs, so a higher-L policy's
    effect is broader than queue depth. No fixed-front arm is added.
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
- **per arm, from its trace:** P's max over complete rounds, the median of the round maxima and
  the median of the round push-medians (with its ceiling L + K − 1), and each round's values,
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

Every reading says three things (revised 9 November):
- the threshold admits preparations and doesn't cap the queue, which can hold up to L + 1 at two
  service threads; it points to each trace's prepared count;
- a reading is the effect of the implemented higher-threshold policy, which also sizes more front
  sets, not proof that the prepared queue's supply is the bottleneck;
- each contrast is at its own nominal 95%, not a familywise claim for "either".

**The traces beside it** (reported, never judged; the service core's thresholds unchanged: PREP
STARVATION at `team.idle_prep_share` ≥ 0.10; FINISH BACKLOG at `queue.finish.median_ms` ≥ 0.5
or `team.idle_back_share` ≥ 0.05; SATURATED at utilisation max ≥ 0.90 with either):
- The gaining arms (LA3 or LA4 a speed gain) and the relieved arms (starved at L 2, window 8, and
  not starved at L 3 or 4) are always named separately (revised 9 November).
- **CONCORDANT AT …:** with reading 1, only for arms that both gain and are relieved. Consistent
  with the higher threshold acting on the teams' supply, not shown to be it.
- **NOT CONCORDANT:** with reading 1, where no arm is both. No combined reading is printed.
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

## Revision for the Researcher's HOLD (REVIEW 6730484783, 9 November 2026)

Before any run, the Researcher's REVIEW 6730484783 held #271 from the queue, independently of
D045 (report and witnesses: `state/reports/service-lookahead-6730378782/` at 99689cec on
`team-state/researcher`). This revision closes H1–H3 at a new freeze.

**Nothing judged changes.** The judged family (LA3, LA4), the outcome map's thresholds, N1 and
its method (C6), the saturation and starvation thresholds and the registered order of the
reading are as above. The changes are stricter evidence checks, the words the report prints and
the model's labels. The correction is applied at this new freeze, not to an outcome: nothing
ran under 4587b54b or 79e41419.

| finding | correction | fixture (`bench/test_service_lookahead.py`) |
|---|---|---|
| H1: P checked only finite and nonnegative; max 99, median > max, and blocks 0 were VALID | **Stricter:** every complete timeline round's prepared count, read from the raw runs, must satisfy 1 ≤ median ≤ max ≤ L + K − 1 with the registered 431 blocks (an expected scanner count tied to the corpus digest, from earlier traces; see the second revision), before any aggregate is read. The summary is checked against the same invariant, and against the rounds it summarises (it must be their median) | `test_prepared_max_99_summary`, `test_prepared_max_99_every_round`, `test_prepared_median_above_max`, `test_prepared_above_its_own_ceiling`, `test_one_round_breaches`, `test_no_blocks`, `test_no_blocks_summary`, `test_summary_not_reconciled` |
| H1: the positive fixture gave L = 3 a P max of 5, above its ceiling of 4 | the fixture was wrong, not the ceiling: L 3 now reaches 4, L 4 reaches 5 | `test_full_run_threshold_binds` |
| H1: occupancy's `prep_max` is the median of the round maxima, and `prep_median` the median of the round medians | labelled so. The report prints the max over complete rounds, the median of round maxima and the median of round medians (which may be half-integral), and keeps each round's values in its JSON | `test_full_run_threshold_binds` |
| H2: occupancy saved no child status; a child that printed a valid line and exited nonzero could pass | **Stricter, `examples/occupancy.rs`:** every child's exit status (its code, or −signal) goes into each raw run. `children_ok` and `children_expected` (rounds × 3 modes) go into the JSON, and the pass exits 1 on any nonzero child or a missing one. The report requires each scheduled (round, mode) once, each with status 0, and `children_ok` | `test_child_nonzero_after_a_valid_line`, `test_child_killed`, `test_children_not_ok`, `test_child_missing`, `test_child_duplicated` |
| H2: empty raw runs were VALID | INVALID, as is a trace with no complete timeline round in its raw runs | `test_raw_runs_empty`, `test_no_complete_round` |
| H2: the recipe trusted the JSON's VALID whatever the report's exit | the recipe writes VALID only when the report exited 0 *and* its JSON says VALID | (the recipe) |
| H2: not a six-complete-round gate | the inherited rule is kept: at least one complete trace round, every scheduled child accounted for. Complete rounds are printed beside the requested ones, so fewer than six shows as lost precision, not invalidity. A summary inconsistent with its raw runs (complete rounds, or requested rounds, that disagree) is INVALID | `test_one_complete_round_still_valid` |
| H3: the mechanism line joined a gain at one arm with relief at another | the gaining arms and the relieved arms are named separately. "CONCORDANT AT …" is printed only for arms that both gain and are relieved; otherwise "NOT CONCORDANT … No combined reading" | `test_gain_and_relief_on_different_arms` (LA3 a GAIN but starved, LA4 NULL but relieved), `test_mechanism` |
| H3: "the threshold binds" read as a queue bottleneck | every reading says it is the effect of the implemented higher-threshold policy (which also sizes more fronts), not proof that the prepared queue's supply is the bottleneck. Each contrast is at its own nominal 95%, not a familywise claim for either | `test_policy_effect_wording` |
| H3: the model's 0.165 and 1.20 | relabelled in "The model" above: a modelled share and a balanced-utilisation model, not removable time or a ceiling. The 30% service idle and the 0.158 ms job wait are hypotheses, not opportunity shown | (the registration) |
| the front bound | recorded in "The model" above | (the registration) |

**What reaching the ceiling means.** P reaching L + 1 in the dry run shows the threshold is
reachable as implemented. It doesn't show that the threshold binds often. The push median
samples arrivals, not time-weighted occupancy.

**The budget.** 160 ab runs at about 2.17 s each is 5.8 minutes. The traces add 90 occupancy
children (5 passes × 6 rounds × 3 modes) plus each pass's setup. About 8 minutes once built is
an estimate, not a bound. Same-plan contrasts aren't guaranteed a smaller variance than #258's.

**Still held:** under D045, for service-core timing, until its B5 evidence closes.

## Second revision, for the Researcher's narrowed HOLD (REVIEW 6730774618, 9 November 2026)

The Researcher accepted the first revision's H1 ceiling and labels, H3's same-arm reading and
wording, the native child status and the recipe's exit check
(`state/reports/lookahead-rereview-6730656466/` at bab88d74 on `team-state/researcher`). It
narrowed the HOLD to four contradictory traces that still read VALID. This revision closes them.

**Unchanged:** LA3, LA4, N1 and its method, and every numerical threshold. These are stricter
evidence checks only.

| witness | correction (`bench/service_lookahead.py`) | fixture |
|---|---|---|
| one requested round, three children, truncated and reconciled, was VALID | **Stricter:** each pass must request the registered 6 rounds, so 18 children, each (round, mode) once with status 0. This is not a six-complete-round gate: at least one complete timeline round is still the rule | `test_three_children_instead_of_eighteen`; the control `test_one_complete_round_six_requested` (6 requested, 18 children, 1 complete round) stays VALID |
| an off child with hash BAD and err failed, beside parity true and status 0, was VALID | **Stricter:** parity, output and error are recomputed from every child's own line. Each needs a hex digest, the registered output size and no error, and all digests must be equal. The summary's `parity` flag is no longer trusted alone | `test_raw_hash_and_error_disagree_with_parity`, `test_raw_hashes_differ`, `test_raw_output_size` |
| one complete round with idle_prep_share 99 was hidden by the median | **Stricter:** in each complete timeline round, every required metric must be present, finite and in its domain before reconciliation. Shares and utilisations (`svc.util.max`, `.0`, `.1`, `team.idle_prep_share`, `team.idle_back_share`) lie in [0, 1]. Waits, backlogs, run wait, blocks and P are at least 0. Blocks must be 431, P within its invariant, and `svc.n.services` 2 | `test_one_complete_round_share_99`, `test_one_complete_round_util_above_one`, `test_one_complete_round_negative_wait` |
| one complete round with idle_prep_share NaN was dropped from the median while still counted complete | **Stricter:** a NaN or absent metric in a complete round makes the trace INVALID, and is never dropped. A genuinely incomplete round (`t.complete=no`) is still not read | `test_one_complete_round_share_nan`, `test_one_complete_round_metric_absent`, `test_incomplete_round_nan_ignored` |

**The 431 blocks** is an explicit expected scanner count (`t.blocks`) for the registered corpus,
whose digest (82f65ace…) is fixed. It comes from earlier traces: #258's workstation trace and
this registration's dry runs all read 431. `bench/wikidata.json` doesn't supply it; it fixes the
corpus, and so the count.

**The reconciliation tolerance** is 1e-6 × max(1, |median|): absolute below one, relative above.
It covers the six-digit formatting of the children's lines. It doesn't replace the per-round
domain checks above.

## Local runs

Local runs on the shared 4-core machine are correctness checks only, never timing evidence. The
recipe's DRY run (`just bench-service-lookahead 20 1`) is untimed, and the report never calls it
VALID.
