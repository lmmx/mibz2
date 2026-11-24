# Coding instructions (phase 3 and phase 4)

The protocol is `docs/journal/2026-11-24-retrospective-proposal.md` on master, §3 as amended by
§3.7 (MONODROMY's six corrections, frozen). Read §3.3, §3.4 and §3.7 before coding. This sheet
only fixes the output format so that the coders' work can be compared.

**Blindness.** Code from `cases.tsv`, `cases.md` and `facts.tsv` on this branch. Do not look up
how any case turned out: no #145 verdict LOGs, no journal verdicts, no evidence bundles' verdict
files, no other `research/retrospective-*` branch. If you learn an outcome by accident, say so in
`notes` for that case. Judge every question as of the case's `commit_to_build` date.

**Recusal.** Coder A (MONODROMY) skips every case with `monodromy_involved` = yes, as its REVIEW
7127184296 lists (authored, modelled, predicted, scored or verified). Coder B codes every case. If
you think a tag is wrong, code the case and say so in `notes`; the Coordinator applies recusal.

**Output:** one TSV, `coder-A.tsv` or `coder-B.tsv`, one row per case coded, these columns:

| column | values | meaning |
|---|---|---|
| `id` | the case id | |
| `gap_I` | yes / no / unclear | the evidence for the expected gain came from inputs unlike the deciding inputs |
| `gap_M` | yes / no / unclear | … from a different machine, core count, thread count or contention regime |
| `gap_P` | yes / no / unclear | … from an idealised algorithm, a harness or a research arm rather than the shipped path, or the base moved between the evidence and the run |
| `gaps_examined` | yes / no / n/a | every gap present was named and either checked or explicitly bet on before the build (the positive definition of B) |
| `contradiction` | yes / no | an earlier dated fact in `facts.tsv` contradicts one of the idea's premises (C1(a)) |
| `contradiction_quote` | text | the premise and the fact, both quoted with their links, if yes |
| `check1` … `check7` | fired / not / unknown | whether each check of §3.7 C4 would have fired before the build, by its operational trigger. `unknown` where the record can't say (C2: never invent a check's result) |
| `why` | one line per fired check | the trigger and the fact that meets it |
| `confidence` | high / medium / low | |
| `notes` | text | tag disputes, accidental outcome knowledge, missing facts |

Commit the TSV to this branch under `docs/journal/2026-11-24-retrospective/`, in your own commit.
Coders don't read each other's file.
