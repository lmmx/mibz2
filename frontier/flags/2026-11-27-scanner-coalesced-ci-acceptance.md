flags: scanner-coalesced-u8-decode
invariant: conservation of evidence; PERFORMANCE-POLICY §2 rule 3
by: codex
status: open

**Evidence.** The #1034 squash, `5ed20c1d`, cites paired run 45921211004. The later ten-round run compares the merged PR head `782f4feb` to its base `f9848a4`. At four threads, `pattern` wall regressed 7.3% (95% CI +2.6% to +12.3%). The two primary gains at four threads were 1.1% for Wikidata and 1.6% for text. The 5–10% secondary-loss band requires a primary gain of at least twice the loss at that thread count, or 14.6%. [Exact-head benchmark](https://github.com/lmmx/mibz2/actions/runs/45947633032) · [Independent review](https://github.com/lmmx/mibz2/pull/1034#pullrequestreview-6084072589)

**Scope.** The assertion that the shipped scanner passed ordinary CI performance acceptance is unresolved. Preserve both results in decisions about retaining or releasing this change. The i9 recipe has its own claim and frozen keep/revert criteria. Its registered grid omits four threads, so a hardware verdict alone cannot adjudicate this CI comparison. The independent B5 review stands.

**What would resolve it.** Reconcile the exact-head result under the existing policy and record the disposition: revert the candidate, qualify a materially corrected implementation on a new paired comparison, or document applicable existing authority or evidence that resolves the four-thread trade. The policy's special CI rerun provision applies to a flagged primary regression.
