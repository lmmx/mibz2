# Sparse4 against two separately stored instances of the same named control

All 144 ten-block rounds completed. Sparse4/named-A was **1.0213 [0.9966, 1.0465]** and sparse4/named-B **1.0205 [0.9875, 1.0545]** for complete inverse CPU. The source-identical B/A controls gave **1.0008 [0.9705, 1.0321]**. Together with the registered eight-arm screen, these results support keeping the source-level representation without promoting it to a whole-worker entry. [Pre-data protocol](PLAN.md) · [Paired clustered contrasts](summary.txt) · [All 4,320 chronological measurements](paired.csv)

The original checksummed ten tapes were read before the first measured round. Each full round covered all three arms on every block. Both named controls passed through one runtime call site with identical arm code and separate warm scratch. Every output was compared byte for byte with dense original Ψ. Each six-round cluster exhausted all six local orders for every block. Blocks changed starting point between rounds and direction between clusters. The 24 complete temporal units determined the t intervals. After 48 rounds the source-identical control alone projected a 24-cluster half-width of 0.01607178 in log ratio, below the frozen 0.07696104 precision-futility cutoff. No observation was trimmed. [Stopping-rule log](verdict.log) · [Cluster observations](clusters.csv)

| Compared arms | Original first root | Publication + second + write | Second + write | Complete |
|---|---:|---:|---:|---:|
| Sparse4 / named-A | .9824 [.9529, 1.0129] | 1.0892 [1.0601, 1.1192] | .9826 [.9542, 1.0117] | 1.0213 [.9966, 1.0465] |
| Sparse4 / named-B | 1.0129 [.9739, 1.0534] | 1.0454 [1.0001, 1.0927] | .9428 [.9002, .9875] | 1.0205 [.9875, 1.0545] |
| Named-B / named-A | .9700 [.9316, 1.0099] | 1.0420 [.9962, 1.0899] | 1.0421 [.9964, 1.0900] | 1.0008 [.9705, 1.0321] |

The publication-inclusive column subtracts the three recorded old-first-root components from complete time. The second-stage column sums the four recorded second phases. Publication itself averaged 1.542 ms per ten-block round on sparse4. The arithmetic mean original-root times were 24.906 / 23.396 / 23.849 ms for A / B / sparse4. The corresponding publication-inclusive post-root means were 14.583 / 16.218 / 15.894 ms. The displayed ratios summarize paired logarithms, rather than ratios of these arithmetic means.

The control B/A ratio changed across the three successive 48-round periods from 1.0367 to 1.0071 to .9601, and individual six-round cluster ratios ranged from .7963 to 1.1046. Sparse4/A remained 1.0181 / 1.0292 / 1.0166 over the same periods; sparse4/B was .9820 / 1.0220 / 1.0589. This temporal variation explains why an isolated phase or control cannot establish a small speed advantage. The original eight-arm trial remains a separate result.

The preregistration and exact source are `e4c16d27bd45936ed1d9d81dc119e88ac2fbfca8`. The measured executable SHA-256 was `cd952c8edb8aa720de9e055ab5472d69bdf51b10dd311e9f9fa73e7a1b59d4fb`. The same Rust 1.99.1 parent library and ten original tapes as the eight-arm study were used.

```bash
source /workspace/work/env.sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/native/twins
out=/tmp/astra-marker-twins-replay
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
nice -n 12 taskset -c 3 "$out/twins" /tmp/astra-clipped/ten.bin > "$out/paired.csv" 2> "$out/verdict.log"
python3 -B "$d/report.py" "$out/paired.csv" "$out"
```
