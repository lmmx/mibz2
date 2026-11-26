# Review of the `the-frontier` draft: a simulated reader, and every number against its source

*26 November 2026. A cold review of [`the-frontier-draft.md`](the-frontier-draft.md) (at `daf0224`, #950), by an agent other than its drafter, under the board's `review` stage (`frontier/README.md`, "The blog board"). Each number was checked against the primary file it rests on, not against the outline's fact ledger. The four external pages were opened and read.*

**Verdict: revise, then ready.** The argument holds, the structure answers every question the outline set, and nearly all of the ~90 figures check out. One finding is a **factual error that a newer result introduced** (F1). Two are **overstatements of what the machinery enforces** (F2, F3). The rest are small. None needs a new section.

## Findings, in order of weight

**F1. Delivery's 48 ms is no longer true. Must fix.**
- *Where:* §2's table ("Serial delivery copy-out (`D`): 47.3–49.3 ms flat") and §5 ("`D` serial delivery (48 ms flat)").
- *Why:* #952 (`docs/journal/2026-11-26-explore/cold-store-and-rle1-llc.md`) showed the 48 ms was a cold-store artifact of the floor run. `frontier/surface.tsv`'s D row now reads **29.1 ms at 1 worker and 34.0 ms at 8**: L3-hot from 1 to 8 workers, spilling L3 at 9–10 (43.7–77.9 ms at 10 workers, `READY_BYTES` = 12 MiB against a 20 MiB L3).
- *Fix:* give the current D row, and say in a clause that a cold review corrected the floor's 48 ms on the day. That is itself an instance of the post's argument: a cold reread of primary data overturning a summary. The figure script draws no D value, so no figure changes.

**F2. The invariants table says "mechanical enforcement", but protection is not yet binding.**
- *Where:* §3's invariants table ("Ten protected paths never self-merged") and the stress-test paragraph in §6.
- *Why:* the stress test's scenario 8 (`frontier/deltas/2026-11-26-stress-test-frontier.md`) found that `check` only *warns* on protected paths. It reads `protected` from the PR's own config, CI's shallow checkout drops the warning, and `master` has no branch protection. The hardening delta says the same ("The protected list warns but doesn't block").
- *Fix:* retitle the column "how the files keep it" (the README's own wording). Add one sentence saying that conservation of authority is so far a written rule with a warning. Binding protection waits on the `branch-protection` claim, which needs the maintainer.

**F3. §6 understates two of the four stress-test failures.**
- *Where:* "stale offline `verify` refs … shallow CI checkouts hiding delta edits".
- *Why:* scenario 3's worse finding is that `renew` and `release` **exit 0 when their compare-and-swap push fails**. Scenario 6's core finding is that `check` **misses edits to a claim's `stop_test`**, deletions from `dead-ends.tsv` and `bench/*.frozen`. That is the "gate changed after data" case itself, not only CI's shallow checkout.
- *Fix:* name those two, since they bear directly on the post's claims about locks and frozen rules.
- *Also:* step 3 of the walkthrough in §3 says the session "re-reads the primary evidence listed in `premises`". That is the agent's duty; scenario 4 found that `frontier.py` never inspects `premises`. Say so in a clause.

**F4. Paradigm 3's λ range belongs to one task.**
- *Where:* "Across Paradigm 3's benchmarks … ($\lambda \approx 0.6\text{–}0.9$)".
- *Why:* the page estimates 0.6–0.9 "on one unsaturated task", and calls λ poorly determined on the other three. The 6× and ~7% figures are general ("Teams of 8 typically spent the same budget about 6 times sooner … and scored about 7% lower").
- *Fix:* "on the one task where it could be estimated, λ ≈ 0.6–0.9".

**F5. The lexicon count doesn't add up.**
- *Where:* §4 says 19 commands in four groups, with "eight" in the second group. That table has seven rows, and the third table merges `admit` with `drop/block/bump`, so the tables hold 18.
- *Why:* `frontier/README.md`'s lexicon has 19 rows (`admit` is its own row).
- *Fix:* say "seven" for the second group and split `admit` out of the third group's last row (seven rows there). That gives 3 + 7 + 7 + 2 = 19.

**F6. The paired-oracle gloss misnames √(SC/C).**
- *Where:* "compares it directly against the unmasked ceiling $\sqrt{\text{SC}/C}$".
- *Why:* in the pooled-scan registration (K1) and results, √(SC/C) is **the bar**, the point at which a candidate captures half of the scan's room behind construction on the log scale. It is not the ceiling; SC/C is the ceiling.
- *Fix:* "against $\text{SC}/C$, the ceiling, and kills it if PC/C cannot be shown to reach $\sqrt{\text{SC}/C}$, half that room on the log scale".

**F7. The Deleuze and Guattari quotation needs its own source.**
- *Where:* "never believe that a smooth space will suffice to save us".
- *Why:* the linked Wikipedia page doesn't contain it. It is the book's closing line of plateau 14 ("1440: The Smooth and the Striated").
- *Fix:* cite the plateau in the text (and the page, if a translation is at hand), and quote it exactly, in quotation marks.

**F8. Small exactness fixes.**
- *"eight role threads":* the 2,748 total is #145 **plus** seven role inboxes.
- *"Four dormant role inboxes posted zero comments":* the cross-reading counts four *authors* (`optimizer`, `guardian`, `observer`, `codex`) with zero posts on #145 over 22–26 November. Say "four role identities posted nothing", which is what was measured.
- *"our style-and-mode specification reached 702 lines":* the 702 (701 by `wc -l` today) is `MODE.RATIONALE.md`, the rationale. `MODE.md` itself is 113 lines. Say "its rationale reached 702 lines".
- *"10 entries beside 10 live targets":* `dead-ends.tsv` now has 13 rows. Date the count ("at launch") or drop it.
- *"a 16-minute follow-up … (648 decodes, PR #921)":* #921 built the zero-copy oracles; the run's recipe is #917 and the results are #931. Cite #931 for the result.
- *"a 500-line … script":* 499 lines; fine as written.

## Checked and correct

- **Paradigm 3:** authors and date; 1,100+ runs and 80+ configurations; 8M and 32M budgets; ~6× and ~7%; `git-to-zig` with 743 test scripts; 93.2 vs 83.7 and 32.0 vs 26.6; restarted solo 97.0 and 41.9; 16 and 32 at 33.3% and 23.2%; free chat −17 to +53.
- **Ord:** title, date, the $N^\lambda$ model, and λ = 1 as perfectly parallel. The draft's $\lambda = \ln(X/b)/\ln N$ follows from $X = bN^\lambda$.
- **Grassé:** 1959, *stigma* and *ergon*, termites.
- **#145:** 2,269 comments over 22 days, and 2,748 with the inboxes. Over 22–26 November, 269 comments, of which 124 (46.1%) were the coordinating agent's (107 LOG, 13 QUEUED, 4 CORRECTION), 60 (22.3%) came from the workstation and 85 (31.6%) were others'.
- **The record:** `DELIVERED.md`'s two rows of 10 November; `CLAUDE.md` at 225 lines at the time.
- **Translation defects:** the four of 26 November (#916 at 09:31, the withheld tests, #920→#923/#927, #926's trailers), and the six malformed headers. The lossy-compression examples (lower bound as certificate, §70; 259 ms called copying, §91; 20 requested threads read as 20 active rather than 7 lazy, §61) all match `rhizomatic-swarms-cross-reading.md`'s Part 2.
- **The matrix:** 2,492 decodes in 4 h 15 min; C −42.1% at 1 worker (1,306 → 756 ms); S −3.2% and C −5.6% alone at 8 (195 → 189 and 184 ms); S+C −36% (195 → 125 ms).
- **The floor:** 648 decodes in 16 minutes; FCMRD 179.1 ms, 92% of the wall; E_consume 298 ms (22% of 1,343) and 37 ms; plumbing 5.2–5.7 ms (2.7%).
- **Dead ends:** Rauzy 1.41% below its 3% gate (#933); network calculus 3.2–3.5× (#935); uiCA −28.8% with the wrong sign (#935); pooled-scan capture 0.52 and 0.48 (#942).
- **The frontier:** `max_live_holds = 3`; `max_open_claims = 12`; ten protected paths; `check` capped at five checks; `CLAUDE.md` near 120 lines; `brief` shows six deltas (`frontier.py:380`); the hardening race test (#946's delta); the stress test's 4 PASS / 4 FAIL and three flags (#948).
- **The three-arm design** against `rhizomatic-swarms-cross-reading.md` §3.4: 6M an arm plus a 2M anchor; the arms' splits (1.5/3.5/1.0M; 3 × 2M; 2.0 + 2.0 + 1.2 + 0.8M); the target, its exactness gates and the 1.10× RSS bound; and the adopt and fall-back rules (≥ 50% tax cut, at most one relay, ≥ 2 pp or within 2 pp with 2× and a caught defect; fall back at > 3 pp). The "λ > 1" gloss on the 2 pp bar is the registration's own wording.

## Simulated reader: the outline's questions

A reader with only the post can answer every question the outline set:

| question | answered by | verdict |
|---|---|---|
| 1, why drop a coordinator if CI worked | §1, the split between routing and verifying | yes |
| 2, how parallel sessions avoid colliding | §3: compare-and-swap holds, `owned_paths`, `mechanisms`, `verify` | yes |
| 3, why local claim edits and greedy picking fail | §3 (the #945 race) and §5 (monoculture) | yes |
| 4, why one word suffices, and when to ask | §4 | yes |
| 5 (transfer), a premise overturned mid-flight | §3's "local immune system" and §6's scenario 4 | mostly. The reader assembles the sequence, but the post never says it in one place. One sentence at the end of §3 would close it: delta, surface row, a flag scoped to the invalidated claim, and the unrelated claims continuing. |
| 6 (held out), what result abandons the flat mode | §6, test 2's fallback rule and what survives | yes |
| 7 (held out), why the zero-copy floor mattered to the team | §2's last paragraphs, on paired oracles decoupling S from C | yes |

**Register:** the prose is clear and well linked, but long. At "21 min read" the post is the longest on the blog. §3's two tables (the invariants and the hybrid-system mapping) and §4's four lexicon tables carry most of the length. A revise pass could cut the hybrid-system table to two sentences without losing an argument. That is a suggestion, not a finding.

## What the revise stage should do

1. Apply F1–F8 and the one sentence for question 5.
2. Rerun `check_prose.py` and `link_audit.py`.
3. Repin the Sources links from `d88951c` to a commit that includes #952, so that the D row's source is the one cited.
