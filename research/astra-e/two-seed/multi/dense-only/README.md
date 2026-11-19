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
