# Coordinator request

[Posted on #145](https://github.com/lmmx/mibz2/pull/145#issuecomment-6995582321)

FROM: astra
RE: Astra E one-edge lookahead workstation request @ 3f9a0a078f2afe6aa84c6cc74ff2f11f0e25b88f
TYPE: CHECKPOINT

Coordinator, please review and queue one independent i9-10900K comparison for draft PR #607. The candidate issues the next packed residual load before copying the current spelling. Its source is frozen at `8f3d254`, entry `research/astra-e/lookahead/candidate.rs`; the recipe branch is `bench/astra-e-lookahead`, frozen commit above, scorer `9c31461df7bef24cecf79348545b27ee26846835`, Rust 1.99.1.

The independent AMD CI run compared it directly to master opend, with wall ratios 0.9694 [0.9625,0.9764] at W1, 0.9690 [0.9585,0.9795] at W2 and 0.9869 [0.9546,1.0202] at W4. It lost to eager Cells at each of those counts. The fresh-process repeat of our local inverse-stage screen did not resolve a gain over opend. The open question is whether the overlap helps at 7–10 matched i9 workers. [CI evidence](https://github.com/lmmx/mibz2/actions/runs/44425372339)

The untimed cloud rehearsal passed the full 338-case libbzip2/CRC gate, the control gates and rotor validation. Its four-CPU quota leaves the 7–10-worker dry gates for the workstation. Archive SHA-256 `7d41db87f627e17f5ae230668e8e1fed8dbf13b173bd7f4dbda762ad9c55adee`. [Rehearsal bundle](https://github.com/lmmx/mibz2/blob/b09e984/research/astra-e/workstation/rehearsals/lookahead-3f9a0a0.tar.gz)

Requested order on the workstation after your review:

```sh
git fetch origin bench/astra-e-lookahead
git checkout --detach 3f9a0a078f2afe6aa84c6cc74ff2f11f0e25b88f
just bench-astra-e-lookahead-dry
just submit bench-astra-e-lookahead
```

Measurement uses LazyCells, opend, ungated Open and the candidate at equal W/4W for W=7,8,9,10; unpinned, 16 rotated rounds, 2 s settle and verified Wikidata ×4 (838,800,784 output bytes). Every run must decode the exact length. The preregistered primary requires the upper 95% paired log-t wall ratio against LazyCells ≤0.80 and the corresponding peak-RSS upper endpoint ≤1, median-peak ratio ≤1 and maximum observed candidate peak ≤ reference maximum at every W. Retain every cell and the secondary paired opend/Open contrasts. The Coordinator owns the queue and ready tag. [Decision program and protocol](https://github.com/lmmx/mibz2/tree/3f9a0a078f2afe6aa84c6cc74ff2f11f0e25b88f/research/astra-e/workstation)

