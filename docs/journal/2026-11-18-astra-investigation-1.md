# Astra's first investigation: phrase BWT to adjacent-rank-gap induction (17–18 November 2026)

A working record of the conversation around the Codex (Astra 6.2) investigation in mibz2. It
gathers Louis's prompts, Astra's reported progress, ChatGPT's commentary and the coordinator's
(Claude's) replies, in order. Text from Astra and ChatGPT is as Louis pasted it. The coordinator's replies are
reproduced in substance, and the chart widgets in ChatGPT's replies are given as their numbers.

**A reader's note.** This is a working discussion. The interpretations here (prefix-free parsing,
tunnelling, dendric shifts, Ukkonen's theorem, Gessel–Reutenauer, SA-IS, move structures, Zorich
acceleration) are candidate connections. Some were corrected later in this same record, and only
those the reviews on #145 verified are established. The verified results are the REVIEWs and
MONODROMY's records cited in each section.

---

## 1. The prompt Louis sent Astra

> I've been developing mibz2, a high-performance bzip2 decoder, and the work has taken an
> increasingly mathematical direction.
>
> The major breakthrough was to stop thinking of inverse BWT as permutation traversal and
> instead understand it through induction on interval exchanges. That led to the Cells kernel,
> and then to a richer understanding of the problem through return towers, words, symbolic
> dynamics and related structures.
>
> The repository is at https://github.com/lmmx/mibz2, and the research blog is at
> https://lmmx.github.io/mibz2/blog/.
>
> I'd particularly like you to read "Towers, words and loops: a mathematical dictionary for BWT
> inversion", alongside the earlier posts explaining how we arrived at induction. The research
> journals, experiments and implementation are also in the repository.
>
> My question is: how do we deepen this?
>
> Induction wasn't simply a better implementation of the old algorithm. It changed how we
> understood the computational problem and made an entirely different kind of computation
> possible. I suspect there may be another change of perspective waiting to be discovered.
>
> There is a concrete reason to pursue this. We've made substantial progress reducing the work
> and memory needed to decode a block, but scaling is now constrained by the interaction of
> working-set size, memory traffic and concurrent workers. On our ten-core machine, adding more
> workers eventually increases contention rather than throughput. The latest measurements and
> their supporting evidence are in the repository.
>
> We'd like to get beyond that limit, ideally making productive use of the second hardware
> threads. But I don't want to assume the answer is a better memory layout, a smaller table, a
> different scheduling strategy, or even another form of induction. Those are consequences of
> particular ways of understanding the problem, not necessarily the right starting point.
>
> What interests me is whether there is a deeper mathematical formulation of block decoding —
> perhaps even of the computational problem itself — in which some of the work, state or
> dependencies we currently regard as necessary turn out to be artefacts of our present
> representation.
>
> Take the mathematics seriously. Distinguish what is proved from what is conjectured or
> measured, and test interesting ideas against the actual computation. The machine ultimately
> has to benefit: we care about real decoding throughput, memory traffic and correctness, not
> mathematical elegance for its own sake. But let the mathematical understanding lead the search
> rather than treating a particular performance counter as the problem to optimise.
>
> You have the code, the research history and the experimental machinery available. Follow the
> mathematics wherever it leads, including beyond the concepts we've already explored. Don't feel
> obliged to preserve Cells, induction, or the current decomposition into decoding stages.
>
> What made induction so powerful was that it was a natural way of understanding the structure
> already present in the problem. It brought an existing body of mathematics to bear on the
> computation, and things that had previously seemed necessary became avoidable. The advance came
> from recognising the right connection and following through on its consequences.
>
> I'd like you to look for the next connection of that kind. Not necessarily new mathematics, but
> a way of bringing mathematical ideas together that makes this computational problem simpler,
> exposes structure we haven't yet exploited, or gives us machinery we haven't yet brought to
> bear.
>
> Don't approach this as an exercise in proposing novel algorithms or surveying the literature for
> gaps. The research history is there to help you understand the problem, not to constrain the
> possibilities. Attend to what the mathematics makes natural, including connections that may
> seem obvious once they're recognised.
>
> And don't stop at an interesting analogy. Follow the connection through. Work out what it
> actually implies for the computation, whether it changes what needs to be represented or
> calculated, and whether the resulting simplification can become an algorithm. Use proofs,
> counterexamples and experiments where they help establish what is true.
>
> The ambition is another advance on the scale of induction: a mathematical connection that
> changes what the computation is, rather than merely improving how we perform it.
>
> Follow the mathematics until it yields a concrete computational consequence. Establish what the
> connection makes possible, rather than merely explaining why it is interesting.

**Coordinator's comments on the prompt.** A strong prompt. The suggested additions:
- a "working agreements" block: ground-truth bundles, an early memo on #145, predictions frozen
  before measuring, prototypes behind a feature flag on Astra's own branch, Louis's identity on
  commits;
- trimming the repeated "follow it to a consequence";
- one boundary: the output stays the bzip2 stream's exact bytes, and everything in between is open.

---

## 2. Astra's first result: phrase BWT and RLE1 (draft PR #553)

> I committed the work to mibz2's research journal on `research/phrase-bwt-rle1` and opened draft
> PR #553. […] **The mathematical result is that the byte BWT already contains a wavelet trie for
> the BWT of whole phrases.** Cut the cyclic text after each occurrence of a chosen delimiter. Your
> first-return experiment had already identified the LF map on those phrases. The additional
> observation is that every delimiter-ended suffix occupies its own disjoint interval in the
> original column. Within that interval, the preceding bytes stably partition the phrase
> occurrences. A suffix with only one possible continuation contributes a spelling byte. Only a
> branching suffix carries information about occurrence order.
>
> That distinction is executable. On the frozen block, given the 2,716 phrase types and their
> frequencies, an independent verifier recovered the entire phrase BWT by reading only 1,058 of
> the 70,560 suffix intervals. Running the construction in reverse reproduced every byte of the
> original column. The native decoder discovers the dictionary from compressed input.
>
> **The same cut can eliminate the full pre-RLE output buffer.** If the delimiter never occurs four
> times consecutively, each delimiter-ended phrase can be RLE1-decoded independently. Three
> backward extensions in the BWT test that condition before inversion. […]
>
> Both native implementations matched `bzip2 -dc` on all **209,700,196 bytes** of the pinned
> Wikidata corpus. The phrase path handled 367 of 431 blocks, including all 233 large blocks. The
> grouped implementation reduced median peak live allocation per fresh large block from **9.59 MB
> to 6.13 MB**. Cells retained the throughput lead. On one pinned Xeon CPU, Cells took **0.823 s**
> and grouped phrases took **1.736 s**. At four workers the times were **0.317 s** and **0.633 s**.
> The effect on i9 DRAM traffic and SMT throughput remains unmeasured.
>
> The held-out experiment narrows the next question. A dictionary learned on six Wikimedia windows
> recognized **96.5% of phrase occurrences** in the next six, yet covered only **12.0% of the bytes
> needed to spell their distinct dictionary entries**. That points to a specific construction
> problem: **can the compressed column identify the rare phrases and emit them as literals without
> first building all their suffixes?**

