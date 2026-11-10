# D083 team-screen price: native coverage first, counted tier not yet priced
Inbox [6759587407](https://github.com/lmmx/mibz2/pull/160#issuecomment-6759587407). Reviewed draft/code at **ffb568913ce9bd6eeb807082ac935570e36c3c8d**. Operating specs a2da49638ac54d7763de1c167bd054f67ffdc763. Proposal and source audit only; no constructor, decoder execution, build, new timing or performance qualification.

## Decision

The coverage gap warrants a bounded affected-PR screen. **The pinned full-Callgrind design is not ready to become a mandatory PR tier.** Its cost is an unvalidated model; its checker has a demonstrated false pass; its base-arm attribution may require symbols that the unmodified merge-base binary does not retain. Recommend native exact-result plus semantic execution/engagement probes as the eventual routine tier, matched full counts as the diagnostic/full tier. Until probes exist and are reviewed, Callgrind is a prospective fallback, not a cheaper equivalent already available.

D083 prefers a bounded screen if cost fits, not unconditional full profiles on every PR. D084 permits this design work separately: it does not repair #334's FAIL-MEASUREMENT, grant its unearned qualifying run, or permit changing a registered binary to obtain convenient symbols.

## 1. What is actually evidenced

Read draft, checker, team_gap_count.py, examples/ab.rs, bench.yml and CLAUDE.md at the requested pin; base coop.rs and ab.rs at c47fbe50cf99fbff930526356b9723bfc453ddb9. Read D083 and D084 reviews, plus the staged-count registration/result addendum.

Retrieved original benchmark job log **129639441404**, run **42758830562**. It built head dd5d8bf76477499292bdd9bd6d2f2afcffcccd59 against c47fbe50cf99fbff930526356b9723bfc453ddb9, not the requested draft pin. Log supports approximately:
- checkout 7.3 s; lbzip2 installation 24.8 s;
- head build 27.08 s; base build 25.14 s;
- Rust cache restored ~64 MB; windows cache ~12 MB;
- corpus generation 6.5 s; window reconstruction 9.8 s;
- A/B run ~235.7 s; stages ~127.9 s; whole visible job ~473 s.

Those are existing log intervals, not measurements made by this review. The cache-hit builds compiled ab plus additional examples; an ab-only job need not have identical cost. The log does **not** establish Valgrind installation, cold builds or this screen's runtime. The runner reports four CPUs/two cores; perf cycles unavailable, perf_event_paranoid=4. Do not base the cheap alternative on PMU access. No multi-run CI distribution was supplied or reconstructed. The draft's local approximately30 s/profile is a transferred estimate, not a CI quantile or dependable upper limit.

The requested pin's CLAUDE.md contains no six-minute/seconds-per-round clause; bench.yml does retain ten rounds and separate stage work. I therefore assess the brief's six-minute budget as the requested constraint, not a newly verified quote from that file. The historical ~473 s job cannot prove budget compliance by itself.

## 2. Complete price and uncertainty

Let T = checkout/toolchain/cache restore + apt + head build + base build + input generation/fetch + native checks/oracles + counted execution + parsing/disassembly + compression/upload/cache-save. All terms consume runner allocation while the job waits or runs. A cache miss is not only compilation: dependency/network work, target compatibility, base invalidation and cache save matter. Do not count the same cold base build twice: it is already in the draft's 300–600 s build range if that means both builds.

| Item | Draft warm seconds | Draft cold seconds | Audit |
|---|---:|---:|---|
| checkout/toolchain/apt | 45 | 45 | lbzip2-only install observed; Valgrind delta and apt tail absent |
| two release builds | 55 | 300–600 | warm supported approximately; cold range unvalidated |
| inputs | 20 | 20 | hit scenario; missing windows/network fallback not bounded |
| four arms × nine inputs | 15 | 15 | 36 decodes; 0.1–0.6 s each implies 3.6–21.6 s, not an observed15 |
| two counted W decodes | 60–120 | 60–120 | local-to-runner transfer assumption |
| parser/disassembly | 20–40 | 20–40 | unmeasured; scales with profile records, not merely gzip size |
| upload | 5 | 5 | compression/cache-save not separately shown |
| **sum of stated rows** | **220–300** | **465–845** | **3.67–5.0 / 7.75–14.08 min, before omissions** |

A1 also requires seven W/C oracle decodes. The 36 count omits them: minimally **43 native decodes**, then two Callgrind executions. Window construction's lbzip2 verification cannot silently stand in for all C oracles and per-input records. Persist their expected output identities with input digests so reuse is explicit. E1/E2 generation, normalization, timeouts, profile manifest validation and mutation/self-tests also need a place.

Price counting through effective throughput, not target-native slowdown. With illustrative total16 GIr across the two arms and non-counting cost170 s (midpoint assumptions), effective Valgrind rate 0.267 / 0.133 / 0.067 / 0.033 GIr/s gives approximately **230 / 290 / 409 / 655 s warm**. Replacing55 s builds with450 s adds395 s: **625 / 685 / 804 / 1050 s cold**. These are sensitivity scenarios, not CI predictions. Exact arms may execute different Ir; schedule changes claims, allocation and coordination. Cache-locality, translation, instruction mix and runner contention affect effective rate. More native CPUs do not proportionally accelerate a serialized instrumented execution.

A midpoint warm260/cold655 model gives E[T]=260+395p seconds for cold-cache probability p, before omitted costs. At p=0.1/0.25/0.5 this is about5.0/6.0/7.6 runner-minutes. No p is established. “About +4 runner-minutes” describes one optimistic hit case, not expected fleet cost. No new paid infrastructure or spending is authorized here.

For @9, another two counted executions cost60–120 s **plus** parsing/disassembly20–40 s, native verification, compression and extra profiles. Thus the incremental known modeled subtotal is **80–160 s**, not60–120. Thread count/profile count and scheduling overhead change too. A3's per-helper233 rule cannot be reused at@9: per-team block counts differ, with stable team identities and total233 unique cooperative blocks required.

### Parallelism does not erase merge latency

If existing required checks finish at time B, screen queue delay is Qs and runtime Ts, added merge-ready latency is:
**max(0, Qs+Ts−B)**, assuming no resource interference. If the new job changes the bench queue, use its changed B too.

With B≈473 s, immediate warm220–300 s adds zero in that particular scenario; cold465–845 s adds0–372 s before omissions. Queues, a rerun or a short existing job can make even warm screening critical. Runner-minutes always rise. On a shared physical/self-hosted runner, concurrent profiling can also invalidate the benchmark's isolation; a separate job definition alone does not prove separate resources. Do not add screen work inside timed rounds or remove frozen cells.

## 3. What one counted execution buys—and fails to prove

One Callgrind execution per arm can establish that instrumented execution's kernel calls and accounting conservation. It does not establish native helper balance, absence of races/deadlock under other interleavings, or target performance. It is appropriate for diagnostic A4; no repetitions are needed merely to pretend its scheduling-dependent counts are deterministic.

The pinned checker needs fixes before it can block merges:
1. **Demonstrated false pass:** four files each containing {} print A1 ok and return0. outputs() uses .get for out/hash/err and compares tuples without required-field/type checks. One record also passes trivially. Require one identified record for every expected build/arm/input; reject missing/duplicate/schema-invalid records.
2. Agreement is insufficient: all valid-input arms can share an error, or all malformed-input arms can unexpectedly succeed. Assert expected success/error independently, the fixed corruptions actually applied, and the oracle result. A 64-bit non-cryptographic output hash is a regression checksum, not literal byte-exact proof. Compare output streams or explicitly freeze the accepted checksum scope with stronger oracle identities.
3. ab child returns error display text and cumulative streamed bytes/hash. It does not emit a structured block identity/error-kind pair. The stub therefore does not establish “same error at the same block.” Buffered output-before-error may have schedule/API semantics requiring explicit treatment. Do not invent a block identifier by comparing strings.
4. A2 counts matching calls across profiles, not distinct validated helper identities tied to the same team/block. Broad matching may include leader calls. A3 classifies any share-calling thread as helper but does not independently assert the exact expected helper roster. Require unique process/thread manifests and per-role conjunctions. Reject stale or mixed prefixes.
5. team_gap_count derives byte counts from particular byte-store assembly and coop.rs line text. Each arm needs **its own source and binary**; passing head source lines with the base executable can misattribute. The stub defaults --coop-src to current src/coop.rs. Compiler lowering, inlining or vectorized stores must yield unsupported-attribution failure rather than zero-credit or a wrong total.
6. **Base symbol problem:** requested draft's own addendum records traverse being inlined before c78c18d3 added inline(never). The merge-base source c47fbe50 has no such annotation. That does not prove its exact binary lacks a symbol—we did not inspect that binary—but means a must-execute standalone traverse call is not a stable base contract. Do not patch base just to pass A2 and still call it the registered base; settle attribution or instrumentation before freezing.
7. The A4 stub writes per-thread stage totals but omits assembly/RLE1/CRC and does not produce the promised walk/inversion Ir/D normalization by itself. Add denominators and whole-process accounting/reconciliation in the report; never compare thread IDs across builds as though they identify the same worker.
8. Aggregate claim/byte conservation can hide compensating errors and says little about missing transition coverage. Freeze the actual short/large transition sequence or semantic coverage observations, not merely233/198 totals. Full-tier stable team/block accounting must not infer team membership from coincidentally equal totals.

These are screen-readiness defects, not evidence that the decoder is wrong.

## 4. Cheaper A2/A3 and the ranked tier

**First choice: native semantic probes plus output/error coverage.** Proposal only: record per-thread local entry counts for kernel identity and path, share/walker invocations, successful/exhausting claims, walked bytes, and solo/relay/resume transitions; publish once on thread exit/join keyed by build, team, helper, block. Count bytes arithmetically per segment/tile, not with an atomic increment per byte. Charge reset, per-block storage and aggregation. Use no global hot counter: it would serialize the team and erase the scheduling property sought. Keep per-thread storage separated to avoid introduced false sharing.

Exact symbol-entry tracing is an alternative only if actual binary locations, inlining, tracing permissions and probe loss are validated. nm/objdump symbol **presence** proves availability, never execution. Sampling can demonstrate positive execution but missing samples cannot prove forbidden code absent or a helper never entered it; it cannot enforce exact233/conservation. The logged perf denial reinforces that PMU sampling is no ready substitute.

Instrumentation changes the binary and may change schedule. A native instrumented screen proves its own semantic engagement, not native timing, fairness or uninstrumented-binary identity. Keep uninstrumented forced head/base and plain A1 records as well; count instrumented records separately. Each extra release build can erase runtime savings: this design must avoid an unpriced third/fourth fat-LTO build or explicitly charge it. Constant64-bit per-thread counters and bounded block ledgers are a design, not an implemented free feature.

**Second choice: retained full Callgrind in a separately frozen diagnostic/full tier.** It supplies A4 and checks the semantic probes against counted accounting at exact pins. Freeze once; no retries to achieve a desirable helper split. Keep the original #334 result unchanged. If Guardian interprets D083 as requiring full matched counts on every affected PR, obtain its prospective ruling before reducing that tier; this review does not waive that obligation.

**Fallback for initial PR coverage:** bounded one-team counted screen only after exact merge-base attribution and checker fixes, with a registered setup/run timeout, invalidity classification and full cost ceiling established from retained CI logs. A timeout is missing coverage, not PASS. Do not silently substitute sampling when count setup fails.

The native-runtime saving available relative to the draft is roughly80–160 s per job from counting plus parsing; apt-Valgrind delta may add savings, but probe builds/checks cost something. Reusing exact immutable build/input artifacts can save duplicate setup, but waiting for the bench artifact serializes dependencies and can put the screen on the critical path. A dedicated producer can avoid duplicated builds; that is a separately reviewed workflow change, not an assumed free optimization.

The trigger set must include the screen workflow, parser, generated-corpus recipe, compiler/config/lockfile changes and other dependencies that can change the covered binary/inputs, not only the six listed source paths. Otherwise the strongest checks can be bypassed unintentionally by changing their own machinery. Deduplicate per head/base pair; cancel obsolete PR attempts without discarding their status/artifacts. Establish D083/C6 dispositions prospectively with Researcher/Guardian. This review recommends economics; it does not adopt a standing acceptance change.

## 5. Whole-decoder opportunity and next falsifier

This screen earns value through previously absent team-path correctness/engagement evidence; it is not a throughput optimization. Relative to repeated wide candidate timing or more speculative compact-table work, a bounded native screen supports the higher-ranked team/service/emitter path at low recurring marginal cost. Two whole-file instrumented profiles on every touched PR need an additional case: demonstrable diagnostic information that the cheaper semantic screen cannot provide.

Expected: few-minute parallel coverage with executed-kernel proof. Found: warm build subtotal supported, complete CI price unmeasured, unconditional zero latency incorrect, A1 false pass and base-symbol attribution unresolved. Gap closed: explicit resource/latency equations and a reproducible checker counterexample; no runtime uncertainty falsely resolved.

Next prospective falsifier: freeze a repaired exact-pin screen and its evidence/invalidity rules, then retain all setup/build/count/parser durations and runner/cache identities from the allowed CI execution. Judge total runner and merge-ready cost against the predeclared budget; compare coverage delivered by native probes, not native speed. This is a proposed registration, no run performed or authorized by this review.

Routing audit reproduced the existing unresolved researcher-inbox LOG6757065549 and five already resolved historical channel items; no new misroute. Migration source head and exact manifest digest verified. The persistent task remains enabled.
