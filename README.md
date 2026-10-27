# Benchmark results

Data only, never merged: master's benchmark results, one per commit the Benchmark workflow
ran on (`.github/workflows/bench.yml`, `bench/report.py record`). Pull requests' runs are
not recorded.

- `records.jsonl`: a line a commit: its SHA, date, bench version, runner CPU and cores, rounds,
  corpora and the run's URL.
- `runs/<commit>.json`: its result (`bench/report.py summary`): every figure, the machine,
  the toolchain and the corpora's sizes and SHA-256.
- `raw/<commit>.jsonl`: the harness's records (`ab run`), one line a run.

Results compare only within one major bench version, one runner CPU model and one set of corpora.
