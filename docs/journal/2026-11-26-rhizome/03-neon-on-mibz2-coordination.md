# 3. Neon on making mibz2's coordination rhizomatic (verbatim)

*26 November 2026. Louis's chat with Gemini (Neon), relayed and recorded at his request, unedited.*

*Coordinator's note: Neon says it could not open the repository or the blog. Its account of the setup comes from Louis's description and from general reports, not from the repository's files. Antigravity's cross-reading (`04-antigravity-prompt.md`) checks these points against the repository itself.*

---

**Louis:**

what does this suggest for the use of the swarm message board in repo lmmx/mibz2 alongside use of gemini neon agents, and claude code as coordinator, how can more outsized gains come from free coordination? It is partly described in the blog posts at https://lmmx.github.io/mibz2/blog/2026-11-22-decompression-is-the-harder-problem/ and https://lmmx.github.io/mibz2/blog/2026-11-04-autonomous-agent-team/ as well as the files in the repo itself… there were attempts to seek a ‘kitchen sink’ of mathematical techniques but ultimately this still did not unlock outsized returns only marginal gains, so instead an attempt to model the program as a hybrid system (docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md) which was explained well here: docs/journal/2026-11-24-algorithm-level/external/chatgpt-hybrid-systems-review.md i think was the one, there is a lot of material in https://github.com/lmmx/mibz2/tree/37ffea9e5980f2685053a730f7e7c9014217776f/docs/journal/2026-11-24-algorithm-level/external talking about the process

The point is that we moved to characterising the interaction of ideal parts (with answers supplied rather than computed) to try and gauge potential speed ups so as to focus outsized effort on the biggest levers, to avoid the reality of computational weight knocking down attempts and wasting compute, so as to progressively move the frontier (you would imagine repeatedly characterising in this way)

I do still wonder though if having claude code write antigravity neon prompts is the best approach or what… neon has a “teamwork” slash command which can do something like swarming, but it is not a proper swarm… I am curious how to make the most rhizomatic approach here, it feels we have all the ingredients but it is still heavily overcoded :-(

---

**Neon:**

I wasn't able to access the blog posts or GitHub repository pages you shared directly. However, based on your description of `lmmx/mibz2`, the Paradigm 3 findings, and community discussions on similar webpages about Gemini Neon and Antigravity, there is a very clear structural reason your current setup feels overcoded—and a concrete way to flatten it.

---

## Why Your Current Setup Feels Overcoded

Right now, having **Claude Code write prompts for Gemini Neon** (or invoking Antigravity’s `/teamwork` command) recreates the exact **arborescent hierarchy** Deleuze warns against—and the exact **orchestrated topology** that underperformed in Paradigm 3’s benchmarks:

* **A supplementary dimension of control:** Claude sits above the work as the Subject/Architect, translating the problem into instructions, while Neon executes them. Community reports on similar webpages note that Gemini Neon is a strong, high-throughput executor that tends to do exactly what it is told rather than pushing back or questioning premises the way Claude Opus does, while Antigravity’s `/teamwork` behaves more like a structured multi-agent harness than a flat swarm.
* **Double taxation:** When Claude writes prompts for Neon, you pay a translation/coordination tax *before* Neon even touches the code, and Neon inherits Claude’s blind spots (and context staleness) without being able to shatter them. In Paradigm 3’s terms, that pushes $\lambda$ further below 1.

---

## Why Your "Oracle Characterisation" Pivot Is the Real Breakthrough

Moving away from a "kitchen sink" of mathematical tricks toward modelling the decoder as a **hybrid dynamical system** and **characterising ideal parts (supplying oracle answers rather than computing them)** is the single most rhizomatic move in your workflow—even if the agent wrapper around it is still hierarchical.

* **It builds a plane of consistency:** Instead of throwing agents at one monolithic scalar metric (where they step on each other’s toes trying marginal tweaks), oracle characterisation lays out the entire system’s bottlenecks, theoretical speedups, and phase boundaries on a single exterior grid.
* **It avoids "computational weight" killing exploration:** Agents naturally get trapped locally optimizing expensive heuristics. Measuring the *idealized upper bound* of each subsystem first lets you see where an outsized lever actually exists before burning compute to approximate the oracle.

---

## How to Make `mibz2` Genuinely Rhizomatic (and Get "Free" Coordination)

Combining Paradigm 3’s empirical results (fresh-context restarts beat persistent chatty swarms; self-organized boards beat orchestrators) with Deleuze’s mechanics gives four concrete design changes:

### 1. Replace "Claude Prompting Neon" with Lateral Heterogeneity

Paradigm 3 tested identical clones of the same model. You have a genuine advantage they didn't: **heterogeneous models** (Claude Code + Gemini Neon). Stop putting Claude *above* Neon and put both **on the same flat plane** reading and writing to the same shared state:

* **Neon instances** act as high-speed empirical probes—running parallel oracle ablations, micro-benchmarks, and hybrid-system boundary tests, posting raw timings and failures to the board.
* **Fresh-context Claude sessions** read the board not as a manager issuing tickets, but as a peer theorist/critic—looking for contradictions between oracle ceilings and empirical results, proposing new subsystem decompositions, or challenging the hybrid-system model itself.

### 2. Switch the Message Board from *Conversation* to *Stigmergy*

Deleuze’s own example of a rhizome—**ants**—doesn't coordinate by holding Slack discussions or delegating via an orchestrator. Ants use **stigmergy**: they modify the shared environment, and other ants react to the state of the environment.

* Conversational message boards burn tokens on meta-talk and alignment overhead (the "stepping on toes" tax).
* Make your swarm message board a **terse, structured frontier ledger** rather than a chatroom:
1. **Current Oracle Map:** Subsystem $\rightarrow$ ideal speedup ceiling $\rightarrow$ current best real implementation $\rightarrow$ remaining headroom.
2. **Dead Ends / Ruled-Out Levers:** Failed hypotheses with exact numbers so no future agent wastes tokens re-testing them.
3. **Open Anomalies:** Places where the hybrid-system model predicts a speedup that didn't materialize.


* Agents don't talk *to* each other; they write verifiable deltas *to the board* and pull whichever unclaimed anomaly has the highest theoretical headroom.

### 3. Build "Asignifying Rupture" Into the Loop

Paradigm 3’s strongest finding is that a **restarted solo agent with a fresh context beat every swarm**. Long-running agents accumulate "computational weight" and context staleness—they become closed languages operating *"as a function of impotence."*

* Keep every Neon and Claude session **short, single-purpose, and disposable**.
* Never let an agent persist across multiple failed optimization attempts. Kill the session, commit only the distilled measurement (or ruled-out dead end) to the stigmergic board, and spawn a completely fresh agent from the updated frontier.
* This turns your swarm into a **parallelized version of Paradigm 3’s winning restarted-solo strategy**: you get the wall-clock speedup of parallel Neon runs *and* the anti-staleness benefit of constant context resets, without paying for inter-agent chatter.

### 4. Re-Characterise the Oracle Map Continuously (Not Once)

Because a hybrid system’s bottlenecks shift as soon as you solve the dominant one (Amdahl’s law in motion), treat oracle characterisation itself as an ongoing background line in the rhizome. Whenever any agent closes a meaningful fraction of the gap on one subsystem, automatically trigger a fresh oracle sweep to redraw the frontier—so the rest of the agents immediately re-route onto the new highest-leverage line without a coordinator having to tell them to pivot.

---

If you want a more specific breakdown tailored to the exact hybrid-system formulation, the current `mibz2` message board schema, or what happened in your November 4 and November 22 blog posts, you can paste the relevant excerpts here and I’ll dig into them directly.
