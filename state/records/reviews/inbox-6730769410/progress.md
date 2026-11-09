# Source findings in progress

#269 source fd60fffada25d475bbcc2b54165b96cf5fd7a92f; result journal66edecbe2b1924594e485e1f956dbee8ed9beda2.

- Phase fallback selector executes outside its timed bracket. A/B fallback source bodies match; attributing the full gap to selector is unsupported.
- Phase nominal c/B divides by total decoded output, not accepted BWT rows. Separate phase medians cannot be added/subtracted to reproduce judged entry ratios.
- Even ideal sort removal leaves +0.218 nominal phase-sum c/B in the reported medians; removing fallback gap as well leaves +0.118. The 1.332 fill is not a measured direct-placement fill.
- Existing emitter writes 4r bytes, around0.248n on accepted r/n=.062. Per-literal wider stores do not mean four times total output traffic.
- Admission is symbols.len/max_len<=1/8, not measured r/n. Actual run lengths and literal count can be accumulated from RUNA/RUNB symbols without MTF byte identities; their cost still belongs on the scanner/front path.
- #270 D049 timing HOLD retained. Five routing audit findings are historical and explicitly resolved.

Next: finish lookup budget, emitter choices, admission and helper critical-path synthesis. No implementation or timing.
