# Price fixed marker choices from their colored return words

For a union of syntax-aligned quote-return buckets, let D(C) be the distinct incoming raw spellings ending at destination component C. The exact lookup count of the [native interval constructor](../native/) is

```text
Q = Σ_C |{proper suffixes of Y : Y ∈ D(C)}|,
```

including the empty suffix for each nonempty component. Start a backward extension at C. Every original-rank lookup examines one unfinished state (C,u), with u the suffix already read. Its children are exactly the nonempty next-byte groups. Uniform section membership at a child follows from the [unequal-return cylinder theorem](../). A hit completes a spelling and stops before the next lookup. Conversely every proper suffix of a real return occurs as an unfinished state, including its empty suffix, since an earlier hit would shorten the return. A given sequence of fixed backward bytes from one component gives one interval; branch grouping combines all its occurrences. Unary contraction in this implementation still charges a physical lookup for each extended byte. Hence this is equality for native Q, without billing terminal states or the heap used for marker selection.

Per component, remove the first byte of every distinct spelling, reverse the rest, and sort lexically. Count one for the empty prefix and add each key's length minus its LCP with the preceding key. This computes Q without expanding a quadratic total of suffix bytes. For B total colored spelling bytes, W colored tags, h components and L summed adjacent LCP lengths, Q=B−W+h−L. In particular Q≤B−W+h. Every first-return occurrence has length at most its component's distinct-suffix count. For M residual rows covering N source bytes this also gives M·Q≥N. A marked raw output alone sees only its cycle. The cyclic corpus images used here cover their full byte blocks; their first-token occurrence histograms independently match the stored first-column buckets. An arbitrary multicycle column needs spellings from every cycle meeting the union.

The native source is pinned at `4dfdcb525c4f9aadb13d8d50163eb616b47e6e7e`. All 27 systematic admitted comparisons at k=1,4,8 reproduced the exact components, residual rows, colored tags, colored spelling bytes and paid lookup count. On frozen at k=8 the explicit colored dictionary has 105,145 bytes and 88,882 lookups. A separate explicit FM expansion over all 3,279 ternary last columns through length seven checked 8,755 unions, 34,757 lookups and 9,273 components, including disjoint cycles. [Native comparisons](verified.csv) · [Small check](small.txt)

The [precommit](PREREGISTRATION.md) at `c841ba4` fixed the eight token byte strings using only frozen frequencies, all 36 singletons/pairs and the decision thresholds. Every candidate was then priced from half-open raw text returns between successive selected occurrences, including the wrap through the block boundary. The next selected occurrence's component colors the incoming spelling. Source rank determines adjacency of selected bucket intervals. [Exact inputs](inputs.csv) · [All counts](fixed.csv)

Of the 35 alternatives, **none** achieved at least 5% fewer queries and rows without increasing spelling bytes on even one of the eight held-out admitted blocks. None Pareto-dominated the reference pointwise or in pooled exact counts at zero margin. The fixed quote-colon-quote reference pooled 954,664 queries, 196,332 rows and 1,408,097 explicit bytes. Pairing its first token with `},"` used .9405× queries and .9040× bytes but 1.2946× rows. The lowest-row singleton, `datatype"`, used .1292× rows with 3.1119× queries and 3.4241× bytes. This rejects promotion under the frozen rule. Other marker families and constructors may have different tradeoffs. These structural prices omit the costs of selecting the section, index setup, branch splits, merge and actual residual traversal; obtaining the decoded raw text or token exports would itself require substantial work inside a decoder. [Decisions](decision.csv) · [Summary](summary.txt)

From the research worktree root, reproduce the independent marked bytes and token dictionaries using the frozen bzip2 corpus, then run the structural screen. The exporter is archived at `31565705f292b3b1dc3d8200eb65cd880e238bc7`.

```sh
d=research/astra-e/formulations/f2-heaviest/union/selection
bash research/astra-e/formulations/f2c/export.sh /tmp/f2-selection-input
export PYTHONDONTWRITEBYTECODE=1
python3 "$d/freeze.py" /tmp/f2-selection-input/export | diff - "$d/frozen.json"
python3 "$d/small.py" | diff - "$d/small.txt"
mkdir -p /tmp/f2-selection
python3 "$d/workload.py" /tmp/f2-selection-input/export /tmp/f2-selection
python3 "$d/decide.py" /tmp/f2-selection | diff - "$d/summary.txt"
for f in fixed.csv verified.csv inputs.csv decision.csv; do diff "$d/$f" "/tmp/f2-selection/$f"; done
```
