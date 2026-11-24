# The retrospective's case list and dated facts (phase 2)

Phase 2 of `docs/journal/2026-11-24-retrospective-proposal.md` §4a, as corrected by MONODROMY 97
(#145 comment 7127184296, corrections C1–C6). It extracts facts and classifies nothing.

## The files

| file | for | what it holds |
|---|---|---|
| `cases.tsv` | coders | one row per idea, the columns below; no verdicts |
| `cases.md` | coders | the same, one section per idea, with links and quotes |
| `facts.tsv` | coders | every same-quantity fact line, one row each: case id, date, source, link, the line |
| `outcomes.sealed.tsv` (**not on this branch**: on `research/retrospective-sealed`) | phase 5 only | outcome category, verdict link and date, the verdict's words, and notes that would reveal an outcome. **Coders don't open it.** |
| `outcomes.sealed.case-definitions.py.txt` (**not on this branch**) | phase 5 only | the hand-entered case definitions, including each outcome. Sealed for the same reason |
| `extract.py.txt` | anyone | the script that produced the other files from the cached sources |

## The case set

- **Admitted (C3):** an idea built and measured on the machine whose measurement decided it, with a
  recorded verdict, from 27 October 2026 on. The deciding machine is recorded per case: CI's paired
  bench, the i9-10900K, or a native run where the verdict was made on one (the Open research PRs of
  20–22 November, recorded in the fidelity ledger, `docs/team/research-charter/CONTEXT.md` §5a).
- **Outcome categories (sealed):** negative, null, positive but not adopted, adopted.
- **Counts:** 113 cases: 66 non-adoptions (43 negative, 18 null, 5 positive but not adopted) and 47
  adoptions.
- **Dedupe rules:**
  - one idea is one case, however many PRs, registrations, runs and journal entries it spans (for
    example, the service core is #251, #258, #299, #302 and #316; the Open port is #590, #604,
    #631, #638 and #630);
  - a re-run, a second attempt or a revised version of the same construction stays in its case
    (the sealed notes list them);
  - a split-out, a different construction on the same stage, or a candidate with its own rule and
    run is its own case (for example #785, split out of #770; #25, #118 and #207 on the MTF head);
  - a bench arm's PR and the production PR it fed are one case (#465 with #462; #477 with #481).

### Excluded, and why

- **Not built, or not measured on the deciding machine:** LazyTunnel and BSAC (#725: a gate measured
  the premise first; the port was stopped unpublished); Open's pipelined emit (#656: stopped before
  timing); D4-delta (#331) and the staged cooperative walk (#334), decided on counts; the
  partitioned walk and segment cost (untimed); split condition variables and the sibling front end
  (nothing built); the lazy-overhead and lazy-letters lean builds (counts, before timing; the
  lazy-letters idea is a case through #447 and #452); MONODROMY's records closed as priced
  (compaction rounds, replay and team, the tower, the locality bound, the feathered screen, the
  cross-block loops); the local-factor census; the origami lift; the run network.
- **Diagnostics that tested no candidate:** knee, imc-ten, miss-cost, cold-start, two-modes,
  telemetry-check, share-nothing pricing, cells-dose, cells-imc, cells-phase, phase-gates (never
  ran), split-051, lazy-occupancy, the walker ladder (#131), the four-thread ceiling, the release
  scaling runs. Where a diagnostic fed a case, it is listed in that case's sources (par-scan in
  parscan-entry, opend-sweep and open-return in open-kernel-port, stop-walk-counters in
  stop-and-walk, w12-stores in w12-rules).
- **Research measured in the investigators' own environments, with no adoption candidate:** Astra
  A–D, the Pro investigations, Cells studies 7, live reuse, owner schedules, mixed steps, the
  endpoint sweep, the deep-research notes, and the Neon teamwork round's natively killed mechanisms
  (`2026-11-22-neon-teamwork-paradigm.md`). Astra E is a case, because it ran on the i9.
- **No verdict:** research PRs closed as stale with no decision (#84–#95's cooperative
  experiments other than the decoder itself, #107, the Codex research PRs #369–#372).

## The columns

- **commit_to_build:** the earliest of: a commission comment (entered by hand where found: T²,
  #785, the team-threshold rule), a registration's first commit, a PR's opening, and the PR's
  earliest commit. A PR's earliest commit is used only if it is within 12 hours of the PR opening
  and is not also listed on another PR; otherwise it may be a base branch's commit, and the notes
  say so. A case with no PR uses its journal entry's first commit, which may be its result.
- **origin:** as the record states it (PR body, LOG, queue entry), with the citation. Before
  6 November one Claude session did all the work; it was later named the Coordinator. Those cases
  say "pre-team era".
- **claimed_gain_source, must_hold:** sentences quoted from the pre-build sources (commission,
  registration at its first commit, first commit message, PR body) that carry a measurement, and the
  queue entry's description. Selected by pattern, with sentences that state an outcome dropped.
  Coders mark the I/M/P gaps from them (C3).
- **cost:**
  - *i9:* minutes per evidence bundle, from `started.txt` to `finished.txt` where the bundle has
    them, otherwise from the bundle's start (its name) to its commit on `evidence` (labelled);
  - *CI:* Benchmark workflow runs on the case's PR branches;
  - *calendar days:* commit_to_build to the verdict's date;
  - *agent runs and tokens:* quoted where a comment naming the PR records tokens, else
    "unrecorded". No estimates.
- **premises:** up to six sentences from the same pre-build sources that state an expectation,
  mechanism or price, outcome sentences dropped. **PR bodies are their current text:** GitHub keeps
  no history of a body's edits, so a body may have been edited after the build. Each premise is
  dated by its source.
- **same_quantity_facts (C1):** searched on the premise's own terms (`kw` in the case definitions,
  shown in `cases.md`), never on the outcome. A fact is a line that matches the terms and carries a
  number with a unit or a ratio, in:
  - every markdown file under `docs/journal/` and `docs/team/research-charter/` on master, and
    MONODROMY's records on #497's branch, each **at its last version before commit_to_build**;
  - every comment on #145 and the inboxes #160, #161, #162, #222, #314, #346 and #363 posted before
    commit_to_build.
  All of them are in `facts.tsv`, supporting or not; `cases.md` shows the latest line from each of
  the ten latest documents. They are line hits: an argument stated without a number on its line is
  missed, though its document is usually listed through another line (for example, the toy-transfer
  entry's "about ten bytes are appended per retained step" for T²). Coders read the document.
- **counterfactual_real_block_le10min (C2):** i9 evidence bundles of at most 10 minutes, started
  before commit_to_build, whose recipe name or queue description matches the premise terms. None is
  invented. Whether one went through the shipped path on real blocks is for the coders.
- **review_routing (check 7):** the earliest comment from the Researcher and from the Optimiser that
  names one of the case's PRs or recipes, and whether it came before commit_to_build, before the
  first i9 bundle, or after. "Unrecorded" means no such comment.
- **monodromy_involved:** cited in the case's own documents (PR bodies, journals, verdict), or a
  record on #497's branch names its PRs or recipes (record 97 itself excluded). A citation can be
  prior evidence rather than involvement: the field says which documents, and MONODROMY's own
  exclusions in 97 stand beside it.
- **coordinator_involved:** pre-team era; the origin names the Coordinator; or a Coordinator
  QUEUED, CHECKPOINT or PROPOSAL naming the case at or before commit_to_build. Every merge or close
  after 6 November is formally the Coordinator's decision; that alone is not counted.
- **notes:** neutral facts: several i9 bundles (check whether master moved between them, C3's "the
  base moved"), and PR commits not used for commit_to_build.

## Sources searched

- `docs/journal/` (245 entries and their subfolders), `docs/journal/monodromy/` on master and on
  #497's branch (records 1–97), `docs/team/research-charter/` (CONTEXT.md's fidelity ledger,
  FRONTIER.md, PROOF-PROGRAMME.md), `docs/journal/release-attribution.md`, `docs/team/DELIVERED.md`,
  `CHANGELOG.md`.
- #145 (2,074 comments, 4–24 November) and the seven inboxes.
- All 781 PRs: bodies, first commits, the comments of the 111 closed or open idea PRs, and the
  Benchmark workflow's runs on their branches.
- `bench/workstation-queue.txt` and its history (94 recipes ever queued).
- The `evidence` branch: 144 bundles, opened for their start and finish times.
- The Observer's `observatory/intention` (90 rebuilds, 6–19 November): read for what was intended;
  it ends on 19 November, so it covers no case after that.

## Limits

- **Hand-entered fields** (sources, origin, search terms, verdict reference, outcome) are one
  extractor's reading, made with the outcomes known. The search terms in particular were chosen by
  someone who knew the outcome; they are written from the premise's own vocabulary, and they are
  printed so that a coder can rerun the search with others.
- **#145 starts on 4 November.** Before it, the record is the journal, the PRs and the issues; the
  issues (#5–#9, #39, #42, #65, #77–#81) were not searched for facts.
- **Pre-6-November i9 runs** are not on `evidence` (it began on 6 November), so their i9 minutes
  are unrecorded.
- **Tokens** are recorded almost nowhere; the column says so.
- **Not read:** the research branches' own journals where they were never merged (for example
  `research/for14-records`, `research/general-plan-impl`); the bench branches' registrations are
  read only through their PRs and queue entries; the Observer's account was read, not searched for
  facts.

## Rebuilding

The caches (`c145.json`, the inboxes, `prs.json`, `prcomments.json`, `prmeta.json`, the queue
history and the bundles' times) were fetched with `gh api` on 24 November 2026. With them in a
directory beside the definitions file copied to `cases_def.py`, and every branch fetched (master,
`evidence`, PR #497's head as `origin/pr497`):

```sh
cp extract.py.txt extract.py
python3 -I extract.py CACHE_DIR REPO_DIR docs/journal/2026-11-24-retrospective
```

## Sealing (added by the Coordinator)

The extractor's branch `research/retrospective-cases` had the sealed files beside the coder files, and in its history. This branch, `research/retrospective-coding`, was made fresh from master with the coder files only: its history never contained an outcome. The sealed files are on `research/retrospective-sealed`. **Coders work from this branch only, and fetch no other `research/retrospective-*` branch.** The Coordinator moved the sealed files without opening them.
