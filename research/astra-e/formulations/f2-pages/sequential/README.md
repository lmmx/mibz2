# Publish a page from its sorted run starts

The reference is the dynamic first-level scheduler at 87d41eb. The two-pass page experiment is preserved at bd7f74f. The page-major builder scans pages and coalesced runs together after computing the byte histogram. It assigns each run its full LF address exactly once. At a page boundary it advances to a run that starts there, then stores that containing run as the base. It consumes every later start strictly before the next boundary and publishes exactly one completed entry. A run covering several whole pages leaves their base unchanged and their cuts zero. The original run-end sentinel is excluded. The split64 variant writes exactly the u32 bases and u64 masks of the original physical representation, including its zero boundary bit.

Each of the five resulting layouts matched every original row and all 205,197,794 reconstructed bytes on 228 real admissions. The independent forced marked tests covered 11,264 cases at each of four page widths. 54,040,620 independent row comparisons on crafted patterns supplement the corpus checks, including exact boundary, singleton, unsplit and densely alternating cases. The per-row oracle reads capped run lengths and byte changes independently of the stored directory.

Two 32-pass, eight-arm crossover processes on the previous nine systematic blocks compared the rebuilt directories to historical split64, two-pass packed32 and two-pass packed44. Every arm occupied each position four times. The two linked processes used the same pinned library as the first study, and verified each output against Segmented after the clock. Absolute CPU times differed sharply between the pins.

| Sequential layout | CPU3 prep+mark ratio | CPU0 prep+mark ratio | CPU3 complete CPU ratio | CPU0 complete CPU ratio |
| --- | ---: | ---: | ---: | ---: |
| Split64 | 0.7523 [0.6663,0.8494] | 0.6924 [0.6417,0.7471] | 0.9908 [0.9029,1.0872] | 0.9551 [0.9213,0.9902] |
| Packed32 | 0.8693 | 0.9487 | 0.9570 [0.9233,0.9920] | 1.0178 [0.9054,1.1442] |
| Packed40 | 0.7994 | 0.7763 | 0.9892 [0.9454,1.0351] | 1.0215 [0.9826,1.0620] |
| Packed44 | 0.7793 [0.7279,0.8344] | 0.7834 [0.7232,0.8485] | 0.9524 [0.9094,0.9974] | 0.9793 [0.9480,1.0116] |
| Interleaved64 | 0.7599 | 0.7140 | 0.9651 [0.9109,1.0226] | 0.9832 [0.8856,1.0916] |

The sequential split64 and packed44 programs preserve a paired preparation gain on both pins. Their full-stage estimates are more variable. On CPU0, packed44's discovery ratio rose to 1.0474 [1.0089,1.0873] versus historical split64. The removal of a construction sweep deserves direct comparison with the independently developed front-counts candidate. The complete ratios require an end-to-end scorer and the intended i9. The two strong entries are split64.rs and candidate.rs (packed44).

```sh
d=research/astra-e/formulations/f2-pages/sequential
bash "$d/run.sh" /tmp/astra-e-sequential-pages
python3 "$d/report.py" /tmp/astra-e-sequential-pages/{first,repeat}.csv
```
