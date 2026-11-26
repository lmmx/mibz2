# Coordinator-Era Open PRs Triage Audit (Pre-#945)

*Date:* 2026-11-26  
*Claim:* `open-prs-triage` (`frontier/claims/open-prs-triage.toml`, P2, striated `ops`)  
*Snapshot:* `origin/master` at `68cf540`, `origin/evidence` at `c71c4a56`, 67 open pre-`#945` PRs audited  
*Action taken in this PR:* **Audit and decisions only — zero PRs closed or merged, zero branches rewritten, force-pushed, or deleted.**

## Executive Summary

All **67** open PRs created before `#945` (the 26 November move to `frontier/`) were cross-referenced against remote tags (`git ls-remote --tags origin`), the `origin/evidence` archive (165 commits), `bench/workstation-queue.txt`, `frontier/dead-ends.tsv`, `frontier/claims/`, `docs/journal/`, `docs/site/blog/`, and `.github/workflows/no-ai-attribution.yml`.

| Disposition | Count | PR Numbers |
|---|---:|---|
| `keep as evidence branch` | 3 | #145, #909, #917 |
| `merge after review + AI-trailer check` | 2 | #137, #586 |
| `salvage / rebase onto frontier claim` | 6 | #690, #762, #790, #806, #861, #879 |
| `close with pointer (keep branch)` | 56 | #160, #161, #162, #222, #313, #314, #497, #553, #562, #590, #604, #607, #609, #612, #635, #640, #652, #653, #654, #656, #657, #659, #664, #666, #707, #720, #722, #725, #734, #735, #740, #741, #742, #746, #749, #765, #770, #771, #778, #781, #782, #783, #784, #787, #821, #835, #838, #841, #843, #846, #849, #850, #857, #862, #889, #932 |
| **Total** | **67** | **All 67 pre-`#945` open PRs** |

### Critical Invariants & Safety Hazards

1. **NEVER close `#145` (`research/compact30-channel`).** Per `CLAUDE.md` and `bench/workstation-queue.txt` (`# channel: 145`), `#145` is retired for agent routing but stays open permanently as the live workstation evidence noticeboard where `just submit` posts bundle identities and SHA-256 digests.
2. **NEVER delete, rewrite, or force-push ANY branch (`stop_test`).** Across the 67 open PRs, **27 PRs** have an exact `ready/*` tag on their head SHA, **1 more** (`#909`) carries ancestor `ready/*` tags plus `#921`'s zero-copy oracles (`adcf607a`), and **8 more** (`#607`, `#609`, `#734`, `#741`, `#770`, `#846`, `#861`, `#879`) are directly measured by `origin/evidence` bundles (**36 evidence/tag-pinned PRs total**, plus `#145` live noticeboard and `#497` blog commit links). Closing a PR on GitHub preserves its git branch; never check 'delete branch'.
3. **`#909` and `#917` hold live characterisation infrastructure missing from `master`.** `bench-zero-copy-floor`, `src/scan_oracle.rs`, `src/supplied_codes.rs`, and `#921`'s zero-copy `OnceLock` oracles do not exist on `master`, yet `frontier/README.md` (`redraw`) calls for re-running `bench-zero-copy-floor` after any >= 5% 8-worker wall move. Both branches carry AI-attributed commits (7 and 17 commits) and are evidence-pinned, so they must **neither merge as-is nor be rewritten** — keep them open as frozen evidence branches until clean-room ported onto `master`.
4. **AI-attribution check (`.github/workflows/no-ai-attribution.yml`):** **17 distinct pre-`#945` PRs** fail the workflow's pattern (**7** in commit history: `#313`, `#497`, `#590`, `#604`, `#889`, `#909`, `#917`; **15** in PR title/body: `#313`, `#497`, `#562`, `#586`, `#604`, `#635`, `#640`, `#734`, `#735`, `#835`, `#838`, `#841`, `#843`, `#909`, `#917`). In particular, merge candidate **`#586`** has a clean commit (`Louis Maddox <louismmx@gmail.com>`) but carries a generator footer in its PR body that **must be stripped before re-running CI and merging**.
5. **Stacked-PR closing order:** Close child PRs before their base PRs (`#654`, `#656`, `#659` before `#653`; `#932` closed while its base `#909` stays open), and never delete parent branches.
6. **Live salvage highlights:** `#762` (`perf/rle1-zero-shift`) already implements zero-shift in-place RLE1 (`src/rle.rs`) with green CI and directly feeds `frontier/claims/single-pass-rle1-crc.toml`; `#806` (`docs/evidence-scope`) is a RULES D draft awaiting Louis to fold into `migrate-rules`; `#861` (`perf/scanner-combined`) and `#879` (`perf/scan-batched-bits`) are held scanner wins awaiting pinned/low-noise testing on v0.5.14+; `#690` and `#790` are Louis-requested blog/art PRs needing rebase and Louis's preview.

## 1. The 8 Named PRs in `frontier/claims/open-prs-triage.toml`

| PR | Branch (`head` -> `base`) | Head SHA | Pins (Tags / Evidence / Queue) | Mergeable | AI Check | Decision |
|---:|---|---|---|---|---|---|
| #497 | `claude/sharp-meitner-cx0v78` -> `master` | `ee5b5c62` | blog commit links (`a2ddc1ef`) | `MERGEABLE` | **FAIL:** 70+ commit(s) + PR body footer | `close with pointer (keep branch)` |
| #821 | `bench/supplied-codes` -> `master` | `5e5495c5` | head tag: `ready/bench-supplied-codes`; evidence: `supplied-codes` (e45314e0); queue: `bench-four-arm`; ancestor of #835 | `CONFLICTING` | clean | `close with pointer (keep branch)` |
| #841 | `bench/character` -> `master` | `94cb742d` | head tag: `ready/bench-character`; evidence: `character` (870e61d7); queue: `bench-character`; ancestor of #843 | `CONFLICTING` | **FAIL:** PR body footer | `close with pointer (keep branch)` |
| #843 | `bench/character-stage2` -> `master` | `1dfb11c8` | head tag: `ready/bench-clock-mechanism`; anc tags: `ready/bench-character`@94cb742d, `ready/bench-character-stage2`@1901c3bb, `ready/bench-character-stage2b`@8423a82b; evidence: `character-stage2` (54896f60); `character-stage2b` (a4f0dcef); `clock-mechanism` (a1f1ddf8) | `CONFLICTING` | **FAIL:** PR body footer | `close with pointer (keep branch)` |
| #889 | `bench/ideal-parts` -> `master` | `e9d5e778` | head tag: `ready/bench-ideal-matrix`; anc tags: `ready/bench-ideal-parts`@23e11fef; evidence: `ideal-parts` (04c9ecd9); `ideal-matrix` (7dd7b070); queue: `bench-ideal-matrix` | `MERGEABLE` | **FAIL:** 6 commit(s) | `close with pointer (keep branch)` |
| #909 | `bench/ideal-matrix-next` -> `master` | `6b4b5f23` | anc tags: `ready/bench-ideal-parts`@23e11fef, `ready/bench-ideal-matrix`@e9d5e778; evidence: `zero-copy-floor` (cc8ba137); `pooled-scan` (c71c4a56); queue: `bench-zero-copy-floor`; base of: #932 | `MERGEABLE` | **FAIL:** 7 commit(s) + PR body footer | `keep as evidence branch` |
| #917 | `bench/zero-copy-floor` -> `master` | `1e27e46f` | head tag: `ready/bench-zero-copy-floor`; anc tags: `ready/bench-ideal-parts`@23e11fef, `ready/bench-ideal-matrix`@e9d5e778; evidence: `zero-copy-floor` (cc8ba137); queue: `bench-zero-copy-floor` | `MERGEABLE` | **FAIL:** 17 commit(s) + PR body footer | `keep as evidence branch` |
| #932 | `research/pooled-scan` -> `bench/ideal-matrix-next` | `1f6a578c` | head tag: `ready/bench-pooled-scan`; evidence: `pooled-scan` (c71c4a56); queue: `bench-pooled-scan` | `MERGEABLE` | clean | `close with pointer (keep branch)` |

