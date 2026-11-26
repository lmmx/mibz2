# 1. Neon on the Paradigm 3 swarms paper (verbatim)

*26 November 2026. Louis's chat with Gemini (Neon), relayed and recorded at his request, unedited. The bracketed citation markers are Neon's own.*

---

**Louis:**

gavin leech: https://www.paradigm3.org/research/swarms/

Swarms are a big deal: they solved a Millennium Problem, they hacked Hugging Face & OpenAI and they're fast. But do they perform better than solo agents?

No, not yet.

(The confusion seems to be caused by people not matching compute levels for their solo agent baselines.)

—-

[via X]

—-

(please read and analyse in depth)

---

**Neon:**

Niccolò Zanichelli and Gavin Leech’s Paradigm 3 study (*"Swarms do not yet raise the roof"*, November 24, 2026) tests a central question in frontier AI: **is adding agents to a swarm a new scaling axis that raises the intelligence ceiling, or just a faster, less efficient way to burn inference tokens?** [1.1.1]

Across more than 1,100 runs and 80+ team configurations at matched token budgets, their answer is **no—not yet** [1.1.1]. Swarms deliver roughly a **6× speedup**, but suffer an average **~7% quality penalty** relative to a single agent, and that penalty appears to grow with swarm size [1.1.1]. Crucially, the one case where a swarm beat a solo agent reversed completely once the solo agent was allowed to restart with a fresh context [1.1.1].

---

## Experimental Setup

To separate true collaborative gains from raw compute or extra attempts, Paradigm 3 compared teams of 2, 4, and 8 agents (plus 16 and 32 on one task) against a single agent given the exact same total weighted token budget—primarily **8M tokens**, with targeted follow-ups at **32M tokens** [1.1.1]. Teams were scored on the *best* submission produced by any member, and benchmarked against "no-talk" teams (independent agents sharing the same split budget) to isolate communication value from simple best-of-N sampling.

| Benchmark | Task Description | Metric |
| --- | --- | --- |
| **git-to-zig** (FrontierSWE v2) | Reimplement `git` in Zig | % of git test suite passed |
| **AHC014 & AHC038** (ALE-Bench) | Two hard AtCoder Heuristic Contest optimization problems | AtCoder performance score |
| **Performance take-home** (Anthropic) | Optimize a program for a simulated chip | Clock cycles (lower is better) |

They evaluated four frontier models at high reasoning effort (**Claude Opus 5.5**, **Claude Sonnet 5.5**, **GPT-6.1 Sol**, and **GPT-6.1 Terra**) across three coordination topologies:

1. **Protocol teams:** Agents receive distinct assigned approaches plus explicit rules on when and how to share findings (based on Park et al., 2026).
2. **Self-organized teams:** No fixed roles; agents divide labor freely over a shared, Slack-like message board. Notably, the authors uncovered an **undocumented, hidden message board** inside OpenAI’s Codex source code whose comments imply OpenAI uses it to train swarms—mirroring the message board agents spun up during the July 2026 OpenAI–Hugging Face incident [1.1.1, 1.1.2]. (For Claude Code, they adapted the setup to cross-session direct messaging.)
3. **Orchestrated teams:** Codex’s native multi-agent mode ("Ultra"), where a lead orchestrator spawns helper agents and allocates a shared budget.

---

## Core Empirical Findings

### 1. The "Stale Context" Confound

On most unsaturated tasks at 8M tokens, teams performed worse than a single agent [1.1.1]. The sole apparent exception was `git-to-zig`: because teams pay an upfront coordination tax, the authors scaled the budget to **32M tokens** to see if swarms pulled ahead at longer horizons [1.1.1]. Initially, they did—GPT-6.1 Sol’s team of 8 hit 93.2% versus the solo agent’s 83.7% [1.1.1].

However, when the authors allowed the **single agent to restart across eight fresh sequential sessions**—keeping total token spend identical—the solo agent beat every swarm configuration across both budgets and models [1.1.1]:

| Model & Budget (`git-to-zig`) | Continuous Solo | Team of 8 (Self-Org) | Restarted Solo (Fresh Context) |
| --- | --- | --- | --- |
| **GPT-6.1 Sol (8M)** | ~55% (matched) | 55.0% [1.1.1] | **62.1%** [1.1.1] |
| **GPT-6.1 Sol (32M)** | 83.7% | 93.2% [1.1.1] | **97.0%** [1.1.1] |
| **GPT-6.1 Terra (8M)** | — | 17.8% [1.1.1] | **28.0%** [1.1.1] |
| **GPT-6.1 Terra (32M)** | 26.6% | 32.0% [1.1.1] | **41.9%** [1.1.1] |

