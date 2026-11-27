claim: none
by: antigravity
outcome: done
evidence: docs/site/blog/2026-11-27-off-wikidata-and-eight-worker-coupling.md; frontier/blog/off-wikidata-and-coupling.toml; docs/journal/2026-11-27-explore/route-census.md (#1023); docs/journal/2026-11-27-explore/eight-worker-coupling.md (#1035); zensical build --strict; check_site.py; check_render.py; check_prose.py

**What was asked:** Publish a primary-source technical blog post covering the off-Wikidata route census (#1023) and the eight-worker coupling dissection (#1035) under the standing blog-publishing rule.
**What changed:**
- Added `docs/site/blog/2026-11-27-off-wikidata-and-eight-worker-coupling.md` with exact primary tables, KaTeX equations, pinned links to `6f8a26f5`, and mechanism-only naming (no agent names, no redirects).
- Added card to `docs/site/blog/index.md`, nav + `llmstxt` entries to `zensical.toml`, social card PNG + manifest via `docs/scripts/social_cards.py`, and blog board card `frontier/blog/off-wikidata-and-coupling.toml` (`stage = "review"`, published v1 under standing blog authority).
**What's next:** Optional follow-up PRs for bespoke causal SVG figures or prose trims after publication (`publish-page` iterate-after-live workflow).
**Caveats:** Version 1 ships without a custom inline SVG figure (data tables + KaTeX equations carry the quantitative argument cleanly); deployment to GitHub Pages follows master merge asynchronously.
