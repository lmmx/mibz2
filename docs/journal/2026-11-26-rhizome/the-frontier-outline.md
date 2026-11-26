# Outline: Dropping the coordinator — file-based stigmergy on the frontier

*Blog card: `frontier/blog/the-frontier.toml` · Stage: `outline` · Prepared 26 November 2026.*
*Primary sources: `frontier/README.md`, `frontier/config.toml`, `frontier/surface.tsv`, `frontier/dead-ends.tsv`, `frontier/claims/stress-test-frontier.toml`, `frontier/claims/three-arm-experiment.toml`, `docs/journal/2026-11-26-rhizome/README.md`, `docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md`, `docs/journal/2026-11-26-rhizome/deleuze-guattari-cross-reading.md`, and the three independent frontier reviews (`docs/journal/2026-11-26-rhizome/` items `06`, `07`, and `08`).*

---

## 1. Target claim, title, dek and takeaways first

### Target claim (one sentence)
On 26 November 2026 we retired our central coordinating agent—whose routing accounted for 46.1% of recent channel traffic and produced every translation-layer error of the day—and replaced central dispatch with an atomic, file-based shared frontier (`frontier/`) whose worth will be decided by a pre-registered three-arm experiment and a daily human readability test.

### Title and dek candidates
The card's working title is *"Mapless: why the team dropped its coordinator"*. Before drafting, note a sharp tension: one of the protocol's five figures of speech is *"The surface is the map."* The new mode is **hubless**, not mapless—it replaces a central narrator with a measured map (`surface.tsv`). The drafter should choose one of the concrete, cold-approachable headlines below (and match the post's URL slug to the chosen title, since published URLs never redirect):

1. **Recommended:** *Dropping the coordinator: file-based stigmergy for an autonomous research team* (proposed slug: `docs/site/blog/2026-11-26-dropping-the-coordinator.md`)
2. **Alternative (closer to card):** *Hubless: why our research team dropped its coordinator*
3. **Alternative (mechanism first):** *Files instead of a hub: replacing an agent coordinator with atomic claims and a measured frontier*

**Dek candidate** (1 sentence, 25 words, 1 comma, 1 figure—within `check_prose.py`'s 16–28 word, ≤ 1 break, ≤ 2 figure band):
> Our coordinating agent wrote 46% of recent team traffic, so we replaced central routing with atomic file locks and fresh sessions on a shared frontier.

### Takeaways first (placed straight after the opening, before Section 1)
- **Central routing was the bottleneck, not verification.** Across five days on the shared channel, the coordinating agent wrote 46.1% of all comments (124 of 269) and introduced four translation-layer defects on a single day, while automated checks (CI, exact-head merge gates, byte-exact tests, frozen registrations) worked without a hub.
- **Fresh contexts beat long-lived swarms.** Paradigm 3's matched-token swarms study showed that apparent 8-agent swarm gains vanished once a single agent restarted with fresh context windows; retiring a permanent hub turns multi-session work into bounded, disposable sessions communicating through lossless files.
- **A measured frontier lets parallel sessions self-route.** Once counterfactual oracle runs mapped every decoder stage's unmasked headroom onto `frontier/surface.tsv`, agents could pull disjoint work through atomic git compare-and-swap holds (`hold/<id>`), leave ≤ 40-line append-only deltas, and let a generated status page (`STATUS.md`, mirrored to issue `#943`) replace chat.
- **One word suffices to ask for work.** A small lexicon (`take one`, `swarm`, `probe`, `build`, `rescue`, `redraw`, `rupture`, and twelve others) works because the repository holds the procedures, capability tags, frozen stop tests and merge gates that fill in the rest.
- **The mode is an experiment, not dogma.** It stays capped at 3 live holds until eight adversarial stress tests pass, and faces two decisive tests: a pre-registered three-arm matched-token experiment against both the old hub and a restarted solo baseline, and whether the maintainer can follow a full day's work from `STATUS.md` in a few minutes.

---

## 2. Intended reader, voice and hard constraints

- **Intended reader:** A systems programmer or researcher who understands git branches, pull requests, CI checks and basic distributed-systems concurrency (locks, compare-and-swap, leases), but has **not** followed `mibz2`'s internal journals, its earlier four-agent setup, or Deleuze and Guattari's philosophy.
- **What the reader does not need to know beforehand:** Any `mibz2` internal journal numbers, issue labels, or decoder internals beyond a two-sentence recap of why the decoder has coupled parallel stages (linked to earlier posts).
- **Hard site & editorial constraints (checked at `review` and `ready`):**
  1. **Zero agent or model names anywhere in the post:** say *"a coordinating agent"*, *"an independent review"*, *"a cold external review"*, *"heterogeneous sessions with different capabilities (shell, Rust, long-context, mathematical review)"*. When citing Paradigm 3's two tested models on `git-to-zig`, refer to them generically as *"two frontier models"* or *"the stronger and weaker models tested"*.
  2. **Mechanism names, not model or persona names:** name techniques by what they do (atomic compare-and-swap hold, paired-oracle kill test, append-only delta ledger, local-scope flag).
  3. **Project voice, British spelling, plain statement:** write as *"we"* and *"the maintainer"*; no internal chat quotes, no process theatre, no mannered prose or filler.
  4. **KaTeX for multi-term equations:** stepping-on-toes scaling $\lambda = \ln(X/b)/\ln N$, backlog growth $\mathbb{E}[Q_n - Q_0] = n(b - 1)$, bottleneck lower bound $L = \max(S, W/k, R)$.
  5. **No redirects, ever:** settle the slug before first publication.
  6. **Independent review & `publish-page`:** `review` stage is done by a different agent from the drafter (`reviewed_by != drafted_by`), checking every number against its primary source; `ready` requires `publish-page` green on the final head.

---

## 3. Argument path (six sections, each answering the last)

### Opening (before Section 1)
- **Claim:** State what happened on 26 November, why the reader should care (multi-agent coding setups routinely recreate management overhead that degrades both quality and human attention), and deliver the five takeaways up front.
- **Method:** Two concise paragraphs connecting back to our [4 November autonomous-team post](2026-11-04-autonomous-agent-team.md) (which automated message passing through GitHub comments) and naming what went wrong three weeks later, followed by the takeaways list and a one-sentence route through sections 1–6.
- **Motivation:** *Supplied* — readers arriving cold need to know why a systems-engineering project is writing about coordination at all, and what concrete evidence triggered the redesign.

### Section 1. Why drop a coordinator that seemed to be working?
- **Salient feature:** Nearly half of the shared channel's traffic was the coordinator talking about its own state, and every defect on 26 November occurred at the translation boundary between the hub and the worker doing the actual build.
- **Claim:** The central coordinating agent bundled two opposite functions—*routing/translating* (which injected staleness, lossy summaries and human copy-paste latency) and *verifying* (CI, head-SHA checks, frozen registrations, byte-exactness). Dropping the first while keeping the second fixes the observed failure modes.
- **Method:** Present three converging lines of evidence, each grounded in primary records:
  1. **External empirical evidence (Paradigm 3 swarms study, 24 November 2026):** Across >1,100 runs and 80+ configurations at matched weighted-token budgets (8M and 32M), swarms bought ~6× wall-clock speedup at an average ~7% quality penalty ($\lambda \approx 0.6\text{–}0.9$ under Toby Ord's stepping-on-toes scaling $\lambda = \ln(X/b)/\ln N$). The single apparent win at 32M tokens on `git-to-zig` (8-agent team beating continuous solo: 93.2% vs. 83.7% on one model; 32.0% vs. 26.6% on another) **reversed completely** once the solo agent restarted across eight fresh contexts at the same budget (97.0% and 41.9%). Scaling further hurt (16 agents plateaued at 33.3%; 32 collapsed to 23.2%). Free-chat mixed teams showed wide variance (−17 to +53 points). Lesson: long-lived context goes stale; fresh resets capture the gain without coordination tax.
  2. **The repository's own record (22–26 November):**
     - Shared channel `#145` accumulated 2,269 comments over 22 days (2,748 across 8 threads). Over 22–26 November (269 comments on `#145`), the coordinating agent wrote **124 (46.1%)**—107 status logs, 13 queue reminders and 4 corrections—while four dormant role inboxes posted **0** comments yet remained wired into active routing scripts.
     - Governance stopped self-pruning: the delivery log froze at 2 rows from 10 November; 38 of the 50 most recent `master` commits touched `docs/` against only 3 touching `src/`; rule files grew by incident patching (the repository's root rules file reached 225 lines; the style-and-mode specification reached 702 lines).
     - **Four translation-layer defects on 26 November alone:** (a) a build prompt specified the wrong base branch (`master`, missing two oracle source files, requiring PR `#916`), (b) two useful pre-port tests were withheld from a brief as a question to the maintainer, (c) a blog brief violated the site's no-agent-names rule (triggering rewrites `#920` → `#927`), and (d) a squash merge (`#926`) carried attribution trailers into `master`. Earlier hub summaries also dropped load-bearing caveats (mistaking the bottleneck lower bound $L = \max(S, W/k, R)$ for an upper completion certificate; attributing 259 ms of real word-spelling work to oracle copying; miscounting 7 lazy workers as 20 active workers) until cold independent reviews read the raw files.
  3. **Structural lens (Deleuze & Guattari, *A Thousand Plateaus*, read strictly for operational consequences):** *Overcoding* (an $n + 1$ hub projecting its own stale frame over lateral work), *asignifying rupture* (hard context resets that break closed loops), and *restratification* (our own 5 November anti-bureaucracy memo hardening into a 702-line rubric). Include the honest caveat from our cross-reading's self-critique: *"never believe that a smooth space will suffice to save us"*—verification strata must stay intact, and stripping away the philosophical vocabulary leaves plain systems engineering that must stand on measurement.
- **Motivation:** *Stated by sources* (`the-frontier.toml`, `rhizome/README.md`, `rhizomatic-swarms-cross-reading.md`).

### Section 2. What makes flat coordination possible? The measured frontier
- **Key step & Reconnection:** Connects the team redesign directly to our parallel [complementary-bottlenecks](2026-11-26-complementary-bottlenecks.md) and zero-copy floor work—showing why a hubless team is viable *now* in a way it was not two weeks ago.
- **Claim:** Flat coordination fails when peers argue in prose over what matters or collide on coupled code. Counterfactual ideal-parts measurement turned subjective priority debates into a shared table (`frontier/surface.tsv`), and zero-copy oracles decoupled coupled bottlenecks onto disjoint files.
- **Method:** Walk through the exact measured numbers that redrew priorities without a hub:
  - At 1 worker, construction holds 42.1% room (`1,306 → 756 ms`); at 8 workers (shipped default, 195 ms), the serial scan alone saves 3.2% and construction alone saves 5.6%, but together they save **36%** (`195 → 125 ms`).
  - The 16-minute zero-copy floor (648 decodes, PR `#921`) collapsed four open debates at once: pure plumbing is **5.2–5.7 ms** (2.7% of 8-worker wall; closed); the serial scan alone is **179.1 ms** at 8 workers (capping worker-only gains at 8% until scanning leaves its serial thread); emission's word spelling takes **298 ms at 1 worker** (22% of wall) and **37 ms at 8**; serial delivery copy-out takes **48 ms flat** (839 MB on one thread).
  - Frozen stop tests pruned dead ends in hours instead of weeks: Rauzy induction order hit a **1.41%** ceiling (< 3% gate, PR `#933`, STOP); network-calculus and static-microarchitecture ports were killed by pre-port tests (PR `#935`); a pooled scan prototype was narrowly killed on K1 (PR `#942`, moving ~175 ms of scan CPU onto workers).
  - Because zero-copy oracles let one session test a scanner change against ideal construction (`PC/C`) while another tests a construction change against an ideal scanner (`SC_cand/S`) on disjoint paths, parallel work stops stepping on its own toes and pushes effective $\lambda$ toward 1.
- **Motivation:** *Recoverable from sources* (`rhizomatic-swarms-cross-reading.md` §2.3, `rhizome/README.md` §5) — supplies the causal link between the decoder's measurement work and the team's new architecture.

### Section 3. What replaced the hub: files, atomic locks and five invariants
- **Claim:** The `frontier/` directory redistributes the coordinator's useful functions—mutual exclusion, memory, prioritisation, safety vetoes and human visibility—into plain version-controlled files and a small standard-library tool (`frontier.py`), with zero dependence on GitHub Actions.
- **Method:** Present the concrete mechanism in two passes—first a toy trace of one claim from discovery to merge, then the components and the hardening pass triggered by three independent reviews (`#945` → `#946`):
  - **Claims (`frontier/claims/*.toml`):** Self-contained work units naming `kind`, `zone` (`smooth` or `striated`), required capabilities (`shell`, `repo-write`, `rust`, `web`, `i9`, `cloud-session`, `math`, `long-context`), `owned_paths`, `mechanisms`, `premises`, `stop_test`, and `needs_louis`.
  - **Atomic holds (`hold/<id>` on origin):** The first draft edited local TOML files—an immediate concurrency bug caught by independent reviews (two sessions pushing separate branches both believed they held the lock). The fix uses git's own compare-and-swap: `git push --force-with-lease=refs/heads/hold/<id>:<expected>` writing a `lease.toml` with a session ID and expiry. Only a printed `acquired` counts; `claim` rejects closed/flagged/`needs_louis` items, overlapping `owned_paths` or `mechanisms`, and claims beyond `max_live_holds = 3`. `verify --session <S>` must pass before merging, tagging, queueing or releasing, and holds stay live until the PR merges (*completion is not integration*). Release writes a released commit rather than deleting the ref, so PR-only sandboxes work identically.
  - **Memory without chat (`deltas/`, `surface.tsv`, `dead-ends.tsv`, `STATUS.md`):**
    - Append-only **≤ 40-line deltas** (`frontier/deltas/`) carry commit SHAs, digests, verdicts and caveats—never edited or deleted (`frontier.py check` enforces append-only history against `origin/master`).
    - **`surface.tsv`** maps live headroom (10 targets); **`dead-ends.tsv`** records killed/stopped directions (10 entries) so negative results never scroll out of a fresh session's 6-delta brief window.
    - **`STATUS.md`** is generated locally by `frontier.py status` and **gitignored** (eliminating the $O(N^2)$ merge-conflict storm that checking it in would cause under parallel PRs), and mirrored optionally to issue `#943` (`Needs Louis` to `#944`).
  - **The two-zone boundary & five conserved invariants:**
    - *Smooth zone* (`docs/`, `examples/`, `frontier/` data, probes): free to explore, author may merge once green.
    - *Striated zone* (`src/`, `bench/`, `.just/`, CI, workstation queue, releases, and `config.toml`'s `protected` governance paths): disjoint `owned_paths`, frozen `stop_test`, exact bytes, Miri + safety argument for `unsafe`, reviewed and merged by a different agent.
    - The five invariants: **exclusive ownership**, **conservation of evidence**, **conservation of authority**, **bounded expenditure** (`max_live_holds = 3`, `max_open_claims = 12`, workstation `flock`), and **traceability** (every PR opens with `Claim: <id>`).
  - **Local immune system (`frontier/flags/`):** Any agent can flag a broken invariant; the flag blocks *only* its named claim while unrelated work continues, and must be resolved by a different agent from the author.
  - **Against monoculture:** Four work kinds (`exploitation`, `exploration`, `characterisation`, `synthesis`), reserving roughly **1 session in 5** for cold `explore` (reading primary data before prior summaries) or recurring `synthesise` (connecting findings across deltas so global integration does not disappear with the hub).
  - **The frontier as a hybrid system:** Brief side-by-side table showing the exact correspondence with our [hybrid-system decoder model](2026-11-26-from-bottlenecks-to-better-algorithms.md) (state = files, flows = sessions, events = claim/delta/release/merge, guards = leases/stop tests/merge gates, reset maps = deltas updating surface and opening claims, ownership = `owned_paths` — differing only in that events are commits rather than code).
  - **The blog board (`frontier/blog/*.toml`):** Same schema plus 7 stages (`idea → outline → draft → review → revise → ready → published`), with `frontier.py check` mechanically enforcing that `review` is done by a different agent from the drafter. Note briefly that this post itself is the first card moved through the board.
- **Motivation:** *Stated by sources* (`the-frontier.toml`, `frontier/README.md`, reviews `06`–`08`).

### Section 4. Asking for work in a word: the lexicon and its figures of speech
- **Claim:** Dropping a central prompt-writer does not mean writing longer human prompts; it means compressing requests to one or two words because the repository's documented protocols, gates and state files supply every remaining detail.
- **Method:**
  - Introduce the **lexicon** from `frontier/README.md` grouped by intent (not a raw unmotivated dump—brief prose lead-in explaining how each group triggers a concrete mechanical loop):
    - *Executing work:* **`take one`** / **`work the frontier`** (run `brief`, claim top matching item, execute, leave delta, open PR, stop), **`swarm`** / **`work it down`** (launch independent sessions per claim until none fit), **`rupture`** (stop cleanly, write delta, leave state readable for a fresh session).
    - *Shaping research:* **`claim: <one line>`**, **`probe X`** (≤ 1-day smooth census on real blocks before theory), **`build X`** (striated: frozen registration, feature-gated prototype, exact bytes, workstation run), **`kill test X`** / **`register X`** (write decision rule before data, paired-oracle form when masked), **`rescue X`** (verdict stands; find missed framing and re-register under same gate), **`redraw`** (re-run zero-copy floor after ≥ 5% move at 8 workers, update `surface.tsv`), **`explore`** / **`synthesise`**.
    - *Stewardship & status:* **`where are we`** / **`surface`**, **`needs me`**, **`review X`** (cold, different agent, primary files), **`ship X`** (merge gates, merge, release if warranted), **`flag X: <invariant>`**, **`admit X`** (`idea → open` within budget), **`drop X`** / **`block X`** / **`bump X`**.
    - *Publishing:* **`blog X`** (add card at `idea`), **`move <card>`** (advance one stage under board rules).
  - Present the **five figures of speech** as compact design rules, unpacking what each prevents:
    1. *Leave a mark, not a message.* (State transitions live in committed files, not ephemeral chat.)
    2. *The surface is the map.* (Work where measured headroom exists.)
    3. *A claim is a lock, not a role.* (Temporary exclusive ownership over a task, not a permanent persona.)
    4. *Smooth to explore, striated to ship.* (Low friction for discovery, hard gates for production.)
    5. *Every number has a home.* (An evidence digest, results folder or table row—never an unsourced prose figure.)
  - **Why one word is enough (and when an agent must still ask):** Contrast a 100-line coordinator brief (which had to repeat base branch, header format, stop test and merge rules—and got them wrong) against saying `probe spelling` or `take one`, where `frontier.py brief`, `surface.tsv`, `dead-ends.tsv`, the root invariant rules file and `RULES.md` supply the exact invariants deterministically. Only genuine ambiguity or `RULES D` (spending, rule/role changes, workstation runs, facts only the maintainer has) triggers `needs_louis`.
- **Motivation:** *Stated by sources* (`the-frontier.toml` explicitly asks the post to teach a reader how to ask for work in a word and why that suffices).

### Section 5. How this should change the work — and what could still go wrong
- **Claim:** The shift targets three concrete operational changes (zero translation-layer defects, fresh-context rigor on every task, and parallel oracle-decoupled workstreams) while facing four structural risks that decentralisation itself creates.
- **Method:**
  - **How the work changes:**
    1. *Zero copy-paste relay:* agents read primary files directly rather than hub-authored briefs; `needs_louis` isolates maintainer decisions onto `#944`.
    2. *Disposable sessions by default:* one claim, one fresh session, one ≤ 40-line delta, then rupture—applying Paradigm 3's winning mechanism across parallel workstreams.
    3. *Parallel attacks on the real frontier:* disjoint claims attack `E_consume` word spelling (298 ms at 1W), `A5+A6a` maximal-run folding, `A6c+E` forward emission, and `D` serial delivery (48 ms flat) without waiting on a single bottlenecked narrator.
  - **What could still go wrong (and the lightweight guardrails):**
    1. *Backlog runaway (reproduction of work):* if each finished claim spawns $b$ new claims on average, expected backlog change after $n$ completions is $\mathbb{E}[Q_n - Q_0] = n(b - 1)$. When $b > 1$, the board becomes a graveyard. Fix: free `idea` creation, capped `max_open_claims = 12` admission budget.
    2. *Intellectual monoculture:* reading the same surface can make diverse sessions converge on the same framing. Fix: cold `explore` and `synthesise` share (~1 in 5 sessions) and allowing competing hypotheses as separate claims.
    3. *Orphaned PRs ("empty" flattening):* if sessions release on PR creation, striated PRs rot unmerged. Fix: explicit `review` status listed *first* in `brief` ("finish before starting") and holds held until merge.
    4. *Axiom creep (rebuilding the coordinator as a linter):* adding a new check after every mishap recreates bureaucracy in code. Fix (`C7`): `frontier.py check` capped at **≤ 5 checks**, the root invariant rules file kept near **120 lines** (one-in, one-out), and measuring governance overhead in tokens and time rather than gameable commit counts.
- **Motivation:** *Recoverable from sources* (`the-frontier.toml`, reviews `06`–`08`, `deleuze-guattari-cross-reading.md` §3).

### Section 6. What will test it (falsification before faith)
- **Payoff:** Closes the loop on `RULES C2` and Paradigm 3—the redesign is not declared a victory by essay; it ships with explicit falsification tests and a fallback path.
- **Claim:** Three staged tests decide whether the frontier scales, stays, or yields to a restarted solo workflow.
- **Method:**
  1. **Canary stress test (`stress-test-frontier`, P1):** Eight concrete adversarial scenarios (simultaneous claim race, mid-claim death and lease takeover, lapsed-holder refusal on `verify`/`renew`/`release`, premise invalidation via flag, independent contradiction of a wrong mathematical argument, post-hoc gate-relaxation catch, duplicate workstation trigger blocked by `owned_paths` + `flock`, self-loosening safety rule blocked by `protected` paths). `max_live_holds` stays at **3** until all eight pass.
  2. **Pre-registered three-arm experiment (`three-arm-experiment`, P5, awaiting maintainer spending approval under `RULES D`):** Matched budget of **6M weighted tokens per arm** (plus a **2M solo calibration anchor** to fit $\lambda$), run on real, decomposable worker-side decoder work behind the zero-copy scanner oracle (byte-exact SHA-256 parity, Miri/clippy green, peak RSS ≤ 1.10×):
     - *Arm 1:* Old orchestrated hub workflow.
     - *Arm 2:* Fresh-context restarted solo baseline (3 × 2M sequential sessions, carrying only git + ≤ 40-line deltas).
     - *Arm 3:* Flat stigmergic frontier (two disjoint builders + systems integration + cold independent audit).
     - *Frozen adoption rule (C2):* Adopt Arm 3 only if it matches or beats Arm 1 while cutting coordination tax by ≥ 50% with ≤ 1 human relay, **and** either beats the stronger restarted solo by ≥ 2 percentage points ($\lambda > 1$) or stays within 2 percentage points with ≥ 2× wall-clock speedup and ≥ 1 verified defect caught by cold review that solo missed. If restarted solo beats Arm 3 by > 3 percentage points with zero uncaught defects, fall back cleanly to restarted solo + cold review using `frontier/` as a solo restart harness.
     - *Nine evaluation dimensions:* validated productivity per million tokens, duplication, intellectual diversity, experimental integrity, integration latency, fragmentation, maintainer overhead, reliability, and governance overhead in tokens and time. (Note honestly that one run per arm is a pilot given Paradigm 3's observed variance.)
  3. **The daily human test:** Can the maintainer understand a day's work from `STATUS.md` (mirrored on `#943`) and the ≤ 40-line deltas in a few minutes? If not, fix the generated status page—never re-introduce a coordinating hub.
- **Motivation:** *Stated by sources* (`the-frontier.toml`, `three-arm-experiment.toml`, `stress-test-frontier.toml`, `frontier/README.md`).

---

## 4. Planned figures and visual analogies

Follow `write-blog-post/SKILL.md`'s visual causal reasoning rule: each figure uses a clear analogy or direct empirical comparison, keeps plain technical labels on every element, and notes in its caption where the analogy stops holding.

1. **Figure 1 (Section 1 / 3): Star hub vs. stigmergic frontier**
   - *Type:* Side-by-side architecture diagram.
   - *Analogy:* Telephone switchboard (one operator re-phrasing every call and holding the whole notebook in memory) vs. a shared harbour chart with radio-locked berths (each vessel reads the live depth chart, locks a berth atomically, logs its sounding, and departs).
   - *Where the analogy stops:* Unlike ships, software sessions have no persistent crew memory between voyages—so the ≤ 40-line delta and `dead-ends.tsv` must carry all continuity.
   - *Plain labels:* Coordinating agent (46.1% of `#145`), 7 role inboxes, pasted briefs vs. `surface.tsv`, `claims/*.toml`, `hold/<id>` CAS locks, `deltas/*.md`, `flags/*.md`, `STATUS.md` (`#943`).
2. **Figure 2 (Section 1): Why fresh contexts beat stale swarms — and where our traffic went**
   - *Type:* Two-panel data chart.
   - *Left panel:* Paradigm 3 `git-to-zig` at 32M tokens (continuous solo vs. 8-agent swarm vs. restarted solo across both tested models: 83.7% / 93.2% / 97.0% and 26.6% / 32.0% / 41.9%, plus the 16- and 32-agent drop to 33.3% and 23.2%).
   - *Right panel:* Five-day `#145` comment breakdown (269 comments: 46.1% coordinating agent, 22.3% automated workstation pings, 31.6% peer reviews and notes; 0 from four dormant inboxes).
3. **Figure 3 (Section 3): One claim's hybrid-system lifecycle (and the blog board)**
   - *Type:* State-transition / swim-lane diagram showing the smooth/striated fork.
   - *Plain labels:* `idea` → (`admit`, within `max_open_claims = 12`) → `open` → (`claim` CAS on `hold/<id>`, within `max_live_holds = 3`) → work flow (smooth vs. striated) → PR (`Claim: <id>`, `status = review`) → `verify --session` + merge gates → `release` + ≤ 40-line delta → **rupture** (session ends). Inset showing the 7-stage blog board (`idea → outline → draft → review [different agent] → revise → ready → published`) with `the-frontier` highlighted.
4. **Figure 4 (Section 6): The three-arm falsification design**
   - *Type:* Budget-allocation bar + decision-region diagram.
   - *Plain labels:* 2M solo calibration anchor + three 6M weighted-token arms (Orchestrated vs. Restarted Solo 3×2M vs. Flat Frontier 2M+2M+1.2M+0.8M) and the frozen C2 thresholds (adopt Arm 3 vs. fall back to restarted solo).

---

## 5. Goal questions, transfer question and held-out questions

### Goal questions (for the simulated-reader pass during `review`)
1. **(`motivation`) Why did the team drop its coordinating agent if verification and CI were working?**
   - *Key points:* [claim] The coordinator conflated central routing/translation with verification; [reason] 46.1% of recent channel traffic was hub bookkeeping and four single-day defects occurred at the prompt/summary boundary; [reason] Paradigm 3 showed long-lived context staleness degrades performance compared to fresh-context restarts; [conclusion] keeping symmetric mechanical checks (CI, head-SHA gates, frozen registrations) while removing central routing eliminates the tax without losing safety.
2. **(`structure`) How does `frontier/` prevent two parallel sessions from colliding or corrupting shared state without a central dispatcher?**
   - *Key points:* [claim] Mutual exclusion and integrity move from a narrator into git-enforced files and invariants; [reason] `frontier.py claim` acquires an atomic remote compare-and-swap lock on `hold/<id>` checking disjoint `owned_paths`/`mechanisms` and `max_live_holds`; [reason] `verify --session` gates consequential actions until the PR merges; [reason] local `flags/` isolate broken invariants without halting unrelated work, and `STATUS.md` is gitignored to avoid merge-conflict storms.
3. **(`obvious-fails`) Why didn't simply telling agents to edit a local claim file and pick the highest-priority task work?**
   - *Key points:* [reason] Local TOML edits pushed to separate branches are non-atomic, allowing duplicate ownership; [reason] always taking the highest-priority known target causes greedy exploitation and intellectual monoculture (missing breakthroughs not yet on the surface); [conclusion] hardening added remote CAS holds, a ~1-in-5 `explore`/`synthesise` share, an admission cap (`max_open_claims = 12`), and a persistent `dead-ends.tsv`.
4. **(`summary`) Why can someone ask for work with a single lexicon word like `probe`, `build`, `rescue` or `rupture`, and when must an agent still ask the maintainer?**
   - *Key points:* [claim] Each lexicon word maps to a documented loop backed by repository state (`brief`, `surface.tsv`, `dead-ends.tsv`, stop tests, merge gates); [reason] the files supply base commits, capabilities, constraints and verification steps deterministically; [conclusion] agents only escalate genuine ambiguity or `RULES D` items (spending, rule/role changes, workstation runs, maintainer-only facts) via `needs_louis`.

### Transfer question (the closing sections must equip the reader to answer this)
5. **(`transfer`) Suppose a new measurement shows that an open claim's core assumption is wrong while two other claims are in flight on unrelated files. What exact sequence of frontier primitives handles it without bringing back a coordinator?**
   - *Expected answer:* Leave a delta with the primary evidence and update `surface.tsv` (and `dead-ends.tsv` if killed); open a `flag` on the invalidated claim naming the invariant, evidence, scope and resolution condition (`claim`/`verify` refuse the flagged claim while the two unrelated claims continue unaffected); a different agent resolves or drops the flagged claim, or opens a `rescue` claim under a new framing with a frozen stop test.

### Held-out questions (do not draft specifically to answer these; test during `review`)
6. **(`non-routine`, held out)** Under what empirical result from the three-arm experiment would the team abandon multi-agent flat swarming, and which parts of `frontier/` would survive even then?
7. **(`justification`, held out)** Why did the zero-copy floor measurement matter to the *team architecture*, not just to the decoder's algorithm priorities?

---

## 6. Link-and-gloss plan (for `draft` and `revise`)

- **Internal blog posts to link at first mention:**
  - [Call on mibz2](2026-11-04-autonomous-agent-team.md) (the original four-agent GitHub-comment team and its five failure modes).
  - [Complementary bottlenecks](2026-11-26-complementary-bottlenecks.md) (the ideal-parts matrix and `S × C` = −36% at 8 workers).
  - [From bottlenecks to better algorithms](2026-11-26-from-bottlenecks-to-better-algorithms.md) (the hybrid-system formulation $\mathcal H = (C, F, D, G)$ and the lower-bound vs. upper-certificate distinction).
- **Outside works to link at first mention (verify URLs before publishing):**
  - Niccolò Zanichelli and Gavin Leech, *"Swarms do not yet raise the roof"*, Paradigm 3 (24 November 2026): `https://www.paradigm3.org/research/swarms/`
  - Toby Ord's stepping-on-toes scaling model (linked via Paradigm 3's reference / Ord's note).
  - Gilles Deleuze and Félix Guattari, *A Thousand Plateaus* (1980 / trans. Massumi 1987) — reference page at first mention.
  - Pierre-Paul Grassé's concept of *stigmergy* — reference link at first mention.
- **Candidate fleuron glosses (3–4 max, skippable asides):**
  1. **Stigmergy:** indirect coordination through marks left in a shared environment (originally coined for termite nest-building), contrasted with direct messaging.
  2. **Compare-and-swap (CAS) in git:** how `git push --force-with-lease=ref:<expected>` turns a git remote into a linearisable lock server without external infrastructure.
  3. **Overcoding and asignifying rupture:** plain two-sentence definitions of the two borrowed concepts so readers need zero philosophy background.
  4. **Paired-oracle kill test:** how testing a real scanner against an idealised constructor (`PC/C`) isolates a masked stage's true gain.

---

## 7. Verified fact and number ledger

Every figure below was checked directly against its primary source on 26 November 2026.

| Claim / figure | Tag | Primary source |
|---|---|---|
| Paradigm 3 study: >1,100 runs, 80+ configs, 8M & 32M weighted tokens, ~6× wall-clock speedup, ~7% quality penalty, $\lambda \approx 0.6\text{–}0.9$ | `[CITED]` | `docs/journal/2026-11-26-rhizome/` item `01`; `docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md` intro |
| `git-to-zig` 32M stale-context reversal: Model A 83.7% solo / 93.2% team-of-8 / **97.0%** restarted solo; Model B 26.6% solo / 32.0% team-of-8 / **41.9%** restarted solo (team-of-16: 33.3%, team-of-32: 23.2%); mixed-chat spread −17 to +53 pts | `[CITED]` | `docs/journal/2026-11-26-rhizome/` item `01`; `docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md` intro |
| PR `#145`: 2,269 comments over 22 days (2,748 across 8 threads); 22–26 Nov sample of 269 comments: coordinating agent wrote **124 (46.1%)** (107 `LOG`, 13 `QUEUED`, 4 `CORRECTION`); workstation pings 60 (22.3%); 4 dormant role inboxes posted 0 | `[MEASURED]` | `docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md` §1.1 |
| Governance drift: `DELIVERED.md` froze at 2 rows (10 Nov); 38/50 recent `master` commits touched `docs/` vs. 3 touching `src/`; root rules file reached 225 lines (205 lines post-handoff); `MODE.md` + `MODE.RATIONALE.md` = 702 lines | `[VERIFIED]` | `docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md` §§1.1, 2.1; line count on repo root rules file |
| Four 26 Nov translation-layer defects: wrong base branch (`#916`), two pre-port tests withheld, blog brief naming an agent (`#920`→`#927`), squash merge trailers (`#926`); plus 6 malformed single-line headers on `#145` | `[VERIFIED]` | `docs/journal/2026-11-26-rhizome/README.md` §2; `docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md` §§1.1, 2.1 |
| Ideal-parts matrix (2,492 decodes, 4 h 15 min): 1W `C` = −42.1% (`1,306→756 ms`); 8W shipped wall 195 ms (194.6 ms), `S` alone −3.2%, `C` alone −5.6%, `S+C` −36% (`195→125 ms`) | `[MEASURED]` | `docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md`; `frontier/surface.tsv` |
| Zero-copy floor (648 decodes, 16 min, PR `#921`): plumbing 5.2–5.7 ms (2.7% of 8W wall); scan alone 179.1 ms at 8W (92% of wall, ≤8% worker-only room); `E_consume` spelling 298 ms at 1W (22% of 1,343 ms) / 37 ms at 8W; delivery copy-out 47.3–49.3 ms (~48 ms flat, 839 MB) | `[MEASURED]` | `docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md`; `frontier/surface.tsv` |
| Recent frozen-gate prunings: Rauzy order STOP at 1.41% ceiling (`#933`), NC (3.2–3.5×) & `uiCA` (−28.8%, wrong sign on L1d spill) KILL (`#935`), pooled scan as built KILL on K1 (capture 0.52 at 8W, 0.48 at 9W, ~175 ms moved to workers, `#942`) | `[MEASURED]` | `frontier/dead-ends.tsv`; `docs/journal/2026-11-24-algorithm-level/pooled-scan/results/README.md` |
| Frontier settings & counts: `max_live_holds = 3` (canary), `max_open_claims = 12`, `status_issue = 943`, `needs_louis_issue = 944`, 10 protected paths, ≤ 40-line deltas, ≤ 5 checks in `frontier.py`, 15 claims, 3 blog cards, 10 `surface.tsv` targets, 10 `dead-ends.tsv` rows | `[VERIFIED]` | `frontier/config.toml`; `frontier/README.md`; `frontier/tools/frontier.py`; `frontier/surface.tsv`; `frontier/dead-ends.tsv` |
| Hardening live CAS race test on origin: two simultaneous claims yielded 1 `acquired` + 1 `reference already exists`; wrong session refused on `verify`/`release` | `[MEASURED]` | `frontier/deltas/2026-11-26-frontier-hardening.md` |
| Three-arm experiment: 6M weighted tokens/arm + 2M calibration anchor (~20M tokens + 3 workstation runs total); 9 scoring dimensions; frozen C2 thresholds (≥50% $\tau_{\text{coord}}$ cut, $H_{\text{relay}} \le 1$, ≥2pp beat or within 2pp + ≥2× speedup + ≥1 defect caught; >3pp solo win triggers fallback) | `[VERIFIED]` | `frontier/claims/three-arm-experiment.toml`; `docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md` §3.4 |
| Canary stress test: 8 adversarial scenarios gating `max_live_holds > 3` | `[VERIFIED]` | `frontier/claims/stress-test-frontier.toml` |
