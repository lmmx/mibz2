# Exact bytes from a terminal quotient grammar

All 228 production admissions reduced the marked orbit to one fixed row. Its immutable grammar spelled all 205,197,794 requested bytes correctly. The terminal writer built the original quote-root dictionary and incoming word runs, applied repeated deterministic-class contractions, then expanded one rotated spelling. It eliminated the second token dictionary and residual traversal on these inputs. The production worker remains unchanged. [Implementation](grammar.rs) · [All byte-vector checks](dense-228.txt) · [Full geometry](admissions-dag.csv)

## Construction and cost

The first stage is the native named quote construction, including caps, frequencies, primary transfer, word discovery and publication. Grammar leaves reference byte ranges in that existing immutable dictionary. Each selected class creates exactly one node containing its old handle, the old successor handle and their combined length. The old grammar survives each fold. The active incoming RLE, the class masses and the current handle per active class use two reusable buffers. Stable weaving and recognition charge every round. No intermediate expanded byte dictionary is copied.

The initial projection sorts the named leaf nodes by source base and maps old word IDs to dense source-ordered IDs. Within a round, recognition identifies the target class of each eligible source and verifies the marked row's exact outgoing destination by occurrence rank. The frozen functional-graph selector finds a maximum-weight compatible set. At publication, selected classes retain their source but delete their successor rows. If that deletion contains the mark, the corresponding surviving source acquires the mark with its old prefix length added to the byte offset. The code evaluates that prefix against the older grammar generation. These are the one-round contraction's exact embedding and offset update. Iteration preserves the entire marked weighted word. [One-round proof](../../PROOF.md) · [Independent source selection](../../native/verify.py) · [Python iteration](../trace.py)

For a round with R incoming runs and K active classes, each target bucket is a disjoint interval in source rank. Intersecting these buckets with the incoming runs has at most R+K nonempty fragments. Consuming those fragments into their target occurrences can introduce at most R additional boundaries. Recognition, maximum-weight selection and publication therefore cost O(R+K). Storage for handles is linear in the number of initial words and selected class productions. Repeatedly rebuilding the run column can still cost quadratic work in the initial number of rows in the worst case. The observed sum of input runs across rounds is a construction cost.

On a marked fixed point the current word is one complete period. For length L and stored offset b, the writer expands ranges [b,L) and [0,b), then doubles the emitted period and clips to n. The spanning grammar is traversed with an explicit stack. Each original word occurrence receives one leaf visit, with a possible second visit to the leaf split by the byte offset. Each internal vertex of the unrolled tree is visited once plus possible repetition on the single split spine. There are no forward row lookups during grammar extraction. The old named object stays intact. A saturated nonterminal object or an uncovered mark takes the original named writer. The explicit quote-absent harness fallback expands the column and calls the crate's Segmented inverse. No such fallback occurred on the production admissions.

A saturated nonterminal stable column exists. Let source buckets be A,A,B,B and the incoming column B,A,B,A. Its forward permutation is [1,3,0,2]. Both source classes land in both classes. Every mark on this four-cycle therefore uses the exact fallback. A singleton source guarantees a deterministic successor when the marked row is nonfixed, but this implementation leaves source IDs unsplit. [Singleton-anchor theorem](../GRAMMAR.md)

## Exact coverage and space

All permutations of tiny raw columns in the harness family were checked at every primary against the bytewise stable-sort LF. Among 13,567 marked cases, 10,030 reached terminal extraction, including 6,373 whose reduced object still contained other rows. The tests exercised 2,538 mark transfers and 5,241 shorter periods requiring a partial final copy. There were 2,159 uncovered marked cycles, 712 saturated nonterminal cases and 666 quote-absent cases. All outputs agreed. [Small receipts](small.csv)