### #497 — research(monodromy): the stopped exchange's walk splits in two from the anchored point; the exchange read is 0.12 r letters
- **Branch:** `claude/sharp-meitner-cx0v78` -> `master` (head `ee5b5c62a9d6980f91cc857f88e40499c7206cab`, 190 commits, 231 files)
- **State:** `OPEN` / `MERGEABLE` | Created `2026-11-16` | Updated `2026-11-26`
- **Pins:** blog commit links (`a2ddc1ef`)
- **AI attribution check:** **FAIL:** 70+ commit(s) + PR body footer
- **Decision:** **`close with pointer (keep branch)`**
- **Justification:** Long-running coordinator-era research session branch (190 commits, 231 files: MONODROMY records 23-97+, retrospective Coder A codes, channel drafts). Published blog post `docs/site/blog/2026-11-16-mathematical-dictionary.md:587` links directly to commits on this branch (`a2ddc1ef`). Carries 70+ AI-attributed commits (`Co-Authored-By` / `noreply@anthropic.com`) and an AI body footer; never intended to merge into `master`.
- **Ready-to-post note:**
  > Closing as a retired coordinator-era research session PR. Branch `claude/sharp-meitner-cx0v78` is preserved intact so published links (`docs/site/blog/2026-11-16-mathematical-dictionary.md`) and MONODROMY records remain reachable. Active work now lives in `frontier/`.

### #821 — bench(supplied-codes): Open's word codes supplied in output order (diagnostic)
- **Branch:** `bench/supplied-codes` -> `master` (head `5e5495c5ea1bcc33ce5f9f27ba1c958e5333d989`, 1 commits, 14 files)
- **State:** `OPEN` / `CONFLICTING` | Created `2026-11-24` | Updated `2026-11-26`
- **Pins:** head tag: `ready/bench-supplied-codes`; evidence: `supplied-codes` (e45314e0); queue: `bench-four-arm`; ancestor of #835
- **AI attribution check:** clean
- **Decision:** **`close with pointer (keep branch)`**
- **Justification:** C1a diagnostic recipe (`bench-supplied-codes`), head pinned by `ready/bench-supplied-codes` (`5e5495c5`) and ancestor of `#835`. Executed on the i9 (`origin/evidence e45314e0`, `supplied-codes/run-20261124T131518Z`); results landed in `docs/journal/2026-11-24-algorithm-level.md` §30 and were replicated by `#835` (§44) and `#889` (`#899`). Conflicted with `master` and never meant to merge.
- **Ready-to-post note:**
  > Closing completed coordinator-era diagnostic recipe PR. Evidence is archived at `origin/evidence` `e45314e0` (`supplied-codes/run-20261124T131518Z`) and recorded in `docs/journal/2026-11-24-algorithm-level.md` §30. Branch `bench/supplied-codes` and tag `ready/bench-supplied-codes` remain intact.

### #841 — bench-character: stage 1, the decoder's character across 1–20 workers (C1a diagnostic)
- **Branch:** `bench/character` -> `master` (head `94cb742dfb2168c9b8aadd27246f43349e171a6d`, 7 commits, 17 files)
- **State:** `OPEN` / `CONFLICTING` | Created `2026-11-24` | Updated `2026-11-26`
- **Pins:** head tag: `ready/bench-character`; evidence: `character` (870e61d7); queue: `bench-character`; ancestor of #843
- **AI attribution check:** **FAIL:** PR body footer
- **Decision:** **`close with pointer (keep branch)`**
- **Justification:** C1a diagnostic recipe (`bench-character`, stage 1), head pinned by `ready/bench-character` (`94cb742d`) and ancestor of `#843`. Executed on the i9 (`origin/evidence 870e61d7`, `character/run-20261125T001135Z`); results and data landed on `master` under `docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/`. Conflicted with `master`; PR body carries an AI generator footer (0 AI commits).
- **Ready-to-post note:**
  > Closing completed coordinator-era diagnostic recipe PR. Stage-1 evidence is archived at `origin/evidence` `870e61d7` (`character/run-20261125T001135Z`) and recorded under `docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/`. Branch `bench/character` and tag `ready/bench-character` remain intact.

### #843 — bench-character-stage2: interventions on the decoder's network at the anchors stage 1 fixed (C1a diagnostic)
- **Branch:** `bench/character-stage2` -> `master` (head `1dfb11c88b0a456c6efce4b8b1641d1ddbedaa2b`, 24 commits, 29 files)
- **State:** `OPEN` / `CONFLICTING` | Created `2026-11-25` | Updated `2026-11-26`
- **Pins:** head tag: `ready/bench-clock-mechanism`; anc tags: `ready/bench-character`@94cb742d, `ready/bench-character-stage2`@1901c3bb, `ready/bench-character-stage2b`@8423a82b; evidence: `character-stage2` (54896f60); `character-stage2b` (a4f0dcef); `clock-mechanism` (a1f1ddf8)
- **AI attribution check:** **FAIL:** PR body footer
- **Decision:** **`close with pointer (keep branch)`**
- **Justification:** C1a diagnostic recipe covering stage 2a, stage 2b, and clock-mechanism; head pinned by `ready/bench-clock-mechanism` (`1dfb11c8`), ancestors pinned by `ready/bench-character` (`94cb742d`), `ready/bench-character-stage2` (`1901c3bb`), and `ready/bench-character-stage2b` (`8423a82b`). All three runs completed (`origin/evidence` `54896f60`, `a4f0dcef`, `a1f1ddf8`), results landed in journal §§78 and 81 (`math-targets/data/character-stage2/`), and the 100 ms pre-spin standard landed in `#874`. Conflicted; PR body carries an AI footer (0 AI commits).
- **Ready-to-post note:**
  > Closing completed coordinator-era diagnostic recipe PR. Evidence is archived at `origin/evidence` `54896f60` (stage 2a), `a4f0dcef` (stage 2b), and `a1f1ddf8` (clock-mechanism), written up in `docs/journal/2026-11-24-algorithm-level.md` §§78 and 81 (with pre-spin landed in `#874`). Branch `bench/character-stage2` and all four `ready/*` tags remain intact.

### #889 — bench-ideal-parts / bench-ideal-matrix: the ideal-parts matrix, slice A and the full SECMRD (characterisation)
- **Branch:** `bench/ideal-parts` -> `master` (head `e9d5e778251d4cba3d51922d29688318ef23eca6`, 12 commits, 26 files)
- **State:** `OPEN` / `MERGEABLE` | Created `2026-11-25` | Updated `2026-11-26`
- **Pins:** head tag: `ready/bench-ideal-matrix`; anc tags: `ready/bench-ideal-parts`@23e11fef; evidence: `ideal-parts` (04c9ecd9); `ideal-matrix` (7dd7b070); queue: `bench-ideal-matrix`
- **AI attribution check:** **FAIL:** 6 commit(s)
- **Decision:** **`close with pointer (keep branch)`**
- **Justification:** Characterisation recipe for ideal-parts slice A and full SECMRD matrix; head pinned by `ready/bench-ideal-matrix` (`e9d5e778`) and ancestor pinned by `ready/bench-ideal-parts` (`23e11fef`). Both runs completed (`origin/evidence` `04c9ecd9` and `7dd7b070`) and results landed on `master` in `#899` and `#906` (`docs/journal/2026-11-24-algorithm-level/ideal-parts/results/`). Wholly subsumed as an ancestor of `#909` and `#917`. Carries 6 AI-attributed commits (`Claude <noreply@anthropic.com>`, `Co-Authored-By`) — must never merge as-is and must never be rewritten.
- **Ready-to-post note:**
  > Closing completed characterisation recipe PR. Both runs are archived (`origin/evidence` `04c9ecd9` slice A, `7dd7b070` full matrix) and written up in `#899` and `#906` (`docs/journal/2026-11-24-algorithm-level/ideal-parts/results/`). Superseded by `#909`/`#917`. Branch `bench/ideal-parts` and tags `ready/bench-ideal-parts` and `ready/bench-ideal-matrix` remain intact and unrewritten.

