<!--
Provenance: three replies from the research chatbot on Kocberber, Falsafi & Grot,
"Asynchronous Memory Access Chaining" (PVLDB 9(4), 2015; VLDB 2016), relayed by the user into
the working session on 29 October 2026, unedited. The user introduced them with "the
researcher pulled out a v interesting paper - AMAC: Asynchronous Memory Access Chaining
(2015) www.vldb.org/pvldb/vol9/p252-kocberber.pdf". The first reply answers whether AMAC
argues against interleaving the Ψ walks; the second and third read the paper in depth. The
links, the inline images' placeholders and the LaTeX are kept as they arrived (the links
carried a utm_source parameter, kept). The paper's figures quoted here were checked against
the paper, and the record's own measurements set beside them, in
docs/journal/2026-10-29-amac.md.
-->

## 1. Does AMAC argue against interleaving?

No—quite the opposite. Interleaving independent Ψ walks is exactly the kind of technique AMAC advocates.

But you've spotted an important distinction that could affect how we implement interleaving.

AMAC isn't an argument against interleaving. It's an argument against rigid interleaving, where every lane must advance through the same stages together.

The difference is between:

- Lockstep interleaving: advance lane 0, then lane 1, then lane 2, repeatedly following a fixed schedule.
- Asynchronous interleaving: each lane maintains its own state and advances whenever its next operation can make progress.

Both expose memory-level parallelism. The second can handle irregular amounts of work more gracefully.

Our room walker already resembles the asynchronous approach because it maintains independent traversal state and schedules multiple chains. The interesting question is how much flexibility its scheduler actually provides.

## Where AMAC might suggest a different approach

