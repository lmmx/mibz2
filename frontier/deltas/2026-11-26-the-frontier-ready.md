claim: the-frontier
by: claude-code
outcome: done
evidence: docs/site/blog/2026-11-26-dropping-the-coordinator.md; zensical strict build (no issues); check_render.py, check_site.py clean; look_pages.py on the post and blog/ (desktop and phone, light and dark), screenshots read

**What was asked:** move blog card `the-frontier` from `revise` to `ready`. The post goes into the site, and the `publish-page` gate passes on the final head.
**What I found:**
- **Build and checks:** the strict build passes.
- **A rendering bug:** `check_render.py` caught one. The display equation inside §5's first bullet split the list, so the next bullets ran into a paragraph. Putting the equation inline fixed it.
- **The screenshots:** I read them for the post (all sections) and the blog index, at desktop and phone width, light and dark.
  - The four figures, the tables and the KaTeX render in both themes.
  - The blog-board inset still marked "draft · this post", which is stale once the post ships. It now reads "draft · one agent" and "review · a different one".
  - On a phone, the wide tables scroll inside their frames and the SVG text is small. That is the site's usual behaviour for wide figures, and `look_pages.py` reported no overflow.
- **No agent or model names** in the post or its figure script.
**What changed:**
- Moved `docs/journal/2026-11-26-rhizome/the-frontier-draft.md` to `docs/site/blog/2026-11-26-dropping-the-coordinator.md`.
- Wired the post into the site: in `zensical.toml`, the Blog nav (top) and `llmstxt` (Optional); a card on the blog index (category Meta); and its social card, with `cards.json` updated. The other cards were left byte-identical.
- In `frontier/blog/the-frontier.toml`: `stage = "ready"`, and `links` now point at the post.
**What's next:**
- **`published`:** merge #959, with `coord.py head-sha` before and after and a squash message written by hand, then set `stage = "published"` and `release` the hold. Merging needs the maintainer's word (`ship #959`).
- **A follow-up PR after publishing:** run `link-and-gloss` on the live post. `link_audit.py` §1 lists six table-header terms (State, Flows, Events, Ownership, Stigmergy, Intellectual monoculture) as used before they are explained.
**Caveats:**
- **Three stages, one agent:** review, revise and ready were all done by this agent. The board needs only `reviewed_by` ≠ `drafted_by`, which holds, but a third reader before the merge would be prudent.
- **The social card** truncates the title after "measured…", as long titles always have.
