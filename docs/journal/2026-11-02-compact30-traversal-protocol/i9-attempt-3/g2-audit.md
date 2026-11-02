# The third i9 capture: a G2 audit from the archive (report 183)

**Outcome:** the capture stays INVALID. G2 failed, and nothing here changes a threshold or
the registered outcome. The ratios below are descriptive. They are not an approved result.

**Recommendation:** a concrete measurement defect. A separately registered correction,
tested in an independently authorised capture, is warranted (the last section).

**The defect:**
- The W4 start barrier is `std::sync::Barrier`, which puts waiting workers to sleep in the
  kernel.
- Its release is near-instant: 1.4 µs from the last arrival to its own walk's start.
- The other three walks start only once the operating system has woken their threads. In
  most steps one of them takes longer than 20 µs.
- 2% of a step is 39 µs, so one slow wake-up fails the gate.
- The first arm of each round adds a second effect: its cores come from about 0.7 s of
  idling during the W1 passes.

## The capture

- **The run:**
  - archive `c30-traversal-20261102T191057Z.tar.gz`, sha256 `3d6667c9…`, kept with
    report 183 on the reports branch;
  - source pin `4635e2c`, with no tracked changes;
  - rustc 1.97.0 / LLVM 22.1.6;
  - binary sha256 `062b89bb…`, the same binary as the second attempt;
  - its `room-false.bounds` reads the i9's own symbol size, 0x20aa.
- **The checks:** every schedule check (S0–S9) passed, as did G1, G3, G4 and G5. G2 failed.
- **The figures (descriptive):**

| figure | C / P | 95% interval |
|---|--:|---|
| w1 | 1.13752 | 1.13613–1.13892 |
| w4 | 0.84187 | 0.83946–0.84429 |
| e4 | 0.85650 | 0.84340–0.86980 |

`g2_audit.py` reproduces every table below from the archived records:

```
python3 g2_audit.py c30-traversal.jsonl c30-analysis.json
```

`g2-audit.txt` is its output.

## 1. The gate, recomputed

G2 (protocol §5) takes each all-full W4 step (four walks of at least 899,000 rows). Its
skew is the latest walk start minus the earliest, over the step's span. These are
row-weighted per arm and round, and every arm and round must be ≤ 0.02.

- The audit recomputes this from the raw records. It matches the analysis to 3e-17.
- 38 of the 40 arm-rounds exceed 0.02. The largest is 0.0796.
- The data covers 2,280 all-full steps. No walk changed CPU.

The guardian's figures reproduce exactly:

| steps | mean | median | share over 0.02 |
|---|--:|--:|--:|
| first arm of the round | 0.0538 | 0.0394 | 0.839 |
| second arm of the round | 0.0227 | 0.0248 | 0.682 |

The second arm would fail too. Its median step is over 2%, though only just.

## 2. Barrier release and actual start

Each W4 step is: prepare, `barrier.wait()`, then the walk, which reads its clocks first. The
release is the latest preparation end of the four: the moment the last worker arrived.

| | median | p90 | max |
|---|--:|--:|--:|
| the last arrival, release to its walk's start | 1.4 µs | 1.9 µs | 27.7 µs |
| the three sleepers, release to start | 26.1 µs | 154.5 µs | 356 µs |

- **The release itself is near-instant.** In 2,250 of 2,280 steps (98.7%), the first walk
  to start belongs to the last worker to arrive. No walk started before the release.
- **The skew is the sleepers' resume.**
  - `std::sync::Barrier` is a mutex and a condition variable. Workers that arrive early
    sleep in the kernel.
  - The last arrival wakes them all and carries on.
  - Each woken thread must then take the mutex again. One that loses that race sleeps a
    second time, until the holder releases it.
- **The sleepers wake in a queue, not all at once:**

| arm position | 1st sleeper to start | 2nd | 3rd |
|---|--:|--:|--:|
| first | 3.9 µs | 51.4 µs | 76.8 µs |
| second | 3.5 µs | 4.5 µs | 45.1 µs |

- **One slow wake-up decides the gate:**

| arm position | steps with a sleeper slower than 20 µs | skew median, none slow | skew median, some slow |
|---|--:|--:|--:|
| first | 88.5% | 0.0016 | 0.0451 |
| second | 75.6% | 0.0015 | 0.0278 |

When every sleeper wakes quickly, a step's skew is about 0.15% of its span, an order of
magnitude inside the gate.

## 3. The first-arm effect

- **It is in every round:** the first-arm/second-arm skew is 2.9/2.0% in round 0 and 4.7/2.5%
  in round 19, and 4.4–8.0% against 1.6–2.8% across the other rounds.
- **It is position, not arm:**
  - P first 0.0482, C first 0.0595;
  - P second 0.0244, C second 0.0210.
- **It fades within the arm** (median skew ratio):

