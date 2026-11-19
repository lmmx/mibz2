# Conjugated Open-Return at seven modeled workers

This partial screen used the official chaingrind simulator and scorer metrics at the 7-worker calibration point. Each arm used the same executable. The first group contains frozen whole-stream ordinal 1. The other group contains two large blocks from the earlier systematic structural audit. Ordinal 41 exceeds the density cutoff and ordinal 85 passes it. The simulator warmed the first block of each group before the measured region. All denominators are decoded output bytes.

| Input | Arm | Instructions/B | Modeled cycles/B | L1↔L2 B/B | L2↔L3 B/B | DRAM B/B | Dependent past-L2 links/B | DRAM MLP |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | LazyCells | 19.47 | 10.21 | 36.08 | 23.58 | 6.99 | 0.0147 | 2.60 |
| 1 | opend | 14.55 | 8.81 | 19.86 | 13.03 | 7.42 | 0.0316 | 2.60 |
| 1 | conjugated | 16.21 | 8.60 | 18.84 | 11.91 | 6.67 | 0.0321 | 2.59 |
| 41, 85 | LazyCells | 23.47 | 24.61 | 65.67 | 55.68 | 17.09 | 0.2064 | 2.93 |
| 41, 85 | opend | 20.88 | 25.64 | 57.04 | 49.46 | 17.43 | 0.2221 | 2.92 |
| 41, 85 | conjugated | 21.58 | 25.10 | 55.55 | 48.31 | 16.24 | 0.2232 | 2.93 |

Conjugation reduced modeled cycles relative to opend by 2.3% on the frozen block and 2.1% on the mixed group. It retired more modeled instructions in both groups. Its ordering against LazyCells reversed between the groups. This sample has no whole-corpus timing, calibration-range check or target-machine measurement. Each JSON preserves the raw counters, model configuration, extracted-symbol digest and all metrics.

## Reproduce

Run from the research branch with Rust 1.99.1, the pinned corpus, Valgrind 3.24.0 and its static tool libraries. The following commands create a separate worktree for the immutable candidate. In this recording the latest scorer path revision was 9c31461, which printed no separate version.

```sh
research_root=$PWD
proxy_work=$(mktemp -d)
git worktree add --detach "$proxy_work/tree" e12e5f63173ee7cc6fcc413510ee76cfee0c1eb1
cd "$proxy_work/tree"
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%h origin/master -- bench/moonshot
# v1 replaced the v0 latency function and changed the memory model.
# Replay the published v0 counters with the exact harness, model and tool.
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
export CARGO_TARGET_DIR="$proxy_work/build"
export MOONSHOT_CANDIDATE="$PWD/research/astra-e/open/candidate.rs"
cargo build --locked --release --manifest-path bench/moonshot/Cargo.toml
bash bench/moonshot/chaingrind/build.sh "$proxy_work/vg"
export VALGRIND_LIB="$proxy_work/vg"
python3 "$research_root/research/astra-e/proxy/screen.py" "$PWD" \
  "$CARGO_TARGET_DIR/release/moonshot" "$research_root/corpora/wikidata.bz2" \
  "$VALGRIND_LIB" "$CARGO_TARGET_DIR" "$proxy_work/results"
```

Use the scorer revision recorded in each manifest when comparing runs. Absolute binary digests can depend on the checkout path because the harness embeds it during the build.

PR #608 introduced scorer v1 at `604dac3`. A fresh v1 screen uses separate receipts, individual warm blocks and the arm-dependent bandwidth model. The historical replay checks its inputs against v0 and records both the refreshed revision and the executed revision.
