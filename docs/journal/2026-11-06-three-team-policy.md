# The three-team policy: admitting 3 × 3 on the ten-core topology

Registered before any run, on 2026-11-06.

## The change

`Plan::for_threads` (`src/parallel.rs`, `plan_with`) chooses the cooperative decoder's plan.
Until now it used teams of 2, as many as keep the live tables near 0.7 of the L3, which is
four teams on the i9-10900K. This change adds one exception:
- **When:** 10 physical cores, a 20 MiB L3 (as `l3_bytes` reads it), and 10 or more threads
  asked for.
- **What:** three teams of three, 9 decoding threads, in a default window of 12 blocks.
- **Everything else:** the old rule, unchanged, including 8 and 9 threads on the i9, which
  stay at four teams of two.

**Why 3 × 3.** #220 qualified 3 × 3 at 10 threads and 3 × 4 at 12 against four teams of two
at 8 (`2026-11-06-three-tables.md`, two runs each). It ranked neither. The Optimiser's design
picked 3 × 3 (#145, its REVIEW 6693638662): about the same speed as 3 × 4, with about 15% less
CPU.

**What it reaches beyond what was measured.**
- **Thread counts:** only 10 was measured. At 11 or more threads the policy also picks 3 × 3,
  which leaves threads unused that the old plan didn't use either.
- **Machines:** any CPU with exactly 10 physical cores and a 20 MiB L3 gets the change, not
  only the i9-10900K. The planner has no CPU model to check.
- **Workloads:** only the Wikidata sample at level 9. Other corpora and levels are covered for
  correctness, not speed.

## The evidence, as Louis decided (`decisions.md`, 2026-11-06)

1. **CI:** the paired bench shows no regression. On its 4-core runners, both builds take the
   old path.
2. **Correctness tests on the changed plan, without the hardware:**
   - the plan table with the topology injected: every row, both boundaries, the budget;
   - three teams of three, through the planned decoder, byte for byte against libbzip2 at
     every level and shape, on several streams;
   - corrupted, truncated and empty input failing as four teams of two fail;
   - windows too small for a block a team.
3. **This registration:** one workstation A/B and its confirmation, below.

## The run

`just bench-three-team-policy` on the i9, at this branch's frozen commit.
- **The arms:** the head build's `mibz2` against the base build's (`mibz2@<base ab>`). Both use
  the default plan, unpinned, as a caller gets it. The base is the head's merge base with
  master; the recipe records both commits.
- **The grid:** 8, 9, 10, 11, 12, 16 and 20 threads, 20 rounds, paired by round, on the bench's
  Wikidata sample.
- **The plans each build must run,** read from every run's thread names:

  | threads | base | head |
  |---|---|---|
  | 8, 9 | 4 × 2 | 4 × 2 |
  | 10, 11, 12, 16, 20 | 4 × 2 | 3 × 3 |

- **The output:** one directory per attempt, its console log included, packed into one tarball.

## The rule

`bench/policy_ab.py`, fixed with this registration. Speed is the base build's wall over the
head's, the geometric mean of the 20 rounds' ratios.

- **Validity:**
  - 20 rounds a cell, each once, no failed run;
  - one output hash and one length across every run;
  - every run's plan as in the table.
- **Controls, at 8 and 9 threads:** as #220's control, flagged when a speed ratio's 95% interval
  lies wholly beyond ±2%. A flagged run fails.
- **Primary, at 10 threads:** speed at least 1.03, with its two-sided 95% lower bound above
  1.00. The head's median CPU-seconds at most 1.5 times the base's, and its median peak RSS at
  most 1.15 times.
- **No harm, at 11, 12, 16 and 20 threads:** each head-over-base wall ratio's one-sided upper
  bound at most 1.02. The bound is Bonferroni over the four: t at 1 − 0.05/4, 19 degrees of
  freedom. An interval too wide to show it is inconclusive, and fails.
- **Admission:** the run passes only if every predicate holds. The policy is admitted only if
  one run and its predeclared confirmation, a second run of the same recipe at the same commit,
  both pass. No pooling, no switching cells, no third run.
- **Invalidation:** one replacement run in the whole experiment, for a run that isn't valid.

## The prediction

- **At 10 threads:** about 1.08. #220 measured 1.076 and 1.087 for 3 × 3 at 10 against 4 × 2 at
  8. The old default at 10 runs 4 × 2 too, and #220 measured `mibz2@10` at 0.999 against
  `mibz2@8`.
- **At 11 to 20 threads:** about the same, since the plan and window are the same.
- **CPU:** about 0.9.
- **Peak memory:** about 0.82.
- **At 8 and 9 threads:** no change.

## What each outcome would mean

- **Both runs pass:** the policy merges, with this PR's CI green.
- **Either fails:** it doesn't merge. The report names the failed predicate, and the change
  stays unmerged until a new registration.

## Frozen

Not yet. It freezes when the Guardian and the Researcher have reviewed it.

## Result

Not yet run.
