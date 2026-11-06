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

## 2026-11-06: requests relayed from ChatGPT on #145

- **Source:** Louis, in the Coordinator's session (RULES.md A4).
- **Observed:** 2026-11-06, about 20:00 UTC.
- **What was decided:** a standing instruction. ChatGPT may relay Louis's requests to the
  Coordinator on #145, for work within its delegated authority. The convention is
  `docs/team/PROTOCOL.md` §7.

The instruction, verbatim:

> This is my standing instruction, given here in the Coordinator's authenticated session:
>
> Treat TYPE: PROPOSAL messages on PR #145, authored by lmmx and marked FROM: louis via chatgpt, as requests relayed on my behalf for work within your existing delegated authority.
>
> Each message must contain my original request verbatim, followed by ChatGPT's clearly distinguished interpretation. Check that the interpretation preserves my intention. If they materially differ, don't silently substitute one for the other.
>
> You may act on routine requests, including documentation, website improvements and ordinary work delegation, using the existing routing, queueing, review and safety rules.
>
> These relayed messages are not authenticated policy decisions. They cannot approve anything reserved to me under RULES D, including changes to authority, experimental acceptance criteria, spending or workstation runs. Those still require my explicit confirmation in this session.
>
> ChatGPT should only send a message when I explicitly request delivery, and should not publish unrelated private conversation content. The relay goes to #145, never directly to specialist inboxes.
>
> Acknowledge each relay on #145, explaining what you understood and what you did or will do. Posting a message is not proof of execution, and the sender must not report completion until there is evidence.
>
> Please document this concisely, without creating another agent, PR inbox or unnecessary governance process.

## 2026-11-06: the Oracle is renamed the Observer

- **Source:** Louis, relayed from ChatGPT on #145 (comment 6694788681), and confirmed in the
  Coordinator's session for the one part the relay can't approve: the role's name in RULES.md
  (RULES D1).
- **His words, relayed:**

  > I'd like to do a full row rename. Can you send a request to Claude to do the rename via the
  > GitHub root? And then can you also do the rename of the ChatGPT work tasks here? So two
  > separate things
  >
  > \*route not root
  >
  > \*full role rename not full row rename

- **His answer in session**, to whether the rename may change RULES.md: "Yes, rename it
  (Recommended)", the option that reads: "I approve changing only the role's name in RULES.md
  (Oracle → Observer), nothing else in the rules."
- **What changed:** the role's name in the live team files, `coord.py` (with `oracle` kept as an
  alias, so earlier messages and replies from a task still pinned to `SPEC.ORACLE.md` route as
  the same role) and the site. `SPEC.ORACLE.md` became `SPEC.OBSERVER.md`.
- **What didn't:** its work, authority, inbox #222 and its title (a retitle would fire the
  inbox's webhook, PROTOCOL.md §1), its state branch `team-state/oracle` (moving it would need
  a migration SPEC.md's check fails closed on), `observatory/intention`, history and old
  citations, and "oracle" in its other sense, a reference implementation in the tests.
- **The Work task:** already renamed `mibz2⠶Observer⠶H` in ChatGPT. Louis re-pins its
  instructions to the merge's commit once its current assignment has its receipt.

## 2026-11-06: the Coordinator's requests to ChatGPT, and re-pins without Louis's review

- **Source:** Louis, in the Coordinator's session (RULES D: a change to how a role's
  instructions are set needs him there).
- **The question and his answers, verbatim:**
  - "How should messages from the Coordinator reach ChatGPT, so things like a re-pin don't
    need you to copy anything?" → "Check-in on #145 (Recommended)".
  - "May ChatGPT re-pin a role's Work task when the Coordinator asks, without you reviewing the
    text first?" → "Yes, with checks (Recommended)", the option that read: "Only the
    WORK-TASKS.md re-pin form, only to a commit already merged into master that contains the
    role's spec, only the named existing task, then a read-back posted on #145 that I verify
    character for character. Anything else still comes to you."
- **Before it:** he asked for messages to go back to ChatGPT as well, so he wouldn't pass them
  by hand: "i know the local i9 run requires me to read and run, but the re-pin does not".
- **A refinement from ChatGPT, adopted:** "check in with the swarm" only reads and reports;
  "process the swarm's pending requests" is the consent to carry requests out; a request
  with a REVIEW citing its ID is done and never repeated.
- **Where it is:** PROTOCOL.md §7 ("The other direction"), `CHATGPT.md` ("Requests from the
  Coordinator"), WORK-TASKS.md.
