# External research report 18: the researcher's burn-down status and completion standard (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 17 (`2026-10-30-external-report-17.md`). It was
written against the PR branch before the ledger's latest push. It still cites the
register's original totals: 31 answered, 6 conditional, 47 open.

**What it contains:**
- **The researcher's estimate:** 15–20 of the 47 open rows have been materially
  narrowed.
- **A completion standard:** every row ends answered, conditionally unresolved (the
  missing evidence named, with the reason it can't be inferred), or superseded.

The coordinator adopted that standard in `2026-10-30-research-portfolio.md` §9. The ledger
there reconciles all 47 open and 6 conditional rows by number, apart from the
literature rows, which a read-only literature agent is working on.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (card labels flattened) and is kept as received.

## The status report, as received

````text
## mibz2 research burn-down status

We're making substantial progress on the science, but the official question inventory has not yet been fully reconciled. We have reduced several major uncertainties to proofs, exact structural counts and measurable cost conditions.

Original research-question baseline

# 31

Answered

# 6

Conditional

# 47

Open

84 deduplicated questions in PR #117's original register. These are the last verified official totals, not a fresh recount.

### What we've actually burned down since that baseline

| Area                                   | Progress                                                                   | Status                         |
| -------------------------------------- | -------------------------------------------------------------------------- | ------------------------------ |
| Ψ composition mathematics              | Correct partition theorem, gap identity and record bounds                  | Established                    |
| Ψ⁸ construction                        | Run-native, no-search, no-sort interval composition                        | Algorithmically established    |
| Cooperative segments                   | Construction-time caps; 10,188 randomized segment checks                   | Correctness validated in tests |
| Ψ⁸ storage                             | All 233 full Wikidata blocks pass 3.6 MB gate                              | Measured pass                  |
| Other corpora                          | Random, records and source-code controls exceed storage budget             | Measured negative              |
| Ψ⁸ construction traffic                | \~23 B/output byte versus \~5 for current build                            | Adverse cost model             |
| MTF batching / P74                     | Exact algebra and actual code constraints reconciled                       | Narrowed                       |
| Compact Ψ and memory-level parallelism | Several impossibility-of-inference results and exact structural identities | Partly resolved                |
| Full-decoder breakthrough              | No faster implementation demonstrated                                      | Open                           |

## The biggest change

At the start of this exercise, N1 was a speculative idea about composing BWT transitions.

It now has a complete mathematical construction and a successful Wikidata storage census.

However, the new traffic model is a serious challenge:

CURRENT T-VECTOR BUILD

# \~5 B/B

Modeled memory traffic

PROPOSED Ψ⁸ BUILD

# \~23 B/B

Modeled memory traffic

The new builder moves approximately 4.6× as many bytes. Its more structured access pattern might compensate, but that is unmeasured.

N1 is therefore mathematically credible, storage-feasible on Wikidata, and economically unproven.

## Where the overall burn-down stands

I've checked the latest PR #117 branch. The official register still reads 31 answered, 6 conditional, 47 open. It has not yet incorporated a revised question-by-question tally.

My assessment is that roughly 15–20 of those 47 open questions have been materially narrowed, although many remain open under their original performance-oriented criteria. That is an estimate of research progress, not 15–20 additional closures.

We've worked through four major clusters:

- Inverse BWT mathematics: the strongest progress, with exact constructions and independently checked proofs.
- Compact representations: clarified their information budgets, dependencies, worst cases and fallback requirements.
- CPU memory-level parallelism: ruled out several unsupported conclusions from timing and CI results.
- Front-end and decoder accounting: distinguished actual work reduction from buffer reuse, pipeline limits and benchmark-boundary artifacts.

The main clusters still needing systematic attention are the six outstanding literature questions (#28–#33), the remaining CPU-specific measurements, and the unresolved application/scheduling questions.

### What completion means

I would consider the burn-down complete when every original question has one of three defensible dispositions:

1. Answered: proof, counterexample, primary-source finding or relevant measurement.
2. Conditionally unresolved: precisely identified evidence is missing, with a reason it cannot be inferred from existing work.
3. Superseded: another question or result fully subsumes it, with no independent uncertainty remaining.

We should not force hardware-dependent questions into the answered column without measurements. But we also should not leave questions open simply because an old journal never recorded the answer.

Current verdict: strong progress, incomplete inventory reconciliation, and no demonstrated decoder breakthrough yet.

We remain in burn-down mode. The next useful action is to reconcile our established results against all 47 open rows, then concentrate on those that still contain genuinely independent scientific uncertainty—not restart implementation planning.
````
