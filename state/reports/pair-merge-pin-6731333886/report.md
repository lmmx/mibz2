# #272 merge-head freeze review

Inbox **6731333886**. Prior Researcher PROCEED: **6731275111**.
Old head: **ae62ea75a0e9e75a6d300264daffb99c6763e9f9**.
Proposed head: **69f97c80df517c3e966c0aa5988f6355fb19df0a**.
Instructions: docs/team/SPEC.md and SPEC.RESEARCHER.md, MODE/RULES/PROTOCOL at
a4f97714cc14366984015869d9d094d89488ac57.

**REVISE the freeze record's “merge-only, unchanged experiment” claim. No pair-kernel
revision requested.** The merge preserves the pair implementation, harness, validator,
thresholds and registration file, but changes an input to the registered synthetic screen.
Record this explicitly as a prospective C1 pin amendment before dispatch, or preserve the
old synthetic text input if an identical-experiment carry-over is intended. Nothing here
relaxes the Q1/Q2 criteria or asks for another performance run.

## Evidence: source identity is not workload identity

The new commit has two parents: the old reviewed head and master
8be686b5d6282f4cecb08b561077849ab8700dc8. Recursive Git trees were complete (not truncated).
Fourteen relevant files have identical blobs, including src/pair_runs.rs and its tests,
examples/pair_runs.rs, both Python validator files, the registration, bwt, coop, mtf, crc,
Cargo.toml/Cargo.lock, sweep.yml and the Wikidata spec. See blob-identity.json.

However, the unchanged harness calls common::corpora(1), then compresses each result at
level 9. The unchanged corpora function reads **every immediate regular file in src/,
tests/ and examples/, sorted within each directory, then README.md**. It concatenates these
as a seed and repeats that seed with copy labels until it reaches at least 3,000,000 bytes.
This is a dependency on repository contents, including code that is never executed by
the benchmark.

Three of those 33 tracked input files change:

| File | Old bytes | New bytes |
|---|---:|---:|
| README.md | 14,965 | 15,053 |
| examples/fuzz.rs | 5,846 | 12,300 |
| src/parallel.rs | 54,309 | 56,831 |
| Seed total, all 33 files | 814,653 | 823,717 |
| Generated synthetic text | **3,258,654** | **3,294,910** |

Each text is four seeds plus three 14-byte copy labels. The increase is **36,256 bytes**.
The Git blob manifests and replay.py reproduce this byte census without compression,
decoding or timing. These are clean-checkout lengths, derived from tracked regular files.

The generator's corpora function, Rng implementation and compressor helper are identical.
The other five synthetic inputs retain their fixed construction and seed. tests/common/mod.rs
changes, but that nested file is not itself read into the text input; its unchanged generator
reads the changed immediate files above. Likewise the new nested unwind test file is excluded.

Therefore the text's compressed bytes and block geometry cannot be assumed unchanged.
No claim is made about the direction or size of the resulting timing difference. Q2's
worst-synthetic point-estimate screen includes this file, so the change matters to the
registered experiment even though Wikidata, the primary judged corpus, is unchanged.

## What carries, and what must be recorded

The implementation and validity findings from 6731275111 carry to this head by blob identity;
there is no reason to rerun their 39 fixtures solely for this merge. The source-bound census
limitations in that review remain unchanged.

For the new head, record before the diagnostic dispatch:
- the full old/new pins and this source-generated synthetic-text change;
- the same Q1/Q2 thresholds, pilot exclusion, timing brackets and 15-round plan;
- that the judged synthetic screen uses the clean checkout's text at the new pin.

Then freeze the **new prospective experiment identity**, rather than claiming identical
input bytes. This deterministic documentation correction needs no additional Researcher
experiment or new performance minimum. If source-independent input equality is desired
instead, reconstruct/pin the old text and review that concrete harness amendment.

This is a C1 diagnostic registration clarification, not a request to alter bench.yml's
reserved workload or merge rule. There is no diagnostic result here to reinterpret.
My previous PROCEED was a review recommendation; the Coordinator owns the freeze record.

## Merge and CI scope

The Miri conflict resolution retains the old Miri step tail byte-for-byte: SSE2/default,
AVX2 fill, the expected-failure reachability control and nightly doc tests. It adds the
worker-recovery selection check and widens relevant path filters. The additions do not
replace the pair checks.

The parallel.rs diff is test instrumentation plus the non-test min_coop(shared) wrapper,
which returns MIN_COOP outside cfg(test). That supports source-level behavior equivalence
for production; it does not prove binary identity. The pair benchmark directly invokes
the inverse-BWT entries, not the parallel worker. The decisive dependency here is that
parallel.rs is also text-corpus data.

At the inspected new-head CI snapshot, rust, miri and bench were **in progress**. Fuzz smoke,
several build/platform/Python checks and other completed checks succeeded; release/deploy
checks were skipped. This is not a green-CI assertion. Keep the requested wait for normal CI;
no dispatch was performed. No independent rerun of the reported 70-pass/2-ignored local test.

## Consequence

Expected: an unchanged experiment with a repaired merge. Observed: measured kernel and
statistical rules unchanged, synthetic text changed through repository-content dependence.
No performance gap is closed and no performance conclusion follows. The bounded run remains
useful after an accurate prospective pin record and normal CI. Preserve D052's scoped safety
closure; this review neither broadens nor redoes B5.

Research only; no implementation, merge, timing, dispatch or automation changes.
