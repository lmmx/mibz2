claim: explore
by: claude-code
outcome: done
evidence: origin/evidence f8b21e4 recalibrate-0518/run-20261128T100510Z (sha256 51fb951b…97f2, recipe b4d25f64 on 7be50513); docs/notebook/2026-11-28-explore-shipped-knee-worker-bound.md

**What was asked:** falsify the top performance premise ("scanner and workers co-limit at the 8-worker knee") against primary evidence on the current candidate, 7be50513.

**What I found:** it does not hold on the shipped build. The v0.5.18 recalibration bundle's frozen verdicts are VOID (harness bug: `two_arm.py` was called without `--ref`, and the unpinned corpus loop ran `--unpinned` as a corpus: 384 empty rows), but its raw rows are intact. Their per-thread CPU (scanner S, worker sum W) shows that Wikidata x4 is **worker-bound at every shipped i9 plan**:
- pinned 8T, 6/24: W/k − S = +78.5 ms;
- pinned 10T, 7/28: +39.6 ms;
- unpinned 10-20T, 8/32: +19 to +29 ms.
S is flat at 146-152 ms for 3 to 8 workers. W/k ≈ S only at 10 workers, where W inflates 17-20% (1,401 → 1,637-1,694 ms) and the wall does not improve (10/40 ÷ 8/32 = 0.999-1.036, every CI spanning 1). The "scanner-bound at 8W" readings (scanner-contention `a7ac01d8` and `1950f77f`, `e98948b6`'s traces) are about the `lanes` arm, `--features forward-anchor-lanes`; `Cargo.toml` has `default = []`.
- **A third term:** wall − max(S, W/k) = 20-25 ms at 8/32 and 34-45 ms at 9-10 workers. That is the known lifecycle term (surface `P`) plus window coupling. 9/27 costs 1.061 [1.038, 1.100] of 8/32 against 9/36's 0.994.
- **Off Wikidata:** text is set by neither station. S is 20-36 ms against a wall of 50-78 ms, and worker CPU grows 132 → 430 ms with the worker count, which is waiting, not demand. Random is scanner-bound from 4 workers.
- **For whoever reruns the recalibration** (a remark, not a verdict): the shipped pinned 10T plan, 7/28, takes 1.1016 [1.0991, 1.1032] of 8/32's Wikidata wall, while text takes 0.934 [0.898, 0.977]. That is a corpus trade-off.

**What changed:** the notebook note; a correction bullet in frontier/DECODER.md; claim `ab-station-ledger` at idea (a read-only reader that prints these columns for any bundle, so the binding station is a lookup rather than a synthesis).

**What's next:**
- On the default build, i9 Wikidata worker levers pay one-for-one at shipped plans up to the 19-40 ms gap; scanner levers pay there only after the lanes (or another ≥ 11% worker cut) are adopted. Scanner levers do pay on text and random.
- `release-0-5-18`'s holder must fix and rerun the VOID recalibration under its frozen rule. These rows are not a substitute.

**Caveats:** `thread_cpu` is CPU time in 10 ms ticks (means over 12 rounds), not traced busy time. It includes spin waits, as text shows. One bundle and one machine. The third term's split is borrowed from earlier dissections, not measured here. No new run, no src/ edit, no queue entry.
