---
title: "Call on mibz2: cutting instructions from bzip2 decoding with an autonomous swarm"
image: assets/social/blog-2026-11-04-autonomous-agent-team.png
description: "Each of four agents does one part of the research. A GitHub doorbell lets them hand work to each other directly, without me acting as their message bus."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Call on mibz2: cutting instructions from bzip2 decoding with an autonomous swarm

Each of four agents does one part of the research. A GitHub doorbell lets them hand work to each
other directly, without me acting as their message bus.
{: .post-dek }

4 November 2026 · 10 min read · [Meta](#7-what-this-establishes)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

Can I let a team of agents carry research from one checkpoint to the next, while keeping
the evidence inspectable and the scope of the work under my control? In mibz2, my bzip2
decoder, I am using a shared repository to do that. The working claim is quite specific:
agents can hand routine research to one another when each handoff names the work, the
evidence and the rules for acting on it. The recorded exchanges show that this handoff
works; its effect on research throughput remains to be measured.

The research itself follows [the compact30 constructor](2026-11-04-compact30-constructor.md).
Compact30 stores the navigation table used during decoding in a smaller form. Building
that table involves several kinds of work: finding instructions to remove, expressing a
change in Rust, inspecting what the compiler emits, and checking that the resulting
records are still correct. A local saving also has to survive the accounting for the
whole builder.

I have given those responsibilities to four agents. They share one way of working:
read a pinned result, do an assigned piece of work, and publish the next result through
GitHub. Their responsibilities differ; the shape of the handoff stays the same. The
sections below follow it from the team, through a checkpoint and its delivery, to the
evidence that lets the next step proceed.

## 1. One workflow, four responsibilities

The Coordinator runs in Claude Code. It implements the registered changes, compiles
them, runs the tests and archives the evidence. When a result is ready, it publishes a
checkpoint: a short message saying what has been built and where the evidence lives.

The Guardian and the Researcher are the paired readers of that work. They read the
same checkpoint with different questions in mind. The Guardian asks what follows under
the agreed research rules: which gates the evidence meets, which work can continue, and
which decisions are reserved for me. The Researcher asks whether the reasoning is sound,
whether another explanation fits, and whether the team is pursuing the most useful
direction. That includes questioning the Guardian's interpretation when the evidence
warrants it.

Meanwhile, the Optimiser studies the instruction-reduction
problem itself. It works from the source, compiled listings and execution counts, develops
transformations and keeps the canonical ledger of their savings. Its proposals give the
Coordinator concrete ideas to implement within the authorised research programme.

All four publish through the repository. A proposal, a checkpoint and a review each
identify something the next reader can inspect. The roles are written down in
[the identity record](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/ROLES.md),
along with the branches they own. The Guardian, the Researcher and the Optimiser all run Astra,
a model I use through ChatGPT, each as a ChatGPT Work task at a fixed reasoning-effort setting
(the badges in the figure below). Sharing a model doesn't make them interchangeable; their
different responsibilities are what distinguish them.

To reproduce the setup, these are the exact settings:

| Role | Runs as | Model | Reasoning effort | Started by |
| --- | --- | --- | --- | --- |
| Coordinator | Claude Code session | Claude, as configured for the session | as configured for the session | a subscription to the PRs it follows |
| Guardian | ChatGPT Work task, *Compact30 (W) Guardian* | Astra | Extra High | any PR comment; acts only on a `CHECKPOINT` |
| Optimiser | ChatGPT Work task, *Compact30 (W) Optimiser* | Astra | Ultra | any PR comment; acts only on a `CHECKPOINT` addressed to it |
| Researcher | ChatGPT Work task, *Compact30 (W) Researcher* | Astra | Extra High | any PR comment; acts only on a message addressed to it |

For another project, I would begin with those responsibilities and their outputs.
Who makes the change? Who checks its evidence against the agreed criteria? Who questions
the argument? Who develops the next idea? Giving each question an owner makes the
handoffs concrete.

<figure class="bf-fig">
<p class="bf-title">The team: three Astra agents and Claude Code, joined by one repository</p>
<svg viewBox="0 0 720 360" role="img" aria-label="Four agents connected to one GitHub repository. Above it, three Astra agents, each a ChatGPT Work task: the Guardian, which applies the charter's gates; the Optimiser, which searches the listings for instructions to cut; and the Researcher, which questions the reasoning. Below it, the Coordinator, Claude Code, which implements and tests each change and posts it as a checkpoint."><rect x="0" y="0" width="720" height="150" rx="14" class="bf-group"/><g transform="translate(16 11) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="42" y="25" class="bf-ts">Astra · ChatGPT Work</text><text x="704" y="25" class="bf-tm" text-anchor="end">each role is an event-triggered task, started by a PR comment</text><rect x="16" y="40" width="216" height="96" rx="10" class="bf-card"/><g transform="translate(30 52) scale(0.6667)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="54" y="65" class="bf-ts">Guardian</text><text x="30" y="88" class="bf-tm">applies the charter's gates</text><text x="30" y="104" class="bf-tm">to every checkpoint</text><rect x="30" y="114" width="80" height="16" rx="8" class="bf-pill"/><text x="70.0" y="125.5" class="bf-ttiny" text-anchor="middle">Extra High</text><rect x="252" y="40" width="216" height="96" rx="10" class="bf-card"/><g transform="translate(266 52) scale(0.6667)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="290" y="65" class="bf-ts">Optimiser</text><text x="266" y="88" class="bf-tm">searches the listings for</text><text x="266" y="104" class="bf-tm">instructions to cut</text><rect x="266" y="114" width="49" height="16" rx="8" class="bf-pill"/><text x="290.5" y="125.5" class="bf-ttiny" text-anchor="middle">Ultra</text><rect x="488" y="40" width="216" height="96" rx="10" class="bf-card"/><g transform="translate(502 52) scale(0.6667)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="526" y="65" class="bf-ts">Researcher</text><text x="502" y="88" class="bf-tm">questions the reasoning</text><text x="502" y="104" class="bf-tm">behind every checkpoint</text><rect x="502" y="114" width="80" height="16" rx="8" class="bf-pill"/><text x="542.0" y="125.5" class="bf-ttiny" text-anchor="middle">Extra High</text><path d="M124 142V174m-5-5 5 5 5-5M119 147l5-5 5 5" class="bf-arrow"/><path d="M360 142V174m-5-5 5 5 5-5M355 147l5-5 5 5" class="bf-arrow"/><path d="M596 142V174m-5-5 5 5 5-5M591 147l5-5 5 5" class="bf-arrow"/><rect x="0" y="180" width="720" height="62" rx="12" class="bf-panel"/><g transform="translate(18 200) scale(0.9167)"><path class="bf-logo-ink" d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></g><text x="52" y="206" class="bf-ts">The repository</text><text x="52" y="226" class="bf-tm">messages are pull-request comments · each agent keeps its state on its own branch · every claim pins a commit</text><path d="M360 248V276m-5-5 5 5 5-5M355 253l5-5 5 5" class="bf-arrow"/><rect x="170" y="282" width="380" height="76" rx="12" class="bf-card-claude"/><g transform="translate(188 300) scale(1.0833)"><path class="bf-logo-claude" d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></g><text x="228" y="310" class="bf-ts">Coordinator · Claude Code</text><text x="228" y="330" class="bf-tm">implements and tests each change,</text><text x="228" y="346" class="bf-tm">then posts it as a checkpoint</text></svg>
<figcaption>The Coordinator implements and publishes checkpoints. The Guardian and the Researcher examine process and reasoning; the Optimiser develops the instruction-reduction work. Each role reads and writes through the same repository.</figcaption>
</figure>

## 2. Give the next reader an exact result

A checkpoint is a package of evidence with an address. For a constructor change, that
package includes the source and its baseline, the compiled listings, the toolchain,
test records, execution counts and the accounting that connects them.

The address is a commit. A branch is where work continues; the commit identifies
the particular version being reviewed. A review cites that version and the governing
documents it applied. Long reports and scripts live in committed files, with a short
comment linking to them.

Suppose the Optimiser predicts that changing a descriptor will remove a load from the
inner loop. The Coordinator's checkpoint lets another agent ask what actually happened
after compilation. Was the load removed? Did setup acquire more work? Did a changed caller
add a spill? How often does each path execute? The compiled code and counters let the
reader follow the claim beyond the sentence that announces it.

Each role owns its publication branch. The Coordinator owns the implementation branches;
the other agents own their reports, decisions and ledgers. That gives each result a
clear writer and keeps the Coordinator's source changes separate from a review of them.

The same arrangement makes a resumed task useful. Its important working state is explicit
in the repository: what it has reviewed, which evidence it used and what remains pending.
Conversation history can help an investigation, while the committed record lets another
reader inspect it without having taken part.

## 3. Let the repository ring the doorbell

A comment carries the handoff; an event starts the next reader. The messages use a small
header. For a checkpoint that needs both process and reasoning reviews, I can address
both readers:

<pre><code>FROM: coordinator
TO: guardian, researcher
RE: prototype 2 @ 29dd071f8787a28560a7a081fd7ac0ef13991466
TYPE: CHECKPOINT</code></pre>

The body says what needs attention and links the evidence. One standing draft PR carries
messages across the project, and individual experiments and reviewers have their own PRs.
The shared channel is #145: a checkpoint there points to an experiment's evidence, and
a short review links back to the report on the reviewer's branch. The PRs give each piece
a home; the comment and its evidence pin connect the pieces.
The [protocol](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/PROTOCOL.md)
gives the main message types a small, practical vocabulary:

| Type | What it carries |
| --- | --- |
| <code>CHECKPOINT</code> | A pinned result ready for assessment |
| <code>REVIEW</code> | An assessment and its evidence |
| <code>PROPOSAL</code> | An idea for consideration |
| <code>DECISION</code> | A decision within the sender's delegated remit |
| <code>ACK</code> | What was understood and what follows |
| <code>ESCALATION</code> | A matter requiring a reserved decision |

The Coordinator subscribes to the relevant PRs. The three Astra roles run as ChatGPT Work
tasks triggered by PR comment events in this repository. Those are two delivery mechanisms
for the same workflow: a result appears, a recipient reads it, and the recipient
publishes its response.

In the events observed here, the payload supplies the comment's ID rather than its text.
A task first fetches that exact comment, then applies its configured filter. The
Researcher's filter looks for its role in the address line. Other tasks also filter
by message type. To obtain paired reviews, the checkpoint must reach both configured
readers; the role diagram alone does not establish that coverage.

A comment can wake a task without giving it any work. A review is itself another event,
so a checkpoint-only task checks its type and stops. A task with an address-based filter
checks whom the message names. The useful boundary is the filter at the start of the
run: it decides whether this particular message belongs to the role.

<figure class="bf-fig">
<p class="bf-title">One cycle: a checkpoint, two readers, the next step</p>
<svg viewBox="0 0 720 330" role="img" aria-label="One cycle. The Coordinator posts a checkpoint as a pull-request comment. That event starts the Guardian and the Researcher, which each post a review. Their reviews wake the Coordinator, which takes the next step. Meanwhile the Optimiser searches the compiled listings for instructions to cut, and its proposals become new checkpoints."><rect x="0" y="70" width="196" height="100" rx="10" class="bf-card-claude"/><g transform="translate(14 84) scale(0.7500)"><path class="bf-logo-claude" d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></g><text x="40" y="98" class="bf-ts">1 · Checkpoint</text><text x="14" y="124" class="bf-tm">a result at a pinned commit,</text><text x="14" y="141" class="bf-tm">posted as a PR comment</text><rect x="262" y="0" width="196" height="100" rx="10" class="bf-card"/><g transform="translate(276 14) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="302" y="28" class="bf-ts">2 · Guardian</text><text x="276" y="54" class="bf-tm">applies the gates,</text><text x="276" y="71" class="bf-tm">posts a REVIEW</text><rect x="262" y="140" width="196" height="100" rx="10" class="bf-card"/><g transform="translate(276 154) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="302" y="168" class="bf-ts">2 · Researcher</text><text x="276" y="194" class="bf-tm">tests the reasoning,</text><text x="276" y="211" class="bf-tm">posts a REVIEW</text><rect x="524" y="70" width="196" height="100" rx="10" class="bf-card-claude"/><g transform="translate(538 84) scale(0.7500)"><path class="bf-logo-claude" d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></g><text x="564" y="98" class="bf-ts">3 · Next step</text><text x="538" y="124" class="bf-tm">the Coordinator wakes,</text><text x="538" y="141" class="bf-tm">acts on both reviews</text><path d="M200 120H230V50H254m-5-5 5 5-5 5M230 120V190H254m-5-5 5 5-5 5" class="bf-arrow"/><path d="M462 50H492V190H462M492 120H516m-5-5 5 5-5 5" class="bf-arrow"/><text x="229" y="134" class="bf-ttiny" text-anchor="end">event</text><text x="496" y="134" class="bf-ttiny">event</text><rect x="0" y="264" width="720" height="62" rx="12" class="bf-lane"/><g transform="translate(16 278) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="42" y="292" class="bf-ts">Meanwhile · Optimiser</text><text x="42" y="312" class="bf-tm">searches the compiled listings for instructions to cut; once implemented, its proposals become new checkpoints</text></svg>
<figcaption>A repository event starts a reader. The reader checks the message and its saved state, does the work assigned to it, then publishes a result and records the receipt. Each arrow is a pull-request comment; no agent polls, and no message passes through me.</figcaption>
</figure>

## 4. Save a receipt for the work

Once the message matches, the task checks its cursor: a file recording the work already
processed. A comment ID identifies the notification. The review contract also keys
technical work by PR, candidate commit and evidence hash, so a repeated request can be
distinguished from a revised candidate.

The task then loads the state and evidence it needs, does the work, publishes its result
and records the receipt. If several comments arrive together, each gets its own check.
One already processed message leaves the others available for assessment.

This is a small amount of state with an important purpose. The next execution has a
durable answer to “have I handled this?” I can also follow the receipt to the published
result and see what was concluded.

Publication and recording the receipt are separate operations. For another team adopting
this pattern, I would make recovery at that boundary explicit: if a result was published
but the cursor was not saved, locate the published result before repeating the write.
Keeping its commit and outgoing comment ID makes that check possible. The cursor supports
deduplication; exactly-once publication also needs that interruption case to be handled.

Pending work is explicit in the same way. A review stays in the queue until a pinned
assessment appears. Work that depends on it waits for that result, while other authorised
work can continue. The queue gives the team a shared account of progress.

Branch ownership assigns responsibility to one role. Concurrent executions of that role
also need serialisation or a way to reconcile competing writes. I treat that as part of
the execution design, alongside recovery, rather than a guarantee supplied by the cursor.
The recorded exchanges demonstrate delivery; interruption recovery and overlapping
executions need their own validation before I can claim those guarantees.

## 5. Decide what a message may authorise

For these handoffs to carry useful work, the recipients need to know what they may do.
I put that in the [research charter](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/2026-11-04-compact30-research-charter.md).
It fixes the representation, the reference builder and the baseline versions. Each
prototype has a registered design, acceptance criteria and a bounded set of attempts.

Within that scope, the Coordinator can implement and test without asking me at every
step. The Guardian can apply the agreed rules and make the decisions delegated to it.
I keep the decisions that expand the work, such as adding a prototype, approving a new
unsafe operation or integrating into production.

The rule for messages follows from that split: **the repository can narrow authority;
widening comes from me in the Coordinator's session.** A supported finding can hold a
component's qualification. A proposal can suggest a new direction. Carrying the proposal
out depends on the authority already granted or on my explicit decision.
I approve the charter and its amendments there; the Coordinator records the decision's
provenance, and reviews cite the governing revision they applied.

Under the protocol, a review that holds qualification returns to the Coordinator on a
subscribed PR. It logs the finding and keeps the dependent work pending until the required
resolution. The checkpoint filter of another role is not the route for that decision.

Every agent uses my GitHub account. The sender label therefore names a role rather than
authenticating it. Recipients check what the message requests against their standing
instructions. This is a rule for their behaviour, distinct from the permissions enforced
by the tools. The protocol also records time-limited interaction restrictions and locks
on named PRs. Those controls limit participation; the charter defines what a participant's
message can authorise.

The channel's test cases make that boundary inspectable. They include a finding, a hold,
an acknowledgement and requests for actions reserved to me. Their value is in checking
both halves of the exchange: the recipient understood the message, and its response stayed
within the permitted next step.

## 6. Make the review reproducible, and its independence explicit

Compact30's qualification figure counts machine work across the whole constructor and
everything it calls. Compiled path lengths are weighted by execution counts on the
corpus. Setup, fallback paths, callers, callees and added memory operations all belong in
the account. A saving moved from one place to another must be counted where it lands.

Correctness has its own evidence: byte-exact output against the reference builder,
release-mode safety arguments, targeted Miri checks for undefined behaviour, and
mutation tests that deliberately break an invariant to show that a test detects it.
These requirements give the Guardian and Researcher specific claims to examine.

A report states what was inspected and what was reproduced. An implementer's journal
contains the implementer's conclusions; the reviewer works from the source, listings,
counters and test records to derive its own. That lets another reader replay the
important parts of the argument.

Before components are combined, the charter asks for an independent review from a
pinned evidence index. This has a different input boundary from an ongoing role.
The continuing Researcher knows the team's discussion and may have contributed ideas.
The independent reviewer receives the permitted evidence and governing specifications,
with earlier involvement disclosed.

A new execution and an independent context are separate properties. To make a review
blind, I need to control which history is supplied to it as well as which files it reads.
The required review context excludes earlier discussion and verdicts. Its findings then
have to be resolved before the components can be combined.

For a team working on another problem, this is a useful way to specify independence:
write down the reviewer's inputs, its earlier involvement and what it reproduced.
A separate role provides a separate responsibility; a recorded evidence boundary explains
what makes a particular assessment independent.

## 7. What this establishes

The [channel log](https://github.com/lmmx/mibz2/blob/688d4f7c4cdd813d13ed2b06708e47cbfdce2ca4/docs/journal/compact30-channel/log.md)
records a completed event-delivery loop. A GitHub comment started the Guardian task;
the Guardian fetched it, published an assessment and saved its cursor; the review
reached the Coordinator. That setup test included confirmation of the commit identity.
The log also records the subsequent D008 assessment of real pinned checkpoints and
the Coordinator's handoff of an evidence index.

That real exchange gives the abstract handoff an address. The Guardian's review on #145
identified the descriptor candidate at commit 29dd071f and linked its assessment at
62a313a8. The Coordinator recorded it in the channel log and supplied the independent
review index at 18ca5d4c. The next step was to obtain that independent assessment; the
checkpoint review alone did not complete the pre-integration requirement.

That establishes delivery and review activity at the scale recorded. The independent
pre-integration assessment is still outstanding in this snapshot. The effect of the
team's arrangement on research throughput remains to be measured, and the constructor's
performance will be established by its own experiments.

The method a reader can transfer is already concrete. Give each responsibility an owned
output. Make a handoff name an exact result. Use events to bring that result to the next
reader, and save enough state to recognise completed work. Define the authority that
travels with a message and the evidence required for a conclusion.

My part is to choose the scope and make the decisions reserved to me. The agents can
carry routine work between those decisions through the repository. A reader arriving
later can follow the same route from the question, through the evidence, to the result
that justified the next step.

---

**Research records:** [Role identities](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/ROLES.md) · [Research charter](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/2026-11-04-compact30-research-charter.md) · [Channel protocol](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/PROTOCOL.md) · [Review contract](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/REVIEWER.md) · [Recorded exchanges](https://github.com/lmmx/mibz2/blob/688d4f7c4cdd813d13ed2b06708e47cbfdce2ca4/docs/journal/compact30-channel/log.md).

