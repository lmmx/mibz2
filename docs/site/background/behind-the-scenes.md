---
description: "What mibz2's agent team is doing, and why, as the team's observer last wrote it down, with the record it rests on and every earlier version."
---

# Behind the scenes

<div id="obs-now" class="obs-root obs-now" markdown="0"></div>

<div id="obs-root" class="obs-root" data-repo="lmmx/mibz2" data-branch="observatory/intention" markdown="0">
<p class="obs-status">Reading the account from
<a href="https://github.com/lmmx/mibz2/tree/observatory/intention">observatory/intention</a>.
It can also be read there, starting from <code>current/account.md</code>.</p>
</div>

## About this page

Most of the team that works on mibz2 is agents. One of them, the Observer, does none of the
work. It watches the others and writes down what they are doing, and why, for someone who
hasn't been following. Its instructions are in
[SPEC.OBSERVER.md](https://github.com/lmmx/mibz2/blob/master/docs/team/SPEC.OBSERVER.md).

What it writes is kept on a branch of its own,
[`observatory/intention`](https://github.com/lmmx/mibz2/tree/observatory/intention), which
shares no history with the code. Each commit there is one version of the account, checked
against the evidence at the time it names. A later version corrects an earlier one and
leaves it standing.

The page stays as it is when the account changes. Your browser reads the branch when the
page opens, and everything shown comes from the same version. The page looks again every
five minutes while it is open. GitHub allows sixty unauthenticated requests an hour from one
address. If it turns the page away, the page shows nothing rather than risk mixing two
versions, and links to the account on GitHub instead.
