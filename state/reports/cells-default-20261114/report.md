# Public sequential Cells integration — PR382

Author: Codex; owner-directed pickup6868355345, concrete review request6868508410,
Coordinator reconciliation6868441092/6868553543. Implementation head
`a9c7716aaf664a7898f3d8f37c955e09d5204a23`, base
`6dec95ba6aaf59d5134e5ab119fd52433e2032ee`.

The minimal public type selections and reviewed kernel import are complete. No
merge/release/workstation action. Existing source correspondence, exact entry-point
map, safety argument, allocation bounds and seven-family i9 evidence are in the
PR's `docs/journal/2026-11-14-cells-default/INTEGRATION.md`.

## Completed local evidence

Rust1.100.0 (0ef1bc386, LLVM23.1.5), custom writable Cargo/Rustup homes.
No performance measurements locally. Correctness tests are untimed qualifications;
their harness elapsed times are not decoder performance evidence.

- `cargo fmt --all --check`: pass.
- `cargo clippy --all-targets --all-features -- -D warnings`: pass.
- `taskset -c 0-3 cargo test`: 109 library +70 integration +1 doctest passed;
  one existing census and the external-input test ignored in this command.
  Local four-CPU affinity matches the managed CPU quota and avoids exercising an
  unrelated five-CPU plan fixture. This is not an i9 binding.
- `cargo test --example ab`: all27 harness/machine/resource tests pass.
- All7 compressed files SHA-256 and byte sizes checked against the immutable
  #380 pins; `source-inputs.json` contains identities. Existing text generated from
  pinned57f51bbc0ca62ac3ee1cc148a1010842eea6222a, not new master.
- `CELLS_PINNED_INPUTS=/workspace/mibz2-selective/cells-corpora cargo test --test
  cells_default public_defaults_pinned_corpus_parity -- --ignored --nocapture`:
  all7 pass, including209,700,196 Wikidata decoded bytes. Full byte comparisons:
  libbzip2, explicit Segmented, public decode, public BzReader, scanned decode_blocks.
- `cargo +nightly miri test --lib -- miri_cells`: all6 pass, including F01 long
  writer reach and selective live-state/mark/15/16/17/reuse. Miri0.1.0 ac8310ba64.
- Existing selective mutation runner applied to an isolated copy of this exact
  head: positive baseline passes; all3 mutants compile and fail expected selected
  payload/descriptor assertions. `mutations/results.json` records the actual
  documented kernel hash1221a9a5f0993daee73d8352fc559b11557ebd4929f16de72bf8eeb50e5f0b58.
  These are live-state kills, not claims of separate output-oracle kills.

The mutation runner is copied alongside its raw logs. Reproduce with its `ROOT`
set to the checked-out PR head before calling `main(Path(fresh_destination))`;
Cargo/Rustup environment must select1.100.0. Existing #380 F01/invariant mutation
campaign remains independent prior evidence; no claim to have repeated it here.

Failures retained: the first precommit fixture incorrectly attempted Spec.clone;
Spec has no Clone implementation. Replaced with slice::from_ref before the only
production commit; subsequent full Clippy/native suite passes. First local Miri
attempt failed before tests because its default cache was read-only (raw
`miri-cache-readonly.log`). Setting `XDG_CACHE_HOME=/workspace/.mibz2-tools/cache`
uses the existing writable cache; no safety flags disabled.

## Pending at this checkpoint

New full-public-route Miri test, forced/default fuzz smoke, final exact-head CI and
ordinary paired PR benchmark are still running. Guardian integration checkpoint
161/6868538108 is independently routed; no completion inferred.

The ordinary benchmark's one-thread `mibz2` arm calls actual BzReader::new;
multiworker arms call unchanged BzReader::parallel. Its separate phases executable
still measures the existing Segmented construction/walk and is not a Cells phase
attribution. Do not interpret that diagnostic as the new default's stage costs.
Whole-reader wall/CPU/RSS decides. No new timing arm, filtering or rerun requested.

