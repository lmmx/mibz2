# Less overcoded, more rhizomatic: the team's own coordination, 26 November 2026

*A meta note: an attempt to make the way the team works more rhizomatic and less overcoded. It is not about the decoder.*

Louis read Deleuze and Guattari on rhizomes beside Paradigm 3's swarms study (Leech and Zanichelli, "Swarms do not yet raise the roof", 24 November 2026), and asked how mibz2's agents could coordinate more freely. The conversations are recorded verbatim, in order:

| file | what it is |
|---|---|
| `01-neon-on-swarms-paper.md` | Neon's analysis of the swarms study. With compute matched, a fresh-context restarted solo agent beat every swarm. Swarms buy wall-clock speed at a stepping-on-toes cost (λ about 0.6–0.9). |
| `02-neon-on-deleuze.md` | Neon on principles 3 and 4 of the rhizome, read against the study. Orchestration is overcoding; a context reset is asignifying rupture; the study's swarms were "arborescent pseudomultiplicities", clones under one scalar measure. The book passage is cited, not reproduced. |
| `03-neon-on-mibz2-coordination.md` | Neon's proposal for mibz2, made without access to the repository. It proposes four moves: lateral heterogeneity in place of Claude prompting Neon; the board as a stigmergic frontier ledger, not a chat; short disposable sessions; and continuous re-characterisation. |
| `04-antigravity-prompt.md` | Louis's prompt to Antigravity: ground those claims in the repository, diagnose, propose a redesign, and specify a falsifiable test against the current workflow and a restarted-solo baseline at matched spend. |
| `05-prompt-deleuze-account.md` | the follow-up: a sourced account of the whole of *A Thousand Plateaus* (every plateau, from secondary sources, paraphrased), then a decisive cross-reading that commits each concept to a consequence, a diagnosis or no bearing |
| `06-chatgpt-on-the-frontier.md` | ChatGPT's review of the frontier (#945): atomic claims, five invariants, against monoculture, synthesis, the immune system, measuring it, breaking it first |
| `07-gemini-on-the-frontier.md` | Gemini's review, made without access to #945: gate bypass, fan-out, lineage, a canary rollout |
| `08-antigravity-on-the-frontier.md` | Antigravity's review, read against the repository: STATUS.md conflicts, path matching, staged claims, a review state, dead ends, entry points |

**Status:** recorded. Antigravity's cross-reading is running and will land beside this folder. Nothing is implemented until it does. The test it proposes decides what changes, so this doesn't become a redesign by assertion.

## The Coordinator's first reading, for the cross-reading to test

These are claims from inside the arrangement being criticised. Weigh them accordingly.

1. **The stale-context finding applies to this Coordinator directly.** The Coordinator is one long-lived session, compacted many times, and it holds the team's state in its context. That is the study's continuous solo agent, the configuration that lost to fresh restarts. The durable state (`plan.md`, the journal, the results folders, #145) already exists, so a fresh Coordinator session could start from files rather than from memory. That is the cheapest rupture available.
2. **Today's errors happened at the translation layer.** Each came from the Coordinator writing a prompt or a brief for someone else:
   - the zero-copy prompt's wrong base branch;
   - two useful tests held back from a prompt as a question;
   - a blog brief that named an agent;
   - a squash merge that carried another session's trailers.

   The work itself was sound. That is evidence for Neon's "translation tax", in this repository's own record.
3. **Not all of the hierarchy is overcoding.** Two different things sit in the Coordinator:
   - **deciding what others should do** (prompts, assignments, routing): the supplementary dimension Deleuze means;
   - **checking what was done** (head-SHA checks, CI, registrations frozen before data, byte-identity, evidence digests).

   The checks are what let anyone's result be trusted without trusting the agent. They are the plane on which lateral work can be accepted, and the study's restarted solo agent had an equivalent in its test suite. A flatter design should keep the checks and drop the routing.
4. **The board is already half a ledger.** `plan.md` §2 is a frontier table, with a question, the number it could move, a stop test and a state for each thread. `ideal-parts/results/` holds the oracle map, and the journal's negative results are the dead ends. What #145 lacks is that agents claim work from that table themselves. Today they wait to be routed.
5. **The characterisation pivot is the one Neon credits,** and today showed it working without a coordinator's insight. The floor run re-drew the frontier on its own: the scan alone is 179 ms at 8 workers, and the spelling is 298 ms at 1 worker. That moved the priorities, and anyone reading the table would have re-routed the same way.