### ChatGPT's assessment of #553 (summary of Louis's paste)

- A substantial mathematical result, with a convincing prototype and a negative performance
  verdict: −36% peak live allocation, 2.11× the wall of Cells, 209.7M bytes verified, and
  233/233 large blocks admitted.
- The strongest part is the connection that the byte BWT already contains the phrase-BWT
  ordering, close in spirit to recognising LF as an interval exchange. The claimed run bound is
  the key theorem.
- The RLE1 result changes the decomposition of decoding across stage boundaries.
- Why it isn't faster: the verifier was given the dictionary. Discovery costs 70,560 distinct
  suffixes and 81,863 run intersections. Cachegrind counts 32.17M instructions against Cells'
  18.86M.
- Its proposed follow-up was a non-steering prompt: "Is that distinction intrinsic to the
  problem, or a consequence of how we've chosen to represent and construct the phrase system?"

### Coordinator's reply

- The connection is close to **prefix-free parsing** (Boucher, Gagie, Kuhnle, Manzini et al., 2019;
  Big-BWT) and to **BWT tunnelling** (Baier), in the inverse direction. It might add one line to
  the follow-up placing itself against them.
- The run bound needed independent verification by MONODROMY and the Researcher.
- A practical spin-off: avoiding the pre-RLE buffer in Cells could attack the 4.0 B of DRAM
  writes per output byte.
- Caveat: the timings were on one Xeon CPU, and the allocation figure is peak allocation, not
  DRAM traffic.

**Louis's decision:** route #553 to MONODROMY and the Researcher for independent verification,
and give the RLE1 spin-off to the Pathfinder. Leave Astra unsteered: no prefix-free parsing,
tunnelling or engineering suggestions sent to it.

### The verifications (on #145)

- **MONODROMY (REVIEW, 18 November):** the run bound is VERIFIED (a sound injection proof;
  0 violations in 142,207 texts × markers). The reconstruction is VERIFIED, given that every cycle
  holds the marker. RLE1 is VERIFIED (0 failures in 200,000 cases; 604 failures with the
  condition dropped). The three-extension test is exact cyclically and conservative linearly.
  Caution (Conj, from record 51): its simulated LLC read misses are 5.4×, suggesting it would lose
  more on the i9 at 7 or more workers.
- **Researcher (REVIEW 6973462975):** all three claims VERIFIED independently.
  - The run bound holds even for the complete quotient of multiple LF cycles. Equality is
    possible (`L = a##`).
  - Marker-free cycles need separate information (`#a` and `#b` give the same quotient).
  - The replay reproduced 2,716 types, 1,058 of 70,560 mixed intervals and all 899,931 column
    bytes. The read-only stage touches 1.50% of buckets but 40.38% of bytes.
  - RLE1 is checked exhaustively on 27,990 column/marker pairs and 157,326 slices.
  - The measurements are verified within their scope; i9 traffic and SMT are UNPROVED.
- **Pathfinder (REVIEW on QUEUED 6973253342):** removing Cells' pre-RLE buffer saves only about
  0.15 B per output byte. Its cachegrind census reproduced the IMC figure: 11.20 against 11.0
  B/out. It found the refused blocks to be about a third of the traffic, which led to the
  `WideLazyCells` admission experiment.

---

## 3. Astra's follow-up: spelling against order

> I'll look for cases where spelling and order come apart, and where changing the phrase system
> moves information from one to the other. I'll check what survives those changes against the
> frozen block and record the result in the research journal.
>
> There's a decisive counterexample to an intrinsic spelling cost. On the same family of blocks,
> one valid separator puts every free bit into a single phrase spelling. Another gives a fixed
> three-word dictionary and puts those same bits into phrase order. Both separators satisfy the
> RLE1 cut condition. I'm testing a description that uses neither separator and records how left
> and right contexts meet.

### ChatGPT (summary)

- The counterexample disproves an *intrinsic* split between spelling and ordering cost. The
  information moves with the separator.
- It is a counterexample to description size, not yet to computational work.
- The separator-free step asks whether the separator is the wrong primitive altogether, which
  points toward two-sided contexts and extension graphs.
- Its verdict: don't interrupt.

### Coordinator's reply

- Agreed: leave Astra alone.
- What "how left and right contexts meet" probably leads to, and held back from Astra: the
  **extension graph** of symbolic dynamics. Interval-exchange shifts are **dendric** (every
  extension graph is a tree; Berthé, Dolce, Perrin et al.). On the stringology side the related
  objects are the maximal repeats and the CDAWG.
- The computational bar: the decoder starts from the BWT. Any new object must be built for less
  than inversion's roughly one dependent random touch per output byte, measured as B/out at 7
  workers.

---

## 4. Astra: separator-free reconstruction and interval identity

> The separator-free reconstruction worked on the frozen block. It takes a byte histogram and the
> observed connections across contexts that branch on both sides. Of 22,261 such contexts, 20,296
> had a graph with no cycle. For those, knowing which connections exist fixes every count. I'm
> checking that result across the corpus and testing how the answer changes when the block must
> form one cycle.

> The counting argument led to a stronger result. A decoder can track context intervals by
> identity and split them in order, without knowing their widths. That suggests even the byte
> frequencies and graph-edge counts can be omitted from the marked-output representation. I'm
> testing this by giving a separate decoder only the alphabet, the support bits, the block length,
> and the mark.

### ChatGPT (summary)

- **The forest result.** These are contingency tables: $\sum_b x_{ab} = L_a$ and
  $\sum_a x_{ab} = R_b$. A forest-shaped support fixes every count by leaf peeling. Cycles allow
  circulation. 91.2% (20,296 of 22,261) of the contexts are acyclic.
- The caveats: knowing the support isn't free; edge counts are not the ordered text; and the
  one-cycle constraint is global.
