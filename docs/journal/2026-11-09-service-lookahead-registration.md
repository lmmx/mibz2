# The service core's window: registration

9 November 2026. Registered before any run. It follows up the service core at two teams
(`2026-11-09-service-2x3-result.md` on master, bundle `service-2x3/20261109T010708Z`, VALID).
That run judged svc2@10 (two teams of three plus two service threads, 8 decode threads)
against today's default at 10 (`mibz2@10`, the pipelined 3 × 3 with park and small):
- speed 1.045 [1.017, 1.073], a SMALL GAIN;
- CPU 1.146 [1.123, 1.170], the default's CPU over svc2@10's. So svc2@10 spends 0.873 of the
  default's CPU: 12.7% less, about 11.0–14.5% less over the interval;
- peak RSS 64 MB against 74 MB.

Its trace of svc2@10 was NOT SATURATED (service utilisation max 0.70) but showed PREP
STARVATION: the teams were idle 16.5% of the time waiting for a prepared block
(`team.idle_prep_share` 0.165, against the 0.10 threshold). The registered reading was that
this is consistent with the prepared queue's lookahead binding, and not shown to be it.

The 2 × 3 cells there ran a window of 8 blocks in flight (`SLOTS_PER_THREAD`, 4, a team).
The default runs 12. This registration asks whether svc2@10 gains from a wider window.

## What is varied, and what isn't

**The window is not the prepared queue's lookahead.** In `src/parallel.rs`
(`StreamDecoder::service_with_fault`) the two are separate bounds:
- **the window** (`slots`, a constructor parameter): the blocks in flight, from the scanner's
  issue to the reader's take. It bounds the jobs the service workers can prepare from;
- **the prepared queue's lookahead** (`let lookahead = teams;`): how many prepared blocks the
  service workers keep ahead of the teams by their rule 3. It is two at 2 × 3, fixed, and
  the front sets are sized from it (`teams + lookahead + services`, six).

This run varies the window alone, with no `src/` change: `slots` is already a parameter of
`BzReader::parallel_service`. The lookahead is not a parameter, and varying it would need an
opt-in `src/` change. So:
- a gain here says the window binds svc2@10 at 8;
- a NULL says it doesn't, and leaves the lookahead itself untested.

**What the earlier trace already suggests** (a prior, not a measurement of these cells):
jobs waited 0.158 ms (median) in the hub before a service worker took them, so the scanner's
supply was there at least some of the time the teams starved. That points away from the
window, towards the lookahead itself or the service workers' order of work (finishing first,
except that the prepared queue is never left empty). The model below expects a NULL.

The question is asked as the coordinator's LOG on #145 put it, "svc2@10 with a larger
prepared-queue window", and this section records which bound that window is.

## The arms

`examples/ab.rs` builds `mibz2-svc2` as `BzReader::parallel_service(file, teams, 3, 2, slots,
..)`, with `teams = ((THREADS - 2) / 3).clamp(1, 3)`, two at 10, and `slots =
SLOTS_PER_THREAD × teams`, eight. The only code added is two arms of the harness, with the
same plan at a fixed window:
- `mibz2-svc2-w12`: `slots` 12;
- `mibz2-svc2-w16`: `slots` 16.

`mibz2-svc2` itself is the window of 8, as bench-service-2x3 ran it. No decoder changes.

## The cells

One `ab run` at THREADS = 10, interleaved by round, 20 rounds, with the arms' order rotated
as ab does:

| cell | plan | decode threads | window |
|---|---|---|---|
| `mibz2@10` | 3 × 3 pipelined, park and small (the 0.2.6 default) | 9 | 12 |
| `mibz2-svc2@10` | 2 × 3 + 2 service | 8 | 8 |
| `mibz2-svc2-w12@10` | 2 × 3 + 2 service | 8 | 12 |
| `mibz2-svc2-w16@10` | 2 × 3 + 2 service | 8 | 16 |
| `mibz2-team3@10` | 3 × 3 plain | 9 | 12 |
| `mibz2-team3-twin@10` | `mibz2-team3` under another name | 9 | 12 |

- **The other threads:** every arm runs the scanner, the reader's feeder and the consumer;
  svc2 adds `mibz2-svc-feed`. None is counted in the budget, as before.
- **The default:** `mibz2` is the pipelined 3 × 3 only on the ten-core topology (10
  physical cores, a 20 MiB L3). The report checks `lscpu.txt` for it.
