# Addendum (descriptive, post hoc): the capacity-aware opportunity on the recorded schedule

**Status:**
- Added 2026-11-02, after external reports 155 and 158.
- This does not change the registered result, the closure (§8) or any class figure:
  - MPMC is still rejected under its registered criterion;
  - wake-affine placement is still an untested hypothesis.
- Nothing here was run on the i9. The calculation reads the archived capture only.

## What these numbers are, and what they are not

They are **opportunity estimates on the recorded schedule:** how much of the recorded time a ready worker and an idle
allocated CPU coincided, capped by what could be filled.

They are **not:**
- a simulation of singleton worker affinity, or of any other policy. Once worker masks change, job claims,
  support-thread execution and phase overlap all change;
- a guaranteed recoverable range: the sensitivity row is a sensitivity analysis, not a lower bound;
- a throughput prediction. Any conversion to throughput assumes an unchanged cost per byte. The walker ladder shows that
  cost rises with concurrency, and nothing measures the marginal cost of the extra activity inside this decoder.

## The question (report 155)

The registered class `runnable.mutex_wait.cpu_idle` (5.65% of W) credits each runnable worker separately whenever any
allocated CPU is idle. It is capped neither by the number of idle CPUs nor by the queued jobs. The capacity-aware figure
is

    E = (1/T) ∫ min(I(t), R(t), Q(t)) dt

in average cores, where:
- **I** = idle allocated CPUs;
- **R** = workers runnable in the receiver's mutex-wait phase;
- **Q** = queued, unclaimed jobs.

## Method

**Script:** `analysis/capacity_E.py`. Results: `analysis/capacity_E.json`. Run from the repository root:

    python3 -I docs/journal/2026-11-01-dispatch-causality-i9-10900K/analysis/capacity_E.py . \
        docs/journal/2026-11-01-dispatch-causality-i9-10900K/archive/capture \
        docs/journal/2026-11-01-dispatch-causality-i9-10900K/analysis/capacity_E.json

**What it reuses:** `bench/dispatch.py` at the pin `48f8cda`, the frozen analysis, unchanged on this branch. Imported
from it:
- `load_marks`, `load_perf`;
- `align`, with its correction and its uncertainty u;
- `sched_reconstruct`, `Step`, `PHASE_OF`, `role_of`, `KV`.

**`run()`'s breakpoint loop is copied unchanged.** That keeps:
- the same breakpoints, each mark's ±u clock guard included;
- midpoint sampling;
- the queued-work gate;
- the unknown-state guard.

**The definitions:**
- **T:** the frozen span, from the first mark to the last `Taken` or `DecodeEnd`. W = 4T. Pooled, T = 2.3567 s and
  W = 9.4269 worker-s, as in §3.
- **I:** CPUs among the four allocated whose occupant is pid 0. This is the frozen test behind `cpu_idle`. No unknown
  occupant occurs inside the span.
- **R:** the workers the frozen loop places in `runnable.mutex_wait.*`. The variant adds `runnable.recv.*`.
- **Q:** the frozen `avail` count: jobs enqueued and not yet past `RecvDone`. Two further definitions give the same E
  to four decimals:
  - "literal": `Issued` (or `Enqueued`) to `JobGot`;
  - "strict": `avail` less the workers running in mutex wait or recv.
- **The sensitivity row:** I less our own scanner, reader and feeder threads that are runnable but not running, which
  also want an idle CPU.
- **Unresolved intervals** (`unresolved.clock` 0.065 s, `unresolved.send_bracket` 0.104 s, `unresolved.sched` 0) are
  excluded from R exactly as the frozen classifier excludes them. If all of them were fillable, E would be at most 5.81%.
- **One mechanical point.** The frozen classification stays on the frozen segments. Extra change points, needed only by
  the literal-Q and sensitivity variants, subdivide those segments and never move the frozen classes.

**Validation:**
- In all 10 runs, the copied loop's class integrals match the archived per-run `classes_s` exactly (max abs diff 0.0 s,
  every class).
- R gated on I > 0 reproduces `runnable.mutex_wait.cpu_idle` to the nanosecond.
- A second run of the committed script from this tree reproduced the scratch result byte for byte.

