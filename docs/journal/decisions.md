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

## 2026-11-07: admitting the pipelined 3 × 3 default (0.2.6) on workstation evidence

- **Source:** Louis, in the Coordinator's session (RULES.md A4; C6 reserves the merge
  acceptance rule to him).
- **Observed:** 2026-11-07, about 09:50 UTC.
- **The question:** the three-team admission above applies to that policy only. CI's 4-core
  runners can't reach the 3 × 3 plan, so making the pipelined team with park and small the
  default at that plan needs its own admission. The evidence so far is one registered i9 run
  (`pipeline-levers/20261107T091926Z`): both levers against plain 3 × 3 at 10 threads, 1.072
  [1.053, 1.091]. No confirmation run has been made, and nothing at 11, 12, 16 or 20 threads.
- **What was decided:** "Confirm first, then ship", chosen from three options (the others were:
  ship now, with the post-release release-scaling run as the confirmation; ship on the
  existing run only). The option as put to him, verbatim:

> Before release, you run one frozen A/B of the new default against 0.2.5's at 10, 11, 12, 16 and 20 threads (about 30–40 min on the i9), on the precedent's terms. Release only if it passes. Slower, but follows the precedent exactly.

- **The precedent's terms:** at least 1.03 at 10 threads with a lower bound above 1, and no more
  than 2% slower at 11, 12, 16 and 20. They are applied exactly as the three-team registration
  defines them.
- **Applies to:** this change only (the pipelined default at the 3 × 3 plan). It is not a
  general change to the merge rule.

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

## 2026-11-06: a workstation run submitted by command counts as handed over (RULES C5)

- **Source:** Louis, relayed from ChatGPT on #145 (comment 6697379232, `FROM: louis via
  chatgpt`). He then approved the wording in the Coordinator's session (RULES A4, D1, D5).
- **Observed:** 2026-11-06, about 22:20 UTC.
- **What was decided:** RULES C5 is amended. A frozen recipe's run counts as evidence when Louis runs
  it himself and either hands its bundle over in session or runs it with the opt-in submission
  command (`just submit <recipe>`, `.just/submit.just`, `bench/submit.sh`). That command
  archives the bundle on the `evidence` branch and posts its identity and digest on #145. The
  run's identity is read from the bundle. The freeze, the provenance checks, the evidence test
  and the prohibition on timing by agents are unchanged. The command is named `submit`, not
  `publish`, on Louis's instruction: "publish" reads as publishing a package, and the Justfile
  already has `publish-rust`.
- **Applies:** RULES C5 and D5; PROTOCOL §6; CLAUDE.md, "Workstation recipes".

Louis's words in the relay, verbatim:

> in regards to what you said about auth, "automatic admission of the results as experimental
> evidence requires … amendment to C5" — i think this has probably been overly locked down,
> the thing about all these conditions on it being accepted is a bit much no? by running them
> with an automated provision to the coordinator via a PR comment in the inbox we are doing
> so… and the thing about "identifying the run" is just an implementation detail (if we need
> it identifying then we can just… put that in the comment or in the tarball itself, we
> should avoid requiring things that force us into a manual path)

The wording put to him in session, and his answer:

> A run of a registered, frozen recipe counts as evidence for its registration's question
> when Louis runs it himself and either hands its bundle over in session, or runs it with the
> opt-in publish command. Publishing archives the complete bundle immutably, on an evidence
> branch with its SHA-256, and posts its identity and digest on #145. Your running the command
> is the authorisation (A4). The run's identity is read from the bundle, not stated by you.
> This authorises no further runs, no changed method and no timing by agents.

> Approve as worded (Recommended)

And on the name, in session: "also hold up do not call it "just publish", that sounds like
publishing a package! be very careful not to tread on existing justfile naming conventions!"

Why there is no attachment: GitHub has no supported scripted way to attach a file to a comment
(`gh issue comment` has no attach option, and uploads go through the browser only). So the
bundle is a git object on the orphan `evidence` branch, linked at its commit. Pushes to that
branch start no workflow: the push triggers are master, main and `py-*` tags only.

## 2026-11-09: a breakthrough-chasing role is asked for (the Pathfinder, proposed)

- **Source:** Louis, in a Claude Code session of its own on the branch
  `claude/agent-swarm-fable-breakthrough-0u7j4d`. This is **not** the Coordinator's session, so
  whether it meets RULES A4 is for the Coordinator to record. Until then the entry is a request,
  not a decision in force.
- **Observed:** 2026-11-09, late evening UTC.
- **What was asked:** a new role for chasing step changes rather than marginal gains, built on
  the existing specs; and use of the workstation queue.
- **Applies:** RULES D8 (adding a role); PROTOCOL §9 (the queue); `docs/team/SPEC.PATHFINDER.md`
  and ROLES.md's "Pathfinder (proposed)", both written as proposed.

His words in that session, verbatim, the parts that bear on authority:

> You will find role specs in the repo for the autonomous agent swarm, including Researcher
> roles. Feel free to build on these to define a new role for this breakthrough chasing role, I
> think it is self explanatory that I am after more than just marginal gains here, I want to find
> the route to unlock seriously enhanced throughput via opportunities that have simply gone
> unnoticed and unexploited to decompress in smarter ways.

> There is an existing workstation queue mechanism you should feel free to use, as well as to
> simply present many potential avenues that you think could have outsized payoff.

And later in the same session:

> (FYI in case you didn't notice the workstation queue is primed and will take any new work you
> have to run and return the results via the repo comment relay mechanism with the tarball as an
> attachment, there are previous examples of this in the repo)

**What was done under it, and what was not:**
- the role's spec and its ROLES.md section were written, each marked proposed;
- a diagnostic recipe, `bench-phrase-knee`, was registered and frozen on that branch
  (`docs/journal/2026-11-09-phrase-structure.md`, section 6);
- **no queue entry was added and no ready tag was set.** PROTOCOL §9 gives both to the
  Coordinator, after the Researcher's PROCEED and green CI on the commit to run.

## 2026-11-10: the Pathfinder recorded as a role (RULES A4, D8)

- **Source:** Louis, in the Coordinator's session. The Coordinator asked him to confirm
  "Record Pathfinder as a role" (an inbox, `coord.py` routing, its ROLES and spec entries on
  master, a `team-state/pathfinder` branch). He replied:

  > I feel like we could potentially try to improve it and take the idea further idk, but yeah
  > we should integrate it into the swarm. Please do 1 and 2 yeah

- **Decided:** the 2026-11-09 request above is now in force.
  - **The role** is defined by `docs/team/SPEC.PATHFINDER.md` and ROLES.md's "Pathfinder".
  - **Inbox:** #314. **State:** `team-state/pathfinder`. **Routing:** `pathfinder` in
    `.github/coordination/coord.py`.
- **Unchanged:** its limits. No `src/` change, no merge, release or workstation timing of its
  own, and no self-certification. Its diagnostics go to the Researcher, and to the queue only
  through the Coordinator (PROTOCOL §9).
- **Louis's steer for its first follow-up:** take the phrase-structure idea further, and try to
  improve the copy walk (#313).
