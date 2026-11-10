[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-357/blog/index.md>)

# A new way back from the BWT

A new way to rebuild text turns repeated pieces into reusable rules and brings a familiar tree into view.

10 November 2026 · 10 min read · [Research in progress](<#6-what-remains-before-production>)

The Burrows–Wheeler transform rearranges a block's bytes so that compression can exploit repeated text. Reversing that rearrangement is one stage of bzip2 decoding. In mibz2's new experiment, a small program of reusable word-building rules reconstructs the block instead of a conventional pointer walk.

That experiment has crossed its first performance threshold. A redesigned kernel recorded less time than its conventional inversion-stage reference in two local Wikidata censuses. The second recorded **1.604 against 2.172 nanoseconds per row**, a derived **26.2% reduction**.

These are indicative single-thread results from the research container, using the fastest of five rounds per block. They are not CI performance evidence, and no confidence intervals accompany them. The new kernel starts with a run list; scanning a column to obtain that list is excluded. There is **no demonstrated gain over the shipped decoder**, and the experiment is not in production.

There are two further findings. The right-end construction has a connection to a known mathematical object, the Lyndon tree, under the primitive-word condition explained below. Changing the order of operations also reduces their count, but the faster kernel cannot yet execute that order cheaply. Neither finding is an additional measured speedup.

The [research journal](<https://github.com/lmmx/mibz2/blob/ae51b11a46014e5fec928b0e25235f532a473a9f/docs/journal/monodromy/2026-11-10-rauzy-veech.md>) and [implementation](<https://github.com/lmmx/mibz2/blob/ae51b11a46014e5fec928b0e25235f532a473a9f/examples/rauzy.rs>) are pinned here at **ae51b11a46014e5fec928b0e25235f532a473a9f**. MONODROMY is the research lane developing this experiment.

## 1\. How can rules replace a pointer walk?

A conventional inverse transform builds a table telling it which row to visit next. Each row supplies a byte, and the next address depends on the current lookup. That dependency can leave the processor waiting for memory.

The table is more structured than an arbitrary permutation. Within a **run**, a maximal stretch of equal bytes in the transformed column, the next-row map advances by one at each position. The map is called **LF**, for last-to-first: it connects positions in the last and first columns of the sorted rotations.

For a small example, sort the rotations of `aab`: `aab`, `aba`, `baa`. The last column is `baa`; the first is `aab`. Matching equal letters in occurrence order gives LF mappings 0→2, 1→0 and 2→1.

Name the one-byte run of `b` B and the two-byte run of `a` A. Their original order is B,A; their image order is A,B. The two orders describe the same three row positions, cut into differently placed intervals.

Two steps rebuild aab from its last column

Each box is one run's interval; its width is the rows it still covers, and its label is the word it carries. Green marks the interval whose word grew. After step 1, B covers one row but carries two bytes: a remaining visit stands for the visits removed from the exchange.

Compare the intervals at the right end. A has length two and B length one. Remove one row from the exchange, shorten A to one, and give B the word `ab`. Both intervals now have length one. Their equal-length step joins A's `a` to B's `ab`, leaving the word `aab`.

Why retain a longer word on a shorter interval? A remaining visit now represents several visits in the original map. Its word records their bytes. This is the first-return idea behind Rauzy–Veech induction: retain where the path returns, together with what happened along the omitted path.

Larger inputs repeat this operation and share word rules. If `X → ab` is reused in `Y → XX`, Y represents `abab` without duplicating X's stored definition. That second rule illustrates reuse, rather than another step in this particular example.

The essential change is where the work happens. The conventional walk repeatedly asks where to go next. Induction changes the description of the remaining problem and saves enough information to reconstruct the answer afterwards.

## 2\. What did the clock actually show?

The original linked-order representation lost. The replacement uses cells that record completed steps without repeatedly moving letters through linked orders.

The journal reports three censuses below. Each row keeps its own reference beside the candidate, so the separate runs do not become an invented matched comparison. The unit is nanoseconds per BWT row, pooled over 233 Wikidata blocks.

| Census | Conventional table build + walk | Experimental kernel | Candidate/reference |
| :--- | ---: | ---: | ---: |
| Original linked orders | 2.172 | 2.770 | 1.276 |
| Cells, first run | 2.244 | 1.697 | 0.756 |
| Cells, second run | 2.172 | 1.604 | 0.738 |

The cells kernel against its own census's reference

Dashed: conventional table build + walk, from a column. Solid: the experimental kernel, from run records (the column-to-run scan, 0.484 ns a row, is excluded). Each kernel is compared only with the reference timed in its own census. Local single-thread fastest-of-five figures from \#351's journal at ae51b11, with no confidence intervals; not CI, not the whole decoder, and not against 0.2.8's records-based path.

The cells kernel recorded lower times on 202 blocks in the first census and 204 in the second. Other work ran on other cores during the first; the second pinned the process to one core with the others idle.

The original 2.770 figure sums the stages' separate fastest rounds. The cells figures time order, induction and output together. Consequently, the apparent 42% drop between the original and latest totals is a historical comparison across runs and timing definitions, not a paired effect estimate.

The input boundary matters too. The conventional arm builds from a column; the experimental kernel receives runs. The separately measured column-to-run scan costs 0.484 ns/row in the journal. A front end that emits runs directly could avoid that scan, but the published table does not measure that integrated path.

Nor does it compare against the newer records-based construction associated with [the front-end work](<https://github.com/lmmx/mibz2/pull/324>). Calling the result a 26% improvement to a released mibz2 version would go beyond the evidence.

The favourable corpus is not the whole story. The cells experiment is 3.93 times its reference on prose, 9.78 times on uniform bytes, and 8.28 times on a near-periodic adversarial input. Those are reported local ratios from the same journal's other-input table.

## 3\. What made the cells cheaper?

Consider a repeated word represented by a rule. The algorithm must retain the word and its destination, but it need not repeatedly update every historical description of that word.

The first cells version still lost: the journal records a ratio of 1.225 in its development sequence. A new representation was an opening, not an immediate win.

A cell is an overlap between an interval in the original order and one in the image order. It groups rows that take the same pair of run labels in succession.

The cells representation applies that distinction to the exchange. It writes completed cells and records where later work belongs. This replaces much of the linked representation's shortening and relinking.

A second change removes position tracking from the induction loop. The original row is marked in the grammar, so the output stage can find the correct starting position afterwards. Moving that information into the representation eliminates work previously repeated at each step.

Finally, lookahead fetches prepare locations before scattered writes need them. The journal's source and instruction analysis support these mechanisms; they do not isolate three independent end-to-end causal effects. The implementations and timings changed together.

The result also uses more memory than the conventional table in the journal's representative block. The benefit is how that memory is accessed, not a demonstrated reduction in total storage.

## 4\. Why does a Lyndon tree appear?

Joining words builds a tree: its leaves are bytes, and each internal node joins its children's words. Sharing identical subtrees turns that tree into a reusable grammar.

A **primitive** word is not an exact repetition of a shorter word. Its lexicographically smallest rotation is a **Lyndon word**, smaller than each of its other rotations. For example, `aab` is smaller than `aba` and `baa`; its standard split is `a | ab`, followed by `a | b`.

The journal identifies the right-end induction's expanded derivation tree with the Lyndon tree of that least rotation. It first relates the construction to a **Cartesian tree**: a hierarchy formed by repeatedly splitting a sequence at its minimum. Here the sequence contains ranks of rotations beginning at the word's internal boundaries. For `aab`, the rotations at positions zero, one and two have ranks zero, one and two. The internal sequence is therefore \[1,2\]. Its minimum splits after the first `a`, giving `a | ab`; the remaining split gives `a | b`. This agrees with the two rules above. In general, the standard Lyndon split uses the longest proper suffix which is itself a Lyndon word.

From the row map to a known tree

The identity concerns the expanded derivation tree; stored rules share subtrees, and the kernel need not build every node. It is proved in \#351's journal and checked by simulators there, under the primitive-word condition (aab is primitive; abab is not). This post checked the record; it does not independently certify the theorem. Green: the two internal nodes, which are the two rules of §1.

The known tree theory is not a new invention. The research contribution under examination is the connection from this interval construction to that tree without first reconstructing the text by a conventional walk.

The journal includes a proof and reports independent simulators and node comparisons. This post has checked that record, not independently certified the theorem. The primitive-word condition matters: `aab` satisfies it, whereas an exact repetition such as `abab` needs separate treatment.

## 5\. Can a better order remove more work?

Yes, in the counted schedules. Whether it saves time is still open.

Let `r` be the number of BWT runs. The journal compares induction steps divided by `r`, so blocks with different run counts can be compared. A “free” step here means that its bookkeeping creates no new boundary; it still costs processor work.

The favourable order takes available free steps first, then chooses the largest cell. Its count is 1.279 steps per run, compared with 1.812 for the right-end sweep.

The order of steps changes how many there are

Counts, not times: the cheaper order is not yet cheap to execute in the cells kernel. The floor holds for any order of these cell steps (r₂ − 1, the cells of both orders' cuts, minus one); a fusion outside this step model can go below it, so it is not a lower bound on inverse BWT.

The journal's identity is `steps = initial cells − 1 + newly introduced boundaries`. In the toy example, the union of both orders' cuts gives three initial cells. Two steps leave one, with no new boundary introduced. A re-seating into an interval's interior can instead add a cut that a later step must remove.

The identity gives a floor of 1.220 steps per run for these cell-step schedules. The favourable order is about 29.4% below the right-end count and 4.8% above that floor.

There is already a qualification to the word “floor.” The journal describes an additional fusion operation outside that step model which can go below it. This is not a lower bound on inverse BWT itself.

The fast cells kernel relies on sweeping the ends of its orders. At the source pin, the journal prices an out-of-order step as built at 152 instructions, against 64 for a swept cell. Fewer steps can therefore lose if each step becomes expensive enough. Whether that cost lies in the step itself or in finding the next useful one is the research lane's next question.

Parallel rounds face the same distinction. A schedule model suggests ample work for up to sixteen workers, but it omits execution overhead. The sequential round-based implementation took about 1.99 times its reference. A model of available parallelism is not a multi-worker speed measurement.

## 6\. What remains before production?

The performance reversal is useful evidence for continuing the experiment. It establishes neither a general-purpose replacement nor a release result.

The next comparison needs matched input boundaries and the relevant records-based reference. The unsafe kernel still needs independent safety review. Tests reported in the journal do not substitute for that argument, and adverse inputs need a bounded fallback or a different reduction.

At the source pin, the research lane's broader verification was still pending in the assignment record. No completed five-lens review is claimed here.

The earlier conjecture about a tree has become a recorded structural identity. Building the full array of text-position ranks first, however, could pay the very reconstruction cost this approach seeks to avoid.

The deeper opportunity is now clear. The program can exploit the hierarchy of words, but its cost depends on how cheaply it discovers and records that hierarchy. The next improvement must make the cheaper mathematical order cheap for the machine as well.