### #909 — bench(ideal-parts): the next matrix run's harness: passes hashed off the reader, per-decode timeout
- **Branch:** `bench/ideal-matrix-next` -> `master` (head `6b4b5f239b3d2830b75dcf9f79534ba1b2e474f2`, 15 commits, 39 files)
- **State:** `OPEN` / `MERGEABLE` | Created `2026-11-25` | Updated `2026-11-26`
- **Pins:** anc tags: `ready/bench-ideal-parts`@23e11fef, `ready/bench-ideal-matrix`@e9d5e778; evidence: `zero-copy-floor` (cc8ba137); `pooled-scan` (c71c4a56); queue: `bench-zero-copy-floor`; base of: #932
- **AI attribution check:** **FAIL:** 7 commit(s) + PR body footer
- **Decision:** **`keep as evidence branch`**
- **Justification:** Carries `#921` (`adcf607a`, zero-copy `OnceLock` oracles, `BlockBuf::Shared`, `E_full`, floor accounting), plus ancestor pins `ready/bench-ideal-parts` (`23e11fef`) and `ready/bench-ideal-matrix` (`e9d5e778`). Active base branch for `#932` and parent of `#917` (`bench-zero-copy-floor`). Neither `src/scan_oracle.rs`, `src/supplied_codes.rs`, nor the zero-copy oracles are on `master`. Carries 7 AI-attributed commits and an AI body footer — NEVER merge as-is and NEVER rewrite. Keep open as frozen evidence/reference branch until a clean-room frontier PR ports the zero-copy harness onto `master` (e.g. for `redraw` / `warm-paired-oracle-k1`).
- **Ready-to-post note:**
  > Keeping open as a frozen evidence/characterisation reference branch (do not merge as-is: carries 7 AI-attributed commits; do not rewrite or delete: pinned by `#917` / `cc8ba137` and `#932` / `c71c4a56`). Required until zero-copy oracles and `bench-zero-copy-floor` are clean-room ported onto `master` for `redraw` (`frontier/README.md`).

### #917 — bench-zero-copy-floor and registration-floor.md: the decoder's residual with the handover copies removed (held for the zero-copy oracles)
- **Branch:** `bench/zero-copy-floor` -> `master` (head `1e27e46f972f37cf25678c1ba43c936453d772ba`, 25 commits, 43 files)
- **State:** `OPEN` / `MERGEABLE` | Created `2026-11-26` | Updated `2026-11-26`
- **Pins:** head tag: `ready/bench-zero-copy-floor`; anc tags: `ready/bench-ideal-parts`@23e11fef, `ready/bench-ideal-matrix`@e9d5e778; evidence: `zero-copy-floor` (cc8ba137); queue: `bench-zero-copy-floor`
- **AI attribution check:** **FAIL:** 17 commit(s) + PR body footer
- **Decision:** **`keep as evidence branch`**
- **Justification:** Carries the `bench-zero-copy-floor` recipe and C1a registration (`registration-floor.md`), head pinned by `ready/bench-zero-copy-floor` (`1e27e46f`) and `origin/evidence cc8ba137` (`zero-copy-floor/run-20261126T112751Z`; results landed in `#931`). Critically, `bench-zero-copy-floor` is NOT on `master`, yet `frontier/README.md` (`redraw`) specifies re-running `bench-zero-copy-floor` whenever a merge moves the shipped 8-worker wall by >= 5%. Carries 17 AI-attributed commits and an AI body marker — NEVER merge as-is and NEVER rewrite. Keep open as frozen evidence/redraw reference branch until a clean-room port lands on `master`.
- **Ready-to-post note:**
  > Keeping open as a frozen evidence/redraw reference branch (do not merge as-is: carries 17 AI-attributed commits; do not rewrite or delete: pinned by `ready/bench-zero-copy-floor` at `1e27e46f` and `origin/evidence` `cc8ba137`, results in `#931`). Holds the only copy of `bench-zero-copy-floor` referenced by `frontier/README.md` (`redraw`) until clean-room ported to `master`.

### #932 — research(pooled-scan): the pooled scan's prototype, exact, and its kill test's recipe
- **Branch:** `research/pooled-scan` -> `bench/ideal-matrix-next` (head `1f6a578ce3c1d48c0e1ff67e43c7c87a4aa11186`, 5 commits, 18 files)
- **State:** `OPEN` / `MERGEABLE` | Created `2026-11-26` | Updated `2026-11-26`
- **Pins:** head tag: `ready/bench-pooled-scan`; evidence: `pooled-scan` (c71c4a56); queue: `bench-pooled-scan`
- **AI attribution check:** clean
- **Decision:** **`close with pointer (keep branch)`**
- **Justification:** Pooled-scan prototype and kill-test recipe (`bench-pooled-scan`, base `bench/ideal-matrix-next`), head pinned by `ready/bench-pooled-scan` (`1f6a578c`) and `origin/evidence c71c4a56` (`pooled-scan/run-20261126T133034Z`). Verdict is final **KILL** on K1 (`#942`, `frontier/dead-ends.tsv`, `docs/journal/2026-11-24-algorithm-level/pooled-scan/results/README.md`; C2 frozen). Zero AI attribution. Close the PR so a KILLED prototype does not sit open, while keeping branch `research/pooled-scan` and tag `ready/bench-pooled-scan` intact for `pooled-scan-hybrid` and `warm-paired-oracle-k1` reference.
- **Ready-to-post note:**
  > Closing after final KILL verdict on K1 (`#942`, `frontier/dead-ends.tsv`, `docs/journal/2026-11-24-algorithm-level/pooled-scan/results/README.md`; `origin/evidence` `c71c4a56`). Follow-up design work is tracked in `frontier/claims/pooled-scan-hybrid.toml`. Branch `research/pooled-scan` and tag `ready/bench-pooled-scan` (`1f6a578c`) remain intact and unrewritten.

## 2. Live Noticeboard & Evidence Branches Kept Open (3 PRs)

These three PRs remain open after triage:

- **#145** (`research/compact30-channel` @ `e52252f5`): **LIVE WORKSTATION EVIDENCE NOTICEBOARD — MUST STAY OPEN.** Per `CLAUDE.md` ('The coordinator-era machinery') and `bench/workstation-queue.txt` (`# channel: 145`), `#145` is retired for agent routing but remains active as the noticeboard where `just submit` posts every workstation evidence bundle's identity and SHA-256 digest.
- **#909** (`bench/ideal-matrix-next` @ `6b4b5f23`): Carries `#921` (`adcf607a`, zero-copy `OnceLock` oracles, `BlockBuf::Shared`, `E_full`, floor accounting), plus ancestor pins `ready/bench-ideal-parts` (`23e11fef`) and `ready/bench-ideal-matrix` (`e9d5e778`). Active base branch for `#932` and parent of `#917` (`bench-zero-copy-floor`). Neither `src/scan_oracle.rs`, `src/supplied_codes.rs`, nor the zero-copy oracles are on `master`. Carries 7 AI-attributed commits and an AI body footer — NEVER merge as-is and NEVER rewrite. Keep open as frozen evidence/reference branch until a clean-room frontier PR ports the zero-copy harness onto `master` (e.g. for `redraw` / `warm-paired-oracle-k1`).
- **#917** (`bench/zero-copy-floor` @ `1e27e46f`): Carries the `bench-zero-copy-floor` recipe and C1a registration (`registration-floor.md`), head pinned by `ready/bench-zero-copy-floor` (`1e27e46f`) and `origin/evidence cc8ba137` (`zero-copy-floor/run-20261126T112751Z`; results landed in `#931`). Critically, `bench-zero-copy-floor` is NOT on `master`, yet `frontier/README.md` (`redraw`) specifies re-running `bench-zero-copy-floor` whenever a merge moves the shipped 8-worker wall by >= 5%. Carries 17 AI-attributed commits and an AI body marker — NEVER merge as-is and NEVER rewrite. Keep open as frozen evidence/redraw reference branch until a clean-room port lands on `master`.

