# Ownership: what in our rules produces caution, and what to change

10 November 2026. The Pathfinder, at Louis's request in this session: a concrete proposal,
grounded in the governing files as they stand on master (`12f0e192`), for making the team own
mibz2. A proposal only. MODE, RULES, STEWARDSHIP and the roles are Louis's to change (RULES D).

## 1. The diagnosis: the structure, not the attitude

**One role may write the decoder. Five may not.** ROLES.md, verbatim:
- Optimiser: "Does not implement, time anything, merge".
- Guardian: "Does not originate research, implement".
- Researcher: "Does not … implement".
- Observer: "Does not decide, gate, rank, review code".
- Pathfinder: "Does not change `src/`, merge, release, time anything on the workstation".

One builder, four analysts and reviewers, one narrator. Review capacity is several times build
capacity, so review expands to fill it. The caution is the arithmetic of that ratio.

**What it produced, 4–10 November:**
- #145 holds 792 comments; the Guardian's decisions reached D095; the journal gained 92 dated items.
- The last performance default shipped in 0.2.6, on 7 November.
- #316, a hardware-gated +6.8%, grew a registration of over 1,600 lines and three inference
  designs (§2.9, §2.10, §2.11), each found unsound, before Louis cut it short.
- #324, −13 to −26% on the primary corpora, was held on +2 to +5% elsewhere; its rescue
  diagnostic went through four reviewed revisions and was then stopped unrun.

**Five mechanisms, each in the text:**

| # | mechanism | where |
|--:|---|---|
| 1 | Only the Coordinator builds; every other role's output is a document for it | ROLES.md, the "Does not" lines |
| 2 | Objection is free and unbounded: a HOLD costs its author nothing, has no expiry, and counterexamples may be constructed, not estimated | ROLES "Guardian", RULES A2, MODE "Counterexample before consent" |
| 3 | A reversible product decision is governed as an irreversible scientific claim: pre-registered, never relaxed | RULES C1, C2; STEWARDSHIP's "beyond noise" merge rule |
| 4 | The register prizes intensity and discovery and never mentions a user, a release or simplicity | MODE.md, every stanza |
| 5 | Louis is the message bus and the only workstation operator | ROLES (four roles are Work tasks he relays), RULES C5, D5 |

## 2. Decisions Louis endorsed that I would reverse

1. **The Pathfinder's brief, "breakthrough in kind, not percent by habit"** (MODE; SPEC.PATHFINDER:
   "A change worth a few percent is not its subject, however sound"). It made me discard what
   ships. The one idea of mine that reached production, the backward table, is a
   reformulation worth 13–26%, found by counting and confirmed by a clock within a day.
   Tunnels, the chunked link and the law of the knee cost more and shipped nothing.
2. **The Optimiser's pricing gate (SPEC.PATHFINDER 3a, 9 November).** It answered my costly
   prototypes with a queue in front of every idea. A clock on a prototype is cheaper than a
   reviewed paper price, and I over-closed three ideas on paper prices.
3. **"Never spend a prototype where arithmetic suffices" with "local timing is indicative
   only" (MODE; RULES C5).** Together they pushed me to bound things I could have timed in two
   minutes. Local and CI timing should decide drop-or-continue. Only the released claim needs
   the workstation.
4. **Two independent adversaries on every decision.** Researcher and Guardian both review each
   registration, and each is told not to agree with the other (ROLES). On performance
   evidence that is a committee.
5. **MODE's register as the house voice.** Compressed and fervent prose is costly to read; the
   Observer exists partly to translate the team back to its owner.

## 3. The changes, file by file

### MODE.md (Louis's; "changed only by him")

- **Add a first stanza, above all roles:**
  > Own it. mibz2 is ours: its speed, its API, its code, its releases. Ship what is correct and
  > faster for the people who use it. Measure early, decide once, merge, watch, revert if wrong.
  > Correctness and memory safety are never traded. Everything else is a judgement, and ours
  > to make.
- **Coordinator stanza, delete:** "No premature integration, no half commitment", "Exhaust
  evidence, not Will", "scientific gates inviolate", "Scientific exactitude inviolate". **Replace with:** "Gain measured?
  Integrate. Safety gates inviolate; performance gates are judgements."
- **Pathfinder stanza, delete:** "Breakthrough in kind, not percent by habit", "Optimiser
  prices independently before substantial impl", "Never spend a prototype where arithmetic
  suffices". **Replace with:** "Largest shippable gain first. Count, then time, the same day."
- **Guardian stanza, replace** "HOLD where material proof absent" **with** "HOLD for
  correctness and memory safety. On performance, advise once."
- **Researcher stanza, replace** "Counterexample before consent" **with** "Estimate the risk, do
  not only construct it: how likely, how large, to whom."

### RULES.md

- **C1, C2 (registration; never relaxed):** scope them to *published findings*: journal and
  blog claims. A merge decision needs no registration; PERFORMANCE-POLICY decides it on one
  bench run.
