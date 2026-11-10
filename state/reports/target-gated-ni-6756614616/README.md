# Reproduction

Independent review of inbox 6756614616, PR316 head eb46b5a80bc1e73f226ebb22a31ef04d3b869f49.

The six service/test Python files are unchanged copies from that head, archived as research fixtures only. links.jsonl is the fetched bench-results history at f99e2f358ba7d0885b8129e1dc7f60e59f30a39a. probes.py is independent review code. probe-results.json records its output.

Environment: Python 3; NumPy 2.3.5; SciPy 1.17.0.

From this directory:

```sh
python -m unittest discover -s . -p test_service_tg.py
python probes.py
python service_tg_power.py links.jsonl --rounds 600 900 960
```

The first command passed 16 tests. The second uses synthetic fixtures, exact Gaussian batch covariance and numerical integration. It performs no decoder timings; the AR(1) output is simulated, not observed CI. Git ancestry checks alone are disabled for the synthetic admission probes. The third reproduces the supplied power calculator; inspect non-finite values rather than interpreting its NaN-to-zero lower-bound presentation as evidence.

No production implementation or workflow is changed. See report.md for assumptions, findings, limitations and sources.
