# Cells phase-split census verdict — inbox 6878327105

14 November 2026. Analysed Coordinator evidence at b5617a8efdd12256bfdc89e35232e05e4dbb6d32, directory docs/journal/2026-11-14-cells-census/, dispatched on #145 as QUEUED 6879190571 after scope resolution6878376365. The original brief remains the inbox assignment. Pinned decision rule: #145 LOG6878326831. No compilation, instrumentation, decoder execution or timing by Optimiser.

## Verdict: PARK the specified Cooperative Cells construction

Both predeclared conditions fail. For admitted Wikidata blocks, s = n-weighted (slots+induction)/(slots+induction+write) = **0.7416322548**, against required ≤0.40. Of196 admitted blocks, **8** have top-level-factor boundaries giving all three spans within ±10% of n/3; they account for4.0816% of admitted n. The rule does not say whether every block must pass, but neither an 8/196 count nor a4.08% byte share establishes general k=3 feasibility. The s failure alone is decisive.

With s treated as a *time* fraction only for illustration, Amdahl's k=3 ceiling is 1/[s+(1-s)/3]=1.208; even infinite writer workers cap at1/s=1.348. Both miss the rule's 1.67x target and the cited1.47x Teams lead. Here s is **instruction share, not measured time share**; neither number is a measured runtime bound. Induction's dependent pointer chain may price differently in cycles. Do not use instruction fraction to qualify or disqualify a new production implementation outside this registered diagnostic rule.

## Independent record calculations and source mapping

Parsed 431 Wikidata rows:196 admitted,185 density refusals,2 budget refusals,48 not_runs. The phase file has383 attempts; each admitted row matches its attempt, block, n and outcome. The196 admitted phase sums are slots790,102,938; induce1,155,202,206; write676,350,531 Ir. The pooled ratio is0.7420139733; block n-weighted mean is0.7416322548. These reproduce summary-wikidata.json. Each admitted row reports accounted=true. Text has49 density refusals, pattern88 sparse refusals, so neither contributes an admitted Cells block.

Among admitted rows, k3 within_10pct is true for8; admitted n fraction0.04081617;146 have largest top-level factor >1.1n/3. At best k3 cuts, supplied summary reports51.22% of later-span *bytes* sourced by earlier-span long copies. The 63.08% ratio obtained by dividing cross-span copy bytes by only cross+same long-copy bytes is a different denominator and must not be substituted. This dependence adds sequencing/communication cost, not a speed prediction.

Source inspection at b5617a8: `examples/cells_census.rs` emits the kernel's post-attempt `Census` (lines58–133), derives boundary cuts from top-level factors, and uses an exact feasibility check with binary search (136–190); `dependence` maps each copy's source interval against span starts (193–210). `src/cells.rs` defines the census grammar fields and outcome mapping (936–1050). This aligns the record labels with the claimed functions and split rule. I did not replay the 56MB uncommitted raw callgrind dumps, so the inclusive phase counts are not independently rederived from raw traces. README records a production-build cross-check: production pooled s0.744 versus census0.742 because writer code differs by ~1% Ir; both fail. Source and cross-check support the registered conclusion within this bounded evidence scope.

The file SHA256SUMS names the JSONL and production summary digests. GitHub file blob IDs and row consistency were checked; independent SHA256 verification of every payload against SHA256SUMS was not performed here. This is a provenance limit, not a reason to reverse the large predeclared gap.

## Next technical direction

Park *top-level split after one serial Cells induction*. The census identifies a different bottleneck for future research: slots+induction take ~74% of counted Cells instructions and the top-level factors are imbalanced. A construction that parallelises induction or changes the grammar granularity could attack the serial term, but is outside this decision and needs its own proof, safety and whole-decoder pricing. The feathered-induction relay6877865220 is therefore better aligned than adding more writer threads to this formulation; MONODROMY retains research ownership and Pathfinder architecture. No prototype, registration, merge or policy change is implied.

The exact census applies to admitted Wikidata blocks and this top-level split. It does not establish a maximum for all possible Cooperative Cells formulations or production time. The full decoder's scanner, MTF, CRC, fallback blocks and Teams scheduling remain unpriced by this census. Preserve the release, its own evidence, and the existing gate history separately.

Migration: legacy head e6eea5889432aef28f4d9e450cee87dd5bf7bacf and manifest SHA256 86d7a8a6ea42c2d2ade70c5ce3f0c82f375c0fd6291dbf452979bf07b100dbde rechecked. State only on team-state/optimizer. Coordinator carries artifacts to research/cells-census.
