# Dispatch first-level rank operations to hardware population count

The target CPU includes POPCNT. Generic x86-64 software also implements Rust count_ones() exactly. Disassembly of the pinned counted candidate from 252646a shows software bit-count sequences in all eight unrolled first-level lookup lanes. On 228 admissions this discovery performed 19,750,839 predecessor queries. The counted preparation also computes per-page popcounts to publish the run numbers.

The new candidate shares the arithmetic and all data structures with that counted candidate. It checks the CPU capability once per setup or discovery call. On capable x86 it executes the same inlined phase with the POPCNT target feature enabled. Other CPUs run the retained generic body. The auxiliary BMI2 arm also enables BZHI for the discovery masks when both features are available. Its generic and POPCNT-only paths remain available. The scorer entry selects POPCNT in both counted preparation and discovery. Transfer of the mark and the recursive second level use their previous code.

The static extraction from the measured binary counted thirteen loads of constants used in the bit-count algorithm and zero POPCNT in the historical build routine. The POPCNT-specialized build contains nine POPCNT instructions in its eight-full-lane and partial-lane paths. The BMI2 build contains nine BZHI and nine POPCNT. The specialized counted preparation contains two POPCNT instructions. The static evidence is reproducible with assembly.py and binutils. Its counts describe machine-code sites.

The property checks passed on all 11,264 small admitted marks and matched the corresponding runs, mask and base arrays. The forced software, each guarded hardware arm and the independent frozen counted candidate reconstructed identical inverses on all 228 real admissions. Together these cover 205,197,794 inverse bytes. Each of two six-arm processes then verified every output in 36 position-balanced rounds on the nine systematic admissions. The immutable library and exact compiler flags are recorded under manifest.txt.

| Enabled phases relative to frozen counted | CPU3 discovery | CPU0 discovery | CPU3 full inverse | CPU0 full inverse |
| --- | ---: | ---: | ---: | ---: |
| Discover | 0.9120 [0.8316,1.0002] | 0.8110 [0.6415,1.0255] | 0.9616 [0.8838,1.0463] | 0.8900 [0.8050,0.9840] |
| Prepare and discover | 0.8448 [0.7427,0.9610] | 0.7673 [0.6417,0.9174] | 0.9041 [0.7989,1.0232] | 0.8639 [0.7900,0.9447] |
| Prepare and BMI2 discover | 0.8926 [0.7746,1.0285] | 0.9089 [0.7298,1.1320] | 0.8926 [0.7895,1.0092] | 0.9270 [0.8289,1.0366] |
| Prepare only | 0.9662 [0.8752,1.0667] | 0.9618 [0.8174,1.1317] | 0.9882 [0.8697,1.1229] | 0.9054 [0.8300,0.9875] |

The generic mode in the expanded source served as a second control. Its complete CPU ratios to frozen counted were 1.0095 [0.9311,1.0944] and 0.8920 [0.8204,0.9698]. Relative to this generic mode the full POPCNT candidate's discovery ratios were 0.8504 [0.7737,0.9348] and 0.7126 [0.5857,0.8669], and its complete ratios were 0.8957 [0.8118,0.9881] and 0.9685 [0.8792,1.0670]. The second process was unusually slow. These control discrepancies limit the precision of the full-stage estimate. They motivated a more focused crossover. Whole-worker, CI and i9 results are separate.

```sh
d=research/astra-e/formulations/f2-pages/instructions
bash "$d/run.sh" /tmp/astra-e-rank-isa
python3 "$d/report.py" /tmp/astra-e-rank-isa/{first,repeat}.csv
python3 "$d/assembly.py" /tmp/astra-e-rank-isa/price
```
