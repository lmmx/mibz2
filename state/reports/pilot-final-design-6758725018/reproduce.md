# Reproduction

This directory is a research calculation, not bench or decoder implementation.

- `menu.json`: full mean-free output of `bench/tg_ni_pilot.py` at 9c6a6b070673c3773e7e52da2e428a904e09adfd, from pilot run 42748681243. Source admission passed. Regenerated Markdown's digest is in evidence.json and agrees with #316 comment 6758585874.
- `power.py`: unchanged copy of `bench/service_tg_power.py` at 30a6620d0a8733536284e82c91ff19be0185f148, used for its quadrature function only. The history CLI is not used.
- `analyse_design.py`: recomputes the planning table, Gaussian batch sensitivity simulations, trend cutoff, budgets and hypothetical harm-flag example. Python 3.12, scipy 1.17.0 and numpy were used. Run `python analyse_design.py`; results go to `design-results.json`.
- To reproduce the waiting-field diagnostics too, download the ten artifacts identified by evidence.json from GitHub Actions run 42748681243; verify each ZIP digest before extraction; extract to `artifacts/<artifact name>/`. The script then reads `*/runs.jsonl`. With no artifacts present, its `waits` object is empty; the power/model calculations still reproduce from menu.json.
- No candidate mean, overall time ratio, qualification bound or verdict is printed by this research. Raw runs are used only by the registered mean-free menu and the stated waiting-field diagnostic. No decoder is built or run.

The pilot menu replay additionally used `bench/service_tg.py` and `bench/svc_target_check.py` at 30a6620d0a8733536284e82c91ff19be0185f148. Call `tg_ni_pilot.analyse` with the aggregate manifest, head/base from evidence.json, rustc `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, the text shard's recorded corpus SHA-256, label pilot and R600. Do not invoke the qualification judge to interpret pilot outcomes.

Numerical powers and the family veto budget are conditional model calculations. Sensitivity models neither estimate CI's batch covariance nor validate nominal coverage.
