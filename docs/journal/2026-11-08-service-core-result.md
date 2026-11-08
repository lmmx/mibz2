# The service core: the registered run's result

8 November 2026. This is the decision run registered in `2026-11-07-service-core-registration.md`,
frozen at 9ef3c17d (#251). The first attempt was INVALID on N1 alone
(`2026-11-08-service-core-first-attempt.md`), and this rerun was registered before it ran.

## The run

- **Bundle:** `service-core/20261108T103216Z.tar.gz` on `evidence` at 1e0bf7dc.
  - Its SHA-256 is 0be40ca542739c7e47ed6b1b447af4fa46447d178920d9365b409c7d38245ee3, which
    matches the notice on #145.
  - It was run at 1394dd97 with `just submit`.
- **VALID:** every check passed.
  - The noise control N1 (team3-twin@10 against team3@10) came out at 1.003 [0.988, 1.018].
  - 20 rounds in every cell, the same bytes from every run, every cell's plan as registered,
    and all six traces complete.

Speed is the reference's wall over the arm's. CPU is the reference's CPU over the arm's, so
above 1 the arm spends less.

## The registered comparisons

| | arm | against | decode threads | speed | outcome | CPU | outcome |
|---|---|---|---|---|---|---|---|
| C1 | svc1@10 | team3@10 | 10 / 9 | 0.854 [0.840, 0.869] | REGRESSION | 1.049 [1.019, 1.080] | SMALL CPU SAVING |
| C2 | svc1@10 | mibz2@10 | 10 / 9 | 0.799 [0.786, 0.812] | REGRESSION | 1.099 [1.079, 1.118] | CPU SAVING |
| C3 | svc2@12 | team3@12 | 11 / 12 | 1.201 [1.181, 1.221] | GAIN | 1.206 [1.187, 1.226] | CPU SAVING |
| C4 | svc2@12 | mibz2@12 | 11 / 9 | 1.058 [1.035, 1.081] | GAIN | 0.901 [0.888, 0.914] | CPU COST |
| D1 | svc2@12 | svc1@12 | 11 / 10 | 1.318 [1.297, 1.339] | GAIN | 0.806 [0.789, 0.823] | CPU COST |

**The architecture's reading: A TRADE.** The service core gains over today's default (C4) for
a CPU cost. Under the registration, that is for Louis to weigh.
- **The extra thread:** C4's gain is NOT SHOWN BEYOND ITS THREADS. Eleven decode threads
  against nine would need a lower bound above 1.222.
- **The registered counterexample: SATURATION BINDS AT K = 1.**
  - One service thread is 96% busy at every budget, with a finish backlog. The teams wait on
    it: 107 ms of run wait at 10 threads.
  - Two service threads are 67–85% busy and unsaturated, and D1 makes K = 2 32% faster at the
    same teams.

**Medians:**

| cell | threads (leaders + helpers + service) | wall s | CPU s | peak RSS MB |
|---|---|---|---|---|
| mibz2@10 (today's default) | 3 + 6 + 0 | 0.143 | 1.025 | 74 |
| svc2@10 | 2 + 4 + 2 | 0.136 | 0.894 | 64 |
| svc2@12 | 3 + 6 + 2 | 0.134 | 1.130 | 83 |
| svc1@10 | 3 + 6 + 1 | 0.178 | 0.925 | 76 |
| team3@10 | 3 + 6 + 0 | 0.152 | 0.975 | 67 |

## What it shows

**Taking the serial work out of the teams works, but only with enough service threads.**
- With one service thread, the work moves to a thread that becomes the bottleneck (C1, C2).
- With two, the teams are fed. svc2@12 is 20% faster than the plain three teams at about equal
  threads (C3, 11 against 12), and 5.8% faster than today's pipelined default (C4).

**Reported, not judged: S4.**
- svc2@10 runs two teams of three plus two service threads, eight decode threads in all. On
  10 threads it is 1.045 [1.023, 1.068] faster than today's default.
- It does that with **1.138 [1.118, 1.158] of the default's CPU, so 12% less**, and 64 MB
  against 74 MB of peak RSS.
- Fewer threads, less CPU and less memory, yet faster: if it holds, it dominates the default on
  every count. It is a reported cell, so it decides nothing.
- It is the strongest candidate this campaign has produced, and it needs its own registration.

**What it doesn't separate:** whether svc2@10's advantage comes from the service threads, or
from running two teams rather than three. The run has no plain 2 × 3 cell at 10 threads.

## What it decides, and what it doesn't

- **The registered architecture reading is A TRADE.** svc2@12 is 5.8% faster than the default
  for 11% more CPU and 8 MB more peak RSS. That is Louis's to weigh. Nothing is adopted.
- **The next registration** is svc2@10 (2 × 3 + 2) against today's default, with a plain 2 × 3
  cell to separate the cause. It is to be qualified as a production change in the usual way,
  with CI's paired bench (C3).
- **The service core is a prototype on research/service-core (#251).** It is not wired into the
  default.

## svc2@12: not selected for production

The registered architecture reading at 12 threads is A TRADE (C4): 1.058 [1.035, 1.081] faster
than today's default, at 0.901 [0.888, 0.914] of its CPU. Its gain is not shown beyond its extra
threads: 11 decode threads against the default's 9, and the extra-thread rule needs a lower
bound above 1.222. A gain bought with more threads, at a CPU cost, is no reason to change the
default. svc2@12 is therefore not selected (research direction, RULES A5).

The line followed instead is svc2@10. It runs fewer threads than the default (8 against 9), and
its reported cell is faster, cheaper in CPU and smaller in memory. Its registered comparison,
with plain and pipelined 2 × 3 controls, is `bench-service-2x3` (#258). Until that run, the
reported figures are not a verdict.
