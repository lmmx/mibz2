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

4 November 2026 · 11 min read · [Method](#9-good-practice-in-one-list)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

The research behind [the last post](2026-11-04-compact30-constructor.md) was done by several
AI agents. One implemented and tested each change to compact30's table builder. Others
searched the machine code for instructions to remove, reviewed each result, and checked that
the experiments followed their registered rules. Each agent could work alone for long stretches.
Together, though, they formed a team only when I copied each agent's messages to
the next. I was the team's message bus.

This post describes the team: why the work is split into four roles, and how they now hand it to
each other through the repository without me relaying. Nothing in the setup is exotic: a
shared repository, a small message format, and one event-driven trigger for each agent. Most of
the effort went into the rules that make it safe for the agents to act on each other's messages
without asking first.

## 1. Four roles, defined by what they're responsible for

The team has four members. Each role is defined by its responsibilities, not by the model behind
it. Three of the roles run on the same model, and that doesn't make them interchangeable.

| Role | Runs as | Responsible for | Never does |
| --- | --- | --- | --- |
| **Coordinator** | Claude Code | implementing, compiling and testing each change; archiving the evidence; posting checkpoints | change the scientific rules, or grant itself authority |
| **Optimiser** | Codex | finding instructions to remove, proving the transformations, keeping the canonical instruction ledger | implement code, or certify its own proposals |
| **Guardian** | Codex | applying the research charter's gates to evidence, resolving delegated decisions, escalating reserved ones | originate research, or become an approval gate |
| **Researcher** | Codex | independent, adversarial challenge, including the formal review before changes are combined | agree by default, or duplicate the Optimiser |

The split follows the questions each role asks. The Optimiser asks how to remove more
instructions. The Guardian asks whether the process is scientifically defensible. The Researcher
asks whether the team's reasoning is sound, and whether the Guardian is making the right call.

<figure class="bf-fig">
<p class="bf-title">The team: three Codex agents and Claude Code, joined by one repository</p>
<svg viewBox="0 0 720 360" role="img" aria-label="Four agents connected to one GitHub repository. Above it, three Codex agents running Astra: the Guardian, which applies the charter's gates; the Optimiser, which searches the listings for instructions to cut; and the Researcher, which questions the reasoning. Below it, the Coordinator, Claude Code, which implements and tests each change and posts it as a checkpoint."><rect x="0" y="0" width="720" height="150" rx="14" class="bf-group"/><g transform="translate(16 11) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="42" y="25" class="bf-ts">Codex · Astra</text><text x="704" y="25" class="bf-tm" text-anchor="end">each role is a ChatGPT Work task, started by a PR comment</text><rect x="16" y="40" width="216" height="96" rx="10" class="bf-card"/><g transform="translate(30 52) scale(0.6667)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="54" y="65" class="bf-ts">Guardian</text><text x="30" y="88" class="bf-tm">applies the charter's gates</text><text x="30" y="104" class="bf-tm">to every checkpoint</text><rect x="30" y="114" width="80" height="16" rx="8" class="bf-pill"/><text x="70.0" y="125.5" class="bf-ttiny" text-anchor="middle">Extra High</text><rect x="252" y="40" width="216" height="96" rx="10" class="bf-card"/><g transform="translate(266 52) scale(0.6667)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="290" y="65" class="bf-ts">Optimiser</text><text x="266" y="88" class="bf-tm">searches the listings for</text><text x="266" y="104" class="bf-tm">instructions to cut</text><rect x="266" y="114" width="49" height="16" rx="8" class="bf-pill"/><text x="290.5" y="125.5" class="bf-ttiny" text-anchor="middle">Ultra</text><rect x="488" y="40" width="216" height="96" rx="10" class="bf-card"/><g transform="translate(502 52) scale(0.6667)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="526" y="65" class="bf-ts">Researcher</text><text x="502" y="88" class="bf-tm">questions the reasoning</text><text x="502" y="104" class="bf-tm">behind every checkpoint</text><rect x="502" y="114" width="80" height="16" rx="8" class="bf-pill"/><text x="542.0" y="125.5" class="bf-ttiny" text-anchor="middle">Extra High</text><path d="M124 142V174m-5-5 5 5 5-5M119 147l5-5 5 5" class="bf-arrow"/><path d="M360 142V174m-5-5 5 5 5-5M355 147l5-5 5 5" class="bf-arrow"/><path d="M596 142V174m-5-5 5 5 5-5M591 147l5-5 5 5" class="bf-arrow"/><rect x="0" y="180" width="720" height="62" rx="12" class="bf-panel"/><g transform="translate(18 200) scale(0.9167)"><path class="bf-logo-ink" d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></g><text x="52" y="206" class="bf-ts">The repository</text><text x="52" y="226" class="bf-tm">messages are pull-request comments · each agent keeps its state on its own branch · every claim pins a commit</text><path d="M360 248V276m-5-5 5 5 5-5M355 253l5-5 5 5" class="bf-arrow"/><rect x="170" y="282" width="380" height="76" rx="12" class="bf-card-claude"/><g transform="translate(188 300) scale(1.0833)"><path class="bf-logo-claude" d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></g><text x="228" y="310" class="bf-ts">Coordinator · Claude Code</text><text x="228" y="330" class="bf-tm">implements and tests each change,</text><text x="228" y="346" class="bf-tm">then posts it as a checkpoint</text></svg>
<figcaption>Every agent reads and writes the same repository and wakes on the same kind of event: a pull-request comment addressed to it.</figcaption>
</figure>

### Why this work needs four roles

Instruction-level optimisation is easy to fool yourself about, in a few specific ways, and each
role answers one of them.

- **Savings are estimates until compiled.** The Optimiser's design for a change predicts which
  instructions disappear. The compiler then decides what's actually emitted, and often spills a
  register or recomputes an address somewhere the design didn't touch. So the agent that
  proposes isn't the one that builds and measures. The Coordinator compiles each change and
  counts every instruction the whole function executes over a real corpus.
- **Targets drift unless someone holds them.** Each experiment registers its gates before it
  runs: correctness first, then a minimum net saving. The Guardian applies them as written,
  which stops a near miss from being reinterpreted after the fact.
- **Accounting errors look like results.** Here the Researcher and the Optimiser checked each
  other's arithmetic, and both checked the Coordinator's. Two runs of a profiler had been
  presented as a bound, and they weren't one. A Miri test covered one unchecked access but not
  the other. Both were caught and corrected before anything was combined.

Keeping those questions in separate agents means a mistake made in one has to get past the
others.

These identities are written down in the repository
([`ROLES.md`](https://github.com/lmmx/mibz2/blob/research/compact30-constructor/docs/journal/compact30-channel/ROLES.md)),
so they survive a session restarting or being replaced.

## 2. The repository is the team's memory

None of the agents keeps anything important in its own conversation. Everything that matters is
a file in the repository, on a branch that only one agent writes to:

- **The charter** says what work is authorised, which gates each experiment must pass, and which
  decisions are reserved for me.
- **The operating specifications** say what each role does and how it reports.
- **Each agent's own state** lives on its own branch: a cursor of what it has already processed,
  and its ledger or decision log.
- **The evidence** is in the checkpoint journals, pinned to commits: source, compiled listings,
  counters and raw profiles.

Everything an agent relies on is cited by commit. A decision says which version of the charter
it applied and which evidence it checked, so anyone can replay it later. One writer per branch
means two agents never edit the same file.

This design is what makes the rest possible. An agent that starts with an empty conversation can
read its specification, its cursor and the evidence, and carry on exactly where its last run
stopped.

## 3. Messages are PR comments with a small header

The agents talk to each other in comments on pull requests. Each comment begins with a short
header:

```
FROM: coordinator
TO: guardian
RE: prototype 2, hot descriptor pointer arrays @ 29dd071f
TYPE: CHECKPOINT
```

The `TYPE` says what kind of message it is:

| Type | Sent by | Means |
| --- | --- | --- |
| `CHECKPOINT` | Coordinator | a result is ready at a pinned commit; the addressed roles should act on it |
| `REVIEW` | Optimiser, Guardian, Researcher | an assessment, with its evidence published on the sender's branch |
| `PROPOSAL` | Optimiser | a new idea; it isn't authorisation to implement it |
| `ACK` | Coordinator | a message was received and logged, and what happens next |

`TO:` routes the message to particular roles, and `RE:` pins the commit it's about. One draft
pull request serves as the team's channel. Every checkpoint is announced there, and each
experiment and each agent also has a draft PR of its own.

The header is a label, not authentication. All the agents post through my GitHub
account, so a line saying `FROM: guardian` proves nothing about who wrote it. The safeguards in
§6 are built around that fact.

## 4. Who wakes whom

An agent that has finished its turn is idle until something starts it again. The two kinds of
agent here wake differently.

**The Coordinator** subscribes to pull requests. A new comment, a review or a CI result on a
subscribed PR starts a turn in its existing session. When another agent replies, the Coordinator
reads it within seconds.

**The Codex agents** needed a different mechanism. A Codex conversation keeps its objective
between turns, but nothing on GitHub can start a turn in it. Before the change, each Codex agent
waited until I typed a nudge, such as "new checkpoint on #151". Messages from
Codex to the Coordinator flowed on their own. Messages in the other direction depended on
me.


## 5. The doorbell

ChatGPT Work can run a task when a GitHub event happens. Each Codex role now has one such task,
subscribed to new comments on the repository's pull requests. A new comment rings the doorbell:
it starts a short, fresh run of that role.

GitHub can't filter on a comment's text before a run starts. So every run begins with the same
guard:

1. **Fetch the triggering comment by its ID.** The event supplies only the ID.
2. **Check that it's addressed to this role.** For the Optimiser, that means a `TYPE: CHECKPOINT`
   with its role in the `TO:` line. If not, stop at once, without doing anything else.
3. **Check the cursor.** If this comment's ID has already been processed, stop.
4. **Rebuild the state.** Read the role's specification, its cursor and its ledger or decision log
   from its own branch.
5. **Do the work.** Reproduce the figures it relies on, then assess, audit or research as the
   message asks.
6. **Publish.** Commit the result to its own branch, and post one `TYPE: REVIEW` comment that
   links to it.
7. **Save the cursor,** and stop.

<figure class="bf-fig">
<p class="bf-title">One cycle: a checkpoint, two readers, the next step</p>
<svg viewBox="0 0 720 330" role="img" aria-label="One cycle. The Coordinator posts a checkpoint as a pull-request comment. That event starts the Guardian and the Researcher, which each post a review. Their reviews wake the Coordinator, which takes the next step. Meanwhile the Optimiser searches the compiled listings for instructions to cut, and its proposals become new checkpoints."><rect x="0" y="70" width="196" height="100" rx="10" class="bf-card-claude"/><g transform="translate(14 84) scale(0.7500)"><path class="bf-logo-claude" d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></g><text x="40" y="98" class="bf-ts">1 · Checkpoint</text><text x="14" y="124" class="bf-tm">a result at a pinned commit,</text><text x="14" y="141" class="bf-tm">posted as a PR comment</text><rect x="262" y="0" width="196" height="100" rx="10" class="bf-card"/><g transform="translate(276 14) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="302" y="28" class="bf-ts">2 · Guardian</text><text x="276" y="54" class="bf-tm">applies the gates,</text><text x="276" y="71" class="bf-tm">posts a REVIEW</text><rect x="262" y="140" width="196" height="100" rx="10" class="bf-card"/><g transform="translate(276 154) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="302" y="168" class="bf-ts">2 · Researcher</text><text x="276" y="194" class="bf-tm">tests the reasoning,</text><text x="276" y="211" class="bf-tm">posts a REVIEW</text><rect x="524" y="70" width="196" height="100" rx="10" class="bf-card-claude"/><g transform="translate(538 84) scale(0.7500)"><path class="bf-logo-claude" d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></g><text x="564" y="98" class="bf-ts">3 · Next step</text><text x="538" y="124" class="bf-tm">the Coordinator wakes,</text><text x="538" y="141" class="bf-tm">acts on both reviews</text><path d="M200 120H230V50H254m-5-5 5 5-5 5M230 120V190H254m-5-5 5 5-5 5" class="bf-arrow"/><path d="M462 50H492V190H462M492 120H516m-5-5 5 5-5 5" class="bf-arrow"/><text x="229" y="134" class="bf-ttiny" text-anchor="end">event</text><text x="496" y="134" class="bf-ttiny">event</text><rect x="0" y="264" width="720" height="62" rx="12" class="bf-lane"/><g transform="translate(16 278) scale(0.7500)"><path class="bf-logo-ink" d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/></g><text x="42" y="292" class="bf-ts">Meanwhile · Optimiser</text><text x="42" y="312" class="bf-tm">searches the compiled listings for instructions to cut; each proposal it posts becomes the next checkpoint</text></svg>
<figcaption>Each arrow is a pull-request comment that starts the next agent's run. No agent polls, and no message passes through me.</figcaption>
</figure>

A run that rings the bell for another role's message costs one comment fetch and stops. Replies
are typed `REVIEW`, never `CHECKPOINT`, so an agent's answer can't start another round of work
by accident.

## 6. Authority lives in the charter, not in the messages

When agents act on each other's messages without asking, it matters what a message is allowed to
make them do. The rule the team follows is simple: **the repository can narrow authority, never
widen it.**

- **Narrowing messages count, whoever wrote them.** A `HOLD`, a `STOP`, a finding, or a request
  to tighten a gate is acted on even if its author can't be confirmed. Obeying a false one only
  costs time.
- **Routine work needs no permission.** Work the charter already authorises goes ahead without
  anyone's go-ahead. The Guardian isn't an approval gate. It steps in when evidence is disputed, a
  safety argument fails or a rule needs interpreting.
- **Widening needs me.** New experiments, unsafe code, timing runs, changes to frozen
  code, merges and new infrastructure all count only when I say so in the
  Coordinator's own session. That session is the one channel whose author is known.
- **Only collaborators can comment.** The repository limits interactions to collaborators, so
  only the team's own agents can post in the channel.

Each agent's specification spells out what it may decide and what it must pass up. The
Guardian, for example, can rule on whether a checkpoint met its gates, or name an independent
reviewer, straight from the repository. It can't approve a new experiment.

## 7. Independence is designed in

An agent that proposed an idea isn't independent when it comes to judging that idea's result.
The team keeps three things separate:

- **The proposer.** The Optimiser flags any audit of its own proposal as not independent.
- **The governor.** The Guardian checks that the rules were followed. It doesn't check its own
  rulings.
- **The independent review before changes are combined.** It goes to a fresh run that's given
  only an index of pinned evidence: no discussion history, no proposals, and no earlier verdicts.
  It must disclose any earlier involvement it had.

The checkpoint journals record the implementer's own claimed results. Reviewers are asked to
treat those as claims to falsify. They re-derive each figure from the source, the compiled
listings, the counters and the raw profiles.

## 8. The details that make it work

A few practical choices turned the idea into something that runs:

- **A guard comes before anything else.** Every run fetches its triggering comment and stops
  unless the comment is addressed to it, before reading anything else.
- **Cursors make runs idempotent.** An event delivered twice, or two events batched into one run,
  don't produce duplicate decisions.
- **One writer per branch.** Each agent owns its branch. The Coordinator owns the implementation
  branches.
- **Pinned evidence everywhere.** Each message names the commit it refers to, and each decision
  names the documents it applied, by blob.
- **A commit identity on every branch.** Every commit is under my own GitHub identity,
  with no AI attribution. Automated runs use the account's GitHub noreply address.
- **A model and an effort level for each role.** The table below has them.
- **No polling.** Nothing runs on a timer. Every run starts because something happened.

| Role | Runs on | Model ⠶ effort |
| --- | --- | --- |
| Coordinator | Claude Code session | Claude |
| Optimiser | ChatGPT Work event-triggered task | Astra ⠶ Ultra |
| Guardian | ChatGPT Work event-triggered task | Astra ⠶ Extra High |
| Researcher | ChatGPT Work event-triggered task | Astra ⠶ Extra High |

## 9. Good practice in one list

1. **Define roles by responsibility,** and write them down where every agent can read them.
2. **Keep all state in the repository,** on branches with a single writer, cited by commit.
3. **Use a tiny, typed message format.** Treat its headers as routing, never as authentication.
4. **Wake agents with events, not timers.** A subscription for one agent, an event-triggered task
   for another: whatever each platform supports.
5. **Make each run small and idempotent:** guard, deduplicate, rebuild state, act, publish, record.
6. **Let messages narrow authority but never widen it.** Keep widening decisions with a person,
   in a channel whose author is known.
7. **Make the governor something other than an approval gate.** Authorised work proceeds without
   waiting for it.
8. **Separate proposing, governing and independent review,** and give the reviewer evidence, not
   history.
9. **Limit who can post,** so the channel carries only the team's own messages.

## 10. What changes

I'm no longer the message bus. A checkpoint posted by the Coordinator reaches the
Guardian and the Optimiser on its own, their reviews reach the Coordinator on their own, and the
next step follows. My part is what it should be: setting the rules, and making the
decisions the rules reserve.

The first cycles of this team are running now, on the compact30 table builder. Once there's a
record to compare, this post will gain a short note on how its throughput compares with the
earlier way of working.
