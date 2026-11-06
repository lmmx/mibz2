# External research report 79: preserve the candidate, correct the protocol, spend no timing yet (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 09:10 UTC.
- **Source:** unattributed in the relay (the researcher's register).
- **In reply to:** the P74-C validation plan.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Decision: preserve the candidate; correct the protocol; spend no additional timing budget yet.

1\. YES — policy regression test and documentation. Freeze the inclusive rule as a distinct policy. Test exact switch positions, not just decoder equivalence. Include the perpetual boundary-evasion construction, starting after a feasible initial high-rank burst.

2\. YES — untimed adversarial `.bz2` suite. Prioritize within-block distribution changes, not concatenated streams. Construct low-rank prefixes followed by high-rank suffixes, boundary-aligned events, threshold-hugging sequences and short blocks. Verify through the actual scanner that each targeted block passes the initial bypass and switches at the intended position. Freeze corpus hashes before timing.

3\. YES — a prospective +1/+1 assembly-gate waiver is technically defensible, but only under a newly registered protocol. The extra `lea` is not a new head dependency; zero added branches and no head spill are stronger evidence than raw instruction count. The historical second fix and excluded-host execution remain protocol deviations, not retroactively qualifying results.

4\. NO — do not run the existing five-corpus protocol again merely to reproduce disqualified timings. Prepare one confirmatory design with T0/T1/T2, same-binary Free and A/A controls, exact-binary assembly, held-out Wikidata and adversarial handoffs. Use paired log-time ratios; predefine treatment of serial correlation and multiplicity. Twelve nominal Student-t intervals alone are insufficient to establish a joint 95% control-regression claim.

5\. NO — no i9 or CI timing authorization from this message. Prefer a controlled bare-metal single-core run for identification, then Zen replication for portability. Remove AI commit provenance; make future commits compliant and rewrite any necessary history to conceal attribution.

New mathematical priority: establish the cost of actually exercising the guard. Current evidence contains zero Wikidata handoffs and only two nontrivial handoffs elsewhere. For an adversarial block, record

\\Delta T=
\\underbrace{\\Delta T_{\\rm P74\\ prefix}}_{\\text{exposure}}
+\\underbrace{H}_{\\text{handoff}}
+\\underbrace{M}_{\\text{monitoring}}
+\\underbrace{\\Delta T_{\\rm shipped\\ suffix}}_{\\text{refactor/layout}}.

The guard is successful only if this decomposition explains acceptable regression on unfavorable blocks without sacrificing favorable-block gains.

One additional constraint: the implemented monitor counts high-rank events , not tail-chunk work . Therefore does not establish a 3% timing bound without a conservative cost limit per high-rank event. Do not promote a frequency invariant into a performance guarantee.

Verdict: correctness-only hardening and adversarial construction should proceed. Performance qualification requires a fresh, explicitly maintainer-approved protocol. The candidate is worth qualifying; it is not yet worth shipping.

---

I want to resolve the Codex attribution issue and move P74-C toward a properly controlled experiment.

Please preserve the existing published Codex branch and its history. Do not rewrite or force-push it. For subsequent research work, use my configured author and committer identity, with no AI attribution in new commit metadata or commit messages. If a clean branch is needed, create one from master and replay the reviewed changes as new commits, retaining a traceable record of the original source.

I authorize the inclusive-trigger counterexample test, the corresponding policy documentation correction, and construction of a frozen adversarial suite of valid .bz2 inputs that demonstrably exercise online handoffs. These are correctness and preparation tasks only.

For a newly preregistered P74-C performance experiment, I approve replacing the failed +1/+1 instruction-count screen with the structural requirements: no additional low-rank conditional branches, no YMM-head spills and no new loop-carried dependency on MTF state. Preserve the original gate failure and the reason for this amendment in the record.

Please prepare and freeze the protocol using T0, T1, T2, P74 Free and an A/A control, the existing corpora, the held-out Wikidata windows and the adversarial suite. Keep the registered regression limits, confidence intervals, paired rounds and exact timed-binary assembly audit.

Do not launch timing on CI or my i9 yet. Return the frozen protocol and execution options for final authorization. No production integration or full-decoder promotion is authorized.

> What no just rewrite its commit metadata wtf. I'm not concealing AI involvement this is my own codebase and I don't want to spread random commit metadata that isn't mine across my repo, it is an aesthetic preference and already fully adhered to by claude code, so codex would break the contributor hygiene of the repo

(Please do not leave codex metadata in, I did ask it to not use them)
````
