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

## The result (measured, the i9, 34c6ef7, 2026-10-29): every gate passed

The run was `coop-decoder-20261029T174657Z`. The files are beside this entry:
- the report (`.md`);
- its figures (`.json`);
- both passes' raw rounds (`.jsonl`).

It ran 20 rounds, interleaved, unpinned and back to back, with the 4-core pass at 10 rounds on CPUs 0–3, one CPU of each of 4 cores. All 400 runs of the full pass and the 20 of the 4-core pass gave one output hash, with no error.

**The gates** (`mibz2-auto` at all 20 CPUs against `mibz2` at 4 workers, paired by round):

| gate | threshold | measured | |
|---|---|--:|---|
| throughput | mean ≥ 1.15 | **1.257** | pass |
| | 95% lower bound > 1.05 | **1.197** (1.197–1.320) | pass |
| bytes | one hash, no error | 1 hash, 0 errors | pass |
| peak RSS | ≤ 1.15× | 1.015 (74 against 72 MB) | pass |
| CPU-seconds | ≤ 1.5× | 1.214 (1.02 against 0.84 s) | pass |
| a 4-core machine | lower bound ≥ 0.97 | 0.997 (0.980–1.014) | pass |

The prediction was 1.15–1.35. The run came in inside it, in its upper half.

**Throughput at each thread count** (MB/s medians; paired against `mibz2` at 4):

| T | today's decoder | the policy | its plan | teams of 2 | teams of 3 |
|--:|--:|--:|---|--:|--:|
| 4 | **1007** | 993 (0.994) | today's 4 workers | 891 | 530 |
| 6 | 858 | 1155 (1.166) | 3 × 2 | 1189 (1.200) | 987 |
| 8 | 674 | 1234 (1.248) | 4 × 2 | 1223 (1.243) | 988 |
| 10 | 632 | 1252 (1.274) | 4 × 2 | 1147 (1.162) | 1234 (1.264) |
| 20 | 471 | 1256 (1.257) | 4 × 2 | 825 (0.838) | 1084 (1.095) |

**What it shows:**
- **Against today's decoder at the same thread count,** what a caller asking for those threads gets today: 1.80× at 8 threads (1.76–1.84) and **2.62× at 20** (2.54–2.71). Twenty threads is Python's default, all CPUs. Today's decoder falls from 1007 MB/s at 4 workers to 471 at 20, with 8.7× the CPU and 3.5× the memory. The policy stays at 1256, with 74 MB and 1.02 s of CPU.
- **The L3 cap is what holds it:**
  - at 10 threads, five teams of 2 (18 MB of tables) give 1.162 against the policy's four teams' 1.274, paired 1.097 (1.083–1.111);
  - at 20, uncapped teams lose: ten teams of 2 on SMT siblings give 0.838, six teams of 3 give 1.095;
  - the policy's 4 × 2 beats six teams of 3 by 1.147 (1.121–1.174).
- **The policy matches the best fixed arm wherever it is the same plan:**
  - at 8 threads, 1.004 against `mibz2-coop2` (0.977–1.032);
  - at 10, 1.008 against three teams of 3 (0.994–1.023), the fastest fixed arm there.
- **At 6 threads,** the policy's 1.166 and `mibz2-coop2`'s 1.200 are the same plan (3 × 2); one slow policy round (r8, 0.219 s) accounts for the gap.
- **Memory:** the cooperative arms keep fewer tables live, so their peak RSS is lower at equal cores (61 MB at 6 against today's 97).
- **On 4 cores the policy is today's decoder:** 0.997 (0.980–1.014). Below 6 physical cores it plans today's workers, as designed.

**Against PR 0's inversion-only figures:**
- PR 0 measured 4 × 2 at 1.439× p4 for MTF, build and walk, pinned (coop journal §17.4).
- The full decoder unpinned gives 1.248 at 8 threads, so about 0.19 of it goes to the stages outside the inversion and to unpinned placement.
- That is at the low end of the plan's 1.26–1.38 model for the decoder, before its discount for unpinned placement.

**What follows:**
- **The gates pass:** the policy can become the default. That change is its own PR, measured on CI. CI's 2-core runners are below the policy's threshold, so its benchmark must show no change there.
- **The release follows that PR,** with the user's go-ahead.
- **Not shown here:**
  - other CPUs (the policy engages only where Linux reports 6 or more physical cores);
  - other block sizes. At `-1` to `-3` the scanner binds first (workstation journal §9), and the policy doesn't yet look at the stream's level.

## The release candidate on the i9: the default decoder itself (measured, ce7cdc6, 2026-10-29)

The run above measured the policy through the `mibz2-auto` arm, at 34c6ef7. #106 then made the
policy the default (`StreamDecoder::new`), which CI's 2-core runners never exercise on the team
path. So the user ran the gates again, on #106's head and through the entry point users call:
- **Build:** ce7cdc6 (#106's head, merged unchanged as 0711dc3).
- **First:** `just test-ci --release`. Every test passed on the i9, including
  `plan_tests::the_default_is_the_plan`, which runs the team path through `new` on a machine
  with 6 or more cores.
- **Then:**
  `just bench-coop-decoder --arms mibz2-workers,mibz2 --gate mibz2 --threads 4,6,ALL --rounds 10`.
  `mibz2` is `BzReader::parallel`, the default, and `mibz2-workers` is the decoder before the
  policy.
- **Files:** [`coop-decoder-20261029T181511Z.md`](2026-10-29-cooperative-decoder/coop-decoder-20261029T181511Z.md),
  its `.json` and both `.jsonl`, beside the first run's.

| gate | measured | pass |
|---|--:|---|
| throughput ≥ 1.15× (the default at 20 CPUs against 4 workers) | 1.223 | yes |
| its 95% lower bound > 1.05 | 1.163 | yes |
| same bytes | one hash in 80 runs, no errors | yes |
| peak RSS ≤ 1.15× | 1.016 | yes |
| CPU-seconds ≤ 1.5× | 1.254 | yes |
| 4 physical cores (CPUs 0–3): lower bound ≥ 0.97 | 1.013 (0.999–1.027) | yes |

The figures were recomputed from the raw rounds, apart from the report script, and agree.

- **At 6 threads** the default gives 1,071 MB/s, 1.217× (1.178–1.258), at 60 MB peak RSS (0.83×)
  and 0.94× the CPU. On this run it is faster and smaller than 4 workers (863 MB/s, 73 MB).
- **Both arms ran slower in absolute terms than in the first run** (4 workers 863 MB/s against
  1,007; the machine's load was 4.3 at the start). The paired ratios reproduce within each
  other's intervals: 1.223 against 1.257 at 20 CPUs, 1.217 against 1.166 at 6.
- **6 threads above 20 here, 1,071 against 1,050,** is within noise. The two plans differ: 3 × 2
  at 6, and 4 × 2 at 8 and above. Which is better is the policy-tuning question, after the
  release.
