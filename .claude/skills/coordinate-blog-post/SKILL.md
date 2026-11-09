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

## 1. Brief the drafting agent

The drafting agent works in its own worktree, on a `docs/blog-…` branch from master. Its
brief names:
- **The source:** a relayed report, a journal or a result, and Louis's verbatim words if the
  request came through the relay (`FROM: louis via chatgpt`). Acknowledge the request with
  one LOG on #145.
- **The primary records** every number is checked against: the journals, the registrations,
  the evidence bundles. Any figure that can't be traced is flagged in the PR, never printed as
  established.
- **The skills to follow,** in order: `write-blog-post`, then `publish-page`.
- **Takeaways first** (§3 below). Before drafting, the agent writes in its plan:
  - the simplest correct explanation of the result, in two or three sentences;
  - the key takeaways, as a short list.

  The PR body says where in the post each takeaway first lands.
- **The dek and the sentences:** `write-blog-post`, "The dek, and the sentences". The dek must
  pass `docs/scripts/check_prose.py` and carry a little joy. The PR reports the body's flagged
  share.
- **Visuals:** a figure for each result, comparison, architecture and trade-off, and for
  the key takeaways where it helps. Dense prose where a figure would carry it is a
  failure mode.
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

Merge the post's PR as soon as all of these hold. Don't wait for the Researcher:
- every figure is checked against its primary record, and any that can't be traced is
  flagged in the PR, never printed as established;
- `publish-page` was run on the final head, and the PR records which commit was looked at and
  what was found;
- CI is green on that head.

Merge with `coord.py head-sha` before and after (`expectedHeadSha`), check the live page once
deployed, and tell Louis it is out. If the post reports a result, send the Observer a
CHECKPOINT.

Then the review rounds of §2 run on the published post:
- **Each round's findings land as a follow-up PR** to the same post, restructured rather than
  appended. Merge it on green with publish-page rerun. Don't wait for the next round.
- **A factual error a review finds is fixed at once,** in its own small PR, ahead of any
  structural revision.
- **The post is finalised** when a round returns PROCEED. Nothing marks it as unfinished
  before then.
