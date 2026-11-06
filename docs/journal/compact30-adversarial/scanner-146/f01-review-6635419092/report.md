# S62-F01 counter-coverage reassessment

FROM: researcher
TO: coordinator, guardian, optimizer
RE: checkpoint 6635419092
TYPE: REVIEW

**The mapped primary-corpus cost changes are now counter-supported.** I found no missing counter or incorrect predicate among the 31 scanner and 34 descriptor classes in the submitted ledgers. I recommend accepting the counter remedy for those classes. This is a targeted adversarial reassessment, not a new blind §6.2 review or permission to integrate.

The stronger phrase “every changed path” needs a scope qualification: the replay enumerates nonzero differences in frequency histograms, not every changed instruction site. The package does not mechanically prove universal path coverage. This is an assurance limitation, not a demonstrated missing cost or grounds to invent another numerical deduction.

## Pins and independent work

- Scanner: ba261eff245f45e1c07fdfb05fa2fe6ff1ced835; predecessor 49cb46b65070059af49e906fa8d98693b808bda2.
- Descriptors: 7620c0c667018772a0ba3be28eb62263b55037f8; predecessor 29dd071f8787a28560a7a081fd7ac0ef13991466.
- Source: examples/c30walk/ctor2su.rs and ctor2d.rs.
- Evidence: docs/journal/2026-11-04-compact30-scanner-f01/ and descriptor-f01/.
- Roles read at 09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47 (the role file was absent at the implementation/own branch paths tried).

I read the source additions against their predecessors, the counter records, complete class tables and mapping code, build listings, identity tool and archived identity/control reports. The adjacent audit.py independently evaluates the counter expressions using an AST restricted to integer arithmetic, compares the separately archived primary record with the ledger's embedded record, checks all class products and interval arithmetic, and locates increments for all 20/27 new fields. Its mutation check changes bitmap_memsets without changing tails: both ledgers detect the disagreement rather than accepting the coincident frequency.

No constructor was compiled or executed, no suites were rerun, and no timing was performed. I did not replay the compressed raw profiles in this reassessment or reproduce the binary builds. The class inventory and per-site profile frequencies remain inputs from the pinned ledger; this check must not be represented as an independent raw-profile census. The existing independent review supplied that earlier profile analysis.

Run audit.py with a repository containing the pinned commits: python3 audit.py /path/to/mibz2. inputs.json lists all exact git objects read; audit.json is the result from the downloaded text inputs. The script also writes source diffs for inspection.

## Coverage findings

| Check | Scanner | Descriptors |
|---|---:|---:|
| New u64 counter fields | 20 | 27 |
| Nonzero ledger classes | 31 | 34 |
| Individually checked expressions | 32 | 38 |
| Mismatching expressions | 0 | 0 |
| Supported own-code delta | −84,060,421 | −58,209,045 |

The counter predicates address the actual branches, not just the numerical coincidences:

- Scanner refill counters inspect write==end immediately before one(), separately for full-tile first/second segments and tail first/second segments (source lines 707, 715, 761, 773). Tail length and parity predicates sit at tail entry. The full-tile path is separately instrumented; the len==16 branch inside the checked tail helper is unreachable under its counted assertion and is not used as the full-tile measurement.
- Run lookahead/no-lookahead and exhausted cases use after.len() and held state at the relevant boundaries (1067–1085). Epoch tiled/short and untailed predicates are distinct. The final-finishes counter uses the same nonempty predicate as close (1118).
- Descriptor second-segment row/refill and tail counters are separate (714–727). Subtracting full-tile second rows and all tail rows from rows_scalar yields the first-segment class; it does not subtract the tails twice.
- Descriptor run-ending counters are immediately before the two successful return paths (834, 847), with run_opens immediately before open (854). fill/vector and complete entry counts use the relevant calls.
- Allocation counters test capacities before replacement/reserve (893–919). The sequential harness keeps one constructor and output pair across the ordered blocks, matching the lifetime model the capacity predicates require. Matching output digest alone would not establish this; the source structure and the separately recorded six capacity-sensitive counts matter.
- Init/sweep counters count actual loop iterations (932, 1078); their division by four/two is a fixed machine-loop trip relation, not a claim of four/two source iterations. Epoch-derived snapshot counts likewise require the fixed 256-entry source loop and pinned unrolling.
- The frequency-230 tail and bitmap paths are counted independently in both prototypes. Descriptor frequency 2 has rec_grows and bitmap_reserves; frequency 1 has rec_grow_frees, snaps_reserves and bitmap_reserves_single. All expressions are checked individually, so equal observed totals do not collapse distinct predicates.

All added operational increments in the inspected source diffs are under COUNT guards; the extra helper reads are called only in those guarded blocks. Serialization, aggregation, consistency assertions and the sequential harness explain the other additions.

## Two qualifications worth retaining

**Coverage is not established by histogram equality alone.** f01_replay.py classes() discards a frequency when baseline and candidate have equal instruction counts. A removed instruction on one path and an added instruction on another equally frequent path can therefore disappear from the class inventory. Also CLASSES is a manually curated list: all listed predicates agreeing does not prove that an unlisted disjoint region at the same frequency has the same meaning. Inclusive-total closure protects the observed arithmetic, but is not a path-coverage theorem for another corpus or control-flow change. Keep the source/listing interpretation with these pins. A future replay should enumerate changed sites before netting and preserve zero-net groups; no current numerical failure has been demonstrated here.

**“Byte-identical” overstates the supplied identity result.** asmid.py drops addresses, normalizes branches/calls and masks RIP displacements, then compares normalized instructions and sizes. Its archived 52-function reports pass and the counted-build controls differ, as intended. The README gives different whole-binary hashes for the predecessor and instrumented binary; rebuilding the predecessor to its archived hash is a separate check. Use “normalized uncounted instruction closure unchanged” unless actual byte ranges are compared. This wording correction does not undermine the instruction-count argument.

## Numerical consequence and recommendation

Independent arithmetic reproduces scanner [84,060,421, 84,060,421] and descriptor [57,830,163, 62,277,795]. For descriptors, 378,882 added library instructions are charged and the 4,447,632 removed memcpy contribution is withheld at the lower end. Those library costs are archived-profile inputs; the 434 call count is supported by the successful-build count, not a newly measured rep-movsb instruction convention.

With the submitted +868 caller charge, the intervals remain [84,059,553, 84,059,553] and [57,829,295, 62,276,927], above their isolated minima. The caller explanation was not independently re-profiled here.

Accept S62-F01's mapped-path counter repair with these explicit evidence limits; do not retain the former blanket claim that the ledger frequencies are profile-only. Do not infer a new universal coverage certificate from this acceptance. S62-F02 overlap and combined-build accounting remain separate, and neither this review nor the isolated arithmetic authorizes integration.

Delivery 6635476486 was fetched separately during this work and skipped because its TO line named coordinator and optimizer, not researcher. No action was taken on that delivery.
