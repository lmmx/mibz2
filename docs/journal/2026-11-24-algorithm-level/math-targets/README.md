# Mathematical targets

What new mathematics in mibz2 is for, what each target is worth, and the measurements behind it. Set up on 24 November 2026 at Louis's request; it replaces the single overview file, which is kept as `overview-2026-11-24.md`.

## The picture

The decoder has two kinds of work:
- **the scanner:** one thread, serial; it reads the bits and decodes the Huffman codes;
- **the workers:** k threads; they run MTF/RLE2, build the inversion (Open, the induction), emit the text, and handle RLE1 and CRC.

The induction, which this programme's mathematics is about, runs in the **workers**.

The ceilings come from the four-arm oracle (journal §44). Each is whole-decoder wall time against the shipped decoder; lower is better.

| target | 1 worker | 6 workers | 10 workers |
|---|---|---|---|
| T1 inversion emission (oracle B) | 0.830 | 0.850 | 0.920 behind the scanner (0.976 alone) |
| T4 Huffman scan (oracle C) | 0.996 | 0.982 | 0.806 |
| T5 both together (oracle D) | 0.825 | 0.830 | 0.741 |

T2 (construction), T3 (MTF/RLE2) and T6 (RLE1, CRC, output) are not yet measured. A worker-split oracle is the next measurement.

## Files

| file | contents |
|---|---|
| `targets.tsv` | one row per target: the stage, where it runs, its oracle, its ceilings, the obstacle, whether it is mathematics or engineering, owner, status and records |
| `metrics.tsv` | what is measured, what isn't yet, and where each figure comes from |
| `data/measurements-four-arm.tsv` | per worker count and arm: rounds, median and mean wall, paired ratio with its 95% interval, median and maximum peak RSS |
| `data/four-arm-runs.tsv`, `data/four-arm-report.txt` | the raw rows and report from the bundle `four-arm/run-20261124T204154Z` (`evidence` at c9302539, SHA-256 736e0624…976c) |
| `scripts/extract_four_arm.py` | regenerates `measurements-four-arm.tsv` from the raw rows |
| `data/measurements-scan-profile.tsv` | per worker count (1–20): walls, scanner work and slot waits, worker busy and idle, rho, efficiency against the busiest server, the decode's own RSS, CPU, faults and context switches |
| `data/scan-profile-runs.tsv`, `data/scan-profile-report.txt` | the raw rows and report from the bundle `scan-profile/run-20261124T223748Z` (`evidence` at 41389cbe, SHA-256 530f97e0…8260) |
| `scripts/extract_scan_profile.py` | regenerates `measurements-scan-profile.tsv` from the raw rows |
| `overview-2026-11-24.md` | the first prose overview |

Engineering leads that bear on these targets are in `../leads/`. One is `vectorised-compaction.md`, an AVX2 left-pack for filter steps in T1–T3.

Regenerate:

```
python3 scripts/extract_four_arm.py data/four-arm-runs.tsv data/four-arm-report.txt > data/measurements-four-arm.tsv
python3 scripts/extract_scan_profile.py data/scan-profile-runs.tsv > data/measurements-scan-profile.tsv
```

## Open questions

These went to MONODROMY in channel #33:
- What else is worth measuring?
- What should be reasoned about formally, rather than measured?

New measurements land in `data/` as TSV, with the script that derives them, and a row is added to `metrics.tsv`.
