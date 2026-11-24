# Coder A's conventions (MONODROMY), applied identically to every batch

Sources allowed: cases.tsv, cases.md, facts.tsv on research/retrospective-coding at 38c50de8 (copies in this
directory), and repository documents ONLY at a commit dated before the case's commit_to_build
(git -C /home/user/mibz2 log -1 --before=<commit_to_build> --format=%H origin/master, then git show <sha>:<path>).
Never: #145 verdict LOGs or any comment after commit_to_build, current master documents, evidence bundles' verdict
or report files dated after commit_to_build, any research/retrospective-* branch other than research/retrospective-coding,
coder-B.tsv. Judge every question as of commit_to_build.

Columns (tab-separated, no tabs/newlines inside fields; join multiple items with " ; "):
id gap_I gap_M gap_P gaps_examined contradiction contradiction_quote check1 check2 check3 check4 check5 check6 check7 why confidence notes

- gap_I / gap_M / gap_P (yes/no/unclear): compare where the expected gain's evidence came from (claimed_gain_source,
  must_hold, premises, or a pre-build document) with the case's deciding inputs / machine / code path.
  I: synthetic, toy, one block, other corpus = yes. M: other machine, core or thread count, or contention regime = yes.
  P: idealised algorithm, harness, research arm, literature algorithm, or the base moved between evidence and the
  run (several bundles with master moving; a prediction from an older base) = yes.
  "no" only when the evidence was on the deciding inputs/machine/path. "unclear" when no evidence or premise can be
  found pre-build. An idea taken from a paper or another project: its evidence is that source's (usually I, M, P yes).
- gaps_examined: n/a if no gap is yes; yes only if the pre-build sources name every yes-gap AND check or explicitly bet
  on it; otherwise no.
- contradiction (yes/no): yes only if a fact dated before commit_to_build (facts.tsv line, or a document at its
  pre-build version) contradicts a premise. Quote both, with links, in contradiction_quote. A premise may be quoted from
  a pre-build source the extractor missed, if cited. Supporting or neutral facts do not count.
- check1: fired iff some gap = yes and gaps_examined = no; not iff no gap is yes (all no) or gaps_examined = yes;
  unknown iff no gap is yes but some are unclear.
- check2: fired iff contradiction = yes; else not.
- check3: fired iff counterfactual_real_block_le10min lists a bundle through the shipped path on real blocks whose
  (pre-build) result contradicts the premise; not iff such a bundle exists and is consistent with the premise;
  unknown iff none is listed (C2: never invent a check).
- check4: not unless the case is split out of another change's measured gain; then fired iff no ablation of that
  gain is dated before commit_to_build, else not.
- check5: not if the premise is not about latency, memory, cache or footprint (record 95's tests don't apply);
  if it is: fired iff data in hand before the build (an m-slope, a latency-free floor, or a footprint against cache)
  puts the claimed gain outside its interval; not iff such data supports it; unknown iff no such data was in hand.
- check6: fired iff a proved bound dated before commit_to_build covers the idea's class (FRONTIER.md obstructions,
  PROOF-PROGRAMME, a proved statement in a journal or MONODROMY record); else not.
- check7 (routed through review): fired = the idea was NOT passed through the Researcher's prior-art check or the
  Optimiser's pricing before commit_to_build (review_routing says unrecorded, or after); not = it was, before;
  unknown = pre-team era (commit_to_build before 2026-11-06, when the roles did not exist); say so in why.
- why: one item per fired check: "checkN: <trigger> — <the fact, with link>".
- confidence: high/medium/low.
- notes: tag disputes; any outcome knowledge MONODROMY had before coding (state the source, e.g. "outcome partly known:
  PROOF-PROGRAMME §5.1 dispatch on master"); missing facts.

Known contamination to flag in notes (MONODROMY read these before coding):
- parscan-stream (#770): brief §2.3 states its verdict. tiled-zero-fill (#785): brief §2.3 states its verdict.
- r5-stack (#734): PROOF-PROGRAMME §5.1(b) states its i9 result. two-ended-emit: master's open.rs (emit_bidirectional).
- open-diet: master's open.rs comments ("the memory diet") and record 88. dense-admission: Cap::dense and the width
  certificate on master (PROOF-PROGRAMME §5.1, §5.4: "shipped (#703)").
- crossover-10, lazy-at-10, lazy-one-a-core, cooperative-decoder, three-team-policy, pipelined-teams,
  p87-in-place, run-fed-stage, cells-writer-once, window-four, open-lanes-r4 (LIMIT 16 on master), open-r4:
  master's dispatch and kernels as read in PROOF-PROGRAMME §5.1 and src/open.rs may reveal or hint the outcome.
- open-routes: derived from Neon's teamwork handoff, whose theorems MONODROMY audited (record 90) — note it.
