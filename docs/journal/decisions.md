# Decisions that authorise, narrow or change policy

One entry per decision (RULES.md E2): its source, the words verbatim, the time observed, and
the rule applied. Routine messages are not copied here; #145 is their record.

## 2026-11-06: the team's standing rules and MODE.md

- **Source:** Louis, in the Coordinator's session (RULES.md A4).
- **Observed:** 2026-11-06, about 13:00 UTC.
- **What was decided:** Louis relayed the response below, then confirmed it in his own words.
- **Applies:** RULES.md, all of it (A–E and S), in force from the re-pin; MODE.md M1–M3; the
  cutover in `docs/journal/2026-11-06-team-neutral.md`.

The relayed response, verbatim:

> Approve the whole RULES.md table, including the substantive adaptations and retirements.
>
> For MODE.md:
>
> - **M1:** Yes. Refer to RULES.md and STEWARDSHIP.md as the live authority.
> - **M2:** Keep "Work? Ir — not time." It is part of my intended register, not an instruction-count acceptance gate.
> - **M3:** Widen to "faster production decoder, released". Inverse BWT remains an important subgoal, not the boundary of the team's mandate.
>
> Proceed with the migration under the reviewed cutover procedure.
>
> Preserve the separation we've established: live team machinery on master, operational state on its own branches, and the Oracle's behind-the-scenes hierarchical intention record on `observatory/intention`, with independent orphan history.
>
> Keep the #220 registration unchanged. I haven't created the Oracle Work task yet, and I'll withhold the i9 results until the Oracle is configured and verified.
>
> Once the migration is complete, give me the final instruction pin to update the three existing renamed Work tasks and create the Oracle.

His confirmation, verbatim:

> (i agree with the chatgpt answer here)

## 2026-11-06: the Oracle explains; its hierarchy stays behind the explanation

- **Source:** Louis, in the Coordinator's session (RULES.md A4).
- **Observed:** 2026-11-06, about 16:45 UTC.
- **What was decided:** the Oracle's output contract changes. It writes a plain-prose account
  (`current/account.md`), answer first, for a reader who hasn't been following. Its levels stay
  as the evidence-bearing representation behind it, and the docs page shows the account first.
  Then one re-pin of the Oracle alone, and one rebuild. MODE.md, the team's authority and the
  observatory branch's architecture are unchanged.

Louis's question, verbatim:

> So I think fundamentally the problem here is that we show more of the structure of the answer than the answer itself, what you highlighted was merely a symptom of this malaise. Likewise with the jargon and internal codenames ("L0 etc"), no human explanation would ever put it that way. Likewise the content itself is not written to be intelligible to a human reader ("Louis's held i9 results"). What would you do to remedy this malaise thoroughly, fervently, durably?

The brief he relayed with it, its operative lines verbatim:

> **Please revise the Oracle's output contract and the Behind the Scenes page around the principle: the product is understanding, not the representation of understanding.**
>
> I want one coherent change to the Oracle specification and its presentation, then one Oracle-only re-pin and one rebuild. Keep MODE, the swarm's authority, and the observational branch architecture intact.
>
> The acceptance test is simple: when I open the page, I should understand what the team is doing and why before I've had occasion to think about how the Oracle represents that knowledge.

## 2026-11-06: admitting the three-team policy on workstation evidence

- **Source:** Louis, in the Coordinator's session (RULES.md A4; C6 reserves the merge
  acceptance rule to him).
- **Observed:** 2026-11-06, about 19:45 UTC.
- **The question:** CI's paired bench can't reach the changed plan (its runners have 4 cores),
  so the policy can't meet the usual rule (an end-to-end figure whose 95% interval excludes
  no change) on CI.
- **What was decided:** "Workstation A/B (Recommended)", chosen from four options (the others
  were: opt-in only; a larger CI runner; hold). The option as put to him, verbatim:

> Benefit shown on your workstation, not on CI. CI must show no regression, plus correctness tests that fake a 10-core topology. You then run one frozen same-thread-count A/B of new vs old default (run plus confirmation): ≥1.03 at 10 threads, lower bound >1, and no more than 2% slower at 11, 12, 16 and 20 threads.

- **Applies to:** this policy change only (`docs/journal/2026-11-06-three-team-policy.md`). It is
  not a general change to the merge rule.
