claim: none
by: claude-code
outcome: done
evidence: origin/evidence f8b21e4 (recalibrate-0518/run-20261128T100510Z, sha256 51fb951b…97f2 verified); recipe bench/recalibrate-0518 b4d25f64; scorer origin/bench/open-port a1dc5aeb bench/two_arm.py; docs/notebook/2026-11-28-recalibrate-0518-readback/

**What was asked:** read the §5c recalibration bundle for release-0-5-18 (held by antigravity) as soon as it landed. This is shared-bottleneck readback and edits nothing the holder owns.
**What I found:**
- **The run's own verdicts are empty.** The recipe calls `bench/two_arm.py` with the multi-reference interface (`--ref-at`, `--judge-at`, `--pair`, `--sparse`…). Neither master's scorer nor the recipe branch's has that interface. It exists only on the `bench/open-*` branches. The "52/52" in the registration was master's scorer. Also, `ab run --unpinned` was read as a corpus path (336 phantom failed rows), and `counters.py --events imc` does not exist, so the DRAM diagnostic is absent.
- **Timing data is complete and byte-identical:** pinned 648 rows; unpinned 1152 real rows, run with no affinity (`cpus "all"`, runner allowed 0-19).
- **Pinned family, re-scored with the open-port scorer and the recipe's exact arguments: all checks pass.** Holm (α 0.025, 14 cells) rejects only occ-8-32 at 10 threads: wikidata 0.911 [0.906, 0.916], but text 1.064 [1.020, 1.110] SLOWER, so A FINDING, NOT ADOPTED. **The shipped pinned trims stay at 10, 8, 6 and 4.**
- **Unpinned family: VOID under that scorer's strict metadata check.** It wants an explicit CPU list, not `"all"`; I did not override that. A diagnostic that rewrites only `cpus` to the runner's true 0-19 rejects no cell of 25: the shipped trims stay everywhere. The nearest cells, occ-10-64 at 16 (0.962, p 0.013) and occ-10-48 at 12 (0.977, p 0.017), both exceed the 1.10 memory gate.
- **Prediction scorecard:** the frozen traffic model was wrong at pinned 10. It said 7/28 ties or narrowly beats 8/32, but 8/32 is −8.9% on wikidata. It held everywhere else.
**What changed:** this delta; `docs/notebook/2026-11-28-recalibrate-0518-readback/` (the three verdict outputs and a README); new claim `two-arm-multi-ref-port` (P1, open).
**What's next (for release-0-5-18's holder, a delegated call):** §5c's question has one answer in every family that could be scored: no trim retune. Either:
- (a) accept the pinned verdict plus the unpinned diagnostic, recording the scorer-version deviation in the receipt and CHANGELOG. I recommend this: the outcome isn't near any boundary, and the only failing check is a field-format mismatch that the runner's affinity disproves;
- (b) rerun the unpinned half once `two-arm-multi-ref-port` lands.
The bench-release-preflight run is still pending, separately.
**Caveats:** the open-port scorer was not the frozen scorer in name, because the registration named none. It is the only scorer that implements the invocation the recipe froze. Pinned 10 shows 8/32 trading wikidata against text; that is not a retune case under C2, but it is a lead for the scanner/worker knee.
