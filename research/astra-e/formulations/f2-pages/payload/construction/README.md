# Construct end-based run records without buffering a previous LF displacement

The original eight-byte endpoint constructor at 112c5c1 delayed publication until it saw the next run. Two alternatives retain its exact emitted index and run response. The coalescing constructor reads all adjacent caps with the same byte, sums their lengths, and publishes the run once at its end. It advances the byte's occurrence cursor by the summed length. The eager constructor publishes a run immediately upon its first cap with its current end. On each additional cap of the same byte it increments the upper 24 endpoint bits by the cap length, leaving the byte and the LF displacement intact. Only a byte change inserts a new boundary in either constructor. Each produces the same exclusive endpoints and stable LF responses for arbitrary cap splitting.

All five states, including the original twelve-byte guarded kernel and its two previously measured compact variants, reconstructed the exact 205,197,794 bytes of all 228 real admissions. The three endpoint constructors produced identical array lengths and capacity high-water marks on each block. The construction variants passed all 11,264 admitted small marks each, and their LF-rank tests cover alternating, long repeated and mixed-size runs across every original row.

Two 60-lap processes price these alternatives in one five-arm binary. Every lap includes all nine systematic admissions. The ten Williams orders occur six times per block; block order rotates and reverses. Every completed inverse was checked against Segmented. The CPU4 run produced unusually wide intervals, so the raw observations and absolute medians are retained.

| Relative to guarded 12-byte run | CPU4 setup+mark | CPU2 setup+mark | CPU4 discovery | CPU2 discovery | CPU4 complete CPU | CPU2 complete CPU |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Coalesced endpoint | 1.2023 | 1.1762 | 0.8752 | 0.8824 | 0.9403 [0.8049,1.0986] | 0.9604 [0.9324,0.9893] |
| Eager endpoint | 1.0227 | 0.9969 | 0.8531 | 0.9217 | 0.8656 [0.7550,0.9924] | 0.9683 [0.9400,0.9974] |

Eager/delayed setup measured 0.8915 [0.7823,1.0161] and 0.8202 [0.7777,0.8650]. Eager/start8 complete CPU measured 1.0150 [0.9396,1.0965] and 1.0094 [0.9730,1.0470]. The direct comparison between the two compact choices remains unresolved. Eager/end and compact starts each used up to 1,048,576 B of retained original-run storage, compared to the twelve-byte control's 1,572,864 B. Eager occupies the original lookup rank exactly once and reaches its exclusive endpoint, byte and LF displacement from that record. Whole-worker peak memory and the target-machine advantage remain to be measured.

The entry for the eager endpoint is eager.rs; candidate.rs selects the coalescing alternative. Source, raw clocks and expected-output checksums are linked through the pinned library and corpus reproduction.

```sh
d=research/astra-e/formulations/f2-pages/payload/construction
bash "$d/run.sh" /tmp/astra-e-end-builders
python3 "$d/report.py" /tmp/astra-e-end-builders/{first,repeat}.csv
```
