# Complete cost of the five dense admissions

On the true 12-stream Wikidata corpus, the unchanged scalar-cap worker spent **1.302% [1.198%, 1.416%] of its full process CPU in the five dense residual emissions**. These admissions accounted for 4,499,868 of 209,697,662 coded bytes (2.146%), and 1,083,380 of 26,567,581 retained rows (4.078%). Their entire inverse including construction cost 2.548% [2.377%, 2.731%] of complete process CPU. The geometric intervals measure ten actual complete-reader observations at W1 with scanner, output, implementation-selected SIMD and block/stream CRC work. [Raw 20 paired controls and metered observations](charge.jsonl) · [Exact workload accounting](charge-summary.json)

Eliminating the dense emissions outright would save their measured CPU share on this host. Applying the independent 17.77% fresh emitter reduction from the five dense blocks gives an estimated **0.231% of process CPU**. This is a conditional scale calculation, and the bracket [0.213%, 0.252%] propagates only uncertainty in the measured CPU share. It does not include uncertainty in the emitter effect or any machine transfer. Mean measured process, dense construction and dense emission times were 694.80, 8.67 and 9.06 ms. The metered/control full process CPU ratio was 1.051 [0.994, 1.112]. The separate completeness rehearsal checked all 209,700,196 emitted bytes against libbzip2. [Rehearsal](rehearsal.json) · [Original preregistration](PLAN.md) · [Fresh emit interval](../fresh/README.md)

The isolated selective worker computes the original capped-record density boolean once. The existing budget rejects unfitting dense blocks. On accepted blocks the scalar constructor builds exactly the original table, then evaluates the one- versus four-forward emitter from the already known bit. The 228 ordinary admissions and the five dense admissions are determined by the unchanged rule on the entire basis, including primary-split handling and its original price. The worker chooses no particular stream ordinals. [Generator](build_worker.py) · [Candidate](candidate.rs)

Run the native cost accounting from the repository root with Rust 1.99.1.

```sh
d=research/astra-e/two-seed/multi/dense-only
tmp=$(mktemp -d)
python3 research/astra-e/edge-displacements/build.py "$tmp/support"
python3 "$d/build_audit.py" "$tmp/native" "$tmp/support/libmibz2.rlib"
bzip2 -dc corpora/wikidata.bz2 > "$tmp/oracle"
"$tmp/native/audit" audit corpora/wikidata.bz2 "$tmp/oracle"
taskset -c 0 nice -n 16 python3 "$d/collect.py" "$tmp/native/audit" corpora/wikidata.bz2 "$tmp/charge.jsonl"
python3 "$d/report_charge.py" | cmp - "$d/charge-summary.json"
```

## Selective implementation

The frozen candidate at `3932c278d9a9a273618d59e4e042f73447b68300` passed the full 338-case authoritative gate, including the 209,700,196-byte Wikidata sample, periodic and multi-cycle cases, malformed RLE1 and CRC checks. In 24 full direct W1 pairs against the same verified scalar executable, the wall ratio was **0.9931 [0.9676, 1.0192]**, CPU was 0.9947 [0.9704, 1.0196] and peak RSS was 1.0033 [0.9905, 1.0162]. The four-round clustered wall interval was [0.9723, 1.0142]. Median process peak RSS was 41,236 KiB on scalar and 41,254 KiB on selective. These processes and their scanner shared one logical CPU at nice 16. The benchmark cannot resolve the 0.23% effect suggested by the phase accounting. [All direct run records](official/runs.jsonl) · [All direct intervals](official/summary.json) · [Full gate](official/gate/summary.json)

The selective source and its exact compiler image remain reproducible. No production or independently verified throughput gain has been established for this selector. The universal four-forward implementation independently regressed at W1, W2 and W4, while these five true dense emissions occupy only 1.3% of full W1 CPU. Testing this selective worker at high W would require a new targeted independent comparison capable of resolving an effect far smaller than the campaign target. [Independent universal four-forward comparison](../../../ci/evidence/four-stream-44467969034.md) · [Fresh-process peak RSS for the headroom audit](rss.jsonl)

Replay the full evidence using `python3 research/astra-e/two-seed/multi/dense-only/official/verify.py`. For a fresh scorer gate, check out `3932c27`, refresh `bench/moonshot .just Justfile` from origin/master, record the scorer revision, run `python3 research/astra-e/two-seed/multi/dense-only/build_worker.py`, then invoke `python3 bench/moonshot/moonshot.py score research/astra-e/two-seed/multi/dense-only/candidate.rs --corpus "$wiki" --no-timing --no-proxy --out "$scratch/gate"`. The direct run uses the archived scalar compiler from the four-forward factorial, verifies the same scorer tree, then invokes `taskset -c 0 nice -n 16 python3 research/astra-e/two-seed/multi/dense-only/official/panel.py run "$scalar_binary" "$selective_binary" "$wiki" "$scratch/runs.jsonl"`. The scorer path for the recorded round was `75297a1d57b2bdf262c3a19f3064848dedf2c493` and it printed `v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. [Exact selective executable](official/worker.xz) · [Compiler and source provenance](official/manifest.json)
