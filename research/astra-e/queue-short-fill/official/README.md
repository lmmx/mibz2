# BOTH1 full scorer gate

The frozen width-one specialization of the combined cap-counter worker passed **338 of 338** authoritative cases. The gate checked libbzip2 output, recomputed block and stream CRCs, exercised periodic and multi-cycle columns, checked 600 mutations against the release kernel, and decoded the 209,700,196-byte Wikidata sample. It ran with one-CPU affinity. Its output contains correctness evidence only. [Raw cases](score/check.jsonl) · [Scorer summary](score/summary.json) · [Independent receipt replay](verified.txt)

The candidate commit was `d38b8af5aac80579da3ede854c65c9469d18bd03` and the development entry was `research/astra-e/queue-short-fill/both1.rs`. The detached worktree refreshed the scorer from master before the round. The master revision was `3df195b10e9fb6f86f0c06ad9fad1feb992406d5`, the last change to the scorer path was `75297a1d57b2bdf262c3a19f3064848dedf2c493`, and both the printed version and the scorer JSON give `v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. The eighteen projected scorer blobs matched the master tree individually. The raw executable SHA-256 was `37f3660caeb77c5b0ec219301f301ac61310d50d1bd44dfe7b3631923eaf1c3d`. The generator embeds absolute include paths. The evidence records both the compiled source hash and a path-normalized constructor hash. [Candidate and executable hashes](hashes.json) · [Scorer blobs](scorer-blobs.txt) · [Build and gate log](stderr.txt)

From a repository checkout with the frozen Wikidata corpus, Rust 1.99.1 and the scorer's prerequisites, use a fresh detached worktree. The scorers compared within this round must have the same versions and blobs. For a new live round, repeat the required refresh and record both versions again.

```sh
oracle=$(realpath corpora/wikidata.bz2)
git worktree add --detach /tmp/astra-both1-gate d38b8af5aac80579da3ede854c65c9469d18bd03
cd /tmp/astra-both1-gate
cat bench/moonshot/README.md
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%h origin/master -- bench/moonshot
python3 research/astra-e/queue-short-fill/prepare.py
CARGO_BUILD_JOBS=1 nice -n 16 taskset -c 0 python3 bench/moonshot/moonshot.py score research/astra-e/queue-short-fill/both1.rs --corpus "$oracle" --no-timing --no-proxy --out /tmp/astra-both1-gate-results
```

The retained executable in the campaign workspace is `/workspace/work/astra-queue-both1-gate/bench/moonshot/target/release/moonshot`. The archived scorer options used its default worktree-local target directory. The standalone receipt check is `python3 research/astra-e/queue-short-fill/official/verify.py`.