## Disposition

Reviewable implementation, not merge-qualified. Carry Researcher6857443323's
native B5 scope and request Guardian's caller/lifetime/fallback integration review.
Keep failed #380 ordinary4t evidence; no claim to resolve its cause from unchanged
parallel source alone. Queued three-build attribution was parked by Coordinator.

Current6775049636 default gate requires other corpora/thread counts level and RSS
within+10%; PERFORMANCE-POLICY also requires owner sign-off for secondary losses
over10%. The previously validated i9 Wiki benefit does not erase pattern+43.5%
wall, Wiki+31.1%RSS or pattern+15.1%RSS. A prospective, narrowly scoped adoption
decision is needed, not a retroactive PASS. Actual-head CI may add blockers.
Separate registered public-default target confirmation before release remains a
Coordinator/owner decision. No run/queue signal or release is part of this task.

Audit via fresh connector comments and unchanged coord.py: six resolved historic
misroutes, none unresolved. REVIEW6868508410 readback exact, misrouted=None.
coord.py verify hard-codes FROM coordinator and flags Codex publications; this
limitation is recorded, not reported as a passing verify or used to impersonate
Coordinator. GitHub comments do not launch a manually activated Codex session.

## Actual proposed-default CI result (first and only run)

Benchmark43534513477, artifact14089133218 (SHA-256
`61eef22374fb9218bb4aedba3bfbc630957e1d2abb7d27b4e4f5a5921a3a6302`).
Exact head/base above, Rust1.100.0, AMD EPYC7763 hosted runner,4CPUs/2physical
cores. All420 observations retained;10rounds ×7corpora ×3thread counts ×2builds.
Compressed identities and decoded hash/length parity checked. Existing arm_order
rotate/reverse schedule checked against source; initial local audit assumption
of strict alternation was rejected, not the data. Full raw artifact is archived.

| Corpus, one thread | Wall ratio [95% CI] | CPU ratio [95% CI] | Peak RSS ratio [95% CI] |
|---|---|---|---|
| wikidata | 0.9924 [0.9686, 1.0168] | 0.9923 [0.9687, 1.0165] | 1.3091 [1.3052, 1.3130] |
| text | 0.9899 [0.9669, 1.0135] | 0.9903 [0.9672, 1.0140] | 1.0001 [0.9965, 1.0037] |
| pattern | 1.5962 [1.5795, 1.6131] | 1.5601 [1.5444, 1.5760] | 1.1412 [1.1356, 1.1469] |
| onebyte | 1.0804 [1.0717, 1.0892] | 1.0735 [1.0625, 1.0846] | 1.0726 [1.0707, 1.0744] |
| random | 1.0014 [0.9964, 1.0064] | 1.0017 [0.9969, 1.0066] | 0.9996 [0.9978, 1.0014] |
| records | 0.9990 [0.9867, 1.0114] | 0.9988 [0.9871, 1.0107] | 0.9989 [0.9957, 1.0021] |
| runs | 1.0008 [0.9903, 1.0114] | 1.0033 [0.9903, 1.0164] | 0.9988 [0.9971, 1.0005] |

No primary wall regression established at any thread count. No parallel wall loss
excludes zero; text4t is −3.9% [−7.0,−0.6]. Its source is unchanged, so do not
attribute that gain to Cells induction. Prior380 failed4t result remains failed;
this new-head comparison is neutral there and does not explain the old cause.

Actual-default Wikidata1t is level (−0.8% [−3.1,+1.7]), not the earlier i9
−13.7% target result. Pattern1t+59.6% and onebyte1t+8.0% are additional current
costs. WikiRSS+30.9% and patternRSS+14.1% remain substantial. All other1t wall
intervals include zero. Full21cell table in ci/comment.md, raw numbers in result.json.

