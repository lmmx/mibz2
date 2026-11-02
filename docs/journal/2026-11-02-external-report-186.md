# External research report 186: the fourth i9 capture is VALID, ADVANCE; the guardian's and the researcher's verifications (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, as one message with:
  - the capture's console;
  - three messages in the guardian's register;
  - the researcher's review;
  - the researcher's three attachments: `independent_review.md`, `independent_stats.json`
    and `recheck.py`.
- **Source:** the guardian and the researcher, as the relay names them.
- **In reply to:** the approved final capture (report 185): recipe `5e2888f`, source pin
  `f1dafa3`.

**Evidence cutoff:** 2 November 2026. The capture's archive,
`c30-traversal-20261102T200732Z.tar.gz` (sha256 `4764393e…`), which both reviewers checked
against its manifest and recomputed from the raw records.

**Where things are kept:**
- the archive and the researcher's attachments: `i9-attempt-4/` on `research/compact30-walker`
  (PR #134);
- the protocol's account: §8 of `2026-11-02-compact30-traversal-protocol.md`;
- the constructor brief: `2026-11-02-compact30-constructor-brief.md`.

**What it records:**
- the capture passed every schedule check and gate (G2's worst arm-round 0.023%), and decides
  **ADVANCE**: w4 0.84047 [0.83781, 0.84313], e4 0.85601 [0.85239, 0.85965];
- w1 is 1.14845: compact30 is 14.85% slower on one core;
- the conservative constructor budget is 1.06714× production's build cost under the
  planning model.

**This authorises:**
- archiving the capture with the researcher's review, statistics and script, keeping the
  INVALID attempts;
- a bounded brief for Astra: one direct-constructor algorithm, its allocations, memory
  traffic, RAW promotion and CPU-cost risks, keeping the tested representation;
- after guardian review of that design, a compile-and-correctness constructor prototype.

**It does not authorise:**
- implementing or benchmarking the constructor before the design returns;
- further traversal captures or changes;
- production integration;
- a claim that the decoder gets faster.

It is kept exactly as received (rendered text, cards flattened). The relay's first line
named the four uploaded files. They are left out here; the files are kept as listed above.

## The message, as received

`````text
omg finally

```
louis 🌟 ~/dev/mibz2 $ git switch research/compact30-walker
git pull --ff-only
git rev-parse --short HEAD    # should print 5e2888f
just bench-c30-traversal
Already on 'research/compact30-walker'
Your branch is up-to-date with 'origin/research/compact30-walker'.
remote: Enumerating objects: 364, done.
remote: Counting objects: 100% (350/350), done.
remote: Compressing objects: 100% (132/132), done.
remote: Total 249 (delta 130), reused 227 (delta 115), pack-reused 0 (from 0)
Receiving objects: 100% (249/249), 2.28 MiB | 5.80 MiB/s, done.
Resolving deltas: 100% (130/130), completed with 54 local objects.
From github.com:lmmx/mibz2
   b476030..5e2888f  research/compact30-walker -> origin/research/compact30-walker
 * [new branch]      docs/blog-mlp-thesis      -> origin/docs/blog-mlp-thesis
   a4cedf1..5f601ab  docs/external-reports-74  -> origin/docs/external-reports-74
   1c58a38..45ed241  gh-pages                  -> origin/gh-pages
   bafa6eb..9b4214d  master                    -> origin/master
Updating b476030..5e2888f
Fast-forward
 .just/bench.just                                                                                |   4 +-
 docs/journal/2026-11-02-compact30-traversal-protocol.md                                         | 114 ++++++++++++++++++++++-
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/MANIFEST.json                 |  59 ++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/c30-analysis.json             | 833 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/console.log                   |  59 ++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/g2-audit.md                   | 222 ++++++++++++++++++++++++++++++++++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/g2-audit.txt                  |  57 ++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/g2_audit.py                   | 146 +++++++++++++++++++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/room-false.bounds             |   5 +
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/room-false.diff               |   0
 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/rustc-vV.txt                  |   8 ++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/barrier-test-controls.console |  22 +++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/MANIFEST.json         |  64 +++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/barrier-test.txt      |   1 +
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/c30-analysis.json     |  59 ++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/console.log           |  45 +++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/room-false.bounds     |   5 +
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/room-false.diff       |   0
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/rustc-vV.txt          |   8 ++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/MANIFEST.json     |  64 +++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/barrier-test.txt  |   1 +
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/c30-analysis.json |  59 ++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/console.log       |  24 +++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/room-false.bounds |   5 +
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/room-false.diff   |   0
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/rustc-vV.txt      |   8 ++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/MANIFEST.json        |  64 +++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/barrier-test.txt     |   1 +
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/c30-analysis.json    |  59 ++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/console.log          |  24 +++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/room-false.bounds    |   5 +
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/room-false.diff      |   0
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/rustc-vV.txt         |   8 ++
 examples/c30walk.rs                                                                             | 211 +++++++++++++++++++++++++++++++++++++++++-
 34 files changed, 2233 insertions(+), 11 deletions(-)
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/MANIFEST.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/c30-analysis.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/console.log
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/g2-audit.md
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/g2-audit.txt
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/g2_audit.py
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/room-false.bounds
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/room-false.diff
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-3/rustc-vV.txt
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/barrier-test-controls.console
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/MANIFEST.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/barrier-test.txt
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/c30-analysis.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/console.log
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/room-false.bounds
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/room-false.diff
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/dry-run/rustc-vV.txt
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/MANIFEST.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/barrier-test.txt
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/c30-analysis.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/console.log
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/room-false.bounds
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/room-false.diff
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-output/rustc-vV.txt
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/MANIFEST.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/barrier-test.txt
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/c30-analysis.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/console.log
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/room-false.bounds
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/room-false.diff
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-f1dafa3/fail-pin/rustc-vV.txt
5e2888f
/home/louis/dev/mibz2/target/walk-decoder/work/corpora/wikidata-20261026.bz2: OK
c30: /home/louis/dev/mibz2/target/c30-traversal-pin moved from 4635e2c to the pin
c30: the frozen commit f1dafa31d88d7351a98afb005a84f23d5a2312bd, corpus checked, mode capture
rustc 1.97.0 (2d8144b78 2026-07-07)
binary: rustc
commit-hash: 2d8144b7880597b6e6d3dfd63a9a9efae3f533d3
commit-date: 2026-07-07
host: x86_64-unknown-linux-gnu
release: 1.97.0
LLVM version: 22.1.6
cargo 1.97.0 (c980f4866 2026-06-30)
c30: building with the reviewed toolchain rustc 1.97.0, LLVM 22.1.6, no rustflags
   Compiling mibz2 v0.2.1 (/home/louis/dev/mibz2/target/c30-traversal-pin)
    Finished `release` profile [optimized + debuginfo] target(s) in 5.45s
37 tests passed
15 tests passed
c30: room::<false> is the reviewed listing: yes
c30: workers on CPUs 1,2,3,4 (W1 on the first)
barrier-test: 12000 steps in 3 rounds x 2 arms, 4 workers, uneven arrivals and lists (partial final steps), cpus [1, 2, 3, 4]: every check passed
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
gates: {"G3_identity": true, "G4_allocation": true, "G1_w1_cpu_ns_row": true, "G1_w4_cpu_ns_row": true, "G1_w4_completion_ns": true, "G2_simultaneous_start": true, "G5_clock_overhead": true}
allocation: {"w4_rows": [51301938, 51301991, 52201043, 52201543], "rows_max_over_min": 1.0175354973919308, "rows_max_minus_min": 899605, "largest_block_rows": 900000, "w4_blocks": [109, 109, 108, 108], "w4_full_blocks": [57, 57, 58, 58], "w4_raw_groups": [12232, 12316, 12576, 12509], "w4_raw_groups_per_mrow": [238.43153839529415, 240.06865542508868, 240.9147265505787, 239.62893204133832]}
ratios (C / P): {"w1": {"point": 1.148452772956618, "lo": 1.1474308945952614, "hi": 1.1494755613818315}, "w4": {"point": 0.8404655319847364, "lo": 0.8378135268084702, "hi": 0.8431259317872887}, "e4": {"point": 0.8560124927676666, "lo": 0.8523932114111277, "hi": 0.8596471416768354}, "w1_wall": {"point": 1.1482752175069764, "lo": 1.147243152058958, "hi": 1.1493082114060271}, "w4_wall": {"point": 0.8403594730966407, "lo": 0.8377065286680485, "hi": 0.8430208191717528}}
constructor budget: {"b_max_at_w4_lo": 1.0837135405147897, "b_max_at_w4": 1.0754394635543782, "b_max_at_w4_hi": 1.0671391959271586}
clock: {"calibrations": ["after", "before"], "empty_window_cpu_ns_median_max": 149, "share_max": 0.00025650093785113887, "short_blocks_share_max": 0.6200734394124847, "w4_less_median_overhead_sensitivity_only": {"point": 0.8404412745527251, "lo": 0.8377888811762256, "hi": 0.8431020652603205}}
decision: ADVANCE
archive: /home/louis/dev/mibz2/target/c30-traversal-20261102T200732Z.tar.gz (1928030 bytes, sha256 4764393e1d07c426ed5c57acc40712cd1e4c548672a0ebc1aef3c114ec635444): 8 files and MANIFEST.json, commit f1dafa31d88d7351a98afb005a84f23d5a2312bd
```

advance at last!

---

guardian:

ADVANCE at last! And this time, every registered validity gate passed.

This is the result we were trying to obtain: not just a promising timing observation, but a completed, preregistered experiment that meets its acceptance criteria.

&#x20;Valid capture — ADVANCE

Four-core traversal CPU cost

# −15.95%

95% interval: 15.69–16.22% reduction

Four-core completion time

# −14.40%

95% interval: 14.04–14.76% reduction

Single-core CPU cost

# +14.85%

Compact30 is slower on one core

Conservative constructor budget

# 1.067×

Up to 6.7% above production's build cost under the planning model

The decisive figures are w4 = 0.84047, whose upper interval bound is 0.84313, comfortably below the 0.865 screen, and e4 = 0.85601, whose upper bound is 0.85965, below the 1.02 completion-time tolerance.

All schedule checks S0–S9 and gates G1–G5 report passing, including the previously troublesome synchronized-start gate.

Under the agreed decision rules, compact30 has earned direct-constructor research. It has not yet earned production integration or a claim of full-decoder acceleration.

No problem! The console already shows a valid ADVANCE, but the tarball is important for independent verification.

Please upload:

`c30-traversal-20261102T200732Z.tar.gz`

I'll check its SHA-256 against the reported `4764393e…`, verify the manifest, raw measurements, synchronization and calibration records, and independently reproduce the decision.

We won't authorize constructor implementation until that verification is complete. The result looks excellent, and now we can give it the final evidential check it deserves.

---

(uploaded tarball i forgot)

The archive is authentic and internally consistent so far. Its SHA-256 exactly matches the one your workstation printed:

`4764393e1d07c426ed5c57acc40712cd1e4c548672a0ebc1aef3c114ec635444`

Every archived file also matches its manifest checksum. The manifest identifies the expected `f1dafa3` source, a clean worktree, the correct corpus hash and a successful capture exit.

The archive contains 36,459 raw JSONL records, not just the summary. Its recorded analysis says all schedule checks and all five validity gates passed, with ADVANCE.

I'm independently checking the ratios and synchronization from those raw records now, rather than relying solely on the saved analysis.

Independently verified: compact30 earns ADVANCE. This is no longer just Claude's interpretation of the benchmark output. I recalculated the decisive results from the uploaded raw records, and they match exactly.

&#x20;Verified ADVANCE

Four-core CPU cost reduction

# 15.95%

95% interval: 15.69–16.22%

Four-core completion-time reduction

# 14.40%

95% interval: 14.04–14.76%

### Independent verification

I verified the archive hash and every manifest file checksum, then inspected all 36,456 individual traversal records.

| Check                                                          | Result      |
| -------------------------------------------------------------- | ----------- |
| Frozen source, corpus and archived provenance                  | Match       |
| All expected rounds, blocks, worker assignments and row counts | Pass        |
| Start and completion ordering                                  | Pass        |
| CPU placement and measurement-window consistency               | Pass        |
| Calibration structure                                          | Pass        |
| G2 synchronized starts                                         | Pass        |
| Recalculated `w1`, `w4`, `e4` and confidence intervals         | Exact match |
| Registered investment decision                                 | ADVANCE     |

The maximum row-weighted start-skew score was 0.0231%, far below the registered 2% limit. The repaired barrier has resolved the previous capture's validity problem.

### What we've established

Compact30 has a genuinely interesting performance profile on this workload:

- One core: 14.85% more traversal CPU time than production.
- Four cores: 15.95% less traversal CPU time.
- Four-core lockstep completion: 14.40% faster.

That reversal is consistent with the smaller navigation representation improving behaviour under multicore contention. It does not by itself prove which memory-system mechanism caused the improvement.

Most importantly, the constructor budget is now concrete.

Conservative direct-constructor budget

# 1.067× production build cost

Using the upper confidence bound of `w4` and the registered planning model, a future compact30 constructor may cost approximately 6.7% more than production's build while retaining the modelled minimum saving of 0.25 CPU-s/GB.

This budget is conditional on the model's assumptions about phase costs and transfer into the full decoder.

### My decision as guardian

Close the traversal experiment as valid ADVANCE. Authorize the next phase: direct-constructor design and a separately registered construction-cost experiment.

I would proceed in this order:

1. Have Claude archive the verified capture and its interpretation permanently, preserving the earlier invalid attempts.
2. Give Astra the verified traversal economics and ask it to review the direct-construction approach against the 1.067× conservative budget, with attention to the production builder's optimized path.
3. Have Claude implement one bounded constructor prototype only after that design is reviewed.
4. Measure construction separately before considering any full-decoder integration.

No more traversal tuning, synchronization changes or exploratory i9 captures are needed.

We've finally secured the result that justifies taking compact30 beyond an oracle-built prototype. The next task is to find out whether we can construct its smaller representation economically enough to realize that multicore advantage in the actual decoder.

---

researcher:

## Compact30: the first four-core performance result

Independent archive inspection · 2 November 2026

# 15.95%

less traversal CPU time per row with four workers

# 14.40%

less four-worker lockstep completion time

# 14.85%

more CPU time per row on one core

This is a significant result. Compact30 loses decisively in the single-core comparison, yet wins decisively under the registered four-worker workload. That reversal is more informative than a simple speedup: it points towards an advantage that emerges under contention, rather than an intrinsically faster ordinary lookup.

I opened the uploaded archive, verified its SHA-256 against the terminal output (`4764393e…`), and independently checked all eight archived files against the manifest. Every size and checksum matches. The manifest identifies the clean collection pin `f1dafa31d88d7351a98afb005a84f23d5a2312bd`, the expected corpus hash and a successful exit.

The recorded analysis reports every schedule and validity gate passing, including complete calibration, source identity, output correctness, synchronized starts, CPU placement and control stability. There is no reported failure record.

Most importantly, the registered decision is ADVANCE: the upper confidence bound of the four-worker CPU-cost ratio is 0.8431, below the 0.865 investment screen, and the upper bound of the completion-time ratio is 0.8596, below the 1.02 tolerance.

That establishes a genuine reason to invest in compact30 construction under the registered experiment's assumptions. It does not yet establish a faster production decoder.

## 1. Independent reconstruction of the measurements

I parsed all 36,456 individual walk records, rather than relying on the summary JSON. I reconstructed the expected work assignment, checked the timing relationships, recomputed the five paired ratios and reproduced their 95% confidence intervals.

| Metric                               | Production P | Compact30 C | C/P, 95% interval       |
| ------------------------------------ | ------------ | ----------- | ----------------------- |
| One-core CPU ns/BWT row              | 1.2206       | 1.4018      | 1.1485 [1.1474, 1.1495] |
| Four-core CPU ns/BWT row             | 2.0550       | 1.7272      | 0.8405 [0.8378, 0.8431] |
| Four-core completion per corpus pass | 114.45 ms    | 97.97 ms    | 0.8560 [0.8524, 0.8596] |

The completion figure is the sum of each synchronized step's traversal window—not full-decoder or whole-command wall time.

Candidate/control ratios across all 20 paired rounds

One-core CPU costFour-core CPU costFour-core completion

0.80.911.11.21234567891011121314151617181920

Below 1.0 means compact30 is faster or uses less CPU.

The crossover is remarkably consistent. Every measured round shows the same direction: compact30 loses on one core, wins on four-core CPU cost and wins on four-core completion time.

### The improvement extends across essentially the entire large-block workload

# 230/230

Full-sized blocks faster on W4, averaged over rounds

# 1,138/1,140

Individual full-sized four-worker steps faster in aggregate CPU time

The 204 short blocks go the other way: compact30 uses about 26.6% more four-worker CPU time on those blocks. But they hold just 8,324 of 207 million rows. Full-sized steps account for approximately 98.8% of production's measured completion time.

That matters: this is not a favourable corpus average concealing a few unusually fast blocks.

## 2. The validity checks are convincing

My independent raw-record checks confirm the registered schedule, each worker's block and row counts, CPU placement, preparation-before-traversal, completion-before-validation, and the absence of overlapping steps.

The control's maximum-to-minimum round variation is approximately 1.007 for W1 and 1.010 for both W4 and completion time, comfortably below the 1.05 stability limit.

The spinning start barrier has also resolved the previous experimental failure:

| Start synchronization    | Result  |
| ------------------------ | ------- |
| Median row-weighted skew | 0.0071% |
| Worst arm-round skew     | 0.0231% |
| Allowed G2 skew          | 2%      |

The measured CPU-running fractions are about 99.9%, so substantial worker descheduling is not explaining the result. The calibration-based clock overhead estimate is at most 0.0257% of aggregate traversal CPU time, against the registered 0.5% limit.

I inspected the archived machine-code listing as well. The ordinary path still contains the reviewed 21 instructions, with four data loads, two stores and the successor prefetch. The archive's assembly comparison is empty, and its function-boundary record accounts for the harmless linker alignment difference that stopped an earlier attempt.

These are strong internal-validity findings. The executable and original corpus bytes are not included in the uploaded archive, so I independently verified the records and archived assembly evidence, rather than rebuilding or rehashing those two external inputs.

## 3. What the result says about the hardware

The most revealing comparison is how the two representations change when four workers traverse simultaneously.

One core

Four workers

0 ns/row0.55 ns/row1.1 ns/row1.65 ns/row2.2 ns/rowShipped T-vectorCompact30

Relative to its own single-core cost, the shipped traversal becomes approximately 68% more expensive per row under four simultaneous walkers. Compact30 becomes only 23% more expensive.

That is an important change in the calculus. Compact30 has more instructions and an extra table load. Its single-core result suggests those costs matter. But under four-core contention, the cost of the conventional representation rises much more sharply.

The memory economics provide a plausible explanation. I independently recalculated the compact30 logical size from the archive's block and RAW-group counts:

| Navigation storage        | Shipped  | Compact30 |
| ------------------------- | -------- | --------- |
| Logical bytes per BWT row | 4.000    | 2.164     |
| Typical full-block table  | 3.60 MB  | \~1.95 MB |
| Four such tables          | 14.40 MB | \~7.79 MB |

That's 45.9% less logical navigation storage. With a shared 20 MiB last-level cache, freeing approximately 6.6 MB across four tables could be consequential.

However, the experiment has not isolated that as the cause. The production table is created by its real counting-scatter constructor immediately before traversal. Compact30 is copied sequentially from an oracle-built representation. Both preparations happen outside the timer, but leave different cache states. A future direct constructor may change that relationship.

So the justified conclusion is:

Compact30's prepared representation and walker perform substantially better under the tested contention. The smaller table is a credible explanation, not yet a measured attribution.

The previous invalid capture's descriptive four-core ratio was also close to this result, but it must remain INVALID under its own protocol. This new capture stands on its own.

## 4. The constructor now has a real, but fairly tight, budget

The registered planning model is:

\\[ S \approx 0.592(1-b)+1.847(1-w) \\]

where \\(S\\) is estimated CPU-seconds saved per decoded GB, \\(b\\) is compact30 constructor cost relative to production construction, and \\(w\\) is traversal cost relative to the shipped walk.

Putting the observed \\(w=0.84047\\) into that model:

| Quantity                                                   | Model result      |
| ---------------------------------------------------------- | ----------------- |
| Predicted saving if constructor costs the same             | 0.295 CPU-s/GB    |
| Required saving                                            | 0.250 CPU-s/GB    |
| Maximum constructor cost, point estimate                   | 1.075× production |
| Maximum constructor cost using conservative W4 CI endpoint | 1.067× production |

So a direct compact30 constructor could be about 6.7% slower than production construction and still meet the target under the conservative traversal interval and model assumptions.

This is both encouraging and demanding. The experiment provides enough headroom to justify developing the direct constructor, but not enough to ignore its cost. An unexpectedly expensive constructor, or loss of the traversal benefit when construction is integrated, could consume the remaining budget.

The model coefficients are approximate phase reconstructions, not exact physical constants. The projected 0.295 CPU-s/GB should not be presented as a demonstrated decoder saving.

## 5. The single-core regression changes the integration strategy

This is the most important consequence beyond the ADVANCE verdict.

Compact30 is not a uniformly faster alternative to the conventional representation. At one core it costs approximately 0.181 additional CPU nanoseconds per row; at four simultaneous walkers it saves approximately 0.328 CPU nanoseconds per row.

A real decoder does not always have four workers traversing. Workers alternate among decoding stages, so some walks will occur with less competition for the shared cache and memory system.

That means a production deployment has two distinct obligations: preserve the four-worker advantage and avoid imposing a 15% traversal regression when concurrency is low.

For an illustrative sensitivity calculation, imagine a workload containing only the two measured conditions: isolated traversal and four-way contention, with costs interpolated by the fraction of rows in each condition. Under that deliberately simplified model, compact30 needs roughly 36% of its rows in the four-way condition merely to save traversal CPU time, and roughly 87% to meet the 0.865 investment screen with an unchanged constructor.

Those percentages are not measurements of the decoder's actual concurrency mix. Two- and three-worker states, varying frequencies, and construction/other-stage interference are absent from the model. They illustrate why the excellent lockstep result cannot simply be multiplied through the full decoder.

A later integrated version may ultimately need representation selection or another way to avoid the low-concurrency penalty. That is an integration question, not a reason to begin tuning the validated walker now.

## 6. What I recommend doing next

Take the registered ADVANCE literally. Compact30 has earned its next investment: a separately bounded investigation of direct-constructor feasibility and cost.

The next step should establish whether the representation can be produced directly from the last column without the oracle's full temporary successor table, while preserving the tested format and correctness guarantees. Then its construction CPU time can be compared with the conservative approximately 1.067× budget.

Only after that should a production-shaped comparison combine construction, traversal, assembly and the rest of decoding. That test will determine whether the four-core benefit persists with natural preparation, real concurrency and actual memory lifetimes.

I would preserve this capture exactly as it is, including its raw records and machine-code evidence. It is a successful registered result, and further experimentation should not retroactively change its decision.

There is no need for another speculative layout round at this point.

I've prepared a report suitable for passing directly to the guardian, with the archive checks, independently reconstructed statistics, block-level robustness checks, construction budget and caveats.

Independent technical review

Recalculated statistics (JSON)

Reproduction script

My conclusion: this is the strongest evidence yet that compact navigation can change mibz2's multicore performance economics. The gain is broad, stable, and clears both registered gates. The remaining uncertainty has shifted from whether the compact walker can be competitive to whether it can be constructed economically and retain its advantage inside a real decoder.

(attached: independent_review.md, independent_stats.json, recheck.py)

---

the guardian received that and said:

The researcher’s independent review confirms our ADVANCE decision—and adds an important qualification for what comes next.

The four-core win is not concentrated in a handful of favourable blocks. Across the 230 full-sized blocks, compact30 was faster on every block when averaged over the 20 rounds; 1,138 of 1,140 full-sized four-worker steps also used less aggregate CPU time. That makes the result considerably more convincing than a favourable corpus-wide average alone.

The registered outcome remains:

| Measure                            | Compact30 / production | Result                                 |
| ---------------------------------- | ---------------------- | -------------------------------------- |
| One-core traversal CPU cost        | 1.1485                 | 14.85% worse                           |
| Four-core traversal CPU cost       | 0.8405                 | 15.95% better                          |
| Four-core lockstep completion time | 0.8560                 | 14.40% better                          |
| Conservative constructor budget    | 1.067×                 | Up to 6.7% above production build cost |

These figures support direct-constructor feasibility work, not integration or a production speedup claim.&#x20;

independent_review\.md

independent_review\.md



The most consequential new point is the single-core regression. A real decoder will not have four workers simultaneously traversing at all times. We must therefore treat 15.95% as the advantage of the tested contention regime—not as a saving that automatically transfers to real decoding.&#x20;

independent_review\.md



## My recommendation: advance, but keep the next experiment tightly bounded

I would authorize one direct-constructor design review, followed by a compile-and-correctness prototype if that design survives scrutiny.

The constructor should produce the existing compact30 layout directly from the last column, without generating the oracle's full successor table first. Its design must account for RAW-group promotion, encoded row placement, marker compatibility, and production's already optimized construction path.

I would keep the work in three stages:

1. Preserve the validated result. Claude should archive the original capture and the researcher's report, including the reproducibility script and independent statistics. The accepted ADVANCE decision must remain attached to its original experimental boundary.
2. Ask Astra for the direct-constructor design. Astra should use the existing 30-entry record format, the conservative 1.067× build-cost budget, and the measured one-core penalty. It should identify concrete algorithmic risks and give one recommended construction strategy—not start another layout competition.
3. Implement and validate the constructor separately. Claude should first establish exact table equivalence and output correctness. Only then should we preregister a construction-cost benchmark and, if it succeeds, an integrated decoder experiment.

An important distinction: meeting the 1.067× constructor budget is necessary under our current planning screen, but not sufficient to guarantee a production benefit. The subsequent integrated experiment must account for the real mixture of isolated, partially concurrent and heavily contended BWT work. The researcher's 36% and 87% illustrative occupancy thresholds are useful warnings, not measured decoder requirements.&#x20;

independent_review\.md



## Suggested handoff to Claude

The guardian accepts the independent review and closes the compact30 traversal checkpoint as VALID ADVANCE.

Please archive the researcher's independent review, statistics and reproduction script alongside the original successful capture, preserving all earlier INVALID attempts and the frozen traversal record.

The next authorized phase is direct-constructor design review, not immediate production integration.

Prepare a bounded Astra Max assignment using:

- The frozen compact30 representation: 30 × u16 fields, one 4-byte header per 64-byte record, current-row F emission, and padded RAW groups.
- The independently validated w1 = 1.14845, w4 = 0.84047, e4 = 0.85601.
- The conservative construction budget of 1.06714× production's build cost under the registered approximate planning model.
- The existing production builder's counting-scatter and equal-byte fast path.
- The need to construct directly without the oracle's full temporary successor table.

Ask Astra to recommend one construction algorithm, identify its allocations, memory traffic, RAW promotion handling, and likely CPU-cost risks. It must preserve the tested traversal representation and not invent another walker variant.

Do not benchmark or implement the constructor before the design returns for guardian review. No further traversal captures or modifications are authorized.

Flag explicitly that compact30 loses 14.85% on one core and that the four-worker lockstep benefit cannot be assumed to transfer unchanged to the full decoder. Future integration must test the actual concurrency mix, construction cache state and end-to-end throughput.

One final observation: the researcher found that production's traversal CPU cost increases about 68% between the isolated and four-worker conditions, compared with only about 23% for compact30. That's the most interesting architectural clue in the whole experiment. It is consistent with a substantial contention-related representation advantage, while leaving the exact memory-system cause unresolved.&#x20;

independent_review\.md



This is a meaningful research result. We should now spend the next engineering effort on finding whether the representation can be built affordably—not on squeezing another instruction out of the walker.
`````