Both the instrumented and timed extraction specializations also passed independent full byte-vector equality on each real admission. The ten systematic cases matched every terminal depth and cumulative input-run and class count in the Python calculation, after charging one extra final one-row check. The full corpus used 31,414 fold rounds. Individual counts ranged from 31 to 1,515. In total, the writer examined 46,981,268 run records and 14,767,748 classes across the rounds, retained 10,085,721 grammar nodes summed over blocks, and made 30,868,579 leaf visits. The deepest observed emission stack held 1,514 entries. These counts distinguish construction and output from the small terminal state.

The full sequence's highest retained vector capacities including output were 8,179,526 bytes for grammar and 8,473,542 bytes for the original named writer. Fresh-process VmHWM was 8,048 and 8,212 KiB respectively. The admitted joint edge constructor was lower at 6,384 KiB. These are single-process native measurements with pre-existing cap tapes. They do not establish production peak RSS. Audit counters in the leaf/internal loop are compile-time disabled in the paid variant. [Named memory](admissions-named.csv) · [Joint memory](admissions-joint.csv) · [Build and input hashes](manifest.json)

## Paid comparison

Across 32 balanced rotations of ten blocks, the terminal grammar took 3.6003 [3.3605, 3.8573] of the first named writer's full inverse CPU time, 3.5426 [3.3558, 3.7398] of the second's and 3.1180 [2.9181, 3.3315] of the scalar cap/counter joint edge constructor's. The named twins' ratio was 1.0163 [0.9550, 1.0815]. All 1,280 timed outputs matched the independent marked bytes. Every arm occupied every position once per block per four-round cluster. Intervals use the eight disjoint chronological clusters. Samples were pinned to CPU0 at nice +16 in the local container. A separate native study occupied CPU3 and a low-priority scorer proxy occupied CPU4. The native output costs are not independent CI or target-machine evidence. [Raw samples](phase.csv) · [Paired calculation and phase means](summary.txt)

The terminal arm averaged 7.289 ms per block. The first named object and marked position accounted for 1.216 ms, DAG projection 0.126 ms, repeated folds 4.178 ms and extraction 1.765 ms. The entire unchanged named decoder averaged 2.055/2.079 ms per block across its two instances, with about 0.77 ms left after its first object. Joint edge took 2.368 ms. Even free DAG extraction would leave repeated contraction substantially slower than the original second stage. Even free contractions would leave this leaf-by-leaf emitter above the available second-stage time. The preregistered promotion rule failed.

A structural follow-up can test incremental updates of the deterministic-class frontier rather than republishing every surviving run. A corresponding output test would need to expand shared subwords once and copy previously emitted spans. Price the recognition of unchanged runs and the construction and use of the output references. The present timing requires improvement in both phases before these two replacements could justify a complete worker.

## Reproduction

The stage starts from the ten frozen capped-record inputs from the earlier native study. The 228-input manifest uses original stream indices while the ten-input file uses local sample ordinals. The ten-input oracle computes complete output bytes directly. For each retained production block, '-' requests the independent full-byte oracle and the separate expected.csv mode avoids including the dense oracle in the HWM process. The common builder verifies the inherited source hashes. Rust 1.99.1 and the existing release research library were used. [Input preparation](../../../../../../native/README.md) · [Frozen comparison rule](PLAN.md)

```sh
source /workspace/work/env.sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/fold/iterated/native
out=/tmp/astra-terminal-grammar
python3 -B "$d/build.py" "$out" "$PWD/target/release/deps"
"$out/grammar"
"$out/grammar" census /tmp/astra-clipped/ten.bin - 2
"$out/grammar" census /tmp/astra-token-all/admissions.bin - 2
for a in 0 2 3; do
  "$out/grammar" census /tmp/astra-token-all/admissions.bin /tmp/astra-clipped-final/expected.csv "$a"
done
taskset -c 0 nice -n16 "$out/grammar" paid /tmp/astra-clipped/ten.bin > /tmp/grammar-phase.csv
python3 "$d/report.py" "$d"
```
