---
name: coordinate-blog-post
description: Use when the coordinator commissions, publishes or revises a mibz2 blog post written by another agent: briefing the drafting agent, publishing as soon as the post is fact-checked and renders, then two rounds of simulated-reader (exegenius) review by the Researcher, revised in flight on the live post. The writing itself is write-blog-post's; the render check is publish-page's.
---

# Coordinating a blog post

`write-blog-post` says how a post is written, and `publish-page` how it is checked once
rendered. This skill is the coordinator's side: who does each step, in what order, and what
each one is given. It records Louis's instruction of 9 November 2026: every post gets two
rounds of exegenius review by the Researcher, and the review must check that the simplest
understanding and the key takeaways land in the opening.

**Publication is not blocked on review** (Louis, 9 November 2026: "i want this asap … the
researcher reviews do not block publication, we re-write in flight (it goes out before
finalisation)"). The post is published as soon as its facts are checked against the records,
`publish-page` passes and CI is green. The two review rounds then run on the live post, and
their revisions land as follow-up PRs.

**The PR opens first, within minutes** (Louis, 15 November 2026). When the text already exists,
because it was relayed or drafted elsewhere, a PR carrying it must be open within minutes of
the hand-over.
- **The order:** commit the text, push, open the PR. Then redraw the figures, run
  `publish-page` and fix the render, all as further commits on the open PR.
- **The PR body** lists what is still unfinished: figures to redraw, checks to run.
- **Never** spend a long local session (figures, render checks, screenshots) before anything
  is pushed. On 15 November an agent worked 20+ minutes locally on a relayed, already-written
  post, and Louis was left asking where the PR was.
- **Every brief to a drafting or publishing agent says this:** "Push and open the PR first,
  then iterate on it."
- **A new post always goes through a PR, never a direct commit to master** (Louis, 18 November
  2026: "we only directly commit when we make amendments, we are meant to open the PR asap").
  - *Direct commits to master* are only for amendments to a post already published, such as a
    retitle or a small edit, and only when Louis asks for one.
  - *A brief never says "don't open a PR"* or "the coordinator will push". On 18 November the
    coordinator's own brief said exactly that. The post was built in full on a branch with no
    PR, and then a push straight to master was attempted.

## 0. Two transmissions, in files, never a précis

This is Louis's instruction of 22 November 2026: a post built from a précis of a long discussion
comes out minimal, and loses the argument's force and purpose. So when a post comes out of a
discussion, the brief goes over in two transmissions. Both are files on a `brief/blog-…` branch,
under `docs/briefs/<date>-<topic>/`. That branch is never merged. A message to the writer only
points to the files and says what to read first. A message carries a summary; a file carries the
material.

1. **Transmission 1, the source material, kept whole** (`01-source-material.md`):
   - **Louis's words verbatim, in order.** Light bracketed glosses are allowed for speech-to-text
     errors (for example [Deleuze] for "Louise").
   - **The coordinator's answers** that Louis responded to, verbatim.
   - **Relayed commentary** (ChatGPT) Louis forwarded, as verbatim excerpts that keep its
     distinctions.
   - **Any rejected outline**, with why it was rejected.
   - **The evidence inventory:** every figure with its source.

   Send it at once, so the writer starts with the full material and version 1 can go out fast.
2. **Transmission 2, the editorial analysis** (`02-editorial.md`). The coordinator works through
   what the latest feedback changes: what the earlier framing got wrong, the sharpened thesis, title
   options, the architecture section by section, the examples, how to handle any analogy, and the
   process. It adds to transmission 1 and doesn't replace it. Where they disagree, Louis's words win.
   It drives a substantial revision of the live post, a restructure on its own branch and PR, not
   cosmetic polish.

The post never quotes the chat, Louis's included. His words are in the brief so the argument keeps
its intensity, not for quotation. For an ordinary results post from a journal or a bundle, the
brief below is enough.

## 1. Brief the drafting agent

The drafting agent works in its own worktree, on a `docs/blog-…` branch from master. Its
brief names:
- **The source:** a relayed report, a journal or a result, and Louis's verbatim words if the
  request came through the relay (`FROM: louis via chatgpt`). Acknowledge the request with
  one LOG on #145.
- **The primary records** every number is checked against: the journals, the registrations,
  the evidence bundles. Any figure that can't be traced is flagged in the PR, never printed as
  established.
- **The material's centre and our part in it** (write-blog-post §1a). The brief states the
  material's central idea in one sentence, in its own terms, and lists our records that
  respond to it: launch briefs, handoffs, journals. A brief that condenses the material
  hands the agent a summary; it links the full material and says to read all of it.
- **The skills to follow,** in order: `write-blog-post`, then `publish-page`. One check pass:
  build, `check_render.py`, one `look_pages.py` run and a look at the figure screenshots,
  then the PR. Polish lands in flight (§4).
- **Takeaways first** (§3 below). Before drafting, the agent writes in its plan:
  - the simplest correct explanation of the result, in two or three sentences;
  - the key takeaways, as a short list.

  The PR body says where in the post each takeaway first lands.
- **The dek and the sentences:** `write-blog-post`, "The dek, and the sentences". The dek must
  pass `docs/scripts/check_prose.py` and carry a little joy. The PR reports the body's flagged
  share.
- **For a results post, results only** (Louis, 22 November 2026): the brief gives the
  drafting agent the findings and their evidence, not the team's deliberations. Don't hand
  over the maintainer's chat messages as material to quote, and don't frame the commission
  around a process or policy story. The title is technical, never a slogan
  (`write-blog-post`, "First: written for serious researchers and programmers").
- **The brief's numbers are a starting point, not the record** (22 November 2026): the
  drafting agent re-checks every figure in the brief against the newest evidence before
  drafting. The memory post's brief said "no loss of speed"; the later i9 release-scaling run
  had already shown 4% faster at 10 threads.
- **Visuals:** a figure for each result, comparison, architecture and trade-off, and for
  the key takeaways where it helps. Dense prose where a figure would carry it is a
  failure mode.
- **Causal visuals, by analogy** (Louis, 10 November 2026). Figures that only draw the data
  (strips, bars, boxes and arrows) leave a post hard to access. Brief the Researcher, or the
  drafting agent, for figures that show *why* each step of the argument holds, through an
  analogy a programmer gets at a glance (the pit-crew post's race track). The brief lists the
  causal steps in order, asks for each analogy's mapping and where it breaks, and says the
  figures must read at phone width. Visuals never block a prose revision: they land in their
  own follow-up.
- **Grounding** (Louis, 10 November 2026: "death by in medias res"). A follow-up post links
  the post it builds on in its first paragraph and recaps what it needs from it. Agents'
  names stay out of research posts; the work is the project's.
- **The plan's goal questions and held-out goal,** recorded in the PR body for the
  reviewer.
- **The rules:** Louis's identity on commits, no AI attribution, the footer stripped from the
  PR body, no merge.

## 2. Two rounds of review by the Researcher

Once the post is published (§4), route a CHECKPOINT to the Researcher with `coord.py route` (QUEUED on
#145 while it is busy). Each round asks for:
1. **The independent baseline, before reading the post:** from the primary records alone,
   the simplest correct explanation of the result and the key takeaways.
2. **The comparison:** where each of those lands in the post. Anything that arrives late,
   or that leaves the reader unable to follow what comes before it, is a finding.
   - The simplest understanding belongs in the opening.
   - The takeaways belong in the opening or section 1, as soon as they can be stated.
     They are never withheld for a later section.
3. **The simulated-reader pass** (`write-blog-post`, the protocol's steps 5 and 6):
   - **dense prose:** each passage that carries a comparison, numbers, a structure or a
     trade-off in words alone, where a figure would make it land (`write-blog-post`,
     "Dense prose without visuals is a failure mode"). Each one is a finding, with the
     figure proposed;
   - the goal questions answered from the rendered text alone;
   - stuck points recorded by level (`symbol`, `statement`, `step`, `chain`, `beyond`);
   - fragmentation, collapse and missing steps;
   - every fact checked against its source.
4. **A verdict:** PROCEED, REVISE (with the findings) or STOP.

**Between the rounds,** the drafting agent revises the live post for the findings, in a follow-up PR (merged on green, with publish-page rerun). It restructures rather
than appending, reruns `publish-page`, and replies in the PR with where each finding was
addressed.

**Round 2** repeats the whole of round 1 on the revision, adding the held-out goal question.
A converging revision has fewer stuck points and few new ones. If round 2 still finds the
takeaways late, or stuck points at the `chain` or `beyond` level, it is a REVISE, and
another round follows. "Two rounds" is the minimum, not a cap.

## 3. What "takeaways first" means

The reader should hold the simplest correct picture of the result before the details, so
that every later section reads as support, not as suspense.
- **The opening** states what the thing is, what was found and its scope, in the
  simplest words that stay true (`write-blog-post`, "The opening").
- **The key takeaways** are stated as soon as they can be: in the opening, or in section 1
  at the latest. That includes the caveats a reader needs to weigh them, such as
  "registered" against "reported, never judged", one machine, and not the default.
- **Later sections** show how each takeaway is known. They add detail and evidence, not new
  conclusions the reader needed earlier.

## 4. Publish first, then revise in flight

**Version 1 merges within minutes of being pushed** (Louis, 22 November 2026: "you are meant to
get blogs out FAST and have failed"; `write-blog-post`, "Iterate, ship, re-ship"). Merge the
post's PR as soon as all of these hold. Don't wait for the Researcher, and don't wait for CI's
`look` job or other CI runs on a docs-only post PR:
- every figure in version 1 is checked against its primary record, and any that can't be
  traced is flagged in the PR, never printed as established;
- the strict build and `check_render.py` pass locally on the pushed head, and the PR says so;
- pre-commit.ci and `check-commit`, if they have already reported, aren't red.

Screenshots and `look_pages.py` run on the live post; their fixes follow as small PRs, merged
at once. If a second post PR is open, merge the first immediately so the second's conflict
(blog index, nav) is resolved once, not after both wait.

Merge with `coord.py head-sha` before and after (`expectedHeadSha`), check the live page once
deployed, and tell Louis it is out. If the post reports a result, send the Observer a
CHECKPOINT.

Then the review rounds of §2 run on the published post:
- **Each round's findings land as a follow-up PR** to the same post, restructured rather than
  appended. Merge it at once, on the same terms as version 1. Don't wait for the next round.
- **A factual error a review finds is fixed at once,** in its own small PR, ahead of any
  structural revision.
- **The post is finalised** when a round returns PROCEED. Nothing marks it as unfinished
  before then.
- **One fresh-reader pass, on the live post:** a subagent given only the post reports where it
  would start, every term met before its explanation, where it felt dropped in medias res, and
  any mannered sentence. Its fixes ship as a follow-up PR. The Researcher's review rounds also
  run on the live post.
- **Fixes to a published post go as a docs commit straight to master** (Louis, 10 November
  2026: "docs commit not PR"), after the publish-page checks pass on the commit. A new post
  still goes through a PR.
- **Facts that arrive after drafting update the post at once.** A correction or a new
  measurement from the source's owner (a CORRECTION, a later PR head) goes beside the claim
  it affects. When it supersedes a figure, the post leads with the new one, and the old one
  is kept only as history.
