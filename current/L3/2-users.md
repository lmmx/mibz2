# 2. Deliver the current decoder to downstream users

- **Doing:** updating wikidata-pq from mibz2 0.2.2 to 0.2.5, superseding this PR's earlier 0.2.4 target.
- **In order to:** 0, deliver released decoder improvements through its real consumer.
- **Status:** [wikidata-pq #219](https://github.com/lmmx/wikidata-pq/pull/219) open at 93240f94d13bd15cd1c261525725596f2a45cc11; [Benchmark 42313607695](https://github.com/lmmx/wikidata-pq/actions/runs/42313607695) in progress when inspected. No new verdict inferred.
- **Earlier result, scoped to 66544d5498a52f5f81256e0f350380b079c39166:** [6672648270](https://github.com/lmmx/wikidata-pq/pull/219#issuecomment-6672648270) reports no end-to-end gain beyond noise: split estimate 1h35m00s→1h33m49s and release estimate 5h05m50s→5h04m45s; parity passed, 68 native split files byte-identical to lbzip2. Peak memory across processes rose 1.303→1.433 GB, +9.8% ±3.4%. Those are modelled completion estimates, not timed full-dump results.
- **Disposition of old revision:** left open for maintainer's call because the project's merge rule requires end-to-end gain beyond noise. It was not waiting on mibz2's release.
- **New premise:** 0.2.5 includes the earlier move-to-front improvement plus the ten-core policy; the latter cannot apply on this project's four-core runners. PR expects only the small earlier effect there, below the old three-round precision. This is an expectation, not the new result.
- **Dependency:** the new revision's own benchmark/merge disposition. A successful i9 decoder experiment neither proves end-to-end pipeline gain nor overrides the downstream rule; no automatic merge authority inferred.
