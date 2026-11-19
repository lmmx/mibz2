# Combined narrow endpoint, immediate words and lookahead under scorer v1

The frozen build is `6a8a366cd786d58a528000b58da12fd2cfb47be8`. The candidate entry has SHA-256 `7b7741a017f12aaf5c9624028e32715ad4fd806210bef0c6b210c7c1d16631d6`, also recorded at the narrow-short workstation freeze. Master supplied scorer v1 at `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`. The four included candidate source hashes, model, executable, corpus and 64 extracted symbol tapes are in the [provenance receipt](checkpoint/provenance.json).

The quick libbzip2/CRC gate passed all 98 cases. The full 338-case gate for this candidate is separately retained with its [workstation rehearsal](../../workstation/rehearsals/narrow-short-0694800.tar.gz). The official proxy runs fresh LazyCells and candidate workers at nominal and elevated DRAM latency, comparing them with v1's deposited reference arms. Its sample has 48 full and 16 tiny blocks, weighted to the corpus populations. Each of the six worker models requires 160 simulator invocations. Each invocation warms and measures one block in a single worker. There is no process wall-time or RSS measurement in this score. [Gate receipts](checkpoint/check.jsonl) · [Invocation and environment](manifest.txt)

## In-progress recovery

The initial checkpoint preserves 240 complete official observations. It includes all 160 W1 rows and all 80 W2 LazyCells rows. A source check confirms that v1's `proxy-runs.jsonl` receives one record per block, with normal file buffering. The live job continues to append independently. The incomplete worker grid supplies no campaign ranking.

The live study runs in `/workspace/work/narrow-proxy/v1-tree` and writes `research/astra-e/results/combined-v1-full-proxy/score/proxy-runs.jsonl`. Its target directory is `/workspace/work/narrow-proxy/v1-target`. To inspect progress without consuming a CPU for simulation, count the complete lines in that file. The root process is the shell executing `/workspace/work/narrow-proxy/v1-run.sh`. If it survives, let the same process finish.

The following historical continuation uses Python 3.13.5 with `PYTHON_CPU_COUNT=1`, Rust 1.99.1, the saved build tree, Valgrind 3.24.0 and its development libraries. Run it only if the original simulator has stopped. The verifier matches the input, model and both executables. The continuation rechecks the gate, regenerates each symbol tape and lets the official scorer execute all missing jobs, model and report. Its only change to the cached runs is reusing the exact simulator response to the same arguments and verified tape. Choose a new output directory. A build whose tool or executable digest differs requires a fresh whole score.

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

For a fresh comparison the command below refreshes from master before scoring. In a frozen historical reproduction, first restore the scorer at the upstream revision above, then run the `moonshot.py score` command recorded in the manifest with that candidate, a fresh output directory and the same options. A changed upstream implementation begins a separate series.

```sh
nice -n 12 bash research/astra-e/score.sh combined-v1-repetition \
  research/astra-e/narrow-short/ahead.rs --target-dir /tmp/astra-combined-v1 \
  --quick --no-timing --proxy-blocks 48+16
```

## Dense blocks and the wider admission experiment

Five full corpus blocks exceed the combined candidate's run-density cutoff. The separate [edge-budget candidate](../../edge-budget/) admits all five. Their share of the canonical output is 2.15%. The official v1 sample includes one of them, stream ordinal 165, and it contributes 2.08% of corpus-weighted sample bytes. In contrast, the highest-latency runs used to estimate each arm's DRAM slope contain none of the five. That distinction matters when using this proxy to discuss policy changes restricted to dense inputs.

At nominal W1 ordinal 165 accounts for 2.93% of the combined arm's modeled cycles, 2.84% of instructions, 19.58% of DRAM read lines and 22.53% of DRAM lines including writes and final dirty drainage. Changing its route could influence traffic more than its work share suggests. These figures describe the combined candidate's existing costs. Buffer lifetime, between-block interference and the effect of another implementation at 7–10 workers require independent measurements. [Population and raw-counter decomposition](checkpoint/route-shares.json) · [Analysis program](route_shares.py)

```sh
python3 research/astra-e/results/combined-v1-proxy/route_shares.py \
  research/astra-e/results/combined-v1-proxy/checkpoint/proxy-runs.jsonl.gz
```

## The calibration's independent controls

The scorer's calibrated reference recovers all nine held-out pairs in its original high-worker comparison. The later 12-pair i9 opend sweep provides another check at equal concurrency. At W2 the v1 reference predicts opend/lazy 0.869 [0.838,0.900]; the i9 saw 0.950 [0.932,0.969]. At W4 the model predicts 0.867 [0.836,0.900], and the i9 saw 0.953 [0.939,0.967]. Its low-concurrency opend estimates miss by more than eight percentage points. At W7, W8 and W10 the respective model estimates are 0.956, 0.947 and 0.944, and their block-bootstrap intervals include the matched i9 observations of 0.934, 0.952 and 0.933. The W7 and W8 reference configurations also occur in the original calibration bundle. These are separate experimental sweeps on the same workstation. [Upstream v1 calibration](https://github.com/lmmx/mibz2/blob/604dac3f46569edd9ac9b6706b23a68d9d7d8f8d/bench/moonshot/calibration.json) · [Raw i9 sweep and pinned recipe](../../prior.md)
