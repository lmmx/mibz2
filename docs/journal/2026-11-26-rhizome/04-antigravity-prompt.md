# 4. The prompt Louis gave Antigravity: a cross-reading of the repository against the swarms paper and the rhizome (verbatim)

*26 November 2026. Written by Louis from the chats in `01`–`03`, and run by him in Antigravity. Recorded unedited. Antigravity is to write its report to `docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md`, or beside this folder.*

---

In lmmx/mibz2, fetch origin/master and complete all three parts below in order. Reading, characterisation, and analytical writing only (do not modify decoder/implementation code). Write the final report to docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md (or match the nearest existing docs/journal/ naming convention if there is an active subfolder). Commits as Louis Maddox <louismmx@gmail.com>, with no AI attribution anywhere, trailers included.

### Context & Lenses to Cross-Read Against the Repo

1. **Paradigm 3 / Leech & Zanichelli (24 Nov 2026), "Swarms do not yet raise the roof" (https://www.paradigm3.org/research/swarms/):**
   - Compared 2, 4, 8 (and 16/32) agent teams vs. a single agent at matched weighted token budgets (8M and 32M) across 4 long-horizon tasks and 3 topologies: protocol teams, self-organized teams (using OpenAI's undocumented Codex message board), and orchestrated teams ("Ultra").
   - Result: ~6x wall-clock speedup, but ~7% quality penalty that worsens with swarm size (stepping-on-toes parameter λ ≈ 0.6–0.9).
   - Crucial confound: At 32M tokens on git-to-zig, self-organized teams of 8 initially beat the continuous solo agent (93.2% vs 83.7% for GPT-6.1 Sol; 32.0% vs 26.6% for GPT-6.1 Terra). Once the solo agent was allowed to restart with fresh contexts across sequential sessions (same total tokens), it beat every swarm (97.0% Sol; 41.9% Terra). Conclusion: apparent swarm quality premiums are largely explained by single-agent context staleness, not true collaborative scaling.

2. **Deleuze & Guattari, *A Thousand Plateaus* ("Introduction: Rhizome", Principles 3 & 4 — read literally as a theory of swarms/multiplicities, not metaphor):**
   - **Principle 3 (Multiplicity):** A multiplicity never allows itself to be overcoded and never has a supplementary dimension (n+1) over and above its number of lines. All multiplicities are flat (plane of consistency). "Unity" / overcoding is a power takeover by a signifier/subject operating in an empty supplementary dimension. Genuine multiplicities change in nature as connections increase, unlike "arborescent pseudomultiplicities" (clones of the One divided under a single scalar unit of measure). Speed transforms points into lines (Glenn Gould).
   - **Principle 4 (Asignifying rupture):** A language closed upon itself operates only as a function of impotence. A rhizome survives by shattering at a given spot and restarting on old or new lines (e.g. ants) — though any line of flight risks restratification/overcoding.

### Part 1: Read & Ground in the Repo

Inspect the actual repo state (do not guess or generalize):
- Locate and read the swarm message board implementation, prompts, logs, and any docs/tooling around Claude Code, Gemini Neon, and Antigravity (including `/teamwork`).
- Read the repo sources for the two blog posts:
  - `2026-11-04-autonomous-agent-team` (https://lmmx.github.io/mibz2/blog/2026-11-04-autonomous-agent-team/)
  - `2026-11-22-decompression-is-the-harder-problem` (https://lmmx.github.io/mibz2/blog/2026-11-22-decompression-is-the-harder-problem/)
- Read the algorithm-level shift from "kitchen sink" math techniques (marginal gains) to hybrid dynamical systems and oracle/ideal-parts characterisation:
  - `docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md`
  - `docs/journal/2026-11-24-algorithm-level/external/chatgpt-hybrid-systems-review.md` and the other process/review files in `docs/journal/2026-11-24-algorithm-level/external/`
  - Any recent characterisation prompts/results (e.g., `rauzy-census` or oracle/ideal-parts experiments) that show how the team is currently trying to gauge theoretical speedups with supplied answers rather than computed ones, avoid computational weight knocking down attempts, and progressively move the frontier.

### Part 2: Deep Cross-Reading & Diagnosis

Using concrete evidence (file paths, quotes, actual board entries/prompt patterns, and experiment outcomes from `mibz2`), answer:
1. **Where is `mibz2` currently overcoded?** Audit the actual division of labor between Claude Code (as coordinator / writer of Antigravity Neon prompts), Gemini Neon, Antigravity `/teamwork`, and the swarm message board. Identify every "supplementary dimension" (top-down overcoding, translation tax, premature stratification, or arborescent pseudomultiplicity) present in the repo right now.
2. **Where does the Paradigm 3 "stale context / λ < 1" critique hit `mibz2`, and where does it miss?**
   - Where in our past runs did the message board or multi-agent setup merely compensate for stale context (which a fresh-context restarted solo agent would have beaten) or waste tokens on coordination overhead / marginal "kitchen sink" tweaks?
   - Where does `mibz2` actually differ from Paradigm 3's setup (e.g., heterogeneous models Claude + Neon rather than identical clones; open-ended structural decomposition via hybrid systems rather than a single closed scalar hill-climb)?
3. **Why did the pivot to hybrid-system modelling and oracle/ideal-parts characterisation matter structurally?** Analyze this pivot through both lenses: how does supplying ideal answers to map theoretical speedup levers construct a Deleuzian "plane of consistency" (avoiding computational weight killing exploration) and change what kind of coordination is actually useful as the frontier moves?

### Part 3: Concrete Rhizomatic Redesign for `mibz2`

Propose a specific, actionable redesign grounded in `mibz2`'s existing files and tools (not abstract theory):
1. **Claude Code + Neon + `/teamwork` topology:** Directly evaluate whether Claude Code writing Antigravity/Neon prompts should be kept, narrowed, or replaced. Compare concrete alternatives for free, flat, lateral coordination between heterogeneous agents (Claude Code, Gemini Neon, and Antigravity's `/teamwork` if it has a valid niche).
2. **Message board redesign (from chat/overcoding to stigmergy + rupture):** Propose an exact schema/protocol for the `mibz2` swarm message board that minimizes stepping-on-toes (λ → 1 or above), builds in "asignifying rupture" (short, disposable fresh-context runs that commit distilled deltas rather than dragging stale context), and prevents restratification into a coordinator hierarchy.
3. **Iterative oracle-characterisation loop:** Show how a flat, self-organizing swarm should couple continuous re-characterisation of ideal parts (redrawing the speedup frontier as levers are pulled) with parallel empirical probes so effort concentrates automatically on outsized levers rather than marginal gains.
4. **Falsifiable test:** Specify a concrete, low-budget experiment inside `mibz2` to test the redesigned flat workflow directly against (a) the current Claude-orchestrated Neon workflow and (b) a Paradigm 3-style fresh-context restarted solo baseline at matched token spend.

Commit the completed writeup cleanly to the repo as Louis Maddox <louismmx@gmail.com> with no AI attribution anywhere, trailers included.