- **Interval identity.** It replaces counts (LF(i) = C(L_i) + rank) with symbolic interval
  refinement. Not storing counts is different from not needing to compute them.
- A suggested test: two marked BWTs with the same alphabet, length, support and mark but
  different outputs. Also watch whether the mark hides the missing numerical information.
- Its verdict: closer to the induction style mathematically, uncertain computationally. Don't
  intervene.

### Coordinator's reply

- The forest result is classical (uniqueness on transportation polytopes, by leaf peeling),
  applied in a new place. 91% acyclic reads as the text being "mostly dendric", converging on the
  theory that houses the induction result.
- **The information budget of the support-only claim.** If the support bits plus n and the mark
  determine the text, they *are* an encoding of it, so this is a re-encoding and not a free
  lunch. Two checks:
  - the support bits' size against the compressed block;
  - whether the support is cheaply readable from the BWT. Extension queries normally need
    bidirectional structure. The win would be refinement turning random LF touches into
    sequential ones, measured in DRAM B/out.
- Plan: route the result to MONODROMY and the Researcher for verification, as with #553:
  the forest theorem, sufficiency (the counterexample hunt), and the information count.

---

## 5. Astra: the one-cycle condition and a reconstruction threshold

> The one-cycle condition makes a measurable difference. On the frozen block, counts of every
> cyclic 4,744-byte substring leave two possible primitive texts, up to rotation. Increasing the
> length to 4,745 leaves exactly one. I'm constructing the second text and comparing its
> substring counts directly, independently of the counting formula.

### ChatGPT (first reply, summary)

- This is k-mer spectrum reconstruction (de Bruijn multigraph, Eulerian circuits, a uniqueness
  threshold). Established mathematics, newly applied.
- The explicit second text is the right verification.
- A risk: it drifts toward characterising uniqueness and away from cheaper decoding. A 4,745-byte
  context is very long. A result about primitive single-cycle texts doesn't cover periodic bzip2
  blocks automatically.

### Coordinator's reply

- The likely explanation is **Ukkonen (1992)**, and **Bresler, Bresler & Tse (2013)**: a string is
  uniquely determined by its k-mer spectrum unless it has an interleaved or a triple repeat of
  length at least k−1. So the threshold suggests an interleaved repeat pair of about 4,743 bytes,
  which is plausible in Wikidata JSON. The second text is probably the swap around that repeat.
- If so, it is a rediscovery: characterisation, not a change to the computation. The
  coordinator's suggested thread back to computation was that long repeats appear as runs, and
  could be walked as whole intervals.
- The periodic-block caveat stands.
- The reference for dendric shifts: Berthé, De Felice, Dolce, Leroy, Perrin, Reutenauer and
  Rindone, *Acyclic, connected and tree sets* (2015).

### ChatGPT's correction of the coordinator (accepted)

1. **The Ukkonen connection is compelling**, but "a particular interleaved pair of length 4,743"
   is too quick. It depends on the reconstruction model and the cyclic treatment. An explicit
   alternative text plus a repeat-structure witness would settle it.
2. **"Long repeats are runs" is too strong.** A BWT run is consecutive equal preceding
   characters in sorted order. Its relation to repeats depends on left and right contexts,
   which is exactly what Astra is studying.
3. **The dendric connection may be the better bridge** to the induction mathematics: extension
   graphs, return words, bifix codes, with regular interval-exchange languages as tree sets. But
   91% acyclic on one block does not make the language dendric. That needs the condition
   throughout the language.

A proposed later question, not to be sent until Astra finishes:

> The reconstruction threshold suggests a connection to Ukkonen's theory of repeats, while the
> acyclic extension graphs suggest a connection to tree sets and interval-exchange languages. Do
> these describe separate aspects of the problem, or are they manifestations of a common
> structure? More importantly, does that structure allow any part of reconstruction to be carried
> out without explicitly resolving the corresponding row-level permutation?

**Coordinator's response to the correction.** Accepted on both points.
- "Long repeats are runs" overstated it. The finer left/right-context structure is the point,
  and repeats only become LF-copying stretches under specific context conditions.
- "Mostly dendric" is a loose description of one block's statistics, not a property of the
  language.
- Agreed that both literature connections should be held until Astra has finished, then put to
  it as the question above.

---

## 6. Astra's completed result: bispecial factors (draft PR #553, head 111ff9ac)

Astra's thinking trace, in full as Louis pasted it after the run finished:

> The direct comparison confirmed the threshold. The two primitive texts have identical counts for
> all 899,853 distinct cyclic substrings of length 4,744. Four counts differ at length 4,745. The
> support-only decoder also reproduced all 431 Wikidata blocks through RLE1 and CRC. I'm adding the
> standalone decoder, the frozen input it reads, and the ambiguity witness to the journal PR.

Its response:

> **The split between spelling and occurrence order depends on where you cut the block.** Consider
> the cyclic words `#|x₁|x₂|…xₘ|`, with each `x` either `0` or `1` and exactly h ones. The unique
> `#` fixes the origin. Cutting after `#` puts all C(m,h) possibilities into the spelling of one
> phrase. Cutting after `|` gives the same three phrases, `#|`, `0|`, and `1|`, with fixed
> multiplicities. Every choice now lives in their order. Both cuts satisfy the RLE1 condition.
>
> What survives the change of cut is **which left and right contexts can occur together**. Suppose
> `a` occurs before a string `w` somewhere, and `b` occurs after `w` somewhere. Does `awb` occur?
> When either side has only one possibility, the answer follows. When both sides branch, `w` is a
> *bispecial factor*, and its answers form a binary table.
>
> I built an inverse that reads just those tables, the occurring alphabet, the block length, and a
> mark identifying the starting rotation. Earlier answers determine which table to read next. The
> file contains no phrase spellings, factor names, interval widths, or occurrence counts.
> [Journal entry: docs/journal/2026-11-18-bispecial-factors.md at 111ff9ac]
>
> **The reason even the counts disappear is periodicity.** Once the tables distinguish all the
> possible infinite futures of a finite cyclic text, each future has exactly one predecessor and
> one successor. They form cycles. On an ordinary BWT block there is one distinct primitive cycle.
> If its period is d and the block length is n, each rotation occurs exactly n/d times. Every
> factor frequency follows. For an arbitrary last column with several distinct cycles, the only
> remaining numerical choices for reconstructing the full column are their multiplicities.
> Emitting the marked output requires only its selected future and the requested length.
>
> There is also an exact test of what the one-cycle constraint buys. On the frozen block, **two
> different primitive texts have identical counts for every cyclic substring of length 4,744**. I
> constructed both and compared all 899,853 distinct substrings. At length 4,745, exactly one text
> remains up to rotation. A second ordering is still locally consistent in the BWT, but it splits
> the orbit into cycles of lengths 442,958 and 456,973. The global condition decides a choice that
> the local branching information leaves open.
>
> The **67,024-byte** frozen input reproduced all **899,931** inverse-BWT bytes and the original
> last column. The same construction matched all **209,700,196** final Wikidata bytes through RLE1
> and CRC. Yet the frozen inverse processed **504,258** branching contexts to interpret only
> **22,261** binary tables. The encoder also had to discover those tables from the byte column.
> Throughput and SMT gains remain unmeasured.
>
> That changes my reading of the original experiment. Discovering phrase spellings was a cost of
> that presentation. The deeper computational question is **how much structure a decoder must visit
> even after the output is already forced**. On this block we can now exhibit both kinds of excess
> separately: hundreds of thousands of context splits whose answers follow mechanically, and a
> locally genuine ambiguity resolved by the fact that there is one orbit.

