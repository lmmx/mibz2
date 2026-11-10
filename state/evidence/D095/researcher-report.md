# Replication final and combined diagnostic revision 3

Inbox 6763960537; queued briefs 6763823117 and 6763780472. Researcher, 2026-11-10.
Operating pin a2da49638ac54d7763de1c167bd054f67ffdc763. Research only; no decoder implementation, build, timing, dispatch, merge or policy adoption.

## 1. #316: REVISE before freeze

Reviewed registration and judge at **21938d1273d1d1bf3ca0f4bb791333cfa1aa4666**, and corresponding tooling at **47bd387376db701906229ba79c7a7e4992d38f3f** (#333). The mixture disclosure is accurate. It does not repair admission. I withdraw any reading of my earlier 12 × R240 planning recommendation as sufficient evidence for a 5% mean-noninferiority gate under the now-disclosed mixtures. Its normal-model power calculation remains conditional, not disproved arithmetic.

### Statistical counterexample, not another power calculation

Independently replayed `mixture.py` against the archived links: all eight scenario results match `mixture.json`. The as-observed maximum is 36.07% (runs/4 wall); without Intel, 39.30% (runs/4 CPU); Intel at 80%, 18.57% (records/4 CPU). Same-variance normal controls remain near 5%. These are **sensitivity models of 10-round neutral contrasts**, not measurements of the R240 candidate programme's false-positive rate. Neither treating them as actual coverage nor dismissing them because R240 is larger is justified: averaging removes within-dispatch noise, not necessarily a persistent dispatch effect.

Let B = log(1.02), a > 0, and an iid dispatch have X = B − a with probability 18/19, X = B + 18a otherwise. Its mean is exactly B. With m rare draws among k, the upper bound minus B, divided by a, is

    −1 + 19m/k + t(.95,k−1) × 19 sqrt[m(k−m)/(k²(k−1))].

Exact binomial summation, not Monte Carlo:

| k | accepted rare-draw counts | false acceptance at the mean boundary |
|---:|---|---:|
| 12 | 0 | 52.267% |
| 24 | 0 | 27.318% |
| 56 | 0, 1 | 19.908% |
| 64 | 0, 1 | 14.314% |
| 128 | 0–3 | 9.056% |
| 256 | 0–8 | 7.449% |

At k56 the probability of *missing* the rare draw falls below 5%, but accepting with **one** rare draw keeps size near 20%. Thus sizing k from `(18/19)^k < .05` is also wrong. A small continuous perturbation preserves the all-ordinary acceptance event; zero sample variance is not the essential issue. At a=0.005 the ordinary and rare ratios are about 1.015 and 1.116, not an astronomical tail construction.

No finite k gives a distribution-free mean guarantee over unrestricted finite-mean populations: make the harmful component rarer and larger while retaining the boundary mean. Resampling the empirical support, trimming, or silently deleting the troublesome cells does not resolve this missing-tail problem. A signed-rank test also needs its own symmetry/location assumptions; it is not a generic asymmetric-mean repair. The IUT remains correct **if each component is level controlled**; cross-cell multiplicity is not the defect.

### Prospective rule and next decision

**Keep every one of the 42 cells. Do not grant a mean-noninferiority verdict from this t bound where its dispatch model is unsupported. Disclosure alone is not my PROCEED.** No cell is exempted based on its inconvenient neutral history, and no qualification data select k, trimming, weights or the test.

For a mean-preserving repair, require a justified population tail/variance envelope before freeze. One explicit sufficient rule is an externally justified bound `Var(D_c) ≤ V_c`, independent dispatches, and

    U_c = exp(mean(D_c) + sqrt(19 V_c / k)) ≤ 1.02, for all 42 cells.

Cantelli's inequality gives one-sided size ≤ .05 without a normality assumption. This is a conservative benchmark, not a claim that these data supply V_c: the fitted τ, maximum of 19 observations, or qualification sample variance cannot simply be substituted. A sharper robust-mean method may replace this benchmark only with its tail assumptions and coverage argument explicit. There is no such envelope in the reviewed package. **Current mean route: NOT SHOWN; no freeze.** This holds this qualification design, not the service line.

If the team instead wants a typical-dispatch adoption policy, make the change of protected quantity explicit under C6, with the Guardian-reviewed decision record, before its data. A concrete alternative for review is **k24, R240, ≥17 strictly sub-margin dispatch means in every cell**, ties counted as failures. This tests `P(D_c < B) > 1/2`, not `E[D_c] < B`: its exact boundary size is 0.0319573 under iid Bernoulli signs, with no magnitude or symmetry assumption. At a true success probability .90 in every cell, individual power is .998316 and the 42-cell union-bound lower power is .929267, without cross-cell independence. The 12-dispatch alternative (≥10 successes) has .019287 size but only .889130 individual power at .90; it cannot inherit the old .934 all-cell power claim.

The k24 alternative roughly doubles the stated runner-minute budget to 5166 and doubles shard-job exposure to 216. It **does not control harmful-tail severity or the mean**, and neither the .90 success probability nor economic acceptability is established. Keep all raw magnitudes, failures and per-model summaries; a median PASS must never be presented as a repaired mean PASS. My recommendation is to retain mean protection unless the team deliberately adopts that different risk policy; the rank construction is a fully specified candidate for that decision, not authority to weaken it. No reclassification of old verdicts.

Prior art: the [NIST sign-test reference](https://www.itl.nist.gov/div898/software/dataplot/refman1/auxillar/signtest.htm) supplies the exact binomial construction and discrete critical values. The protected probability and the two-point mean counterexample above are derived here. A scalar binomial proof requires stable independent dispatch draws; changing host mixes or serial dependence remain separate limitations.

### Independent tooling blockers

The committed 30 judge tests and 12 gate tests pass. Additional synthetic probes expose an admission gap at both reviewed tooling pins:

1. **Missing job times can conceal an overlap.** `window()` (service_tg.py:1045) discards missing/unparseable timestamps. In a complete fixture, set one first-dispatch shard completion to +2h; the second dispatch starts at +1h. The programme correctly rejects overlap. Delete only that shard's completion timestamp: `ni_judge` and `replicated` now accept admission, taking the other shards' +30min maximum. Unknown completion has become evidence of non-overlap. Require every relevant job/attempt timestamp, finite and ordered, before computing the extrema; the fixture must stay INVALID after deletion.
2. **The last completion is unchecked.** Delete all job completion timestamps from dispatch 12: its `completed` becomes null, yet dispatch and programme admission remain valid. §2.11 explicitly rejects unknown times. Validate every dispatch window, including the final one, independently of adjacent-pair checks. The direct helper also admits nonfinite numeric times unless explicitly checked; the concrete archive-level witnesses above do not depend on NaN.
3. **Condition 7 still needs a defined validated input contract.** `confirm()` accepts `valid: true, runs: []` on both inputs and reports a supplied flag confirmed; it also still supports single-dispatch shapes. This is a synthetic interface witness, not a claim that the current CLI normally emits empty programmes. Before condition 7 is frozen as two programmes, require that schema, the frozen k/R/head, nonempty unique run IDs of the proper cardinality, disjointness and derivation from admitted raw programmes. Resolve the text's residual “confirmation dispatch” versus “two programme verdicts” explicitly. No confirmation data enter condition 2.

The first two are direct violations of the proposed admission contract, independent of the statistical objection. Fixes need negative tests; passing current fixtures does not close them. `probes.py` reproduces both with actual manifest/judge paths, `check_git=False` and R16 synthetic fixtures. It does not certify real artifact identity or exercise the workflow in Actions.

## 2. Combined team-ports diagnostic: PROCEED with revision-3 design

Reviewed **a9e0e5c7377be7c149ace0f65d22bafe1744385f**. This closes my design objection to the conflicting checker-exit rules. **PROCEED to implement the bounded diagnostic as specified; not yet freeze or run.** The §8a implementation, every §12a test, archived dry run and remaining Guardian/B5 conditions are still explicit pre-freeze obligations. This verdict does not assert that unwritten code enforces the design.

The checker must be git blob **eb7c392d2d766e870de3e14a31b5fe940787dd3f**, taken from the registered source and hash-checked, rather than the stale checker in the baseline decoder checkout. I ran its 20 tests and independent synthetic ledgers: balanced helpers 1/1 produce VALID/all hold/ENGAGED/exit0; 1/99 produce VALID/(b) only fails/NOT ENGAGED/exit1; 0/0 likewise exit1 with two zero-work calls. Each agrees with §8a. These latter outcomes are presence/integrity observations, not proof of useful helper work, balance, capacity or speedup.

The complete-output grammar, predicate/outcome/exit consistency, empty stderr, timeout/signal rejection and explicit (a)/(c)/(d) requirement distinguish a balance-only verdict from a crash. Do not implement that distinction with exit1 acceptance alone. Both exit0 and exit1 require complete parsing. Required-predicate failure, missing terminal record, duplicate/reordered/extra output, truncation, traceback, timeout, signal and contradictory exit/outcome must fail the adjudicator; successful checker tests are not adjudicator tests.

On the queued brief's new-gate question: the copied-blob hash check enforces the chosen checker identity; empty stderr enforces this prospective complete-verdict contract. I accept both as artifact/execution admission checks, not new performance or useful-work minima. They can reject an execution and must therefore be fixed before data, as this draft does; they must not be retroactively applied to rewrite an earlier registered verdict.

The independent per-arm residual interval `[ΔD_d − U_base, ΔD_d + U_cand]` follows directly by subtracting residuals each bounded separately between zero and its U. It requires no coupling assumption. C1–C5 remain descriptive; W1–W7, output identity, inventories, deadlines, no rerun and B6 stops remain binding. Continuing an independent complete-input step must not consume a partial or stale predecessor.

## Outcome, limits, next value

Expected: a freeze-ready replication design and a resolved diagnostic exit contradiction. Found: the contradiction is resolved in the diagnostic specification; replication still lacks robust mean coverage and loses admission evidence when timestamps disappear. No production performance gap was closed by this review. The statistical mechanism is rare large dispatch effects missed or underweighted by Studentization; the tooling mechanism is treating an incomplete maximum as a complete one. The combined diagnostic has the next information value without waiting for the separate CI acceptance repair: implement and dry-run its already specified adjudication, then return the exact implementation for freeze review. Do not spend the 2583-runner-minute qualification budget on the present bound.

Artifacts: replay output, exact counterexample/probe script and results, source digest manifest. Eight compared operational files are identical at #316/#333 pins (judge, gate, their tests, stamp, workflow, target checker and its test); the recipe test file is not claimed identical. No real CI programme or native decoder run was performed. The pinned 19-link archive is a planning input, not newly collected evidence.

Reproduction: retrieve `links.jsonl` from **ebb793af492f841e494e0d8bf051dbb0bcda8317** (blob `0bcc2f6bffb8a958296aceecd56973acf0ecff55`), `mixture.py` from `docs/journal/2026-11-10-tg-ni-replicated/` at the #316 pin, and the `bench/` judge/fixture dependencies at that same pin. Retrieve the walk-stats checker and test from **8b62e14d3e8e0bfd4b07ae8503ca89cbcbba5e9c**; its blob is checked above. With these under the report directory, run `python mixture.py links.jsonl mixture-replay.json` and `python probes.py`. The probes use NumPy/SciPy dependencies already present in the environment. `manifest.json` gives the exact input digests. No dependency installation was performed. The #318 workstation result and submitted bundle were not independently re-audited in this review; they do not repair the CI mixture argument.
