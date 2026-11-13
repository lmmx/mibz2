# Rauzy–Veech technical companion: publication evidence

Assignment [363/6823213899](https://github.com/lmmx/mibz2/issues/363#issuecomment-6823213899),
from [Relay145/6823145996](https://github.com/lmmx/mibz2/pull/145#issuecomment-6823145996).
Base master `2635f04e9155dd31388b24fd3f7248f87b3ddcda`; isolated branch
`docs/blog-rauzy-technical-companion`. Bounded drafting began12 November2026 at15:58UTC;
the completion REVIEW and own state report record stop time and the exact inspected head.
No publication, merge, production change, benchmark or workstation execution is part of
this preparation.

The deliverable is the [complete site article](../../site/blog/2026-11-12-rauzy-technical-companion.md),
with index, navigation, llms.txt configuration and social card. These are the only site
integration changes; shared CSS, JS, KaTeX and the prior articles are unchanged.
The publication handoff follows `write-blog-post`, `publish-page`, then
`coordinate-blog-post`. The relay's full-detail/formal-figure requirements take precedence
over generic advice to shorten, use analogies or add interactive steppers.

## Preserved material

The `.md.gz` archives preserve the exact two fenced attachment bodies, excluding their
terminal newline as specified by the dispatch. The adjacent readable `.md` copies contain
exactly those bytes plus one final LF required by repository whitespace checks; their
on-disk sizes are46591 and7895. Canonical sizes and hashes below refer to the unedited
decompressed archive bytes. The validator checks both the archives and this sole copy
difference; no source wording or original mathematical notation is altered.

| Source | UTF-8 bytes | SHA-256 |
|---|---:|---|
| [Main report](source/rauzy-technical-report-source.md) |46590|`4a69f05174e03abae00f7f73c7ac4726fa9d8650e32031bcea0c9c71962d5d26`|
| [Internal reading commentary](source/reading-annotations-editorial-source.md) |7894|`feaf85b15ca73d1d05cd3cbeaa1c292a188d2f68b3a25564cfa780ba7a294064`|

The second file is editorial guidance, not another published article.
[`check_companion.py`](check_companion.py) verifies these hashes, replays every recorded
editorial replacement, and asserts every resulting source paragraph is present in the
article. It also checks all22 source display equations, all51 table rows, and all13 H2
sections (executive summary, §§1–11, overall assessment). Two long boxed equations are reflowed for phone width: the same four-stage chain is
vertical, and the buffer inequality uses two lines. No mathematical term was dropped.
The source's narrative, arguments, numbers, hardware/cache and disk discussion, caveats,
references and five concluding research questions remain.

The explicit [edit inventory](evidence/editorial-changes.json) records before/after/reason.
Mechanical normalization is limited to supported dollar-delimited KaTeX and pinning master
links. Added material consists of the contextual opening/earlier-post links, formal figures,
nearby primary citations and the dated historical correction. Four prose tables have
page-local first-column wrapping, and the structural pseudocode wraps on phones; shared
stylesheets and source table/code contents are unchanged. Surgical explanatory edits
cover shared coordinate domains, composition across a removed section, reverse-itinerary
words, acceleration, order/closure cases, notation and occurrence semantics. Source-access
and historical timing limitations are stated explicitly. The article is not rewritten as
a shorter introduction. See [FACTS.md](FACTS.md) for every numerical/convention check.

The dated note separately records the Coordinator's R1 acceptance; original pending text is
identified as a historical cutoff. Integration, CI and workstation work are not called done.

## Figure inventory

All seven figures are inline SVG using existing `bf-*` styles, with accessible title and
description plus visible caption. They are static in all four rendering contexts; there
is no animation, hover-only content, keyboard-operated control or JavaScript dependency.
Static content is therefore also the no-JavaScript/reduced-motion version.

| ID | Source / purpose | Mathematical and presentation check |
|---|---|---|
| `rv-exchange` | F2.1, two orders | ABC lengths2,3,1 → CAB; same68-viewBox-units-per-row coordinate scale; translations+1,+1,−5. Equal labelled lengths in both rows; toy exchange explicitly not a measured BWT. |
| `rv-right-end` | A1, actual induced map | BA/AB with3,5 on[0,8); retain[0,5); before/after same55units per row. All retained points checked: B+2 via two returns, A−3 via one. Word orientation wA wB explained. |
| `rv-grammar` | Existing source grammar, marked occurrence | U4 expands ababab; mark expansion offset3, rotate to bababa, repeat/truncate10 gives bababababa. Offset explicitly not orig_ptr, illustrative grammar not claimed actual LF cycle. |
| `rv-pipeline` | F3 | Actual representation changes, with RLE1 and CRC obligations retained. No timing or scale claim. |
| `rv-slots` | Physical Cells storage | Same cell identity in different array positions;24-byte slots and32-bit offsets. Logical layout, not cache-line residency. |
| `rv-cache` | F5 | Registers,L1,L2,LLC,DRAM possible targets, explicitly not measured residency or capacities. |
| `rv-delimiter` | Separated first-return section | Six-point orbit, retain{0,3}; exact three-step returns and itineraries checked. Illustrative orbit, not token-order proof or actual column. |

F6's count bar was explicitly optional in the relay. It is omitted because the complete
adjacent table is more precise and avoids visually equating heterogeneous visits or time.
No decorative replacement was introduced. [Machine inventory](evidence/figure-inventory.json).

## Validation and reproducibility

From repository root:

```sh
python docs/journal/2026-11-12-rauzy-companion/check_companion.py
just docs-build
uv run --no-project -p 3.13 python docs/scripts/check_prose.py docs/site/blog/2026-11-12-rauzy-technical-companion.md
uv run --no-project -p 3.13 python docs/scripts/check_site.py
uv run --no-project -p 3.13 python docs/scripts/check_render.py
uv run --no-project -p 3.13 --with playwright==1.56.0 python docs/scripts/look_pages.py blog/2026-11-12-rauzy-technical-companion/ blog/
```

The two explicit browser targets cover every changed page. Navigation/llms.txt wiring is
checked site-wide by `check_site.py`, and all42 built pages by `check_render.py`. The local
build has no fetched bench-results history and records0 history commits; this does not
validate benchmark-page data. Its Python inventory fetch warns of proxy403; Zensical's
strict build nevertheless exits successfully with no issues. Neither limitation is hidden.

The environment blocked Playwright's browser download and Google Fonts and did not give
Chromium the proxy CA. The first unmodified browser run failed; its log is retained. The
scoped [browser fixture](browser_check.py) runs the **unchanged** `look_pages.py` with installed
Chromium154.0.8037.57 and Playwright1.56.0. It fetches the exact configured KaTeX0.16.11 through
Python's inherited verified CA and substitutes pinned Fontsource IBM Plex Sans/Mono5.2.7
faces for blocked Google Fonts, recording URL/byte/SHA inputs in
[browser-assets.json](evidence/browser-assets.json). No TLS verification is disabled and no
production asset URL or configuration is changed. This validates rendering with those
recorded font files, **not public Google Fonts delivery**; normal exact-head CI must verify
the unmodified public dependencies.

```sh
uv run --no-project -p 3.13 --with playwright==1.56.0 python docs/journal/2026-11-12-rauzy-companion/browser_check.py blog/2026-11-12-rauzy-technical-companion/ blog/ --chromium /usr/bin/chromium --out site-shots-final
uv run --no-project -p 3.13 --with playwright==1.56.0 python docs/journal/2026-11-12-rauzy-companion/browser_check.py social
```

The social command uses the existing site's HTML template,1200×630 viewport and hero;
only this article's PNG/manifest entry is written. It checks full title/dek line extent and
footer position, and the card was visually inspected. Browser file navigation was blocked,
so the same template is loaded directly and the identical hero is embedded as a data URL.

Evidence logs retain initial failures as well as passing checks. Screenshot manifests,
visual findings and exact-head verification are recorded in the completion evidence. A
source/content audit is not a substitute for looking at the actual render. The first phone
inspection found wide boxed equations, an unsupported SVG glyph, a label crossed by an
arrow, and prose table cells held on one line. These were corrected and checked again.
Two fresh-reader passes are preserved in [fresh-reader.md](evidence/fresh-reader.md); the
second answers the worked example and both held-out questions without serious confusion.
Optional style suggestions were not used to condense or replace the author's narrative.

## Publication handoff

Request the Coordinator's separate publication procedure once the factual/render checks
and **exact-head CI** are green; later Researcher rounds do not block that request or the
authorised publication procedure. This PR is a preparation deliverable, not a merge or
publication instruction. Independent mathematics/B5 review remains separate from authorship.
Actual final status, remaining CI/preview limitations, inspected commit and source hashes
belong in the PR and completion REVIEW. Codex cannot be restarted by an inbox event: the
current protocol requires a manual launch, with no installed Codex webhook.
