# Benchmark results

Data only, never merged: master's benchmark results, and each merged pull request's own
paired benchmark (`.github/workflows/bench.yml`, `bench/report.py record`,
`bench/links.py`).

- `records.jsonl`: a line a master commit the Benchmark workflow ran on: its SHA, date,
  bench version, runner CPU and cores, rounds, corpora and the run's URL.
- `runs/<commit>.json`: its result (`bench/report.py summary`): every figure, the machine,
  the toolchain and the corpora's sizes and SHA-256.
- `raw/<commit>.jsonl`: the harness's records (`ab run`), one line a run.
- `links.jsonl`: a line a merge commit whose pull request's last benchmark run (head
  against base, on one runner) is kept (`bench/links.py`): the pull request, its head and
  base, the run, the runner, and per corpus and thread count the rounds' log ratios, head
  over base, of wall, CPU and peak RSS.
- `pr/<commit>.json` and `pr-raw/<commit>.jsonl`: that run's result and records, as they
  were.

A master record's figure compares only within the major bench version that last set its
meaning (MEANING in bench/report.py, bench/CHANGELOG.md), one runner CPU model and one set
of corpora. A link is a paired change, on its own runner: links chain into a cumulative
change, on the assumption that a relative change holds across runners.
