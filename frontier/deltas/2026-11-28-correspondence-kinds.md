claim: correspondence-kinds
by: antigravity
outcome: done
evidence: frontier/README.md (section "The frontier is itself a hybrid system", outside <!-- brief -->); frontier/sensate-backpressure.md; docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md; frontier/claims/correspondence-kinds.toml

**What was asked:** Classify each row of `frontier/README.md`'s hybrid-system correspondence table as an **identity** (same mathematical object), **model** (mapping with stated assumptions), or **analogy** (suggestive; state where it breaks), editing `frontier/README.md` only to add the classification.

**What I found (0 identities, 3 models, 3 analogies):**
1. **state** (`blocks, queues, buffers` ↔ files) — **analogy.** Breaks: shared state also lives outside tracked files (`hold/*`, `frontier-admission`, `wait/*`, GitHub PR/CI state), active session work lives off-repo until pushed, and deltas/claims grow an unbounded archive rather than moving inside a fixed finite-dimensional state space.
2. **flows** (`workers decoding` ↔ sessions working on a claim) — **analogy.** Breaks: shared state is static between commits, with no continuous differential inclusion $\dot{x} \in F(x)$ or monotonically decreasing remaining-work scalar; sessions backtrack, dead-end, or vanish without moving shared state.
3. **events** (`admit, dispatch, finish, take` ↔ `claim, delta, release, merge`) — **model.** Assumptions: models only the discrete-event transition skeleton of atomic CAS pushes (`claim`, `renew`, `release`, review transfers) and exact-head merges, dropping continuous flow and abstracting local edits until committed (a delta is not an atomic event until merged).
4. **guards** (`admission window, in-order reader` ↔ `needs`, leases, frozen stop tests, merge gates) — **model (partial).** Assumptions/break: holds for mechanical preconditions (`max_live_holds`, disjoint `owned_paths`, open flags, Git CAS, `check`, `expectedHeadSha`) assuming all writers use the protocol tools; breaks for self-declared or judgemental gates (`needs`, declared identity, judging a `stop_test`, substantive review), which are not computable predicates $x \in D$.
5. **reset maps** (`block changes owner` ↔ delta updates surface and opens claims) — **analogy.** Breaks: a delta's findings, new claims, and surface edits are open-ended creative outputs rather than a pre-specified map $x^+ \in G(x)$ of the prior state; only mechanical lease updates act as fixed reset maps.
6. **ownership** (`block's owner` ↔ `owned_paths` in striated zone) — **model.** Assumptions/break: cooperative mutual exclusion assuming all writers claim via `frontier.py`, declare accurate `owned_paths`, and `verify` before merging; unlike Rust/decoder ownership, Git does not block uncooperative pushes, leases expire by wall-clock TTL, and the smooth zone has no path ownership.

**Conclusion:** The table is a mixed mapping — a discrete-event mutual-exclusion *model* for the mechanical hold/CAS/merge skeleton and an *analogy* for research work — never an identity. Design arguments must not import continuous-flow theorems (lifetime bounds $U$, control stability) from $\mathcal{H} = (C, F, D, G)$.

**What changed:** Updated only lines 120–140 of `frontier/README.md` (outside `<!-- brief -->`, zero rule/gate/authority changes; ordinary doc edit under `CLAUDE.md` RULES O), added this delta, and marked `correspondence-kinds` `done`.