The original [AMAC paper](https://www.vldb.org/pvldb/vol9/p252-kocberber.pdf?utm_source=chatgpt.com) specifically identifies irregular chains, early termination and differing amounts of work as situations where asynchronous scheduling can outperform fixed groups.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.vldb.org\&sz=32)

VLDB



For mibz2, that creates an interesting distinction.

A conventional Ψ step is highly regular: load a link, recover the next row, repeat. Fixed interleaving is well suited to this.

But our decoder has irregularities:

- Segments finish at different times.
- Lanes need replacement work.
- Some scheduler operations require more instructions.
- A future compact Ψ representation might occasionally require a second dependent payload load.

Those are the cases where AMAC-style scheduling might help.

For example, imagine one compressed Ψ lane needs an extra payload fetch while seven other lanes can advance normally. A rigid schedule may repeatedly revisit the stalled lane. An asynchronous state machine could leave it pending while continuing useful work elsewhere.

There's a catch, though: software doesn't ordinarily know whether a prefetched cache line has arrived. AMAC uses saved state and scheduled revisits; it doesn't magically receive a notification whenever a load finishes. Greater flexibility also costs instructions, branches and state-management overhead.

So making the scheduler more asynchronous could actually slow our existing regular 32-bit traversal.

### What I'd conclude

We are not doing the wrong thing by interleaving. Our measured 25.7% full-decoder improvement is powerful evidence that the overall approach works.

The genuinely interesting question is narrower:

Does the room walk already provide enough independent progress, or would finer-grained scheduling help when some lanes have more work than others?

I wouldn't change the released implementation on the strength of the analogy.

But there's one particularly intriguing implication for compact Ψ: the best encoding and the best lane scheduler may need to be designed together.

A two-tier representation that looks slow under fixed interleaving might behave better if its exceptional payload loads can overlap with work from other lanes.

That's a plausible future research avenue—not a reason to rethink our successful cooperative architecture.

## 2. A deeper look at AMAC: are we leaving parallelism on the table?

I went back to the actual paper: [Asynchronous Memory Access Chaining](https://homepages.inf.ed.ac.uk/bgrot/pubs/AMAC_VLDB16.pdf?utm_source=chatgpt.com), by Kocberber, Falsafi and Grot (PVLDB, 2015).

There's an important surprise.

AMAC isn't a scheduler that detects which memory request has finished. It's a scheduler that allows independent traversals to be at different stages of their work.

That's subtler—and more relevant to our future compressed Ψ representation—than the simple description I gave earlier.

### 1. What AMAC actually does

The authors compare three strategies for multiple independent pointer-chasing operations.

Group prefetching

All active lookups advance together through the same processing stage. This works well if every lookup has the same number of dependent memory accesses.

Software-pipelined prefetching

Lookups occupy fixed pipeline stages. It can overlap memory requests effectively, but divergent lookup paths complicate the pipeline.

AMAC

Each lookup carries its own state: its current pointer, processing stage and other information needed to resume. A circular scheduler visits these independent states and advances each according to its own situation.

The fundamental difference is stage independence, not abandoning round-robin traversal.

In fact, the AMAC implementation itself uses a circular buffer and advances through its slots in round-robin order.

When one lookup ends, its slot can immediately start another.

The paper's improvement comes from avoiding synchronization between otherwise independent chains—not from magically knowing when each prefetched cache line becomes ready.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://homepages.inf.ed.ac.uk\&sz=32)

homepages.inf.ed.ac.uk



That corrects a possible misconception from my earlier reply: a round-robin scheduler can be thoroughly AMAC-like.

### 2. Why this might matter for compressed Ψ

Our present 32-bit T-vector has a beautifully regular operation:

Current direct-table lane

Load T[j]

Get Ψ(j)

Prefetch next

Every ordinary navigation step performs essentially the same kind of work.

That regularity makes the current interleaving approach attractive. There's little benefit in assigning lanes complicated processing stages when they mostly do the same thing.

But the proposed two-tier Ψ representation is different.

A translation group can answer directly from the descriptor.

A nontranslation group needs another payload.

Two-tier Ψ: different lanes need different amounts of work

If we process every lane synchronously, a slow-path lane can interfere with the progress of a translation lane.

AMAC suggests keeping the two situations separate.

A lane might be in either of these states:

- Descriptor stage: interpret the descriptor; if translation, immediately calculate the successor; otherwise issue a payload prefetch.
- Payload stage: retrieve the payload and recover the successor.

Crucially, a lane waiting for a payload needn't prevent another lane from advancing through its translation fast path.

That is the interesting architectural possibility.

### 3. The most interesting trick is where we suspend a lane

This is where I think the paper gives us a genuinely useful idea.

Suppose a lane encounters a nontranslation descriptor.

A straightforward implementation might do this:

```
Load descriptor
Discover that payload is needed
Load payload
Decode successor
Prefetch successor
Move to next lane
```

The payload load depends on the descriptor load, so the processor cannot issue both at the same time.

If the payload misses cache, the current lane may stall before the scheduler reaches the next lane.

An AMAC-style implementation separates the work:

```
Visit lane A:
    Load descriptor
    Discover payload is needed
    Prefetch payload
    Mark A as awaiting payload
    Move to lane B

Visit lane B:
    Decode a translation
    Prefetch its next descriptor
    Move to lane C

Later, revisit lane A:
    Load payload
    Decode successor
    Prefetch its next descriptor
```

That changes the timing.

The slow-path lane's extra memory access can overlap with useful work from other lanes.

We're not removing the extra dependency. We're allowing other chains to make progress while it is pending.

And that's exactly the distinction AMAC was designed to exploit.

The paper's implementation preserves a per-lookup stage and pointer in a circular buffer, and it advances the requested stage when that lookup is revisited. Its reported advantages are strongest when lookup paths have irregular lengths or stages.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://homepages.inf.ed.ac.uk\&sz=32)

homepages.inf.ed.ac.uk



### 4. But there's a potential trap

Consider what happens when a lane is revisited too soon.

A software prefetch does not expose a completion signal.

So if we prefetch a payload and then revisit its lane before the data arrives, an ordinary load may still stall.

AMAC doesn't eliminate that possibility.

It relies on enough independent work being scheduled between the prefetch and the eventual load.

This introduces a useful concept: prefetch distance measured in independent lane work.

A simple approximate model is:

\\[ \text{useful separation} \approx (N-1)\times C \\]

where \\(N\\) is the number of interleaved lanes and \\(C\\) is the average number of cycles spent processing another lane.

If the resulting separation is short compared with the relevant cache-miss latency, the payload prefetch may arrive too late.

And if many lanes take the slow path simultaneously, the scheduler might find itself revisiting several outstanding payloads before any have arrived.

That means the proposed architecture has two separate questions:

1. How many payload loads are needed?
2. How much other useful work can overlap each payload load?

The second is the part our earlier compact-Ψ analysis hadn't explored thoroughly.

### 5. A particularly interesting consequence: the best format could depend on the scheduler

Consider two representations.

| Property                 | Fixed-line FOR/EF           | Two-tier descriptor                |
| ------------------------ | --------------------------- | ---------------------------------- |
| Main record              | One cache line              | One small descriptor               |
| Exceptional data         | Usually contained in record | Separate payload                   |
| Memory dependency stages | Usually one                 | Sometimes two                      |
| Decoder complexity       | Bit extraction or select    | Very simple translation fast path  |
| Scheduling opportunity   | Limited additional stages   | Potential overlap at payload stage |
| Working-set footprint    | About 16 bits/row model     | About 5.7 bits/row model           |

Previously, we regarded the two-tier design's second dependent load as a serious disadvantage.

And it remains one.

But AMAC suggests its effective cost may be partly hidden by an asynchronous scheduler.

That changes the nature of the comparison.

The two-tier format could be worse under a rigid walker and better under a staged walker, even when both use exactly the same compressed data.

The reverse is possible too: extra stage checks, larger lane state and reduced prefetch distance could make the staged version slower.

The paper doesn't settle which outcome applies to mibz2.

It tells us that comparing representations using just one scheduling strategy could miss an important part of the design space.

### 6. There's an even subtler possibility: descriptor fetches can overlap payload fetches

The key requirement is keeping independent memory requests in flight.

At a given moment, an AMAC-style decoder might have:

Illustrative pipeline occupancy across eight Ψ lanes

Lane 1

Next descriptor

Lane 2

Payload pending

Lane 3

Translation

Lane 4

Next descriptor

Lane 5

Payload pending

Lane 6

Translation

Lane 7

Next descriptor

Lane 8

Payload pending

Asynchronous stages let lanes have different outstanding work, rather than requiring all eight to finish the same stage together.

There's a subtle advantage here.

An ordinary translation lane may be able to determine its next descriptor address with very little computation.

That lets it issue another prefetch quickly.

Meanwhile, a slow-path lane can leave its payload access outstanding.

The CPU may therefore be able to maintain a useful mixture of outstanding descriptor and payload requests.

This is exactly where cooperative interleaving and compact representation design might reinforce one another.

But there are limits.

Modern CPUs have finite load buffers, miss-handling resources and bandwidth.

Maintaining more outstanding requests is helpful only until one of those resources becomes saturated.

The original AMAC paper identifies limits on the number of outstanding L1 and last-level cache misses as important constraints on achievable memory-level parallelism.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.research.ed.ac.uk\&sz=32)

University of Edinburgh Research Explorer



And extra prefetches can compete with demand loads for the same resources.

This means AMAC-style staging should be understood as a way to make better use of available memory parallelism—not as an unlimited supply of it.

### 7. Does the existing room walker already do this?

Here's where I need to be careful.

We know the cooperative room walker maintains multiple independent traversal lanes, and that segment handoffs are already integrated into its scheduling.

But I haven't inspected the current release-ready source for this particular question.

So I cannot claim that the existing scheduler is rigid, or that it lacks useful asynchronous behaviour.

The distinction I would investigate in its control flow is straightforward:

Does the walker have the ability to suspend an individual lane after initiating a memory access, while allowing other lanes to advance independently through different processing stages?

If yes, the architecture may already possess most of what AMAC offers.

If no, the present direct-table format may simply not need that extra generality.

Neither outcome undermines the measured cooperative improvement.

### 8. One reason not to get carried away

The AMAC paper's strongest results concern database hash lookups with irregular numbers of pointer dereferences, early exits and synchronization.

Those operations can have dramatically different execution paths.

Our current direct-table Ψ walk is much more regular.

Every ordinary step advances one permutation link.

Consequently, we should expect much less benefit from a sophisticated state machine in the existing direct-table walker than in the paper's highly irregular database workloads.

The paper reports up to 2.3× improvement over competing prefetching methods under irregular workloads, but that number has no predictive value for our current decoder.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://homepages.inf.ed.ac.uk\&sz=32)

homepages.inf.ed.ac.uk



The more relevant future scenario is a genuinely irregular two-tier compact representation.

There, AMAC's distinctive feature becomes useful: one lane may need an extra dependent memory operation while another does not.

That's where the analogy becomes substantive.

### 9. The genuinely new research question

I think we've arrived at a more interesting question than “should we use AMAC?”

It's this:

> Can a compact Ψ representation with occasional extra memory dependencies achieve direct-table-like successor throughput when each lane is allowed to advance independently through its lookup stages?

That question connects three threads of our research:

- Our measured Ψ compressibility.
- The cooperative walker's ability to interleave many independent chains.
- AMAC's ability to preserve memory-level parallelism when different chains require different amounts of work.

It's also testable in a much cleaner way than rebuilding the whole decoder.

The conceptual comparison is:

A

32-bit direct

Existing regular cooperative walk

B

Two-tier Ψ

Immediate payload loads, no suspension

C

Two-tier Ψ

AMAC-style staged payload loads

If C substantially outperforms B, we've discovered that the scheduler matters to the economics of compact Ψ.

If C approaches A while using much less table memory, that becomes particularly interesting for our throughput–memory frontier.

If C gains nothing over B, then the extra complexity is probably unnecessary for this workload.

One particularly useful measurement would be the performance of C relative to B as a function of the number of active lanes and the fraction of accesses requiring payloads.

That would isolate exactly the mechanism AMAC predicts.

### My conclusion

The most important thing I've learned from reading the actual AMAC paper is that we shouldn't equate fixed round-robin scheduling with rigid lockstep execution.

AMAC itself can use round-robin visitation.

What matters is whether independent lanes can occupy different stages, and whether one lane's exceptional work forces others to wait.

For the existing regular T-vector walker, the architecture we've built is already very close in spirit to AMAC.

For a future two-tier Ψ design, however, an AMAC-style state machine might make the difference between an unacceptable second dependent load and a tolerable one.

And that creates a rather intriguing possibility:

We may eventually discover that the best compressed Ψ representation is not the one with the cheapest isolated lookup, but the one whose occasional extra lookups are easiest for our cooperative scheduler to overlap.

That's a different way to think about the design space—and a credible new insight from stepping outside the BWT literature.

## 3. Following AMAC to its logical conclusion for mibz2

I've gone back through Kocberber, Falsafi and Grot's [Asynchronous Memory Access Chaining](https://homepages.inf.ed.ac.uk/bgrot/pubs/AMAC_VLDB16.pdf?utm_source=chatgpt.com) in more detail, including the pseudocode, scheduling design, performance breakdowns and hardware bottleneck analysis. I've also followed the idea into subsequent coroutine-based database systems.

I think there's more here than merely adding a `PAYLOAD_PENDING` state to our walker.

The deeper insight is that a compressed representation and its traversal scheduler should be treated as one algorithm.

We have been evaluating compression partly by asking how much extra work it adds to a Ψ lookup.

AMAC suggests a different question:

At which exact point does each lookup become dependent on memory, and can we arrange for another independent lookup to advance at that point?

That change of perspective produces several useful deductions.

I'll distinguish what the paper demonstrates from what we can infer for mibz2, and follow those deductions until the remaining questions genuinely require measurements or inspection of the current implementation.

### 1. First, the key correction: AMAC isn't about a smarter round-robin order

The paper's Figure 4 and Listing 1 are worth studying closely.

[CSDL | IEEE Computer Society](https://images.openai.com/static-rsc-4/XczztSqX35GFAPkFYjR4YmV1TQmTwitCIIDvY9psgMr4r92Vo2eNKhzmdMdiV-oKj9S9icTTYN0NixWa7zoBOO1uShR7_sFagpRrtlQHF5vTQ2tOEMyYddAp11cfSG0PjdIviZWqloQJ6R5kmqBBOMVZESzO-xjfnGFL8T1M6UU?purpose=inline)

AMAC uses a circular array of independent lookup states.

Each slot records the information needed to resume a traversal: its pointer, processing stage, and other contextual data.

The scheduler visits slots in a simple circular order.

After visiting a slot, it either updates that traversal's state or starts another lookup if the previous one finished.

There is no elaborate ready queue, no hardware notification of cache-line arrival, and no requirement to execute every lane's stages together.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://homepages.inf.ed.ac.uk\&sz=32)

homepages.inf.ed.ac.uk



This matters because our room walker's existing round-robin design may already have the right scheduling foundation.

The thing to borrow is stage independence, not a replacement scheduler.

In the simplest case, we need only extend the state a lane can occupy.

### 2. The first deeper deduction: suspension points should follow unknown addresses, not codec formats

We initially imagined:

- Translation state.
- FOR state.
- EF state.
- Payload state.

But that would be the wrong abstraction.

FOR, EF and translation are decoding operations. They're not necessarily different memory-wait stages.

If all the fields needed by a format are contained in the same cache line, it usually makes sense to decode them together.

The state machine should instead follow memory dependencies.

For example:

A memory-dependency state machine

Conceptual states for a two-tier Ψ lookup. A suspended payload lookup resumes on a later scheduler visit.

Notice that there is no special state for an inline FOR record.

If the descriptor or cache-line record already contains everything needed, we simply compute the successor and continue.

Only a lookup requiring an additional address-dependent fetch needs suspension.

This is a useful simplification.

It means we can potentially support several compression formats without making the walker a complicated general interpreter.

The state machine's complexity is governed primarily by the number of dependent memory stages, not the number of bit encodings.

### 3. The second deduction: keep one outstanding successor operation per lane

This follows from a structural property of Ψ.

For any traversal lane:

\\[ j\_{t+1}=\Psi(j_t). \\]

The next row is unknown until the previous row is resolved.

So even with asynchronous execution, a lane cannot normally have two future Ψ steps in flight at once.

But it can be in one of two situations:

1. Waiting to retrieve the record needed for its current successor.
2. Waiting to retrieve a secondary payload required to resolve that successor.

Once the successor is calculated, the lane immediately knows its next primary record address.

This suggests a compact invariant:

> Each live lane should represent exactly one unresolved navigation step, with at most one currently useful next-stage prefetch.

Why is that helpful?

Because it bounds the work represented by a lane.

We don't need a queue of arbitrary outstanding requests per lane.

We need one stage indicator and the address or row index relevant to that stage.

That keeps the state machine relatively small.

It also gives us a correctness-friendly interpretation of the scheduler: advancing a lane means either resolving its current Ψ step or moving that step one memory-dependency stage closer to resolution.

### 4. The third deduction: the critical optimization is preserving the refill rate of memory requests

This is the part of AMAC that I hadn't fully appreciated before.

The authors explicitly describe merging the completion of one lookup with the initiation of the next. They do this to avoid losing an opportunity to launch a new memory request.

Their aim is to maintain a steady supply of independent accesses.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://homepages.inf.ed.ac.uk\&sz=32)

homepages.inf.ed.ac.uk



For mibz2, this suggests a stronger rule than simply “switch away after a prefetch”.

Whenever a lane completes a Ψ step, calculate and prefetch its next address as soon as possible.

For a translation:

```
Read descriptor
Calculate next row
Prefetch next descriptor
Save state
Move to next lane
```

For an exceptional group:

```
Read descriptor
Calculate payload address
Prefetch payload
Save pending state
Move to next lane

Later:
    Read payload
    Calculate next row
    Prefetch next descriptor
    Save state
    Move to next lane
```

The important invariant isn't that every lane receives equal CPU time.

It's that after each available stage completes, any newly known memory address is issued promptly.

This is the principle I'd most want to preserve in the actual decoder implementation.

A scheduler can be perfectly fair and still perform poorly if it frequently postpones known next addresses.

Conversely, a very simple round-robin scheduler can work well if it keeps useful memory operations in flight.

### 5. The fourth deduction: the number of lanes should relate to outstanding misses, not simply thread count

AMAC's Figure 6 contains a particularly useful result.

On the authors' Xeon X5670, performance improved as the number of independent in-flight lookups increased, then largely saturated around ten.

Their explanation was an architectural limit on outstanding L1 data-cache misses, rather than a shortage of independent logical lookups.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://homepages.inf.ed.ac.uk\&sz=32)

homepages.inf.ed.ac.uk



That's not a claim about our i9-10900K. Its microarchitecture, cache hierarchy and execution resources differ.

But the principle is directly relevant:

Beyond a certain point, additional software lanes increase bookkeeping without increasing useful memory-level parallelism.

For mibz2, 32 lanes may be an effective choice, but we shouldn't interpret 32 as a hardware requirement.

It is an engineering parameter that trades available independent work against scheduler overhead and CPU resources.

And a two-tier representation could change the optimum.

A slow-path lookup may occupy a lane for longer because it traverses two dependent memory stages. That could mean more independent lanes are useful—or it could mean extra requests saturate the same hardware resources sooner.

Both are plausible.

The way to distinguish them is not through a generic benchmark of 32 independent integer lookups, but through actual Ψ successor chains with varying slow-path frequencies.

### 6. An unexpectedly important result: AMAC itself can lose when data fits in cache

This is where the paper becomes a much better guide than its headline speedups suggest.

For a small hash table that fits in the authors' last-level cache, their more complicated grouped and software-pipelined prefetching methods were actually slower than the ordinary implementation.

In one experiment, AMAC still won, but the result was much more modest: around 21% better than baseline, versus much larger gains on out-of-cache workloads.

The instruction counts explain why.

| Method                         | Instructions per tuple | Cycles per tuple |
| ------------------------------ | ---------------------- | ---------------- |
| Ordinary traversal             | 36                     | 27               |
| Group prefetching              | 90                     | 37               |
| Software-pipelined prefetching | 67                     | 28               |
| AMAC                           | 55                     | 22               |

AMAC paper, Table 3: the small hash-table configuration on Xeon X5670.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://homepages.inf.ed.ac.uk\&sz=32)

homepages.inf.ed.ac.uk

AMAC added roughly 53% more instructions than the baseline and still improved throughput, but the other methods' additional instruction costs overwhelmed their benefits.

This is a remarkably good warning for mibz2.

Our successful cooperative policy has already improved cache behaviour by reducing the number of simultaneously live tables.

That means the new implementation may spend more time in precisely the cache-friendly regime where sophisticated software prefetching has less to offer.

A staged walker could be useful for the compact representation while still regressing the direct-table representation.

That leads to a conclusion I wouldn't have drawn from the abstract alone:

AMAC-style staging should be evaluated as part of a particular memory layout, not as an unconditional upgrade to the existing scheduler.

And it suggests preserving a very small, predictable direct-table hot loop rather than adding unnecessary stages to a path that has already demonstrated good performance.

### 7. Now the most interesting part: what does AMAC imply for the two-tier design's actual cost?

Let's revisit the two-tier proposal.

Our measured model puts about 77.4% of rows in translation-eligible groups.

The remaining 22.6% require some other representation.

The two-tier proposal places one 64-bit descriptor per 32 rows, with auxiliary payloads for nontranslation groups.

That gives us an exceptionally small primary table.

But the important performance metric is no longer simply the number of bytes stored.

It's the sequence of requests needed to resolve a successor.

#### An approximate memory-request model

Let \\(p\\) be the fraction of visited rows whose descriptors answer directly.

In a full traversal covering every row once, using the census as an approximation:

\\[ p\approx0.774. \\]

Suppose every row needs one descriptor lookup, and every nontranslation row needs one additional payload lookup.

Then the expected logical lookup count per successor is:

\\[ E[D]=p(1)+(1-p)(2). \\]

So:

\\[ \boxed{E[D]\approx1.226}. \\]

That is about 22.6% more logical memory requests than a one-record representation.

This is not a cache-miss count.

Many descriptor requests may hit cache, several accesses may reuse a cache line, and secondary payload requests may also hit cache.

Still, the result exposes the central trade-off.

The two-tier representation can dramatically reduce the working set, but in exchange it creates additional dependent requests for a minority of steps.

AMAC can potentially overlap those requests across lanes. It cannot eliminate them.

That's a crucial distinction.

##### There's another subtlety

The average fraction of exceptional accesses doesn't describe how those accesses arrive.

Because Ψ is a permutation, the full traversal eventually visits all rows. But the order in which 32 independent lanes encounter exceptional groups is not determined by their overall frequency.

We could have fairly evenly distributed exceptions.

Or several lanes could encounter exceptions in bursts.

Those two situations may produce very different resource pressure.

So the earlier idea of measuring slow-path adjacency remains relevant, but the AMAC connection adds a stronger question:

How many lanes are simultaneously in a payload-pending state during actual traversal?

That is a temporal measurement, not just a property of the encoded table.

It also depends on the scheduler.

This is probably the most important missing statistic for evaluating the two-tier design.

### 8. A new and useful distinction: exceptional frequency versus exceptional occupancy

Imagine that 22.6% of all navigation steps require payloads.

It doesn't follow that 22.6% of active lanes will be waiting on payloads at any particular instant.

Why?

Because payload stages can have different service times.

If an exceptional step takes longer, then exceptional stages may occupy a disproportionately large fraction of the scheduler's active state, even though they represent a minority of completed navigation steps.

This is a general queueing phenomenon.

A simple approximation is:

\\[ f\_{\text{pending}} \approx \frac{(1-p)T\_{\text{payload}}} {pT\_{\text{direct}}+(1-p)T\_{\text{exceptional}}}, \\]

where the \\(T\\)'s denote effective time spent in the relevant scheduler states.

This is only an illustrative model; useful work, prefetch overlap and resource contention prevent us from treating it as a measured prediction.

But the implication is important.

The instantaneous composition of the scheduler can look much more exception-heavy than the encoded data itself.

If so, the hot loop's common case may not be quite as dominant as the 77.4% statistic suggests.

The converse is also possible: if payloads are usually cache-resident, an exceptional state may clear quickly and occupy little of the active pipeline.

That is one reason the payload layout matters enormously.

It determines not just storage overhead, but how long exceptional lookups remain active.

### 9. We should think of the payload address as a second-stage continuation

This is the cleanest way I've found to express the AMAC connection.

A compressed descriptor isn't necessarily the complete answer to Ψ.

Sometimes it is a continuation: it tells the decoder where the rest of the answer can be found.

This suggests representing a lane as a miniature continuation state.

Conceptually:

```
enum Stage {
    Descriptor,
    Payload,
}

struct Lane {
    row: u32,
    stage: Stage,
    pending_payload: usize,
    // Existing output and segment bookkeeping
}
```

The key detail is that we don't need to save a complicated decoder context.

For a nontranslation lookup, the continuation may require only:

- The current row or slot.
- The address of the payload.
- The processing stage.
- Existing output and segment state.

At the payload stage, the decoder resumes and calculates the successor.

This is precisely the kind of small state machine AMAC advocates.

It is also where an implementation can go wrong.

If adding the state requires many additional loads and stores to the lane array, the scheduling machinery may consume the speedup.

AMAC's authors were careful to minimize state overhead and to merge stages where possible.

A later database-engine line of work, including [CoroBase](https://vldb.org/pvldb/vol14/p431-he.pdf?utm_source=chatgpt.com), arrives at a similar conclusion: suspending work around memory accesses is valuable only when saving and restoring the work's state is sufficiently cheap.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://vldb.org\&sz=32)

VLDB



For mibz2, I would initially favour a hand-written, small state machine over a general coroutine framework.

That's a deduction from our hot-loop constraints, not a claim that coroutines are necessarily slower in Rust.

### 10. A surprisingly important question: what happens when a lane finishes?

AMAC stresses immediately replacing completed database lookups with new work.

That is how it keeps its circular buffer full of useful independent traversals.

Our cooperative decoder has a comparable event: a lane reaches the end of its assigned segment.

It then needs to take another segment, or become inactive if none remain.

We've already built machinery for exactly this.

That means a useful part of AMAC's design philosophy is already present in mibz2.

But there is one interaction worth noticing.

Suppose a lane reaches a segment endpoint just as other lanes are waiting on exceptional payloads.

If replacement work is available, immediately starting a new segment could inject another independent descriptor request.

If replacement is delayed until a group-wide synchronization point, the number of outstanding useful requests could temporarily fall.

This suggests an architectural invariant for any staged implementation:

> A completed segment should be replaced without waiting for unrelated lanes to finish their current lookup stages.

I'm not claiming our current implementation violates this. The coordinator's code audit would be needed to determine the exact behaviour.

I'm saying this is one of the properties that make AMAC robust.

It is likely more important than changing the order in which existing lanes are visited.

### 11. The output-order issue is less difficult than it first appears

There's another detail in the AMAC paper that's unusually relevant.

Database lookups may finish in a different order from the input records.

The authors preserve the original ordering by storing the input record identifier in each lookup's saved state.

That makes result placement independent of completion order.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://homepages.inf.ed.ac.uk\&sz=32)

homepages.inf.ed.ac.uk



mibz2 has an analogous advantage.

Our independent Ψ segments have predetermined output destinations.

We don't need their traversal steps to finish in the same order.

We need each lane to write the correct bytes into its assigned region, and to respect the segment's boundaries.

So an asynchronous payload stage shouldn't require a global output reorder buffer.

The output address is part of the lane state, not something that must be rediscovered from the order of completion.

That's good news.

It means the scheduling irregularity can remain local to each lane.

There is an important correctness condition, however: when a lane is suspended, its output cursor and current Ψ row must not advance prematurely.

The navigation step should be committed exactly once, when its successor and associated output information are available.

That suggests a useful separation between:

- Initiating a navigation step: prefetch descriptor or payload.
- Committing the navigation step: emit its output, advance the cursor and update segment bookkeeping.

This separation is a clean way to avoid accidentally producing duplicate or missing output when a lane is resumed.

It also works naturally with existing CRC and output verification, provided segment buffers remain disjoint and the decoder's final verification covers the resulting bytes.

### 12. The important question I hadn't considered: can we make the slow path predictable before it begins?

This is where the data representation and scheduler start to interact more deeply.

In our current two-tier design, a descriptor tells us whether a payload is required.

But we cannot know which descriptor to inspect until we've calculated the current row.

Once we know that row, however, its group index is simply:

\\[ g=j\gg5. \\]

And the descriptor address is therefore immediately computable.

That gives us an opportunity to separate two useful operations:

1. Prefetch the descriptor immediately upon calculating \\(j\\).
2. When revisiting the lane, inspect the descriptor and, if necessary, prefetch its payload.

That's an important detail because it makes the two-tier lookup a sequence of naturally staged operations:

\\[ \text{Known row} \rightarrow \text{descriptor} \rightarrow \text{optional payload} \rightarrow \text{next row}. \\]

The next descriptor can be requested as soon as its row is known.

There is no need to wait until a different lane completes its own payload stage.

This creates an implementation opportunity: each lane can independently progress through the dependency chain whenever it is visited.

And the scheduler doesn't need to understand FOR, EF, translation or other formats.

It needs only to understand whether the lane has another dependent memory stage to execute.

### 13. A first implementation sketch

Not production Rust, but a model of the desired control flow:

```
enum Stage {
    Descriptor,
    Payload,
}

fn advance_lane(lane: &mut Lane, table: &CompactPsi) {
    match lane.stage {
        Stage::Descriptor => {
            let descriptor = table.descriptor(lane.row);

            if descriptor.is_inline() {
                let next = descriptor.decode(lane.row);

                commit_navigation_step(lane, descriptor.symbol(), next);
                prefetch_descriptor(next);
            } else {
                lane.payload_address =
                    descriptor.payload_address();

                prefetch(lane.payload_address);
                lane.stage = Stage::Payload;
            }
        }

        Stage::Payload => {
            let payload = table.payload(lane.payload_address);
            let next = payload.decode(lane.row);

            commit_navigation_step(lane, payload.symbol(), next);
            prefetch_descriptor(next);

            lane.stage = Stage::Descriptor;
        }
    }
}
```

The real implementation would have to handle segment markers, boundaries, output positions and exceptional cases precisely.

But notice what this sketch does not require.

It doesn't require multiple OS threads per traversal lane.

It doesn't require async I/O.

It doesn't require a coroutine runtime.

It doesn't require tracking completion of hardware prefetches.

It's just a circular array of small state machines.

AMAC's contribution is recognizing that this simple technique can maintain useful memory-level parallelism even when some lookups take more stages than others.

#### There's one important correction to the sketch

A pending payload might need a symbol or marker obtained from the descriptor.

Those fields would have to be preserved in the lane state or recovered without an additional memory access.

We must not silently assume that compacting the pointer leaves output and segment bookkeeping unchanged.

That is one reason the coordinator's marker audit is relevant to this design, not just its bit budget.

### 14. The fifth deduction: don't prefetch data that is already cheap to load

This point emerges from the authors' cache-resident experiments.

We tend to think of prefetching as nearly free.

It isn't.

Prefetch instructions consume instruction bandwidth and may occupy memory-system resources.

And prefetching data that is already in L1 or L2 may yield little benefit.

Consider our two-tier descriptor table.

At approximately 2 bits per Ψ row, a 900,000-row full block has only about 225 KB of primary descriptors.

That's dramatically smaller than the current 3.6 MB T-vector.

The descriptor table could therefore have much better locality than the original navigation table.

But it isn't guaranteed to stay in a private cache.

For example, 225 KB is larger than the i9's 32 KB L1 data cache, and the access order is still driven by a permutation.

Nevertheless, it may be relatively cheap compared with exceptional payload access.

This creates a subtle possibility:

The best staged walker might prefetch payloads aggressively but use ordinary demand loads for descriptors.

Why?

Because software prefetching a descriptor introduces additional instructions even if the descriptor is already readily available from cache.

Meanwhile, the payload is a genuinely additional dependent address that may benefit more from being issued early.

I wouldn't assume this is true.

But it reveals that there are at least three plausible strategies:

| Strategy          | Descriptor access | Payload access       |
| ----------------- | ----------------- | -------------------- |
| Fully prefetched  | Prefetch          | Prefetch             |
| Selective staging | Demand load       | Prefetch and suspend |
| Minimal           | Demand load       | Demand load          |

The surprising point is that the middle strategy could be best.

It retains AMAC's key ability to overlap the exceptional lookup while avoiding the overhead of explicitly staging the common descriptor lookup.

That is a more economical interpretation of AMAC than converting every operation into a separate state.

And it might integrate more naturally with the existing room walker.

### 15. The sixth deduction: we must distinguish latency hiding from cache improvement

This connects back to Seward and our throughput–memory frontier.

The two-tier design potentially helps in two different ways.

First, it reduces working-set size.

A smaller table means fewer distinct cache lines contain the navigation information.

That may improve cache hit rates across concurrent blocks.

Second, staging can overlap exceptional payload latency.

This doesn't reduce the amount of information loaded. It reduces how much of the lookup latency prevents other work from progressing.

These benefits are complementary, but they also interact.

Suppose the compact table becomes so cache-resident that the payload accesses are cheap.

Then staging those accesses may waste instructions.

Alternatively, suppose the compact table is still too large for the relevant cache.

Then staging might matter substantially.

This creates a useful two-dimensional interpretation of performance:

Where AMAC-style staging is most likely to matter

|                       | Regular one-stage lookup      | Irregular two-stage lookup          |
| --------------------- | ----------------------------- | ----------------------------------- |
| Mostly cache-resident | Keep the hot loop simple      | Staging may cost more than it saves |
| Frequent cache misses | Interleave independent chains | Staging has its strongest rationale |

Reasoning from the dependency structure, not measured mibz2 results.

This means a benchmark performed at only one concurrency level could give a misleading answer.

A strategy may behave very differently when one compact block is active than when several are competing for L3.

It also means our new automatic cooperative policy could interact with staging in unexpected ways.

That policy changes the number of live tables.

Staging changes how each core handles memory latency.

These need not have independent effects.

### 16. A quantitative way to think about whether staging can help

We can derive a useful, if simplified, performance condition.

Let:

- \\(f\\) be the fraction of steps requiring a payload.
- \\(L\\) be the latency of a payload access that would otherwise stall progress.
- \\(H\\) be the fraction of that latency successfully hidden by advancing other lanes.
- \\(O\\) be the additional scheduling overhead per Ψ step.

Then the approximate benefit per step from staging is:

\\[ B\approx fHL-O. \\]

Staging is profitable when:

\\[ \boxed{fHL>O}. \\]

This is not a predictive CPU model; it deliberately omits cache contention, instruction overlap, branch prediction and hardware resource limits.

But it exposes an important fact:

A relatively small slow-path fraction can justify staging if those accesses are expensive and enough independent work exists to hide them.

With \\(f\approx0.226\\), even a minority slow path might matter.

On the other hand, if nearly all exceptional payload accesses hit a nearby cache, \\(L\\) may be too small to compensate for the extra state checks.

The model also explains why improving the compressed representation can reduce the benefit of the scheduler.

A smaller or better-localized payload table reduces \\(L\\), making staging less necessary.

So we should not expect staging to be uniformly helpful as compression improves.

### 17. The seventh deduction: more lanes aren't always the answer to a slow path

The AMAC paper is particularly useful here because it examines the saturation point.

If additional lanes merely compete for the same finite miss-handling resources, they don't improve throughput indefinitely.

For mibz2, this suggests there are three fundamentally different ways to improve slow-path behaviour:

Reduce slow-path frequency. More groups answered inline, fewer payloads.

Reduce slow-path latency. Better payload layout, improved cache residency.

Hide slow-path latency. More effective interleaving through staging.

Those three methods can substitute for one another.

For instance, if we can incorporate more nontranslation groups directly into a descriptor, we may reduce the frequency of payload lookups enough that staging becomes unnecessary.

Or if the payloads are extremely compact and cluster well in cache, their latency may already be modest.

Conversely, if the two-tier table's extreme compactness saves enough memory, staging might make the remaining dependencies tolerable.

This leads to a useful research principle:

> Choose the simplest combination of representation and scheduler that achieves the desired outstanding-memory-request throughput.

Not the smallest representation independently.

Not the cleverest scheduler independently.

The smallest combined cost.

That is the algorithmic fixed point we're approaching.

### 18. The eighth deduction: AMAC may improve the construction of the compact design's benchmark, not merely the design

There's one methodological implication I think is easy to underestimate.

Imagine we compare:

A. Our existing 32-bit T-vector walker.

B. A two-tier Ψ walker that immediately performs any required payload load.

Suppose B is 15% slower.

It would be tempting to conclude that the two-tier representation is intrinsically too expensive.

But that wouldn't follow.

We would have measured the combination of the representation and one particular scheduling policy.

The same representation under AMAC-style staging might have different performance.

So a fair assessment of the two-tier architecture should distinguish the cost of the extra dependency from the cost of failing to overlap that dependency.

Conversely, benchmarking only a sophisticated staged walker could unfairly burden simple formats with state-machine overhead they do not require.

That gives us a clean comparison framework:

| Candidate                           | Purpose                                                     |
| ----------------------------------- | ----------------------------------------------------------- |
| Direct T-vector, current walker     | Established baseline                                        |
| Two-tier Ψ, immediate payload       | Measure straightforward lookup cost                         |
| Two-tier Ψ, staged payload          | Isolate the benefit of asynchronous suspension              |
| Fixed-line compact Ψ, simple walker | Compare one-stage compression against the two-tier approach |

This isn't a recommendation to launch four new projects. It's the minimum scientific distinction needed to understand a result if the two-tier prototype is eventually measured.

### 19. The ninth deduction: optimize the moment an address becomes known

I think this may be the most general and durable principle to extract from AMAC.

In a dependent traversal, the critical path is a sequence of moments when new addresses become known:

\\[ \text{address}\_0 \rightarrow \text{value}\_0 \rightarrow \text{address}\_1 \rightarrow \text{value}\_1 \rightarrow\cdots \\]

A good implementation should minimize the interval between learning an address and issuing its memory request.

Everything else—prefetch instructions, lane scheduling, record packing, stage handling—is a means to that end.

For mibz2, this suggests paying attention to what happens between obtaining the next Ψ row and issuing the next record's prefetch.

If symbol extraction, marker processing, output writes or lane bookkeeping are all performed first, we may be delaying a memory request whose address is already known.

It might be profitable to issue that prefetch earlier, then perform the independent housekeeping while the request is in flight.

That is potentially relevant even to the current 32-bit representation.

However, two cautions matter.

First, I haven't inspected the current source ordering, so this isn't a finding that it delays prefetching.

Second, the compiler and processor may reorder independent instructions. The source-code order alone does not establish the actual machine-level critical path.

Still, the correct question is precise:

Once the next navigation address becomes available, how many dependent instructions and how much scheduling work lie before the next memory request can be issued?

That question unifies our direct-table walker, fixed-line FOR/EF and staged two-tier design.

It's the lowest-level form of our performance objective.

### 20. The tenth deduction: compressed Ψ might need fewer lanes than the direct table

This initially sounds counterintuitive.

We might expect extra dependent loads to require more independent lanes.

But cache-residency changes the picture.

Suppose compact Ψ reduces the average memory latency enough that many descriptor accesses hit L2 or L3 rather than requiring trips to DRAM.

Then the amount of independent work needed to hide each access could fall.

A simple latency-hiding approximation is:

\\[ N\_{\text{needed}}\approx\frac{L\_{\text{memory}}}{C\_{\text{lane work}}}, \\]

where \\(C\_{\text{lane work}}\\) is the time between servicing independent lanes.

This is not a hardware resource model, but it gives the right qualitative dependence.

As the relevant memory latency falls, fewer independent lanes may suffice.

Meanwhile, the packed decoding instructions increase \\(C\_{\text{lane work}}\\), potentially reducing the number of lanes needed for latency coverage while increasing CPU computation cost.

So compact Ψ could shift the optimum lane count in either direction depending on which effect dominates.

That's a subtle but important lesson.

The optimal number of traversal lanes may be a property of the representation, not a universal constant.

This also raises the possibility that a compact representation could simplify parts of the scheduler by needing less interleaving to reach its throughput ceiling.

No current measurement establishes that.

But it's a logical consequence of the hardware model worth remembering when interpreting future results.

### 21. The eleventh deduction: output segmentation and AMAC fit together naturally, but correctness requires a clear commit point

One of the most pleasing aspects of the design is that Ψ segments are independently navigable.

That makes them good candidates for asynchronous execution.

But segment markers and output cursors are stateful.

So I would insist on one conceptual invariant:

A lane advances its output position exactly once for every completed Ψ step, never when merely initiating a prefetch.

A pending payload stage must preserve the row and output cursor until the step can be committed.

This gives us a simple state transition:

Pending

Row unchanged; no output emitted

Resolved

Successor and byte known

Commit one navigation step

Emit byte · Advance cursor · Check endpoint · Issue next work

This is not complicated algorithmically, but it gives an important guarantee.

The asynchronous scheduling machinery changes when a lane advances, not what output it produces.

And that means byte-for-byte comparison against the existing decoder remains an appropriate correctness test.

A further implication is that the segment-marker audit and AMAC staging should be considered together. Moving marker information out of table entries may change which fields a suspended lane must retain.

### 22. The twelfth deduction: there may be a very cheap form of AMAC that doesn't look like a state machine

This is the last architectural idea I find worth extracting.

A full AMAC implementation stores each traversal's stage and switches between them.

But perhaps mibz2 needs only a small variation of its current walker.

Imagine preserving the direct fast loop and adding a small collection of pending exceptional payloads.

When an exceptional descriptor is encountered:

1. Prefetch its payload.
2. Record the lane as payload-pending.
3. Continue with another lane.
4. Resolve the payload on a later visit.

This is functionally a state machine, but it needn't require a large abstraction or a fully general task scheduler.

It could amount to one extra stage bit plus the pending address in each lane's existing state.

The important difference from a generic coroutine engine is that we know the complete set of possible suspension points ahead of time.

That allows highly specialized code.

However, maintaining a separate queue of ready lanes may not be worthwhile. The AMAC paper itself achieves its benefits with a circular buffer and simple stage dispatch.

So the likely starting point is actually very conservative:

Keep the circular scheduling pattern; allow some lanes to be pending a payload.

There is no reason to assume anything more elaborate will help until this simplest model has been evaluated.

### 23. Where the deductions converge

We've now extracted essentially every transferable idea I can justify from the paper and its related database systems literature without seeing new measurements.

Here's the resulting synthesis.

| Principle                                                | AMAC evidence                 | mibz2 implication                                                        |
| -------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------ |
| Interleave independent traversals                        | Demonstrated                  | Already central to our successful decoder                                |
| Preserve independent stages                              | Demonstrated                  | Relevant if compact Ψ has dependent payloads                             |
| Use a circular state buffer                              | Demonstrated                  | Existing room scheduling may be a good foundation                        |
| Refill useful memory work promptly                       | Demonstrated                  | Prefetch each newly known address early                                  |
| Keep state overhead small                                | Demonstrated                  | Avoid general coroutine machinery unless it earns its cost               |
| Extra lanes have diminishing returns                     | Demonstrated                  | Lane count is representation- and hardware-dependent                     |
| Cache-resident workloads can be instruction-sensitive    | Demonstrated                  | Don't add staging overhead indiscriminately                              |
| Immediate versus staged payloads may perform differently | Inference                     | Compare them if two-tier Ψ is implemented                                |
| Exceptional stage occupancy may differ from frequency    | Inference                     | Measure pending-lane distribution, not merely exceptional-row percentage |
| Output can remain correct despite asynchronous progress  | Supported by analogous method | Keep a precise per-lane commit invariant                                 |

The most important conclusion is that AMAC doesn't tell us to abandon the existing room walker or redesign cooperative decoding.

It tells us how to extend its execution model in the presence of irregular compressed navigation.

### 24. The fixed point: what we know, what we can infer, and what requires evidence

I think we've reached a useful stopping point in the deduction itself.

There are three levels of certainty.

Established

Paper + mibz2 results

Interleaving independent dependent-memory traversals is effective. Small per-traversal state machines can preserve parallel progress through irregular lookup paths. Their overhead matters, especially when data is cache-resident.

Strong architectural inference

Not yet measured

A compact Ψ walker could suspend lanes after an exceptional payload prefetch, allowing other lanes to continue. Such staging might make two-tier compression substantially more competitive without changing its stored representation.

Requires direct evidence

Open

Whether staging improves mibz2's actual throughput; whether payloads miss cache often enough to justify it; whether 32 lanes remain optimal; and whether the combined footprint-plus-scheduling improvement beats fixed-line compression or the current direct T-vector.

There is no honest way to deduce those last answers solely from AMAC's database benchmarks.

Its pointer chains, caches, hardware and data distributions differ from ours.

The remaining questions are empirical.

And importantly, the full cooperative decoder has just passed a major validation milestone. None of this provides evidence that its existing regular walker should be changed before the performance release.

### The most useful result of this entire excursion

We began by wondering whether AMAC meant we were wrong to interleave Ψ walks.

We've ended somewhere much more interesting.

Our current decoder is already exploiting the same fundamental resource as AMAC: independent traversals that allow a processor to make progress while unpredictable memory requests are outstanding.

The new possibility is that future compressed navigation may need a richer notion of progress.

For an uncompressed T-vector, one stage is enough: load a pointer, then continue.

For a compact two-tier Ψ representation, progress might mean either obtaining the next row or discovering and initiating the payload fetch needed to obtain it.

That creates a natural two-stage state machine.

And the major insight is this:

> Compression should be evaluated by the cost of advancing a whole population of dependent traversals, not merely by the latency of decoding one isolated pointer.

That unifies nearly everything we've been studying: Seward's memory trade-offs, direct Ψ navigation, cache-line Elias–Fano, our cooperative walker, shared-cache residency and AMAC.

The best representation may be slightly more expensive to query in isolation but faster when the scheduler can overlap its irregular work across many lanes.

Equally, a slightly larger one-stage representation may win by avoiding the extra dependency altogether.

That is the real design choice AMAC has exposed for mibz2.

We don't need to decide it theoretically, and we shouldn't expect the paper's results to decide it for us.

But we now understand exactly what the implementation comparison would have to distinguish—and why a naive compressed-pointer benchmark could otherwise lead us to the wrong conclusion.
