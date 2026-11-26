# 7. Gemini's review of the move away from a coordinator, verbatim

*26 November 2026. Relayed by Louis and recorded unedited. Gemini couldn't read #945, so it reasoned from the repository's earlier architecture. Its recommendations overlap ChatGPT's (`06`). It added two that were adopted: every PR names its claim (lineage), and the rollout starts as a canary with at most 3 live holds.*

---

I cannot read the live diff of lmmx/mibz2#945 directly—the PR is not yet indexed in web search and I have no direct GitHub fetch access. Based on mibz2's existing agent-team architecture (role-based state branches like team-state/guardian, Guardian HOLD vetoes, evidence receipts, and the workstation benchmark queue), moving a repo swarm toward a rhizomatic (decentralized, lateral, non-hierarchical) topology introduces a predictable set of failure modes.
Below are the main ways a rhizomatic shift tends to go wrong and the concrete guardrails that contain them.
1. Gate Bypass and Safety Erosion
In a flat, lateral network, peers can start building on each other's unverified branches or treat peer consensus as sufficient to land changes—undermining hard invariants (byte-for-byte libbzip2 parity, CRC checks, Miri, mutation tests).
 * Decouple exploration from promotion: Keep discovery, hypothesis generation, and peer critique rhizomatic, but keep promotion to master strictly hierarchical and mechanical.
 * Enforce non-overridable vetoes: Preserve the Guardian HOLD as an asymmetric veto backed by GitHub branch protection / required status checks, so no lateral agreement among worker agents can merge around a hold.
 * Require machine-verifiable receipts: Any branch that another agent builds on should carry a verifiable receipt (commit SHA, test/Miri pass, evidence run ID) rather than prose claims of correctness.
2. Fan-Out Explosions and Queue Saturation
When any node can spawn or trigger peers laterally, one ambiguous benchmark result or refactor idea can cascade into dozens of speculative branches, saturating both token budgets and the workstation benchmark queue.
 * Hard fan-out and depth caps: Attach a depth and budget counter to every root task (or human trigger). Refuse new spawns once depth exceeds a small cap (e.g., 2–3 hops) or active open PRs exceed a hard ceiling.
 * Queue-driven backpressure: Gate speculative work on the depth of bench/workstation-queue—if the benchmark queue is full, agents can only close, refine, or triage existing work, not open new fronts.
 * Cooldowns and circuit breakers: Add an automated kill switch that pauses autonomous spawns if PR creation rate, CI failure rate, or commit churn crosses a threshold in a rolling window.
3. Duplicate Work and Merge-Conflict Storms
Without a central dispatcher, multiple agents naturally converge on the same hot files (e.g., src/bwt.rs, CLAUDE.md, Justfile) and produce incompatible edits.
 * Lightweight, expiring claims: Use short-TTL lease files or issue/PR labels scoped to a subsystem or hypothesis (e.g., claim:bwt-walk, expires in 2 hours). An agent checks active claims before starting and yields if a peer already holds it.
 * Append-only shared state: Keep cross-agent coordination artifacts (journals, notes, receipts, state logs) strictly append-only—one file per event/entry—rather than having multiple agents edit the same summary file concurrently.
4. "Telephone Game" Context Drift
In a rhizome, Agent C often reads Agent B's summary of Agent A's benchmark run. Subtle caveats (like noisy cloud-VM numbers vs. i9-10900K numbers, or provisional hypotheses) get dropped at each hop, turning tentative guesses into "established facts."
 * Primary-source provenance: Require every technical or performance claim passed between agents to link directly to a pinned commit SHA, file/line, or evidence artifact. Treat unsourced summaries as unverified.
 * Bounded TTL on working memory: Auto-archive or explicitly mark lateral notes as stale once master advances past a threshold or the underlying code changes, forcing re-verification instead of relying on drifted context.
5. Loss of Human Observability
A flat swarm is much harder to debug after the fact when there is no single spine showing why a branch exists.
 * Mandatory lineage trailers: Require every agent-created branch, commit, or PR to include structured metadata (Origin-Trigger, Parent-Task, Hypothesis, Stop-Condition) so you can reconstruct the causal DAG at a glance.
 * Staged rollout (shadow / canary mode): Land the rhizomatic coordination layer behind a cap (e.g., max 2–3 concurrent lateral peers, read-only or draft-PR-only initially) and measure duplicate-work rate, conflict rate, and wasted benchmark runs before opening it up.
If you paste the PR description, the diff, or the updated CLAUDE.md / docs/team/ files from #945, I can audit the exact mechanics and point to specific edge cases in the implementation.