## Results

Pooled = Σ∫ ÷ ΣT. Per run: the median and range over the 10 runs. % of 4 cores is the same as % of W.

| quantity | pooled cores | pooled % of 4 cores | per run, % of 4: median [min, max] |
|---|--:|--:|--:|
| R, any occupancy (mutex wait) | 0.3220 | 8.049 | 7.99 [7.09, 9.14] |
| R while a CPU is idle: the registered class, reproduced | 0.2261 | 5.653 | 5.70 [4.97, 6.57] |
| min(I, R) | 0.2251 | 5.627 | 5.70 [4.95, 6.53] |
| **E = min(I, R, Q)** | 0.2251 | 5.627 | 5.70 [4.95, 6.53] |
| E, Q strict | 0.2251 | 5.627 | 5.70 [4.95, 6.53] |
| E, I less our own runnable scanner, reader and feeder (sensitivity) | 0.1779 | 4.447 | 4.41 [3.52, 5.25] |
| recv variant: R while a CPU is idle | 0.2600 | 6.499 | 6.43 [5.59, 7.44] |
| recv variant: E | 0.2571 | 6.426 | 6.36 [5.51, 7.36] |
| recv variant: E, sensitivity | 0.2066 | 5.166 | 5.26 [4.00, 6.22] |


**Which cap binds,** over the 0.5305 s (pooled) with R > 0 and I > 0:

| | share of that time |
|---|--:|
| none binds | 99.55% |
| I binds (R = 2, I = 1) | 0.25% |
| I and Q tie | 0.20% |
| Q binds | 0 |

In the recv variant, Q binds for 0.12 ms in all.

**Why the caps hardly bind:**
- R = 1 for 99.55% of that time, and I = 1 for 95.9% of it.
- The receiver mutex hands off to one waiter at a time. The other waiters are asleep (`rx.blocked_by_owner`), not
  runnable.
- So the per-worker sum was already almost capacity-aware in this trace.

**Placement of E's mass:**
- 87.2% falls in runnable intervals placed on the waker's CPU (0.4626 of 0.5305 s). The test is the wakeup's
  `target_cpu` against the CPU that issued `sched_waking`, as matched by `analysis/waker.py`.
- 12.8% was placed elsewhere.
- The recv variant has 77.3% on the waker's CPU.
- This shows where the kernel placed the thread, not why.

**Support demand:** in about 21% of E's mass, one of our own scanner, reader or feeder threads was also runnable and
waiting while a CPU was idle. That is the sensitivity row: 0.178 cores (4.45%).

### Per run (mutex-wait R)

| run | span T (s) | clock uncertainty u (ns) | class check (max abs diff, s) | registered class (cores) | E (cores) | E, sensitivity (cores) |
|---|--:|--:|--:|--:|--:|--:|
| w4-r00 | 0.2338 | 194.5 | 0.0 | 0.1988 | 0.1979 | 0.1407 |
| w4-r01 | 0.2303 | 255.5 | 0.0 | 0.2366 | 0.2350 | 0.2052 |
| w4-r02 | 0.2354 | 250.5 | 0.0 | 0.1994 | 0.1994 | 0.1592 |
| w4-r03 | 0.2347 | 250.0 | 0.0 | 0.2602 | 0.2583 | 0.2015 |
| w4-r04 | 0.2340 | 228.5 | 0.0 | 0.2033 | 0.2032 | 0.1470 |
| w4-r05 | 0.2388 | 282.5 | 0.0 | 0.2412 | 0.2408 | 0.1860 |
| w4-r06 | 0.2379 | 259.5 | 0.0 | 0.2289 | 0.2287 | 0.2018 |
| w4-r07 | 0.2440 | 201.5 | 0.0 | 0.2269 | 0.2269 | 0.2100 |
| w4-r08 | 0.2331 | 253.0 | 0.0 | 0.2026 | 0.1992 | 0.1591 |
| w4-r09 | 0.2347 | 258.5 | 0.0 | 0.2630 | 0.2610 | 0.1668 |


### Inputs

The capture files as committed at `4ef5f36` (the archive `i9-10900K.tar.gz`, SHA-256 `d73d330b…`, as uploaded) and the
frozen analysis module:

