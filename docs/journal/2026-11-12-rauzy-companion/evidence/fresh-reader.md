# Fresh-reader editorial review: Rauzy–Veech technical companion

Article: `/workspace/mibz2-blog/docs/site/blog/2026-11-12-rauzy-technical-companion.md`

Reader profile: technically engaged programmer comfortable with arrays, caches and discrete mathematics, with overview-level familiarity with BWT. This is a long technical companion; preserving its depth is appropriate.

Scope: Both passes read only the article, including SVG text and figure captions. No linked sources, repository instructions, author plan or other repository files were inspected. This is an editorial fresh-reader pass, not independent B5 review or full mathematical certification. The first-pass record below faithfully summarizes the findings already delivered in conversation; the second pass reflects a rereading of the revised article. No article or repository files were edited.

## First pass

### Reading route and section purposes

I would begin with the executive summary and then read sections 1–4 continuously. The opening links locate the companion, but I would not follow them before reading. Sections 1–3 establish conventional inversion, interval induction and reconstruction obligations. Sections 4–5 explain Cells and its physical costs. Section 6 distinguishes delimiter first returns. Sections 7–8 discuss representation maintenance, grammar reuse and allocation. Sections 9–10 organize the evidence and diagnostic. Section 11 and the assessment identify remaining research questions and the immediate experimental priority.

The article's depth is appropriate. The worked BA/AB figure substantially improves the mathematical explanation. The first three sections have no serious comprehension barrier once word orientation is understood; making that orientation explicit earlier would smooth the reading.

### Comprehension checks

1. The BA/AB example is understandable and internally consistent. Initially B occupies [0,3) and maps by +5 to [5,8); A occupies [3,8) and maps by -3 to [0,5). Retaining [0,5) makes every B point take two old returns, for net translation +2 into [2,5). The retained A part [3,5) maps immediately by -3 into [0,2). New top order is B3,A2 and bottom order A2,B3. LF visits B then A, but the stored forward-text word is w_A w_B, with return time |w_B| + |w_A|.
2. Cells and delimiter first returns share first-return invariants but differ in sections, representations and costs. Cells repeatedly cuts a terminal part from a prefix and stores permanent completed pieces. Delimiter induction retains a generally disconnected set of rows and propagates/splits fragments to construct a residual permutation and return words.
3. Fewer counted induction or traversal operations do not imply the same reduction in CPU time or memory. Construction, comparisons, grammar expansion, allocation, output and downstream work remain. The article makes this explicit and reports a failed delimiter memory budget.
4. Equal contents permit shared grammar definitions or expanded bytes; they do not permit substituting one occurrence's destination or origin marker for another's. Section 3 states this clearly. Repeating the boundary in section 8 would help where reuse is proposed.
5. Matching off/on categories would not establish causal savings in the hooks-off build. The initial article discussed instrumentation effects but did not define category matching, so this particular inference was insufficiently explained from the text alone. My initial suggested wording interpreted matching as corresponding work categories; the revised article correctly makes the intended subject performance conclusion categories instead.

### Stuck points and minimal proposed fixes

| Location | Class | First-pass finding and minimal fix |
|---|---|---|
| Section 1 to 2 | Step | LF and the forward walker run in opposite directions. Add an early sentence that induction follows LF while stored words reverse the visitation order. |
| Section 2.2 intervals | Symbol | Specify that [0,m) contains integer row coordinates, not a continuous state space. |
| Section 2.2 return words | Step | "Emitted" versus "stored in forward text order" becomes clear only in 2.3. State the reversal convention at the first definition. |
| Right-end figure caption | Symbol | "Annotation A1" is undefined locally. Replace with a self-contained description. |
| Section 2.3 update table | Step | "Change top/bottom order" does not specify the reordering, and the two-letter figure cannot demonstrate a nontrivial one. Add the insertion rules or explicitly delegate them. |
| Section 2.4 S/r | Symbol | Define S as elementary induction steps before using it. |
| Section 2.4 adversarial example | Statement | "Two-run input such as (ab)^k a" sounds like a claim about the displayed text's runs. Specify that its BWT has two runs. |
| Section 2.4 O(n) | Statement | "Require O(n)" does not express a required linear cost. If supported, say "can take linearly many steps". |
| Section 3 Lyndon decomposition | Beyond | Undefined but explicitly nonessential. A brief gloss would help; a tutorial is unnecessary. |
| Section 3 marker propagation | Chain | The invariant is stated, but a sample marker-offset update under concatenation would help bridge it to implementation. Distinguish expansion offset from orig_ptr. |
| Section 3 marked-spine/F01 | Symbol | Replace or gloss internal vocabulary as the grammar path carrying the marker and independent writer-safety review. |
| Section 4 closure | Step | Define component closure and relate it to matching terminal identities. |
| Section 4 marked-letter floor | Symbol | Explain the named admission condition. |
| Section 6 token ordering | Beyond | The correspondence is correctly qualified but its condition is not stated. Name the condition or make its construction-specific scope explicit. |
| Section 7 O(bD+S) | Symbol | S is undefined here and previously means induction steps. Define the new meaning, preferably under another symbol. |
| Section 10 categories | Chain | Explain why agreement of off/on conclusions does not establish equal effects or causal savings in the uninstrumented build. |

