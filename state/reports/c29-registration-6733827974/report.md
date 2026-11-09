# C29 registration review: REVISE environment admission, retain design
Inbox 6733827974; queued 6733270759. Source dab79fe7664c040808189b9906d08c9ce985b381.
Specs/MODE/RULES/PROTOCOL: a4f97714cc14366984015869d9d094d89488ac57.

## Verdict
REVISE narrowly before freeze: bind the judged result to the intended executed kernel and runner capacity. The thresholds, matched control and single primary are sound for the bounded diagnostic. No line STOP, new B5 HOLD, default-on approval or production gate change follows.

## Thresholds and estimand
Retain wall point estimate <=0.98, wall upper <1, CPU upper <=1.02; STOP wall lower >0.99; otherwise INCONCLUSIVE. The gap avoids pretending every non-LEAD establishes failure. This means evidence of some wall improvement plus a 2% point-estimate hurdle, **not** 95% confidence of at least 2% improvement. For example wall (.98,.975,.985), CPU (1,.995,1.005) is LEAD. That is consistent with the written rule; label it accurately.

The prior reg/ctl result motivates a fresh prospective hurdle; it does not predict a shipped-worker gain with certainty. From 0.9739 [0.9682,0.9797] over 15 rounds, log-ratio standard deviation is roughly 0.0107. Holding variance and independence fixed, 20 rounds gives log half-width about 0.0050 (absolute ratio half-width about 0.0049). The stated precision forecast is reasonable **conditional on those assumptions**; more rounds do not guarantee it. The reporter uses df19 correctly. No optional extension after inspecting the 20 rounds.

The CPU cap is an economic no-harm constraint, not a physical law: less wall at more CPU can arise from increased overlap or scheduling. A failure of this cap declines this candidate under this registration, not all register-list mechanisms.

Keep 9 threads as the only judged cell. The 3-thread secondary is valuable precisely because shared-resource pressure can reverse rankings. Report its estimate and interval even when inconvenient; never promote it to rescue a failed primary. No additional judged family or post-hoc selection. This research registration does not change bench.yml's workload, repeat policy or merge acceptance.

## Matched control: source check
examples/ab.rs constructs both arms with teams=max(threads/3,1), k=3 and slots=SLOTS_PER_THREAD*teams. The control's “both” levers are park=true, small=true, spin2k=false, register_list=false; the treatment inherits PIPELINED_LEVERS and changes only register_list=true. Both use Kernels::detect(), the same reader/drain/hash and binary.

src/parallel.rs passes that runtime bool to front_stage, selecting mtf_checkpoints_reg versus mtf_checkpoints. The stage counter field, initialization and fetch_add are all cfg(feature="c29-reach"). Cargo defaults are empty; sweep's build requests no features. The source therefore closes D060's timed atomic confound. The bundled treatment remains register-list/no-s2u plus short-run stores, not a list-only causal test. Common binary layout removes between-build layout differences; it does not make the two kernels' instruction-cache footprints identical, nor should it.

I inspected source, not fresh compiled listings, and did not rerun native B5 tests. D060's scoped evidence is separate from this scientific registration review.

## Missing executable-environment predicate
mtf_checkpoints_reg explicitly returns mtf_checkpoints_scalar when k.avx2 is false. The control also has a scalar path. The intended AVX2 register-list/short-store treatment is then absent. A precise ratio near one is a fallback/control result, not a test rejecting that treatment.

sweep uses moving ubuntu-latest; it writes ab env separately, prints nproc/model, but its identity and reporter do not require AVX2 or the registered four-vCPU allocation. CPU capacity is material: the justification explicitly depends on nine threads oversubscribing four vCPUs. “This runner's result” limits generalization but does not establish that the registered setting was run.

Reproduction: all **four existing reporter tests pass**. The standard ratio-one fixture, containing no AVX2 or capacity evidence, is VALID (JUDGED) STOP at 9 threads, 1.0000 [0.9986,1.0014]. Adding illustrative identity fields avx2=false and cpu_count=2 leaves it unchanged; these are deliberately unrecognized fields, not existing schema keys. This demonstrates absent validation, not that an actual CI runner lacked AVX2. No actual C29 timing bundle was submitted or judged here.

Closure: record the actual detected kernel capability and effective available CPU allocation (affinity/quota where relevant), bind them to the run identity, and refuse a judged treatment reading when required evidence is absent or the registered setting differs. Retain such runs as UNJUDGED/fallback diagnostics, without retrying until the verdict passes. Alternatively revise the prospective question and explicit strata before freezing; do not silently broaden it after data. This is diagnostic instrumentation, not a bench.yml grid change. No need to restore the timed reach counter.

The external sampled-name predicate also retains D059's false-INVALID risk: inherited names or short-lived helpers may be missed. Longer corpus duration is mitigation, not completeness proof. This need not create a new safety HOLD; disclose the risk and never waive plans or discard rounds after seeing it. Do not import #270's timed recorder without also recording that source/instrumentation change.

## Freeze identity
Freeze a full source commit containing the final journal, both kernels/switch/arm dispatch, Cargo manifest/lock and build configuration, sweep workflow, c29 reporter and team_size t-table dependency, and corpus recipe/digest. The dispatched ref must resolve to that full registered_head, not merely supply an unrelated matching-looking input. Retain toolchain/features, binary/corpus before/after hashes, actual environment, raw 80 records, progress, admission and failure output. Specify the bounded 20-round attempt and no result-driven reruns. Workflow builds featureless release; any source or feature change needs a new named registration, not reuse of this evidence.

No master merge is required by the matched-control design itself: both arms are in one build. If integration changes are intended, merge them before selecting this exact freeze and rerun appropriate checks. Current journal is still DRAFT.

## Off-path evidence and next value
The newer +0.2% [-0.8%,+1.1%] on EPYC 7763 does not reproduce the earlier +1.1% on EPYC 9V74, but its interval still includes +1.1%. That is not equivalence or proof of no regression. Identical Ir does not fix alignment/timing. The +0.3% placebo supports layout sensitivity, not attribution of the entire earlier effect. Four flagged comparisons among 21 are not explained solely by saying “95%”; correlation and multiplicity matter. None of this blocks the bounded exposed-path diagnostic; preserve the unresolved off-path interpretation for adoption.

Expected advance: test whether the research-worker lead transfers to the shipped worker, without the prior atomic confound. What closed: control definition and instrumentation mismatch. What remains: trustworthy identification of the executed treatment/environment. Its repair costs no new performance matrix and prevents a precise but irrelevant STOP. After closure, this diagnostic retains priority/value; default-on still needs D060's switched panic/reuse and exact-constructor reach plus separately registered exposed-path performance and the standing CI adoption gate.

## Evidence and process
Source snapshots bench/ reproduce: python bench/test_c29_checkpoints.py; python probe.py.
probe-results.json records the independent probes. No implementation, native timing, merge, tags, queue or automation changes.
Migration manifest SHA256 verified 9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0; legacy head matches cursor. Pinned routing audit: zero unresolved, five resolved historical entries. REVIEW routes to #145. Exact-body readback and role-aware routing follow publication; pinned verify's hardcoded Coordinator sender limitation is reported separately.
