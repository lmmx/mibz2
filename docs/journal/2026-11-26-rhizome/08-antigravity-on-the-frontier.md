# 8. Antigravity's review of the frontier (#945), verbatim

*26 November 2026. Relayed by Louis and recorded unedited, its tool log omitted. It reviewed #945 as merged. Several findings were already fixed in #946: the atomic lock, the needs_louis and overlap guards, protected instructions. Its new findings were adopted in #946 (delta `2026-11-26-frontier-hardening.md`).*

---

PR #945 (`23a23cb`) cleanly removes the Coordinator's routing bottleneck, **but** as currently built it has several concrete mechanical bugs and multi-agent failure modes that will bite as soon as 2+ sessions run in parallel.

Below is what can go wrong—split into **immediate protocol/code breaks** and **emergent swarm side effects**—along with lightweight mitigations that stay inside the frontier's own anti-bloat cap (`C7`: ≤ 5 checks, no rebuilt coordinator).

---

### 1. Immediate mechanical breaks (will hit on the first parallel run)

#### A. The "claim lock" doesn't lock, breaks PR-only agents, and blows the governance-commit cap
* **What's wrong:** [`frontier/README.md`](file:///home/antigravity/lmmx/mibz2/frontier/README.md#L12) and [`frontier.py`](file:///home/antigravity/lmmx/mibz2/frontier/tools/frontier.py#L289) tell agents to *"Commit and push that change alone, first, so others see the claim; that is the lock."*
  * Pushing to a **feature branch** hides the claim from other sessions reading `master`, so every parallel agent takes the same P1 claim.
  * Pushing directly to **`master`** fails for PR-only/sandboxed cloud agents (`HTTP 403`), bypasses `no-ai-attribution.yml`, leaves `STATUS.md` stale, and adds a governance commit for every single claim (quickly blowing past the ≤ 20% governance-commit budget on [`frontier/README.md`](file:///home/antigravity/lmmx/mibz2/frontier/README.md#L174)).
* **Mitigation:** Use **atomic claim branches** instead of `master` commits: `git push origin HEAD:refs/heads/claim/<id>` (works for PR-only agents, fails atomically on GitHub if another agent already pushed `claim/<id>`, zero `master` history clutter). Have `brief` and [`frontier.yml`](file:///home/antigravity/lmmx/mibz2/.github/workflows/frontier.yml) inspect `git ls-remote --heads origin 'claim/*'` (or open PRs) to show active locks.

#### B. Guaranteed `frontier/STATUS.md` merge-conflict storm under `swarm`
* **What's wrong:** Every PR is instructed to run `frontier.py status` ([`frontier/README.md`](file:///home/antigravity/lmmx/mibz2/frontier/README.md#L27)), which rewrites [`frontier/STATUS.md`](file:///home/antigravity/lmmx/mibz2/frontier/STATUS.md) and embeds the current `HEAD` hash ([`frontier.py`](file:///home/antigravity/lmmx/mibz2/frontier/tools/frontier.py#L98)—already stale by 1 commit on `master`, showing `d54c450a3` instead of `23a23cb`). With $N$ concurrent PRs, every merge conflicts with the other $N-1$ PRs on `STATUS.md`, forcing an $O(N^2)$ rebase + CI + `head-sha` cascade (re-introducing Paradigm 3's $\lambda < 1$ stepping-on-toes tax at the coordination layer).
* **Mitigation:** Stop editing `STATUS.md` in work PRs. [`frontier.yml`](file:///home/antigravity/lmmx/mibz2/.github/workflows/frontier.yml#L32-L50) already regenerates `STATUS.md` on push to `master` and mirrors it to `#943`/`#944`; let that post-merge job be the sole writer (or keep `STATUS.md` un-committed / generated locally on demand) and drop `at commit <head>` from any checked-in file.

#### C. Concrete bugs in [`frontier.py`](file:///home/antigravity/lmmx/mibz2/frontier/tools/frontier.py) and [`frontier.yml`](file:///home/antigravity/lmmx/mibz2/.github/workflows/frontier.yml)
* **`claim` skips safety guards:** [`main()`](file:///home/antigravity/lmmx/mibz2/frontier/tools/frontier.py#L270-L290) never checks `needs_louis` (an agent can run `frontier.py claim three-arm-experiment` and succeed) and never checks striated `owned_paths` collisions (agents only find out in PR CI, after doing the work).
* **Path-prefix bug in `owned_paths`:** [`check()`](file:///home/antigravity/lmmx/mibz2/frontier/tools/frontier.py#L180) uses raw `pa.startswith(pb)`—so `"src/open_dense.rs"` falsely collides with `"src/open"`, while `"./src/open.rs"` misses `"src/open.rs"`. Normalize with `pathlib.PurePosixPath` and check path-segment containment (`pa == pb or pa in pb.parents or pb in pa.parents`).
* **Fragile TOML editing:** [`set_fields()`](file:///home/antigravity/lmmx/mibz2/frontier/tools/frontier.py#L239-L247) regex-replaces across the entire file without escaping quotes, so a line inside `question = """..."""` starting with `status =` gets corrupted.
* **`check` skipped on code PRs:** [`frontier.yml`](file:///home/antigravity/lmmx/mibz2/.github/workflows/frontier.yml#L8-L9) only runs on `paths: ["frontier/**"]`. A PR touching `src/` or `CLAUDE.md` without touching `frontier/` bypasses `frontier.py check` completely.
* **Blog review guard unenforced:** [`check()`](file:///home/antigravity/lmmx/mibz2/frontier/tools/frontier.py#L167-L168) never verifies `reviewed_by != drafted_by` for `revise`/`ready`/`published` cards, and `release --status done` can close a card still at `stage = "idea"`.

#### D. Stranded build claims & dead-letter workstation notices (already broken on `master`)
* **Build claims invisible to cloud agents:** Both striated `src/` claims ([`mtf-construction-fold.toml`](file:///home/antigravity/lmmx/mibz2/frontier/claims/mtf-construction-fold.toml#L5) and [`forward-emission.toml`](file:///home/antigravity/lmmx/mibz2/frontier/claims/forward-emission.toml#L5)) require `needs = ["shell", "rust", "i9"]`. No cloud session has `i9`, so `brief --can ...` will **never offer them**, even though their census, registration, and prototype steps run in the cloud and queue a `ready/<recipe>` tag under standing authorisation. Blog cards have a similar mismatch (`needs = ["long-context", "web"]` bundles drafting with `publish-page`'s shell/Playwright pass).
* **Workstation results post into a retired PR:** [`CLAUDE.md`](file:///home/antigravity/lmmx/mibz2/CLAUDE.md#L116) and [`bench/await-submit.sh`](file:///home/antigravity/lmmx/mibz2/bench/await-submit.sh#L40) still post `just submit` evidence and failure notices to `#145`, while [`CLAUDE.md`](file:///home/antigravity/lmmx/mibz2/CLAUDE.md#L200) retires `#145` (and [`channel-routing.yml`](file:///home/antigravity/lmmx/mibz2/.github/workflows/channel-routing.yml) is still active on `#145`). Finished i9 runs will sit unread unless `brief` checks recent commits on the `evidence` branch (or `await-submit.sh` posts to `#943` / a claim).

---

### 2. Emergent swarm & governance side effects (days 2–14)

#### E. Orphaned striated PRs & silent stewardship rot ("empty BwO")
* **What goes wrong:** [`deleuze-guattari-cross-reading.md`](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-26-rhizome/deleuze-guattari-cross-reading.md#L126-L139) explicitly warned against retiring the Coordinator's *verification/integration* duties before the 3-arm test. Trace a striated claim under PR #945: Session A builds a prototype, opens a PR, runs `release --status done`, and stops. Now **nobody sees that PR**:
  * If released `done`, it vanishes from `Open claims` and rots unmerged (joining the 30+ open PRs already in the repo).
  * If released `open`, Session B thinks it's unstarted and duplicates the work.
  * Flaky CI failures, workstation queue readiness, and release checks (`docs/RELEASING.md`) also have no pull mechanism.
* **Mitigation:**
  1. Add a **`status = "review"`** state (with `pr = <N>`, `authored_by = "..."`) and have `brief` list `review` items **above** new open claims (finish-before-start / WIP discipline).
  2. Have `brief` print a compact 3-line live pulse when git/`gh` is available: open striated PRs awaiting merge, uncollected `evidence` branch bundles, and `src/` commits since the last tag.

#### F. Unprotected smooth/striated boundary & rule-mutation loophole
* **What goes wrong:** `master` has no GitHub branch protection or rulesets (`rulesets: []`), and [`frontier/README.md`](file:///home/antigravity/lmmx/mibz2/frontier/README.md#L16) puts all of `frontier/` in the **self-mergeable smooth zone** while leaving `CLAUDE.md` unclassified. Meanwhile, [`open-prs-triage.toml`](file:///home/antigravity/lmmx/mibz2/frontier/claims/open-prs-triage.toml#L4) (merging/closing 30+ PRs, some evidence-pinned) and [`archive-team-strata.toml`](file:///home/antigravity/lmmx/mibz2/frontier/claims/archive-team-strata.toml#L4) (closing 7 issues and moving team docs) are marked `smooth`. An agent can self-merge rule changes to `frontier/README.md`, `frontier.py`, or `CLAUDE.md`—bypassing `RULES D`—or self-merge a "smooth" PR that quietly edits `src/`.
* **Mitigation:**
  1. Explicitly place `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `frontier/README.md`, `frontier/tools/`, and `.github/` in the **striated zone** (with rule changes marked `needs_louis` per `RULES D`).
  2. Reclassify `open-prs-triage` and `archive-team-strata` as `striated`.
  3. Add a minimal GitHub ruleset on `master` requiring `no AI attribution` and `frontier` checks, and have `check` verify that any PR touching striated paths isn't travelling under a `smooth` claim.

#### G. Heterogeneous-agent entry-point drift & AI-attribution footguns
* **What goes wrong:**
  * [`AGENTS.md`](file:///home/antigravity/lmmx/mibz2/AGENTS.md) and [`GEMINI.md`](file:///home/antigravity/lmmx/mibz2/GEMINI.md) are 13-line pointers to [`CLAUDE.md`](file:///home/antigravity/lmmx/mibz2/CLAUDE.md) (which is already 206 lines vs. the 120-line target). Non-Claude agents auto-loading `AGENTS.md`/`GEMINI.md` can easily miss the exact git author/committer rule, `expectedHeadSha`, or macOS runner rule—or interpret step 3 (*"Take one claim… open one PR, stop"*) literally when you only asked a question.
  * [`frontier/deltas/README.md`](file:///home/antigravity/lmmx/mibz2/frontier/deltas/README.md#L7) writes `by: claude-code` (mirrored into `STATUS.md` and `#943`). If an agent copies that into a PR body/squash commit or blog draft, it can trip [`no-ai-attribution.yml`](file:///home/antigravity/lmmx/mibz2/.github/workflows/no-ai-attribution.yml#L34) or violate the blog's no-agent-names rule.
  * Several skills ([`publish-page/SKILL.md`](file:///home/antigravity/lmmx/mibz2/.claude/skills/publish-page/SKILL.md#L43), [`register/SKILL.md`](file:///home/antigravity/lmmx/mibz2/.claude/skills/register/SKILL.md#L130)) still reference the retired `coordinate-blog-post` skill.
* **Mitigation:** Inline the 4 hard invariants directly into `AGENTS.md`/`GEMINI.md` (including *"only run the frontier loop when asked, e.g. `take one`/`swarm`"*), trim `CLAUDE.md` back near 120 lines by offloading the workstation details to a single doc, and note clearly that `by:` / `drafted_by` are repo-internal provenance tags that must never be copied into commit/PR metadata or blog prose.

#### H. Swarm thrashing, claim bloat ("Stock"), and memory loss across ruptures
* **What goes wrong:**
  * **False independence in `swarm`:** `rauzy-output` and `spelling-census` both target `E-spelling`; `mtf-construction-fold` and `forward-emission` both own `src/open.rs`. Launching `swarm` across coupled claims recreates $\lambda < 1$ thrashing.
  * **Multi-session claims vs. single-session rupture:** Claims like `mtf-construction-fold` bundle census → registration → prototype → async i9 run → surface update. A single session cannot finish that cleanly, encouraging either huge stale contexts or premature `done` box-ticking.
  * **Truncated memory + hand-edited `surface.tsv`:** `brief` only shows the last 6 deltas (`STATUS.md` last 12), and `surface.tsv` was made hand-edited (dropping `C3`'s mechanical generation). Under an active swarm, killed hypotheses scroll out of `brief` within hours and fresh sessions re-explore dead ends—while unbounded *"new work becomes new claim files"* turns `frontier/claims/` into an unreadable backlog.
  * **Untested permanence:** [`three-arm-experiment.toml`](file:///home/antigravity/lmmx/mibz2/frontier/claims/three-arm-experiment.toml) is buried at P5, so the flat mode can persist by inertia even if restarted-solo wins.
* **Mitigation:**
  1. **Split multi-session claims into atomic single-session stages** (e.g., `mtf-fold-census` [smooth, `shell,rust`] → `mtf-fold-prototype` [striated, `shell,rust`] → `mtf-fold-run` [striated, `i9`]).
  2. **Add a soft WIP / backlog cap:** max 1 active smooth probe per `surface.tsv` row, max 2–3 active striated claims, and archive/drop stale open claims if open count exceeds ~15 so `STATUS.md` stays readable in minutes.
  3. **Preserve negative memory on the surface:** add a `dead_ends` pointer (or require every killed/stopped claim to add a terse entry in `surface.tsv` with its evidence SHA) so dead ends never scroll off the 6-delta window.
  4. **Set a concrete 1–2 week check-in** (governance-commit ratio ≤ 20%, median unmerged striated PR age, duplicate-work count, and whether you can read a day in 3 minutes) as a cheap smoke test before deciding whether to fund `three-arm-experiment`.

---

### Suggested rollout (smallest high-leverage fixes first)

1. **Before running any parallel `swarm`:**
   * Fix the claim lock (atomic `claim/<id>` branch checked by `brief`, no `master` lock commits) and remove `STATUS.md` from feature PR diffs.
   * Fix [`frontier.py`](file:///home/antigravity/lmmx/mibz2/frontier/tools/frontier.py) (`claim` guarding `needs_louis` + `owned_paths`, `PurePosixPath` containment, safe TOML updates, blog `reviewed_by != drafted_by`).
   * Split `i9` out of the cloud-doable stages of `mtf-construction-fold` and `forward-emission`.
2. **Before the first striated code PR:**
   * Add `status = "review"` (prioritized first in `brief`) + live pulse for open PRs and `evidence` branch bundles.
   * Tighten the smooth/striated boundary (governance files + `open-prs-triage` + `archive-team-strata` striated; `frontier.yml` running on all PRs; minimal GitHub ruleset on `master`).
   * Clarify `AGENTS.md`/`GEMINI.md` trigger conditions and fix the `#145` vs. `evidence` contradiction in `CLAUDE.md`.

Because rule and protocol changes fall under `RULES D`, I haven't edited any files. If you'd like, tell me which of these mitigations you want adopted and I can implement them in one clean follow-up PR.
