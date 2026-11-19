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