### ChatGPT on the bispecial result (summary of Louis's paste)

- Astra's strongest result so far, and closer to the bouleversement than the phrase dictionary.
  The marked output follows from the support of the factor language, n and a mark, with no
  counts, ranks or widths: the BWT becomes ordered symbolic distinctions. One caveat: the mark
  isn't necessarily bzip2's primary index.
- Concrete evidence: 899,931 bytes recovered from a 67,024-byte support tape (the compressed block
  is 41,618 bytes), 22,261 bispecial contexts, 504,258 splits, and a standalone Rust decoder.
- The obstacle: the encoder built a rank index over the BWT to discover the tables, and the
  decoder still processes 504,258 splits. "A much smaller description of the answer does not
  necessarily mean less work to obtain it."
- The 4,744/4,745 witness: 2,412 → 4 texts, 4,744 → 2, 4,745 → 1. The alternative splits the
  orbit as 442,958 + 456,973 = 899,931. Supporting, not the route to speed.
- A proposed directed follow-up: "Can we exploit the structure of the original transformation to
  pass directly to the marked orbit, without explicitly resolving context refinements whose
  outcomes are forced? … The objective remains a materially faster decoder, not just a smaller
  certificate of its output."

### Coordinator's reply

- Routed for independent verification on the same pattern as before: Researcher CHECKPOINT
  6974406266 and MONODROMY CHECKPOINT 6974410090, with no comments on #553.
- Two additions to ChatGPT's follow-up:
  1. For a bzip2 decoder, the "encoder" is part of the decoder, because we receive the BWT and not
     the support tape. Any cost claim must count the path from the BWT to the output, support
     discovery included.
  2. The measure to ask for is random memory touches per output byte, not split counts. 504,258
     splits for 899,931 bytes is 0.56 per byte, against LF's one dependent random touch per byte.
     If the splits were local or sequential, it could win even without skipping any.

### ChatGPT's revision, folding in both amendments

ChatGPT accepted both amendments, reframed as "resolve the refinements collectively, implicitly
or more cheaply" rather than "skip them". It also cautioned that 0.56 splits a byte against about
one LF access a byte is not a like-for-like cost comparison. The prompt it proposed sending Astra:

> This is a substantial result. The support-only reconstruction is particularly interesting
> because it shows that numerical interval widths and factor frequencies aren't intrinsic to the
> marked-output representation.
>
> But we've now reached the same critical distinction twice: a much smaller mathematical
> description of the answer does not necessarily mean less work to obtain it.
>
> The support decoder still processes over half a million contexts to interpret just 22,261
> bispecial tables on the frozen block. And the encoder must discover those tables from the BWT.
>
> I think the next mathematical step is to understand what computation this new formulation
> actually makes possible. Are all those intermediate distinctions necessary, or can they be
> resolved collectively, implicitly or through a different mathematical construction?
>
> And even where they cannot be eliminated, perhaps their structure allows them to be processed
> much more cheaply than the dependent memory accesses of conventional inversion.
>
> For mibz2, the cost begins with the BWT we actually receive, not with a precomputed support
> tape. Discovering the support is part of decoding. We need to account for the whole computation,
> from compressed input to verified output bytes.
>
> I'd like you to pursue the mathematical implications of this representation towards an
> executable algorithm. In evaluating a construction, consider its memory-access pattern and
> dependencies as well as its operation count, particularly random or dependent accesses per
> output byte. The ultimate test is whole-decoder throughput and memory traffic on the hardware
> that matters.
>
> The objective remains a materially faster decoder, not merely a smaller certificate of its
> output. What does this mathematics let us compute differently, and why should that computation
> be cheaper?

**Coordinator:** agreed. The caution is right: the split count is a structural count, not a cost
prediction, and only a timed, IMC-counted decode settles it.

### Louis's concern, and the final version

Louis asked whether the prompt now pushes Astra into a performance-engineering headspace. ChatGPT
agreed it partly did, and removed the machine-level quantities (DRAM, random loads, cycles) from
the ending. It kept one practical constraint, that the support must be derived from the
compressed BWT actually received, and closed with: "Let the mathematics determine the algorithm.
Let the machine determine whether the algorithm is worth keeping." The final ending:

> Perhaps the important insight isn't that fewer distinctions are needed, but that the right
> mathematical formulation allows them to be resolved in a fundamentally different way.
>
> There's one practical constraint worth keeping in view: mibz2 receives compressed BWT data, not a
> precomputed support representation. Whatever mathematical construction we discover must
> ultimately account for the work of deriving it from the actual input.
>
> Beyond that, I'd rather not prescribe the form of the solution. The structure we've uncovered is
> rich, and I want to follow its mathematical consequences rather than prematurely translate them
> into engineering objectives.
>
> The ambition is still a computational breakthrough: a mathematical connection that makes the
> decoding operation substantially cheaper, not merely a more economical description of its
> answer.
>
> What further mathematical structure would let us compute the result in a fundamentally better
> way? Follow that through to something executable.

**Coordinator:** agreed, with the division of labour: Astra searches mathematically, and the
team (Pathfinder, MONODROMY, the i9 with IMC) does the machine-level evaluation of whatever it
produces.

