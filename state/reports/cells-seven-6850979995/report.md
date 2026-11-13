# Seven-family Cells screen: actual evidence

Implementation PR #378, head de1e5a8b4850cf0d33028682986ae4a641e67d6c.
Plan a49e307 was committed before the first full-corpus/control execution.
Queue6850979995; owner shipping mission. Frozen decoder d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2.

All **23 executions completed once locally**: two touched-memory controls, seven
families × Segmented/Cells/bundled libbzip2. All bytes, lengths and SHA-256s match;
all reported block CRCs match. No measurement samples were collected. Exact-head screen CI43417618214 and packaging CI43417618208 passed; hosted
legacy pre-commit passed. General Test/Benchmark still run at this checkpoint.

| Family | Cells admitted / total blocks | Other routes | All-small admitted blocks | Native peak KiB Segmented / Cells / oracle |
|---|---:|---|---:|---:|
| Wikidata | 196 / 431 | 185 density, 2 budget, 48 column | 0 | 24448 / 33164 / 5656 |
| Onebyte | 0 / 1 | 1 budget | — | 31948 / 34412 / 6768 |
| Pattern | 22 / 22 | — | 22 | 14268 / 18500 / 8624 |
| Random | 0 / 9 | 9 column | — | 21176 / 21040 / 5424 |
| Records | 0 / 11 | 11 column | — | 21296 / 21356 / 5424 |
| Runs | 0 / 1 | 1 density | — | 21316 / 21364 / 6240 |
| Text | 0 / 14 | 14 density | — | 17340 / 17328 / 5552 |

These are instrumented, untimed correctness processes, not production memory
qualification. In particular the eligibility inspector allocates one usize per
stored rule, up to 321664 bytes on Wikidata and 1028592 on pattern. Do not use
these RSS ratios to qualify/reject the production candidate.

The positive controls reported VmHWM 2232 / 66772 KiB, a 64540 KiB separation.
wait4 reported 2224 / 66764 KiB for the exact same child PIDs. This rejects the
old uniform41168KiB measuring outcome without assuming its cause. All child
records, startup/end RSS and VmHWM are retained.

## Candidate consequence

Pattern has 2700009 small spine seeds, zero long seeds, maximum unmarked factor7,
reachable unmarked expansion depth0. Its position initialization totals10800036
logical bytes and its two full rule scans total5400018 iterations. A guard in the
existing first marked-chain walk can potentially omit those costs plus the third
long-only walk. No separate classification pass is needed for this writer guard.

Wikidata has1432 small and5414 long seeds across196 admitted blocks; every such
block has at least one long seed. Maximum unmarked factor860287, expansion depth119.
Thus this guard must retain the existing writer for all admitted Wikidata blocks;
its branch/feature cost still needs whole-reader measurement. The screen supports
**one guarded writer candidate before a threshold dispatcher**, not a claimed
solution to the53% pattern slowdown. MONODROMY retains the mathematical disposition;
independent review and matched hooks-off qualification remain required.

No evidence here establishes missing wall/CPU results for random, records, runs or
text. They remain NOT MEASURED on the current i9 build. Historical CI results are
not substituted. The previously completed i9 result remains Wikidata0.881643,
pattern1.530338, onebyte0.535508 C/S wall; onebyte budget-refuses. Hooks-on phase
attribution remains STOP, and the current candidate has not been timed.

## Validation and provenance

- Six harness tests: flat/false memory controls, PID mismatch, CRC/timing rejection,
  actual empty/concatenated/error readers, missing executable, signal and timeout.
- Five original live reader/trace tests; two original F01 native tests; original
  arbitrary-column/corruption/concatenated-stream integration test pass.
- Frozen source files plus both additive patches are preserved in source.json and
  generated/. No original decoder operation was removed. Builds occurred before
  the implementation commit; build.json truthfully records that dirty preparation
  state. Exact source and binary SHA-256s, not a false clean-build claim, bind the
  local execution. CI builds the committed head independently.
- All compressed pins verified, text source exact57f51bbc; ordered path/hash
  manifest retained. Four previously missing decoded hashes are in screen/summary.json.
- Partial/error evidence is retained. The initial Miri invocation could not create
  its default read-only sysroot cache; a writable-cache invocation passed both F01 tests under Miri (25.37 seconds).
  This is regression validation, not independent B5 self-certification.
- F01 independent review6790022921 is not reopened; #365 integration DEFER6790163736
  and original gate6775049636 remain. No default/team/threshold change, i9 run,
  ready tag, queue entry, merge or release.

## Full bytes

The 21 byte streams were compared directly before lossless compression. Their
individual compressed sizes/hashes are in complete-output-identities.json. To
avoid storing three identical decoded streams per family, complete-oracle-bytes.tar
retains one complete gzip stream per family, reconstructing each arm's verified
bytes. Original per-arm output metadata, native rows and counters remain in screen/.
Archive SHA256: 00c083169ef0f6c32f92425c76fc21a7e47656b9b6a306dfaf86d0e3e3289f1c.

Next: reconcile exact-head PR CI and deliver screen completion. Coordinator should
route the single guarded writer implementation/qualification, reconciling MONODROMY
rather than creating a duplicate proposal. No prospective140+21+2 timing admission
is implied. GitHub comments do not automatically start Codex.
