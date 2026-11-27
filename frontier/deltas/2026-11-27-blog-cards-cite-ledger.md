claim: blog-cards-cite-ledger
by: antigravity
outcome: done
evidence: frontier/tools/frontier.py; tests/test_frontier.py; frontier/README.md; .claude/skills/write-blog-post/SKILL.md; frontier/ledger.tsv

**What was asked:** Tie blog cards past `idea` to the shipped-changes ledger (`frontier/ledger.tsv`) and `origin/master`: `advance` cards require `changes = [<PR numbers>]` (each ledgered and merged on `origin/master`), `discovery` cards require `evidence = [paths]` (each existing); add read-only `frontier.py news` listing `default` and `experimental` ledger rows merged since the newest published post and not yet named by any card; update `frontier/README.md`, `write-blog-post`, and `tests/test_frontier.py`.
**What changed:**
- Updated `frontier/tools/frontier.py`:
  - Added `_ledger_rows()`, `_merged_prs_on_master()`, `_newest_published_post_date()`, and the read-only `news` subcommand (`python3 frontier/tools/frontier.py news`).
  - Extended `check()` for non-`legacy` blog cards past `idea`: `news_kind = "advance"` requires non-empty `changes = [<PR numbers>]` with every PR present in `frontier/ledger.tsv` and merged (first-parent `(#N)`) on `origin/master`; `news_kind = "discovery"` requires non-empty `evidence = [paths]` with every path existing relative to repo root. Legacy cards remain grandfathered.
- Updated `tests/test_frontier.py`: added `test_blog_advance_cites_ledger_and_master_and_discovery_cites_evidence` (verifying missing `changes`, unledgered PR, unmerged PR, merged+ledgered PR, missing vs existing `evidence`) and `test_frontier_news_lists_unposted_default_and_experimental_changes` (verifying `default`/`experimental` inclusion, covered-PR omission, pre-cutoff omission, and `prototype`/`target` exclusion), and updated the existing discovery fixture to supply `evidence`.
- Updated `frontier/README.md` (`"The claim schema"`, `"The blog board"`) and `.claude/skills/write-blog-post/SKILL.md` (`"First: is it a post at all?"`), and marked `blog-cards-cite-ledger` `done`.
**Verification:** `tests/test_frontier.py` (48/48 pass), `tests/test_ledger.py` (5/5 pass), `frontier.py check` (0 errors), `frontier.py news` lists the 9 unposted `default`/`experimental` rows from `2026-11-27`, and `ruff check` + `ruff format --check` pass.
**What's next:** Pairs with `claims-match-state` (`docs/scripts/check_prose.py`) so headlines and body prose match the cited ledger states.
