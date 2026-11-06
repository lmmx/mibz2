---
description: "What mibz2's agent team is doing and why, read live from the Oracle's record: one line, five lines, the tree, each front, and every version since it began."
---

# Behind the scenes

mibz2 is developed by a small team of agents with a human maintainer. One of them, the
Oracle, keeps a running account of what the team is doing and why. It writes no code and
gives no orders. It reads the channel and the evidence, and settles one version of the
account at a time. The account is kept on an orphan branch,
[`observatory/intention`](https://github.com/lmmx/mibz2/tree/observatory/intention), apart
from the code, and its history is the record of how the team's intentions changed.

This page reads that branch when you open it, so it shows the latest settled version
without being redeployed. Each claim links to its evidence.

| depth | section below | what you get |
|---|---|---|
| **L0** | Now | what the team is doing, and in order to do what, in a line or two |
| **L1** | In five lines | the aim, the question, what was found, what is happening now, what is waited on |
| **L2** | The tree | every front, each serving the one above it, with its status |
| **L3** | Each front | per front: doing, in order to, status, what is known, and its sources |
| ↔ | Where the "why" is weak | where a front's reason is stale, contradicted or unestablished |
| meta | This version | what this version covered, and the evidence it was checked against |
| meta | Every move | every settled version, newest first, with what changed and the diff |

<div id="obs-root" class="obs-root" data-repo="lmmx/mibz2" data-branch="observatory/intention" markdown="0">
<p class="obs-status">Loading the Oracle's record from
<a href="https://github.com/lmmx/mibz2/tree/observatory/intention">observatory/intention</a>.
If nothing appears, read it there: <code>current/L0.md</code> first, then
<code>L1.md</code>, <code>L2.md</code> and <code>L3/</code>.</p>
</div>

## How this page works

- **One deploy, no churn.** The page is part of the site and changes only when the site
  does. The Oracle's commits go to its own branch, which has no CI and no deploys.
- **Read at view time.** Your browser asks GitHub's API for the branch's newest commit, then
  fetches that commit's files from `raw.githubusercontent.com`. Every level comes from that
  one commit, so the levels never mix two versions.
- **Limits.** GitHub allows 60 unauthenticated API requests an hour from one address, and
  this page uses two. If the API refuses, the page falls back to the branch's files, which
  GitHub may cache for about five minutes, and the history is left out.
- **What a version is.** Each commit on the branch is one settled version. Its
  `manifest.md` names the channel comment it covers through, and the evidence it was
  checked against. A version is corrected by a later one, never by rewriting. The Oracle's
  spec is
  [docs/team/SPEC.ORACLE.md](https://github.com/lmmx/mibz2/blob/master/docs/team/SPEC.ORACLE.md).
