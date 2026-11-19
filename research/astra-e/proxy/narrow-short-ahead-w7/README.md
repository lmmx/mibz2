# Narrow endpoint, short words and one-edge lookahead at the W7 model point

The official Chaingrind model estimates 3.5% fewer cycles than opend on frozen stream block 1 and 1.4% fewer on the pair 41 and 85. The candidate reduces instruction work and cache-line transfers at each level on both samples. Its dependent past-L2 links per output byte rise slightly on both, and the longest chain rises from 1,170 to 1,263 on the pair. DRAM MLP changes by less than 0.4%. Against LazyCells the candidate reduces modeled cycles/B by 16.7% on the frozen block and increases them by 2.7% on the pair. These directional differences limit extrapolation to a corpus or to other concurrency.

| Whole-stream block ordinals | Arm | Instructions/B | Modeled cycles/B | L1↔L2 B/B | L2↔L3 B/B | DRAM B/B |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1 | lazy | 19.475 | 10.204 | 36.069 | 23.580 | 6.988 |
| 1 | opend | 14.553 | 8.808 | 19.860 | 13.030 | 7.419 |
| 1 | candidate | 14.228 | 8.498 | 18.124 | 12.411 | 7.119 |
| 41,85 | lazy | 23.466 | 24.603 | 65.675 | 55.687 | 17.086 |
| 41,85 | opend | 20.879 | 25.645 | 57.037 | 49.453 | 17.435 |
| 41,85 | candidate | 20.633 | 25.277 | 56.075 | 48.654 | 17.001 |

| Blocks | Arm | Dependent past-L2 / DRAM links per B | Longest past-L2 / DRAM chain | DRAM MLP | Overlap per dependent DRAM link | Full counters |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| 1 | lazy | .01466 / .000037 | 18 / 12 | 2.608 | 7.03 | [receipt](frozen-lazy.json) |
| 1 | opend | .03159 / .000037 | 357 / 12 | 2.598 | 7.03 | [receipt](frozen-opend.json) |
| 1 | candidate | .03182 / .000037 | 358 / 12 | 2.589 | 7.15 | [receipt](frozen-candidate.json) |
| 41,85 | lazy | .20637 / .006612 | 759 / 104 | 2.927 | 3.48 | [receipt](density-pair-lazy.json) |
| 41,85 | opend | .22208 / .006614 | 1170 / 104 | 2.923 | 3.47 | [receipt](density-pair-opend.json) |
| 41,85 | candidate | .22218 / .006620 | 1263 / 104 | 2.924 | 3.48 | [receipt](density-pair-candidate.json) |

Each group runs in one thread at 2,925 KiB LLC share, 66-cycle L3 and 549-cycle DRAM. The scorer warms the first block outside the measured region and decodes the entire group inside it. Block 41 exceeds the route density threshold and block 85 is admitted. All six counters came from one frozen workstation-rehearsal executable, SHA-256 `a17f57b69946e4b74715174c06e274f8757bddfe002185d4a42dc3ea9148ab98`. This partial screen omits the whole-score calibration envelope. [Model, input and tool provenance](manifest.json)

Across 11 different prepared run fronts, 21 matched process-local passes found a candidate/opend wall ratio of .94960 [.91498,.98553] and a thread CPU ratio of .95065 [.91712,.98541] during a concurrent dry run. The full-stream three-pair pilot gave candidate/lazy .797 [.325,1.950] at W1 and .655 [.360,1.192] at W4. Its paired candidate/opend point estimates were .893 and 1.051, respectively. Prepared fronts omit input processing, RLE1 and CRC. The whole-stream pilot and model cover different populations, and only the first measures wall time. The independent 1/2/4-thread CI comparison was pending at the time this model was recorded. [Stage](../../narrow-short/stage/summary.csv) · [Pilot](../../results/combined-ahead-pilot/score/summary.json) · [CI](https://github.com/lmmx/mibz2/actions/runs/44427324957)

## Freeze and repeat

Source and recipe share commit `0694800bf4b2ef5ece720f7c43b8ab0ed7dced9b`. Fetch and prescribed path checkout selected scorer `9c31461df7bef24cecf79348545b27ee26846835`, whose binary prints no version field. The exact executable passed 338 full-gate cases on the four-CPU host. Its rehearsal recorded zero target worker counts exercised. All four target counts remain necessary in the independent dry run.

Regeneration in a separate detached checkout produced the same narrow endpoint and two constructors as the earlier stage manifest. The table lists the sources included for the selected arm. The generated adapter implements the standard inverse trait through the existing density gate. All four frozen benchmark arms take the default hook returning `None` for the experimental fully expanded route. The release run-record accessor still returns the same slice. The new decoder hook and public accessor are guarded by the research feature. [Stage source hashes](../../narrow-short/stage/manifest.txt) · [Frozen decoder changes versus release](https://github.com/lmmx/mibz2/compare/530ed404a014a414ef0b62adfd947fa14d4533e6...0694800bf4b2ef5ece720f7c43b8ab0ed7dced9b) · [Worker and measurement protocol](../../workstation/README.md)

| Selected file in narrow-short | SHA-256 |
| --- | --- |
| ahead.rs | 7b7741a017f12aaf5c9624028e32715ad4fd806210bef0c6b210c7c1d16631d6 |
| adapter.inc.rs | 210e294a9e91abd4a6ef39a474a69d41b1e434b06decf208d52636196f836e79 |
| narrow.rs | 252301a541ee67617a447dc1755ab39d337ab9975e43816345a2fa6296d58224 |
| constructor_ahead.rs | c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f |

The screen driver is tracked in the research branch. The harness embeds build paths, so a recompile from a different directory can change the executable digest. Use Rust 1.99.1 and Valgrind 3.24.0 with the scorer's static tool libraries.

```sh
research_root=$PWD
proxy_work=$(mktemp -d)
git worktree add --detach "$proxy_work/tree" 0694800bf4b2ef5ece720f7c43b8ab0ed7dced9b
cd "$proxy_work/tree"
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%h origin/master -- bench/moonshot
bash research/astra-e/prepare.sh research/astra-e/narrow-short/ahead.rs
export CARGO_TARGET_DIR="$proxy_work/build"
export MOONSHOT_CANDIDATE="$PWD/research/astra-e/narrow-short/ahead.rs"
cargo build --locked --release --manifest-path bench/moonshot/Cargo.toml
bash bench/moonshot/chaingrind/build.sh "$proxy_work/vg"
export VALGRIND_LIB="$proxy_work/vg"
python3 "$research_root/research/astra-e/proxy/screen.py" "$PWD" \
  "$CARGO_TARGET_DIR/release/moonshot" "$research_root/corpora/wikidata.bz2" \
  "$VALGRIND_LIB" "$CARGO_TARGET_DIR" "$proxy_work/results"
```
