# Reproduce the cycle 29 port measurement

Use Linux x86-64, the pinned Rust toolchain, Python 3, bzip2, lbzip2 and four available cores. Run from the candidate checkout. The generator modifies a disposable worktree. The uninstrumented binary times all headline comparisons. A second binary adds a clock around each worker's call to `decode_block` for the per-block corpus table.

```bash
set -euo pipefail
repo=$(git rev-parse --show-toplevel)
study="$repo/docs/journal/2026-11-22-open-burst"
old="$repo/docs/journal/2026-11-22-open-construction"
work=$(mktemp -d)
export WIKIDATA="$work/wikidata.bz2" MIBZ2_ADMITTED_BWT="$work/admitted-bwt"
export DENSE_DIR="$work/dense" BENCH_TREE="$work/paired"
mkdir "$DENSE_DIR"
git fetch origin refs/heads/data/admitted-bwt:refs/remotes/origin/data/admitted-bwt
git archive 41c03a5b22d630adb171231f5225da348b8055af | tar -x -C "$work"
python3 "$old/prepare_blocks.py" auth "$MIBZ2_ADMITTED_BWT"
bash bench/moonshot/corpus.sh "$WIKIDATA"
git worktree add --detach "$BENCH_TREE" HEAD
python3 "$study/build_harness.py"
cd "$BENCH_TREE"
for kind in wall detail; do
  features=()
  if [[ $kind == detail ]]; then features=(--features block-clocks); fi
  cargo test --locked --release --lib --no-run "${features[@]}" --message-format=json |
    python3 -c 'import json,sys,shutil; [shutil.copy(r["executable"],sys.argv[1]) for line in sys.stdin if (r:=json.loads(line)).get("executable") and r["target"]["name"]=="mibz2"]' "$work/$kind"
  test -x "$work/$kind"
done
cd "$work"
taskset -c 0-3 ./wall corpuscheck::all_blocks --exact --ignored --nocapture > parity.log
python3 "$old/prepare_blocks.py" dense "$DENSE_DIR" parity.log
python3 "$old/time_pairs.py" ./wall wall a,b sparity,dparity 1
python3 "$old/time_pairs.py" ./wall wall a,b st1,st4,dt1,dt4,scorpus1,scorpus4 12
python3 "$old/time_pairs.py" ./detail blocks a,b scorpus1,scorpus4 12
python3 "$old/summarize.py" wall.jsonl "$study/arms.json" paired.csv
python3 "$study/summarize_blocks.py" blocks.jsonl corpus-blocks.csv
```

The `MEM` records in the parity rounds are the column size and retained-row count. Check the six manifest entries and the five extracted dense blocks against #719's [block inputs](../2026-11-22-open-construction/block-inputs.csv). The separate production checks use the same example, worker counts, bzip2 reference and explicit moonshot arm as [#719](../2026-11-22-open-construction/reproduce.md).

```bash
cd "$repo"
taskset -c 0-3 cargo test --locked --release
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo fmt --all --check
cargo build --locked --release --example bzcat
for w in 1 4 8 10; do
  taskset -c 0-3 target/release/examples/bzcat "$w" < "$WIKIDATA" | sha256sum
done
bzip2 -dc "$WIKIDATA" | sha256sum
echo 'pub type Candidate = mibz2::bwt::OpenCells;' > "$work/open.rs"
MOONSHOT_CANDIDATE="$work/open.rs" taskset -c 0-3 cargo run --locked --release \
  --manifest-path bench/moonshot/Cargo.toml --bin moonshot -- check --arm candidate --wikidata "$WIKIDATA"
```
