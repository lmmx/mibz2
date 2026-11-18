# The ChatGPT Pro investigation: the brief, the trace and the discussion (18 November 2026)

**Reader's note.** This is a discussion record, not a research result.
- **The primary record:** `2026-11-18-pro-investigation-recovered.md`, the session's own recovery
  document, itself a transcript-based reconstruction.
- **What this file holds:** the progress messages, as Louis pasted them from the session's trace,
  and the interpretations by ChatGPT and the coordinator discussed alongside. Some are
  speculative, and none is verified.
- **The literature checks** (the macro scheme, Φ, Policriti and Prezza) were done by search
  during the discussion. They are pointers, not an assessment.

## 0. The brief

The session was given the brief and the nine attachments prepared on 18 November: the statistics
packet, the reading list, both Astra journals, the discussion of investigation 2, and the
open-return proof and README. The brief asked for an independent mathematical investigation of
whether decoding solves a harder problem than it needs to.

## 1. Pro's first updates

> I'll start with the packet, the two investigations and their reviews. I'll use the recorded
> block measurements to choose which mathematical direction deserves a full derivation, then
> test that derivation against a decoder or exact small-case oracle.

> The exact Wikidata corpus is available, including the frozen block. I've checked one tempting
> shortcut against it. Giving every run a literal and copying the remaining bytes in
> power-of-two rank layers resolves the dependency cycles, but it breaks the frozen block into
> 347,803 copy operations even with every address supplied. I'm testing whether groups of rows
> can instead stay together through LF long enough to avoid constructing those addresses.

> I found a way to recover return words after discarding their spelling grammar. Keep the first
> row of each run as a literal. At every unmarked row, the row immediately to its left emits the
> same byte. If several adjacent trajectories share a return word, its leftmost trajectory can
> therefore read from just outside the group. Whenever that source refers to another unknown
> byte, its representative BWT rank is strictly smaller. This gives a termination proof for a
> dependency system that can look cyclic when drawn one whole word at a time. I'm checking how
> much work it takes to construct and evaluate that system on the real blocks.