| steps of the arm | first arm | second arm |
|---|--:|--:|
| 0–4 | 0.0690 | 0.0031 |
| 5–14 | 0.0809 | 0.0245 |
| 15–29 | 0.0649 | 0.0262 |
| 30–59 | 0.0293 | 0.0251 |

**What comes before each arm:**
- The first arm follows the round's two W1 passes, which run on worker 0's CPU alone. The
  other three cores have then idled for about 0.7 s (the median for both passes).
- Each arm also spawns four new threads.
- In the second arm, all four cores have just run the first arm.

**A consistent reading** is idle-state exit and the idle governor's history. It takes some
tens of steps for wake-ups to return to the second arm's level. The records can't
distinguish this from other scheduler effects. They do show that it is tied to the arm's
position, and that it decays with steps.

**By worker:**
- Workers 2 and 3 are the last arrivals in 87–92% of steps: their preparations finish last.
- In the second arm, sleepers on workers 0 and 1 resume in a median 3.8 µs, against about
  40 µs on workers 2 and 3.

## 4. What the skew does to the figures

**w4 (CPU cost):** independent of the order and of the skew.

| per-round C/P | P first | C first |
|---|---|---|
| w1 | 1.1372 (1.1350–1.1437) | 1.1379 (1.1320–1.1429) |
| w4 | 0.8407 (0.8335–0.8477) | 0.8431 (0.8332–0.8499) |
| e4 | 0.8324 (0.8162–0.8458) | 0.8815 (0.8530–0.8994) |

- Across 1,140 paired steps (the same round, step, blocks and workers), the C-minus-P skew
  difference doesn't predict log(C CPU / P CPU): slope −0.022, r = −0.027. At the mean
  difference that is a shift of −0.00009.
- A late start lowers no thread's CPU time: each walk's CPU clock starts only when it does.

**e4 (completion time):** depends on the order.
- A step's span runs from the earliest start, so the late wake-ups are inside it. The first
  arm's larger skew lengthens whichever arm runs first.
- e4 is 0.832 when P runs first and 0.882 when C does.
- Alternating the order balances the mean, but widens e4's interval. e4 enters the decision
  rules, so the skew affects the registered decision through it.
- The common overlap of the four walks follows the same pattern: P 0.794/0.834 and C
  0.757/0.815 (first/second arm).

## 5. Did G2 measure the intended property?

**The intent:** protocol §5 says G2 "checks the barrier, not the walks' lengths". It is meant
to show that the four walks start together, so that their contention is the one measured.

**What it measured:** the start dispersion of the walks. That includes the barrier's release,
which is near-instant, and the operating system's resume of the three sleeping workers, which
is not.

**So the gate was right to fail.** The walks really did start up to 200 µs apart, and that
moved e4. It is a real lack of simultaneous execution, not an artefact of the formula.

**The threshold suits the property, but the mechanism can't meet it.**
- Without a slow wake-up, the skew is about 0.15% of a step.
- A sleeping barrier's slow path, one futex wake-up of tens of µs, already exceeds 2% of a
  1.9 ms step.
- Dry runs suppress G2, and VM timings aren't read, so the dispersion of the harness's
  barrier on the target was never seen before the capture.

That is the measurement-design defect.

**What it rules out:**
- a fault in compact30's walk: the skew is the same in P;
- a fault in the gate's arithmetic: it reproduces;
- an outlier: 38 of 40 arm-rounds fail, in every round.

## 6. The recommendation: one separately registered correction

**Keep this capture INVALID.** Its ratios are descriptive and decide nothing. The
recommendation is a protocol revision with one change, tested only by an independently
authorised capture.

**The change, in the collector's synchronisation only:** a spinning start barrier for W4.
- An arrival count and a generation number, both atomic.
- Workers that arrive early spin on the generation with a pause hint, so none is asleep at
  the release.
- The last arrival's increment releases all four.
- The completion barrier may stay as it is: walk end times are recorded before it.

**Spinning costs nothing in the measured windows.**
- Each worker is pinned to its own physical core.
- The spin is outside the walk window. Thread CPU time is read inside the window, so the
  spin enters neither w1 nor w4.

**Unchanged:**
- the compact30 walker and its compiled path;
- the arms, the workload, the allocation and the corpus;
- G2's 2% threshold and every other gate and decision rule;
- the toolchain;
- the original measurements.

**Registered before the capture, so the test can fail:**
- Under a spinning barrier, the median step skew should be near this capture's fast-path
  level (about 0.0015) in both arms, and the first/second-arm gap should close.
- If G2 still fails with a spinning barrier, the skew isn't the barrier's wake path. The
  recommendation then becomes: retain INVALID and stop.

**Validation without timing:**
- on the VM: the schedule checks, the injected failure paths, and that a failure exits
  while other workers spin;
- then the i9, only with separate authorisation.

**Not proposed:**
- a different threshold;
- excluding slow steps;
- re-analysing this capture under a new rule;
- persistent threads, warm-up steps or any second change. Spinning alone removes the
  sleeping wake-up that both effects ride on.
