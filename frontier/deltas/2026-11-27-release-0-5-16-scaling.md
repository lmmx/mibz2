claim: release-0-5-16
by: claude-code
outcome: partial
evidence: origin/evidence 4710ef99 (release-scaling/run-20261127T170116Z.tar.gz, sha256 13998ef2d07bbd6be34973a03594172ceebc403d86cd374536ecdbb13a86ab8a); bench/release-scaling/2026-11-27-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.16/

**What was asked:** RELEASING §5.2 for 0.5.16: record the i9 release-scaling bundle.
**What changed:**
- Verified the bundle's SHA-256 bit-for-bit against the evidence commit; meta names arms 0.5.16 (`mibz2`) and 0.5.15, i9-10900K, 10 rounds, threads 1/2/4/8/10/12/16/20, Wikidata 210 MB; 240/240 runs `termination: success`, same bytes everywhere.
- Copied `recorded/` (env.json, meta.json, runs.jsonl) to `bench/release-scaling/2026-11-27-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.16/`.
- **Verdict:** 0.5.15/0.5.16 wall spans 1 at every count (pinned 1.012, 1.003, 0.994, 1.005, 1.007 at 1/2/4/8/10; unpinned 0.995, 0.984, 1.025 at 12/16/20). Unchanged on Wikidata, as predicted: #1175 moves no Wikidata block. §5b: no row. Recorded in CHANGELOG 0.5.16.
**Remaining (claim stays `open`):** `bench-crossover` at 4e261140 is queued (`ready/bench-crossover` confirmed); record its §5a verdict in CHANGELOG when its bundle lands, check the live History page shows "0.5.16 against 0.5.15", bump `lmmx/wikidata-pq`, mark the claim done. Do not re-signal or re-queue.
