# Combined narrow endpoint, immediate words and lookahead under scorer v1

The frozen build is `6a8a366cd786d58a528000b58da12fd2cfb47be8`. The candidate entry has SHA-256 `7b7741a017f12aaf5c9624028e32715ad4fd806210bef0c6b210c7c1d16631d6`, also recorded at the narrow-short workstation freeze. Master supplied scorer v1 at `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`. The four included candidate source hashes, model, executable, corpus and 64 extracted symbol tapes are in the [provenance receipt](checkpoint/provenance.json).

The quick libbzip2/CRC gate passed all 98 cases. The full 338-case gate for this candidate is separately retained with its [workstation rehearsal](../../workstation/rehearsals/narrow-short-0694800.tar.gz). The official proxy runs fresh LazyCells and candidate workers at nominal and elevated DRAM latency, comparing them with v1's deposited reference arms. Its sample has 48 full and 16 tiny blocks, weighted to the corpus populations. Each of the six worker models requires 160 simulator invocations. Each invocation warms and measures one block in a single worker. Whole-process wall time and peak RSS belong to the paired benchmarks. [Gate receipts](checkpoint/check.jsonl) · [Invocation and environment](manifest.txt)

## Complete official score

The scorer printed `v1` at `6a8a366cd786d58a528000b58da12fd2cfb47be8`. The 960 nominal and high-latency simulations took 58 minutes 42 seconds on one low-priority worker on the shared Xeon. Reloading them through the pinned scorer reproduced every field of its calibrated proxy, including the 400 paired block-bootstrap comparisons. [Official score](score/summary.txt) · [All simulator counters](score/proxy-runs.jsonl) · [Rebuilt comparisons](comparison.csv)

| W model | Candidate / lazy [95%] | Deposited opend / lazy | Ratio of point estimates | Candidate / opend instructions | Candidate / opend DRAM B/B | Candidate / opend dependent past-L2 links/B | Reference envelope |
|---:|---|---:|---:|---:|---:|---:|---|
| 1 | 0.8596 [0.8293,0.8905] | 0.8695 | 0.9886 | 0.9759 | 0.9719 | 1.0021 | dep_dram_links_per_byte 1.182e-05 outside [1.588e-05, 1.66e-05] |
| 2 | 0.8589 [0.8284,0.8899] | 0.8688 | 0.9886 | 0.9759 | 0.9719 | 1.0021 | dep_dram_links_per_byte 1.182e-05 outside [1.588e-05, 1.66e-05] |
| 4 | 0.8570 [0.8258,0.8889] | 0.8674 | 0.9880 | 0.9759 | 0.9589 | 1.0022 | dep_dram_links_per_byte 1.297e-05 outside [1.588e-05, 2.241e-05] |
| 7 | 0.9102 [0.8811,0.9376] | 0.9558 | 0.9522 | 0.9759 | 0.8964 | 1.0016 | within |
| 8 | 0.9118 [0.8817,0.9396] | 0.9466 | 0.9632 | 0.9759 | 0.9458 | 1.0007 | within |
| 10 | 0.9155 [0.8740,0.9516] | 0.9445 | 0.9693 | 0.9759 | 0.9630 | 1.0001 | within |

