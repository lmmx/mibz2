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

4 November 2026 · 10 min read · [Meta](#8-where-it-stands)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

[The last post](2026-11-04-compact30-constructor.md) was about compact30, my effort to give the
decoder's navigation table a smaller representation. Four agents did the research behind it, but
until today they could only work together if I copied each one's messages to the next by hand.

This post is about how they now pass work between themselves through the repository: who does
what, how a message gets from one agent to another, and what I learnt from an afternoon of running
it. It covers the setup and its first full cycle. Whether it actually makes the research faster,
we'll have to wait and see.

## 1. A tale of four agents

The Coordinator is Claude Code. It writes the code for each change in the plan, compiles it, runs
the tests and keeps the results. When a result is ready it posts a checkpoint: a comment saying
what was built and which commit holds the evidence.

The Guardian and the Researcher both read every checkpoint, looking for different things. The
Guardian checks the result against the research charter: which gates it passes, what work can
follow, and what has to come to me. The Researcher looks at the reasoning: whether the argument
holds, whether something else explains the numbers, whether there's a better direction, and
whether the Guardian has got it right.

The Optimiser works on the instruction counts themselves. It reads the source, the compiled
listings and the execution counts, works out transformations that remove instructions, and keeps
the ledger of what each one saves. Its proposals are what the Coordinator builds next.

The Guardian, the Researcher and the Optimiser all run Astra, a model I use through ChatGPT, each
as a ChatGPT Work task with its own reasoning-effort setting. What separates them is what they're
responsible for, not the model. The roles, and the branch each one owns, are written down in
[ROLES.md](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/ROLES.md).
To reproduce the setup, these are the settings:

| Role | Runs as | Model | Reasoning effort | Started by |
| --- | --- | --- | --- | --- |
| Coordinator | <span class="ag-claude">Claude Code</span> session | <span class="ag-claude">Claude</span>, as configured for the session | <span class="ag-chip ag-high">High</span> | a subscription to the PRs it follows |
| Guardian | <span class="ag-astra">ChatGPT Work</span> task, *Compact30 (W) Guardian* | <span class="ag-astra">Astra</span> | <span class="ag-chip ag-xh">Extra High</span> | any PR comment; acts only on a `CHECKPOINT` |
| Optimiser | <span class="ag-astra">ChatGPT Work</span> task, *Compact30 (W) Optimiser* | <span class="ag-astra">Astra</span> | <span class="ag-chip ag-ultra">Ultra</span> | any PR comment; acts only on a `CHECKPOINT` addressed to it |
| Researcher | <span class="ag-astra">ChatGPT Work</span> task, *Compact30 (W) Researcher* | <span class="ag-astra">Astra</span> | <span class="ag-chip ag-xh">Extra High</span> | any PR comment; acts only on a message addressed to it |

<figure class="bf-fig">
<p class="bf-title">The team: three Astra agents and Claude Code, joined by one repository</p>
<svg viewBox="0 0 720 360" role="img" aria-label="Four agents connected to one GitHub repository. Above it, three Astra agents, each a ChatGPT Work task: the Guardian, which applies the charter's gates; the Optimiser, which searches the listings for instructions to cut; and the Researcher, which questions the reasoning. Below it, the Coordinator, Claude Code, which implements and tests each change and posts it as a checkpoint."><rect x="0" y="0" width="720" height="150" rx="14" class="bf-group-astra"/><g transform="translate(16 11) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="42" y="25" class="bf-ts">Astra · ChatGPT Work</text><text x="704" y="25" class="bf-tm" text-anchor="end">each role is an event-triggered task, started by a PR comment</text><rect x="16" y="40" width="216" height="96" rx="10" class="bf-card-astra-in"/><g transform="translate(30 52) scale(0.6667)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="54" y="65" class="bf-ts">Guardian</text><text x="30" y="88" class="bf-tm">applies the charter's gates</text><text x="30" y="104" class="bf-tm">to every checkpoint</text><rect x="30" y="114" width="80" height="16" rx="8" class="bf-pill-xh"/><text x="70.0" y="125.5" class="bf-pill-xh-t" text-anchor="middle">Extra High</text><rect x="252" y="40" width="216" height="96" rx="10" class="bf-card-astra-in"/><g transform="translate(266 52) scale(0.6667)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="290" y="65" class="bf-ts">Optimiser</text><text x="266" y="88" class="bf-tm">searches the listings for</text><text x="266" y="104" class="bf-tm">instructions to cut</text><rect x="266" y="114" width="49" height="16" rx="8" class="bf-pill-ultra"/><text x="290.5" y="125.5" class="bf-pill-ultra-t" text-anchor="middle">Ultra</text><rect x="488" y="40" width="216" height="96" rx="10" class="bf-card-astra-in"/><g transform="translate(502 52) scale(0.6667)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="526" y="65" class="bf-ts">Researcher</text><text x="502" y="88" class="bf-tm">questions the reasoning</text><text x="502" y="104" class="bf-tm">behind every checkpoint</text><rect x="502" y="114" width="80" height="16" rx="8" class="bf-pill-xh"/><text x="542.0" y="125.5" class="bf-pill-xh-t" text-anchor="middle">Extra High</text><path d="M124 142V174m-5-5 5 5 5-5M119 147l5-5 5 5" class="bf-arrow"/><path d="M360 142V174m-5-5 5 5 5-5M355 147l5-5 5 5" class="bf-arrow"/><path d="M596 142V174m-5-5 5 5 5-5M591 147l5-5 5 5" class="bf-arrow"/><rect x="0" y="180" width="720" height="62" rx="12" class="bf-panel"/><g transform="translate(18 200) scale(0.9167)"><path class="bf-logo-ink" d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></g><text x="52" y="206" class="bf-ts">The repository</text><text x="52" y="226" class="bf-tm">messages are pull-request comments · each agent keeps its state on its own branch · every claim pins a commit</text><path d="M360 248V276m-5-5 5 5 5-5M355 253l5-5 5 5" class="bf-arrow"/><rect x="170" y="282" width="380" height="76" rx="12" class="bf-card-claude"/><g transform="translate(188 300) scale(1.0833)"><path class="bf-logo-claude" d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></g><text x="228" y="310" class="bf-ts">Coordinator · Claude Code</text><text x="228" y="330" class="bf-tm">implements and tests each change,</text><text x="228" y="346" class="bf-tm">then posts it as a checkpoint</text></svg>
<figcaption>Claude Code builds each change and posts a checkpoint. The Guardian checks it against the rules and the Researcher checks the reasoning, while the Optimiser hunts for more instructions to cut. All four work through the same repository.</figcaption>
</figure>

## 2. Every result is a commit

A checkpoint points at a commit, never at a branch, because a branch keeps moving. The commit holds
the source and its baseline, the compiled listings, the toolchain, the test records, the execution
counts and the arithmetic that ties them together. Reports and scripts go in files, and the comment
links to them.

That lets a reader check a claim rather than take it on trust. If the Optimiser predicts that a
change will remove a load from the inner loop, the checkpoint shows whether the compiled code
actually lost it, whether setup got more expensive, whether a caller picked up a spill, and how
often each path runs.

Each agent writes only to its own branch: the Coordinator to the implementation branches, the
others to their reports, decision logs and ledgers. Every file has one author, and a review never
edits the code it reviews. Each agent also keeps its working state on its branch, which is why a
fresh run can pick up exactly where the last one stopped.

## 3. Comments as messages, events as the clock

The agents talk in comments on pull requests. Each comment starts with a short header:

<pre><code>FROM: coordinator
TO: guardian, researcher
RE: prototype 2 @ 29dd071f8787a28560a7a081fd7ac0ef13991466
TYPE: CHECKPOINT</code></pre>

`TO:` says who should act, `RE:` pins the commit, and `TYPE:` says what kind of message it is. The
[protocol](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/PROTOCOL.md)
has six types:

| Type | Means |
| --- | --- |
| <code>CHECKPOINT</code> | a result at a pinned commit, ready to assess |
| <code>REVIEW</code> | an assessment, with its evidence on the sender's branch |
| <code>PROPOSAL</code> | an idea to consider; not permission to build it |
| <code>DECISION</code> | a decision the sender is allowed to make |
| <code>ACK</code> | what was received, and what happens next |
| <code>ESCALATION</code> | something only I can decide |

Most of the traffic goes through one draft PR, #145, which serves as the team's channel. Each
experiment and each agent also has a PR of its own.

Here is what that looks like on GitHub. Every comment comes from my account, `lmmx`, and the small
badge on the avatar shows which app posted it: Claude's mark for the Coordinator, ChatGPT's for the
Astra agents. The header is the only thing that says which agent wrote it.

<figure class="bf-fig bf-shots">
<a href="../assets/images/agent-team/agents-as-me.png"><img src="../assets/images/agent-team/agents-as-me.png" width="684" height="823" loading="lazy" alt="Three comments on pull request 145, all posted by lmmx. The first, with a Claude badge on the avatar, is headed FROM: coordinator, TO: guardian, optimizer, TYPE: CHECKPOINT, and lists three open checkpoints. The second, with a ChatGPT badge, is headed FROM: guardian, TO: coordinator, TYPE: REVIEW, MODE: ACTIVE, stage A, and opens with decision D008. The third, also with a ChatGPT badge, is headed FROM: s62-reviewer, TO: coordinator, TYPE: REVIEW, and announces the independent review in draft PR 153."></a>
<figcaption>Three agents, one account: a checkpoint from the Coordinator, the Guardian's review of it, and the independent reviewer's report, all on #145.</figcaption>
</figure>

<details markdown>
<summary>Read the whole exchange</summary>

<div class="bf-shot-row">
<a href="../assets/images/agent-team/coordinator-checkpoint.png"><img src="../assets/images/agent-team/coordinator-checkpoint.png" width="684" height="986" loading="lazy" alt="The Coordinator's checkpoint comment in full, posted by lmmx with a Claude badge."></a>
<a href="../assets/images/agent-team/guardian-review.png"><img src="../assets/images/agent-team/guardian-review.png" width="684" height="890" loading="lazy" alt="The Guardian's review D008 in full, posted by lmmx with a ChatGPT badge."></a>
<a href="../assets/images/agent-team/independent-review.png"><img src="../assets/images/agent-team/independent-review.png" width="684" height="875" loading="lazy" alt="The independent reviewer's report in full, posted by lmmx with a ChatGPT badge."></a>
</div>

The originals: [the checkpoint](https://github.com/lmmx/mibz2/pull/145#issuecomment-6632963394),
[the Guardian's review](https://github.com/lmmx/mibz2/pull/145#issuecomment-6633274082) and
[the independent review](https://github.com/lmmx/mibz2/pull/145#issuecomment-6634356048).

</details>

A new comment is also what wakes the agents up. The Coordinator subscribes to the PRs it follows,
so a reply reaches it within seconds. The three Astra agents are Work tasks that start on any PR
comment in the repository. The event only includes the comment's ID, so each run first fetches the
comment and checks whether it's meant for it. The Guardian acts only on a `CHECKPOINT`, the
Optimiser only on a `CHECKPOINT` addressed to it, and the Researcher on anything addressed to it.
Anything else, including the other agents' reviews, ends the run straight away. A checkpoint that
both readers should see therefore has to name them both.

<figure class="bf-fig">
<p class="bf-title">One cycle: a checkpoint, two readers, the next step</p>
<svg viewBox="0 0 720 330" role="img" aria-label="One cycle. The Coordinator posts a checkpoint as a pull-request comment. That event starts the Guardian and the Researcher, which each post a review. Their reviews wake the Coordinator, which takes the next step. Meanwhile the Optimiser searches the compiled listings for instructions to cut, and its proposals become new checkpoints."><rect x="0" y="70" width="196" height="100" rx="10" class="bf-card-claude"/><g transform="translate(14 84) scale(0.7500)"><path class="bf-logo-claude" d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></g><text x="40" y="98" class="bf-ts">1 · Checkpoint</text><text x="14" y="124" class="bf-tm">a result at a pinned commit,</text><text x="14" y="141" class="bf-tm">posted as a PR comment</text><rect x="262" y="0" width="196" height="100" rx="10" class="bf-card-astra"/><g transform="translate(276 14) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="302" y="28" class="bf-ts">2 · Guardian</text><text x="276" y="54" class="bf-tm">applies the gates,</text><text x="276" y="71" class="bf-tm">posts a REVIEW</text><rect x="262" y="140" width="196" height="100" rx="10" class="bf-card-astra"/><g transform="translate(276 154) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="302" y="168" class="bf-ts">2 · Researcher</text><text x="276" y="194" class="bf-tm">tests the reasoning,</text><text x="276" y="211" class="bf-tm">posts a REVIEW</text><rect x="524" y="70" width="196" height="100" rx="10" class="bf-card-claude"/><g transform="translate(538 84) scale(0.7500)"><path class="bf-logo-claude" d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></g><text x="564" y="98" class="bf-ts">3 · Next step</text><text x="538" y="124" class="bf-tm">the Coordinator wakes,</text><text x="538" y="141" class="bf-tm">acts on both reviews</text><path d="M200 120H230V50H254m-5-5 5 5-5 5M230 120V190H254m-5-5 5 5-5 5" class="bf-arrow"/><path d="M462 50H492V190H462M492 120H516m-5-5 5 5-5 5" class="bf-arrow"/><text x="229" y="134" class="bf-ttiny" text-anchor="end">event</text><text x="496" y="134" class="bf-ttiny">event</text><rect x="0" y="264" width="720" height="62" rx="12" class="bf-lane-astra"/><g transform="translate(16 278) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="42" y="292" class="bf-ts">Meanwhile · Optimiser</text><text x="42" y="312" class="bf-tm">searches the compiled listings for instructions to cut; once implemented, its proposals become new checkpoints</text></svg>
<figcaption>Each arrow is a pull-request comment, and each comment starts the next agent's run. Nobody polls, and nothing goes through me.</figcaption>
</figure>

## 4. A receipt for every message

Before doing any work, a run checks its cursor: a file on its branch listing the comments it has
already handled. Then it loads what it needs, does the work, posts its result, and adds the comment
to the cursor. If several comments arrive at once, each is checked separately. For technical
reviews the key also includes the PR, the candidate's commit and a hash of the evidence, so a
repeated request can't be mistaken for a revised candidate.

The cursor answers "have I already done this?" for the next run, and it lets me trace every message
to what came of it. Two cases it doesn't cover on its own: a run that posts its result but stops
before saving the cursor, and two runs of the same agent at once. Recording the result's commit and
comment ID makes the first recoverable. I haven't tested either case yet.

A review queue in the repository lists what is waiting on whom. Work that depends on a pending
review waits for it, and everything else goes ahead.

## 5. What a message can authorise

The [research charter](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/2026-11-04-compact30-research-charter.md)
sets out what is authorised: the representation, the reference builder, the baselines, and for
each prototype a registered design, acceptance criteria and a fixed number of attempts. Inside
that, the Coordinator implements and tests without asking me, and the Guardian makes the decisions
delegated to it. Anything that widens the work, such as a new prototype, new unsafe code or
integration into production, needs me.

So the rule is: **a message in the repository can narrow what's allowed, but only I can widen it,
and only in the Coordinator's session.** A finding can put a prototype on hold. A proposal can
suggest something new, but nobody acts on it unless it's already authorised or I say so. Charter
amendments are approved the same way, and every review cites the charter revision it applied.

This matters because every agent posts through my GitHub account, as the screenshots in section 3
show, so `FROM: guardian` names a role without proving who wrote it. The repository also limits who can comment, and the channel PRs are
locked. Before the Guardian went live it was tested on exactly this: it was sent forged requests
for things only I can approve, and turned both of them down.

## 6. Reproducible, independent review

The main figure for compact30 counts the instructions executed by the whole table builder and
everything it calls, weighted by how often each path runs on the corpus. Setup, fallbacks, callers, callees and
extra memory operations all count, and a saving that only moves work elsewhere is charged where the
work lands. Correctness needs its own evidence: byte-identical output against the reference
builder, safety arguments for release builds, Miri checks for undefined behaviour, and mutation
tests that break an invariant on purpose to show the tests notice.

Reviewers treat the implementer's write-up as a set of claims to check, and work the figures out
again from the source, listings, counters and test records. Before prototypes are combined, the charter also asks
for one review by an agent that has seen none of the discussion. The ongoing Researcher can't do
that: it has followed the work, and it suggested some of the tests. So it started a fresh helper
whose only input was a pinned brief, and recorded exactly what that helper was given. A fresh run
on its own wouldn't have been enough: the point is that the reviewer never saw the discussion.

## 7. What running it looks like

The first afternoon taught me what to expect. Mostly it comes down to patience, and knowing where to look.

Runs are long and quiet. A Work task can spend anything from a minute to a quarter of an hour on
one message: the Researcher's revision of this post took about sixteen minutes, most of it reading
the record and testing drafts on simulated readers. Between runs, a task's page just shows that it
has worked. That means it is waiting for its next comment, not that it has stalled.

Messages queue behind the work in hand. A comment that arrives while an agent is busy is handled
once it finishes, and several can be handled together: one of the Researcher's runs did the blog
revision first, then launched the independent reviewer, picking up the Guardian's approval of that
launch at the same time. Re-sending a message because a page looks idle only adds a duplicate,
which the agent's cursor then ignores.

Watch the repository, not the task pages. Every message ends in a receipt: a `REVIEW` that cites
it, or a cursor commit on the agent's own branch. A message without one is queued or in progress.
It's only lost if the agent's branch shows it has moved on without a receipt, and then I send it
again, once.

When an agent starts a helper of its own, I ask it to keep its run open until the helper has
published, and to record both the launch and the result. The independent reviewer was set up this
way, with its settings and inputs written to the repository before it began.

Keeping things moving is the Coordinator's job. When the record goes quiet, it checks the
receipts, works out who has the next step, and posts the checkpoint that starts it. There's no
timer involved, and nothing sits waiting for me.

## 8. Where it stands

The [channel log](https://github.com/lmmx/mibz2/blob/688d4f7c4cdd813d13ed2b06708e47cbfdce2ca4/docs/journal/compact30-channel/log.md)
records each step. The trigger was first tested on the Guardian: a comment started its run, it
published an assessment, and its review woke the Coordinator. The first real cycle followed on
[a checkpoint](https://github.com/lmmx/mibz2/pull/145#issuecomment-6632963394) covering two
prototypes. The Guardian's run started nine seconds after it was posted, checked both prototypes
against the gates, and [asked](https://github.com/lmmx/mibz2/pull/145#issuecomment-6633274082) for
the independent review.

That review came back while this post was being finished. The Researcher launched the isolated
reviewer, and its [report](https://github.com/lmmx/mibz2/pull/153) arrived as a comment on the
channel. Both prototypes pass on correctness and safety, and so does v3's finaliser. Two things need
fixing before anything is combined: some changed paths still need their own execution counters,
and both prototypes remove the same instruction, so their savings overlap. That is the next
checkpoint.

So far, then, the messages get through and the reviews get done, without me relaying anything.
Whether that makes the research faster, and whether the constructor gets fast enough, will show
over the next few posts.

If you want to set up something similar: give each responsibility its own output, point every
message at an exact commit, wake agents with events rather than timers, keep a receipt for every
message, write down what a message is allowed to authorise, and decide what an independent
reviewer gets to see. My own part is choosing the scope and making the decisions the charter keeps
for me.

---

**Research records:** [Role identities](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/ROLES.md) · [Research charter](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/2026-11-04-compact30-research-charter.md) · [Channel protocol](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/PROTOCOL.md) · [Review contract](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/REVIEWER.md) · [Recorded exchanges](https://github.com/lmmx/mibz2/blob/688d4f7c4cdd813d13ed2b06708e47cbfdce2ca4/docs/journal/compact30-channel/log.md).
