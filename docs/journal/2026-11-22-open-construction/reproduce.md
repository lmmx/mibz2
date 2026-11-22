# Reproduce the native comparisons

The binary generator starts from the scripts posted with #714. It imports each wall and phase arm from Git, keeps the master wall kernel intact and adds a test for each direct pair. Run the generators in disposable worktrees. The checked-in bundle contains the measured source snapshots, including the experimental layouts, with #714 as its prerequisite. The refs below import into a local benchmark namespace. The measured two-ended revision is `89f95ce`. The PR adds the measurements, explanatory comments and a chunk-rollover test to that final idea commit. The release implementation used for the timing is identical apart from comments.

```bash
repo=$(git rev-parse --show-toplevel)
study="$repo/docs/journal/2026-11-22-open-construction"
work=$(mktemp -d)
git fetch origin refs/heads/data/admitted-bwt:refs/remotes/origin/data/admitted-bwt
git bundle verify "$study/experimental.bundle"
git fetch "$study/experimental.bundle" 'refs/heads/*:refs/bench/open-construction/*'
git archive origin/data/admitted-bwt | tar -x -C "$work"
bash bench/moonshot/corpus.sh
export WIKIDATA="$repo/corpora/wikidata.bz2"
export MIBZ2_ADMITTED_BWT="$work/admitted-bwt"
export DENSE_DIR="$work/dense"
mkdir -p "$DENSE_DIR"
python3 "$study/prepare_blocks.py" auth "$MIBZ2_ADMITTED_BWT"
export BENCH_TREE="$work/paired"
git worktree add --detach "$BENCH_TREE" 8ff1187af6a920f7d57fd338680db3a5633a3ac5
xargs python3 "$study/build_harness.py" < "$study/refs.txt"
cd "$BENCH_TREE"
cargo test --locked --release --lib --no-run
exe=$(find "$BENCH_TREE/target/release/deps" -maxdepth 1 -type f -executable -name 'mibz2-*' -print -quit)
cd "$work"
taskset -c 0-3 "$exe" corpuscheck::all_blocks --exact --ignored --nocapture > parity.log
python3 "$study/prepare_blocks.py" dense "$DENSE_DIR" parity.log
arms=a,p,l,lv,t,tv,g,gv,b,bv,e,ev,o,ov,i,iv,k,kv,x,xv,q,qv
python3 "$study/time_pairs.py" "$exe" measured "$arms" sparity,dparity 1
python3 "$study/time_pairs.py" "$exe" measured "$arms" st1,st4,sstage,sstage4,dt1,dt4,dstage,dstage4,scorpus1,scorpus4,scs1,scs4 12
python3 "$study/time_pairs.py" "$exe" measured a,x,q st2 12
python3 "$study/summarize.py" measured.jsonl "$study/arms.json" paired.csv
```

The membership and order of the two panels come from their manifest files. The driver appends one JSON row after each successful process and resumes from complete rows. The prefix `s` selects the six columns, `d` the dense five. Corpus modes always use the whole compressed file. Arm suffix `v` selects a direct comparison with the previous revision. The table records elapsed-time ratios, including all denominators and intervals. Its units are microseconds for block timings and milliseconds for stream timings. A blank ratio indicates a removed or unexecuted phase.

The heap measurement has its own binary, using the allocator instrument from #703. The caller's output vector is preallocated before setting the live-heap baseline.

```bash
export BENCH_TREE="$work/heap"
git -C "$repo" worktree add --detach "$BENCH_TREE" 8ff1187af6a920f7d57fd338680db3a5633a3ac5
xargs python3 "$study/memory_harness.py" mm=8ff1187 pp=26a5f6a < "$study/refs.txt"
cd "$BENCH_TREE"
CARGO_PROFILE_TEST_OPT_LEVEL=1 CARGO_PROFILE_TEST_DEBUG=0 cargo test --lib memo::corpus_memory -- --nocapture
```

To obtain the retained capacities of the four live decoder workers, reuse the heap worktree and replace its instrumentation before running the worker test.

```bash
xargs python3 "$study/four_workers.py" mm=8ff1187 pp=26a5f6a < "$study/refs.txt"
CARGO_PROFILE_TEST_OPT_LEVEL=1 CARGO_PROFILE_TEST_DEBUG=0 taskset -c 0-3 cargo test --lib memo::four_workers -- --nocapture
```

## Overlap and ablation binaries

Use a fresh master worktree for each binary and rerun the build and capture steps above. The ablation binaries used seven stage counters. Their original S5 clock includes the separate S3b pass on master. The published per-commit table uses the final eight-counter binary, with exclusive S5 and an additional nested bottom-walk clock.

```bash
PROFILE_VERSION=screen PARENT_g=q python3 "$study/build_harness.py" q=96cd177 z=34be86f h=8040767 g=4b42ba0 b=129e6df e=96413f9 o=1707116 i=fffd91d k=0dde60a x=6e1e026 u=455e624
# Separate binary for the assembled geometry and cursor comparisons
PROFILE_VERSION=screen PARENT_w=u PARENT_j=u PARENT_c=j python3 "$study/build_harness.py" g=96cd177 u=455e624 w=6bea321 j=7ac1bda c=9a105f5
```

For the assembled geometry comparison, time arms `a,w,wv` using the second binary. For the four-way wiring comparison, time `a,u,uv` using the first binary. Both use the same 12-round mode list as the main study. In ablations.csv the isolated reruns are named `geometry` and `unroll`. The earlier screening runs are named `screen` and `overlap`. The ablation CSV uses microseconds throughout. The raw ablation archive prefixes each measurement with its experiment name.

The exactness checks used the shipped example and the explicit Open candidate in moonshot.

```bash
cd "$repo"
taskset -c 0-3 cargo test --release
cargo clippy --all-targets -- -D warnings
cargo build --release --example bzcat
bzip2 -dc "$WIKIDATA" | sha256sum
for workers in 1 4 8 10; do
  taskset -c 0-3 target/release/examples/bzcat "$workers" < "$WIKIDATA" | sha256sum
done
mkdir -p "$work/moon-candidate"
echo 'pub type Candidate = mibz2::bwt::OpenCells;' > "$work/moon-candidate/open.rs"
MOONSHOT_CANDIDATE="$work/moon-candidate/open.rs" taskset -c 0-3 cargo run --locked --release --manifest-path bench/moonshot/Cargo.toml --bin moonshot -- check --arm candidate --wikidata "$WIKIDATA"
```


The mutation table in the journal gives the source replacement and test for each idea. Restore the original file between replacements. The four-way wiring trial used a small reader-level check to isolate the individual lanes. Generate its test binary in the scratch directory.

```bash
git show 455e624:src/open/outq.rs > "$work/xor-trial.rs"
cat "$study/xor_lane_test.rs" >> "$work/xor-trial.rs"
# outq uses pub(super) items, so include the trial inside a module.
echo '#[path = "xor-trial.rs"] mod queue;' > "$work/xor-module.rs"
rustc --edition 2021 --test -A warnings "$work/xor-module.rs" -o "$work/xor-test"
"$work/xor-test" eight_nonmonotone
```
