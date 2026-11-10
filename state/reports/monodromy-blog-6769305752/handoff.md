# Publication handoff: 6769305752

Title: A new way back from the BWT.
Source head: ae51b11a46014e5fec928b0e25235f532a473a9f (PR351 fetched in this run).
Site conventions read from master 8ed2fbdf3e8c05cdbfa426676883c652054ef17d.
Instruction pin: a2da49638ac54d7763de1c167bd054f67ffdc763.
Suggested destination: docs/site/blog/2026-11-10-rauzy-veech-reconstruction.md.
Deliverable: post.md, a full draft with four inline SVG figures. No raster supplied by the relay is embedded.

## Publication status

Manuscript and fact checks complete; **not publication-ready**. No docs branch or PR created: SPEC says write only to own state branch plus REVIEW145. Retrieved assignment/skills cannot expand that boundary. Coordinator can port the draft, wire index/navigation/llmstxt/social cards, run the site's checks, create its PR and publish. No merge or implementation was performed here.

publish-page record: full site build, check_render.py, look_pages.py and screenshot inspection **NOT RUN/PASSED**. No site checkout/build prepared for the state-branch manuscript. A local Playwright launch was attempted; after correcting TMPDIR to the writable workspace, it failed because chromium_headless_shell-1234 is not installed. This does not claim that browser validation is generally impossible. The publish-page contract must be completed on the actual integrated docs head.

check_prose.py from the master pin: dek PASS, 19 words, zero breaks, reading ease58; prior to the final one-sentence stale-example fix, body137 sentences,3 flagged (2%), reading ease48. Final rerun recorded separately if changed.
Final SVG layout and actual site CSS still need inspection in desktop/phone, light/dark. No “green” render claim.

## Goal questions and takeaway placement

Target: Changing the representation made a grammar-based inversion experiment competitive, with its comparison boundaries explicit.
Reader: programmer familiar with caches and pointers, not bzip2.
Simplest explanation and argument path were written before the draft in local blog78-plan.txt, not committed per writing skill.

| Goal | Key points | First landing |
|---|---|---|
| Motivation: why this approach? | dependent LF lookup; interval shifts; retain omitted bytes in rules | opening and §1 |
| Summary: what improved? | two local censuses; candidate/reference; not shipped decoder | opening, all table details §2 |
| Structure: what tree identity means? | primitive least rotation; expanded derivation; known tree | opening qualification, §4 worked example |
| Justification: why count is not time? | arbitrary updates cost more; free means no new boundary | opening and §5 |
| Status: what prevents deployment? | matching input boundary; unsafe review; fallback | opening, §6 |
| Held-out transfer | half steps × triple per-step cost=1.5× step time; not whole-decoder ratio | reader round2 correct |

Key-facts audit: BWT and inversion are defined in opening; run and LF before toy; image order shown by B,A versus A,B; row lengths distinguished from stored-word lengths in caption; ns/row denominators before table; run normalization before count chart; primitive/Lyndon/Cartesian definitions before tree identity. “Room walk” replaced by plain table build + walk. New word-rule notion illustrated before cell storage discussion.

## Factual audit against journal and source

| Claim | Primary location / disposition |
|---|---|
| 2.770/2.172 | §6.1: sums of separate stage minima, not paired with later cells census |
| 1.697/2.244 and 1.604/2.172 | §6.2, first and second cells runs; both retained |
| 202 and 204 /233 | §6.2; local winning blocks, not statistical significance |
| 26.2% | independently calculated 1−1.604/2.172=26.151%; rounded |
| Apparent42% original→latest | explicitly labelled historical comparison across runs/timing definitions |
| Fastest of5, Xeon container, one thread | §6 heading; no95% intervals supplied; none fabricated |
| Scan0.484 excluded | §6 input-boundary description; no assumed integrated front-end saving |
| 1.225 intermediate | §6.2 development table; contextual history, no 7.7% causal estimate |
| 1.327 and7.7% in earlier relay | not traced at pinned journal; omitted |
| 1.812/2.137/1.279/1.220 | §5.6 all233-block table; retain qualified cell-step floor |
| 29.4% and4.8% | independent arithmetic, respectively count reduction and excess over floor |
| r2−1 floor | §5.6 Theorem4; r2 initial common-refinement cells, U newly introduced boundaries; fusion can exceed model |
| Tree identity | §5.6 primitive-word hypothesis and expanded derivation; no independent certification claimed |
| cells promises/marked row/prefetch | examples/rauzy.rs and §§3.5,6.2 |
| 3.93/9.78/8.28 regressions | §6.2 other-input table; retained next to generality limits |
| 1.99 round-based ratio | §8 item5 pointing to§5.4; no inferred parallel measurement |
| #324 status | journal's stale “closed/not on master” conflicts with relay's release framing; no release assertion made. State only benchmark excludes records-based arm |
| Five-lens status | assignment says running; manuscript claims no completion at source pin |
| CI success | omitted: not independently checked, and green CI would not certify local timing |
| aab toy | independently computed rotations aab,aba,baa; L=baa; LF=[2,0,1]; two source-rule applications give ab then aab |

The next clean kernel comparison is not itself sufficient for deployment: safety, admissibility, complete decoder integration and performance qualification remain.

## Reader checks and revision

Fresh reader /root/blog78_reader got only draft and intended audience. First pass answered scope correctly but flagged missing actual induction step, undefined cell, unexplained room walk and no concrete Cartesian split. Revised around aab, with full lengths/word trace, cell definition and ranks[1,2]; removed room jargon.

Second pass answered every goal including held-out transfer. It found a stale sentence referring to the superseded periodic toy; final manuscript corrects this to aab primitive versus abab periodic. Remaining limited abstraction: general re-seating mechanics are delegated to the primary journal, not necessary to interpret the count-versus-time result. These are simulated-reader checks, not the independent post-publication exegenius review.

Coordinator LOG6769385980 assigns independent post-publication rounds to Pathfinder. This handoff honours that separation; no Researcher certification of own draft.

Follow-up QUEUED6769376567 incorporated as historical context: intermediate cells version still lost; conjecture became recorded identity; full inverse-rank construction could erase savings. Missing pasted values resolved from journal. Relay raster images not used as quantitative evidence.

## Coordinator publication checklist

Port post.md to proposed path on a docs branch from current master; harmonize SVG classes with live CSS and supply hero if house template requires it. Add matching index card, navigation, llmstxt and social card. Run check_prose, docs-build, check_render, look_pages at both widths/themes, inspect every screenshot, and record exact integrated head. Create PR with this audit/goals/first-landings and honest render record. No performance release headline. Route independent published-post review to Pathfinder per LOG6769385980.
