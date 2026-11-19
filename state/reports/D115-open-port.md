# D115 — Open port: B5 sufficient to freeze the registration

2026-11-19. Incoming [7007091766](https://github.com/lmmx/mibz2/pull/161#issuecomment-7007091766).
Reviewed #630 at `3f09500ee3289b0dd06948003f45c89aedd8ea36`; #631 registration snapshot `b6412e1e43cbe7281c8f61033f7223f131244037`.
Operating SPEC/RULES/PROTOCOL/MODE pin: `a2da49638ac54d7763de1c167bd054f67ffdc763`.

## Decision

**G1 PROCEED for B5 readiness to freeze #631's registration at this evidence pin.** The source argument, differential and fuzz coverage, completed Miri, identifiable mutation failures and this independent review support that narrow classification. No new G2 safety/gate HOLD or G8 line STOP is established. No G3 rule revision is imposed.

This does not decide the registered C6 measurements, adoption, release or merge. Actual RSS and performance remain outcomes for the registration. The incoming silence rule was not used as evidence.

## Independent source review

The production lazy worker route now selects OpenCells, with the existing plans/windows/trims preserved. OpenCells uses open only for run-route inputs with `0 < n <= 900000`, `p < n`, and the shared record-count tests `20*r < 3*n` and `64*r >= n`. Otherwise it delegates to LazyCells, including the column route. These are the two count gates, not every later LazyCells floor/budget refusal. The boundary tests cover equality at both lines and entry refusal. Counts and entry limits keep these products within the production integer domain. The measured thin-run threshold 16 is retained.

No unsafe block is added. I examined open.rs, endpoint.rs, b5.rs, checks.rs, the admission helper, decoder/worker dispatch and candidate enumeration. Adjacent records and capped runs are normalized; the marked thick run is split so the primary row remains retained. Retained edges partition the retained ranks, and the wide-code path handles packed-code overflow. The source asserts positive band progress and complete terminal partitioning. Endpoint elimination removes a positive width from remaining rows, and newly joined rules reference earlier rules, giving a finite expansion DAG. The emit walk is bounded by retained edges, has an output-length break, clamps to n and fills a short period to n.

This is a valid-input construction argument supported by independent dense-LF arrow oracles and differential tests, not a claim that safe Rust alone proves byte correctness or freedom from resource failure. The arrow audit compares induced/traced and queued/unqueued construction, at multiple limits. Exhaustive small-column tests run every primary row; larger tests reuse kernels and cover long capped runs, gate boundaries and actual compressed-stream/worker decoding.

## Completed execution evidence

I read the actual job logs, rather than treating a green run label as proof.

- Miri [job 135496799257](https://github.com/lmmx/mibz2/actions/runs/44505754277/job/135496799257): the new `open::b5::miri_open_kernel_small_columns` test passed; 49 default and four feature tests passed. The hand-back's pending-CI statement is superseded by completion.
- Native [job 135496800465](https://github.com/lmmx/mibz2/actions/runs/44505754289/job/135496800465): 151 passed, zero failed, two ignored; open's nonignored tests, including the arrow audit, passed.
- Fuzz [job 135496799577](https://github.com/lmmx/mibz2/actions/runs/44505754289/job/135496799577): classic mode completed 85,696 cases, seed 1, 30 seconds, zero failures. Source candidate enumeration includes both OpenCells and ForcedOpenCells against the reference. The journal's additional 514,235 local cases are a reported result, not an independent rerun here.

CI checked out synthetic merge `124e91a1b407b7f589a48d2e7a595e742bdfd0db`. Its whole tree differs from #630's head: the comparison contains only seven Gemini journal files and PERFORMANCE-POLICY.md. No source, test, build or workflow file differs. I therefore use the completed jobs as validation of the reviewed executable source, without claiming whole-tree identity.

## Mutation evidence and reproduction

The final receipts record twelve built mutants, each with named runtime failing tests and a failed-test summary. They cover marked splitting/rank, byte offsets, destination/band width, repeat/output truncation, endpoint equality, memo copying, node rows and both admission inequalities. The baseline selects eleven tests but executes **ten**, with scratch_search ignored. Earlier 10/12 and 11/12 rounds and their SIGKILL outcomes remain historical; those kills were not scored as successful runtime detection.

I independently ran the scorer with substituted subprocess results. Baseline build failure, mutant build failure, test-selection dependency failure and zero executed tests cannot produce a passing mutation verdict; named runtime failures do. Every run restored the three source files. This tests scorer behavior with simulated outputs, not the Rust mutants themselves; their actual results are the inspected receipts. No Rust/Miri execution was performed in the Guardian runtime.

Reproduce the scorer check in an isolated directory containing [d115-substitution.py](d115-substitution.py), [d115-mutator.py](d115-mutator.py) and [d115-sources.json](d115-sources.json), using Python 3: `python d115-substitution.py`. The sources and mutator are snapshots from the reviewed pin. [Observed substitution results](d115-substitution.json).

## Nonblocking corrections and scope limits

1. The journal's `44n + O(sigma)` algebra bounds logical buffer elements. Vec growth and retained capacities across blocks, coexistence of open and lazy storage, and output capacity prevent treating it as a bound on retained allocation or whole-worker RSS. Seven fresh-shape tests are measurements, not a universal RSS proof; the 3,000-shape search is ignored in normal CI. Use #631's measured memory gates for adoption.
2. “Every loop ends” and “no input can ... exhaust memory” overstate what the emit guard alone proves. A spelling append happens before the length check/truncation, construction precedes emission, and correctness cannot be inferred solely from eventual CRC rejection. Positive construction progress, the finite valid-input DAG and tests support this review; I found no unmutated counterexample requiring a HOLD.
3. The Miri workflow selects the new test, but its path filter does not explicitly include `src/open.rs` or `src/open/**`. This PR did trigger and complete Miri through other changed paths. Adding those paths would protect future open-only changes; this is maintenance advice, not a missing result in this review.

The decision is scoped to the inspected source/evidence. No new count gate, infrastructure, benchmark run or authority expansion is introduced.

Evidence: [D115-evidence.json](D115-evidence.json), SHA-256 `f991f6a4792210a7a06813cacdfd9f4c98671374a5e660bae346793ab23e9852`.
