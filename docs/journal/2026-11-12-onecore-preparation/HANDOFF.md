# #366 preparation: reviewed repair incorporated

Assignment [363/6810684246](https://github.com/lmmx/mibz2/issues/363#issuecomment-6810684246)
restores the implementation scope of6791014104 after its completed pause handoff.
This is actual incorporation and untimed validation, not workstation readiness.
Master/policy remains2635f04e9155dd31388b24fd3f7248f87b3ddcda.

## Appended history and exact source correspondence

Re-read/fetched existing branch `coord/onecore-inline-i9-20261110`; remote #366
remained `ae13742fe4c6e2726c178e2409a1d69ef8fca939`. No concurrent source changes
were present. These two `git cherry-pick -x` operations applied without conflict:

| Reviewed #368 commit | Appended #366 commit |
| --- | --- |
| `4d8bd424681de0b798106ab7d7dc5cc4ca3240b8` | `3b4350bb88944d50fe0dd2f9f99d466952b1ad10` |
| `bfd5d322651974f9803f4c03942cf3ba3b5161d2` | `4e671215051d5f5f9720c7bfa1944912b83643a2` |

Both stable patch IDs match their originals. All41 files changed by #368,
including source, Occupancy workflow, ledger documentation and prior failed
evidence, are byte-identical to the reviewed head. Full Git blob IDs, SHA256
digests, sizes, patch IDs and parent correspondence are in
[evidence/correspondence.json](evidence/correspondence.json). This journal is
appended after that implementation head; the final exact pushed head belongs
in the completion REVIEW and Codex state report.

The copied repair keeps separate route-history accounting for retained
allocations, the finite independent tt bound including AVX2 n+8 reservation,
logical lower bounds and real assertions. No accounting-model change, invented
replacement repair or new unsafe operation was added. Guardian D103 at reviewed
bfd5d322 independently lifted the scoped accounting restriction:
[6790889882](https://github.com/lmmx/mibz2/pull/145#issuecomment-6790889882),
report `b7af95428922d3c81d8007d548ce22002db57eaf`.
Identical-source correspondence is not a new independent safety review.

## Frozen diagnostic remains unchanged

- Recipe: `bench-onecore-inline-i9`.
- Candidate: `d33787e138426cb441db6975f54b2eccca02e89a` (#361).
- Baseline: `b65b2aa54488f072cf005b8fe62461c32e6ef083` (v0.2.9).
- Ten paired Wikidata rounds, one thread; original baseline/candidate ratio
  and95% interval rule unchanged.
- Release-scaling driver SHA256:
  `6167171b7889706226340adc0beba89bfbf5e933af944d7f68e49d97bc7bfa07`.

The complete `.just/bench.just`, original registration, recipe test and Test
workflow byte-match the old #366 head. Benchmark/Miri workflow files and
`bench/workstation-queue.txt` also remain byte-identical. No ready/hold tag,
queue entry or Run signal was created. Old #366 failed Occupancy42923929886
remains a failure at its original head. #361's adverse text2t result, #365
DEFER, #371 REVISE METADATA and #368 records2t+3.8% [0.2,7.5] tradeoff stand.

## Actual untimed validation

The earlier lack of local Just is resolved using `rust-just==1.38.0` in an
isolated writable uv tool cache, matching the CI Just version. No repository
dependency or environment configuration changed. Python3.12.14; Rustfmt1.90.0.

**Four of four recipe tests pass, no skips**, with `ONECORE_REQUIRE_JUST=1`:
frozen driver digest, exact candidate/baseline/grid, fetch-failure propagation,
and actual Just parser/dry-run. Both the direct `.just/bench.just` and root
`Justfile` dry runs print exactly:

```text
git fetch -q origin perf/onecore-walk-inline
just bench-release-scaling d33787e138426cb441db6975f54b2eccca02e89a b65b2aa54488f072cf005b8fe62461c32e6ef083 10 "1"
```

These are printed commands, not executed fetch/build/benchmark operations.
The shell wiring tests execute only stand-ins. The exit17 fetch witness never
starts its stand-in measurement command. Rust formatting, Python compilation
and diff whitespace checks pass. Reviewed source/patch and frozen-file identity
checks pass. No local decoder measurement, Occupancy run or Miri run is claimed;
the normal changed-head CI supplies those applicable checks separately.

Reproduce the untimed tests (with Just1.38.0 on PATH):

```sh
ONECORE_REQUIRE_JUST=1 python3 bench/test_onecore_recipe.py -v
just --justfile .just/bench.just --dry-run bench-onecore-inline-i9
just --dry-run bench-onecore-inline-i9
cargo +1.90.0 fmt --all --check
python3 -m py_compile bench/test_onecore_recipe.py
git diff --check ae13742fe4c6e2726c178e2409a1d69ef8fca939 HEAD
```

Actual local invocations, environment, exit codes and logs are retained in
[evidence/checks.json](evidence/checks.json). Their incidental test durations
are not performance evidence. [SHA256.json](SHA256.json) binds this handoff and
all new evidence; copied #368 evidence retains its original manifests.

## Exact-head CI and remaining integration blockers

Normal PR synchronization will trigger the substantive changed-head checks,
including recipe Test, corrected ordinary/team Occupancy, Miri, other Actions
and legacy status. The exact observed run/job IDs and conclusions are recorded
after push in Codex's state report and #145 REVIEW, avoiding a documentation
push solely to restart CI. Pending is pending; no all-green claim is made here.
No manual workflow dispatch/rerun or scheduled polling is authorised or used.

**Implementation and local preparation are complete.** Workstation eligibility
still requires applicable CI/legacy success on the final exact recipe commit,
Coordinator reconciliation of this incorporation and diagnostic, and separate
deliberate integration into master plus PROTOCOL9 queue/tag/readiness steps.
This delivery neither performs those actions nor claims the job is ready.

After reconciling this result, the Coordinator should arrange the separate
matched production-versus-research inversion/whole-decoder context diagnostic
identified by #372, reconciled with Optimiser6782313693's ten-pair live-reader
plan and the superseded three-round proposal. That question is distinct from
this frozen #361 one-core recipe. It is not implemented, launched or
self-dispatched here. Only assignment6810684246 is completed by this delivery.
