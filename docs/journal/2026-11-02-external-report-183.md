# External research report 183: the third i9 capture completes, INVALID on G2, and a G2 audit is requested (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with the capture's console and archive, kept
  below the message.
- **Source:** unattributed in the relay (the guardian's register, addressed to "Claude and
  the researcher").
- **In reply to:** the corrected capture (`just bench-c30-traversal`, branch head `b476030`,
  source pin `4635e2c`, rustc 1.97.0), run on the i9.

**Evidence cutoff:** 2 November 2026. The guardian's inspection of the archive,
`target/c30-traversal-20261102T191057Z.tar.gz` (sha256 `3d6667c9…`), including its raw
timestamps. The archive is kept beside this report, in `2026-11-02-external-report-183/`.

**What happened:**
- The capture ran all 20 measured rounds.
- Every schedule check and G1, G3, G4 and G5 passed. G2 (simultaneous start) failed, so the
  registered outcome is INVALID, with no decision.
- The guardian's timestamp inspection found the start skew larger in the first W4 arm of
  each round (about 5.38%) than in the second (about 2.27%).

The descriptive ratios are w1 1.13752, w4 0.84187 and e4 0.85650. They are not a result.

**This authorises:** an independent G2 investigation from the existing archive:
- G2's formula and the raw timestamps: by round order, worker, first or second arm, and
  outliers;
- the relation of the skew to overlap and to the ratios;
- barrier synchronisation against actual simultaneous execution;
- whether G2 measured the intended property, or its threshold was unsuitable;
- one recommendation: retain INVALID and stop, or a concrete measurement defect, with one
  separately registered correction and how a future authorised capture could test it.

**It does not authorise:**
- changing thresholds after the fact, reclassifying INVALID, or discarding observations;
- changing the walker, its compiled path or the measurements;
- new architecture, tuning arms or a counter campaign;
- any workstation run.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
This is the first actual performance result—and it is much more interesting than the previous failures. The benchmark completed its 20 paired rounds, passed all correctness, identity, stability and clock-overhead checks, then reported INVALID because G2, the synchronized-start gate, failed.

I examined the raw timestamps in your archive. The numbers are real recorded measurements, but the registered protocol does not allow us to declare ADVANCE while G2 is false.

Four-core traversal CPU cost

# −15.8%

Compact30 versus shipped walker; descriptive, not an approved result

Four-core lockstep completion time

# −14.4%

Also descriptive while G2 is failed

One-core traversal CPU cost

# +13.8%

Compact30 is slower without four-core contention

G2 synchronized-start score

# 7.96%

Registered maximum: 2%

The striking reversal—from slower on one core to faster on four—is compatible with compact30 benefiting from its smaller memory footprint under contention. It does not yet establish that mechanism or satisfy the experiment's validity rules.

## Why G2 failed

The raw timestamps reveal something the final INVALID label doesn't explain.

Across the full-block steps, start skew is systematically higher in the first four-core arm of a round than in the second:

| Position in round | Mean start skew | Share of steps above 2% |
| ----------------- | --------------- | ----------------------- |
| First arm         | 5.38%           | 83.9%                   |
| Second arm        | 2.27%           | 68.2%                   |

This is not just one anomalous round. It repeats across the alternating P/C order.

That points toward a measurement-synchronization or execution-environment issue, rather than a particular compact30 decoding failure. The timestamps alone cannot tell us whether the delay came from barrier release, scheduling, a descheduled thread or another cause.

There is also a limitation to the gate itself: start timestamps are captured independently by threads after a barrier releases them. A barrier aligns participation, but does not guarantee that all four threads resume execution in the same instant.

Still, the preregistered 2% threshold has been exceeded substantially. We must not quietly relax it because the performance numbers look encouraging.

## My decision

Preserve this as a completed but INVALID experiment, with a highly promising exploratory result.

The result warrants a focused investigation of G2, not another compact30 redesign. We should determine whether the gate is rejecting an otherwise usable workload or identifying a meaningful lack of synchronized contention.

That diagnosis should use the existing archive first. No hardware rerun or threshold change is authorized yet.

## Send Claude and the researcher this

The third i9 attempt completed all 20 measured paired rounds, but the registered outcome is INVALID because G2 failed. Please preserve this as a completed invalid capture, not an ADVANCE.

Archive: `c30-traversal-20261102T191057Z.tar.gz`, SHA-256 `3d6667c9…`. I have the complete raw records.

The descriptive measurements are:

- w1 = 1.13752 [1.13613, 1.13892]
- w4 = 0.84187 [0.83946, 0.84429]
- e4 = 0.85650 [0.84340, 0.86980]
- All checks and gates pass except G2.

The guardian's initial timestamp inspection finds G2 start skew averaging approximately 5.38% in the first four-core arm per round versus 2.27% in the second, across full-block steps. This ordering effect deserves explanation. Do not interpret the ratios as an approved performance result.

**Please independently investigate G2 using the existing archive first.**

Audit the exact G2 formula and raw timestamp distribution, including round order, worker identity, first-versus-second arm, possible outliers and the relationship between start skew, overlap and measured ratios.

Distinguish barrier synchronization from actual simultaneous thread execution. Assess whether G2 was measuring the intended validity property or whether its threshold was unsuitable for the registered lockstep implementation.

Do not change thresholds post hoc, silently reclassify INVALID, or discard observations to rescue the candidate. If a real measurement-design defect is identified, propose one separately registered correction and explain how an independently justified future capture could test it.

Keep the compact30 walker, compiled path and original measurements frozen. No new architecture, tuning arms, counter campaign or workstation run is authorized.

Return one clear recommendation: retain INVALID and stop, or identify a concrete measurement defect warranting a separately authorized experiment.

## What this means for compact30

There are two different conclusions here.

Scientifically, this trial is invalid under its registered rules. We cannot claim the 15.8% reduction as an established result.

Practically, compact30 looks worth investigating further. A large, consistent reversal between one-core and four-core behaviour is exactly the sort of architectural effect we hoped a smaller table might uncover. It is substantial enough to justify understanding a synchronization problem rather than abandoning the idea immediately.

And to address your frustration: this time the benchmark did not crash or fail to execute. It collected the complete experiment and reported a predeclared validity failure. That is meaningfully different from the preceding preflight problems.

You've provided enough hardware evidence for now. No fourth run. I would put the machine aside, preserve the archive and let the researcher audit the existing timestamps before deciding whether another controlled measurement is warranted.
`````

## The maintainer's remark and the capture's console, as relayed

`````text
what on earth... how did this fail yet again??!

louis 🌟 ~/dev/mibz2 $ git switch research/compact30-walker && git pull --ff-only   # head b476030
just bench-c30-traversal
Already on 'research/compact30-walker'
Your branch is up-to-date with 'origin/research/compact30-walker'.
remote: Enumerating objects: 210, done.
remote: Counting objects: 100% (200/200), done.
remote: Compressing objects: 100% (59/59), done.
remote: Total 114 (delta 57), reused 109 (delta 54), pack-reused 0 (from 0)
Receiving objects: 100% (114/114), 78.09 KiB | 1.22 MiB/s, done.
Resolving deltas: 100% (57/57), completed with 47 local objects.
From github.com:lmmx/mibz2
   0d85566..b476030  research/compact30-walker   -> origin/research/compact30-walker
   8823dcf..a4cedf1  docs/external-reports-74    -> origin/docs/external-reports-74
   a04e8b5..1c58a38  gh-pages                    -> origin/gh-pages
   c74f5e2..bafa6eb  master                      -> origin/master
 * [new branch]      pre-commit-ci-update-config -> origin/pre-commit-ci-update-config
Updating 0d85566..b476030
Fast-forward
 .just/bench.just                                                                                           |   10 +-
 bench/asm_listing.py                                                                                       |  143 +++++++++++---
 bench/test_asm_listing.py                                                                                  |  188 +++++++++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol.md                                                    |   82 +++++++-
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-2/MANIFEST.json                            |   44 +++++
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-2/console.log                              |   34 ++++
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-2/room-false.asm                           | 1560 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-2/room-false.diff                          |    2 +
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-2/rustc-vV.txt                             |    8 +
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/MANIFEST.json     |   59 ++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/c30-analysis.json |   59 ++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/console.log       |   44 +++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/room-false.bounds |    5 +
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/room-false.diff   |    0
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/rustc-vV.txt      |    8 +
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/refused-local-changes.console            |    2 +
 16 files changed, 2212 insertions(+), 36 deletions(-)
 create mode 100644 bench/test_asm_listing.py
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-2/MANIFEST.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-2/console.log
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-2/room-false.asm
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-2/room-false.diff
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-2/rustc-vV.txt
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/MANIFEST.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/c30-analysis.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/console.log
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/room-false.bounds
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/room-false.diff
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/dry-run-default-1.98.1/rustc-vV.txt
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-4635e2c/refused-local-changes.console
/home/louis/dev/mibz2/target/walk-decoder/work/corpora/wikidata-20261026.bz2: OK
c30: /home/louis/dev/mibz2/target/c30-traversal-pin moved from 9267433 to the pin
c30: the frozen commit 4635e2cb9f7b61315314c51affaebe843cd8ef36, corpus checked, mode capture
rustc 1.97.0 (2d8144b78 2026-07-07)
binary: rustc
commit-hash: 2d8144b7880597b6e6d3dfd63a9a9efae3f533d3
commit-date: 2026-07-07
host: x86_64-unknown-linux-gnu
release: 1.97.0
LLVM version: 22.1.6
cargo 1.97.0 (c980f4866 2026-06-30)
c30: building with the reviewed toolchain rustc 1.97.0, LLVM 22.1.6, no rustflags
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling libm v0.2.16
   Compiling thiserror v2.0.21
   Compiling libc v0.2.191
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling syn v3.0.9
   Compiling fearless_simd v1.2.0
   Compiling thiserror-impl v2.0.21
   Compiling bzip2 v0.4.4
   Compiling pbz2-core v0.2.0
   Compiling pbz2 v0.2.0
   Compiling mibz2 v0.2.1 (/home/louis/dev/mibz2/target/c30-traversal-pin)
    Finished `release` profile [optimized + debuginfo] target(s) in 13.17s
37 tests passed
15 tests passed
c30: room::<false> is the reviewed listing: yes
c30: workers on CPUs 1,2,3,4 (W1 on the first)
grid: round -1 done
grid: round 0 done
grid: round 1 done
grid: round 2 done
grid: round 3 done
grid: round 4 done
grid: round 5 done
grid: round 6 done
grid: round 7 done
grid: round 8 done
grid: round 9 done
grid: round 10 done
grid: round 11 done
grid: round 12 done
grid: round 13 done
grid: round 14 done
grid: round 15 done
grid: round 16 done
grid: round 17 done
grid: round 18 done
grid: round 19 done
c30: the grid exited with status 0
schedule: {"S0_no_failure": true, "S0_no_unknown_records": true, "S1_meta": true, "S2_rounds": true, "S3_record_set": true, "S4_placement": true, "S5_walk_windows": true, "S6_w1_sequence": true, "S7_start_barrier": true, "S7_completion_barrier": true, "S7_cross_step": true, "S8_pass_order": true, "S9_calibration": true}
gates: {"G3_identity": true, "G4_allocation": true, "G1_w1_cpu_ns_row": true, "G1_w4_cpu_ns_row": true, "G1_w4_completion_ns": true, "G2_simultaneous_start": false, "G5_clock_overhead": true}
allocation: {"w4_rows": [51301938, 51301991, 52201043, 52201543], "rows_max_over_min": 1.0175354973919308, "rows_max_minus_min": 899605, "largest_block_rows": 900000, "w4_blocks": [109, 109, 108, 108], "w4_full_blocks": [57, 57, 58, 58], "w4_raw_groups": [12232, 12316, 12576, 12509], "w4_raw_groups_per_mrow": [238.43153839529415, 240.06865542508868, 240.9147265505787, 239.62893204133832]}
ratios (C / P): {"w1": {"point": 1.1375213105914501, "lo": 1.1361283261546362, "hi": 1.1389160029388903}, "w4": {"point": 0.8418714013708521, "lo": 0.8394587495143678, "hi": 0.8442909873250319}, "e4": {"point": 0.8564990685104668, "lo": 0.8434019535438108, "hi": 0.8697995674267673}, "w1_wall": {"point": 1.1373707238340987, "lo": 1.135970507912847, "hi": 1.1387726656845116}, "w4_wall": {"point": 0.8417879574144312, "lo": 0.8393463083432815, "hi": 0.8442367092155595}}
constructor budget: {"b_max_at_w4_lo": 1.0785805568360856, "b_max_at_w4": 1.0710532460608717, "b_max_at_w4_hi": 1.0635043013693684}
clock: {"calibrations": ["after", "before"], "empty_window_cpu_ns_median_max": 162, "share_max": 0.0002785686298441, "short_blocks_share_max": 0.6212847555129435, "w4_less_median_overhead_sensitivity_only": {"point": 0.8418451792840081, "lo": 0.8394321590345248, "hi": 0.8442651359686301}}
decision: INVALID: a validity gate failed; no decision
archive: /home/louis/dev/mibz2/target/c30-traversal-20261102T191057Z.tar.gz (1935960 bytes, sha256 3d6667c9fcc534ebfffb3c1f876f0e1275f52191207f00dc43c32609421e8d39): 7 files and MANIFEST.json, commit 4635e2cb9f7b61315314c51affaebe843cd8ef36
error: Recipe `bench-c30-traversal` failed with exit code 1
louis 🌟 ~/dev/mibz2 $
`````