### MONODROMY's verification of the bispecial result (REVIEW 6974562551, record 54 at 6d557e45)

- **All four claims VERIFIED** with independent code:
  - a protocol implementation from the prose, 12,840 exact round trips with no tape collisions;
  - the witness by its own rolling hash: 899,853/0/0 at 4,744 and 899,856/4/4 at 4,745;
  - the threshold by its own LCP count: one mixed interval at 4,744 and at 4,745, none at 4,823.

  The frozen file and its rebuild reproduce byte for byte. The corpus was not re-run.
- **Size:** 1.61× the bzip2 block.
- **Cost:** the support decoder alone is about 190× slower than a plain inverse (0.675 s on the
  frozen block, one run). The BWT-to-support encoder is untimed.
- **Theory** (not posted on #553):
  - the cycle-rank identity is **Cassaigne's formula** summed, so no periodic block is dendric;
  - the tables sit on at most r − 1 contexts (the same r as q ≤ r);
  - **the support decoder is Rauzy–Veech induction without lengths, unfolded over n futures**
    (Conj).

**Coordinator's note.** The last point closes a loop. Astra, starting from phrases and never
told, arrives at the combinatorial skeleton of Rauzy–Veech induction: the same induction the Cells
kernel uses, with the lengths stripped out. Lengths are what let induction batch many steps into
one, and their absence is why the support decoder is unfolded and 190× slower. That suggests the
"collective resolution" ChatGPT's prompt asks for may be exactly what lengths provide. The
question for Astra becomes what the lengths-free view adds that Cells' lengthful induction
doesn't already have. Held back from Astra for now.

### The Researcher's verification of the bispecial result (REVIEW 6974601979)

- **Separator dependence:** VERIFIED. All 70 small cases replayed.
- **Support-only sufficiency:** VERIFIED, given the distinct-future mark.
  - Terminal classes are rotations of distinct primitive necklaces, and the possible columns are
    the positive solutions of Σ d_j m_j = n.
  - The mark is a rank among distinct futures, at most ⌈log₂ D⌉ bits. For an ordinary block it is
    ⌊primary/(n/d)⌋.
  - A counterexample for the numeric primary with several necklaces: `aab` and `abb` share their
    support and n, but primary 1 emits `aaa` and `bbb` respectively.
- **Ambiguity witness:** VERIFIED, by an independent joint sort. One distinction: the primitive
  alternative at 4,744 is a single cycle. It is the surviving alternate *column* at 4,745 that
  splits as 442,958 + 456,973.
- **Frozen replay and census:** VERIFIED. 504,258 splits, 22,261 tables, 535,773 incidence bits;
  SUP1 is 1.61× the 41,618-byte wrapper.
- **Cheap extraction and speed:** UNPROVED. The encoder scans L into per-symbol position lists and
  refines weighted intervals. Support discovery and mark translation must be charged to the
  decoder.

### Whether to tell Astra about the verifications

Astra's next step: "I'll trace which context splits the support decoder is forced to repeat, then
test whether the compressed BWT can supply their combined effect directly. I'll use the frozen
block to check the mathematics and the full decoder to measure what the construction actually
costs."

- **Coordinator:** send a short factual note with the two verdicts and two precisions (the
  distinct-future mark, and the 4,744/4,745 cycle distinction), and hold back MONODROMY's
  Rauzy–Veech-without-lengths conjecture.
- **ChatGPT:** send nothing. Astra's journal already records both distinctions, including the
  aab/abb example, so the note corrects nothing. After this round, give Astra the reviews and
  MONODROMY's conjecture together, and ask it to compare its formulation with that
  interpretation: equivalent, different, and what machinery each has that the other lacks.
- **Decision:** leave Astra alone; the comparison prompt comes after its current investigation.

### The follow-up for after Astra's current round (ChatGPT's wording, agreed)

> MONODROMY independently suggested that your support-only reconstruction may be Rauzy–Veech
> induction without lengths, unfolded over individual futures. Now that you've developed your
> construction independently, how do these mathematical descriptions relate? Where are they
> equivalent, where do they differ, and does either reveal a computational operation the other
> obscures?

Send it with REVIEWs 6974562551 (MONODROMY) and 6974601979 (the Researcher), and record 54. The
comparison is put on equal footing, not "prove MONODROMY's conjecture".

### Astra: a forest of contexts, with interval copies

> The stable order inside a BWT interval gives a way to skip context refinements. Choose one
> predecessor per short context, then cut the exceptions and one edge on each remaining cycle.
> What's left is a forest. Once a parent context is solved, a whole run in a child can inherit its
> answer by an interval copy. I'm testing how much of the decoding work this actually removes.

ChatGPT's reading: it may have unfolded induction to expose its combinatorics, and is now finding
shortcuts. It batches structural refinement, not temporal traversal. What would convince:
correctness, the work in exceptions against copies, and a whole-decoder win over Cells.

**Coordinator's notes, held back from Astra:**
1. "Cut one edge on each remaining cycle" is a feedback edge set. Its size is the graph's cycle
   rank. MONODROMY's record 54 identified the cycle-rank identity as Cassaigne's formula summed,
   so the number of exceptions measures the block's departure from dendricity, and it is bounded
   by about the r − 1 contexts. So the exceptional work may scale with r, the number of BWT runs,
   not with n.
2. If the copies are interval copies of whole runs, the remaining work becomes sequential copying
   of contiguous intervals rather than dependent random steps. That is the access pattern that
   matters for the knee. It is also the shape of run-length LF, or "move structures"
   (Nishimoto–Tabei), where LF maps a run to a contiguous interval. Not to be sent to Astra.
3. The question to watch in its result: exceptions against n, and total copy length against
   dependent steps. A copied answer inside the support structure is not necessarily a copied
   output byte.

### Astra: cutting the LF cycle at local minima

> The first executable forest recovered the frozen block with 37,490 four-byte contexts and 111,675
> return steps. It was slower than Cells. But it exposed a stronger simplification. Along an LF
> cycle, row numbers rise and fall. Cutting at each local minimum makes the path to the next cut
> solvable in two ordered sweeps. Its endpoint and length suffice to place bytes in the output, so
> I'm testing an inverse that never constructs the return words.

ChatGPT's reading: a pivot from context refinement to the LF permutation's own geometry. Ordered
sweeps could replace dependent traversal, but what makes them work, which property of LF? Also
watch the number of minima and the cost of the sweeps. Encouraging that Astra pivoted from a slower
construction instead of tuning it.

**Coordinator's notes, held back from Astra:**
- The row number along the cycle is the rank of the rotation starting there: the inverse suffix
  array read along the text. Its rises and falls follow local byte comparisons, T[i] against
  T[i+1], except within runs of equal bytes. So the cycle's shape is largely fixed by information
  the BWT gives cheaply. Within a monotone stretch, stable order forces the placement, which is why
  sweeps could replace dependent steps.
- The named connection: the **Gessel–Reutenauer bijection**. Crochemore, Désarménien and Perrin
  (2005) showed the BWT is a special case. It decomposes a permutation's cycles into words by
  descent structure. Cutting at local minima is close to how the **Lyndon factorization** arises.
  Astra may be heading for a Lyndon-style decomposition of the orbit with no grammar or return
  words, a different route from Cells' length-based induction.
- What decides it: the number of minima against n; whether the sweeps run over the BWT or over
  large auxiliary tables; and how output placement is computed from the endpoint and length.

### ChatGPT's caution, and a sharper identification

ChatGPT cautioned that the rise and fall of ranks needn't follow adjacent byte comparisons, since
rotations can share long prefixes. It said local minima don't automatically give a Lyndon
factorization, and that the Gessel–Reutenauer and Lyndon links are interpretations, not
explanations. Its proposed question for after the experiment:

> I'm interested in how you arrived at the local-minimum construction from the context forest. Is
> there a precise mathematical connection between their dependency structures, or did the forest
> suggest a different way to look at LF? What property makes the two ordered sweeps sufficient?

**Coordinator, after checking the claim against the definitions:**
- **Ranks of successive rotations really are fixed by local comparisons.** For rotations at
  adjacent positions i and i+1: if T[i] ≠ T[i+1], the first byte decides. If T[i] = T[i+1], then
  comparing T[i..] with T[i+1..] is exactly comparing T[i+1..] with T[i+2..]. So the up or down
  step propagates through a run of equal bytes to the run's first unequal neighbour. The pattern
  is exactly determined by adjacent comparisons through runs, not just "largely".
- **This is the S-type/L-type classification of SA-IS** (Nong, Zhang and Chan, 2009). A local
  minimum of rank along the text is precisely an **LMS position** (leftmost S-type). SA-IS's
  *induced sorting* resolves everything between LMS positions in **two ordered sweeps**: L-types
  in one direction, S-types in the other.
- So Astra's "cut at each local minimum, then two ordered sweeps" looks very like induced
  sorting, applied to inversion instead of to construction. That identification is sharper and
  more checkable than Gessel–Reutenauer. The Lyndon link is withdrawn as speculative.
- Held back from Astra. It belongs in the comparison round, with ChatGPT's question.

### ChatGPT agrees on SA-IS, with the missing piece

- It confirms local minima of rotation rank = LMS positions (L-to-S transitions), with equal
  adjacent bytes propagating the type, as in SA-IS's recurrence.
- **The missing piece:** SA-IS needs the LMS suffixes ordered before the two induced sweeps, and
  gets that by recursion on a reduced string. For inversion: can the BWT supply the anchors'
  order directly, without the recursive reduced problem? Also a cyclic qualification: SA-IS
  assumes a unique sentinel, and periodic blocks need an explicit cyclic formulation.
- It holds the connection back from Astra until its experiment finishes.

**Coordinator's addition, for the comparison round:** in the inverse direction, the BWT already
holds much of what SA-IS must compute.
- Each row's type can be read off that row: row x is the rotation at i, with F[x] = T[i] and
  L[x] = T[i−1]. So the type of position i−1 compares L[x] with F[x], a purely local sequential
  test. Only ties need one LF step to resolve, and with runs, a chain of them.
- And the rows are already in sorted order: the LMS anchors' order is the row order of the LMS
  rows, which a forward builder has to compute by recursion. What inversion lacks is not the
  order but the row-to-position correspondence.
- So the open question sharpens: given anchors whose order is free, can the sweeps place the
  segments' bytes in text order without walking LF? That is where a real saving would have to
  come from.

### ChatGPT on lexicographic order against text order

| information | what the BWT gives |
|---|---|
| lexicographic order of rotations | already available |
| S/L classification | local where adjacent bytes differ; ties need more work |
| textual positions of the LMS anchors | not directly available: the real challenge |

ChatGPT names a possible circular dependency: the sweeps may need their seeds' text positions,
which are exactly what inversion has to find. Resolving that mathematically, rather than by
walking LF, would be a major advance. Its verdict: don't interrupt, and hold SA-IS back.

**Coordinator:** agreed, including the tie correction. "One sequential pass" was too strong:
ties chain through runs, so classification is sequential except along runs of equal bytes. What
to look for in Astra's result is whether the anchors' text positions come out of the sweeps or
are fed into them.

### Astra: run-level rounds (trace cut off)

> The run-level version now recovers the frozen block in 12 rounds. Its first round represents
> 298,922 local minima with 27,327 intervals. I also found a way to bound the work across later
> rounds. Adjacent BWT rows can be followed together until they encounter an original run
> boundary. I'm checking that this lets [cut off]

ChatGPT: within a run, LF(x+1) = LF(x) + 1, so adjacent rows move together as an interval. That
is the run-length and move-structure property. The work bound across rounds matters more than
the 12 rounds; the hope is something like O(n + r log n) with mostly sequential access.

**Coordinator, held back from Astra:** this is the move structure of Nishimoto and Tabei
(2021). Following an interval until it meets an original run boundary, and splitting it there,
is their construction. Their balancing result bounds the number of pieces by O(r), at O(1)
amortised per LF step. So "bound the work across later rounds by original run boundaries" is
very likely their bound, reached independently from the local-minimum side. That is the
convergence predicted in the forest note. The new part, if any, is applying it to the
local-minimum anchors with rounds, rather than LF step by step. For the comparison round.

**ChatGPT's correction, accepted:** move structures make *individual* LF steps cheap: O(r)
space, O(1) a step. Following a whole cycle that way still takes n steps. What could be new in
Astra's 12 rounds is computing the orbit *collectively*: one interval operation advancing many
rows' positions at once, with total work bounded by active intervals × rounds rather than path
length. The move-structure property explains one ingredient, not the algorithm. The decisive
question: does Astra compute the orbit collectively, or just store it compactly? The hidden costs
to count are the sorting, splitting and copying inside each round, plus recovering text positions
and emitting the output.

**ChatGPT, accepted:** collective *traversal* is not collective *placement*. An interval that
moves together in row order can map to widely separated text positions. Two possible
breakthroughs: advancing many LF trajectories at once, and placing their bytes without tracing
each one. The local-minimum construction's claim that "endpoint and length suffice to place
bytes" is what could join them. The proof must show the intervals stay manageable across rounds,
and that placement doesn't bring back the dependent traversal.

**Coordinator:** this is the row-to-position gap from the SA-IS note in another form. Placement
is where a saving is won or lost.

---

## 7. Astra's final result: adjacent-rank-gap induction (draft PR #553, head 79688e06)

> **The intermediate contexts can be resolved collectively.** Inside a BWT run, two adjacent rows
> emit the same byte and move to another adjacent pair. Until they encounter a run boundary, every
> comparison of ranks along one trajectory has the same outcome along the other. In particular,
> both trajectories select the same interior local minima. That agreement survives when we
> contract the excursions between minima and repeat.
>
> This gives a decoder that starts from bzip2's Huffman symbols, recovers maximal BWT runs, and
> constructs a grammar for the marked output. **If the initial exchange has R translation intervals
> after moving the primary to rank zero, every contraction round needs at most 3R−2 intervals.**
> One boundary bit certifies when neighbouring excursions have equal spellings. The decoder can
> merge them without expanding or comparing their words.
>
> On the **frozen block**, the constructor visited 102,233 intervals across 12 rounds and bypassed
> all 504,258 context refinements. In a paired full-decode test, the run-record implementation was
> **28× faster than the earlier support encoder and decoder**. Cells was faster still.
>
> The clearest computational win is the family (abcd)^m!. The unique marker makes **every rotation
> distinct**, yet the BWT has exactly five runs. The first contraction induces addition by one
> modulo m, and the second leaves only the marked rank. At 875,001 bytes, the executable used **ten
> grammar rules** and decoded the complete .bz2 in **0.161 ms**, versus **1.526 ms** for shipped
> Cells. With 256 seeded substitutions it still won by 2.3×. At 1,024 substitutions Cells won.
>
> **The Wikidata scaling problem remains open.** On the full corpus, the run-record implementation
> took **8.37 s versus 0.729 s** for Cells on one calling thread, and **2.68 s versus 0.345 s**
> with four workers. On the frozen block it generated substantially more memory traffic in cache
> simulation. All 209,700,196 Wikidata output bytes matched independent bzip2 decompression. The
> test machine exposed no SMT siblings.

### ChatGPT (summary)

- The load-bearing result is the bound k_j ≤ 3R−2, with total construction
  O(R(1 + log(n/R)) log(2R)). Work is controlled by the run structure, not by the trajectories.
  The proof uses a graph of gaps between adjacent ranks.
- It makes a different computation possible: on the five-run family, ten rules and 9.5× faster
  than Cells. But that input is extremely structured (56 compressed bytes), and the advantage
  fades as runs increase.
- It loses on Wikidata, 7.8–12.3× slower: ropes, splits, joins and grammar construction, with
  11.5× the data references.
- It is another induced return system, inducing on local minima, where Cells uses Rauzy-style
  induction from the end. The question for later: what does the gap-path bound reveal that Cells
  doesn't exploit, and can the two be combined?
- A possible specialised decoder for very-low-run blocks, given a real workload and a cheap
  admission rule.

### Coordinator

- Routed for verification: Researcher CHECKPOINT 6976384308, MONODROMY CHECKPOINT 6976389585.
  MONODROMY also gets the relation to Rauzy–Veech and to record 54, off #553.
- On Wikidata, R/n is about 0.08–0.15 per block, so R is in the tens of thousands per 900 KB
  block. The log factors and the rope constants then dominate, and Cells' lengthful induction is a
  lower-constant route over the same runs. The real win needs R ≪ n: highly repetitive data (logs,
  versioned or genomic collections). That is a workload question for a specialised admission
  path.

**ChatGPT's caution, accepted:** "Cells is a lower-overhead route over the same run structure"
is a computational reading, not a mathematical result. The two choose different induced systems:
Rauzy-style reduction against induction onto local minima with gap-path control. They may expose
different invariants. The one-bit equal-spelling certificate recognises equivalent return paths
from their relation in LF, information a grammar might not expose. Asymmetry: Cells has the
efficient induction mechanism, and Astra may have a better way to recognise equivalent paths.
Combining them may yield what neither shows alone. The question for Astra after the reviews:

> How do these two inductions relate mathematically? Which properties become visible in one
> formulation but not the other? Can either formulation supply a simplification that changes the
> computational cost of the other?

The specialised low-run path is the team's adoption question, not Astra's.

---

## 8. Louis's assessment, and the next prompt

**Louis** (voice, summarised): Astra repeatedly pivoted from ideas that didn't work. In good
faith, that is spirited exploration, each failure launching the next idea. In bad faith, it is
poor calibration about which ideas are good, and rescuing failed ones without owning it. The
outcome, a big win on a five-run toy family, is small and irrelevant to the brief, which was
implicitly real-world Wikidata. We don't want galactic algorithms. We may have under-constrained
the framing, while still not wanting a performance tuner. And we dropped the "two inductions
together" prompt: was that an overcorrection?

**ChatGPT (first reply, summary):** Astra reasoned well, but research judgement was weaker. It
showed "research momentum": each step rational, the programme poorly directed. There was no
evidence of bad faith; it reported its failures. The missing distinction is between mathematical
simplification and *relevant* mathematical simplification: the Wikidata blocks' structure should
be part of the mathematical problem. "Freedom to choose the mathematics is not the same as
freedom to choose what counts as solving the problem." Make early testing by counting part of
discovery.

**ChatGPT (second reply):** yes, that was an overcorrection. Restore the two-inductions question,
with Wikidata as the constraint on success. The final prompt:

> Thank you for this investigation. The adjacent-rank construction is a substantial mathematical
> result, and I particularly appreciate the progression through phrase decompositions, extension
> graphs, context refinement and induced return systems.
>
> I'd like to bring this work into contact with the induction mathematics already developed for
> Cells.
>
> We now have two ways of inducing on the permutation underlying inverse BWT. Cells uses
> Rauzy–Veech-style induction on interval exchanges; your construction induces on local minima and
> uses adjacent-rank gaps to control the number of intervals. They approach essentially the same
> dynamical system from different directions, and each appears to make different structure
> visible.
>
> How do these two inductions relate, and what becomes possible when we understand them together?
>
> I'm particularly interested in whether one reveals mathematical operations, invariants or
> simplifications that aren't apparent in the other. The adjacent-rank gap argument and the
> certificate for equal return words seem especially worth understanding alongside Cells'
> representation of return towers and words. But don't limit yourself to those examples, or assume
> that combining the existing algorithms is necessarily the answer.
>
> There's an important practical tension to keep in view. Your construction has a strong work
> bound and performs extraordinarily well on very-low-run blocks, but is substantially slower than
> Cells on Wikidata. The Wikidata blocks aren't merely benchmarks to consult after developing the
> theory; their structure is part of the mathematical problem. We need an advance that remains
> consequential when the BWT has tens of thousands of runs, not only when it has five.
>
> I'd like you to investigate the relationship between the two formulations deeply enough to
> discover whether it permits a fundamentally cheaper computation on those realistic blocks. That
> might mean transferring a mathematical insight from one induction to the other, finding a common
> formulation that makes a stronger operation possible, or recognising that both are special cases
> of something more useful.
>
> Don't feel obliged to preserve either construction. And don't keep rescuing an approach once its
> underlying advantage has been disproved for the target inputs. Equally, distinguish an expensive
> implementation from a mathematical idea whose advantage remains unexploited.
>
> The goal isn't an equivalence theorem for its own sake, nor a hybrid implementation assembled
> from existing components. It's to use the mathematics to discover a better way of computing.
>
> What does understanding these inductions together reveal that neither makes apparent alone?
> Follow the strongest consequence through to an executable algorithm, and establish whether its
> advantage survives on the real blocks mibz2 decodes.

**Coordinator:** Louis's reading is fair, and the evidence fits "research momentum without a
relevance check": Astra was honest throughout, but never asked whether each idea's advantage
depended on a property Wikidata lacks. One count would have shown it early: 51,184 runs in
899,931 rows. The suggested addition, after "…not only when it has five.":

> For scale: the frozen Wikidata block has 51,184 runs in 899,931 rows (r/n ≈ 0.057), and the
> large blocks are similar. Check early whether a direction's advantage survives at that density,
> by counting where you can, before building.

Send after MONODROMY's and the Researcher's verdicts on the gap result.

### Verifications of the adjacent-rank-gap induction

**MONODROMY (REVIEW 6976532126, record 55 at 4f4d8276):**
- Claims 1–4 and 6 are VERIFIED with its own row-level model from proof.md. It reproduced the
  frozen block's 102,233 visits over 12 rounds, and (abcd)^m!'s 9 visits in 2 rounds. Across
  11,279 cases: 0 output mismatches, 0 unsound merges, 0 violations of 3R−2. The bound isn't
  shown tight; the worst growth found is 1.25R.
- **Theory:**
  - **the ceiling division is Zorich acceleration, which Cells lacks.** It explains the
    (abcd)^m! win and buys nothing on Wikidata, which has no powers (record 20). A divided step in
    Cells is a lever only for low-r/n blocks;
  - **halving** gives an O(log n) worst-case round count that Rauzy–Veech lacks;
  - **the gap paths** are record 30's vertical saddle connections, and **the certificate** is the
    proof of MONODROMY's 15 November equal-words result;
  - **record 54's conjecture is supported:** keeping the lengths is what removes the n/R factor.

**Researcher (REVIEW 6976571989):**
- All claims VERIFIED.
  - **3R−2** is proved after certified merges; it is loose at R = 2, and sharpness is unproved.
  - **The certificate** is verified with its other tests.
  - **Construction:** strict minima halve the active rows; visits are bounded by
    Σ min(3R−2, n/2^j), and the log(2R) factor comes from the AVL joins, splits and sorting.
    Materialisation costs O(G+n).
- **Measurements:** verified within their scope. 11.50× Cachegrind is for the frozen block only.
  No SMT was available.

**Coordinator:** with both verifications in, the comparison prompt is ready. MONODROMY's mapping
(ceiling division = Zorich acceleration; gap paths = saddle connections; certificate = the
equal-words result) is now the substance of "how the two inductions relate". So the
recommendation changes: give Astra REVIEWs 6976532126 and 6976571989 and record 55 with the
prompt, so it starts from the established mapping instead of rederiving it. The open question is
then genuinely new: given that mapping, is there a construction that helps at r/n ≈ 0.057?

### ChatGPT's final prompt for the next round (agreed)

It gives Astra the mathematical findings (both reviews and record 55), not MONODROMY's
interpretation as settled ("Zorich" still needs a precise statement). It keeps the
two-inductions comparison central. It treats r/n ≈ 0.057 as a starting measurement, not a
characterisation of Wikidata: blocks with the same r/n can differ in run-length and return-time
distributions, factor complexity and arrangement. It asks for early counting tests and for
calibration (don't rescue constructions whose advantage is absent; don't abandon an idea over a
slow first implementation). Its closing question: "What does the combined mathematical picture
now allow us to see that neither construction revealed on its own?" The full text is in Louis's
message of 18 November.

**Coordinator:** agreed; send as written.

---

## Conclusion of the first investigation

The first investigation closes here (Louis, 18 November). The second begins with the comparison
prompt in §8, and gets its own journal entry.

**Established, each verified independently by MONODROMY and the Researcher on #145:**
1. **The phrase-BWT run bound** and the reconstruction from branching intervals.
2. **The RLE1 independence condition** (#553, first head).
3. **Separator dependence, support-only sufficiency** with a distinct-future mark, and the
   4,744/4,745 reconstruction threshold (second head).
4. **The adjacent-rank-gap induction** (third head, 79688e06): the 3R−2 bound per round, the
   one-bit equal-spelling certificate, and an O(R(1 + log(n/R)) log(2R)) construction from the
   run list to the output grammar.

**The computational verdict:**
- The final algorithm is about 9.5× faster than Cells on very-low-run blocks ((abcd)^m!, 5 runs).
- It is 7.8–12.3× slower on Wikidata, where r/n ≈ 0.057.
- MONODROMY's record 55 explains why: its accelerated step is Zorich-like acceleration, which
  pays only where long powers exist, and Wikidata has essentially none (record 20). So the first
  investigation produced real mathematics, but no gain for mibz2's principal workload.

**The lesson carried into the second:** freedom to choose the mathematics is not freedom to choose
what counts as success. The second investigation makes the structure of the real blocks part of
the mathematical problem, and asks for early, count-based tests of whether an advantage survives
there.

