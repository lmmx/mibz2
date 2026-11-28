# 2026-11-17 · MONODROMY 46 · The busy term rebuilt from per-thread CPU, not from walls: workers get min(0.93·W, N − ν − ι) CPUs. The cache-budget rule is its special case where the CPU budget doesn't bind; at 8 threads the budget binds first, which gives 6. Per-domain L3 for multi-CCX and two-socket CPUs. Predictions, frozen before the low sweep's data

Asked by the Coordinator (09:04 and 09:07):
- *#522's cache-budget rule:* derive W_cache = min(cores, ⌊0.7 · L3 / 2 MB⌋) as a special case,
  or say why it fails at 8 threads;
- *domains:* specify the model on multi-domain CPUs;
- *the Researcher's correction (REVIEW 6943617731):* predict the busy term from something
  independent of the wall time.

**Labels:** **C** counted · **M** modelled · **P** proved · **Conj**.

## 0. The correction, accepted

**Record 44's u-match is close to an accounting identity.** With the clock flat, total cycles are
the clock times the CPU time of every thread. So cycles a byte ÷ (W · u), with u = task-clock ÷
(wall · W), equals the wall up to a constant, whatever the cause.
- *What does not follow:* its agreement within 0.01 is not evidence that idle workers are the
  cause, and its 1.7 non-worker cores is not a measurement.
- *What stands:* records 43 and 44 stay as frozen.
- *What changes:* the busy term below is built only from per-thread CPU (REVIEW 6943617731's
  samples, C):

| W (10 threads) | 10 | 9 | 8 | 7 | 6 |
|---|--:|--:|--:|--:|--:|
| worker busy share | 0.763 | 0.857 | 0.931 | 0.925 | 0.931 |
| non-worker CPU ÷ wall, ν | 1.077 | 1.093 | 1.067 | 1.004 | 0.910 |
| CPUs idle, 10 − W·share − ν | **1.29** | **1.19** | 1.48 | 2.52 | 3.50 |

## 1. The CPU budget (M on C, no wall time used)

**The law.** Each worker's busy share saturates at b₀ = 0.93: what the pipeline allows when CPUs are
free (W = 6–8). Where the workers would need more, they get what is left. The process's CPU,
N = 10, goes to the non-workers (ν, counted) and to ι CPUs that a fully subscribed set leaves idle:

  **worker CPUs = min(b₀ · W, N − ν − ι)**

ι is 1.29 and 1.19 at W = 10 and 9, the two oversubscribed arms, so ι ≈ 1.25 = **0.125 N**.

**The check.** With these two constants and the counted ν, the predicted shares are:

| W | 10 | 9 | 8 | 7 | 6 |
|---|--:|--:|--:|--:|--:|
| predicted | 0.767 | 0.851 | 0.930 | 0.930 | 0.930 |
| counted | 0.763 | 0.857 | 0.931 | 0.925 | 0.931 |

**ι on a second machine, independent of the i9 (C).** The runner's occupancy rows at 4 threads give:
- decoding share 0.86;
- CPU over elapsed a block, 1.40 / 1.58 ms.

So worker CPU is about 0.76 × 4 = 3.05 of 4 CPUs. With the scanner and reader at about 0.4–0.5,
ι ≈ 0.5 = **0.125 N** again.
- *The form:* I take ι = 0.125 N as primary, with a constant ι = 1.25 as the rival.
- *The cause (Conj):* the scheduler's slack on a fully subscribed CPU set: wake-up latency of the
  serial scanner, and run-queue imbalance.

**Cycles a byte, refined with the same inputs.** Capacity misses grow with the workers holding
footprints (W). Miss cost grows with the workers running at once, W_run = worker CPUs ÷ b₀. So

  **c(W) = a + b · W · W_run** (a = 0.422, b = 0.0068)

That gives c(10) = 0.983 against 1.000 counted. The plain W² curve gives 1.099, which was record
44's unexplained 9%. Partly idle workers keep their footprints in the cache but miss less often.

**The wall follows.** Wall ∝ c / worker CPUs. At 10 threads that gives 0.944, 0.899, 0.905 and 0.933
for W = 9, 8, 7, 6, against 0.982, 0.902, 0.885 and 0.922 counted.
- *Within the low arms:* 0.4 to 2%.
- *At W = 9:* 4% off. Its c comes out 4% under the count (0.926 against 0.966).

## 2. The cache-budget rule is the special case (P, within the model)

**The two limits.** With the wall ∝ c(W) / min(b₀ W, N − ν − ι), the best W is the smaller of two:
- **W_L3 ≈ 0.8 · L / F(s)** (record 45): the contention optimum where the CPU budget doesn't
  bind. #522's ⌊0.7 · L3 / 2 MB⌋ = 7 is this limit with its constant rounded. Mine is 7.9 at 2.1 MB.
