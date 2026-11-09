# C29 environment admission: preserve unknown quota

Trigger: [6734526651](https://github.com/lmmx/mibz2/pull/162#issuecomment-6734526651), dispatching QUEUED 6734517214. Prior REVIEW: 6734359855.
Reviewed source: **7ed5b29349e4281247dc77ea9391bc5f71317836**, PR #281.
Instructions: docs/team/SPEC.md and SPEC.RESEARCHER.md plus MODE/RULES/PROTOCOL at **a4f97714cc14366984015869d9d094d89488ac57**.

**REVISE narrowly before freeze: distinguish unknown quota from known unlimited quota.** The AVX2 and per-record CPU-allocation objections are closed. The remaining defect is in evidence collection/encoding, not the wall/CPU thresholds or the matched control. No implementation, native benchmark, dispatch, merge or default-on approval here.

## Evidence and counterexample

The ten repository reporter tests pass. An additional 24 untimed probes, archived beside this report, establish:

- Missing avx2, cpus_allowed, available_parallelism or cpus in record 0, 39 or 79 produces exit 3, UNJUDGED (FALLBACK).
- Explicit false, 2.0, string "4", NaN and infinity quotas produce exit 3; a missing top-level quota key also produces exit 3.
- Correct positive fixtures retain JUDGED STOP at wall 1.0000 [0.9986, 1.0014] and LEAD at 0.97. Invalid child status retains INVALID precedence, exit 2.
- **Absence mapped to null instead produces exit 0, VALID (JUDGED), STOP at 1.0000 [0.9986, 1.0014].**

That last distinction is lost in two source locations:

1. examples/ab.rs:981–1004 builds an Option<Option<f64>>. A successful unlimited read is Some(None); failure to read/parse both quota paths is None. At lines 1048–1050, both become JSON null: only Some(Some(q)) is rendered as a number.
2. sweep.yml writes cpu_quota using env.get("cpu_quota"). An absent key likewise becomes null.

environment_problems then explicitly accepts None as "no quota." Its missing-key check cannot recover information the producer discarded. The archived workflow_absent_quota_mapped_to_null probe supplies the three four-CPU counts and true AVX2, omits quota from the env object, and applies the exact workflow mapping. The unreadable_quota_serialized_as_null probe exercises the collector's output representation. These are synthetic evidence counterexamples, **not evidence that a CI runner had too little capacity**.

Four reported CPU counts do not establish that this independent quota collector successfully measured an unlimited quota. If the registration means those counts alone are sufficient, that would be a different predicate; it currently promises "no quota or at least four." Preserve that promise rather than silently erasing the unknown case.

**Minimal closure:** retain a distinguishable detection state (known unlimited / known finite / unknown), preserve it through identity construction, and send unknown or malformed quota evidence to UNJUDGED (FALLBACK), exit 3. Keep successful unlimited and finite >=4 positive controls, and add collector-to-identity fixtures for unavailable/malformed quota evidence and absent env keys. No new treatment, threshold, repeat policy or performance run is needed. No retry-to-pass.

## Other boundaries, proportionately

The harness-parent status is now retained and the workflow returns it whenever the reporter returns zero. That is a real repair. The reporter itself ignores ab_run_status: a complete 80-record synthetic set plus status 101 still prints JUDGED STOP and exits zero. The workflow then fails with 101; therefore this is **not an additional dispatch blocker** here. Read the reporter together with the run status; do not call a failed workflow an admitted attempt just because its summary says JUDGED. It would be useful to reflect this contradiction in the report as well. No actual failed complete run was observed.

AVX2 detection is the same Kernels::detect() used by these in-process arms; the new facts are collected after the wall/CPU snapshots. This supports the intended treatment check without charging an added detection to the measured decode. It does not itself replace B5, prove every kernel call, or measure host contention. The inherited sampled-thread census can still false-INVALID; the revised disclosure and no-waiver/no-rerun policy are appropriate.

## The three explicit open questions

**Recorded rather than pinned stable toolchain: acceptable for this bounded, one-build paired CI diagnostic.** Both arms share one binary. Record rustc -vV, Cargo, flags and binary digests as R2.1 specifies. A later stable build is not an exact compiler replication; same source SHA alone must not be presented as identical generated code. No extra compiler-version gate is necessary for this question.

**features: [] by construction: acceptable under the frozen workflow.** Cargo.toml at the reviewed SHA has default = []; the command is cargo build --locked --release with no --features. The field means no optional mibz2 features, not literally no dependency features (pbz2 explicitly enables simd). It is a provenance assertion backed by the build invocation and frozen configuration, not an independent measurement; state that scope. No new runtime attestation required.

**Missing run IDs: provenance correction, not a scientific blocker.** Link EPYC 7763 run [42572506720](https://github.com/lmmx/mibz2/actions/runs/42572506720) at dab79fe7, placebo [42572691025](https://github.com/lmmx/mibz2/actions/runs/42572691025) at 2542d954, and original [42569223731](https://github.com/lmmx/mibz2/actions/runs/42569223731) at 08db6405. These associations are recorded in Coordinator LOG 6733823754 and the journal; this pass does not re-download/recompute their timing. Preserve R2's caution: non-reproduction is not equivalence, and the placebo does not identify the entire earlier +1.1%.

## Disposition and next value

Expected: close the environment admission gap. Observed: AVX2/capacity fields and positive/negative readings now behave correctly; the quota producer still collapses missing evidence into the admitted unlimited state. Most of the gap is closed. The mechanism learned is an observability failure across the producer/reporter boundary, not evidence against C29's kernel.

Next useful work is this small evidence-state repair and its untimed regression fixture, followed by the Coordinator's explicit freeze at the actual runnable source. The proposed CI experiment retains value: it tests the exposed shipped pipelined path with the matched control, 9-thread primary, 3-thread secondary, 20 rounds and unchanged corpus. A LEAD remains evidence of some improvement with a 2% point hurdle, not 95% confidence of a 2% gain. D060 and production adoption evidence remain separate.

## Reproduction and scope

Run python bench/test_c29_checkpoints.py (10 pass), then python probes.py (24 checks). The three bench Python files are exact source snapshots from the reviewed commit. probes.py contains only review fixtures; it does not change the implementation. probe-results.json retains full outputs. source/ holds the relevant Rust harness, workflow, journal and Cargo.toml snapshots. audit.json records the pinned routing audit: no unresolved misroutes. State publication remains on team-state/researcher, with one REVIEW on #145.
