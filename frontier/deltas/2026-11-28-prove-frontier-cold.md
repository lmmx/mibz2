claim: prove-frontier-cold
by: antigravity
outcome: done
evidence: docs/notebook/2026-11-28-prove-frontier-cold.md, frontier/claims/frontier-cold-loop-fixes.toml

**What was asked:** Follow `frontier/README.md`, `CLAUDE.md`, `AGENTS.md`, and `frontier/CONTINUATIONS.md` cold on a private bare clone of `master` (`77de5b69`, two stranger clones, nothing pushed to `lmmx/mibz2`) and catalogue every `breaks`, `misleads`, and `friction` defect.

**What I found:** **16 findings** (**5 `breaks`**, **6 `misleads`**, **5 `friction`**) with verbatim commands, exit codes, and source lines in `docs/notebook/2026-11-28-prove-frontier-cold.md`:
- **`breaks` (5):** (B1) `continuation.py` `manual:<text>` pins cannot be resumed by hand (`EXIT=15`, no `--manual` flag); (B2) unbounded/zero/negative `--hours` on `frontier.py` (`claim`/`renew`) and `continuation.py` (`resume`) allow 417-day locks (`--hours 9999`) or retroactive expiry (`--hours -5`); (B3) `frontier.py mark --status review --pr 999` silently drops `--pr` and `check` passes `status = "review"` with no `pr`; (B4) `frontier.py check:2309` enforces `resolved_by != by` (flag filer) instead of != flagged work's author, inverting `flags/README.md:18`; (B5) `frontier.py:3079–3187` crashes with raw `IndexError`/`ValueError` tracebacks on 8 missing/bad-arg invocations.
- **`misleads` (6):** (M1) `frontier.py` silently ignores unknown flags/typos (`--session` on `claim`, `--agent`, `--hour`, `--bogus`); (M2) missing `--by`/`--as` silently acquire/publish as `"unknown"`; (M3) `continuation.py resume` printed `done` command omits required `--session`; (M4) `verify`/`renew` on a released hold report `"has lapsed"`; (M5) `--help` and `README.md:41` omit `review` from `mark --status`; (M6) `CLAUDE.md:25,77` omits `.github/coordination/` path, `GITHUB_TOKEN`, and `gh`'s `--match-head-commit`.
- **`friction` (5):** (F1) `frontier.py --help` exits `2`; (F2) `brief` reprints 16 KB of `README.md` before state and `--can shell,rust,repo-write` silently hides `explore`/`synthesise` (`long-context`); (F3) double `release` pushes a duplicate `released = true` commit; (F4) omitted `--session` prints `not held by session None`; (F5) `continuation.py done` example leaves `next = ""`, `show` prints raw Unix epochs, and `CLAUDE.md` is 245 lines vs `README.md:285`'s 120-line budget.
- **What passed cleanly:** atomic `claim` CAS, `owned_paths` dir/component overlap, `mechanisms` disjointness, `needs_louis`/non-open/`max_live_holds` refusals, stale-checkout authoritative `origin/master` reads for `claim` & `verify` (including path-scoped flags), non-GitHub fail-closed guards, `ledger.py`, and `continuation.py` CAS / `--independent` refusal.

**What changed:** Added `docs/notebook/2026-11-28-prove-frontier-cold.md`, registered follow-on `frontier/claims/frontier-cold-loop-fixes.toml` (`idea`, P2, sequenced after `#1341` and `brief-state-first` so `frontier/tools/frontier.py` has zero collision), and marked `prove-frontier-cold` `done`.

**What's next:** Admit and execute `frontier-cold-loop-fixes` once `#1341` and `brief-state-first` release `frontier/tools/frontier.py`. Threads deferred past the 40-min cap: live GitHub PR API `take-review`/`finish-review` paths, concurrent `continuation.py watch` contention (`EXIT=11`/`14`), and `check_prose.py` / blog-card error paths.

**Caveats:** Tested on a local bare remote without GitHub PR API; live GitHub review-transfer paths verified only for fail-closed handling here.
