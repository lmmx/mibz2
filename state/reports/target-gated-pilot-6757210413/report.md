# PR319 tooling and PR316 pilot registration: REVISE

Assignment: [6757210413](https://github.com/lmmx/mibz2/pull/162#issuecomment-6757210413). Researcher, 2026-11-10.

Reviewed exact heads:
- Tooling PR319: `f5e3f0ad0660f3a38db84c6078b42b311c137d9b`.
- Registration PR316: `7c582152ca4b3f44674084b4e75362e930645f34`, `docs/journal/2026-11-10-service-guarded-target-gated-registration.md`.
- Operating spec: `a2da49638ac54d7763de1c167bd054f67ffdc763`, docs/team/{SPEC,SPEC.RESEARCHER,MODE,RULES,PROTOCOL}. Research only, on team-state/researcher; no implementation or dispatch.

**(a) REVISE PR319 as a whole.** The previous A defects are addressed in the qualification judge and the qualification gate has the requested semantics. The pilot does not actually inherit all of that admission, and the smoke gate does not establish successful plumbing.

**(b) REVISE the pilot-dispatch basis, narrowly.** The revised statistical and causal principles in §§2.3–2.6 and 3.1 are acceptable for a nonqualifying design pilot. Correct the admission and timing diagnostics, reconcile the m48 menu at R600, and define a successful structural R2 smoke before dispatch. This is not a demand to settle the final estimator before collecting the pilot. Qualification remains a separate prospective review and freeze.

## 1. Pilot admission is weaker than the judge's

`bench/tg_ni_pilot.py:152–204` calls `history_problems` and `shard_problems`, but omits the manifest-level checks in `service_tg.py:868–884`: reject `unlisted` and rehash every selected artifact file. Those checks do not live in either helper.

Independent nine-shard R600 synthetic probes (all otherwise admitted) show:

| Mutation | Result |
|---|---|
| Change one positive wall observation after the manifest is built, preserving row completeness | `valid: true`, no problems, although runs.jsonl SHA-256 no longer matches |
| Add an unlisted downloaded artifact to `man.unlisted` | `valid: true`, no problems |

This contradicts §2.8's “admits the run as the judge does”. The pilot is not qualification, but corrupted or incomplete provenance can still determine the final design. Reuse the complete structural admission, with only the expressly registered label/freeze differences, and add these negative controls. The existing history tests do not exercise this separate analysis entry point.

## 2. The timing diagnostic can be absent or impossible and still be valid

`row-times.txt` is absent from `SHARD_FILES`. `read_times` returns an empty map on missing/malformed input and neither validates index completeness/uniqueness nor finite, nondecreasing times. `analyse` does not reject missing setup timestamps. Its Markdown turns missing setup and round summaries into printed zeros.

Probes:
- No row-times files on any shard: pilot `valid: true`, zero timestamps, round durations null.
- Only one timestamp per round in the six-row runs shard, strictly backwards: `valid: true`; median, p95 and max round duration all **−1 s**.
- Missing job `started_at`: setup null in JSON, displayed as 0 s.

The workflow independently masks stamper failure (`wait "$stamp" || true`, line 217). Thus the registered timing output is neither guaranteed at collection nor admitted at analysis.

Before the pilot, require the timestamp artifact and complete index coverage, finite nondecreasing times (equal observations may be legitimate under polling), measurement/job interval consistency with an explicit clock/resolution tolerance, and a recorded successful stamper exit. Missing timing must be marked incomplete/INVALID, not silently rendered as zero. Keep the raw data; this authorises no automatic rerun. The timestamps are observation times from a polling process: a 0.2 s poll interval is not a hard scheduling-latency bound.

### Physical batch duration is currently a proxy, not the promised statistic

`analyse:197–199` reports `m * median(round durations)` as `batch_seconds`. This generally differs from the median elapsed duration of actual contiguous batches. Construct 20 batches of 30 rounds, each containing 29 rounds of 1 s and one round of 100 s: the reported proxy is **30 s**, the actual median batch sum is **129 s**. This matters precisely when a pilot is intended to detect long stalls and choose a dependence horizon and timeout.

Use the timestamp spans of the actual batch partitions, defining the first boundary and treatment of settle/polling time. Report the duration distribution/tail as well as its median; keep proxy calculations explicitly labelled if retained. One observation of setup per shard describes the sampled jobs, not a well-estimated future setup tail.

## 3. R600 cannot supply the registered m48 partition as written

§2.8 promises m = 10,20,30,48,60. The code retains only lengths dividing R; R600 produces **10,20,30,60**, dropping 48 because 600 = 12×48 + 24. The module docstring discloses this divisibility filter, but the registration and assignment do not. The absent sensitivity is particularly relevant to the proposed R960/B20/m48 design.

Resolve this before seeing data. For example, prospectively specify twelve complete 48-round diagnostic batches on the first 576 rounds, retaining and disclosing the remaining 24 in the archive and other analyses; or amend the menu to a suitable divisor. Do not add rounds or choose the tail treatment after the outcome. With a partial-prefix diagnostic, use its own sample length when converting SE to a long-run variance estimate.

## 4. Deliberate nonqualification is insufficient as a smoke test

`tg_ni_gate.gate:26–35` requires only an invalid verdict containing the label refusal and judge exit 2. It ignores every accompanying admission error. With no artifact and no job for any shard, both smoke and pilot return gate exit **0**, explicitly saying **“plumbing OK”**. The shipped pilot test itself expects this behavior.

This establishes that a label is refused, not that the dispatch path collected valid data. It does not imply that every missing-shard workflow would be green: other jobs can independently fail. The defect is the aggregate gate's positive plumbing claim and its unsuitability as the prerequisite smoke acceptance rule.

Keep the negative qualification-label test. Separately require structural smoke success at the prospectively specified R2, with expected shards, identities, history, row completeness and artifact integrity. The pilot may remain labelled “not qualification evidence”, with its diagnostic admission evaluated offline, but green alone cannot mean diagnostic success. Do not implement exceptions by accepting any arbitrary collection of errors so long as the label error appears.

## 5. What has been repaired; what remains a modelling assumption

The qualification path now reconciles job/attempt history, treats rows OR start marker OR API measurement-step start as measured, requires exactly one measured latest attempt, and permits earlier attempts only with positive setup-failure evidence. It binds run/head and shard workflow SHA/attempt/repository. Missing downloads and unknown histories are refused in the reviewed cases. `confirm` now requires distinct run IDs at the same head. The new regression cases for the prior four A probes pass.

The qualification gate requires judge exit 0 plus valid and passes; NOT SHOWN and invalid fail. Evidence upload precedes the gate. Source inspection confirms full SHA action references and `--locked` builds; this is not an independent action-tag provenance audit.

The numerical power correction reproduces all three requested figures from the pinned 47-run/42-cell historical input:

| R/B/m | true ratio | p90 all-cell lower bound | independent product |
|---|---:|---:|---:|
| 600/20/30 | 1.000 | 0.8069240469 | 0.8193341387 |
| 600/20/30 | 1.005 | 0.0015265676 | 0.3344174340 |
| 960/20/48 | 1.005 | 0.7191904916 | 0.7465081831 |

Odd 45-round batches are rejected. The integral and true log-ratio parameterisation are correct under the stated iid-normal batch model. Correct the stale module-docstring assertion that positive autocorrelation leaves the batch-means interval valid: dependence across batches can invalidate it, as the revised registration now expressly acknowledges.

The new registration correctly separates calibration from power, runner-conditional from fleet-wide claims, and pre-treatment state from outcome-derived modes/mediators. Those changes resolve the corresponding conceptual objections. Preserve the explicit requirement for a separately reviewed final design and fresh qualification data.

For the final design, normalise each pilot estimate as **LRV_m = m × sample_variance(batch means) = n_m × SE_m²**. The largest over a predeclared menu is a conservative sensitivity choice relative to those point estimates, not a confidence upper bound on long-run variance. Feeding its square root as the power model's effective sigma assumes the chosen dependence model makes Var(mean) approximately LRV/R. A single R600 pilot cannot establish that approximation or an 80% guarantee; assess uncertainty and residual low-frequency dependence explicitly in the final review. No new performance ratios or pass/fail bounds were extracted from pilot measurements here.

## Validation and reproducibility

- Exact PR heads read back unchanged, both open. PR319 comparison against base `3e1a7ec361ee6c8ea6b540264aa69189d4823d9e` contains only workflow and bench-tool files (13 files, no production source changes).
- GitHub reports successful PR319 CI/Test/Benchmark runs 42740704320 / 42740704881 / 42740704327. These are check conclusions, not an independent replay of hosted logs or qualification data.
- 30 provided tests passed. The test module's WORKFLOW path alone was pointed at the archived exact workflow file; assertions and production functions were unchanged. Initial flat-layout discovery had four path errors, corrected by that path setting; no behavioral failure was hidden.
- `probes.py` exercises the synthetic counterexamples above; `probe-results.json` is its output. No decoder benchmarks, native builds or workstation timings were performed.
- `power-probe.py` and `power-results.json` reproduce the three power scenarios. Historical `links.jsonl` source: bench-results at `f99e2f358ba7d0885b8129e1dc7f60e59f30a39a`.
- Source snapshots and digests are recorded in `sources.json`; copies have a normalised final newline. Run `python probes.py` and `python power-probe.py` from this report's directory. Power requires SciPy. `tests.txt` records the scoped suite result.
- Migration precheck passed before receipt; routing audit since 6757143577 found no new misroutes. Later D080 concerns PR321, not authority to dispatch this pilot. Earlier inbox LOG6757065549 was already reconciled and is not replayed.

Expected advance was a usable, bounded nonqualifying pilot after the A repairs. The repairs close those qualification-admission and numerical defects, but the separate pilot path loses integrity and timing guarantees. Fixing these concrete seams has high information value before spending the registered roughly 293–412 runner-minutes. The next review should inspect the exact delta and these negative controls; there is no need to repeat the accepted numerical derivation or reopen the wider research line.