| input | SHA-256 |
|---|---|
| `archive/capture/w4-r00.perf.txt.gz` | `0dd647c743820d0e6251360d1149eb598c0dae03e3fce4ac42c9b3b4b2930840` |
| `archive/capture/w4-r00.tsv.gz` | `61dd6095aa3d12ffe747e13690696ca09cda98087adb5f7de3b4533c83e318fa` |
| `archive/capture/w4-r01.perf.txt.gz` | `a01e69069e8de64357a3f212e431403ce570f29189967a7823b46847c11f1f59` |
| `archive/capture/w4-r01.tsv.gz` | `7fadc6aec46ff61c4691930f6a83d14a935dc24b240cbe49a01fc319f6ad94f5` |
| `archive/capture/w4-r02.perf.txt.gz` | `ce9a87c699f1127c3074f3932cb450fc697e72e78654d2974ad3b6a8ce1ad367` |
| `archive/capture/w4-r02.tsv.gz` | `c02bb95649028cc0414b2e495b292f330cdc44b2e534c31ad76f157a0ae3953b` |
| `archive/capture/w4-r03.perf.txt.gz` | `e81ae3c1996abee760e9b3ac44206bec448b0bcdd0681f3fb801a6850aa19196` |
| `archive/capture/w4-r03.tsv.gz` | `d917597b76de94ec036c7a8e24cec06c412fb70f7c6b92d406ed0af6bd9aab6b` |
| `archive/capture/w4-r04.perf.txt.gz` | `cf95ec5d37e4928399d941e8aa54dfc5f21b7ebf4c498cd904aa2e0273af75c1` |
| `archive/capture/w4-r04.tsv.gz` | `f90eb2defc37e8084e7c118c10802db950c41e372a5313e158346f4834375ccb` |
| `archive/capture/w4-r05.perf.txt.gz` | `0d9754a38af396acbdae0e2f68210e0d16d8178398c31ed75310b0bf13307880` |
| `archive/capture/w4-r05.tsv.gz` | `433e76e934567eb407d79c60a89fb9907fb521ac1cd488276f97d35cd9fd16dc` |
| `archive/capture/w4-r06.perf.txt.gz` | `d0ef5571dbe373630dceb4671e3a545dc61962632139054617bc5c71f5fa5c9a` |
| `archive/capture/w4-r06.tsv.gz` | `5118ecd29ac8579e00635bcc6ddfab3d60411d67f18e99950b19fa671f211e92` |
| `archive/capture/w4-r07.perf.txt.gz` | `0094c98d7e7f534ed593849e8374e69f63cf55c8c56200e04e667cfe43bac271` |
| `archive/capture/w4-r07.tsv.gz` | `bf924c909b4e3cc162ad8aab4d384180c8fb93fbd956911ba6de28464bff793e` |
| `archive/capture/w4-r08.perf.txt.gz` | `17c82e5beb0212b61ce3f6f961e63dd4595bf15c4baf037fca286bcbb63b5a4d` |
| `archive/capture/w4-r08.tsv.gz` | `5374d13feb537fdb1cc3923e130f6a32c6d030950ee6c85fb5a134c1843d1202` |
| `archive/capture/w4-r09.perf.txt.gz` | `29cef5465932fc317ba6a84dc59958cce50b97ab8e07b64a10e4fd505528fb78` |
| `archive/capture/w4-r09.tsv.gz` | `7d442431c3cb025c58eb0d6305b0f9c7c87f5acb63c22b409f59328bffad7761` |
| `bench/dispatch.py` | `cf814e42499a81825f26a10f4cc671012c0b92a958ba635a01a22f49afa49e02` |

## Caveats

- **Idealised:** lock acquisition is taken to cost nothing, and every CPU is taken as eligible for every thread.
- **"Idle"** means a switch to pid 0 on the four allocated CPUs. A foreign task there counts as busy.
- **The variants' choices are named above.** Each changes E by under 0.01 point except the sensitivity row.
- **Post hoc:** this figure was not registered. It answers a reviewer's question about the opportunity's size, not the
  diagnostic's.