The candidate/lazy bootstrap compares the same blocks measured by this invocation. Opend uses the v1 reference deposited on master. Only the fresh candidate/lazy comparison supplies a paired interval. Each result models a worker at its share of LLC. The actual CI and i9 perform the paired wall and peak-RSS tests. [Corresponding independent CI](https://github.com/lmmx/mibz2/actions/runs/44427324957) · [Complete route shares](route-shares.json)

Recompute the calibrated grid and check the archived interpretation with the following command, supplying a checkout of the historical v1 scorer.

```sh
python3 research/astra-e/results/combined-v1-proxy/read_v1.py /path/to/frozen/checkout
```

## Earlier checkpoint and recovery

The checkpoint preserves 800 complete official observations, all W1, W2, W4, W7 and W8 rows. The 240-, 480- and 640-row checkpoints remain at commits `6a5d160`, `b633eed` and `1c895fc`. W8 gives 0.9118 [0.8817,0.9396] against fresh lazy, within the v1 reference envelope. A source check confirms that v1's `proxy-runs.jsonl` receives one record per block, with normal file buffering. The full official reading above includes these observations.

The original study ran in `/workspace/work/narrow-proxy/v1-tree` and wrote `research/astra-e/results/combined-v1-full-proxy/score/proxy-runs.jsonl`. Its target directory is `/workspace/work/narrow-proxy/v1-target`. Its shell entry was `/workspace/work/narrow-proxy/v1-run.sh`. The archived counters above contain all complete rows.

The following historical continuation uses Python 3.13.5 with `PYTHON_CPU_COUNT=1`, Rust 1.99.1, the saved build tree, Valgrind 3.24.0 and its development libraries. The verifier matches the input, model and both executables. The continuation rechecks the gate, regenerates each symbol tape and lets the official scorer execute all missing jobs, model and report. Its only change to the cached runs is reusing the exact simulator response to the same arguments and verified tape. Choose a new output directory. A build whose tool or executable digest differs requires a fresh whole score.

```sh
source /workspace/work/env.sh
source /workspace/work/proxy-tools/env.sh
export PYTHON_CPU_COUNT=1 CARGO_BUILD_JOBS=1
replay=research/astra-e/results/combined-v1-proxy/checkpoint/resume.py
frozen=/workspace/work/narrow-proxy/v1-tree
target=/workspace/work/narrow-proxy/v1-target
/usr/bin/python3.13 "$replay" "$frozen" "$target" /tmp/astra-e-combined-v1-recovery --check-only
nice -n 12 /usr/bin/python3.13 "$replay" "$frozen" "$target" /tmp/astra-e-combined-v1-recovery
```

The following command refreshes from master for a new comparison. A changed upstream implementation begins a separate series.

```sh
nice -n 12 bash research/astra-e/score.sh combined-v1-repetition \
  research/astra-e/narrow-short/ahead.rs --target-dir /tmp/astra-combined-v1 \
  --quick --no-timing --proxy-blocks 48+16
```

To replay the historical scorer independently, start a fresh checkout of the research build, use Rust 1.99.1, and have Valgrind 3.24.0 and its static headers/runtime available. The result directory must be new. The corpus recipe requires lbzip2 and verifies the Wikidata input.

```sh
git checkout --detach 6a8a366cd786d58a528000b58da12fd2cfb47be8
git checkout 604dac3f46569edd9ac9b6706b23a68d9d7d8f8d -- bench/moonshot .just Justfile
study=research/astra-e/narrow-short/ahead.rs
bash research/astra-e/prepare.sh "$study"
bash bench/moonshot/corpus.sh
PYTHON_CPU_COUNT=1 CARGO_BUILD_JOBS=1 nice -n 12 python3.13 bench/moonshot/moonshot.py \
  score "$study" --target-dir /tmp/astra-v1-historical-build --out /tmp/astra-v1-historical-score \
  --quick --no-timing --proxy-blocks 48+16 --record /tmp/astra-v1-historical-score/record.jsonl
```

## Dense blocks and the wider admission experiment

Five full corpus blocks exceed the combined candidate's run-density cutoff. The separate [edge-budget candidate](../../edge-budget/) admits all five. Their share of the canonical output is 2.15%. The official v1 sample includes one of them, stream ordinal 165, and it contributes 2.08% of corpus-weighted sample bytes. In contrast, the highest-latency runs used to estimate each arm's DRAM slope contain none of the five. That distinction matters when using this proxy to discuss policy changes restricted to dense inputs.

At nominal W1 ordinal 165 accounts for 2.93% of the combined arm's modeled cycles, 2.84% of instructions, 19.58% of DRAM read lines and 22.53% of DRAM lines including writes and final dirty drainage. Changing its route could influence traffic more than its work share suggests. These figures describe the combined candidate's existing costs. Buffer lifetime, between-block interference and the effect of another implementation at 7–10 workers require independent measurements. [Population and raw-counter decomposition](checkpoint/route-shares.json) · [Analysis program](route_shares.py)

```sh
python3 research/astra-e/results/combined-v1-proxy/route_shares.py \
  research/astra-e/results/combined-v1-proxy/checkpoint/proxy-runs.jsonl.gz
```

The same accounting gives a conditional lower bound for an isolated change to those dense blocks. At fixed nominal latency let C/L be the combined arm's nominal cycle ratio over lazy, and f the combined arm's fraction of cycles on the sampled dense block. Even giving that block zero cost leaves C(1−f)/L if all other measured work stays fixed. At W7, f=0.07417 and C/L=0.88983 leave **0.82382**. At W8 and W10 the corresponding floors are **0.78620** and **0.71940**. Under those W7 assumptions, reaching 0.80 needs a change to another block or to shared costs. The one sampled dense block, arm-specific latency feedback, reuse across worker slots and allocator-capacity effects limit this inference about the workstation. [Charged fractions](route-shares.json) · [Nominal ratios](score/proxy.json)

## The calibration's independent controls

The scorer's calibrated reference recovers all nine held-out pairs in its original high-worker comparison. The later 12-pair i9 opend sweep provides another check at equal concurrency. At W2 the v1 reference predicts opend/lazy 0.869 [0.838,0.900]; the i9 saw 0.950 [0.932,0.969]. At W4 the model predicts 0.867 [0.836,0.900], and the i9 saw 0.953 [0.939,0.967]. Its low-concurrency opend estimates miss by more than eight percentage points. At W7, W8 and W10 the respective model estimates are 0.956, 0.947 and 0.944, and their block-bootstrap intervals include the matched i9 observations of 0.934, 0.952 and 0.933. The W7 and W8 reference configurations also occur in the original calibration bundle. These are separate experimental sweeps on the same workstation. [Upstream v1 calibration](https://github.com/lmmx/mibz2/blob/604dac3f46569edd9ac9b6706b23a68d9d7d8f8d/bench/moonshot/calibration.json) · [Raw i9 sweep and pinned recipe](../../prior.md)


## Seven-worker result from the complete W7 grid

Once W7 is present, the scorer has the fresh LazyCells anchor for its bandwidth fixed point. Its 400 paired stratified draws model candidate/lazy at W7 as **0.9102 [0.8811,0.9376]**, within the reference envelope. Candidate/lazy is 0.7116 for instructions and 0.8898 for cycles at the nominal latency. Feedback from the modeled DRAM traffic multiplies the nominal cycle ratio by 1.0228. W7's deposited opend reference is 0.9558 of its calibration lazy, yielding a candidate/opend ratio of *point estimates* of 0.9522. The comparison uses different executables and has no paired candidate/opend interval. W1, W2 and W4 candidate readings are outside the reference range for dependent DRAM links. [Retained fixed-point reading](checkpoint/partial-prediction.json)

On W7, the sole sampled large dense block (ordinal 165) accounts for 7.42% of the combined candidate's corpus-weighted nominal cycles and 88.92% of its dependent DRAM loads, although it supplies 2.08% of the modeled bytes. Its raw count was 11,844 dependent DRAM loads. The other 63 sampled blocks together produced 1,385. LazyCells records a similar concentration at the same block. The elevated DRAM latency subsample has no large dense block. This gives a specific source of uncertainty in extrapolating model dependency or latency-sensitivity differences to the [wider edge gate](../../edge-budget/). [Weighted contributions](checkpoint/route-shares.json)

The conditional distribution explains why the large dependency counter need not imply a comparable speed effect. Of the official 400 W7 paired bootstrap draws, 177 omitted ordinal 165, 123 included it once and 100 included it at least twice. In those groups the candidate's median dependent DRAM links per million output bytes were respectively **34.8, 308.1 and 582.6**. The corresponding median modeled cycle ratios against LazyCells were **0.9060, 0.9160 and 0.9195**. The two arms share the dense fallback. These are strata of the same simulated bootstrap, with overlapping ratio ranges. [All conditional quantiles](checkpoint/W7-bootstrap-strata.csv) · [Recompute from the raw W7 observations](stratum_check.py)