## 3. Merge Candidates (`merge after review + AI-trailer check`, 2 PRs)

### #137 — chore(pre-commit): autoupdate hooks
- **Branch:** `pre-commit-ci-update-config` -> `master` (`d939c5a3665f2d49e9061fdae2e9360097fe4ac1`)
- **AI check:** clean (bot author) | **Mergeable:** `MERGEABLE`
- **Justification:** Automated `pre-commit.ci` hook version bump on `.pre-commit-config.yaml` (`ty-pre-commit` v0.0.91 -> v0.0.108, `conventional-pre-commit` v4.4.0 -> v4.5.0). Still `MERGEABLE` with green CI (13 SUCCESS); commit author is `pre-commit-ci[bot]` (not an AI agent, passes `no-ai-attribution.yml`). Merging unblocks fresh `pre-commit.ci` autoupdates; alternatively close so `pre-commit.ci` opens a newer update.
- **Execution checklist:** Ready for independent review and merge: verify `no AI attribution`, `check-commit`, and `pre-commit.ci` pass on current base, then squash-merge with an explicit commit message by `Louis Maddox <louismmx@gmail.com>` (or close to trigger a fresh `pre-commit.ci` autoupdate PR).

### #586 — docs(cells): the one-scan writer's safety premise, and its memory-lever reader assertion (Guardian D113 notes)
- **Branch:** `docs/owned-writer-notes` -> `master` (`8cd2d81f079798859e0b30506a53bf70e6561008`)
- **AI check:** **FAIL:** PR body footer | **Mergeable:** `MERGEABLE`
- **Justification:** Guardian D113 notes 5 and 6 on `src/cells_owned.rs`: clarifies the conditional memory-safety proof in comments and adds a `#[cfg(all(test, feature = "memory-lever"))]` entry assertion in `write_owned`. Zero production path changes, cleanly `MERGEABLE`, CI was green (25 SUCCESS), and the single commit has 0 AI attribution (`Louis Maddox <louismmx@gmail.com>`). **Blocker:** PR body carries a `Generated by [Claude` footer — MUST strip the footer from the PR body before re-running `no AI attribution` and merging.
- **Execution checklist:** Ready for independent review and merge after cleanup: (1) strip the AI generator footer from the PR body so `.github/workflows/no-ai-attribution.yml` passes (the commit itself is clean); (2) confirm CI (`no AI attribution`, `check-commit`, `pre-commit.ci`, Rust/Miri on `src/cells_owned.rs`) is green; (3) squash-merge with an explicit message via `coord.py head-sha` + `expectedHeadSha`.

## 4. Salvage / Rebase onto Frontier Claims (6 PRs)

