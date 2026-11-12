# PR374 bounded owner-requested follow-up

Inspected final source commit: `8a01fc7922f27e41894ba02c37830e1277a155f7`; parent `5d8f11c7d4b85c3fd6d81e442195723abace1017`.
Append-only update of the existing documentation branch. No merge or publication.
This direct owner follow-up belongs to completed assignment 6823213899; no new inbox assignment is fabricated.

## Exact changes

- Figure `rv-exchange`, “Two orders, one interval exchange”, §2.1: rotate each of the three existing V-shaped arrowheads around its existing tip. A/B: −35.882137°; C: +74.545444°. Connector paths and endpoints, interval rectangles, scale, labels, caption and accessible title/description remain identical.
- Title/frontmatter/H1: “Reconstructing text from compact programs”.
- Dek/description: “This technical companion traces how compact word programs reconstruct text from the BWT, including the costs and limits of building and expanding them.”
- Replace the first contextual paragraph only: credit the earlier post's interval-exchange/Rauzy–Veech background; identify this companion's full reconstruction constructions, build/expansion costs and decoder interpretation; explicitly retain endpoint Cells versus delimiter first returns. Second contextual paragraph unchanged.
- Synchronise that article's blog-index entry, navigation/llmstxt metadata, social-card PNG and cards.json fields. Same URL.
- Append three explicit SVG corrections to the existing source-preservation editorial ledger. No previous ledger entries rewritten. Both source attachments and their hashes remain unchanged.

The entire article from its style block onward is byte-identical to the previous head after undoing only the three arrowhead transforms. No equations, tables, numerical results, section order, historical note, citations, shared CSS/JS, research code, decoder code or performance evidence changed.

## Validation and inspection

- `just docs-build`: exit 0, clean strict build. Existing limitation: Python inventory URL returns proxy403; benchmark-history data unavailable (0 series); neither changed generated tracked files.
- `check_prose.py` for the article: pass; dek 23 counted words, one break, reading ease44; body42/398 flagged (11%, target≤15%). Initial candidate dek failed two-break limit and was corrected before the committed version.
- `check_site.py`: 4,459 internal links checked; pass.
- `check_render.py`: 42 pages checked; pass.
- `check_companion.py`: pass; both source hashes,337 preserved paragraphs,22 display equations,51 table rows,13 original sections, all7 formal figures/toy maps. Initial stale-ledger failure was resolved by appending the three explicit SVG edits, not weakening the checker.
- Additional local numerical check: transformed arrowhead tip-to-base-midpoint axis has positive dot product with its connector and normalised cross product below1e-8 for each arrow. Removing only those transforms restores the exact prior figure and technical body.
- Unchanged `look_pages.py` via the previously recorded `browser_check.py` fixture: article and blog index;1280px desktop/390px phone; both themes;132 screenshots, zero reported problems. All132 inspected through22 contact sheets; opening and changed figure also inspected at full size in all4 contexts; social card inspected at1200×630.44 screenshots remain byte-identical to the previous inspected head; others reflect changed framing/figure or fractional screenshot alignment. Representative actual screenshots and full manifest retained here.
- Same pinned KaTeX0.16.11 and Fontsource IBM Plex5.2.7 fixture as original PR; browser-assets manifest unchanged. This verifies local rendering, not public CDN delivery. Normal exact-head CI/preview remains separately required for publication.
- One fresh-reader pass, scoped to revised opening per writing skill: understood headline subject, companion purpose and separate constructions; no serious confusion.
- `git diff --check`: pass. Six PR files changed,26 insertions/11 deletions plus the regenerated PNG.

## Handoff

Local bounded revision complete. Coordinator retains exact-head CI/preview reconciliation and the separate publication procedure; no merge or publish by Codex. This revision neither reopens the finished drafting assignment nor changes benchmarking dependencies.

Routing: `coord.py route --to coordinator --type REVIEW` gives145. Direct audit is proxy403-blocked; no audit-pass claim. The existing verifier hardcodes a Coordinator sender; Codex receipt is instead checked by exact authenticated connector read-back and routing/header checks, without claiming that subcommand passed.

Exact-head GitHub snapshot after push: legacy pre-commit success; site43278736712 and Test43278736806 in progress, packaging CI43278736857 queued. PR374 open/unmerged at the inspected head. No all-green or publication-ready claim.
