claim: reopening-a6b-and-segmented
by: antigravity
outcome: done
evidence: docs/site/blog/2026-11-27-reopening-a6b-and-segmented.md

Published `docs/site/blog/2026-11-27-reopening-a6b-and-segmented.md` under standing
blog-publishing authority (`CLAUDE.md`), connecting four already-merged primary results
on `origin/master`:

- `#1040` (`2915d7e7`, shipped): `within_cap` dense-band precheck in `src/open.rs`,
  cutting refused-block median ratio `1.536x -> 1.027x` (24 blocks, 5 corpora, 0 `unsafe`),
  CI pinned AMD EPYC `text` 4t `-3.7%` and `wikidata` 1t `-0.2%`.
- `#1042` (`9f89ba50`, PASS): `Segmented` fallback dissection across 78/86 off-Wikidata
  blocks, cutting `prose/text + source` `-15.44%` base / `-16.05%` native CPU (`-18.43%` /
  `-18.54%` across 6 corpora, zero extra memory) with `C_trustfill+rev`, and recording
  KKP 2-step `LF^2` (`+104.5%..+121.4%` slower) in `frontier/dead-ends.tsv`.
- `#1046` (`05c13b98`, PASS): whole-`A6b` redesign (`comb_nobc_soa_gen`), cutting `A6b`
  `-27.4%..-28.8%` and full `C+E` `-5.4%..-7.6%` across all 4 build x emit cells
  (projected `+48.7..+67.7 ms` 1W / `+8.2..+9.1 ms/w` 8W, 0 `unsafe`, 431/431 byte-exact,
  `A7` flat `-0.44%..+0.32%`).
- `#1038` (`3bb552a1`, scoped STOP): contracted return-map constructor rescue, cutting
  contraction `-9.0%` base / `-13.9%` native and fixing `runs.bz2` yet stopping cleanly on
  a `30.6 ms` bare 4-lane chase floor exceeding the `~24-25 ms` VM budget.

Included three inline SVG figures, generated the social card PNG and updated `cards.json`,
`docs/site/blog/index.md`, and `zensical.toml` (nav + `llmstxt`). Verified locally with
`zensical build --clean --strict`, `check_site.py`, `check_render.py`, `check_prose.py`
(dek PASS, 13% body flagged), zero agent names, and `frontier.py check`. Card left at
`stage = "review"` so a different agent can run an editorial pass to `published`.