### #690 — docs(blog): besieging hard problems, rewritten around the wormhole and the research lines
- **Branch:** `docs/blog-besieging-rewrite` -> `master` (`9ff0b449cd5b46454206a21233d01a95f2759441`)
- **Pins:** none | **AI check:** clean | **Mergeable:** `CONFLICTING`
- **Justification:** Rewrites `docs/site/blog/2026-11-21-besieging-hard-problems.md` in place (at Louis's request after `#689`). Master still serves the original `#689` text. Conflicted on `zensical.toml`, `docs/site/blog/index.md`, and `cards.json`; zero AI attribution. Needs Louis's review (`needs_louis`) and rebase + site-rule pass (ensure zero agent/model names, run `publish-page`; no redirects).
- **Ready-to-post pointer / routing note:**
  > Salvage via a frontier blog card (or confirm with Louis under `needs_louis` whether the in-place rewrite of `2026-11-21-besieging-hard-problems.md` is still wanted). Rebase onto `master`, audit for zero agent/model names, and pass `publish-page` before merging; keep branch `docs/blog-besieging-rewrite` until decided.

### #762 — perf(rle): in-place RLE1 with no upfront shift
- **Branch:** `perf/rle1-zero-shift` -> `master` (`58ab9b3d8da917fc4a12d7a5ac22d12a143a2d58`)
- **Pins:** none | **AI check:** clean | **Mergeable:** `MERGEABLE`
- **Justification:** Zero-shift backward-zone in-place RLE1 (`src/rle.rs`, `examples/occupancy.rs`, `src/trace.rs`), `MERGEABLE`, 0 AI attribution, green CI (27 SUCCESS). Was neutral on 4T CI EPYC (`+0.4% [-0.7%, +1.6%]`), never i9-tested at 8-10W where `#952` (`single-pass-rle1-crc`) just uncovered RLE1's 2.55-2.9x LLC eviction cliff (+767 MiB DRAM) driven by `unrle_in_place`'s upfront `copy_within` shift. Directly feeds frontier claim `single-pass-rle1-crc`.
- **Ready-to-post pointer / routing note:**
  > Salvage onto frontier claim `single-pass-rle1-crc`: evaluate this zero-shift `unrle_in_place` implementation against that claim's frozen `stop_test` (1W A9 reduction >= 35% or 8-10W LLC/DRAM savings >= 2.0% worker CPU) before any production merge. Do not merge on old 4T CI data alone.

### #790 — art(thomas-murphy): the calendar-head winged serpent as the post's burrowers
- **Branch:** `docs/burrowers-calendar` -> `master` (`0cd7e9bde97644b127f3f35a97c1a7787e14ac29`)
- **Pins:** none | **AI check:** clean | **Mergeable:** `CONFLICTING`
- **Justification:** Replaces 7 of 9 burrowers in `docs/site/blog/conversations/2026-11-22-thomas-murphy.html` with the calendar-head winged serpent from brief 5 (the creature Louis kept). Explicitly gated on Louis's visual preview and approval; conflicted on `convo.js`/HTML; 0 AI attribution.
- **Ready-to-post pointer / routing note:**
  > Salvage by rebasing `docs/burrowers-calendar` onto `master` and routing preview/approval to Louis (`needs_louis`), or close if Louis declines the visual update.

### #806 — docs(method): every quoted figure carries its evidence scope; next-round candidates for the retrospective
- **Branch:** `docs/evidence-scope` -> `master` (`75c57470020cc2df19a73c5308074e6eadfa03fa`)
- **Pins:** none | **AI check:** clean | **Mergeable:** `MERGEABLE`
- **Justification:** Explicit **RULES D** rule-change draft awaiting Louis (`docs/team/research-charter/METHOD.md` §6a six-label evidence scope + `docs/journal/2026-11-24-next-round-candidates.md`). `MERGEABLE`, green CI, 0 AI attribution. Because `docs/team/` is being migrated/archived under `migrate-rules` and `archive-team-strata`, fold §6a into `migrate-rules` (gated on Louis under RULES D) and salvage the journal note cleanly.
- **Ready-to-post pointer / routing note:**
  > Salvage into frontier claim `migrate-rules` (RULES D — requires Louis's approval for the evidence-scope rule) and split or carry `docs/journal/2026-11-24-next-round-candidates.md` onto `master`. Keep branch `docs/evidence-scope` intact until integrated.

### #861 — perf(scanner): pooled symbol buffers without re-zeroing, and the seam window, together (#846 + #857)
- **Branch:** `perf/scanner-combined` -> `master` (`3ab91bf9d4502a9b06f205a51e37591a746c3a37`)
- **Pins:** evidence: `scanner-plumbing` (33568fad) | **AI check:** clean | **Mergeable:** `MERGEABLE`
- **Justification:** Combined scanner copy-elimination (`#846` pooled `block::Syms` no re-zeroing + `#857` seam-window copy fix), `MERGEABLE`, 0 AI attribution. In journal §79 (`origin/evidence 33568fad`), unspun 9T was noisy (NOT ADOPTED), while default 10/12T were FASTER (~2%) and explicit 9-10W were FASTER by **6-8%** with -2-3.5% RSS. Journal §§84 and 86 left `#861` held for spun/pinned re-testing on the v0.5.14 8-worker regime (`#878`), where S is 179 ms of the 195 ms wall.
- **Ready-to-post pointer / routing note:**
  > Salvage onto a dedicated frontier claim (rebased on v0.5.14+ `master`) with a frozen registration using 100 ms pre-spin and pinned-scanner or high-round noise control (`docs/journal/2026-11-24-algorithm-level.md` §§79, 84, 86). Subsumes `#846` and `#857`.

### #879 — perf(scan): one bit-count subtraction a batch in the Huffman decode
- **Branch:** `perf/scan-batched-bits` -> `master` (`48d9f7d9ee772b9f26438bb6399520402e91c5fd`)
- **Pins:** evidence: `scanner-plumbing` (a93afab5) | **AI check:** clean | **Mergeable:** `MERGEABLE`
- **Justification:** Batched bit-count subtraction in Huffman group decode (`src/huff.rs`, `src/bits.rs`), `MERGEABLE`, 0 AI attribution. Tested on i9 in journal §86 (`origin/evidence a93afab5`): NOT ADOPTED at 10/12T (`1.001`, `0.994`, ±2-3% placement noise), FASTER at explicit 9W (`0.972 [0.953, 0.991]`), ~2% scanner micro-win locally. Explicitly held (not rejected) in §86 for a pinned-scanner or 64-round registration.
- **Ready-to-post pointer / routing note:**
  > Low-priority salvage candidate: keep branch `perf/scan-batched-bits` (pinned by `a93afab5` in §86) and either bundle with `#861` in a future low-noise scanner-micro frontier claim or close if superseded by `pooled-scan-hybrid`.

## 5. Complete Triage Table: All 59 Remaining Pre-#945 Open PRs

Every open pre-`#945` PR outside Section 1's 8 named PRs is listed below, grouped by origin. Together, Section 1 (8 PRs) and Section 5 (59 PRs) account for all **67** open coordinator-era PRs.

### 5.1 Live Infrastructure, Merge Candidates & Salvage PRs (9 PRs — details in Sections 2-4)

| PR | Branch (`head` -> `base`) | Head SHA | Evidence / Tag Pin Status | Mergeable | AI Check | Decision | Ready-to-Post Pointer Note |
|---:|---|---|---|---|---|---|---|
| #137 | `pre-commit-ci-update-config` -> `master` | `d939c5a3` | none | `MERGEABLE` | clean (bot author) | `merge after review + AI-trailer check` | Ready for independent review and merge: verify `no AI attribution`, `check-commit`, and `pre-commit.ci` pass on current base, then squash-merge with an explicit commit message by `Louis Maddox <louismmx@gmail.com>` (or close to trigger a fresh `pre-commit.ci` autoupdate PR). |
| #145 | `research/compact30-channel` -> `research/compact30-constructor` | `e52252f5` | live `# channel: 145` noticeboard | `MERGEABLE` | clean | `keep as evidence branch` | Keep open permanently as the workstation evidence noticeboard (`CLAUDE.md`, `bench/workstation-queue.txt` `# channel: 145`). Never merge, never close, never rewrite. |
| #586 | `docs/owned-writer-notes` -> `master` | `8cd2d81f` | none | `MERGEABLE` | **FAIL:** PR body footer | `merge after review + AI-trailer check` | Ready for independent review and merge after cleanup: (1) strip the AI generator footer from the PR body so `.github/workflows/no-ai-attribution.yml` passes (the commit itself is clean); (2) confirm CI (`no AI attribution`, `check-commit`, `pre-commit.ci`, Rust/Miri on `src/cells_owned.rs`) is green; (3) squash-merge with an explicit message via `coord.py head-sha` + `expectedHeadSha`. |
| #690 | `docs/blog-besieging-rewrite` -> `master` | `9ff0b449` | none | `CONFLICTING` | clean | `salvage / rebase onto frontier claim` | Salvage via a frontier blog card (or confirm with Louis under `needs_louis` whether the in-place rewrite of `2026-11-21-besieging-hard-problems.md` is still wanted). Rebase onto `master`, audit for zero agent/model names, and pass `publish-page` before merging; keep branch `docs/blog-besieging-rewrite` until decided. |
| #762 | `perf/rle1-zero-shift` -> `master` | `58ab9b3d` | none | `MERGEABLE` | clean | `salvage / rebase onto frontier claim` | Salvage onto frontier claim `single-pass-rle1-crc`: evaluate this zero-shift `unrle_in_place` implementation against that claim's frozen `stop_test` (1W A9 reduction >= 35% or 8-10W LLC/DRAM savings >= 2.0% worker CPU) before any production merge. Do not merge on old 4T CI data alone. |
| #790 | `docs/burrowers-calendar` -> `master` | `0cd7e9bd` | none | `CONFLICTING` | clean | `salvage / rebase onto frontier claim` | Salvage by rebasing `docs/burrowers-calendar` onto `master` and routing preview/approval to Louis (`needs_louis`), or close if Louis declines the visual update. |
| #806 | `docs/evidence-scope` -> `master` | `75c57470` | none | `MERGEABLE` | clean | `salvage / rebase onto frontier claim` | Salvage into frontier claim `migrate-rules` (RULES D — requires Louis's approval for the evidence-scope rule) and split or carry `docs/journal/2026-11-24-next-round-candidates.md` onto `master`. Keep branch `docs/evidence-scope` intact until integrated. |
| #861 | `perf/scanner-combined` -> `master` | `3ab91bf9` | evidence: `scanner-plumbing` (33568fad) | `MERGEABLE` | clean | `salvage / rebase onto frontier claim` | Salvage onto a dedicated frontier claim (rebased on v0.5.14+ `master`) with a frozen registration using 100 ms pre-spin and pinned-scanner or high-round noise control (`docs/journal/2026-11-24-algorithm-level.md` §§79, 84, 86). Subsumes `#846` and `#857`. |
| #879 | `perf/scan-batched-bits` -> `master` | `48d9f7d9` | evidence: `scanner-plumbing` (a93afab5) | `MERGEABLE` | clean | `salvage / rebase onto frontier claim` | Low-priority salvage candidate: keep branch `perf/scan-batched-bits` (pinned by `a93afab5` in §86) and either bundle with `#861` in a future low-noise scanner-micro frontier claim or close if superseded by `pooled-scan-hybrid`. |

### 5.2 Retired Coordinator-Era Role Inboxes (5 PRs — coordinated with `archive-team-strata`)

| PR | Branch (`head` -> `base`) | Head SHA | Evidence / Tag Pin Status | Mergeable | AI Check | Decision | Ready-to-Post Pointer Note |
|---:|---|---|---|---|---|---|---|
| #160 | `optimizer/inbox` -> `optimizer/inbox-base` | `9ace940b` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing retired coordinator-era role inbox (26 November 2026 move to `frontier/`, `CLAUDE.md`, `archive-team-strata`). Work now runs through `frontier/claims/` and `frontier/deltas/`. Branch `optimizer/inbox` is preserved as history. |
| #161 | `guardian/inbox` -> `guardian/inbox-base` | `53f12093` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing retired coordinator-era role inbox (26 November 2026 move to `frontier/`, `CLAUDE.md`, `archive-team-strata`). Safety objections now live in `frontier/flags/`. Branch `guardian/inbox` is preserved as history. |
| #162 | `researcher/inbox` -> `researcher/inbox-base` | `8b186e3b` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing retired coordinator-era role inbox (26 November 2026 move to `frontier/`, `CLAUDE.md`, `archive-team-strata`). Registrations and reviews now live in `frontier/claims/` and `frontier/deltas/`. Branch `researcher/inbox` is preserved as history. |
| #222 | `review/oracle-intention` -> `master` | `38251872` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing retired coordinator-era role inbox (26 November 2026 move to `frontier/`, `CLAUDE.md`, `archive-team-strata`). Shared status is now generated by `frontier/tools/frontier.py status` (mirrored to `#943`). Branch `review/oracle-intention` is preserved as history. |
| #314 | `pathfinder/inbox` -> `pathfinder/inbox-base` | `232b87e9` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing retired coordinator-era role inbox (26 November 2026 move to `frontier/`, `CLAUDE.md`, `archive-team-strata`). Exploration and probes now live in `frontier/claims/`. Branch `pathfinder/inbox` is preserved as history. |

### 5.3 Retired Session & Campaign Branches (3 PRs, besides `#497`)

| PR | Branch (`head` -> `base`) | Head SHA | Evidence / Tag Pin Status | Mergeable | AI Check | Decision | Ready-to-Post Pointer Note |
|---:|---|---|---|---|---|---|---|
| #313 | `claude/agent-swarm-fable-breakthrough-0u7j4d` -> `master` | `f1fe26f4` | none | `MERGEABLE` | **FAIL:** 3 commit(s) + PR body footer | `close with pointer (keep branch)` | Closing historical Pathfinder research session PR (9-10 Nov). Cited designs and measurements are recorded in `docs/journal/2026-11-10-*` and `docs/journal/monodromy/2026-11-10-rauzy-veech.md`. Branch `claude/agent-swarm-fable-breakthrough-0u7j4d` is preserved intact. |
| #553 | `research/phrase-bwt-rle1` -> `master` | `79688e06` | none | `MERGEABLE` | clean (noreply author) | `close with pointer (keep branch)` | Closing stale coordinator-era draft research PR. Branch `research/phrase-bwt-rle1` is preserved intact as historical archive; active algorithmic work is tracked in `frontier/claims/`. |
| #607 | `research/astra-e-moonshot` -> `master` | `cc879af7` | evidence: `astra-e` (f039a7a9,8bced3f8,c381da23,83c10a7b+3); queue: `bench-astra-e-lookahead`, `bench-astra-e-narrow-short-dry`, `bench-astra-e-narrow-short`+6 | `MERGEABLE` | clean (first 100 of 1146) | `close with pointer (keep branch)` | Closing superseded Astra E moonshot draft PR. All 7 workstation bundles remain archived under `origin/evidence` (`astra-e/*`) and branch `research/astra-e-moonshot` is preserved intact. |

### 5.4 Completed Negative / Exploratory Research Prototypes (8 PRs)

| PR | Branch (`head` -> `base`) | Head SHA | Evidence / Tag Pin Status | Mergeable | AI Check | Decision | Ready-to-Post Pointer Note |
|---:|---|---|---|---|---|---|---|
| #652 | `research/nspc-prototype` -> `master` | `edb3c1b8` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed research PR (verdict: NOT WORTH IT, recorded in `docs/site/blog/2026-11-20-research-by-obligation.md`). Branch `research/nspc-prototype` is preserved intact. |
| #653 | `research/open-phase-split` -> `master` | `b12b5ff8` | base of: #654, #656, #659 | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing completed measurement-only research PR (superseded by `character` and `ideal-parts`; cited in `docs/site/blog/2026-11-20-research-by-obligation.md`). Close after child PRs `#654`, `#656`, `#659`; branch `research/open-phase-split` is preserved intact. |
| #654 | `research/open-fused-wiring` -> `research/open-phase-split` | `f04537f9` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed negative research PR (4.6-9.3% slower natively; recorded in `docs/site/blog/2026-11-20-research-by-obligation.md`). Branch `research/open-fused-wiring` is preserved intact. |
| #656 | `research/open-emit-pipeline` -> `research/open-phase-split` | `31b77121` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed negative research PR (not adopted; recorded in `docs/site/blog/2026-11-20-research-by-obligation.md`). Branch `research/open-emit-pipeline` is preserved intact. |
| #657 | `research/open-limit-preference` -> `bench/open-admission` | `75cdfba5` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed research census PR (not adopted; `LIMIT = 16` ceiling recorded in `docs/site/blog/2026-11-20-research-by-obligation.md`). Branch `research/open-limit-preference` is preserved intact. |
| #659 | `research/open-backward-native` -> `research/open-phase-split` | `20a6e5eb` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed negative research PR (1.10-1.15x slower natively; recorded in `docs/site/blog/2026-11-20-research-by-obligation.md`). Branch `research/open-backward-native` is preserved intact. |
| #664 | `research/open-u16q` -> `master` | `58bd0c43` | none | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing superseded research PR (null first port documented in `docs/journal/2026-11-20-open-u16q.md`; shipped via `#668`). Branch `research/open-u16q` is preserved intact. |
| #666 | `research/open-lrb-vec16` -> `master` | `ef248f78` | none | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing completed research PR (counter queues shipped via `#668`, fusion rejected; recorded in `docs/site/blog/2026-11-21-sixteen-at-a-stroke.md`). Branch `research/open-lrb-vec16` is preserved intact. |

### 5.5 NOT ADOPTED / Null / Subsumed `perf/*` PRs & Unqueued Draft Recipe (11 PRs)

| PR | Branch (`head` -> `base`) | Head SHA | Evidence / Tag Pin Status | Mergeable | AI Check | Decision | Ready-to-Post Pointer Note |
|---:|---|---|---|---|---|---|---|
| #734 | `perf/open-r5-stack` -> `master` | `e8f1afa2` | evidence: `open-r5` (34c0b8bd,b59e93e6); queue: `bench-open-r5` | `MERGEABLE` | **FAIL:** PR body footer | `close with pointer (keep branch)` | Closing NOT ADOPTED candidate PR (both i9 runs archived at `origin/evidence` `34c0b8bd` and `b59e93e6`; saved CPU 0.950 with flat wall 1.000, recorded in `docs/site/blog/2026-11-22-correctness-is-uniform-cost-is-not.md`). Branch `perf/open-r5-stack` is preserved intact. |
| #741 | `perf/parscan-c37` -> `master` | `d57ec321` | evidence: `parscan-c37` (5fed1ffc); queue: `bench-parscan-c37` | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing NOT ADOPTED candidate PR (`origin/evidence` `5fed1ffc`; Decision 2 block reader merged separately via `#745`, Decision 1 parscan not adopted per `docs/journal/2026-11-22-parscan-c37-registration.md`). Branch `perf/parscan-c37` is preserved intact. |
| #765 | `perf/mtf-u128` -> `master` | `13a99995` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing un-adopted scalar-only MTF PR (CI x86_64 bench neutral at +1.3% [-0.5%, +3.2%]; superseded by `mtf-fold-census` / `mtf-fold-build`). Branch `perf/mtf-u128` is preserved intact if a non-AVX2 claim revisits scalar MTF. |
| #770 | `perf/parscan-stream` -> `master` | `2618c54c` | evidence: `parscan-stream` (f693e20b); queue: `bench-parscan-stream` | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing NOT ADOPTED candidate PR (`origin/evidence` `f693e20b`; regression cause recorded in `#930`, tiled zero-fill split tested in `#785`). Follow-up scan work is tracked in `frontier/claims/pooled-scan-hybrid.toml`. Branch `perf/parscan-stream` is preserved intact. |
| #778 | `perf/huff-table-builder` -> `master` | `637d48b4` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing un-adopted micro-optimisation PR (CI paired bench neutral at +1.1% [-0.7%, +2.9%]). Branch `perf/huff-table-builder` is preserved intact. |
| #781 | `perf/open-route-no-mark` -> `master` | `ad69b0c9` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing un-adopted micro-optimisation PR (CI bench null at -0.3% [-1.9%, +1.4%]; superseded by `forward-emission`). Branch `perf/open-route-no-mark` is preserved intact. |
| #782 | `perf/open-route-small-block` -> `master` | `bb27b38f` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing un-adopted micro-optimisation PR (CI bench inconclusive at -3.8% [-9.6%, +2.4%], median 1,464 vs 1,464 MB/s; superseded by `forward-emission` / `rauzy-output`). Branch `perf/open-route-small-block` is preserved intact. |
| #783 | `perf/open-route-adaptive` -> `master` | `7f89772f` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing un-adopted candidate PR (CI paired bench null at -0.9% [-3.1%, +1.3%], never queued on i9; superseded by `forward-emission` and `rauzy-output`). Branch `perf/open-route-adaptive` is preserved intact. |
| #784 | `bench/open-route-adaptive` -> `master` | `09290846` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing unqueued draft bench recipe PR (paired with `#783`; no ready tag or evidence bundle). Branch `bench/open-route-adaptive` is preserved intact. |
| #846 | `perf/scanner-plumbing` -> `master` | `793ed644` | evidence: `scanner-plumbing` (78a85fc4) | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing component PR subsumed into `#861` (`perf/scanner-combined`). Run-1 evidence is archived at `origin/evidence` `78a85fc4` (journal §77). Branch `perf/scanner-plumbing` is preserved intact. |
| #857 | `perf/scanner-window` -> `master` | `946ba572` | none | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing component PR subsumed into `#861` (`perf/scanner-combined`). Branch `perf/scanner-window` is preserved intact. |

### 5.6 Completed Frozen Workstation Bench Recipes (23 PRs, besides `#821`, `#841`, `#843`, `#889`, `#932`)

| PR | Branch (`head` -> `base`) | Head SHA | Evidence / Tag Pin Status | Mergeable | AI Check | Decision | Ready-to-Post Pointer Note |
|---:|---|---|---|---|---|---|---|
| #562 | `bench/recalibrate` -> `master` | `cee99296` | head tag: `ready/bench-recalibrate`; evidence: `recalibrate` (2b244991); queue: `bench-recalibrate` | `CONFLICTING` | **FAIL:** PR body footer | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `2b244991`). Branch `bench/recalibrate` and tag `ready/bench-recalibrate` remain intact. |
| #590 | `bench/open-return` -> `master` | `8045ce81` | head tag: `ready/bench-open-return`; anc tags: `ready/bench-recalibrate`@cee99296; evidence: `open-return` (fa9707e9); queue: `bench-open-return` | `CONFLICTING` | **FAIL:** 1 commit(s) | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `fa9707e9`; write-up in `docs/journal/2026-11-18-open-return/`). Branch `bench/open-return` and tag `ready/bench-open-return` remain intact and unrewritten. |
| #604 | `bench/opend-sweep` -> `master` | `23b7ae31` | head tag: `ready/bench-opend-sweep`; anc tags: `ready/bench-recalibrate`@cee99296, `ready/bench-open-return`@8045ce81; evidence: `opend-sweep` (824d0954); queue: `bench-opend-sweep` | `CONFLICTING` | **FAIL:** 1 commit(s) + PR body footer | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `824d0954`). Branch `bench/opend-sweep` and tag `ready/bench-opend-sweep` remain intact and unrewritten. |
| #609 | `bench/astra-e-lookahead` -> `master` | `e5f232b8` | evidence: `astra-e` (f039a7a9); queue: `bench-astra-e-lookahead` | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed draft Astra E bench recipe PR (evidence archived under `origin/evidence` `astra-e/*`). Branch `bench/astra-e-lookahead` is preserved intact. |
| #612 | `bench/astra-e-narrow-short` -> `master` | `8c9640d8` | head tag: `ready/bench-astra-e-narrow-short`, `ready/bench-astra-e-narrow-short-dry`; evidence: `astra-e` (8bced3f8,c381da23); queue: `bench-astra-e-narrow-short-dry`, `bench-astra-e-narrow-short` | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed draft Astra E bench recipe PR (evidence archived under `origin/evidence` `astra-e/*`). Branch `bench/astra-e-narrow-short` and its `ready/*` tags remain intact. |
| #635 | `bench/open-port-2` -> `bench/open-port` | `29436646` | none | `MERGEABLE` | **FAIL:** PR body footer | `close with pointer (keep branch)` | Closing superseded draft registration PR (superseded by `bench-open-port-3` / `#638`, `origin/evidence` `b9c4966a`, which adopted 0.5.7). Branch `bench/open-port-2` is preserved intact. |
| #640 | `bench/recalibrate-057` -> `master` | `9ba4416a` | head tag: `ready/bench-recalibrate-057`; anc tags: `ready/bench-recalibrate`@cee99296; evidence: `recalibrate-057` (f6c32d79); queue: `bench-recalibrate-057` | `CONFLICTING` | **FAIL:** PR body footer | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `f6c32d79`). Branch `bench/recalibrate-057` and tag `ready/bench-recalibrate-057` remain intact. |
| #707 | `bench/open-relay8` -> `master` | `69f9f988` | head tag: `ready/bench-open-relay8`; evidence: `open-relay8` (dfe4b37e); queue: `bench-open-relay8` | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `dfe4b37e`). Branch `bench/open-relay8` and tag `ready/bench-open-relay8` remain intact. |
| #720 | `bench/open-stack` -> `master` | `be71c41d` | head tag: `ready/bench-open-stack`; evidence: `open-stack` (91f58375); queue: `bench-open-stack` | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `91f58375`; `#719` adopted and merged). Branch `bench/open-stack` and tag `ready/bench-open-stack` remain intact. |
| #722 | `bench/recalibrate-stack` -> `master` | `4792f7fc` | head tag: `ready/bench-recalibrate-stack`; anc tags: `ready/bench-recalibrate`@cee99296, `ready/bench-recalibrate-057`@9ba4416a, `ready/bench-recalibrate-058`@c695b930, `ready/bench-recalibrate-diet`@d62890fb; evidence: `recalibrate-stack` (9f203342); queue: `bench-recalibrate-stack` | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `9f203342`). Branch `bench/recalibrate-stack` and its `ready/*` tags remain intact. |
| #725 | `bench/emit-cost` -> `master` | `1aa87575` | head tag: `ready/bench-emit-cost`; evidence: `emit-cost` (d7a13999); queue: `bench-emit-cost` | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing completed diagnostic bench recipe PR (`origin/evidence` `d7a13999`). Branch `bench/emit-cost` and tag `ready/bench-emit-cost` remain intact. |
| #735 | `bench/open-r5` -> `master` | `ef8e9d88` | head tag: `ready/bench-open-r5`; evidence: `open-r5` (34c0b8bd,b59e93e6); queue: `bench-open-r5` | `CONFLICTING` | **FAIL:** PR body footer | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `34c0b8bd`, `b59e93e6`; `#734` NOT ADOPTED). Branch `bench/open-r5` and tag `ready/bench-open-r5` remain intact. |
| #740 | `bench/wormhole-plateau` -> `master` | `3dd05e45` | head tag: `ready/bench-wormhole-plateau`; evidence: `wormhole-plateau` (a811364d); queue: `bench-wormhole-plateau` | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `a811364d`; `#739` adopted and merged). Branch `bench/wormhole-plateau` and tag `ready/bench-wormhole-plateau` remain intact. |
| #742 | `bench/parscan-c37` -> `master` | `d022f2b6` | head tag: `ready/bench-parscan-c37`; evidence: `parscan-c37` (5fed1ffc); queue: `bench-parscan-c37` | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `5fed1ffc`; registration in `docs/journal/2026-11-22-parscan-c37-registration.md`). Branch `bench/parscan-c37` and tag `ready/bench-parscan-c37` remain intact. |
| #746 | `bench/block-reader-on-wormhole` -> `master` | `c3718eb1` | head tag: `ready/bench-block-reader-on-wormhole`; evidence: `block-reader-on-wormhole` (026062a9); queue: `bench-block-reader-on-wormhole` | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `026062a9`; `#745` adopted and merged). Branch `bench/block-reader-on-wormhole` and tag `ready/bench-block-reader-on-wormhole` remain intact. |
| #749 | `bench/recalibrate-plateau` -> `master` | `1bc8c8b8` | head tag: `ready/bench-recalibrate-plateau`; anc tags: `ready/bench-recalibrate`@cee99296, `ready/bench-recalibrate-057`@9ba4416a, `ready/bench-recalibrate-058`@c695b930, `ready/bench-recalibrate-diet`@d62890fb, `ready/bench-recalibrate-stack`@4792f7fc; evidence: `recalibrate-plateau` (472ce552); queue: `bench-recalibrate-plateau` | `CONFLICTING` | clean | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `472ce552`). Branch `bench/recalibrate-plateau` and its `ready/*` tags remain intact. |
| #771 | `bench/parscan-stream` -> `master` | `584d0c2e` | head tag: `ready/bench-parscan-stream`; evidence: `parscan-stream` (f693e20b); queue: `bench-parscan-stream` | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `f693e20b`; `#770` NOT ADOPTED). Branch `bench/parscan-stream` and tag `ready/bench-parscan-stream` remain intact. |
| #787 | `bench/squared-trace-prescreen` -> `master` | `7aea9ffa` | head tag: `ready/bench-squared-trace-prescreen`; evidence: `squared-trace-prescreen` (5218e05b); queue: `bench-squared-trace-prescreen` | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed diagnostic bench recipe PR (`origin/evidence` `5218e05b`; follow-up recorded in `docs/journal/2026-11-23-emit-cost-sq.md`). Branch `bench/squared-trace-prescreen` and tag `ready/bench-squared-trace-prescreen` remain intact. |
| #835 | `bench/four-arm` -> `master` | `5731db19` | head tag: `ready/bench-four-arm`; anc tags: `ready/bench-supplied-codes`@5e5495c5; evidence: `four-arm` (c9302539); queue: `bench-four-arm` | `CONFLICTING` | **FAIL:** PR body footer | `close with pointer (keep branch)` | Closing completed diagnostic bench recipe PR (`origin/evidence` `c9302539`, journal §44, data under `docs/journal/2026-11-24-algorithm-level/math-targets/data/four-arm-*`). Branch `bench/four-arm` and tag `ready/bench-four-arm` remain intact. |
| #838 | `bench/scan-profile` -> `master` | `a02e59e7` | head tag: `ready/bench-scan-profile`; evidence: `scan-profile` (41389cbe); queue: `bench-scan-profile` | `CONFLICTING` | **FAIL:** PR body footer | `close with pointer (keep branch)` | Closing completed diagnostic bench recipe PR (`origin/evidence` `41389cbe`, data under `docs/journal/2026-11-24-algorithm-level/math-targets/data/scan-profile-*`). Branch `bench/scan-profile` and tag `ready/bench-scan-profile` remain intact. |
| #849 | `bench/kernel-count` -> `master` | `97d5cfa6` | head tag: `ready/bench-kernel-count`; evidence: `kernel-count` (15646bfc) | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed diagnostic bench recipe PR (`origin/evidence` `15646bfc`, journal §76, data under `docs/journal/2026-11-24-algorithm-level/math-targets/data/kernel-count/`). Branch `bench/kernel-count` and tag `ready/bench-kernel-count` remain intact. |
| #850 | `bench/scanner-plumbing` -> `master` | `a86e5d86` | head tag: `ready/bench-scanner-plumbing`; evidence: `scanner-plumbing` (78a85fc4,33568fad,a93afab5) | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `78a85fc4`, `33568fad`, `a93afab5`; journal §§77, 79, 86). Branch `bench/scanner-plumbing` and tag `ready/bench-scanner-plumbing` remain intact. |
| #862 | `bench/kernel-choice` -> `master` | `55a4a81d` | head tag: `ready/bench-kernel-choice`; evidence: `kernel-choice` (b6ab31a6,94728a2c,fb6ab984,5d259c88) | `MERGEABLE` | clean | `close with pointer (keep branch)` | Closing completed workstation bench recipe PR (`origin/evidence` `b6ab31a6`, `94728a2c`, `fb6ab984`, `5d259c88`; `#868` and `#878` adopted and merged in v0.5.14). Branch `bench/kernel-choice` and tag `ready/bench-kernel-choice` remain intact. |

## 6. AI-Attribution Hazard Inventory & Safe Execution Sequence

### PRs Failing `.github/workflows/no-ai-attribution.yml` (17 Distinct PRs)

- **Commit-level AI attribution (7 PRs — NEVER merge as-is; NEVER rewrite evidence-pinned branches):** `#313` (3 commits), `#497` (70+ commits), `#590` (1 commit), `#604` (1 commit), `#889` (6 commits), `#909` (7 commits), `#917` (17 commits).
- **PR title/body AI footer only (10 additional PRs; 15 total with body hits):** `#562`, **`#586` (merge candidate — strip body footer first)**, `#635`, `#640`, `#734`, `#735`, `#835`, `#838`, `#841`, `#843` (plus `#313`, `#497`, `#604`, `#909`, `#917` above).

### Execution Checklist (After Independent Review & Merge of This Audit PR)

1. **Verify `stop_test`:** Never delete, rewrite, or force-push any branch; closing a PR on GitHub leaves its branch and `ready/*` tags intact.
2. **Keep open:** `#145` (live workstation evidence noticeboard), `#909` (`bench/ideal-matrix-next`), and `#917` (`bench/zero-copy-floor`).
3. **Execute closures (56 PRs):** Post each PR's ready-to-post pointer comment and close (`gh pr close <N>` WITHOUT `--delete-branch`). Close child PRs `#654`, `#656`, `#659` before closing `#653`. Coordinate role inbox closures (`#160`, `#161`, `#162`, `#222`, `#314`) with `archive-team-strata`.
4. **Execute merges (2 PRs):** On `#586`, strip the PR body's AI generator footer first, verify `no AI attribution` + Rust/Miri + `pre-commit.ci` + `check-commit` are green, then squash-merge via `coord.py head-sha` + `expectedHeadSha` with a hand-written message. Repeat standard merge gates on `#137` (or close to let `pre-commit.ci` open a fresh autoupdate).
5. **Route salvage PRs (6 PRs):** Link `#762` into `single-pass-rle1-crc`; fold `#806` into `migrate-rules` (RULES D -> Louis); route `#690` and `#790` to Louis for rebase/preview decisions; keep `#861` and `#879` linked as scanner salvage leads until re-registered under pinned/low-noise methodology.
6. **Follow-up outside this claim:** Pruning completed historical lines from `bench/workstation-queue.txt` (protected path) and clean-room porting `bench-zero-copy-floor` + `#921`'s zero-copy oracles onto `master` (for `redraw`) belong in separate claims owning those paths.
