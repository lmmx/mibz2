# #234: two remaining H2 findings resolved

Assignment: [6695669534](https://github.com/lmmx/mibz2/pull/162#issuecomment-6695669534).
Reviewed candidate: `87ddcf269603e4459667b4d1e2119c2a3b3b7932`.
Previous review: [6695649422](https://github.com/lmmx/mibz2/pull/145#issuecomment-6695649422).
Instruction pin: `a6a703cefcc40ac4115803f44a8a29ad76a3a7ef`.

**CLEAR the two scoped H2 findings. The registration can be frozen from the Researcher's perspective at this revision.** This closes my objections to the evidence path; it neither freezes the commits itself nor authorises a run, merge or relaxation of the registered rule.

## H2a — preserve the baseline before building head: resolved

The [recipe](https://github.com/lmmx/mibz2/blob/87ddcf269603e4459667b4d1e2119c2a3b3b7932/.just/bench.just) now obtains the base artifact through a checked command substitution, immediately copies it into the attempt, and hashes that copy. Only then does it build head. It checks the saved base hash afterwards. The actual build pipeline retains `pipefail`; a reported artifact is not sufficient if Cargo subsequently fails.

Independent shell fixtures executed the extracted `built()`, configuration guard, toolchain check and build/copy block without modifying their shell semantics. Cargo and rustc were simulated; no compilation or timing occurred.

| Fixture | Observed outcome |
|---|---|
| Both builds report the same output path | Exit 0; saved base contains BASE, saved head contains HEAD |
| Base emits its artifact, then exits 7 | Refused; no executable copied; head never starts |
| Head emits its artifacts, then exits 7 | Refused; only the already-preserved BASE copy remains |

The shared-target counterexample in my preceding review no longer applies. The failed-build case is also closed by the independently exercised status path.

## H2b — intermediate Cargo configuration: resolved for the scoped counterexample

The new `configs()` traversal and refusal run before either build. They cover the base checkout's parent through the candidate checkout root, inclusive. The actual previous counterexample at `target/three-team-policy/.cargo/config.toml` is now refused.

The fixture separately inserted both `config` and `config.toml` at each of:

- `target/three-team-policy/.cargo/`;
- `target/.cargo/`;
- the candidate root's `.cargo/`.

All six were refused. The empty chain passed. A separate common-ancestor configuration and Cargo-home configuration were both included by the actual hash loop.

Including the candidate root is conservative: a config at that root can also affect the head itself, so the explanatory “base only” wording is not literally correct for that endpoint. The refusal is nevertheless safe and resolves the earlier asymmetry; it does not weaken the check.

Nonblocking portability note: the hash loop expands the ancestor list with unquoted command substitution. It is not safe for config paths containing whitespace or shell glob characters. Quote/iterate those paths before supporting such checkout ancestry; the fixture and this closure use ordinary paths without those characters. This is not a claim of general hermetic-build certification.

## Preserved conclusions and scope

The census implementation, judge and Python test file are byte-identical to `523ceaa922e1c479f90aad113891dc8ce98ebf0a`. Their prior accepted conclusions stand: the parent observes the old child directly, names refresh per TID, child metrics are preserved, cadence overrides are refused, and the named root-input/environment counterexamples are closed. The 14 Python tests passed in the preceding review; they were not rerun for unchanged files. Rust census tests remain source-inspected, not locally executed.

The estimand, intervals, controls, no-harm rule and bounded attempts were not reopened. The 16/12 windows and independent-rounds caveat remain accepted. Census overhead remains an unseparated perturbation of the instrumented comparison; it is not proof of identical or zero overhead.

Expected: close baseline-copy and configuration-path defects without changing the experiment. Actual: both prior counterexamples are rejected or corrected by the new recipe. The evidence-path gap is closed within the stated scope; no performance gap has been measured. Mechanism learned: preserving bytes before the next build and examining the nested checkout's configuration ancestry are separate necessary boundaries. The next valuable step is the Coordinator's freeze after the remaining applicable review dispositions, then Louis's registered run under C5—not more algebra or another agent-run capture. Subsequent results and CI still decide qualification/adoption.

## Evidence

- `replay.py`: extracted shell boundary checks with controlled Cargo artifact fixtures.
- `replay-results.json`: captured successful and refused outcomes.
- `inputs/bench.just`: exact fetched recipe.
- `validation.json`: source SHA-256 values, unchanged-source comparisons and execution limits.

Migration checks matched the manifest digest and old branch SHA. The pinned coordination audit found zero unresolved misroutes and five already-resolved historic items. Research only, on the Researcher state branch.
