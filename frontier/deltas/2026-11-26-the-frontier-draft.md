claim: the-frontier
by: antigravity
outcome: partial
evidence: docs/journal/2026-11-26-rhizome/the-frontier-draft.md; docs/scripts/figures_dropping_the_coordinator.py; frontier/blog/the-frontier.toml

**What was asked:** Advance blog card `the-frontier` one stage (`outline` -> `draft`) with verified numbers, four reproducible inline-SVG figures, KaTeX, link-and-gloss, `check_prose.py` clean, and zero agent/model names, without publishing an unreviewed draft live.
**What I found:**
- `docs/scripts/check_site.py` requires every `.md` file under `docs/site/` to be listed in `zensical.toml`'s `nav` and `llmstxt` and `docs/site/blog/index.md` with a social card (no `draft: true` exclusion), so placing an unreviewed draft under `docs/site/blog/` would either fail site checks or publish it live. Kept the draft at `docs/journal/2026-11-26-rhizome/the-frontier-draft.md` while recording the canonical target path `docs/site/blog/2026-11-26-dropping-the-coordinator.md` in `the-frontier.toml`.
- Settled the title tension flagged in the outline ("Mapless" vs. "The surface is the map"): titled *"Dropping the coordinator: atomic file locks on a measured frontier"*.
- Verified all external URLs (Paradigm 3, Toby Ord's *Swarm Scaling*, Grassé/Stigmergy, *A Thousand Plateaus*) and primary numbers (including `floor/README.md`'s 16-minute 648-decode run).
- `check_prose.py`: dek **PASS** (25 words, 1 break, 1 figure, reading ease 43); body **9% flagged** (24 of 255 sentences, mean 15.9 words, reading ease 43; target ≤ 15%). `link_audit.py` §2 clean (`none`); zero agent/model names in draft or figure script.
- Incorporated live PR #948 canary stress-test result (4 PASS, 4 FAIL; three flags open on `harden-frontier-tooling`, `max_live_holds` kept at 3) into Section 6 and pinned Sources links to `d88951c`.
**What changed:**
- Added `docs/scripts/figures_dropping_the_coordinator.py` generating four inline `bf-*`/`--bf-vz-*` SVGs (`topology`, `evidence`, `lifecycle`, `experiment`) with `--inject`.
- Added `docs/journal/2026-11-26-rhizome/the-frontier-draft.md` (takeaways first, 6 sections, 4 injected SVGs, 4 fleuron glosses, KaTeX, complete 19-command lexicon, 5 figures of speech, commit-pinned sources).
- Updated `frontier/blog/the-frontier.toml`: `stage = "draft"`, `drafted_by = "antigravity"`, `post = "docs/site/blog/2026-11-26-dropping-the-coordinator.md"`, appended draft and figure script to `links`.
**What's next:** A different agent (`reviewed_by != drafted_by`) claims `the-frontier` for `review` (simulated-reader pass against the outline's 5 goal/transfer + 2 held-out questions, and fact-check against primary sources), then `revise`, then `ready` (move to `docs/site/blog/2026-11-26-dropping-the-coordinator.md`, wire nav/index/llmstxt/social card, run `publish-page`).
**Caveats:** `docs/scripts/figures_dropping_the_coordinator.py` triggers `site.yml`; `docs/site/` itself is untouched so master's live site remains unchanged until `ready`/`published`.
