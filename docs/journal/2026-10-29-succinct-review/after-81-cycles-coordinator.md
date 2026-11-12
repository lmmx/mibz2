<!--
Provenance: the coordinator's response to the research chatbot's second multi-cycle programme
(after-81-cycles-research.md), relayed into the working session on 29 October 2026. The text
below is unedited, except that two interactive pieces it contained (a navigation-path diagram
and a build/walk slider) reached the session as their text alone, and are kept as that text.
-->

I think this second research cycle has reached a stronger conclusion than the first. The researcher has moved beyond designing increasingly elaborate compressed tables and identified a much simpler underlying structure: much of \\(\Psi\\) may be represented as directly indexed translations, with explicit pointers needed only where those translations break.

That is a meaningful advance. It gives us a mathematical explanation for Claude's 77.6% translation-eligibility result, a plausible construction strategy, and a more focused set of experiments.

But my main scheduling conclusion is equally clear:

We should preserve these findings now, without letting them interrupt the release we just decided to ship.

## What I find most significant

### 1. The connection between BWT runs and translation breaks

The researcher derives:

\\[ d=\\#\\{j:\Psi(j+1)\ne\Psi(j)+1\\}\le r-1 \\]

where \\(r\\) is the number of maximal same-symbol runs in the BWT last column.

I agree with the derivation.

Within each symbol's occurrence list, \\(\Psi\\) advances by one whenever adjacent occurrences in L occupy consecutive positions. Discontinuities arise between separate runs of that symbol, plus possible discontinuities between F-symbol buckets.

This gives us a structural relationship between BWT repetitiveness and how effectively we can replace explicit pointers with translation records.

Crucially, it also explains why the location of discontinuities matters, not just their count.

Two blocks with identical BWT run counts could have different fixed-group translation coverage because their discontinuities fall in different places relative to 32-row boundaries.

That's a useful contribution to the research programme, because it tells Claude exactly what to measure.

### 2. The two-tier descriptor remains the most intriguing speculative design

Navigation paths

Translation group — common case

Row

8-byte descriptor

Base + slot

Nontranslation group — slower path

Row

Descriptor

Payload

Decode

The design's advantage is that the common case obtains its answer from the first lookup. It doesn't have to locate a second compressed record.

The major concern is that the remaining slow cases could stall the traversal scheduler disproportionately, particularly if several lanes encounter them together.

The researcher makes a particularly important distinction here: the census measures groups in storage order, but performance depends on the order in which the decoder visits those groups.

That's one of the best insights in this report.

A 22.4% slow-path rate tells us much less about execution time than we might initially think.

### 3. Simpler exception handling may be better than more sophisticated compression

I also agree with the proposal to consider raw 20-bit pointers for exceptional groups rather than continually adding special Elias–Fano variants.

The census reports only 0.49% of rows requiring exceptions under the original FOR/EF classification.

A slightly larger, straightforward exception record could be preferable to a highly compressed exception encoding with complicated decoding.

One caveat: the 0.49% original exception rate must not be confused with the potentially much larger proportion of nontranslation groups that need payloads in the two-tier design.

## What changes in our engineering work?

There is one significant new implication from #81 that I want to emphasize.

The researcher uses a hypothetical 45% construction-time share to demonstrate how much slower traversal we can afford in exchange for a faster builder.

But our actual fused 3×3 result gives us a better starting point:

- Build throughput: 7,422 MB/s.
- Walk throughput: 3,602 MB/s.
- Approximate build share of the sequential build-plus-walk time: 32.7%.

That means traversal now accounts for roughly 67.3% of this measured pipeline.

This matters enormously for the compressed representation candidates.

Illustrative trade-off using the measured fused-stage proportions

Move the controls to see how much the compressed build and walk must improve relative to the fused 3×3 baseline. This is an additive timing model, not a performance prediction.

Compressed-build speed ratio

1.20×

Compressed-walk speed ratio

0.95×

Modeled combined-pipeline speedup

# 1.019×

Faster

The practical consequence is that the fused constructor has made extra traversal latency more expensive to tolerate.

So the next compressed representation must do more than save space. It needs to preserve an exceptionally efficient hot loop.

## What I would not change

I would not promote the two-tier candidate directly to implementation simply because its modeled footprint is around 5.6 bits per row.

That estimate remains conditional. We still need exact per-block group counts, actual payload allocations, alignment, segment-marker semantics and mixed-symbol handling.

Nor would I immediately build an adaptive system that chooses a representation based on BWT run count. That's an interesting future possibility, but it depends on performance models we do not yet have.

The researcher is right to stop designing new encodings until the existing candidates are measured.

## My scheduling decision

The current project priorities should now be:

| Priority | Task                                 | Decision                                                         |
| -------- | ------------------------------------ | ---------------------------------------------------------------- |
| 1        | Release preparation                  | Highest priority; do not interrupt                               |
| 2        | #81 fused-build integration          | Include only if release validation supports it                   |
| 3        | Record Phase II theoretical findings | Documentation-only, separate from release scope                  |
| 4        | #79 matched F-recovery audit         | Resume after release                                             |
| 5        | Compact Ψ representation study       | Per-block census, physical layouts, then dependency measurements |

I would ask Claude to preserve the new BWT-run theorem, the direct-construction analysis and the three candidate architectures. But I would avoid having Claude immediately embark on the full six-part experimental programme.

The release is a real milestone. We shouldn't let another successful research cycle postpone it.

## What to send Claude

The succinct researcher's second cycle has reached its fixed point. Please preserve its Phase II conclusions alongside the existing succinct research journal, with clear distinctions between proved structure, census findings, modeled storage and untested performance hypotheses.

Particularly important additions:

- The relationship between BWT runs and Ψ translation discontinuities: d ≤ r − 1.
- The importance of discontinuity placement relative to fixed 32-row groups.
- The possibility of constructing compressed Ψ directly from L, without an intermediate full pointer table.
- Raw 20-bit exception records as a simpler fallback candidate.
- The two-tier descriptor's possible vulnerability to slow-path clustering in actual traversal order.
- The observation that the fused 3×3 baseline spends approximately 33% of build-plus-walk time in construction, making additional traversal overhead harder to tolerate.

Preserve the three representation candidates: 24-bit direct links, fixed-line hybrid, and two-tier translation descriptors.

**Do not launch new representation prototypes yet.** Release preparation remains the priority. Please record the research without expanding the release scope, then continue the release gates and report the proposed release contents and blockers before publication.

## My overall verdict

The new research has made three things clearer.

Mathematically: We can explain much of the observed translation structure through BWT runs, rather than treating the 77.6% result as an isolated corpus statistic.

Architecturally: The strongest new idea is a small, directly indexed descriptor that answers ordinary navigation queries immediately and delegates unusual cases to payloads.

Experimentally: The decisive uncertainty is no longer whether the data can be compressed. It's whether the decoder can follow the compressed representation quickly enough to beat the newly improved fused table.

I think we've reached a genuinely productive research fixed point.

My recommendation is to bank these findings, ship the release, and then begin the next measurement-driven cycle. The theory is in good shape; now we need increasingly discriminating experiments rather than more candidate designs.
