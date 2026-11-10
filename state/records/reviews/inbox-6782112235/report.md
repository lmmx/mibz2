# Cells stage-to-decoder gap — work in progress

Inbox #160/6782112235; frozen implementation 57f51bbc0ca62ac3ee1cc148a1010842eea6222a. Scope: source/archive analysis only.

Both archive digests verified locally:
- i9 cells-vs-prod/20261110T175613Z at evidence d85b2074099f027d048aa7a057da10d25f2b70bb: 552e26aaf3b41636afdbd5a195ad5f0ce1c66aed945ec2911a508057baed705c.
- final CI artifact 13802703606: c36061ac6a7e8ac2991acbd82e2935fd20f9a3d075e102bd42c67b46cb575141.

Replayed raw CI paired wall geometric ratios: Wikidata 1.000093, pattern 2.231663, onebyte 1.109001. All output hashes match within corpus.

Replaying frozen density/floor/budget gates on the i9 census predicts 196 admitted blocks (176398965/209697662 rows, 84.1206%), 35 density refusals, no floor refusals, 2 budget refusals. This is a source-derived prediction, not observed CI admission. All 233 taking the records path did not mean all would take bounded Cells. Selecting the admitted blocks actually improves the idealised i9 inversion ratio to 0.773–0.782 before attempt costs, so coverage alone is not the cause.

The example measures independent per-block minima over repeated warmed calls, runs unbounded induce_cells::<false>(), and times records-to-output only. Frozen production validates, checks a budget each cell, retains bounded-layout scratch, and may pay attempt plus full fallback. Existing CI stages are a separate example; path equivalence must be checked before using them for Amdahl.

Remaining: finish source/cost accounting, quantify same-input/machine limitations, rank <=3 explanations, register one recommended diagnostic recipe without implementing or running it.
