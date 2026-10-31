# Room walk, constant-index 32-lane round: the registered result (CI run 41222735026)

**The run (measured):**
- The trial at `b145e14` (protocol `9597f3c` and amendment 1), started by
  `research/walk-unroll-ci-trigger` (`406d3e7`).
- Three `ubuntu-24.04` jobs, all built with rustc 1.97.0 and `--locked`. The binary's
  SHA-256 is `48ddfd16…`, identical in all three.
- Every job's binary passed the amended structural gates before timing:
  - 32 of 32 lane-steps keep their core;
  - 25 steps are exactly 10 instructions;
  - 10.44 instructions a lane-step, and no calls.
- Every job checked that each arm equals the plain walk on all 431 Wikidata blocks
  (209,706,557 bytes).
- One copy ran on CPU 1. The other CPUs were 0.3–0.6% busy, and steal was 0.

## Registered figures: one copy, complete walk (setup, passes, tail, assembly), 20 paired rounds

| Job | CPU | A (shipped) | B (arm) | B/A [95%] | A′/A [95%] | Counted? |
|---|---|---|---|---|---|---|
| r1 | EPYC 7763 (Zen 3) | 5.34 c/B, 1.645 ns/B | 4.91 c/B, 1.514 ns/B | **0.918** [0.914, 0.921] | 0.9965 [0.9931, **0.99998**] | **void:** A′/A excludes 1 (§6) |
| r3 | EPYC 7763 (Zen 3) | 6.08 c/B, 1.875 ns/B | 5.32 c/B, 1.641 ns/B | **0.877** [0.863, **0.891**] | 1.004 [0.985, 1.022] | **counted** |
| r2 | EPYC 9V74 (Zen 4) | 6.09 c/B, 1.617 ns/B | 5.32 c/B, 1.412 ns/B | **0.869** [0.851, 0.888] | 1.001 [0.971, 1.031] | replication |

The c/B figures are at the reported clock: 3.244, 3.242 and 3.767 GHz.

## Registered verdict (§6, mechanical): **PASS**

- **Correctness (§3):** passes. Structure (§4, amendment 1): passes on every job's
  binary.
- **The counted Zen 3 job (r3):** the B/A upper 95% bound is 0.891, below 0.95.
- **r1 is void.** Its A/A null interval ends at 0.99998, so it excludes 1 by
  2 × 10⁻⁵, which makes the job void by the registered rule. Its B/A of
  0.918 [0.914, 0.921] would have passed too.
- **The replication (Zen 4, r2):** B/A 0.869 [0.851, 0.888].

## Secondary: a copy on every CPU (both SMT siblings of both cores busy; reported, not gated)

| Job | CPU | B/A by copy (CPU 0–3) |
|---|---|---|
| r1 | 7763 | 0.920, 0.926, 0.920, 0.926 (upper bounds 0.931–0.946) |
| r3 | 7763 | 0.939, 0.947, 0.921, 0.937 (upper bounds 0.946–0.963) |
| r2 | 9V74 | 0.971, 0.980, 0.987, 0.983 (upper bounds 0.998–1.015) |

## Reading (deduced, not a registered result)

1. **The saving is larger than predicted.**
   - Measured: 0.43 to 0.77 c/B off the complete walk (−8% to −13% one copy), with
     ns/B falling by the same proportions.
   - The analysis's conditional estimate was 4.49 → 4.0–4.3 c/B on the room walk alone
     (0.2–0.5 c/B). This run's baseline includes setup, tail and assembly, and its A
     differs between the two 7763 jobs (5.34 and 6.08 c/B).
2. **The gain shrinks when the SMT sibling is busy:** −5% to −8% on Zen 3 and only −1%
   to −3% on Zen 4.
   - This fits the pack's SMT tension (D4′): with two threads on a core, a shared
     resource other than this thread's instruction stream binds, and removing
     instructions helps less.
   - **For the decoder:** gains will depend on whether workers share physical cores. The
     i9 trial's layout, 4 workers on 4 physical cores with idle siblings, is the
     one-copy case. CI's 4 vCPUs on 2 cores is the all-CPU case.
3. **The mechanism is as the analysis predicted.** The removed lane-loop bookkeeping was
   real cost, and the 32 chains were not collapsed into fewer.
4. **Next under the protocol** (not started): a PASS proposes a separate, pre-registered
   decoder-level trial. The arm stays a research arm: no production change, no merge.