**ChatGPT (summary):**
- **The ordering:** a rank-decreasing order on byte-level dependencies would make the apparent
  whole-word cycles (Astra's 5,063-byte obstruction) an artefact of grouping.
- **Complementary to Astra:** Astra supplies the bands, and Pro may supply their words without
  tracing.
- **Termination is not efficiency:** chains could decrease one rank at a time.
- **What would convince:** exactness, collective (interval) evaluation, and a real cost win from
  the compressed input.
- **Don't interrupt.**

**Coordinator:**
- **The literature (confirmed by search, 18 November):** the observation "the row to the left emits the
  same byte except at a run head" is the BWT side of the Φ function, Φ(SA[i]) = SA[i−1].
  - *Φ itself:* Kärkkäinen, Manzini and Puglisi's PLCP construction; also the r-index's toehold
    and Φ with O(r) samples (Gagie, Navarro and Prezza, SODA 2018 / JACM 2020).
  - *In text coordinates:* the text copies itself from Φ(p) everywhere except at the text
    positions of run heads.
  - *A macro scheme:* BWT runs induce a valid bidirectional macro scheme of at most 2r phrases
    (Navarro, Ochoa and Prezza, "On the approximation ratio of ordered parsings", arXiv
    1803.09517, Theorem 7; LATIN 2018). Its phrases start at the text positions of the run
    heads, through the suffix array. Their validity proof is the counterpart of Pro's
    rank-decrease argument.
  - *Run heads as an attractor:* the run heads also form a string attractor of size r (Kempa and
    Prezza, STOC 2018).
- **If so:** Pro's termination proof re-derives that validity in the decoder's terms. The new
  content would be in its next step: evaluating the scheme collectively along shared LF
  trajectories (Astra's bands).
- **Decoding a macro scheme is the known hard part.** Rank-by-one chains can be linear, the
  efficiency question ChatGPT raises. That is the same obstruction investigation 2 measured
  (5,063 of 848,747 bytes by whole spans), and Pro's 347,803-operation layering is its cost when
  forced.
- **For later verification,** not to send to Pro: check the novelty claim against Φ, the 2r
  macro scheme and the string-attractor literature. The test that matters is whether the
  band-level evaluation beats representative tracing (U = 50,709 visits on the frozen block)
  and the retained table (m = 89,874).

## 2. On the macro-scheme connection

**ChatGPT (summary):**
- **Agreed:** a macro scheme existing is not the same as evaluating it efficiently, which echoes
  both investigations.
- **Possibly complementary:** the macro-scheme view gives the dependencies, and open-return gives
  families of trajectories to resolve together.
- **Correction:** don't make the ~3,200 width-16 bands a requirement. Pro may find a different
  grouping, or show that grouping needs expensive preprocessing.
- **Unchanged:** launch investigation 3 independently, and withhold the macro-scheme reading
  from Pro.

**Coordinator:**
- **The correction is accepted.** The bar is the total cost of decoding from the compressed
  input, against open-return's and against 0.5.5's, not a match to one cut's band count.
- **The frozen-block figures** (U = 50,709, m = 89,874, B = 3,204) are reference points for
  where open-return's costs lie, not targets.

## 3. An exact copy construction, and a negative result on the dictionary

> The construction now reproduces all 431 Wikidata blocks, with every block CRC and all 12 stream
> CRCs checked. On the frozen block, 2,976 substring copies and 3,448 literal roots describe 61,393
> positions in the return-word dictionary. The timing comparison gives a different verdict: the
> existing labelled endpoint already expands its grammar cheaply. I'm testing whether those copy
> relations can produce sequential output cheaply enough to change that comparison.

**ChatGPT (summary):**
- **An exact alternative:** a correct way to build Open-Return's dictionary, but not a saving,
  since the endpoint already expands its grammar cheaply. That is useful early falsification.
- **3,448 literal roots** equal the wide strips at t = 16, so Pro works in Astra's structures.
- **Next:** sequential output, which isn't automatically cheaper.
- **The final report should separate** discovery, dictionary, retained permutation and emission,
  and measured traffic from assumed traffic.
- **Don't intervene.**

**Coordinator:**
- **Pro's negative result agrees with record 57's cost analysis, reached independently.** The
  dictionary (61,393 positions here; U = 50,709 traced letters) is not where open-return's cost
  lies. The retained system is: m = 89,874 rows, with an LF and a 4–8 B table entry each. Pro is
  now moving towards the part that remains.
- **The cost of output:** it is bounded below by the writes, 899,931 bytes, and that floor is
  shared by every decoder. A sequential-output scheme gains only if it avoids the retained
  system's dependent random accesses. That is the same question investigation 3 asks from the
  dynamics side, so the two sessions may now be converging on the same component.

## 4. Fewer positional samples; a 19% cut in induction steps; no speed gain

> Only a subset of run boundaries needs an independently recovered text position. On the frozen
> block, 17,578 sampled ranks determined all 67,252 distinct head and tail positions. I also
> tested that reduction from the BWT runs. It produces the predicted copy program, but obtaining
> the samples still costs more than the labelled control spends on the entire inverse.

> One simplification carries back to the existing labelled endpoint. Removing ranks that touch
> no complementary arrow reduced induction steps on every one of the 233 large blocks, by 19% at
> the median. I tested two ways to restore those ranks with the same output writer. Both still
> expanded almost the entire original dictionary, and neither showed a reliable whole-inverse
> speed gain.

**ChatGPT (summary):**
- **The samples:** 74% fewer independent samples, a structural reduction within this copy
  representation, not a lower bound for every decoder. Obtaining them still costs more than the
  whole inverse.
- **The 19% median cut in steps** holds across all 233 blocks, but the dictionary is still
  expanded almost entirely, and there is no reliable speed gain.
- **A pattern:** each reduction is genuine, but obtaining or expanding the information eats the
  advantage. Watch for rescue momentum. The hope is that Pro returns to the question of *why*
  removing structure keeps failing to remove cost.
- **Don't intervene.**

**Coordinator:**
- **One concrete reason for the pattern, at least for the copy-based line:**
  - *Two coordinate systems:* macro-scheme copies are addressed in text coordinates, while the
    decoder holds only BWT coordinates.
  - *The conversion is the inversion:* a run boundary's text position is a suffix-array sample,
    and recovering one from the BWT means walking LF from the primary, which is the inverse
    itself.
  - *Why the r-index manages:* it stores those samples, built at indexing time, and bzip2's
    format doesn't carry them.
  - *So:* any scheme that needs text positions pays something like an inversion to get them.
    That matches Pro's "obtaining the samples still costs more than the entire inverse". Fewer
    samples (17,578 against 67,252) doesn't help if each is found by dependent walking.
  - *What this predicts:* a copy-based route can win only if its samples are found collectively,
    without walking: a property of the BWT side, not the text side.
- **The 19% step reduction** belongs to Open-Return's endpoint constructor, which isn't production.
  Note it for the Pathfinder in case the i9 comparison makes Open-Return a candidate. The
  constructor's cost wasn't dominant (record 57), so expect little.

## 5. The oracle copy schedule: 11,145 against 64,258

> I checked how much of the gap belongs to the copy schedule itself. With the decoder's own seed
> bytes, the frozen text needs exactly 11,145 nonoverlapping copies if any earlier occurrence may
> be used. The current construction uses 64,258. A seed assembled from LF chains in rank order,
> which needs no text-position samples, gives essentially the same optimum at 11,130. These counts
> use the decoded text as an oracle. They identify which information a cheaper constructor might
> dispense with.

**ChatGPT (summary):**
- **Slack:** there is 83% slack between the construction and the oracle optimum. It separates
  executing a schedule from discovering one.
- **An oracle result:** an existence result, "exact" only relative to the model and seed.
- **The LF-chain seed** needs no position samples and gives the same optimum, so the schedule's
  quality may not depend on the seed.
- **The open question:** can the BWT yield a near-optimal schedule without the output?
- **Don't interrupt.**

**Coordinator:**
- **"Any earlier occurrence, nonoverlapping copies" is the LZ77 model**, where a greedy longest
  match is optimal. So 11,145 is in effect an LZ77-style parse of the frozen text, given the seed
  literals. For scale, record 40 gives z ≈ 0.39 r over the corpus, about 20,000 phrases for a
  block with r = 51,184 and no seed.
- **Executing it is gzip's problem,** and cheap: sequential copies from earlier output, with no
  dependent jumps across a table.
- **Discovering it from the BWT is a known problem with a known cost** (confirmed by search).
  Policriti and Prezza compute the LZ77 parse from the run-length BWT in O(r log n) bits but
  **O(n log r) time** (DCC 2016, arXiv 1510.06257; Algorithmica 2018). Their key device is at
  most two suffix-array samples per run, the sampling cost Pro met. That is asymptotically more
  than the O(n) direct inverse.
- **So the question becomes** whether Wikidata's structure lets the parse be found in less than
  the log r factor: from the bundles, or from Pro's LF-chain seed, which needs no samples. Pro's
  11,130 points that way. If parse-finding stays at n log r, the oracle gap is out of reach,
  which is the outcome ChatGPT names.

## 6. ChatGPT's qualifications on the LZ77 reading

- **Not necessarily the standard greedy LZ77 objective:** the seed changes what counts as a
  literal, and the copy and overlap rules matter. The optimum is for Pro's model.
- **The core obstruction is the change of coordinates:** BWT order against text order. A copy
  schedule is cheap to run but may be dear to find.
- **The LF-chain seed:** that it can be *defined* without samples is shown. That the schedule can
  be *discovered* cheaply from it is not.
- **A caution on the asymptotics:** n log r against n compares upper bounds, not lower bounds.
  Samples needn't each need a separate walk, since work can be shared.
- **The broader opportunity:** a computation in which output order emerges collectively,
  without choosing between the two coordinate systems.

**Coordinator:**
- **All accepted.** The upper-bound caution corrects my wording ("each found by dependent
  walking"). Policriti and Prezza themselves obtain all their samples in one shared pass; the
  n log r is for the whole parse.
- **The correct claim:** no known method finds the schedule from the BWT in less than the cost
  of a direct inverse. That is not a proof that none exists.

## 7. The session ended without a handoff

The Pro chat finished without delivering its final message, a ChatGPT interface bug. Louis is
trying to recover it; its files and programs appear lost. The trace updates in §§1–5 are the
surviving record. They are claims from a trace, unverified, with no code or evidence behind them.

## 8. The recovery

The session's recovery document is committed beside this file, verbatim. Assessment by MONODROMY
and the Researcher is queued behind their verification of investigation 3. The first item is the
proposed oracle-input experiment: whether the copy construction can beat the labelled inverse
when its endpoints, translations and literals are given for free.