No internal contradiction was found in the worked exchange, grammar expansion, reverse-itinerary convention or local contraction formula. The toy grammar's arbitrary output length is explicitly separated from an actual LF-cycle claim. The ambiguous two-run wording and O(n) phrasing were exposition issues, not independently established mathematical errors. The initial update table and closure description did not suffice to implement all cases from the article alone.

### Style and historical status

Local wording suggestions included replacing "more radical", "This is not merely theoretical", "subtle but important", "genuinely a different design choice", "physical-computing design" and "decide intelligently" with direct descriptions. "Currently unpriced well enough" should be "not yet priced well enough". The final "not another proof of correctness" is stronger than necessary given the writer-correctness history; an immediate-priority sentence would be more precise. None of these requires shortening the technical treatment.

The editorial update and historical snapshot should be distinguished explicitly. The paragraph about checks pending at the last check needed an opening such as "At the source report's cutoff".

## Second pass: revised article

### Direct answers from the revised article

**1. What does the BA/AB example do on [0,5), including words?**

The original exchange on [0,8) has top B3,A5 and bottom A5,B3. Its maps are x -> x+5 for x in [0,3), and x -> x-3 for x in [3,8). The inducing section removes [5,8) from the retained set, not from the represented trajectory.

For each x in [0,3), the first old return goes to x+5 in the removed suffix; the second returns through A to x+2. Thus the induced B branch maps [0,3) to [2,5). Its LF itinerary is B then A, and its stored forward-text word is w_A w_B. Its time is |w_B|+|w_A|.

For x in [3,5), the first old return already lands in [0,2), so the induced A branch remains x -> x-3, with the unchanged word w_A and time |w_A|. The new top order is B3,A2; the bottom order is A2,B3. For example, 1 -> 6 -> 3 becomes a single induced transition 1 -> 3.

**2. Can equal word contents substitute for exact routing or a marked occurrence?**

No. Section 2.3 requires destinations, times, bytes and marked occurrences to be preserved together. Section 3 explicitly says equal words can have different return destinations and the marker identifies a particular occurrence or trajectory. Equal content can support shared word storage or expansion; it cannot replace per-occurrence routing or origin information. Section 8 is readable under that established invariant, even though it does not repeat the qualification.

**3. Do matching off/on performance conclusion categories establish causal off-build savings?**

No. The revision says this directly and gives a concrete example: Cells/baseline ratios 0.9 with hooks off and 0.45 with hooks on could both fall into a "faster" category while representing different effects. Stage attribution describes the instrumented path; agreement in broad conclusion categories does not transfer those phase savings causally to the uninstrumented path. The exact frozen interval classification rule is not defined here, but the example's logical point is understandable without it.

### Resolution of earlier concerns

The important local notation and transition gaps are resolved:

- Section 2.1 now defines intervals as integer rows.
- The figure caption no longer depends on unexplained "annotation A1".
- Section 2.3 gives both strict-inequality ordering moves, the equal-length placement, and the matching-label closure criterion. These bridge the summary table to the later Cells description.
- Section 2.4 defines S, distinguishes the adversarial text from its two-run BWT, and says "linearly many" steps.
- Section 4 defines the marked-letter floor's quantity.
- Section 7 defines S as distinct interior cuts and explicitly disambiguates it from section 2.4's induction steps.
- Section 10 now explicitly distinguishes matching performance conclusion categories from equal effects and causal hooks-off attribution.
- The historical paragraph now opens with an explicit historical-cutoff qualifier.

Not every optional suggestion was adopted. The reversal convention still arrives in section 2.3 rather than in the first definition of w_alpha; the marker-update mechanism remains summarized rather than worked through; "Lyndon-related", "marked-spine" and "F01" still lack local definitions; token ordering's precise sufficient conditions remain outside the article. These are nonblocking for the assigned reader and scope. They would matter to someone trying to reproduce the entire implementation or proof from this article alone, which is not how I read the companion.

The expression "linearly many (O(n))" is now understandable in prose, though Theta(n) would be the precise notation for an asserted linear order of growth if that is the proved result. This is a minor notation polish, not a remaining serious confusion.

The marked-letter floor parenthesis defines the number but could still say what it is compared against or why it causes refusal. That omission does not obscure the bounded-admission argument.

The historical paragraph mixes a past-tense introduction with "still fails", "remains red" and "exists separately". Its scope is now clear, but consistently using past tense would be cleaner. The first-pass mannered-language suggestions also largely remain optional polish.

### Second-pass disposition

No remaining serious confusion prevents understanding the mathematical replacement, the worked induction step, the occurrence/routing invariant, or the evidence limits. I found no new internal mathematical inconsistency in this rereading. The article retains the necessary distinction between counted reductions, physical costs and measured speedup. Its depth should remain intact.

This disposition is an editorial comprehension finding from the article alone, not certification of linked claims, source correspondence, implementation correctness, experimental results or benchmark methodology.