- **W_CPU ≈ (N − ν − ι) / b₀:** past it, a worker adds no CPU, only footprint and traffic.

**So W\* ≈ min(C, W_L3, W_CPU).** The cache rule holds wherever W_CPU ≥ W_L3:
- *past the cores:* N = 20 unpinned gives W_CPU ≈ 17;
- *at 10 threads:* W_CPU = (10 − 1.08 − 1.25) / 0.93 = 8.2, against W_L3 7.9, so the cache rule gives
  7 and the measured best is 7.

**Why it fails at 8 threads:** W_CPU = (8 − 0.85 − 1.0) / 0.93 = 6.6, below W_L3. The budget binds
first.
- *The model at 8 threads:* 7 workers' share falls to 0.88, and 6 against 8 is 0.940. Counted:
  0.929.
- *Its miss:* the model puts 7 level with 6 (0.942). Counted, 7 is 0.975, worse than 6 by 5%. So
  the budget term is not yet sharp enough there.

## 3. Domains (M)

**What `l3_bytes()` gets wrong.** It reads cpu0's `index3`, one L3. On a CPU with D L3 domains (AMD
CCXs or CCDs, two sockets), the model needs, from sysfs for the CPUs in the affinity:
1. **the domains spanned:** group the allowed CPUs by `cache/index3/shared_cpu_list`, with each
   domain's size L_d;
2. **each domain's cores and CPUs:** C_d and N_d, via `topology/core_id` within the domain.

**The L3 term becomes per domain:**

  W_L3 = Σ_d min(C_d, 0.8 · L_d / F(s))

This holds only if the workers are placed n_d ≈ min(C_d, 0.8 L_d / F) a domain. Linux's balancer
spreads runnable threads across LLC domains, but it does not guarantee proportion.
- *Placement:* give each worker a soft affinity to its domain's CPU set, not to a CPU. Worker i
  goes to the domain with the most room left.
- *The block's input and output:* they cross domains once a block, about 2 bytes a byte. A
  remote-L3 or NUMA transfer of 0.9 MB a block is about 0.1 ms, against a block's decode of
  1 ms or more. That is a small constant, not a contention term (M).

**The CPU budget stays process-wide.** N counts the allowed CPUs; ν and ι are process-wide.
- *On SMT,* N is in core-equivalents: C + (σ − 1)(N_logical − C).
- *On two sockets,* the scanner's domain also holds its share of workers.

**On the runner:** one L3 instance spans all 4 vCPUs, so D = 1 there.

## 4. Predictions, frozen before the low sweep's data (M)

**The inputs.** Lazy workers, N = T pinned. ν scales with throughput: about 0.85, 0.7 and 0.5 at 8,
6 and 4 threads. Ratios are against lazy at T workers.

| T | best W, ι = 0.125 N (primary) | wall ratios | best W, ι = 1.25 (rival) | wall ratios |
|---|---|---|---|---|
| 8 | **6** | 6: 0.940 · 7: 0.942 · 5: 1.002 | 6 | 6: 0.919 · 7: 0.944 · 5: 0.979 |
| 6 | **5** | 5: 0.946 · 4: 1.044 | 5 | 5: 0.951 · 4: 0.964 |
| 4 | **4** | 3: 1.019 · 2: 1.42 | **3** | 3: 0.966 · 2: 1.11 |

**What tells the two ι forms apart:** each worker's busy share at T workers.

| T | primary (ι ∝ N) | rival (ι constant) |
|---|---|---|
| 4 | 0.75 | 0.56 |
| 6 | 0.76 | 0.67 |
| 8 | 0.77 | 0.74 |

The primary predicts about 0.76 at every T: ι, ν and N all scale together. This counter needs no
wall time.

**Against record 44** (frozen, kept): the same best W (6, 5, 4) as the primary, now from CPU
accounting. Its magnitudes differ (5 against 6 at 8 threads: 1.066 here, 1.02 there).

**Refuted if:**
1. the share at T workers differs from 0.76 by more than 0.05 at 4 and 6 threads, and from the
   rival's too; or
2. the best W at 6 threads is 4 by more than 2%, under both forms; or
3. c(W) at W = 2–5, against a + b · W · W_run, is off by more than 5%.

## 5. For #522

**The rule I'd put in the plan**, if the low sweep confirms it:

  **W\* = min(C, Σ_d min(C_d, 0.8 L_d / F(s)), ⌊(N − ν − 0.125 N) / 0.93⌋)**,

checking the floor and the ceiling of the last term.

**Its inputs.**
- *Topology:* C, N, the domains and L_d, all read at runtime.
- *Block size:* s, from the stream header.
- *Non-worker load:* ν is the one input not read today. It is the non-worker CPU a byte times
  throughput, about 0.11 CPUs per worker-equivalent on Wikidata. A constant per kernel suffices.