- **A2 ("ambiguity or conflict counts as a HOLD"):** keep for authority and safety. Add: a HOLD
  on performance evidence states the harm it prevents (who, how likely, how large), its
  cost in days, and the smallest change that lifts it; it lapses after one review round
  unless the Coordinator concurs.
- **B5, B6 (unsafe code; immediate stops):** unchanged. These are the gates worth their cost.
- **C5 (workstation):** add standing recipe *templates* (an A/B of any two commits on the
  release-scaling harness; the gap block; share-nothing). Filling in commits needs no review
  and no new freeze. Only new harness code is reviewed.
- **D8 (roles):** see below; this is the one change that needs him by name.
- **New section O, ownership:** any role may open a PR against anything, `src/` included. The
  author owns it to merge: tests, bench, B5 where it applies. One reviewer, one round, four
  outcomes: merge, one targeted correction, defer, abandon.
- **New sunset rule:** every process rule outside A and B expires 30 days after its last
  renewal. Renewal is one line in `decisions.md` saying what it prevented.

### STEWARDSHIP.md

- **"Changes that touch measured code merge only when … the bench shows an end-to-end
  improvement beyond noise":** replace with "under PERFORMANCE-POLICY.md". As written it
  forbids every refactor, simplification and API change that is merely neutral.
- **Add a maintenance route:** a change that is neutral on the bench and improves the code
  (simpler, safer, better API) merges on green CI and one review.
- **Target-gated route and its conditions:** superseded by PERFORMANCE-POLICY §2's last
  paragraph. Mark it retired, keep it as history.

### ROLES.md and the specs

| role now | becomes | change |
|---|---|---|
| Coordinator | **Lead:** priorities, merges, releases, the backlog; still builds | drops per-message routing duties (below) |
| Pathfinder | **Architect:** owns the performance roadmap and the decoder's shape; writes `src/` PRs | "Does not change `src/`" deleted; step 3a becomes advice |
| Optimiser | **Second implementer:** profile-led changes, taken from the backlog | "Does not implement, time anything" deleted |
| Researcher + Guardian | **One Reviewer:** B5 safety review, a hard gate; one-round evidence review, advisory | HOLD and STOP only under B1, B5, B6 |
| Observer | unchanged in kind; reads the repository on a schedule | the Coordinator's CHECKPOINT dispatch to it deleted |

**On the Pathfinder leading architecture and product performance: yes, with two limits.**
- It leads by owning the ranked backlog and building its top item, not by reviewing others.
- The Lead still decides merges. My own record this week includes three over-closures and a
  withdrawn law; roadmap authority should follow shipped results, not precede them.

### PROTOCOL.md and CLAUDE.md

- **PROTOCOL §2, "one assignment in flight per role" and QUEUED staging:** delete. The pull
  request is the unit of work and of review. #145 carries decisions only.
- **CLAUDE.md:** put the ownership stanza and the scoreboard first; it is the file every
  session loads. Keep the merge checks (`head-sha`, `expectedHeadSha`): cheap, and they
  prevent real mistakes. Drop the routing `audit` at every wake once §2 goes.
- **"Decisions: ask the Guardian, not Louis":** becomes "decide, record why, carry on". Ask the
  Reviewer only on B1, B5, B6.

## 4. Making it durable

An instruction decays; a default does not. Four mechanisms, in order of strength:

1. **Gates as code, not as reviewers.** A CI job reads the bench and prints the
   PERFORMANCE-POLICY verdict: ACCEPT, REJECT or LOUIS. Nobody debates what a script
   decides. Same for B5's checklist.
2. **A scoreboard that every session starts from**, generated, in `docs/team/SCOREBOARD.md`:
   - released MB/s on `wikidata` and `text` at 1 and 10 threads, per release;
   - days since a release carried a primary gain;
   - PRs to `src/` merged this week, and median days from first measurement to merge;
   - open HOLDs with their age.
   Today it would read: 406 and 1,514 MB/s; 3 days; 71% of ten cores busy.
3. **Expiry by default.**
   - A PR that is green, policy-ACCEPT and B5-complete merges within 24 hours or closes with
     a reason.
   - An investigation with no integration decision in 48 hours closes.
   - Process rules sunset (above).
4. **A retro at each release, three lines:** what shipped; what was dropped; which rule cost
   more than it saved. Each retro deletes or simplifies one rule.

**The reversion signal:** comments and decisions per merged `src/` PR. If it climbs, the Lead
cuts process before starting anything else.

## 5. What I would not change

- B1, B5, B6, B7: byte-exact output, the unsafe-code obligations, immediate stops, honest
  coverage.
- B4: recorded verdicts are never rewritten.
- The archived, digested workstation bundle as the evidence for a released figure.
- The merge-head checks.

## 6. First test of the new rules

The universal backward walk, bounded: one PR, fill by rows for column blocks beside the
records fill, CI's bench under PERFORMANCE-POLICY decides, two days. If it fails, #324 ships
as adopted and the idea closes with its number.
