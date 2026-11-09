---
name: coordinate-blog-post
description: Use when the coordinator commissions, reviews or publishes a mibz2 blog post written by another agent: briefing the drafting agent, routing two rounds of simulated-reader (exegenius) review to the Researcher, the revision between rounds, and the merge. The writing itself is write-blog-post's; the render check is publish-page's.
---

# Coordinating a blog post

`write-blog-post` says how a post is written, and `publish-page` how it is checked once
rendered. This skill is the coordinator's side: who does each step, in what order, and what
each one is given. It records Louis's instruction of 9 November 2026: every post gets two
rounds of exegenius review by the Researcher, and the review must check that the simplest
understanding and the key takeaways land in the opening.

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
- **The plan's goal questions and held-out goal,** recorded in the PR body for the
  reviewer.
- **The rules:** Louis's identity on commits, no AI attribution, the footer stripped from the
  PR body, no merge.

## 2. Two rounds of review by the Researcher

Once the PR is up, route a CHECKPOINT to the Researcher with `coord.py route` (QUEUED on
#145 while it is busy). Each round asks for:
1. **The independent baseline, before reading the post:** from the primary records alone,
   the simplest correct explanation of the result and the key takeaways.
2. **The comparison:** where each of those lands in the post. Anything that arrives late,
   or that leaves the reader unable to follow what comes before it, is a finding.
   - The simplest understanding belongs in the opening.
   - The takeaways belong in the opening or section 1, as soon as they can be stated.
     They are never withheld for a later section.
3. **The simulated-reader pass** (`write-blog-post`, the protocol's steps 5 and 6):
   - the goal questions answered from the rendered text alone;
   - stuck points recorded by level (`symbol`, `statement`, `step`, `chain`, `beyond`);
   - fragmentation, collapse and missing steps;
   - every fact checked against its source.
4. **A verdict:** PROCEED, REVISE (with the findings) or STOP.

**Between the rounds,** the drafting agent revises for the findings. It restructures rather
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

## 4. Publish

Merge only when all of these hold:
- the Researcher's latest round is PROCEED;
- `publish-page` was rerun on the final head, and the PR records which commit was looked at
  and what was found;
- CI is green on that head.

Then merge with `coord.py head-sha` before and after (`expectedHeadSha`). Check the live
page once it is deployed, and tell Louis it is out. If the post reports a result, send the
Observer a CHECKPOINT.
