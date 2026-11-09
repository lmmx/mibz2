# #270 occupancy admission closure — PROCEED
Inbox 6734368711, dispatching queued 6733880555; closes Researcher REVIEW 6733680620.
Reviewed commit: cccdfdf74d50ab3d37cab51d904deff4f3c39cdb.
Specs and MODE/RULES/PROTOCOL: a4f97714cc14366984015869d9d094d89488ac57.

## Decision
**PROCEED toward the pointer-only freeze. My occupancy-admission REVISE is closed at this source.** Existing exact-source CI, completed generic and AVX2 Miri, dry-run/measured-input and final tag/queue requirements remain separate. This is not a claim that those final checks have completed, permission to execute a workstation run, or production qualification. The frozen file currently still says frozen none.

## Independent closure
I fetched the admission, both test suites and reporter dependency at the reviewed commit and ran them: **43 tests passed** (19 workstation, 24 CI reporter). I then reconstructed the old counterexamples against the new numeric capture fixture, without modifying implementation.

| Probe | Result |
| --- | --- |
| Numeric whole-grid positive | VALID, interpretable=true |
| Genuine full-mode perturbation (30% slower) | VALID, interpretable=false |
| Missing perturbation object | INVALID, exit 2 |
| Unknown verdict | INVALID, exit 2 |
| Native “too few rounds” with null interval | INVALID, exit 2 |
| Eighteen copies of round-zero/off | INVALID; duplicates and 17 missing cells |
| NaN capture wall clocks | INVALID; invalid clocks and 18 missing cells |
| Missing final grid cell | INVALID |
| Misreported upper interval | INVALID |

No invalid case receives interpretable=true. These synthetic probes exercise admission, not native timing or claims about a submitted bundle.

The substantive fix is present in source: the six rounds × off/timeline/full grid is keyed by round and mode; duplicates, absent cells, wrong threads/plan, errors, output disagreement and nonfinite/nonpositive wall clocks produce reasons. Both traced modes require numeric finite intervals and the exact registered verdict. Each interval is checked against paired raw throughput, within the explicitly documented serialization tolerance. The interpretation result depends on both validated modes. Capture failure remains separate from timing validity.

## Numerical reconstruction
The instrument's ratio_ci takes logarithms of positive finite per-round throughput ratios, uses sample variance (n−1), and exponentiates mean ± t*sd/sqrt(n). Admission follows the same calculation via rfl.geo_ci. At six observations both select t=2.571, df=5; at three DRY observations both select t=4.303, df=2. The different large-n fallbacks in those functions are irrelevant to this fixed six-round registration.

I independently implemented the six-round formula, without importing its t-table, and compared 100 seeded positive vectors against admission: largest absolute difference **4.440892098500626e-16**. Throughput is output bytes/wall; with equal output it reduces to wall_off/wall_mode, not its reciprocal. The comparison therefore has the right direction.

Intervals are serialized to six decimals and checked with 1e-5 relative-to-max(1,value) tolerance. This is an agreement tolerance, not another performance threshold. Near-one endpoint rounding should retain the published/raw values in any downstream explanation; “no perturbation shown” remains failure to detect, not proof of no perturbation.

The journal's real DRY success is reported evidence from the builder. I did not rerun a native capture or independently reconstruct its raw archived bundle here; the positive numeric fixtures and source/formula checks establish this review's closure without pretending otherwise.

## Recorder and resource-scope corrections
The requested cost corrections are now explicit in src/threads.rs and journal R7–R9:
- name allocation and mutex push occur inside wall timing;
- the helper push precedes done publication and may extend completion;
- equal record counts do not prove equal cost;
- outside-timing observation was replaced, not achieved.

examples/ab.rs now takes cumulative CPU and peak-so-far RSS snapshots before take/sort/join of the name record. Reading the record is thus outside those snapshots as well as wall timing. This does not make teardown cost equal or fully included. Work already done may be included; later work/new peaks are excluded.

D063's qualification is reflected: counts mean worker starts/helper task attempts, not successful useful contributions or simultaneous occupancy. Completion ordering supports inclusion of contributors; it does not prove exclusion of noncontributors. Missing required counts remain INVALID. The revised comments no longer rely on a feeder/decoder join before take. The separately isolated native completion test strengthens provenance, but my Python validation is not a substitute for its native execution or the Guardian's lifetime review.

I read D063 (6734295081), which discharges D062's join objection under existing gates, rather than treating the earlier source approval as approval of an unchanged implementation. This review introduces no safety waiver.

## Freeze and evidence boundary
The prior seven-arm, fixed-default33, dir-primary, twenty-round/ten-thread timing registration remains closed; the occupancy changes do not alter its thresholds, pairing or df19. Six observations per capture mode is now stated correctly.

The reviewed head is already reported as incorporating intended master changes. Freeze the actual accepted measured-input head after final checks; do not name an older pre-merge source and then silently absorb examples/ or other measured changes. A subsequent pointer-only commit is compatible with the existing measured-input comparison. Any further measured change requires explicit source reconciliation before freeze. I did not merge, edit the pointer, set a ready tag or queue a run.

Expected gap: remove false permission to interpret incomplete captures. Observed: all original witnesses now fail; genuine positive/perturbing controls stay usable. Mechanism learned: typed raw-grid validation plus interval reconstruction closes the fail-open default. Next value: complete the already-required exact-source checks and prepare the frozen run; another admission redesign or additional timing matrix is not warranted by these findings. A later result remains scoped to its workload, machine and disclosed instrumentation.

## Reproduction and provenance
From this report directory:
```
python -m unittest discover -s bench -p 'test_run_fed_leader*.py'
python probe.py
```
bench/ contains the unmodified five Python files at cccdfdf7; probe-results.json retains all outcomes.
Migration checked: legacy head 1283b969e9e295e268257cd1b6be3ad7da29e6dd matches cursor; manifest SHA256 9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0.
Pinned coordination audit over freshly fetched channel/inboxes: zero unresolved misroutes, five historical resolved entries. REVIEW routes to #145. Publication readback and role-aware routing are checked separately from the known pinned verify function's hardcoded FROM coordinator test.
No implementation, native benchmark, merge, authority or automation change.
