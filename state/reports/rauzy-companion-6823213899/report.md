# Rauzy–Veech companion: completed blog preparation

Existing assignment: [363/6823213899](https://github.com/lmmx/mibz2/issues/363#issuecomment-6823213899).
Source: [full Relay 145/6823145996](https://github.com/lmmx/mibz2/pull/145#issuecomment-6823145996).
Deliverable: [PR #374](https://github.com/lmmx/mibz2/pull/374), branch
`docs/blog-rauzy-technical-companion`, exact inspected head
`5d8f11c7d4b85c3fd6d81e442195723abace1017`, based on master
`2635f04e9155dd31388b24fd3f7248f87b3ddcda`.

The complete article and local publication checks are delivered. Publication is not
performed or self-authorised. CI/preview disposition remains separately recorded below.
No production, benchmark, workstation, gate, team path, #373 or other assignment changed.

## Content and source fidelity

The main report is faithfully ported, not shortened: all 13 H2 sections, 22 display
mathematical equations and 51 table rows remain. The replayable edit ledger checks
337 source-derived paragraphs after explicit replacements. Two boxed displays have
layout-only phone reflows; every term and relation remains. Seven formal static SVGs
cover the exchange, exact right-end first return, grammar occurrence, reconstruction
pipeline, Cells arrays, possible cache targets and delimiter return section. The optional
F6 heterogeneous-count bar is omitted; the full numerical table is preserved.

Canonical raw attachment bytes are retained in deterministic gzip archives. Readable
Markdown copies add only the final LF required by repository whitespace policy:

| Attachment | Canonical bytes | SHA-256 |
|---|---:|---|
| Main report | 46,590 | `4a69f05174e03abae00f7f73c7ac4726fa9d8650e32031bcea0c9c71962d5d26` |
| Internal annotations | 7,894 | `feaf85b15ca73d1d05cd3cbeaa1c292a188d2f68b3a25564cfa780ba7a294064` |

[Preservation/figure inventory, commands and limits](https://github.com/lmmx/mibz2/blob/5d8f11c7d4b85c3fd6d81e442195723abace1017/docs/journal/2026-11-12-rauzy-companion/README.md).
[Claim/convention audit](https://github.com/lmmx/mibz2/blob/5d8f11c7d4b85c3fd6d81e442195723abace1017/docs/journal/2026-11-12-rauzy-companion/FACTS.md).
[Explicit before/after changes](https://github.com/lmmx/mibz2/blob/5d8f11c7d4b85c3fd6d81e442195723abace1017/docs/journal/2026-11-12-rauzy-companion/evidence/editorial-changes.json).

Changes beyond formatting: contextual opening and earlier-post links; shared-coordinate,
return composition, reverse-itinerary, order/closure, marked-occurrence and acceleration
bridges; precise source-access/count-versus-time/working-set qualifications. The dated
12 November note cites R1 acceptance 6822234208 and subsequent status 6823246963. The
original pending reconciliation remains identifiable as source-cutoff history. Integration,
CI reconciliation and authorised workstation timing are not represented as completed.

The article retains the distinct endpoint Cells and delimiter algorithms, all numerical
results and hardware/cache qualifications. No new mathematical error was established;
clarifications and genuine source/measurement limits are explicit. Toy-map assertions and
author-side source checks are not independent mathematical or B5 certification.

## Exact-head validation and actual rendering

The post-commit build and checks were run at the exact head above with no tracked changes:

- Strict `just docs-build`: pass. Python inventory fetch warns of proxy 403; local benchmark
  history is unavailable (zero records); neither is counted as validated remote data.
- `check_companion.py`: pass, source hashes/content/formulas/tables plus every integer point
  of the mathematical toy maps and SVG interval geometry.
- `check_prose.py`: pass, 397 sentences, 41 flagged (10%, below 15%); 21-word dek passes.
- `check_site.py`: pass, 4,459 internal links; `check_render.py`: pass, 42 built pages.
- Scoped Ruff lint/format, TOML format, repository whitespace checks: pass before commit;
  the exact committed code is unchanged. `git diff --check`: pass.
- Unchanged `look_pages.py`, via the documented local asset fixture: pass for article and
  index at desktop 1280×900 and phone 390×844, light and dark; 132 section screenshots.
- **Every screenshot was visually inspected.** After the last layout corrections, all 42
  changed images were inspected again; the other 90 were identical. The post-commit rerun
  produced **132/132 byte-identical images** to that inspected set (see manifest.json).
- Social card inspected at actual 1200×630. Two fresh-reader passes are in the PR evidence.

Fixed actual render defects: two label/arrow collisions, an unsupported SVG mapsto glyph,
wide boxed equations, a phone history table requiring horizontal scroll, first-column
nowrap and pseudocode line overflow. All table contents and mathematical meanings survive.

[All 132 original PNGs and browser report](screenshots.tar.gz).
[Exact-head per-image hashes/comparison](manifest.json).
The `gallery/` directory exposes representative light/dark desktop/phone induction and
phone history screenshots directly. This archive is separate from the article commit so
its report can cite the final immutable head without a self-referential commit hash.

Local dependency limitation: installed Chromium 154.0.8037.57 cannot use the proxy CA and
Google Fonts is blocked. The fixture uses verified Python CA requests, exact KaTeX 0.16.11
and pinned Fontsource IBM Plex 5.2.7 assets, recording every URL/size/hash. TLS verification
and production asset URLs are unchanged. This verifies the recorded local fonts, not
public CDN delivery. Ordinary site CI and the deployed preview remain the public-path
checks. No separate image-generation tool or decorative analogy replaces the mathematics.

Earlier failed attempts remain in the PR evidence: initial blocked browser assets and a
404 caused by a concurrent local rebuild (resolved by sequential build/look). This report
also retains three failed default-cache check attempts; setting the authorised writable
UV_CACHE_DIR resolved them and all actual checks then passed. No result is omitted to
claim success.

## Coordination and next action

Current CI snapshot is in ci.json. Pre-commit, documentation build, all observed Python
versions, typing, fuzz-smoke, submit and packaging jobs pass. Site look/preview and the
Rust test job remain in progress in the recorded snapshot. No CI rerun, manual
performance workflow or workstation job was launched. A pending job is not green.

Request Coordinator reconciliation and the separately authorised publication procedure
once the factual checks, rendering and **exact-head CI** are green. The later independent
Researcher exegenius rounds may run on the published article; they are not a reason to
wait before requesting publication. Direct access to the public preview fails with the environment proxy 403 (preview-access.json).
The prescribed deployed-preview inspection requires a network-capable environment; local
exact-head checks are complete. No merge or publication is performed by Codex.

`coord.py route` selects #145. Direct `coord.py audit` fails with the environment's GitHub
API proxy 403; connector reads work and are used for durable comment readback. The verify
attempt/result will be recorded with the completion receipt; a connector readback is not
mislabelled as a successful direct audit.

A GitHub comment **cannot restart this Codex session**. Current PROTOCOL §1 explicitly
requires a manual launch, with no installed Codex webhook. Do not assume dispatch or
receipt triggers execution.

Drafting began 2026-11-12 15:58 UTC. The completion cursor records stop time, within the
three-hour bound ending 18:58 UTC. All requested article content is delivered; remaining
CI/preview/publication decisions are operational dependencies, not silently omitted prose.

## Screenshot gallery

[Desktop light induction](gallery/desktop-light-induction.png) ·
[Desktop dark induction](gallery/desktop-dark-induction.png) ·
[Phone light induction](gallery/phone-light-induction.png) ·
[Phone dark induction](gallery/phone-dark-induction.png) ·
[Phone light history table](gallery/phone-light-history.png) ·
[Phone dark history table](gallery/phone-dark-history.png).

Preparation checkpoint recorded at 2026-11-12T16:49:10.721305+00:00. Packaging workflow
43261706491 is now successful; site and Test remain in progress.

Receipt: [REVIEW 6829978946](https://github.com/lmmx/mibz2/pull/145#issuecomment-6829978946)
was posted after the completed cursor and read back through the GitHub connector with
an exact body match. Direct `coord.py verify` remains blocked by proxy 403; its raw result
is retained in coord-verify.log. Site preview job 131336486023 subsequently passed;
local direct access remains blocked, and site look/Rust tests were still pending at receipt.
