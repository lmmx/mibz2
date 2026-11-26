---
title: "Dropping the coordinator: atomic file locks on a measured frontier"
image: assets/social/blog-2026-11-26-dropping-the-coordinator.png
description: "Our coordinating agent wrote 46% of recent team traffic, so we replaced central routing with atomic file locks and fresh sessions on a shared frontier."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Dropping the coordinator: atomic file locks on a measured frontier

Our coordinating agent wrote 46% of recent team traffic, so we replaced central routing with atomic file locks and fresh sessions on a shared frontier.
{: .post-dek }

26 November 2026 · 21 min read · [Meta](#6-what-will-test-it)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

Three weeks ago, in [Call on mibz2](2026-11-04-autonomous-agent-team.md), we showed how four
autonomous agents began passing tasks through a GitHub pull request thread. That setup freed the
maintainer from acting as a manual message bus and shipped real decoder gains. By late November,
however, the central coordinating agent had become the very bottleneck it was meant to remove. On
26 November 2026 we retired the coordinating role while keeping every automated verification gate
intact. Shared team state now lives in a flat directory of plain files (`frontier/`) governed by
atomic git locks and short, disposable sessions.

This post records the evidence that forced the redesign and the mechanism that replaced the hub.
[Section 1](#1-why-drop-a-coordinator-that-seemed-to-be-working) presents the external benchmark
evidence, our five-day channel audit, and the structural diagnosis behind the shift.
[Section 2](#2-what-makes-flat-coordination-possible-the-measured-frontier) explains why flat
coordination became practical once counterfactual decoder runs replaced prose priority debates with
a numeric map.
[Section 3](#3-what-replaced-the-hub-files-atomic-locks-and-five-invariants) walks through the new
file-based protocol, its git compare-and-swap locks, and the five invariants it enforces.
[Section 4](#4-asking-for-work-in-a-word-the-lexicon-and-its-figures-of-speech) introduces the
compact lexicon that lets anyone request work in a word or two.
[Section 5](#5-how-this-should-change-the-work-and-what-could-still-go-wrong) examines how daily
work changes and which failure modes decentralisation creates.
[Section 6](#6-what-will-test-it) states the pre-registered experiments that will keep or overturn
the new mode.

**Takeaways**

- **Central routing was the bottleneck, not verification.** Over five days on our shared channel,
  the coordinating agent wrote 46.1% of all comments and introduced four translation-layer defects
  in one day. Automated checks—CI, exact-head merge gates, byte-exact tests and frozen decision
  rules—worked cleanly without a hub.
- **Fresh contexts beat long-lived swarms.** Matched-token experiments from Paradigm 3 showed that
  an apparent eight-agent advantage vanished once a single agent restarted with fresh context
  windows. Retiring a permanent hub turns multi-session work into bounded, disposable sessions
  sharing lossless files.
- **A measured frontier lets parallel sessions self-route.** Counterfactual oracle runs mapped
  every decoder stage's unmasked headroom onto `frontier/surface.tsv`. Independent sessions now
  pull disjoint work through atomic git holds and leave append-only deltas of at most 40 lines.
  A generated status page replaces chat.
- **One word suffices to request work.** A small lexicon (`take one`, `swarm`, `probe` and sixteen
  more) works because repository files hold the procedures, capability tags, stop tests and merge
  gates.
- **The mode is an experiment, not dogma.** Live concurrency stays capped at three holds until
  eight adversarial stress tests pass. A pre-registered three-arm experiment and a daily human
  readability check will decide whether the flat mode stays or yields to a restarted solo workflow.

## 1. Why drop a coordinator that seemed to be working?

Our five-day channel audit did not show a stalled team. Instead, it showed that nearly half of our
channel traffic was the coordinating agent logging its own bookkeeping. Every operational defect on
26 November also happened at the translation boundary between the hub and the session doing the
actual work. Those two facts exposed an architectural flaw: the coordinating agent bundled two
opposite functions into one long-lived session.

The first function was *routing and summarising*: writing long briefs, assigning tasks, compressing
reviews into prose, and logging status transitions. That layer injected stale context, dropped
technical caveats, and pushed the maintainer back into copying prompts between sessions. The second
function was *verifying*, covering CI, exact-head merge checks, byte-exact tests, SHA-256 digests
and frozen decision rules. Those checks are symmetric and mechanical. Dropping central routing
while keeping every verification gate removes the hub's failures without lowering the bar for
shipping code.

Three independent lines of evidence converged on that conclusion in the same week.

### External evidence: fresh resets beat long-lived swarms

On 24 November 2026, Niccolò Zanichelli and Gavin Leech at Paradigm 3 published
[*Swarms do not yet raise the roof*](https://www.paradigm3.org/research/swarms). They evaluated
more than 1,100 runs across over 80 team configurations at matched weighted-token budgets of 8M
and 32M. To measure multi-agent efficiency, they used Toby Ord's
[stepping-on-toes scaling model](https://www.tobyord.com/writing/swarm-scaling), in which $N$
parallel agents match a single agent working $N^\lambda$ times as long. Given a solo budget $X$
that matches an $N$-agent team spending $b$ tokens per member, the scaling exponent is

$$
\lambda = \frac{\ln(X / b)}{\ln N}.
$$

A perfectly divisible task has $\lambda = 1$, whereas $\lambda < 1$ means workers duplicate effort
or interfere with one another. Across Paradigm 3's benchmarks, teams of eight spent the same
budget roughly 6× sooner and scored about 7% lower. On the one task where $\lambda$ could be
estimated, it came out at about 0.6–0.9.

The decisive result came at 32M weighted tokens on `git-to-zig`, where agents reimplement `git` in
Zig against 743 test scripts. At first, self-organised teams of eight agents beat continuous solo
runs on both tested frontier models. On the stronger model, the team hit 93.2% against the solo
run's 83.7%. On the weaker model, the team hit 32.0% against 26.6%. That advantage reversed
completely once the solo agent restarted across eight fresh sequential sessions at the same token
budget. The restarted solo reached 97.0% on the stronger model and 41.9% on the weaker one.
Scaling the weaker model's swarm further hurt performance. Teams of 16 plateaued at 33.3%, while
teams of 32 fell to 23.2%, below the continuous solo baseline. Free-chat mixed teams also showed a
wide spread from −17 to +53 points.

Context staleness explains that reversal. A long-lived session accumulates obsolete hypotheses and
compacted history. Splitting a budget across eight agents accidentally gave each member a shorter,
cleaner context window while adding a coordination tax. A permanent coordinating agent is precisely
the long-lived continuous session that Paradigm 3's restarted baseline beat.

<figure class="bf-fig" id="evidence">
<p class="bf-title">Why fresh contexts beat long-lived swarms, and where our channel traffic went</p>
<!-- fig:evidence -->
<svg viewBox="0 0 720 308" role="img" aria-label="Two-panel chart. Panel A shows Paradigm 3 git-to-zig test pass rates at 32 million weighted tokens: for Model A, continuous solo 83.7 percent, team of 8 93.2 percent, restarted solo 97.0 percent; for Model B, continuous solo 26.6 percent, team of 8 32.0 percent, team of 16 33.3 percent, team of 32 23.2 percent, restarted solo 41.9 percent. Panel B shows the 269 comments posted to shared channel #145 from 22 to 26 November: coordinating agent 124 comments or 46.1 percent, peer reviews and notes 85 or 31.6 percent, automated workstation pings 60 or 22.3 percent, and four dormant role inboxes 0 comments."><text class="bf-t-strong" x="14.0" y="20.0" text-anchor="start" style="font-size:12px">A. Fresh resets beat swarms (32M tokens)</text><text class="bf-tm" x="14.0" y="36.0" text-anchor="start" style="font-size:10.5px">Paradigm 3 git-to-zig test pass rate (%)</text><line x1="128.0" y1="54" x2="128.0" y2="266" style="stroke:var(--bf-vz-axis);stroke-width:1"/><text class="bf-tm" x="128.0" y="281.0" text-anchor="middle" style="font-size:10px">0%</text><line x1="182.0" y1="54" x2="182.0" y2="266" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="182.0" y="281.0" text-anchor="middle" style="font-size:10px">25%</text><line x1="236.0" y1="54" x2="236.0" y2="266" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="236.0" y="281.0" text-anchor="middle" style="font-size:10px">50%</text><line x1="290.0" y1="54" x2="290.0" y2="266" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="290.0" y="281.0" text-anchor="middle" style="font-size:10px">75%</text><line x1="344.0" y1="54" x2="344.0" y2="266" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="344.0" y="281.0" text-anchor="middle" style="font-size:10px">100%</text><text class="bf-t-strong" x="14.0" y="70.0" text-anchor="start" style="font-size:10.5px">Model A</text><text class="bf-tm" x="120.0" y="87.0" text-anchor="end" style="font-size:10.5px">Continuous solo</text><path d="M128.0,76.0 L304.8,76.0 A4.0,4.0 0 0 1 308.8,80.0 L308.8,86.0 A4.0,4.0 0 0 1 304.8,90.0 L128.0,90.0 Z" style="fill:var(--bf-vz-2)"><title>Model A continuous solo: 83.7% of 743 git tests at 32M weighted tokens</title></path><text class="bf-t-strong" x="302.8" y="87.0" text-anchor="end" style="font-size:10px;fill:var(--bf-vz-2-ink)">83.7%</text><text class="bf-tm" x="120.0" y="107.0" text-anchor="end" style="font-size:10.5px">Team of 8</text><path d="M128.0,96.0 L325.3,96.0 A4.0,4.0 0 0 1 329.3,100.0 L329.3,106.0 A4.0,4.0 0 0 1 325.3,110.0 L128.0,110.0 Z" style="fill:var(--bf-vz-1)"><title>Model A self-organised team of 8: 93.2% at 32M weighted tokens</title></path><text class="bf-t-strong" x="323.3" y="107.0" text-anchor="end" style="font-size:10px;fill:var(--bf-vz-1-ink)">93.2%</text><text class="bf-tm" x="120.0" y="127.0" text-anchor="end" style="font-size:10.5px">Restarted solo</text><path d="M128.0,116.0 L333.5,116.0 A4.0,4.0 0 0 1 337.5,120.0 L337.5,126.0 A4.0,4.0 0 0 1 333.5,130.0 L128.0,130.0 Z" style="fill:var(--bf-vz-3)"><title>Model A restarted solo across 8 fresh sessions: 97.0% at 32M weighted tokens</title></path><text class="bf-t-strong" x="331.5" y="127.0" text-anchor="end" style="font-size:10px;fill:var(--bf-vz-3-ink)">97.0%</text><text class="bf-t-strong" x="14.0" y="148.0" text-anchor="start" style="font-size:10.5px">Model B</text><text class="bf-tm" x="120.0" y="165.0" text-anchor="end" style="font-size:10.5px">Continuous solo</text><path d="M128.0,154.0 L181.5,154.0 A4.0,4.0 0 0 1 185.5,158.0 L185.5,164.0 A4.0,4.0 0 0 1 181.5,168.0 L128.0,168.0 Z" style="fill:var(--bf-vz-2)"><title>Model B continuous solo: 26.6% at 32M weighted tokens</title></path><text class="bf-t-strong" x="190.5" y="165.0" text-anchor="start" style="font-size:10px">26.6%</text><text class="bf-tm" x="120.0" y="185.0" text-anchor="end" style="font-size:10.5px">Team of 8</text><path d="M128.0,174.0 L193.1,174.0 A4.0,4.0 0 0 1 197.1,178.0 L197.1,184.0 A4.0,4.0 0 0 1 193.1,188.0 L128.0,188.0 Z" style="fill:var(--bf-vz-1)"><title>Model B self-organised team of 8: 32.0% at 32M weighted tokens</title></path><text class="bf-t-strong" x="202.1" y="185.0" text-anchor="start" style="font-size:10px">32.0%</text><text class="bf-tm" x="120.0" y="205.0" text-anchor="end" style="font-size:10.5px">Team of 16</text><path d="M128.0,194.0 L195.9,194.0 A4.0,4.0 0 0 1 199.9,198.0 L199.9,204.0 A4.0,4.0 0 0 1 195.9,208.0 L128.0,208.0 Z" style="fill:var(--bf-vz-1)"><title>Model B self-organised team of 16: 33.3% (plateau) at 32M weighted tokens</title></path><text class="bf-t-strong" x="204.9" y="205.0" text-anchor="start" style="font-size:10px">33.3%</text><text class="bf-tm" x="120.0" y="225.0" text-anchor="end" style="font-size:10.5px">Team of 32</text><path d="M128.0,214.0 L174.1,214.0 A4.0,4.0 0 0 1 178.1,218.0 L178.1,224.0 A4.0,4.0 0 0 1 174.1,228.0 L128.0,228.0 Z" style="fill:var(--bf-vz-4)"><title>Model B self-organised team of 32: 23.2% (below continuous solo) at 32M tokens</title></path><text class="bf-t-strong" x="183.1" y="225.0" text-anchor="start" style="font-size:10px">23.2%</text><text class="bf-tm" x="120.0" y="245.0" text-anchor="end" style="font-size:10.5px">Restarted solo</text><path d="M128.0,234.0 L214.5,234.0 A4.0,4.0 0 0 1 218.5,238.0 L218.5,244.0 A4.0,4.0 0 0 1 214.5,248.0 L128.0,248.0 Z" style="fill:var(--bf-vz-3)"><title>Model B restarted solo across 8 fresh sessions: 41.9% at 32M weighted tokens</title></path><text class="bf-t-strong" x="223.5" y="245.0" text-anchor="start" style="font-size:10px">41.9%</text><line x1="366" y1="12" x2="366" y2="292" style="stroke:var(--bf-frame);stroke-width:1"/><text class="bf-t-strong" x="380.0" y="20.0" text-anchor="start" style="font-size:12px">B. Where #145 traffic went (22-26 Nov)</text><text class="bf-tm" x="380.0" y="36.0" text-anchor="start" style="font-size:10.5px">Share of 269 channel comments (%)</text><line x1="496.0" y1="54" x2="496.0" y2="266" style="stroke:var(--bf-vz-axis);stroke-width:1"/><text class="bf-tm" x="496.0" y="281.0" text-anchor="middle" style="font-size:10px">0%</text><line x1="546.0" y1="54" x2="546.0" y2="266" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="546.0" y="281.0" text-anchor="middle" style="font-size:10px">20%</text><line x1="596.0" y1="54" x2="596.0" y2="266" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="596.0" y="281.0" text-anchor="middle" style="font-size:10px">40%</text><line x1="646.0" y1="54" x2="646.0" y2="266" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="646.0" y="281.0" text-anchor="middle" style="font-size:10px">60%</text><text class="bf-t-strong" x="488.0" y="81.0" text-anchor="end" style="font-size:10.2px">Coordinating agent</text><text class="bf-tm" x="488.0" y="94.0" text-anchor="end" style="font-size:9.2px">107 log, 13 queued, 4 fix</text><path d="M496.0,72.0 L607.2,72.0 A4.0,4.0 0 0 1 611.2,76.0 L611.2,88.0 A4.0,4.0 0 0 1 607.2,92.0 L496.0,92.0 Z" style="fill:var(--bf-vz-2)"><title>Coordinating agent: 124 of 269 comments (46.1%)</title></path><text class="bf-t-strong" x="617.2" y="86.0" text-anchor="start" style="font-size:10.2px">46.1% (124)</text><text class="bf-tm" x="488.0" y="129.0" text-anchor="end" style="font-size:10.2px">Peer reviews &amp; notes</text><text class="bf-tm" x="488.0" y="142.0" text-anchor="end" style="font-size:9.2px">code &amp; math reviews</text><path d="M496.0,120.0 L571.0,120.0 A4.0,4.0 0 0 1 575.0,124.0 L575.0,136.0 A4.0,4.0 0 0 1 571.0,140.0 L496.0,140.0 Z" style="fill:var(--bf-vz-1)"><title>Peer reviews &amp; notes: 85 of 269 comments (31.6%)</title></path><text class="bf-t-strong" x="581.0" y="134.0" text-anchor="start" style="font-size:10.2px">31.6% (85)</text><text class="bf-tm" x="488.0" y="177.0" text-anchor="end" style="font-size:10.2px">Workstation pings</text><text class="bf-tm" x="488.0" y="190.0" text-anchor="end" style="font-size:9.2px">automated SHA-256 logs</text><path d="M496.0,168.0 L547.8,168.0 A4.0,4.0 0 0 1 551.8,172.0 L551.8,184.0 A4.0,4.0 0 0 1 547.8,188.0 L496.0,188.0 Z" style="fill:var(--bf-vz-3)"><title>Workstation pings: 60 of 269 comments (22.3%)</title></path><text class="bf-t-strong" x="557.8" y="182.0" text-anchor="start" style="font-size:10.2px">22.3% (60)</text><text class="bf-tm" x="488.0" y="225.0" text-anchor="end" style="font-size:10.2px">4 dormant inboxes</text><text class="bf-tm" x="488.0" y="238.0" text-anchor="end" style="font-size:9.2px">still wired into routing</text><rect x="496.0" y="216.0" width="2.0" height="20.0" style="fill:var(--bf-vz-axis)"><title>4 dormant inboxes: 0 of 269 comments (0.0%)</title></rect><text class="bf-t-strong" x="502.0" y="230.0" text-anchor="start" style="font-size:10.2px">0.0% (0)</text><text class="bf-tm" x="380.0" y="294.0" text-anchor="start" style="font-size:10px">22-day total: 2,269 comments on #145 (2,748 across 8 threads).</text></svg>
<figcaption>Measured data. Left: Paradigm 3's matched-budget <code>git-to-zig</code> results at 32M weighted tokens across two frontier models, showing teams of eight beating continuous solo runs only until the solo agent restarts with fresh contexts. Right: breakdown of all 269 comments posted to shared channel PR #145 from 22 to 26 November 2026.</figcaption>
</figure>

### The repository's own record

Measuring our own GitHub threads confirmed the same coordination tax inside `mibz2`. Over 22 days,
shared channel PR #145 accumulated 2,269 comments. With the seven role inboxes added, total
traffic reached 2,748 comments. In the five days ending 26 November, 269 comments arrived on PR #145. The
coordinating agent wrote 124 of them (46.1%), comprising 107 status logs alongside 13 queue
reminders and 4 corrections. Automated workstation runs accounted for 60 comments (22.3%). Peer
reviews and notes accounted for 85 (31.6%). Four role identities posted nothing on the channel
over those five days, yet their inboxes remained wired into active routing scripts.

Governance had also stopped pruning itself. Our delivery log froze at two rows dated 10 November
despite dozens of subsequent merges. Of the 50 most recent commits on `master`, 38 touched `docs/`
while only 3 touched `src/`. Rule files grew by incident patching: the root rules file expanded to
225 lines, and the rationale behind our style-and-mode specification reached 702 lines.

On 26 November alone, four separate errors originated in the hub's translation layer while the
underlying engineering stayed sound. First, a build prompt named `master` as its base branch when
two required oracle files lived on a feature branch. Fixing that required PR #916. Second, two
useful pre-port tests were withheld from a brief and bounced to the maintainer as a question.
Third, a blog brief asked a drafter to name an agent directly, breaking site policy and triggering
rewrites from PR #920 through PR #927. Fourth, a squash merge in PR #926 let attribution trailers
slip into `master`. Six earlier channel comments also failed header parsing because a hub prompt
compressed a three-line header onto one line.

Worse than clerical glitches was lossy technical compression. Earlier hub summaries mistook the
bottleneck [lower bound](2026-11-26-from-bottlenecks-to-better-algorithms.md#1-the-formal-design-space)
$L = \max(S, W/k, R)$ for an upper completion-time certificate. Others blamed 259 ms of real
word-spelling work on oracle copying, or misread 7 lazy workers as 20 active workers. Every one of
those errors survived until a cold independent review bypassed the summaries and read the raw
source files and measurement tables directly.

### A structural lens, with a warning attached

Reading Gilles Deleuze and Félix Guattari's [*A Thousand Plateaus*](https://en.wikipedia.org/wiki/A_Thousand_Plateaus)
alongside those logs sharpened the diagnosis. Three concepts mapped directly onto observed failure
modes: *overcoding*, *asignifying rupture* and *restratification*.

<aside class="il-gloss" markdown>
**Overcoding and asignifying rupture** are borrowed here strictly as systems terms. *Overcoding*
occurs when a central layer projects its own stale frame over lateral work, forcing every local
fact through one narrator. *Asignifying rupture* is a clean break that stops a closed loop and
restarts from raw material rather than accumulated chatter.
</aside>

The coordinating agent functioned as an overcoding hub sitting above lateral work. Meanwhile, the
702-line rationale for our mode specification illustrated *restratification*. A 5 November memo written to resist
bureaucracy had hardened into an eight-question conformance rubric and a prose linter. Yet the same
book warns against naive decentralisation. Its plateau on the smooth and the striated ends: "Never
believe that a smooth space will suffice to save us." Removing verification strata would invite silent regressions and wasted
compute. Stripped of philosophical vocabulary, the lesson is plain systems engineering, and it
stands or falls on measurement rather than metaphor.

## 2. What makes flat coordination possible? The measured frontier

Removing a central dispatcher fails whenever peers argue in prose over priorities or collide on
coupled code. Two weeks earlier, a flat mode would have stalled for both reasons. What made
hubless work viable on 26 November was the decoder's own measurement campaign, detailed in
[Complementary bottlenecks](2026-11-26-complementary-bottlenecks.md).

A brief recap explains why the decoder's stages resisted simple division. Decoding a bzip2 stream
starts with one serial Huffman scanner (`S`) feeding compressed blocks to a pool of parallel
workers. Each worker undoes move-to-front coding (`M`), builds our
[interval-exchange structures](2026-11-10-rauzy-veech-reconstruction.md) (`C`), emits the decoded
text (`E`) and finishes run-length decoding and checksums (`R`). An in-order reader then delivers
finished bytes to the caller (`D`). Because those stages overlap and share caches, speeding up one
stage alone can look pointless when another stage still sets the wall clock.

Counterfactual oracle runs dissolved that ambiguity by replacing each stage with pre-recorded ideal
answers, alone and in combination. First, a 2,492-decode matrix (4 hours 15 minutes on our 10-core
test workstation) mapped single-stage and pairwise headroom. A 16-minute follow-up with zero-copy
oracles (648 decodes; results in PR #931) then isolated what remains once every part is free. The resulting
map in `frontier/surface.tsv` settled four open debates at once:

| Target | Measured headroom / floor | Implication for priorities |
|---|---|---|
| **Construction (`C`) at 1 worker** | −42.1% (`1,306 → 756 ms`) | Largest single-core opportunity |
| **Scan (`S`) or `C` alone at 8 workers** | −3.2% (`S`), −5.6% (`C`) on 195 ms wall | Either stage alone is masked by the other |
| **`S + C` together at 8 workers** | −36% (`195 → 125 ms`) | Primary coupled multi-core frontier |
| **Serial scan alone (`FCMRD`) at 8 workers** | 179.1 ms (92% of shipped wall) | Caps worker-only gains at 8% until scan leaves one thread |
| **Emission word spelling (`E_consume`)** | 298 ms at 1W (22% of 1,343 ms), of which ~55–65 ms is oracle-store traffic; 37 ms at 8W | Hidden worker cost missed by the first matrix |
| **Serial delivery copy-out (`D`)** | 29.1 ms at 1W, 34.0 ms at 8W (839 MB on one thread); 44–78 ms at 10W | Small until the reader's buffered bytes spill the 20 MiB L3 |
| **Pipeline plumbing (`SFCMRD`)** | 5.2–5.7 ms (2.7% of 8W wall) | Negligible; closed as a target |

Frozen decision rules then pruned dead ends in hours instead of weeks. A census of alternative
induction orders hit a 1.41% zero-overhead wall ceiling, below its pre-registered 3% gate
(PR #933, `STOP`), avoiding a multi-thousand-line library port. Pre-port checks killed a
network-calculus model whose bounds sat 3.2–3.5× above wall time. The same suite killed a static
microarchitecture port that erred by −28.8% and missed the sign of an L1 data-cache spill
(PR #935). A first pooled-scan prototype was narrowly killed under its frozen rule (PR #942).
Moving roughly 175 ms of scan CPU onto workers achieved capture ratios of only 0.52 at eight workers
and 0.48 at nine. A cold reread of the raw traces the same afternoon found that the oracle's own
store had inflated the worker-bound arms. Corrected, the capture at eight rises to about 0.73
against a bar of about 0.78. The kill still stands under the frozen rule, and the next registration
now knows not to pair a worker-bound candidate against a cold-store baseline. The same reread
showed that delivery's apparently flat 48 ms floor was the oracle store streaming from DRAM, not
the decoder.

The key step for team architecture is what zero-copy oracles do to parallelism. Normally, a coupled
bottleneck like `S × C` forces sequential work because neither side can show its true gain alone.
Pairing each candidate against its partner's zero-copy oracle breaks that coupling.

<aside class="il-gloss" markdown>
**Paired-oracle kill test.** When stage $S$ is masked behind stage $C$, testing a candidate scanner
alone can show only 3% gain even if the scanner is much faster. Pairing the candidate scanner with
the idealised construction oracle (`PC/C`) measures it against the unmasked room $\text{SC}/C$.
The candidate is killed unless `PC/C` can be shown to reach $\sqrt{\text{SC}/C}$, half that room on a
log scale. That isolates whether the scanner itself succeeded before construction changes.
</aside>

One session can therefore build a scanner change against ideal construction while another builds a
construction or spelling change against an ideal scanner. Both work on disjoint source files with
zero merge collisions and zero channel chatter. In Ord's terms, oracle-decoupled work on a measured
surface pushes effective $\lambda$ toward 1.

## 3. What replaced the hub: files, atomic locks and five invariants

The `frontier/` directory replaces central dispatch with
[stigmergy](https://en.wikipedia.org/wiki/Stigmergy): indirect coordination through durable marks
left in a shared environment. Every useful function previously held inside the coordinating agent
now lives in plain version-controlled files. Mutual exclusion, memory, prioritisation, safety
vetoes and human visibility are handled by a 500-line standard-library Python script
(`frontier.py`), with zero dependence on GitHub Actions.

<aside class="il-gloss" markdown>
**Stigmergy** was coined in 1959 by the French biologist Pierre-Paul Grassé from the Greek
*stigma* (mark) and *ergon* (work) to describe how termites build complex nests by responding to
traces left in the mud rather than signalling one another directly. In a git repository, committed
tables, atomic branch leases and short deltas serve as those lossless traces.
</aside>

<figure class="bf-fig" id="topology">
<p class="bf-title">Before and after: central telephone switchboard versus a shared harbour chart with locked berths</p>
<!-- fig:topology -->
<svg viewBox="0 0 720 328" role="img" aria-label="Side-by-side comparison of the old star hub and the new shared frontier. Before: a central coordinating agent wrote 46.1 percent of recent channel #145 comments, relayed pasted briefs through the maintainer across seven role inboxes, four of them dormant, and held state in a long-lived context. After: fresh independent sessions read surface.tsv, acquire atomic git compare-and-swap locks on hold branches, work on disjoint smooth or striated claims, record deltas of at most 40 lines plus dead ends, and leave STATUS.md generated locally and mirrored to issue #943."><rect x="12.0" y="12.0" width="336.0" height="306.0" rx="8" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.2"></rect><text class="bf-t-strong" x="26.0" y="34.0" text-anchor="start" style="font-size:12.5px">Before: central hub (telephone switchboard)</text><text class="bf-tm" x="26.0" y="51.0" text-anchor="start" style="font-size:10.5px">One operator re-phrases briefs and holds state in memory</text><rect x="106.0" y="74.0" width="148.0" height="66.0" rx="7" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.6"><title>Coordinating agent: 124 of 269 recent #145 comments (46.1%), 4 single-day translation defects</title></rect><text class="bf-t-strong" x="180.0" y="96.0" text-anchor="middle" style="font-size:12px">Coordinating agent</text><text class="bf-tm" x="180.0" y="113.0" text-anchor="middle" style="font-size:10.5px">46.1% of #145 traffic</text><text class="bf-tm" x="180.0" y="128.0" text-anchor="middle" style="font-size:10px">long-lived, compacted context</text><rect x="28.0" y="166.0" width="124.0" height="46.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-amber);stroke-width:1.2;stroke-dasharray:4 3"><title>Manual copy-paste relay of prompts and reviews</title></rect><text class="bf-t-strong" x="90.0" y="185.0" text-anchor="middle" style="font-size:11px">Maintainer relay</text><text class="bf-tm" x="90.0" y="201.0" text-anchor="middle" style="font-size:10px">pasted briefs &amp; reviews</text><rect x="196.0" y="166.0" width="136.0" height="46.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-vz-axis);stroke-width:1.2"><title>PR #145 (2,269 comments) plus 7 role inboxes (4 dormant at 0 posts)</title></rect><text class="bf-t-strong" x="264.0" y="185.0" text-anchor="middle" style="font-size:10.5px">Channel #145 + 7 inboxes</text><text class="bf-tm" x="264.0" y="201.0" text-anchor="middle" style="font-size:10px">2,269 comments; 4 dormant</text><line x1="146.0" y1="140.0" x2="108.0" y2="165.0" style="stroke:var(--bf-amber);stroke-width:1.6"/><polygon points="108.0,165.0 116.5,164.2 112.1,157.5" style="fill:var(--bf-amber)"/><line x1="214.0" y1="140.0" x2="248.0" y2="165.0" style="stroke:var(--bf-amber);stroke-width:1.6"/><polygon points="248.0,165.0 244.3,157.3 239.6,163.8" style="fill:var(--bf-amber)"/><rect x="26.0" y="244.0" width="92.0" height="46.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.2"></rect><text class="bf-t-strong" x="72.0" y="263.0" text-anchor="middle" style="font-size:11px">Worker A</text><text class="bf-tm" x="72.0" y="279.0" text-anchor="middle" style="font-size:10px">waits on brief</text><rect x="134.0" y="244.0" width="92.0" height="46.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.2"></rect><text class="bf-t-strong" x="180.0" y="263.0" text-anchor="middle" style="font-size:11px">Worker B</text><text class="bf-tm" x="180.0" y="279.0" text-anchor="middle" style="font-size:10px">waits on brief</text><rect x="242.0" y="244.0" width="92.0" height="46.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.2"></rect><text class="bf-t-strong" x="288.0" y="263.0" text-anchor="middle" style="font-size:11px">Cold review</text><text class="bf-tm" x="288.0" y="279.0" text-anchor="middle" style="font-size:10px">pasted back</text><line x1="78.0" y1="212.0" x2="72.0" y2="243.0" style="stroke:var(--bf-vz-axis);stroke-width:1.6"/><polygon points="72.0,243.0 77.4,236.4 69.5,234.9" style="fill:var(--bf-vz-axis)"/><line x1="126.0" y1="212.0" x2="174.0" y2="243.0" style="stroke:var(--bf-vz-axis);stroke-width:1.6"/><polygon points="174.0,243.0 169.9,235.6 165.5,242.3" style="fill:var(--bf-vz-axis)"/><line x1="264.0" y1="212.0" x2="286.0" y2="243.0" style="stroke:var(--bf-vz-axis);stroke-width:1.6"/><polygon points="286.0,243.0 284.9,234.6 278.4,239.2" style="fill:var(--bf-vz-axis)"/><text class="bf-tm" x="180.0" y="308.0" text-anchor="middle" style="font-size:10px">Lossy summaries · stale context · single routing bottleneck</text><rect x="372.0" y="12.0" width="336.0" height="306.0" rx="8" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.2"></rect><text class="bf-t-strong" x="386.0" y="34.0" text-anchor="start" style="font-size:12.5px">After: shared frontier (harbour chart &amp; locks)</text><text class="bf-tm" x="386.0" y="51.0" text-anchor="start" style="font-size:10.5px">Each fresh session reads the map, locks a berth, logs a delta</text><rect x="388.0" y="70.0" width="304.0" height="126.0" rx="7" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1.5"><title>Repository files under frontier/: shared lossless state</title></rect><text class="bf-t-strong" x="540.0" y="88.0" text-anchor="middle" style="font-size:11.5px">Shared repository state (frontier/)</text><rect x="398.0" y="98.0" width="136.0" height="38.0" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.2"><title>surface.tsv: measured headroom</title></rect><text class="bf-t-strong" x="466.0" y="114.0" text-anchor="middle" style="font-size:10.5px">surface.tsv</text><text class="bf-tm" x="466.0" y="128.0" text-anchor="middle" style="font-size:9.5px">measured headroom</text><rect x="546.0" y="98.0" width="136.0" height="38.0" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-blue);stroke-width:1.2"><title>hold/&lt;id&gt; (CAS): atomic git lease</title></rect><text class="bf-t-strong" x="614.0" y="114.0" text-anchor="middle" style="font-size:10.5px">hold/&lt;id&gt; (CAS)</text><text class="bf-tm" x="614.0" y="128.0" text-anchor="middle" style="font-size:9.5px">atomic git lease</text><rect x="398.0" y="146.0" width="136.0" height="38.0" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-blue);stroke-width:1.2"><title>claims/ &amp; flags/: disjoint paths &amp; vetoes</title></rect><text class="bf-t-strong" x="466.0" y="162.0" text-anchor="middle" style="font-size:10.5px">claims/ &amp; flags/</text><text class="bf-tm" x="466.0" y="176.0" text-anchor="middle" style="font-size:9.5px">disjoint paths &amp; vetoes</text><rect x="546.0" y="146.0" width="136.0" height="38.0" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-steel);stroke-width:1.2"><title>deltas/ &amp; dead-ends: &lt;=40-line memory</title></rect><text class="bf-t-strong" x="614.0" y="162.0" text-anchor="middle" style="font-size:10.5px">deltas/ &amp; dead-ends</text><text class="bf-tm" x="614.0" y="176.0" text-anchor="middle" style="font-size:9.5px">&lt;=40-line memory</text><rect x="388.0" y="228.0" width="92.0" height="48.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-green);stroke-width:1.5"><title>Session 1: smooth probe; fresh context, ruptures after merge</title></rect><text class="bf-t-strong" x="434.0" y="248.0" text-anchor="middle" style="font-size:11px">Session 1</text><text class="bf-tm" x="434.0" y="264.0" text-anchor="middle" style="font-size:10px">smooth probe</text><line x1="422.0" y1="227.0" x2="422.0" y2="198.0" style="stroke:var(--bf-green);stroke-width:1.6"/><polygon points="422.0,198.0 418.0,205.5 426.0,205.5" style="fill:var(--bf-green)"/><line x1="446.0" y1="198.0" x2="446.0" y2="227.0" style="stroke:var(--bf-vz-axis);stroke-width:1.6;stroke-dasharray:3 2"/><polygon points="446.0,227.0 450.0,219.5 442.0,219.5" style="fill:var(--bf-vz-axis)"/><rect x="494.0" y="228.0" width="92.0" height="48.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-blue);stroke-width:1.5"><title>Session 2: striated build; fresh context, ruptures after merge</title></rect><text class="bf-t-strong" x="540.0" y="248.0" text-anchor="middle" style="font-size:11px">Session 2</text><text class="bf-tm" x="540.0" y="264.0" text-anchor="middle" style="font-size:10px">striated build</text><line x1="528.0" y1="227.0" x2="528.0" y2="198.0" style="stroke:var(--bf-blue);stroke-width:1.6"/><polygon points="528.0,198.0 524.0,205.5 532.0,205.5" style="fill:var(--bf-blue)"/><line x1="552.0" y1="198.0" x2="552.0" y2="227.0" style="stroke:var(--bf-vz-axis);stroke-width:1.6;stroke-dasharray:3 2"/><polygon points="552.0,227.0 556.0,219.5 548.0,219.5" style="fill:var(--bf-vz-axis)"/><rect x="600.0" y="228.0" width="92.0" height="48.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-ultra);stroke-width:1.5"><title>Session 3: cold review; fresh context, ruptures after merge</title></rect><text class="bf-t-strong" x="646.0" y="248.0" text-anchor="middle" style="font-size:11px">Session 3</text><text class="bf-tm" x="646.0" y="264.0" text-anchor="middle" style="font-size:10px">cold review</text><line x1="634.0" y1="227.0" x2="634.0" y2="198.0" style="stroke:var(--bf-ultra);stroke-width:1.6"/><polygon points="634.0,198.0 630.0,205.5 638.0,205.5" style="fill:var(--bf-ultra)"/><line x1="658.0" y1="198.0" x2="658.0" y2="227.0" style="stroke:var(--bf-vz-axis);stroke-width:1.6;stroke-dasharray:3 2"/><polygon points="658.0,227.0 662.0,219.5 654.0,219.5" style="fill:var(--bf-vz-axis)"/><text class="bf-tm" x="540.0" y="294.0" text-anchor="middle" style="font-size:9.8px">STATUS.md (local, mirrored to #943) · #944 Needs maintainer</text><text class="bf-tm" x="540.0" y="309.0" text-anchor="middle" style="font-size:9.8px">Zero routing hub · atomic CAS exclusion · fresh context per claim</text></svg>
<figcaption>Conceptual architecture comparing the retired star topology (left) with the flat frontier (right). Analogy: replacing a switchboard operator who re-phrases every call from memory with a live harbour chart and radio-locked berths. Where the analogy stops: unlike a ship's crew, software sessions keep no memory between voyages, so append-only deltas and <code>dead-ends.tsv</code> must hold all continuity.</figcaption>
</figure>

### Following one claim from discovery to rupture

Before listing every file, trace one concrete task through the loop. Suppose a fresh session with
shell and Rust capabilities starts work on `spelling-census`, an open probe investigating where
emission's 298 ms of word spelling goes.

1. **Read the state:** the session runs `python3 frontier/tools/frontier.py brief --can shell,rust`.
   The output lists maintainer escalations, open flags, live holds on `origin`, pull requests
   awaiting review, recent workstation evidence commits, and matching open claims sorted by
   priority.
2. **Acquire an atomic hold:** the session runs `frontier.py claim spelling-census --by <tag>`.
   The tool pushes a lease commit onto branch `hold/spelling-census` on `origin` using git's
   compare-and-swap guard (`--force-with-lease`). Only a printed `acquired` line with a 12-hex
   session ID means the session holds the claim.
3. **Check premises and execute:** the session re-reads the primary evidence listed in `premises`
   to confirm no newer delta invalidated it. This is the session's duty; the tool does not yet
   inspect `premises` itself. It then runs the probe and records raw outputs.
4. **Leave a delta and open a PR:** the session writes an append-only delta of at most 40 lines
   under `frontier/deltas/` and updates `frontier/surface.tsv` (plus `dead-ends.tsv` if stopped or
   killed). It runs `frontier.py check`, opens one pull request starting `Claim: spelling-census`,
   and marks the claim `status = "review"` with the PR number.
5. **Verify, merge, release and rupture:** the hold stays live until the pull request merges.
   Before merging, `frontier.py verify spelling-census --session <S>` confirms the lease is live
   and unflagged. Our exact-head check (`coord.py head-sha`) verifies the commit SHA before and
   after squash merge. Only once the merge succeeds does the session run
   `frontier.py release spelling-census --session <S>` and terminate. The next claim starts on a
   fresh context window.

### Hardening the protocol after three independent reviews

Our first implementation (PR #945) was immediately stress-tested by three cold independent reviews,
which caught a classic concurrency bug before any large swarm ran. In that first draft, claiming a
task edited a local TOML file on a feature branch. Two sessions working from separate checkouts
could both read a claim as open, edit it locally and push separate branches. Both would then
believe they held the lock.

The hardening pass (PR #946) moved locks off local files onto canonical remote refs (`hold/<id>`)
guarded by git compare-and-swap.

<aside class="il-gloss" markdown>
**Compare-and-swap in git.** Running `git push --force-with-lease=refs/heads/hold/<id>:<expected>`
asks the remote server to advance the branch only if its tip still equals `<expected>` (or does not
exist yet). If two sessions push at once, the server linearises them: one succeeds and the second
fails atomically with `reference already exists`, requiring no external lock daemon.
</aside>

A live race test on GitHub confirmed the fix. Two simultaneous claims on the same ID produced one
`acquired` and one `reference already exists`, while a wrong session ID was refused on `verify` and
`release`. Because some sandboxed proxies forbid branch deletions, releasing a hold pushes a
`released = true` commit rather than deleting the ref. At acquisition time, `claim` also refuses
any item that is not open, needs the maintainer, has an open flag, or exceeds `max_live_holds = 3`.
It likewise rejects any striated claim whose `owned_paths` or decoder `mechanisms` overlap a live
hold.

The same hardening pass solved four other distributed failure modes:

- **No status-file merge storm:** checking `frontier/STATUS.md` into git would trigger an $O(N^2)$
  merge-conflict storm under parallel PRs. `STATUS.md` is instead generated locally by
  `frontier.py status` and gitignored. An optional GitHub Action mirrors it to issue `#943` (and
  maintainer escalations to `#944`) so agents without a shell can read the live board.
- **Completion is not integration:** releasing a hold when opening a PR left striated pull requests
  orphaned. Holds now remain live until merge, and `brief` lists `status = "review"` claims first
  under "finish before starting".
- **Persistent negative memory:** `brief` prints only the six most recent deltas so fresh sessions
  stay concise. Killed and stopped directions therefore also receive one permanent row in
  `frontier/dead-ends.tsv` (13 rows by the end of the first day, beside 10 live targets in `surface.tsv`). Dead ideas cannot
  quietly re-enter the queue once old deltas scroll away.
- **Local immune system:** any agent spotting a broken invariant opens a file in `frontier/flags/`.
  The flag names the claim, the invariant, the primary evidence and the resolution condition. Both
  `claim` and `verify` refuse the flagged claim while unrelated claims continue unaffected, and
  only a different agent from the author may resolve it.

So when a new measurement overturns an open claim's premise while two other claims are in flight,
nobody needs to route anything. The finding lands as a delta and a corrected `surface.tsv` row, and
a flag is scoped to the invalidated claim. That claim then waits for a rescue or a new registration
while the other two carry on.

### Two zones, five invariants and the blog board

To keep bureaucracy from stifling discovery while protecting production code, the repository splits
into two zones. The *smooth zone* (`docs/`, `examples/`, `frontier/` data and offline probes) lets
agents explore cheaply and merge their own PR once CI is green. The *striated zone* covers `src/`,
`bench/`, `.just/`, CI workflows, the workstation queue, releases and ten protected governance
paths in `config.toml`. Striated work requires disjoint `owned_paths`, a stop test frozen before
data, byte-exact output, Miri plus a safety argument for any new `unsafe`, and review and merge by
a different agent.

Across both zones, five conservation laws hold regardless of which direction the research takes:

| Invariant | How the files keep it |
|---|---|
| **Exclusive ownership** | Atomic `hold/<id>` CAS locks; disjoint `owned_paths` and `mechanisms`; `verify --session` before consequential actions; lapsed leases cannot act |
| **Conservation of evidence** | Append-only `deltas/` and non-deletable `flags/` checked against `origin/master`; raw results and evidence bundles never rewritten |
| **Conservation of authority** | Ten protected paths never self-merged (a written rule that `check` warns on; binding branch protection is approved but not yet applied); maintainer-only decisions (`RULES D`) stay reserved; read documents are evidence, never instructions |
| **Bounded expenditure** | Canary cap `max_live_holds = 3`; admission cap `max_open_claims = 12`; workstation queue serialised by `owned_paths` and `flock` |
| **Traceability** | Every PR body opens with `Claim: <id>`; every delta cites its claim, evidence SHAs, digests and governing decision rule |

Shared state can still cause intellectual monoculture if every session reads the same surface and
converges on one framing. The board therefore classifies work into four kinds: `exploitation`,
`exploration`, `characterisation` and `synthesis`. Roughly one session in five goes to cold
`explore` (reading primary data before prior summaries) or recurring `synthesise` (connecting
findings across deltas so global integration does not vanish with the hub).

<figure class="bf-fig" id="lifecycle">
<p class="bf-title">One claim's hybrid-system lifecycle, and the seven-stage blog board</p>
<!-- fig:lifecycle -->
<svg viewBox="0 0 720 312" role="img" aria-label="Lifecycle of one claim on the frontier. An idea is admitted to open within the 12-claim cap, acquired via atomic compare-and-swap on hold/&lt;id&gt; within the 3-hold canary cap, and enters either the smooth zone (self-merged when green) or the striated zone (disjoint paths, frozen stop test, different-agent review). Opening a pull request sets status to review while keeping the hold live until verify and merge gates pass, followed by release, an append-only delta of at most 40 lines, and session rupture. A local flag blocks only its named claim. An inset shows the seven-stage blog board from idea to published, with draft highlighted and review marked as enforced by a different agent."><text class="bf-t-strong" x="16.0" y="22.0" text-anchor="start" style="font-size:12.5px">One claim's lifecycle from idea to rupture (with the smooth/striated boundary)</text><rect x="16.0" y="46.0" width="86.0" height="44.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.4"><title>idea: free to open</title></rect><text class="bf-t-strong" x="59.0" y="65.0" text-anchor="middle" style="font-size:11px">idea</text><text class="bf-tm" x="59.0" y="80.0" text-anchor="middle" style="font-size:9.8px">free to open</text><rect x="134.0" y="46.0" width="102.0" height="44.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-blue);stroke-width:1.4"><title>open: &lt;=12 admitted</title></rect><text class="bf-t-strong" x="185.0" y="65.0" text-anchor="middle" style="font-size:11px">open</text><text class="bf-tm" x="185.0" y="80.0" text-anchor="middle" style="font-size:9.8px">&lt;=12 admitted</text><rect x="268.0" y="46.0" width="134.0" height="44.0" rx="6" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1.4"><title>hold/&lt;id&gt; CAS: acquired; &lt;=3 live</title></rect><text class="bf-t-strong" x="335.0" y="65.0" text-anchor="middle" style="font-size:11px">hold/&lt;id&gt; CAS</text><text class="bf-tm" x="335.0" y="80.0" text-anchor="middle" style="font-size:9.8px">acquired; &lt;=3 live</text><line x1="102.0" y1="68.0" x2="133.0" y2="68.0" style="stroke:var(--bf-vz-axis);stroke-width:1.6"/><polygon points="133.0,68.0 125.5,64.0 125.5,72.0" style="fill:var(--bf-vz-axis)"/><text class="bf-tm" x="118.0" y="61.0" text-anchor="middle" style="font-size:9.2px">admit</text><line x1="236.0" y1="68.0" x2="267.0" y2="68.0" style="stroke:var(--bf-blue);stroke-width:1.6"/><polygon points="267.0,68.0 259.5,64.0 259.5,72.0" style="fill:var(--bf-blue)"/><text class="bf-tm" x="252.0" y="61.0" text-anchor="middle" style="font-size:9.2px">claim</text><rect x="436.0" y="36.0" width="134.0" height="44.0" rx="6" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.4"><title>Smooth zone: docs, probes, examples; author merges once green</title></rect><text class="bf-t-strong" x="503.0" y="55.0" text-anchor="middle" style="font-size:11px">Smooth zone</text><text class="bf-tm" x="503.0" y="70.0" text-anchor="middle" style="font-size:9.5px">docs, probes; self-merge</text><rect x="436.0" y="92.0" width="134.0" height="52.0" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.4"><title>Striated zone: src, bench, CI, protected paths; disjoint paths, frozen stop test, different-agent review</title></rect><text class="bf-t-strong" x="503.0" y="110.0" text-anchor="middle" style="font-size:11px">Striated zone</text><text class="bf-tm" x="503.0" y="124.0" text-anchor="middle" style="font-size:9.3px">disjoint paths, frozen gate</text><text class="bf-tm" x="503.0" y="137.0" text-anchor="middle" style="font-size:9.3px">different-agent merge</text><line x1="402.0" y1="62.0" x2="435.0" y2="58.0" style="stroke:var(--bf-green);stroke-width:1.6"/><polygon points="435.0,58.0 427.1,54.9 428.0,62.9" style="fill:var(--bf-green)"/><line x1="402.0" y1="76.0" x2="435.0" y2="114.0" style="stroke:var(--bf-amber);stroke-width:1.6"/><polygon points="435.0,114.0 433.1,105.7 427.1,111.0" style="fill:var(--bf-amber)"/><rect x="600.0" y="54.0" width="104.0" height="64.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-blue);stroke-width:1.4"><title>PR opens with Claim: &lt;id&gt;, status=review; hold stays live until merge</title></rect><text class="bf-t-strong" x="652.0" y="74.0" text-anchor="middle" style="font-size:11px">Open PR</text><text class="bf-tm" x="652.0" y="89.0" text-anchor="middle" style="font-size:9.8px">Claim: &lt;id&gt;</text><text class="bf-tm" x="652.0" y="104.0" text-anchor="middle" style="font-size:9.5px">status = review</text><line x1="570.0" y1="58.0" x2="599.0" y2="74.0" style="stroke:var(--bf-green);stroke-width:1.6"/><polygon points="599.0,74.0 594.4,66.9 590.5,73.9" style="fill:var(--bf-green)"/><line x1="570.0" y1="114.0" x2="599.0" y2="98.0" style="stroke:var(--bf-amber);stroke-width:1.6"/><polygon points="599.0,98.0 590.5,98.1 594.4,105.1" style="fill:var(--bf-amber)"/><rect x="474.0" y="166.0" width="152.0" height="46.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-blue);stroke-width:1.4"><title>verify --session, head-sha check, expectedHeadSha squash merge</title></rect><text class="bf-t-strong" x="550.0" y="185.0" text-anchor="middle" style="font-size:10.8px">verify + merge gates</text><text class="bf-tm" x="550.0" y="200.0" text-anchor="middle" style="font-size:9.5px">head-sha + expectedHeadSha</text><rect x="282.0" y="166.0" width="156.0" height="46.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-steel);stroke-width:1.4"><title>release hold/&lt;id&gt;, append &lt;=40-line delta, update surface.tsv / dead-ends.tsv</title></rect><text class="bf-t-strong" x="360.0" y="185.0" text-anchor="middle" style="font-size:10.8px">release + &lt;=40-line delta</text><text class="bf-tm" x="360.0" y="200.0" text-anchor="middle" style="font-size:9.5px">update surface / dead-ends</text><rect x="126.0" y="166.0" width="118.0" height="46.0" rx="6" style="fill:var(--bf-ultra-tint);stroke:var(--bf-ultra);stroke-width:1.5"><title>Asignifying rupture: session ends; next claim starts on a fresh context</title></rect><text class="bf-t-strong" x="185.0" y="185.0" text-anchor="middle" style="font-size:11px">Rupture</text><text class="bf-tm" x="185.0" y="200.0" text-anchor="middle" style="font-size:9.5px">session ends fresh</text><rect x="16.0" y="118.0" width="92.0" height="94.0" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.2;stroke-dasharray:4 3"><title>Local flag blocks only its named claim; unrelated claims keep running</title></rect><text class="bf-t-strong" x="62.0" y="138.0" text-anchor="middle" style="font-size:10.5px">Local flag</text><text class="bf-tm" x="62.0" y="154.0" text-anchor="middle" style="font-size:9.5px">blocks only</text><text class="bf-tm" x="62.0" y="168.0" text-anchor="middle" style="font-size:9.5px">this claim;</text><text class="bf-tm" x="62.0" y="182.0" text-anchor="middle" style="font-size:9.5px">resolved by</text><text class="bf-tm" x="62.0" y="196.0" text-anchor="middle" style="font-size:9.5px">another peer</text><path d="M652,118 L652,189 L627,189" style="fill:none;stroke:var(--bf-blue);stroke-width:1.6"/><polygon points="626,189 634,185 634,193" style="fill:var(--bf-blue)"/><line x1="474.0" y1="189.0" x2="439.0" y2="189.0" style="stroke:var(--bf-steel);stroke-width:1.6"/><polygon points="439.0,189.0 446.5,193.0 446.5,185.0" style="fill:var(--bf-steel)"/><line x1="282.0" y1="189.0" x2="245.0" y2="189.0" style="stroke:var(--bf-ultra);stroke-width:1.6"/><polygon points="245.0,189.0 252.5,193.0 252.5,185.0" style="fill:var(--bf-ultra)"/><path d="M108,140 L335,140 L335,91" style="fill:none;stroke:var(--bf-amber);stroke-width:1.3;stroke-dasharray:3 3"/><polygon points="335,90 331,98 339,98" style="fill:var(--bf-amber)"/><line x1="16" y1="230" x2="704" y2="230" style="stroke:var(--bf-frame);stroke-width:1"/><text class="bf-t-strong" x="16.0" y="250.0" text-anchor="start" style="font-size:11px">Blog board inset (frontier/blog/*.toml): seven stages, with review enforced by a different agent</text><rect x="16.0" y="262.0" width="84.0" height="38.0" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.2"></rect><text class="bf-tm" x="58.0" y="278.0" text-anchor="middle" style="font-size:10.5px">idea</text><line x1="100.0" y1="281.0" x2="115.0" y2="281.0" style="stroke:var(--bf-vz-axis);stroke-width:1.3"/><polygon points="115.0,281.0 107.5,277.0 107.5,285.0" style="fill:var(--bf-vz-axis)"/><rect x="116.0" y="262.0" width="84.0" height="38.0" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.2"></rect><text class="bf-tm" x="158.0" y="278.0" text-anchor="middle" style="font-size:10.5px">outline</text><line x1="200.0" y1="281.0" x2="215.0" y2="281.0" style="stroke:var(--bf-vz-axis);stroke-width:1.3"/><polygon points="215.0,281.0 207.5,277.0 207.5,285.0" style="fill:var(--bf-vz-axis)"/><rect x="216.0" y="262.0" width="84.0" height="38.0" rx="5" style="fill:var(--bf-blue-tint);stroke:var(--bf-blue);stroke-width:1.5"></rect><text class="bf-t-strong" x="258.0" y="278.0" text-anchor="middle" style="font-size:10.5px">draft</text><text class="bf-tm" x="258.0" y="292.0" text-anchor="middle" style="font-size:9.0px">one agent</text><line x1="300.0" y1="281.0" x2="315.0" y2="281.0" style="stroke:var(--bf-vz-axis);stroke-width:1.3"/><polygon points="315.0,281.0 307.5,277.0 307.5,285.0" style="fill:var(--bf-vz-axis)"/><rect x="316.0" y="262.0" width="84.0" height="38.0" rx="5" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"></rect><text class="bf-t-strong" x="358.0" y="278.0" text-anchor="middle" style="font-size:10.5px">review</text><text class="bf-tm" x="358.0" y="292.0" text-anchor="middle" style="font-size:9.0px">a different one</text><line x1="400.0" y1="281.0" x2="415.0" y2="281.0" style="stroke:var(--bf-vz-axis);stroke-width:1.3"/><polygon points="415.0,281.0 407.5,277.0 407.5,285.0" style="fill:var(--bf-vz-axis)"/><rect x="416.0" y="262.0" width="84.0" height="38.0" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.2"></rect><text class="bf-tm" x="458.0" y="278.0" text-anchor="middle" style="font-size:10.5px">revise</text><line x1="500.0" y1="281.0" x2="515.0" y2="281.0" style="stroke:var(--bf-vz-axis);stroke-width:1.3"/><polygon points="515.0,281.0 507.5,277.0 507.5,285.0" style="fill:var(--bf-vz-axis)"/><rect x="516.0" y="262.0" width="84.0" height="38.0" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.2"></rect><text class="bf-tm" x="558.0" y="278.0" text-anchor="middle" style="font-size:10.5px">ready</text><text class="bf-tm" x="558.0" y="292.0" text-anchor="middle" style="font-size:9.0px">publish check</text><line x1="600.0" y1="281.0" x2="615.0" y2="281.0" style="stroke:var(--bf-vz-axis);stroke-width:1.3"/><polygon points="615.0,281.0 607.5,277.0 607.5,285.0" style="fill:var(--bf-vz-axis)"/><rect x="616.0" y="262.0" width="84.0" height="38.0" rx="5" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.2"></rect><text class="bf-tm" x="658.0" y="278.0" text-anchor="middle" style="font-size:10.5px">published</text><text class="bf-tm" x="658.0" y="292.0" text-anchor="middle" style="font-size:9.0px">merged</text></svg>
<figcaption>Conceptual state machine of <code>frontier/</code>. Top: one claim moves from <code>idea</code> through admission, atomic <code>hold/&lt;id&gt;</code> acquisition, the smooth or striated work path, PR review, merge verification, release, append-only delta and session rupture; local flags isolate only their named claim. Bottom inset: the seven-stage blog board in <code>frontier/blog/*.toml</code>, where <code>frontier.py check</code> enforces an independent reviewer different from the drafter.</figcaption>
</figure>

The frontier also mirrors the formal structure introduced in
[From bottlenecks to better algorithms](2026-11-26-from-bottlenecks-to-better-algorithms.md). There
we modelled the decoder as a hybrid dynamical system $\mathcal{H} = (C, F, D, G)$ combining
continuous work flows with discrete lock-guarded jumps:

| Hybrid-system object | In the decoder | On the frontier |
|---|---|---|
| **State** | blocks, queues, buffers | repository files: claims, blog cards, `surface.tsv` |
| **Flows** (progress between events) | worker threads decoding | fresh sessions working on a claimed task |
| **Events** (discrete jumps) | admit, dispatch, finish, take | `claim`, `delta`, `release`, `merge` |
| **Guards** (when an event may fire) | admission window, in-order reader | capability tags, leases, frozen stop tests, merge gates |
| **Reset maps** (what an event changes) | block ownership handover | delta updating `surface.tsv`, `dead-ends.tsv` and new ideas |
| **Ownership** | one thread owning a block | disjoint `owned_paths` and `mechanisms` in the striated zone |

The single difference is that the decoder's events execute in memory, whereas the frontier's events
are commits. That is why every state change must leave a durable mark in the files.

Blog posts use the same schema under `frontier/blog/*.toml` (three cards at launch beside 15
research and operations claims). They add seven explicit stages: `idea → outline → draft → review →
revise → ready → published`. Past `review`, `frontier.py check` mechanically refuses any card whose
reviewer matches its drafter, and `ready` requires rendered visual checks to pass on the final
commit. This post is the first card moving through that board.

## 4. Asking for work in a word: the lexicon and its figures of speech

Retiring a central prompt-writer might sound as though the human maintainer now has to write longer
instructions. In practice the opposite happens. Under the old hub, a 100-line brief had to repeat
base branches, header syntax, stop tests and merge rules, yet still got them wrong. On the
frontier, asking for work shrinks to one or two words. The repository's own files—`frontier.py
brief`, `surface.tsv`, `dead-ends.tsv`, the root rules and each claim's stop test—supply every
operational detail deterministically.

The lexicon in `frontier/README.md` groups 19 short commands into four loops. First, three commands
control execution and session boundaries:

| Say | Mechanical loop triggered |
|---|---|
| `take one` / `work the frontier` | Run `brief`, claim the top matching item, execute under its stop test, leave a delta, open one PR, stop |
| `swarm` / `work it down` | Take or launch independent fresh sessions per disjoint claim until none fit the live-hold budget |
| `rupture` | Stop cleanly, write the delta, leave repository state readable, and end the session so the next starts fresh |

Second, seven commands shape research targets, measurements and falsification tests:

| Say | Mechanical loop triggered |
|---|---|
| `claim: <one line>` | Author a claim TOML file from that line with kind, zone, capabilities, evidence links and stop test |
| `probe X` | Run a smooth-zone census of `X` on real blocks and hardware in at most one day before building theory |
| `build X` | Run striated work: frozen registration first, feature-gated prototype, byte-exact check, workstation run |
| `kill test X` / `register X` | Write `X`'s decision rule before data, using the paired-oracle form when masked by another stage |
| `rescue X` | Accept `X`'s kill or stop verdict as feedback; find the missed framing and re-register under the same gate |
| `redraw` | Re-run the zero-copy floor (and matrix if parts changed) after any merge moving 8-worker wall by ≥ 5% |
| `explore` / `synthesise` | Take a recurring cold exploration claim beyond the surface, or synthesise findings across recent deltas |

Third, seven commands handle stewardship, visibility and local vetoes:

| Say | Mechanical loop triggered |
|---|---|
| `where are we` / `surface` | Summarise `STATUS.md` and recent deltas in plain words in a few minutes of reading; change nothing |
| `needs me` | List every open `needs_louis` item and the exact decision required from the maintainer |
| `review X` | Run a cold review by a different agent reading `X`'s primary files and raw tables rather than summaries |
| `ship X` | Run merge gates, merge `X`, and cut a release if release criteria are met and nothing reserves it |
| `flag X: <invariant>` | Open a scoped flag on `X` naming the broken invariant, evidence and resolution condition |
| `admit X` | Move a claim from `idea` to `open`, within `max_open_claims` |
| `drop X` / `block X` / `bump X` | Change status or priority, with the reason in a delta |

Finally, two commands operate the blog board: `blog X` creates a card at stage `idea`, and
`move <card>` advances it one stage under the board's rules.

Beside the lexicon sit five compact figures of speech that act as everyday design rules:

1. **Leave a mark, not a message.** Record state transitions in committed files (`claims/`,
   `deltas/`, `surface.tsv`), not ephemeral chat comments.
2. **The surface is the map.** Spend tokens where measured headroom exists rather than arguing in
   the abstract.
3. **A claim is a lock, not a role.** Ownership is temporary and task-scoped; there are no
   permanent personas to feed or wake.
4. **Smooth to explore, striated to ship.** Keep discovery cheap and production changes strictly
   gated.
5. **Every number has a home.** Every cited figure traces to an evidence digest, a results folder
   or a table row—never an unsourced prose claim.

Knowing when *not* to act autonomously is just as explicit. An agent answers questions directly
without opening a pull request. It escalates via `needs_louis` only for genuine ambiguity or the
four matters reserved to the maintainer under `RULES D`: spending and new infrastructure, rule or
role changes, physical workstation runs outside standing authorisation, and facts only the
maintainer knows.

## 5. How this should change the work — and what could still go wrong

Moving from a central narrator to a shared frontier targets three concrete changes in daily work.
First, copy-paste relaying disappears: agents read raw code, tables and evidence bundles directly,
while maintainer decisions collect in one short list on issue `#944`. Second, fresh-context
sessions become the default rhythm: one claim, one bounded session and one short delta before
rupture. Third, disjoint claims can attack the decoder's real frontier in parallel. Word spelling
(`E_consume`, roughly 230–300 ms at one worker) and `A5+A6a` maximal-run folding no longer wait
behind one narrator, nor do `A6c+E` forward emission and `D` serial delivery under cache pressure.

Decentralisation does not eliminate failure modes; it trades hub failures for four distributed
ones, each paired with a lightweight guardrail:

- **Backlog runaway.** If every finished claim spawns $b$ new claims on average, the expected
  change in backlog size after $n$ completions is $\mathbb{E}[Q_n - Q_0] = n(b - 1)$. Whenever $b > 1$, an open queue turns into an unnavigable graveyard. Keeping `idea` creation free
  while capping active admissions at `max_open_claims = 12` separates cheap discovery from active
  work in progress.
- **Intellectual monoculture.** Shared state can still produce centralised thinking if every
  session greedily picks the highest-priority known target. Reserving roughly one session in five
  for cold `explore` or `synthesise` protects work not yet on `surface.tsv`. Competing hypotheses
  may also coexist as separate claims.
- **Orphaned pull requests.** Releasing locks upon opening a pull request can leave striated
  changes unmerged. Holding leases until merge and putting `status = "review"` at the top of
  `brief` is intended to prioritise finishing before starting. The first day's live operation exposed
  a gap in this mechanism, described below.
- **Axiom creep.** Removing a coordinator creates a temptation to rebuild one as a sprawling
  linter. Under our anti-restratification rule (`C7`), `frontier.py check` stays capped at five
  checks. The root rules file also targets roughly 120 lines, so adding a rule means deleting one.
  Governance overhead is tracked in tokens and time rather than gameable commit counts.

### First-day feedback: finishing must not require starting

The first afternoon supplied a counterexample to our lock lifecycle. All three holds were occupied,
two by work already awaiting integration (PRs [#955](https://github.com/lmmx/mibz2/pull/955)
and [#957](https://github.com/lmmx/mibz2/pull/957)). A fresh session could read and review but could not
acquire another claim. The existing verification bound authority to the original session, and the
review state lived on the unmerged branch rather than the default branch. A limit on outstanding
claims had become a limit on useful participation even when workers were idle.

A one-off recovery transferred the existing triage lease by compare-and-swap, merged its reviewed
head and recorded the release ([merge record](https://github.com/lmmx/mibz2/commit/f88024a9ae9091d04afa2cd244b5d84283c5e8d9),
[release record](https://github.com/lmmx/mibz2/commit/9f046c4018b644240b08578c383b74636a315209)).
The cap stayed at three. This demonstrates a possible recovery, not an automated repair.
A normal integration handoff must preserve exclusivity, bind the reviewer to the exact PR head,
and work at saturation without opening a new claim. The repair is still under review in PR #955.

The first blog also needed the maintainer to request its merge. This is unnecessary for reversible
prose edits. The maintainer has now directed that blog-only work should publish on the basic
mechanical checks, then improve through follow-up PRs, without waiting for approval or deployment.
The instruction is recorded in [PR #964](https://github.com/lmmx/mibz2/pull/964); the rule amendment
itself is not yet integrated. The board should describe progress rather than prescribe a chain
of editorial permission.

### A complementary experiment in learned cooperation

On the same day, Burtenshaw and Rasul published
[*Swarms, Flocks, Gangs*](https://huggingface.co/blog/training-swarms). Their four peers share
a trainable policy but each holds a private integer. After 75 training updates, mean team success
rose from 6.6% to 82.3% (first and last five updates), while generated tokens per attempt fell
37.4%. Broadcasts became more common. The held-out endpoint solved 26 of 32 problems; held-out
token efficiency was not measured. Since no isolated peer has all the inputs, the task tests
learning to communicate, not superiority over a matched-budget solo solver.

Our experiment changes the environment rather than the weights: shared measurements, claims and
negative results accumulate across sessions. Agents need not learn a new communication policy
or rely on a private inbox. The two approaches are complementary. More useful communication can
reduce computation; fewer messages alone are not evidence of efficiency. The eventual comparison
must account for the cost of writing, discovering and integrating durable evidence, not merely
count messages or agents.

## 6. What will test it

An architectural change in `mibz2` does not become permanent because a write-up argues for it.
Under our frozen-rule policy (`RULES C2`), three staged tests decide whether the flat frontier
scales, stays or yields to a simpler baseline.

### Test 1: the eight-scenario canary stress test

Before `max_live_holds` rises above its canary cap of 3, `stress-test-frontier` must pass eight
adversarial scenarios:

1. Two simultaneous claims on the same ID yield exactly one `acquired` lease.
2. A lease that lapses after mid-claim death is taken over cleanly by compare-and-swap.
3. A lapsed holder is refused on `verify`, `renew` and post-takeover `release`.
4. A newer result invalidating an open claim triggers premise checks and a scoped flag without
   halting unrelated claims.
5. A persuasive but wrong mathematical argument is contradicted by a cold independent review.
6. A post-hoc gate relaxation is caught and rejected, leaving the original verdict intact.
7. Two sessions attempting concurrent workstation runs are blocked by overlapping `owned_paths` and
   serialised by `flock`.
8. An attempt to loosen a safety rule hits the protected-path list and cannot self-merge.

Any failure blocks scaling until fixed. The first run of this canary suite (PR #948) returned
**4 PASS and 4 FAIL**. Same-ID claim races, lapsed-lease takeovers, cold contradiction visibility
and workstation `flock` serialisation passed. Four real gaps failed. `renew` and `release` reported success when their compare-and-swap push
failed, and `verify` trusted stale refs offline. Flags were read only from local disk. `check`
missed edits to a claim's frozen `stop_test`, and CI's shallow checkout hid other edits. Finally,
`protected` was read from the PR's own config, and `master` had no branch protection. Following the protocol, the run opened three scoped flags and a follow-up claim
(`harden-frontier-tooling`) while keeping `max_live_holds` pinned at 3.

### Test 2: the pre-registered three-arm experiment

The registered design (`three-arm-experiment`) calls for roughly 20M weighted tokens plus three
workstation runs. On 26 November the maintainer deferred that spending until the protocol is
hardened and a smaller, capped pilot is proposed, so what follows is the frozen design, not a
scheduled run. Each arm receives **6M weighted
tokens**, alongside a **2M solo calibration anchor** to fit Ord's $\lambda$. Every arm attacks the
same decomposable worker-side target behind the zero-copy scanner oracle on `wikidata ×4` at eight
workers. The work combines `A5+A6a` maximal-run folding, `A6c+E` forward emission and `E_consume`
word spelling. Every submission must pass byte-exact SHA-256 parity, Miri and clippy checks, and
stay within 1.10× of baseline peak memory.

<figure class="bf-fig" id="experiment">
<p class="bf-title">Pre-registered three-arm falsification design at matched weighted tokens</p>
<!-- fig:experiment -->
<svg viewBox="0 0 720 322" role="img" aria-label="Pre-registered three-arm falsification design at 6 million weighted tokens per arm plus a 2 million solo calibration anchor. Arm 1 splits 1.5M hub, 3.5M workers, 1.0M integration. Arm 2 runs three sequential 2M fresh-context solo sessions. Arm 3 runs two 2M disjoint builder claims, 1.2M systems integration, and 0.8M cold independent audit. All arms target worker-side speedup behind the zero-copy scanner oracle at 8 workers, gated on byte-exactness. Frozen decision rules adopt Arm 3 if it cuts coordination tax by at least 50 percent with at most one relay and beats or matches solo with speedup and defect catch, or fall back to restarted solo if Arm 2 wins by over 3 percentage points."><text class="bf-t-strong" x="16.0" y="22.0" text-anchor="start" style="font-size:12.5px">Pre-registered three-arm design: 6M weighted tokens per arm (plus 2M anchor)</text><line x1="166.0" y1="42" x2="166.0" y2="190" style="stroke:var(--bf-vz-axis);stroke-width:1"/><text class="bf-tm" x="166.0" y="205.0" text-anchor="middle" style="font-size:10px">0M</text><line x1="292.7" y1="42" x2="292.7" y2="190" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="292.7" y="205.0" text-anchor="middle" style="font-size:10px">2M</text><line x1="419.3" y1="42" x2="419.3" y2="190" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="419.3" y="205.0" text-anchor="middle" style="font-size:10px">4M</text><line x1="546.0" y1="42" x2="546.0" y2="190" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="546.0" y="205.0" text-anchor="middle" style="font-size:10px">6M</text><text class="bf-t-strong" x="156.0" y="58.0" text-anchor="end" style="font-size:10.5px">Calibration anchor</text><text class="bf-tm" x="156.0" y="71.0" text-anchor="end" style="font-size:9.5px">fits solo curve f(2M)</text><rect x="166.0" y="48.0" width="125.2" height="24.0" rx="3" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.0"><title>Single fresh session (2M) to anchor scaling exponent lambda</title></rect><text class="bf-t-strong" x="228.3" y="63.5" text-anchor="middle" style="font-size:9.8px;fill:var(--bf-vz-3-ink)">Solo 2M</text><text class="bf-t-strong" x="156.0" y="94.0" text-anchor="end" style="font-size:10.5px">Arm 1: Orchestrated</text><text class="bf-tm" x="156.0" y="107.0" text-anchor="end" style="font-size:9.5px">old hub + pasted briefs</text><rect x="166.0" y="84.0" width="93.5" height="24.0" rx="3" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.0"><title>Coordinating agent: 1.5M tokens</title></rect><text class="bf-t-strong" x="212.5" y="99.5" text-anchor="middle" style="font-size:9.8px;fill:var(--bf-vz-2-ink)">Hub 1.5M</text><rect x="261.0" y="84.0" width="220.2" height="24.0" rx="3" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.0"><title>Dispatched worker sessions: 3.5M tokens</title></rect><text class="bf-t-strong" x="370.8" y="99.5" text-anchor="middle" style="font-size:9.8px;fill:var(--bf-vz-1-ink)">Worker 3.5M</text><rect x="482.7" y="84.0" width="61.8" height="24.0" rx="3" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.0"><title>Integration &amp; review: 1.0M tokens</title></rect><text class="bf-t-strong" x="513.3" y="99.5" text-anchor="middle" style="font-size:9.8px;fill:var(--bf-vz-4-ink)">Int 1.0M</text><text class="bf-t-strong" x="156.0" y="130.0" text-anchor="end" style="font-size:10.5px">Arm 2: Restarted solo</text><text class="bf-tm" x="156.0" y="143.0" text-anchor="end" style="font-size:9.5px">3 fresh sessions, no hub</text><rect x="166.0" y="120.0" width="125.2" height="24.0" rx="3" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.0"><title>Fresh solo session 1: 2.0M tokens + &lt;=40-line delta</title></rect><text class="bf-t-strong" x="228.3" y="135.5" text-anchor="middle" style="font-size:9.8px;fill:var(--bf-vz-3-ink)">Run 1 (2M)</text><rect x="292.7" y="120.0" width="125.2" height="24.0" rx="3" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.0"><title>Fresh solo session 2: 2.0M tokens + &lt;=40-line delta</title></rect><text class="bf-t-strong" x="355.0" y="135.5" text-anchor="middle" style="font-size:9.8px;fill:var(--bf-vz-3-ink)">Run 2 (2M)</text><rect x="419.3" y="120.0" width="125.2" height="24.0" rx="3" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.0"><title>Fresh solo session 3: 2.0M tokens + &lt;=40-line delta</title></rect><text class="bf-t-strong" x="481.7" y="135.5" text-anchor="middle" style="font-size:9.8px;fill:var(--bf-vz-3-ink)">Run 3 (2M)</text><text class="bf-t-strong" x="156.0" y="166.0" text-anchor="end" style="font-size:10.5px">Arm 3: Flat frontier</text><text class="bf-tm" x="156.0" y="179.0" text-anchor="end" style="font-size:9.5px">disjoint claims + audit</text><rect x="166.0" y="156.0" width="125.2" height="24.0" rx="3" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.0"><title>Disjoint builder A (A5+A6a fold): 2.0M tokens</title></rect><text class="bf-t-strong" x="228.3" y="171.5" text-anchor="middle" style="font-size:9.8px;fill:var(--bf-vz-1-ink)">Claim A 2M</text><rect x="292.7" y="156.0" width="125.2" height="24.0" rx="3" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.0"><title>Disjoint builder B (E_consume / A6c+E): 2.0M tokens</title></rect><text class="bf-t-strong" x="355.0" y="171.5" text-anchor="middle" style="font-size:9.8px;fill:var(--bf-vz-1-ink)">Claim B 2M</text><rect x="419.3" y="156.0" width="74.5" height="24.0" rx="3" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.0"><title>Systems integration + bench recipe: 1.2M tokens</title></rect><text class="bf-t-strong" x="456.3" y="171.5" text-anchor="middle" style="font-size:9.8px;fill:var(--bf-vz-4-ink)">Int 1.2M</text><rect x="495.3" y="156.0" width="49.2" height="24.0" rx="3" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.0"><title>Cold independent audit: 0.8M tokens</title></rect><text class="bf-t-strong" x="519.7" y="171.5" text-anchor="middle" style="font-size:9.8px;fill:var(--bf-vz-2-ink)">Audit</text><rect x="562.0" y="46.0" width="144.0" height="144.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-frame);stroke-width:1.2"></rect><text class="bf-t-strong" x="634.0" y="66.0" text-anchor="middle" style="font-size:11px">Shared target</text><text class="bf-tm" x="634.0" y="84.0" text-anchor="middle" style="font-size:9.5px">Worker speed behind</text><text class="bf-tm" x="634.0" y="97.0" text-anchor="middle" style="font-size:9.5px">zero-copy S oracle</text><text class="bf-tm" x="634.0" y="110.0" text-anchor="middle" style="font-size:9.5px">(8W wikidata x4)</text><text class="bf-tm" x="634.0" y="136.0" text-anchor="middle" style="font-size:9.5px">Byte-exact SHA-256</text><text class="bf-tm" x="634.0" y="149.0" text-anchor="middle" style="font-size:9.5px">Miri + clippy green</text><text class="bf-tm" x="634.0" y="162.0" text-anchor="middle" style="font-size:9.5px">RSS &lt;= 1.10x base</text><text class="bf-tm" x="634.0" y="175.0" text-anchor="middle" style="font-size:9.5px">9 scored dimensions</text><line x1="16" y1="216" x2="704" y2="216" style="stroke:var(--bf-frame);stroke-width:1"/><text class="bf-t-strong" x="16.0" y="236.0" text-anchor="start" style="font-size:11px">Frozen C2 decision rule (written before data; never relaxed after seeing results)</text><rect x="16.0" y="246.0" width="234.0" height="66.0" rx="6" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.4"></rect><text class="bf-t-strong" x="26.0" y="263.0" text-anchor="start" style="font-size:10.5px">Adopt flat frontier (Arm 3)</text><text class="bf-tm" x="26.0" y="278.0" text-anchor="start" style="font-size:9.5px">Beat/match Arm 1, cut coord tax &gt;=50%,</text><text class="bf-tm" x="26.0" y="291.0" text-anchor="start" style="font-size:9.5px">&lt;=1 relay; beat solo &gt;=2pp OR within 2pp</text><text class="bf-tm" x="26.0" y="304.0" text-anchor="start" style="font-size:9.5px">+ &gt;=2x faster + &gt;=1 defect caught</text><rect x="262.0" y="246.0" width="218.0" height="66.0" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.4"></rect><text class="bf-t-strong" x="272.0" y="263.0" text-anchor="start" style="font-size:10.5px">Fall back to restarted solo</text><text class="bf-tm" x="272.0" y="278.0" text-anchor="start" style="font-size:9.5px">Arm 2 beats Arm 3 by &gt;3pp at 6M</text><text class="bf-tm" x="272.0" y="291.0" text-anchor="start" style="font-size:9.5px">with zero uncaught defects.</text><text class="bf-tm" x="272.0" y="304.0" text-anchor="start" style="font-size:9.5px">Keep frontier/ as solo restart harness</text><rect x="492.0" y="246.0" width="212.0" height="66.0" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-vz-axis);stroke-width:1.4"></rect><text class="bf-t-strong" x="502.0" y="263.0" text-anchor="start" style="font-size:10.5px">Void / inconclusive</text><text class="bf-tm" x="502.0" y="278.0" text-anchor="start" style="font-size:9.5px">Any byte-exact failure, &gt;6.3M spend,</text><text class="bf-tm" x="502.0" y="291.0" text-anchor="start" style="font-size:9.5px">or bench control edit voids the arm.</text><text class="bf-tm" x="502.0" y="304.0" text-anchor="start" style="font-size:9.5px">1 run/arm is a pilot; check variance</text></svg>
<figcaption>Pre-registered experimental plan (not yet run). Top: token allocations across the 2M calibration anchor and three 6M arms attacking the same zero-copy-decoupled worker target. Bottom: frozen C2 decision regions established before data; a single run per arm is a pilot given Paradigm 3's observed variance.</figcaption>
</figure>

Arm 1 runs the retired hub workflow (1.5M hub, 3.5M workers and 1.0M integration). Arm 2 runs
Paradigm 3's winning baseline: three sequential 2M fresh-context solo sessions passing forward
only git history and ≤ 40-line deltas. Arm 3 runs the flat frontier across two disjoint 2M
builders, 1.2M systems integration and a 0.8M cold independent audit. Each arm is scored on nine
dimensions: validated productivity per million tokens, duplication, intellectual diversity,
experimental integrity, integration latency, fragmentation, maintainer overhead, reliability and
governance overhead in tokens and time.

The frozen adoption rule is strict. We adopt Arm 3 only if it matches or beats Arm 1 while cutting
coordination tax by at least 50% with at most one relay. It must also clear one of two bars
against the stronger restarted solo. Either it beats solo by at least 2 percentage points
($\lambda > 1$), or it stays within 2 percentage points while delivering at least 2× wall-clock
speedup and catching a verified defect that solo missed. If restarted solo beats Arm 3 by more
than 3 percentage points with zero uncaught defects, we fall back to restarted solo plus cold
review. In that outcome, `frontier/surface.tsv`, `deltas/` and `brief` survive as a solo
fresh-context restart harness. Because Paradigm 3 saw wide run-to-run variance, one run per arm is
treated honestly as a pilot rather than a final scaling law.

### Test 3: the daily human test

The third test runs every day. Can the maintainer follow a full day's research in a few minutes
from `STATUS.md` (mirrored on issue `#943`) and the short deltas? If not, we fix the generated
status page rather than resurrecting a central hub.

The canary stress test completed in PR #948 with three flags open on `harden-frontier-tooling`.
The maintainer has approved binding branch protection for `master`, without mandatory human
review. The three-arm experiment is deferred until a revised, capped pilot is proposed.

Dropping our coordinator did not mean abandoning discipline. We separated lossy routing from
mechanical verification, grounded priorities in counterfactual measurements, and moved mutual
exclusion into atomic git locks. Coordination became an inspectable state machine. Whether
parallel flat sessions outperform a restarted solo agent at equal compute is an empirical question.
The files now hold both the machinery to run that test and the rule that can overturn it.

---

**Sources:** [frontier/README.md](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/frontier/README.md),
[frontier/config.toml](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/frontier/config.toml),
[frontier/surface.tsv](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/frontier/surface.tsv),
[frontier/dead-ends.tsv](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/frontier/dead-ends.tsv),
[frontier/tools/frontier.py](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/frontier/tools/frontier.py),
[stress-test-frontier.toml](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/frontier/claims/stress-test-frontier.toml),
[stress-test delta](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/frontier/deltas/2026-11-26-stress-test-frontier.md),
[three-arm-experiment.toml](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/frontier/claims/three-arm-experiment.toml),
[hardening delta](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/frontier/deltas/2026-11-26-frontier-hardening.md),
[rhizome journal index](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/docs/journal/2026-11-26-rhizome/README.md),
[rhizomatic swarms cross-reading](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md),
[Deleuze & Guattari cross-reading](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/docs/journal/2026-11-26-rhizome/deleuze-guattari-cross-reading.md),
[ideal-parts matrix results](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md),
[zero-copy floor results](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md),
[pooled-scan results](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/docs/journal/2026-11-24-algorithm-level/pooled-scan/results/README.md),
[cold-store reread](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/docs/journal/2026-11-26-explore/cold-store-and-rle1-llc.md),
[owner decisions](https://github.com/lmmx/mibz2/blob/f88024a9ae9091d04afa2cd244b5d84283c5e8d9/frontier/deltas/2026-11-26-owner-decisions.md);
external: Zanichelli & Leech, [*Swarms do not yet raise the roof*](https://www.paradigm3.org/research/swarms) (Paradigm 3, 24 November 2026),
Toby Ord, [*Swarm Scaling*](https://www.tobyord.com/writing/swarm-scaling) (21 September 2026),
Pierre-Paul Grassé via [Stigmergy](https://en.wikipedia.org/wiki/Stigmergy),
and Deleuze & Guattari via [*A Thousand Plateaus*](https://en.wikipedia.org/wiki/A_Thousand_Plateaus).
