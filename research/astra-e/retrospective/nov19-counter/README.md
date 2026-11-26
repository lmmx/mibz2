# Forecasts after the five-arm cap-counter workstation run

At data commit 9f42c534 the refreshed evaluator read 166 local forecasts and 67 independent worker-count results. The four new target results are combined/Lazy wall 0.866, 0.882, 0.883 and 0.868 at W7–10. None had a local forecast at the same W for the candidate. The scorer's v1 target proxy calibration therefore still contains three cells, with mean absolute log error 0.015 and zero within-worker ranking pairs. Across other hosts it reports 64 v1 local wall comparisons, mean absolute log error 0.152 and 144/240 concordant pairs. That rating uses different machines and worker counts from the target decision.

The original measured five-arm incremental comparison against the verified scalar worker is preserved independently of the evaluator's Lazy-relative table. Combined/scalar pooled wall was 0.99980 [0.99452,1.00512]. [Raw evaluator output](official.txt) · [Version, data cutoff and replay command](source.json) · [Independent paired target replay](../../workstation/counter-independent-i9.md)

To reproduce, check out the manifest's data commit, project bench/moonshot, .just and Justfile from its master commit and execute the recorded argv at the repository root. To start a new scoring round, refresh the scorer from current master first.