- **The traces:** an occupancy pass of `--svc 2` at 10 threads at each window, 6 rounds each,
  with its `t.svc.*` figures. Occupancy gives two teams a window of `slots × 2 / 10`, so the
  passes run `--slots 40`, `60` and `80` (windows 8, 12 and 16), and the report checks each
  pass's recorded slots.

## The model (before any run)

Extrapolations, not measurements of these cells:
- svc2@10 at 8 ran 0.137 s (median); the default 0.144 s.
- If the window bound the teams' supply, the teams' 16.5% prep idle is the most a wider
  window could recover: at most about 1.20 for svc2@12 or @16 against svc2@8, and less, since
  the service workers' own 30% idle shows they were not always the bottleneck either.
- The jobs' 0.158 ms median wait in the hub says supply was usually there. So the model
  expects L1 and L2 to be NULL, with prep starvation at every window, and the window's extra
  blocks to cost a little peak RSS (more blocks' symbols and output held).

## The registered comparisons

Speed is the reference's wall over the arm's. CPU is the reference's CPU over the arm's.
Above 1, the arm is faster, or spends less. Each comparison is paired by round, with the
geometric mean and its 95% t-interval over the 20 registered rounds. **Speed and CPU are
co-primary**, each with its own outcome.

**Judged, fixed now:**
- **L1:** svc2@12 (`mibz2-svc2-w12@10`) against svc2@8 (`mibz2-svc2@10`);
- **L2:** svc2@16 (`mibz2-svc2-w16@10`) against svc2@8.

Both are 8 decode threads against 8, so the extra-thread rule doesn't apply.

**Reported, never judged:**
- **D8, D12, D16:** each svc2 window against `mibz2@10`. D8 replicates bench-service-2x3's P1;
- **the best svc2 window against the default:** whichever of D8, D12 and D16 has the highest
  geometric-mean speed, with its outcome words. It is chosen after the run from three, so its
  interval is optimistic, and it decides nothing;
- **L3:** svc2@16 against svc2@12;
- **R1:** `mibz2@10` against team3@10, as before;
- **the traces per window:** `team.idle_prep_share`, saturation, prep starvation and finish
  backlog, with the queue waits and backlogs.

## The outcome map (numeric)

As the service core's and the service-2x3 registrations, unchanged. On a ratio's 95%
interval [lo, hi], speed and CPU each:

| outcome | condition | CPU's wording |
|---|---|---|
| GAIN | lo > 1.02 | CPU SAVING |
| SMALL GAIN | 1 < lo ≤ 1.02 | SMALL CPU SAVING |
| NULL | lo ≥ 0.97, hi ≤ 1.03, and 1 inside | NULL |
| REGRESSION | hi < 1 | CPU COST |
| INCONCLUSIVE | anything else | INCONCLUSIVE |

NULL is bounded uncertainty (within ±3%, a small effect is not excluded). CPU INCONCLUSIVE is
no detected CPU cost, not a CPU cost shown to be absent.

## The reading (registered, computed mechanically by the report)

In this order:
1. **A gain at either window and a regression at neither:**
   - **1a. THE WINDOW BINDS SVC2@10 AT 8:** no gaining contrast has a CPU COST;
   - **1b. A WIDER WINDOW IS FASTER FOR A CPU COST:** a gaining contrast has a CPU COST.
     That is for Louis to weigh.
2. **INCONCLUSIVE (MIXED):** a gain at one window and a regression at the other.
3. **INCONCLUSIVE:** either is INCONCLUSIVE, and neither is a gain.
4. **A WIDER WINDOW IS SLOWER:** both REGRESSION.
5. **NO WINDOW GAIN DETECTED BEYOND 8:** otherwise (both NULL, or one NULL and one
   REGRESSION). The window doesn't bind at this resolution, and whether the prepared queue's
   lookahead binds is untested by this run.

Every reading says that the window is the blocks in flight, not the prepared queue's own
lookahead.

**The traces beside it** (reported, never judged; the service core's thresholds unchanged:
PREP STARVATION at `team.idle_prep_share` ≥ 0.10; FINISH BACKLOG at
`queue.finish.median_ms` ≥ 0.5 or `team.idle_back_share` ≥ 0.05; SATURATED at utilisation max
≥ 0.90 with either):
- **PREP STARVATION AT 8, RELIEVED AT A WIDER WINDOW:** with reading 1, starvation at 8 and
  not at 12 or 16. Consistent with the window bounding the prepared queue's supply at 8, not
  shown to be it.
- **PREP STARVATION PERSISTS AT EVERY WINDOW:** with reading 4 or 5, starvation at all three.
  Not relieved by the window. The lookahead and the service workers' order of work remain
  candidates, untested here.
- Otherwise the windows with starvation are listed, and not read.
- Any window SATURATED is named.

**Whatever the reading,** nothing is adopted by this run. A change of svc2's window, like
svc2 itself, is qualified in the usual way (the plan in the service-2x3 result: a Plan rule,
CI's paired bench under RULES C3 and C5, a workstation confirmation, the Guardian's B5 review
of the service core, and fuzz evidence to D042's standard).

## Validity (the report checks each)

The run is VALID only if all of these hold, with bench/service_2x3.py's strict checks,
imported unchanged:
- the registered 20 rounds;
- a clean tree; the frozen inputs unchanged (no `NOT-FROZEN.txt`); not a dry run;
  `commit.txt` the checkout's head;
- the ten-core topology in `lscpu.txt` (1 socket, 10 cores, a 20 MiB L3);
- every run's record usable: a hex output digest, the registered sample's 209,700,196
  decoded bytes, finite positive wall, CPU and peak RSS, the corpus `wikidata`;
- every run the same bytes as the reference `mibz2@10`'s, none failed;
- `corpus.sha256` the registered corpus's digest (82f65ace…, `bench/wikidata.json`);
- every registered cell's rounds 0–19 once each, every run with its registered plan (team
  leaders, helpers and service threads counted from ab's per-thread CPU); no other cell;
- **each of the three traces** complete and as registered: one result at 10 threads, the
  plan "the service core, mibz2-svc2's plan", 2 service threads, its slots the registered
  ones for its window (40, 60, 80, recorded as numbers), its commit the run's, parity true,
  the registered output size, and every metric the report reads finite and in range; and
  saturation resolved in each;
- **the noise control N1:** team3-twin@10 against team3@10 within 1 ± 0.03, the paired
  geometric-mean ratio of wall over the 20 rounds, both bounds of its 95% t-interval. N1 and
  its method are copied unchanged. That method is reserved to Louis (RULES C6).

**An INVALID run judges nothing.** What follows one is not decided here. Any rerun is
registered before it runs, and a question about N1's method goes to Louis (C6).

## Size and run time

- **Twenty rounds**, as for the service core and service-2x3. In service-2x3, P1's paired
  log-ratio gave intervals of about ±2.7% (speed) at 20 rounds. L1 and L2 compare the same
  plan at two windows, so their sd should be no larger than P1's, where the plans differed.
  The model's expected NULL needs both bounds within ±3%, which 20 rounds give if it holds.
- **Run time:** service-2x3's 120 runs took about 4.3 minutes. Here it is 6 cells × 20
  rounds, the same 120 runs, plus three traces of 6 rounds instead of one, about half a
  minute more. That is **about 6 minutes once built**, or about 10 minutes from a cold build.

## How it is frozen

`bench/service-lookahead.frozen` names the commit that all of these are frozen at: this
registration, the harness, the recipe (`just bench-service-lookahead`) and the report
(`bench/service_lookahead.py`, which imports `bench/service_2x3.py`'s and
`bench/service_core.py`'s predicates unchanged). The recipe refuses to run if any measured
input differs from it. The measured inputs are `src`, the Cargo files and toolchain,
`examples`, `tests/common`, the three reports and their helpers (`bench/team_size.py`,
`bench/wikidata.py`), `bench/wikidata.json`, `bench/bundle.sh`, `.just/bench.just` and this
file. The frozen commit is the commit before that file's last change, since a registration
can't name its own commit's SHA. The report calls nothing VALID but the registered run.

## What this design does not separate

- **The lookahead:** fixed at `teams`. See above.
- **The window's other effects:** a wider window also lets the reader's ready blocks and
  the scanner run further ahead, holding more memory. Peak RSS is reported; the L3 footprint
  is not measured, and RSS is not cache residency.
- **The default's window:** `mibz2@10` keeps 12. D12 compares svc2 and the default at the
  same window, D8 and D16 don't.

## Safety

Inherited unchanged. The service core is an opt-in research prototype on
`research/service-core` (#251), and this run adds no decoder code. Guardian D042's scoped
production-safety HOLD on the composed pipeline stands, pending reached-path fuzz and
mutation evidence, and this run can't discharge it. The service core's own B5 review is
still to come. Matching output hashes and green CI are not safety evidence.

## Local runs

Local runs on the shared 4-core machine are correctness checks only, never timing evidence.
The recipe's DRY run (`just bench-service-lookahead 20 1`) is untimed, and the report never
calls it VALID.
