# Full-wave AVX2 gathers and inclusive prefix popcount

For page offset o in 0..64, shifting a u64 run-start mask left by 63-o drops exactly the positions above o. Its population count therefore gives the same inclusive rank prefix. Rust's wrapping shift consumes the low six bits of !x for the full original row x. The scalar arm tested this equivalent expression on the existing eight-slot preload. Two x86-64 AVX2 arms batched every full wave, with runtime AVX2 and POPCNT checks before the discovery loop and the portable implementation available on other machines or short waves.

Each full batch checks all eight page numbers against both vector lengths, gathers eight u32 base ordinals and two sets of four u64 masks, then applies lane-specific shifts. One arm uses scalar counts on the gathered masks. The other computes vector counts via two nibble lookups and summation of adjacent byte counts independently within each u64. Both add the bases and then issue the original eight compact response reads. There are no new persistent vectors. The binary's vpgather, vpsllv, vpshufb, vpsadbw and POPCNT sites are archived. This compares the complete checked implementations. No individual instruction latency or hardware traffic is inferred.

A distinct property check enumerated 131,072 lane prefixes and compared both AVX2 variants and their scalar ranks with independent bit-by-bit counts. It covers every within-page offset and mixed lane indices with empty, full, end-only, start-only and pseudorandom page masks. All four arms reconstructed 205,197,794 inverse bytes over 228 admitted Wikidata blocks against Segmented. The marked-column and record tests passed.

| Whole inverse CPU vs fixed preload | 64 balanced laps, CPU 4 | 64 balanced laps, CPU 2 |
|---|---|---|
| Scalar truncating shift | 1.0000 [0.9584,1.0433] | 1.0253 [0.9591,1.0960] |
| AVX2 gather with scalar popcount | 1.2179 [1.1427,1.2980] | 1.2089 [1.1382,1.2841] |
| AVX2 gather with vector popcount | 1.2317 [1.1591,1.3088] | 1.1562 [1.0785,1.2394] |

The gather variants cost 42–45% more first-level discovery CPU in both processes. They are retired on this local architecture. The scalar variant has no resolved whole-inverse advantage. The i9 and this shared virtual host have different microarchitectures. These measurements give no positive basis for a workstation request. Current best eight-slot source is still 6758831.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots/gather
bash "$d/run.sh" /tmp/astra-e-gather
bash "$d/edge.sh" /tmp/astra-e-gather-edge
python3 "$d/report.py" /tmp/astra-e-gather/{first,repeat}.csv
```

The runner checks the canonical corpus hash. PAGE_DEPS can select the frozen release moonshot library from 252646a. Each process measures nine systematic admissions and checks each timed output.
