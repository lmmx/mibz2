# The frontier: how work happens in mibz2

*Adopted 26 November 2026 at Louis's direction: "no more Coordinator … agents would be responsible for making themselves intelligible and this would be an emergent group property". Why: `docs/journal/2026-11-26-rhizome/` and the two cross-readings beside it.*

There is no coordinator and no routing. The repository's files are the shared state. Any agent (Claude, ChatGPT, Gemini or Antigravity, Codex, a human) works the same way, according to what it can do. Each one leaves the state readable for the next.

<!-- brief -->
## The loop: what every agent does (read this, then act)

1. **Read the state.** Run `python3 frontier/tools/frontier.py brief --can <your capabilities>`. If you have no shell, read `frontier/STATUS.md` and the files in `frontier/claims/` and `frontier/blog/`.
   - **Capabilities:** `shell`, `repo-write`, `rust`, `web`, `i9` (Louis's workstation), `cloud-session`, `math`, `long-context`.
2. **Take one thing.** The highest-priority open claim or blog card whose `needs` you meet: `frontier.py claim <id> --by <agent-kind> [--hours N]`. Commit and push that change alone, first, so others see the claim; that is the lock.
   - **An expired claim is open again.**
   - **Never take a claim with `needs_louis`.** It waits for him.
3. **Do it,** under the rules below and the claim's own `stop_test`.
   - **The smooth zone** (`docs/`, `examples/`, `frontier/`, reading, censuses) is free. Explore, and don't wait for anyone.
   - **The striated zone** (`src/`, `bench/`, `.just/`, CI, the workstation queue, releases) requires that you:
     - own the claim's `owned_paths`;
     - freeze the stop test before any data;
     - get exact bytes;
     - add no new `unsafe` without a safety argument and Miri.
4. **Leave a delta** of at most 40 lines: `frontier/deltas/YYYY-MM-DD-<claim>.md`, using the template in `frontier/deltas/README.md`. It says what you found, with evidence (commit SHAs and digests), and what changed.
   - **New work** you discovered becomes new files in `frontier/claims/`.
   - **Measured rooms or states** that changed get updated in `frontier/surface.tsv`.
5. **Release and regenerate:**
   - `frontier.py release <id> --status done|open|blocked|dropped`;
   - `frontier.py status`, which rewrites `STATUS.md`;
   - `frontier.py check`.
6. **Open one PR,** then stop. A fresh session takes the next thing. Don't carry a long session from task to task: a fresh start from the files beats a stale context.

**Making yourself intelligible is your job,** not a coordinator's. Your delta, your PR body and the surface row are how the group knows what happened. Write them for a reader who wasn't there, in plain words, with the evidence.

**Merging:**
- **The smooth zone:** the author may merge its own PR once its checks are green.
- **The striated zone:** a different session verifies and merges. Note that in your delta.
- **Every merge:**
  - `coord.py head-sha --pr N --sha <head>` before and after;
  - `expectedHeadSha`;
  - a squash title and message you write yourself;
  - no AI attribution anywhere (see `CLAUDE.md`).
<!-- /brief -->


## The lexicon: how to ask for things in a word or two

Louis, or anyone, asks for work with these words and nothing more. An agent hearing one does what this table says, filling in the rest from the repository, not by asking. Only something that is genuinely ambiguous, or that RULES D reserves, is worth a question.

| say | it means |
|---|---|
| **take one** / **work the frontier** | Run `brief`, take the top claim you can do, do it, leave a delta, open a PR, stop. |
| **swarm** / **work it down** | Keep taking claims until none fit. Use a fresh session per claim where you can. If you can launch other sessions, launch one per independent claim. |
| **claim: *one line*** | Write a new claim file from that line. Choose kind, zone, needs and priority yourself, write the question, why and stop test from the repository's evidence, and commit it. |
| **probe *X*** | A smooth-zone census of X on real blocks and the hardware, done in a day at most, before any theory (nomad first). Characterisation only; no gate. |
| **build *X*** | Striated work: a registration with a frozen kill test first, then a prototype behind a research feature, exact bytes, then the workstation run. |
| **kill test *X*** / **register *X*** | Write X's decision rule before any data, using the paired-oracle form when X is masked by another part. |
| **rescue *X*** | X was killed or stopped. Treat its verdict as feedback, not overturned: find the framing it missed, and open new claims with the same frozen gate. |
| **redraw** | Re-run the characterisation (`bench-zero-copy-floor`, and the matrix if the parts changed), then update `surface.tsv`. Do it after any merge that moves the shipped 8-worker wall by 5% or more. |
| **where are we** / **surface** | Read `STATUS.md` and the recent deltas back in plain words, in a few minutes' reading at most. Change nothing. |
| **needs me** | List the `needs_louis` items, and what each needs from him. |
| **blog *X*** | Add a blog card for X at stage idea. |
| **move *card*** | Take the card to its next stage, under the board's rules. |
| **review *X*** | A cold review by a different agent from X's author: read X's primary files and evidence, not summaries. Leave a delta with the findings. |
| **ship *X*** | Integrate X: the merge gates, the merge, and a release if `docs/RELEASING.md` says it's worth one and nothing reserves it. |
| **drop *X*** / **block *X*: *reason*** / **bump *X*** | Change X's status or priority, with the reason in a delta. |
| **rupture** | Stop. Write your delta, leave the state readable, and end the session. The next one starts fresh from the files. |

**Figures of speech:**
- **Leave a mark, not a message.** Change the state (a claim, a delta, a row), not a chat.
- **The surface is the map.** Work where the room is.
- **A claim is a lock, not a role.**
- **Smooth to explore, striated to ship.**
- **Every number has a home:** an evidence digest, a results folder, a row.

## The frontier is itself a hybrid system

The decoder was modelled as a hybrid system (`docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md`). The frontier has the same structure, so the team runs as one rather than having one explained to it:

| hybrid system | the decoder | the frontier |
|---|---|---|
| state | blocks, queues, buffers | the files: the claims, the cards, `surface.tsv` |
| flows (progress between events) | workers decoding | sessions working on a claim |
| events (discrete jumps) | admit, dispatch, finish, take | claim, delta, release, merge |
| guards (when an event may fire) | the admission window, the in-order reader | `needs`, leases, frozen stop tests, the merge gates |
| reset maps (what an event changes) | a block changes owner | a delta updates the surface and opens claims |
| ownership | a block's owner | `owned_paths` in the striated zone |

**The structural levers** (`levers.md`) are edited here as files:
- a flow's rate, by who takes what;
- a guard, by a gate or a lease;
- a reset, by what a delta must change;
- the state space itself, by the files' formats.

**The one difference:** the decoder's events are code; the frontier's are commits. That is why every event must leave a mark in the files.

## The files

| path | what it is | who writes it |
|---|---|---|
| `claims/<id>.toml` | a unit of work: question, why it matters, stop test, done-when, capabilities, zone, owned paths, status | anyone, when they find work; the claimer, when they take or release it |
| `blog/<id>.toml` | a blog card on the board, the same schema plus `stage` | anyone |
| `deltas/*.md` | one ≤ 40-line write-up per finished session, the group's memory | the session that did the work |
| `surface.tsv` | the measured frontier: each target, its room at 1 and 8 workers, its state, its evidence, its claims | whoever's result changes a row |
| `STATUS.md` | the generated page: Needs Louis, in progress, open claims, the blog board, recent deltas, the surface | `frontier.py status`, never by hand |
| `config.toml` | the two issues the optional Action mirrors to | rarely changed |
| `tools/frontier.py` | brief, status, check, claim, release; standard library only | anyone, keeping `check` small |

**Nothing depends on GitHub Actions.** `STATUS.md` is regenerated locally by whoever changes the frontier. `.github/workflows/frontier.yml` only mirrors it to issue #943 ("Frontier status") and #944 ("Needs Louis"), and runs `check` on PRs as a courtesy.

## The claim schema

```toml
id = "spelling-census"          # equals the file name
title = "Where emission's 298 ms of spelling goes"
kind = "probe"                   # probe | build | run | review | write | ops | decision | blog
zone = "smooth"                  # smooth | striated
needs = ["shell", "rust"]        # the capabilities required
priority = 1                     # 1 is the highest
status = "open"                  # open | claimed | done | dropped | blocked
claimed_by = ""
claimed_at = ""
expires = ""
owned_paths = []                 # the striated zone only: paths no other live claim may own
needs_louis = ""                 # a reason, if only Louis can unblock it (RULES D: spending, rules, roles, his workstation, releases he reserves)
question = """…"""
why = """… which number it could move …"""
stop_test = """… frozen before data …"""
done_when = """…"""
links = []
```

Blog cards add `stage` (idea → outline → draft → review → revise → ready → published), `post` (its file under `docs/site/blog/`), `drafted_by` and `reviewed_by`.

## The blog board

Any agent moves any card forward one stage, and records the move in a delta:
- **outline:** the claim, the reader, the takeaways first (`write-blog-post`);
- **draft:** the post, with figures made by the `dataviz` method;
- **review:** done by a different agent from the drafter. It is a simulated-reader pass, and every number is checked against its source;
- **revise;**
- **ready:** `publish-page` has passed on the final head;
- **published:** merged.

**The site's rules still hold:**
- no agent names in a post;
- no redirects;
- equations in KaTeX;
- mechanism names, not model names.

## The rules that don't change

They are in `CLAUDE.md` (and `AGENTS.md`, which points here):
- no AI attribution;
- the merge gates and what counts as green;
- CI incidents;
- releases by `docs/RELEASING.md`;
- workstation recipes;
- no redirects;
- naming by mechanism;
- `docs/team/PERFORMANCE-POLICY.md`;
- the exact checks.

**Louis decides only what RULES D keeps:**
- spending and new infrastructure;
- rule and role changes;
- his workstation runs;
- what only he knows.

Mark such a claim `needs_louis`, and it appears on #944 and at the top of `STATUS.md`.

## Keeping the frontier light

From the cross-reading of *A Thousand Plateaus*, consequence C7:
- **Size:** `frontier/tools/frontier.py`'s `check` stays at most 5 checks, and `CLAUDE.md` stays near 120 lines. When you add a rule, delete one.
- **Governance:** governance commits stay under a fifth of the week's commits.
- **The test of this mode:** Louis can follow a day's work from `STATUS.md` and the deltas in a few minutes. If he can't, fix the page, not by bringing back a hub.
