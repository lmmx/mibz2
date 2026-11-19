# Independently built complete joint control

The official full gate passed all 338 cases. This binary instantiates the unchanged scalar cap/counter decoder. The generated constructor agrees byte-for-byte with `kernel.rs` at `50b8ecb8946a498601e3fd894ed84a2687df6cc8` after restoring its three relative include paths. Inputs, executable, scorer tree, production tree, target paths and SHA-256 hashes appear in [manifest.json](manifest.json). The gate's [JSONL](check.jsonl) and [summary](summary.json) are preserved. This receipt contains no worker timing or peak-RSS sample.

To rebuild from a new detached worktree, run the following from an Astra E checkout with the frozen corpus at the path below. The refresh on this run fetched `origin/master` at `3df195b10e9fb6f86f0c06ad9fad1feb992406d5`. Its most recent scorer path commit was `75297a1`. The scorer printed v1 at `81585172f5fb9e7a59f330124779ba084fd6bf04` from the worktree's history. Both have scorer tree `75f7b0321bd71c5086cbfb34747f7ea963463784`. A historical reproduction can select this frozen tree after refreshing, if master has since moved.

```sh
git worktree add --detach /workspace/work/astra-qfill-both0 d38b8af5aac80579da3ede854c65c9469d18bd03
cd /workspace/work/astra-qfill-both0
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%h origin/master -- bench/moonshot
source /workspace/work/env.sh
export CARGO_BUILD_JOBS=1
taskset -c 0 nice -n16 python3 research/astra-e/queue-short-fill/prepare.py
taskset -c 0 nice -n16 python3 bench/moonshot/moonshot.py score research/astra-e/queue-short-fill/both0.rs \
  --target-dir /workspace/work/astra-qfill-both0-build --out /workspace/work/astra-qfill-both0-receipt \
  --corpus /workspace/work/astra-e/corpora/wikidata.bz2 --no-timing --no-proxy
```

The v1 command line exposes building through `score`, which calls `Scorer.build`. The build uses `cargo build --release --locked` and the candidate path in `MOONSHOT_CANDIDATE`. For direct repeat-two W1 measurements of the resulting executable, the scorer uses the following invocation.

```sh
/workspace/work/astra-qfill-both0-build/release/moonshot run --arm candidate --workers 1 \
  --file /workspace/work/astra-e/corpora/wikidata.bz2 --repeat 2
```