Text here has the standing CI source-derived identity acb5f8a27b49b265cd251fe3ea55f09c6229b991d55d655ee456178d386b6f5a,
shared by both measured builds. It is not the pinned i9 f0031809… text identity.
No corpus recipe was changed and the separate untimed pinned-i9 test passed.
Do not combine these studies or call their environments interchangeable.

The unchanged standard CI phases executable still exercises Segmented, not the
new calling-thread Cells route. Only whole-reader rows are used for the verdict.

Additional adoption blocker: onebyte lies in the5–10% secondary tier, but there
is no ≥16% primary gain at1t on this CI runner. Rule2.3 is not satisfied. Pattern
exceeds10%; the specific default gate also lacks Wikidata1t gain and violates
other-corpora-level and+10%RSS. No rerun requested: no primary regression needs
confirmation, and secondary retries are prohibited. Green execution is not a
performance PASS. A prospective adoption disposition must explicitly cover
these current results, not only the historical i9 pattern/RSS costs.

New public-route Miri passed (358.82s); all7 relevant local Miri tests now pass.
Forced Cells fuzz:28,369 cases/30s seed42, all production refusals and admitted/
periodic reach; forced refusals zero. Mixed/default fuzz:6,152 cases/30s seed42,
zero failures, including public decode and parallel candidate/error comparisons.
No new decoder defect found. Remaining CI: full Miri; independent Guardian
integration review still pending at this checkpoint.

## Independent review and Coordinator disposition

Guardian D1056868606509 / report13b01b9905f732821c0db3e5efaaa3f20e468d2a independently
PROCEEDs on caller/lifetime/error/fallback design and carries exact native B5.
No concrete new safety or decoding defect. Execution snapshot was PARTIAL; the
local completed ignored-corpus, public-route Miri and fuzz logs supplied above
resolve the author-side missing evidence. Coordinator can reconcile the raw logs
without a second full review. The exact-head CI merge tree739e56f99c9190aea515065d5f03ab45826b0fab
is byte-identical to source head (treef617d63498e248c39e133ef19da5ca3221b6e594).

Coordinator6868667961 accepts D105 source review and deliberately DEFERs adoption
of this current head under unchanged policy. No further implementation/review
project is commissioned while it fails adoption; D105's nonblocking range-error
and wrapper-selection test suggestions are explicitly retained for a later
targeted revision. The author does not silently fix/rebenchmark them now.

The narrow prospective decision must explicitly cover: pattern59.6%CI/43.5%i9
secondary loss; onebyte8.0%CI without the required twice-as-large1t primary gain;
WikiRSS30.9%CI/31.1%i9 and patternRSS14.1%CI/15.1%i9; and whether target-machine
actual-default qualification may supersede the existing universal/default rule.
It must preserve correctness, independent B5 and no-primary-loss requirements,
and cannot relabel either old380 or current382 gate failure as a historical pass.
The current owner instruction does not grant that exception. No claim of actual
public-default i9 gain before a separately registered/authorised confirmation.

All63 existing paired wall/CPU/RSS intervals independently re-evaluated with the
unchanged report helper and equal the archived result exactly; no rows filtered.

## Delivery checkpoint

Requested implementation and reviewable PR are complete at unchanged
a9c7716aaf664a7898f3d8f37c955e09d5204a23. No merge/release/workstation execution.
Coordinator6868667961's adoption DEFER remains. At the final snapshot, six
Actions workflows complete successfully, hosted legacy succeeds, full Miri
43534513415 remains running. Its unfinished log endpoint returned404; that is
not classified as an infrastructure/test failure. All7 scoped local Miri tests
passed. Do not call the full exact-head CI suite green before this final job ends.

Native/Python/platform/occupancy/benchmark/doc/walk checks and raw GitHub check
snapshots are archived in ci/. No repeats or new code head. The live PR body gives
both actual-default CI and historical i9 tradeoffs. Remaining work is deliberate
adoption disposition and final hosted-check reconciliation, not another automatic
implementation assignment. Future Codex work still requires a manual launch.
