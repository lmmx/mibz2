# #272 count-reconciliation closure

**PROCEED to freeze at ae62ea75a0e9e75a6d300264daffb99c6763e9f9.**
Inbox 6731216735; previous REVISE 6731167553 / inbox 6731077460.
Specs: docs/team/SPEC.md and SPEC.RESEARCHER.md, plus MODE/RULES/PROTOCOL,
at a4f97714cc14366984015869d9d094d89488ac57.

The remaining Researcher objection is resolved. This recommends freezing the diagnostic
registration; it does not name a run, dispatch it, approve integration or claim a speedup.

## Evidence and reproduction

Read the candidate validator, its tests, the harness, journal amendment, actual Wikidata spec
and bwt threshold. Inspected the complete GitHub comparison from the previously reviewed
4c70dfc42536c32e9b55c6f2e95da5b1d8538b39 to this head: validator, fixture tests, journal,
and the pair test counter's coverage-label correction. No measured harness/kernel change
appears in that comparison.

- **39 repository fixture tests passed.**
- **Independent replay:** the four previous false-VALID mutations all become NOT VALID,
  with no registered readings: child 400M rows, single-copy 20B rows/M1 flip, negative paired
  blocks in every copy, and one-byte child output.
- Added domain-valid 199M-row mutations so the domain bounds cannot mask an absent
  reconciliation check. A child-only change and a common-mode CSV-only change both fail.
  Paired blocks with zero rows and duplicate census lines also fail.
- The unchanged control remains VALID. A deliberately consistent CSV-and-log row change
  remains VALID; this establishes the limitation below, not a reason to reopen the objection.
- AST comparison shows `ratio`, `diff`, `readings`, `check_plan` and
  `check_inventory` unchanged from the previous head. Thus the numerical decision rules,
  interval method and earlier clock/source/inventory checks are preserved.

Reproduce from this directory:
`python3 -m unittest discover -s bench -p test_pair_runs_validity.py -v`,
then `python3 probes.py`. The archived files are research copies, not implementation edits.

## Why the repair closes the finding

`validate` now retains every parsed copy's metadata and calls `check_counts` before
`readings`. The latter compares each file with the first copy, then with that copy's logged
census. The domain rejects negative counts, inconsistent zero/nonzero pairs and paired-row
totals outside paired_blocks × [65,536, 900,000]. The lower bound matches
`bwt::MIN_SEGMENTED`.

The harness prints the exact prefixes matched by CENSUS and IDENTITY
(`examples/pair_runs.rs`, lines 382 and 395). Paired blocks and rows print as integers;
output MB prints with one decimal place, so its reconciliation correctly allows rounding.
Wikidata output is additionally checked exactly against the spec, rather than inferred from MB.

The actual spec has **11,854,933 compressed bytes**, **209,700,196 decoded bytes**, and
**431 total blocks** summed over its windows. The code uses 431 for total logged blocks,
not for the paired subset. Each copy's compressed/decoded identity is checked against that
spec. This is the distinction the previous review required.

The two adjusted old tests remain meaningful: a legitimate unpaired synthetic file changes
both CSV and log and retains zero build times; zero paired rows with positive paired blocks
now fail earlier, while a direct `readings` assertion still tests the missing-M1 safeguard.
No coverage obligation was removed merely to make tests pass.

## Limits: state these narrowly, without another freeze gate

The validator establishes consistency, domains and selected independent corpus facts; it
does not independently re-decode the corpus or prove every census number. In particular,
the pinned spec does **not** pin Wikidata's paired-row denominator, any more than it pins
the synthetic paired-row denominators. Changing Wikidata's paired rows from 200M to 199M
in every CSV and corresponding log, while remaining within bounds, still passes the fixture.
The opening claim that no denominator can differ “from the pinned spec” therefore needs
this qualification: output bytes and total blocks are pinned; paired rows are reconciled
with the source-bound census. This consistent-fabrication scenario is outside the previous
CSV/log-disagreement objection. I do not require another decoder or new corpus census
experiment to freeze this registration.

For synthetics, the log's MB field constrains output within ±50,000 bytes, with a tiny
floating-point tolerance; it is not byte-exact independent validation. The harness's CRC
and output checks remain their own evidence. Print-format compatibility was inspected in
source and exercised with fixtures, not a real dispatch.

## Safety, expectations and next step

Guardian D052, comment 6731230049, closes the D046/D051 timing safety-evidence HOLD at
80637086691f707a86adce94d1a0430c4c0b009a after retrieving completed CI Miri evidence and
the corrected presized coverage label. This review does not redo or expand that B5 decision.
I proposed an earlier construction and am not an uninvolved safety sign-off.
No Rust, Miri, fuzz campaign, decoder or timing was executed here.

Expected: the prior denominator/inventory contradiction closed. Observed: all four old
witnesses rejected, stronger within-domain reconciliation checks passed, 39 fixtures green.
**No performance gap has yet closed.** The next bounded registered run still has information
value: Q1 tests unconditional builder savings; Q2 tests the admitted inverse-BWT hybrid.
It does not itself qualify whole-decoder production performance or identify the consumer's
critical-path bottleneck.

Coordinator may freeze this exact reviewed head under C1. Normal CI and the frozen run
identity remain separate; no timing, dispatch, merge or authority expansion is performed here.
