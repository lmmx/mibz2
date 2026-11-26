# Complete queue-select worker and scorer correctness receipt

The immutable worker is `a098919de8016b425fd7dd97186d1378c12eb41c`, entry `research/astra-e/edge-displacements/bucket/entry.rs`. Its local generator reads the measured edge constructor, both original endpoint variants and admission at `badb2aec3624bd65606c429b2e6f79e9d9d0050f` and applies the committed bounded-queue transform. Generated sources live under its ignored target directory. The decoder's production defaults and allocator are unchanged.

In a fresh detached worktree, the master refresh gave scorer **path revision `75297a1d57b2bdf262c3a19f3064848dedf2c493`**. The scorer **printed v1 at `81585172f5fb9e7a59f330124779ba084fd6bf04`**. Its full gate passed **338/338**. The raw JSONL records every case, the 209,700,196-byte Wikidata check and the differential 600-mutation case. The scorer checks decoded bytes against libbzip2 and independently recomputes block and combined CRCs. The build and gate ran at nice 16 pinned to CPU0 with one Cargo build job. The scorer reported one available CPU. This run collected correctness only. [All case outcomes](score/check.jsonl) · [Scorer report](score/summary.json) · [Registration](register.jsonl) · [Executed source, corpus and binary hashes](manifest.txt)

Preserve this worker for its tested memory tradeoff. On the full native census its peak retained capacity for the residual and emitted dictionary was 614,472 bytes, versus 1,585,440 for the frozen edge. In twelve fresh local single-worker processes per arm its median peak RSS was 18,570 KiB versus 19,816 KiB for edge, with maxima 18,628 and 19,892 KiB respectively. The paid native inverse used 1.8760 [1.8115, 1.9427] times edge's thread CPU. Its construction and marked emission ratios were respectively 1.1798 and 2.7001. These separate phase and single-worker memory measurements do not establish high-worker i9 throughput or peak RSS. [Paired stage and geometry](../../summary.json) · [Fresh-process memory rows](../../memory.csv)

## Reproduction

The archived run used Rust 1.99.1 and the Wikidata SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. Beginning from the repository, run the following in the isolated worktree. The corpus generation step can use a local copy of the file with the recorded SHA-256. A future master refresh may change the scorer version.

```sh
git worktree add --detach /tmp/astra-queue-select-gate a098919de8016b425fd7dd97186d1378c12eb41c
cd /tmp/astra-queue-select-gate
cat bench/moonshot/README.md
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%h origin/master -- bench/moonshot
bash bench/moonshot/corpus.sh
d=research/astra-e/edge-displacements/bucket
nice -n 16 taskset -c 0 python3 "$d/prepare.py"
CARGO_BUILD_JOBS=1 nice -n 16 taskset -c 0 python3 bench/moonshot/moonshot.py score "$d/entry.rs" --corpus corpora/wikidata.bz2 --no-timing --no-proxy --out target/receipt/score --record target/receipt/register.jsonl
```