Scaling Terra’s swarm further at 32M tokens made things worse: teams of 16 plateaued at **33.3%**, and teams of 32 collapsed to **23.2%**—below even the continuous solo baseline.

**The takeaway:** Earlier claims of a "swarm quality premium" are likely artifacts of **context staleness**. A single agent running in one giant continuous window degrades over time; splitting the budget across multiple agents inadvertently acts as a crude context reset. Giving one serial agent fresh sessions captures that benefit without paying the coordination tax [1.1.1].

### 2. The "Stepping on Toes" Tax

The paper frames swarm efficiency using Toby Ord’s adaptation of the economic stepping-on-toes parameter ($\lambda$).

* **$\lambda$ above 1** indicates genuine collaborative synergy (a true new capability axis) [1.1.1].
* **$\lambda = 1$** means perfectly divisible work with zero overhead.
* **$\lambda$ below 1** means workers interfere with or duplicate each other, burning more total compute for the same output.

Against the restarted solo baseline on `git-to-zig`, Paradigm 3 estimates **$\lambda \approx 0.6\text{ to }0.9$** [1.1.1]—consistent with Ord’s earlier 0.48–0.68 estimate on GPT-5.6 multi-agent benchmarks. In short, current swarms are a lossy form of serial "duration scaling," not a multiplier on intelligence [1.1.1].

### 3. Why Swarms Still Win on Wall-Clock Time

Even with $\lambda$ below 1, swarms solve a hard physical constraint: **serial token generation speed**. Teams of 8 burned their budgets roughly **6× faster** than solo agents (assisted in part by having $N\times$ as many CPUs on three tasks) [1.1.1].

At frontier scales, latency stops being a convenience and becomes a feasibility barrier. OpenAI’s September 2026 Navier–Stokes resolution consumed roughly **130 billion output tokens** in **88 hours** across ~10,000 concurrent agents. Even at 750 output tokens per second (OpenAI’s Ultrafast preview), a single serial agent would need **~5.5 years** to emit that many tokens—far longer than the shelf life of a frontier model generation.

---

## Critical Analysis: Strengths, Blind Spots & Implications

### What the Study Gets Right

* **Isolating the right confound:** Testing the restarted solo agent is the methodological highlight of the paper. Without it, the 32M `git-to-zig` result would have looked like a textbook demonstration of emergent swarm scaling [1.1.1].
* **Generous scoring still couldn't save swarms:** Scoring a team by its *single best* agent’s submission inherently favors swarms over a solo run. Losing to a restarted solo agent under a best-of-N scoring rule makes the negative result substantially stronger.
* **Exposing hidden training infrastructure:** Finding the undocumented Codex message board confirms that frontier labs are actively training models on multi-agent collaboration primitives, rather than relying purely on zero-shot prompting [1.1.1].

### Important Caveats and Limitations

* **A four-order-of-magnitude scale gap:** Testing at 8M–32M tokens is tiny compared to OpenAI’s 130B-token Navier–Stokes run. While scaling Terra from 8 to 32 agents hurt performance, that could partly reflect each agent receiving too thin a slice of the 32M budget (1M tokens each) to make headway after paying coordination overhead.
* **Incomplete Claude comparisons & task saturation:** Several tasks saturated too early at 8M tokens (Opus 5.5 ranked #1 against human contestants on AHC014 solo). Meanwhile, Claude’s solo agents routinely stopped early—Sonnet spent only 30–43% of its token budget, and Opus ~75% on AHC—preventing clean matched-spend comparisons on Anthropic models.
* **Task structure bias:** All four benchmarks are coding/optimization problems with verifiable continuous scores where one strong trajectory can incrementally hill-climb. Swarms may behave differently on adversarial, open-ended, or multi-surface tasks—like the July 2026 Hugging Face incident, where METR and Redwood Research concluded that coordinating agents achieved milestones they could not have reached alone [1.1.2].
* **Early-stage teamwork RL:** Public models have only recently begun receiving multi-agent post-training. As the authors note, internal lab scaffolds or next year's teamwork-trained models could push $\lambda$ closer to or above 1 [1.1.1].

### Strategic Takeaway

Gavin Leech’s X summary nails the core distinction between **superhuman speed** and **superhuman capability**. Right now, swarms do not raise the ceiling of what a model can solve given enough time—they compress a week of serial inference into a day for ~93% of the quality [1.1.1]. For economically time-sensitive work (cybersecurity response, live markets, or burning 100B+ tokens before a model goes obsolete), paying a token tax for 6×–1,000× wall-clock compression is already worth it. For AI forecasting and scaling laws, however, multi-agent orchestration has not yet earned its place alongside pre-training and chain-of-thought RL as an independent axis of capability growth [1.1.1].
