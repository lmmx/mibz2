# The cooperative decoder, end to end: the A/B that decides the release (pre-registered before it runs)

The cooperative inverse BWT is now in the library:
- the MTF stage makes the count checkpoints (#101);
- a panic-safe pool (#103);
- the fused build, the shared room walk and `bwt::Cooperative<K>` (#104);
- a policy, `Plan::for_threads`, that chooses between today's workers and cooperative teams (this branch).

The research measured the inversion alone, pinned. This A/B measures the decoder: a `.bz2` file to hashed output, every CRC checked, unpinned, as users run it (`just bench-coop-decoder`, bench/coop_decoder.py).

## The policy under test (v1)

- **Where it keeps today's decoder:** the machine's topology can't be read (only Linux's is, so far), or the threads asked for are served by fewer than 6 physical cores (counted from the process's allowed CPUs). There it is today's `threads` workers.
- **Elsewhere, teams of 2** on the physical cores, never SMT siblings, as many as keep the live tables near 0.7 of the L3. That is the knee measured on the i9 at two block sizes (workstation journal §9):
  - 4 teams on the i9 (20 MiB L3) at 8, 10 or 20 threads;
  - 3 teams at 6.
- **Why teams of 2:** 4x2 led 3x3 in PR 0 (coop journal §17.4): 1.439 against 1.375 of p4 for the inversion, on 8 cores against 9, better balanced, and by a model cheaper in CPU.

## The arms

- `mibz2` (today's decoder at T workers);
- `mibz2-coop2` and `mibz2-coop3` (T cores as T/2 or T/3 teams);
- `mibz2-auto` (the policy at T).

They run at T = 4, 6, 8, 10 and all CPUs, 20 rounds, interleaved, back to back. A second pass is limited to one CPU on each of 4 physical cores.

## The gates (the user's, fixed before the run)

On `mibz2-auto` at all CPUs (Python's default thread count) against `mibz2` at 4 workers (production's peak on the i9), paired by round:

| gate | threshold |
|---|---|
| throughput | geometric mean ≥ 1.15; 95% lower bound > 1.05 |
| bytes | every run of every arm the same output hash, no error |
| peak RSS | ≤ 1.15 × the reference's median |
| CPU-seconds | ≤ 1.5 × the reference's median |
| a smaller machine (4 cores) | `mibz2-auto` no slower than `mibz2` there: lower bound ≥ 0.97 |

Plus the CI benchmark, on the PR: no material slowdown on its workloads. Its 2-core runners are below the policy's threshold, so there the default path must be unchanged.

A gain below the headline that is still reproducible (a lower bound above 1.00) goes to the user as a decision, with its resource costs. The gates are not moved.

## The prediction

- **The model** (the integration plan §5): 1.26–1.38× p4 for the decoder at 3x3 or 4x2. It adds the scanner, RLE1, the CRC and the joining to PR 0's inversion figures.
- **The risks:**
  - unpinned placement and SMT, the largest and untested;
  - the helpers' wake-ups between blocks;
  - the build's window checks (about 2% of a team's round, if the VM's 7–8% of the build holds).
- **The expectation:** most likely 1.15–1.35× for `mibz2-auto`, and a gain of 1.05 or less is a credible outcome. The 4-core pass should show 1.00.

## The command (the i9, browser and other busy programs closed)

```
git fetch origin perf/coop-policy
git checkout --detach origin/perf/coop-policy
just bench-coop-decoder
```

It takes about 10 minutes. Upload `coop-decoder-*.md`, `coop-decoder-*.json` and the two `.jsonl` files.
