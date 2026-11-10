# Performance trade-off policy

*Set by Louis on 10 November 2026, in the Coordinator's session. It is his rule (RULES D), and it applies from this date forward.*

> "Correctness, memory safety, and reproducible evidence remain mandatory. Perfect performance across every workload, complete causal explanations, and exhaustive experimental certainty do not."
>
> "Optimise for useful speedups merged per unit of engineering effort. Every further investigation needs a specific, near-term integration decision. Stop allowing process to become the product."

## 1. Never negotiable

A change merges or becomes a default only with all of these:

- **Correctness:**
  - byte-identical output on every bench corpus;
  - its equivalence and differential tests pass;
  - corrupt input gives the same error as before.
- **Memory safety:** new `unsafe` code meets B5. That means a written safety argument; Miri, differential, fuzz and mutation testing; and an adversarial review by an agent other than the implementer.
- **Reproducible evidence:** the figures come from CI's paired bench, or from a workstation run under C5, archived with its digest.
- **Green CI** on the exact head merged.

## 2. Performance: net useful gain, weighted to what the decoder is for

The decoder exists mainly for Wikidata dumps and text-like data.

| class | corpora |
|---|---|
| **primary** | `wikidata`, `text` |
| **secondary** | `records`, `random`, `pattern`, `onebyte`, `runs` |

A performance change is **accepted** on its CI paired bench when all of these hold:

1. **A primary gain:** at least one primary corpus is faster in wall time beyond noise (its 95% interval excludes zero) at one or more thread counts.
2. **No primary loss:** no primary corpus is slower beyond noise at any thread count.
3. **Bounded secondary losses:**
   - **Up to 5% slower** (point estimate, wall time) at any thread count: accepted, and listed in the PR.
   - **More than 5%, up to 10%:** accepted only if the PR shows the primary gain at that thread count is at least twice the loss. It must also say why the workload matters less.
   - **More than 10%:** needs Louis's sign-off.
4. **Memory:** peak RSS grows by no more than 10% on any corpus, or the PR justifies the growth for the target machine.

A change built for a specific machine, gated on its hardware, is judged on that machine (C5) for its gain. On every other machine it must show that it stays inactive. A deterministic check that the route is not taken, plus a neutral CI bench, settles that. Statistical non-inferiority is not required where the code path is not taken.

**One bench run decides.** A regression flagged in a secondary corpus is weighed under rule 3, not re-run to settle it. Only a flagged primary regression gets one re-run; if it shows again, it stands.

## 3. Statistical tests

A test used to qualify a change must be one the Researcher and the Guardian agree is sound for its claim. When a test is shown unsound, it is replaced by a simpler sound one. It is never just dropped, and never kept on trust.

Prefer designs that are small and exact: paired runs, sign tests, a deterministic route check. Avoid large replicated programmes.

## 4. Investigations

Every experiment names the integration decision it will settle, and when. An instruction-count result is followed promptly by a wall-clock measurement. If the change is faster, it moves towards integration; if not, it is dropped.

Complete causal explanations are welcome, but they are never a precondition for merging a measured gain.

## 5. History

- **Earlier verdicts stand as recorded.** #324's registered no-harm rule failed (`random` 1t +5.3%, `records` 1t +2.1% wall, run 3 of 42747668401), and that verdict is not relabelled.
- **#324 itself** is judged under this policy on its merged head's fresh CI bench.
- **Why the numbers are what they are:** the 5% and 10% tiers were chosen with #324's figures known, following Louis's statement that its trade-off is acceptable. They are not presented as blind.
