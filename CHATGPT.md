# Working in mibz2 from ChatGPT

There is no coordinator (retired 26 November 2026). The repository's files under
[`frontier/`](frontier/README.md) are the shared state, and PR `#145` is no longer a coordination
channel (it remains only as the workstation evidence noticeboard).

**Start here:**
1. Read [`frontier/README.md`](frontier/README.md) for the protocol and the lexicon (`take one`,
   `where are we`, `review`, `needs me`, etc.). A question gets an answer, not a PR.
2. Read the published status page on **issue #943** (and Needs Louis on **issue #944**) together
   with `frontier/claims/`, `frontier/deltas/` and `frontier/surface.tsv`.
3. For read-only work (review, exploration, mathematical audit, synthesis), read the primary files,
   commits and evidence bundles directly — never rely on prose summaries.

**Hard invariants (full rules in [`CLAUDE.md`](CLAUDE.md)):**
- **Identity:** commits as `Louis Maddox <louismmx@gmail.com>`, author and committer. No AI
  attribution anywhere in commits, PR titles, PR bodies or blog posts.
- **Provenance:** `by:`, `drafted_by` and `reviewed_by` fields in `frontier/` are internal records;
  never copy them into commit or PR metadata or blog posts.
- **Merging:** `coord.py head-sha` before and after, `expectedHeadSha`, a hand-written squash
  message, no self-merging protected files (`frontier/config.toml`), and owner approval for RULES D
  (`docs/team/RULES.md`: spending, rules, roles, Louis's workstation).
- **The site:** no redirects, ever. Name algorithms and changes by mechanism, never by model.
- **Evidence:** deltas are append-only, a stop test is frozen before data, and a verdict is never
  relaxed after the data (RULES C2).
