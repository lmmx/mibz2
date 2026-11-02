# External research report 154: the researcher's theoretical review of P87 (candidate 1): the source-level correctness argument, the memory accounting, the existing CI production timing, and the validation design (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with reports 153, 155 and 156.
- **Source:** the independent researcher of reports 138–153.
- **In reply to:** the maintainer's instruction, quoted below.

> ok fine 1,2,3. do 1 first, contemplate it in depth and try to resolve what can be resolved by thinking hard and doing theoretical inquiries (including research online if appropriate) to solidify it. only number 1.

**Evidence cutoff:** 2 November 2026. Source review of PR #128 at `0c121c5` and an audit of
its existing CI artifact; no new i9 run, compilation or decoder benchmark.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
**P87 now has a solid source-level correctness argument and a precise memory-saving argument. Its performance case is also stronger than I previously reported: the production candidate already has a positive automated CI benchmark.**

What remains is narrower than “does buffer reuse make sense?” It is whether this particular production change delivers a worthwhile benefit on the target machine while satisfying the regression controls.

I reviewed PR #128 at **`0c121c588a44c5fd41c40c46ab18ccc7a49478bd`**, its original diagnostic, the relevant Rust implementation, and primary documentation. I also downloaded and checked the existing CI artifact; I did not run a new decoder benchmark.

## 1. An important correction: the production candidate has already been timed on CI

My earlier description—“the production candidate exists and is untimed”—was incomplete. **It remains untimed in the proposed production-shaped i9 validation, but GitHub Actions already benchmarked it against its production base.**

That run compared candidate `0c121c5` with base `13e9854`, using ten paired rounds. Its relevant Wikidata results were:

| Configuration | Wall-time change | CPU-cost change | Peak RSS |
|---|---:|---:|---:|
| One thread | +0.2% [−0.5%, +1.0%] | +0.2% [−0.5%, +1.0%] | 26 → 25 MB |
| Two threads | −0.8% [−2.3%, +0.7%] | −0.9% [−2.0%, +0.2%] | 50 → 46 MB |
| Four threads | **−1.9% [−3.2%, −0.7%]** | **−2.3% [−3.4%, −1.3%]** | **74 → 69 MB** |

These are paired estimates, not ratios calculated from the rounded medians. Importantly, the four-thread configuration used **two physical EPYC cores with SMT**, not four physical i9 cores. 

I independently checked the artifact’s SHA-256 against GitHub’s recorded digest, verified its **420 measurement rows and 210 matched pairs**, checked output-hash and output-size agreement, and reproduced the principal point estimates. The confidence intervals above are the published CI analysis’s intervals, rather than a fresh independent reconstruction of its interval procedure.

This adds useful evidence:

> We have both a positive i9 **diagnostic** comparison and a positive CI **production-candidate** comparison, at different hardware and concurrency boundaries.

They should not be pooled into one speedup estimate. Nor does CI settle the no-harm requirements: for example, its four-thread `records` wall-time interval still permits a regression of several percent. But P87 is no longer supported solely by a diagnostic implementation.  

## 2. Precisely what P87 changes

The existing segmented path is conceptually:

```text
MTF produces last column in ll
    → build the T-vector from ll
    → walk the T-vector into segment buffers
    → assemble segments into rle
    → expand RLE1 from rle into final output
    → check the final output's CRC
```

P87 changes the assembly destination and subsequent RLE1 input:

```text
MTF produces last column in ll
    → build the T-vector from ll
    → walk the T-vector into segment buffers
    → assemble segments over the old contents of ll
    → expand RLE1 from ll into final output
    → check the final output's CRC
```

The production patch leaves the table construction, traversal, segment buffers and assembly algorithm in place. 

**This is storage reuse, not copy elimination.** For a segmented block with \(n\) BWT rows, assembly still reads and writes \(n\) bytes. RLE1 still processes the reconstructed stream. The final output is still a separate allocation.

That corrects an ambiguity in my earlier theoretical summary: P87 does not earn a saving by deleting the assembly copy. It can earn a saving by making the existing work touch fewer distinct locations, and by avoiding the separate destination allocation and its growth.

## 3. The correctness argument can be resolved without another performance experiment

### The decisive property: the table owns everything traversal needs

The production implementation of `Segmented::invert_in_place` is essentially:

```rust
build(ll, counts, &mut self.tt);
walk_segments_room(&mut self.tt, orig_ptr, ll, &mut self.seg);
```

The T-vector is an owned `Vec<u32>`. Construction copies the necessary byte and successor information into it; it does not retain references into `ll`. The ordinary segmented traversal then receives the table and segment scratch, not the original last-column slice.  

That gives a straightforward equivalence argument.

Let \(L\) be the original last column, \(T=B(L)\) the constructed table, and \(R=W(T)\) the reconstructed RLE1-coded stream. The old implementation stores \(R\) in a separate vector. The new implementation first computes the same \(T\), then stores the same \(R\) over \(L\).

Once \(B(L)\) has finished, the old contents of \(L\) are no longer an input to \(W\). Therefore, **changing where \(R\) is stored does not change its value**, provided the destination remains separate from the table and segment buffers. The source satisfies that separation through distinct owned vectors.

This is a proof of the *transformation*, assuming the existing builder and walker fulfil their contracts. It is not necessary to re-prove the entire inverse-BWT algorithm.

### The destination already has enough capacity

The scalar and AVX2 MTF paths both clear `ll` and reserve the block’s permitted maximum length before producing its contents. On successful decoding:

\[
n=\operatorname{len}(ll)\leq\operatorname{capacity}(ll).
\]

The inverse BWT produces exactly \(n\) bytes before RLE1 expansion. The plain walker performs \(n\) output steps; segmented assembly stops when its destination reaches \(n\), truncating the final segment contribution where necessary. Thus:

\[
\boxed{\text{inverse-BWT output length}\leq\text{existing }ll\text{ capacity}.}
\]

No larger destination is required by the transformation.  

Rust’s `Vec::clear` retains capacity, and `reserve` does nothing when capacity is already sufficient. This supports the reuse argument without introducing a custom allocator or manual memory-management scheme. [doc.rust-lang.org](https://doc.rust-lang.org/std/vec/struct.Vec.html)

Crucially, \(n\) is the **BWT-row count**, not the final decoded output length. RLE1 expansion happens later into `out`; P87 is not trying to fit the expanded output into the last-column buffer.

### Short blocks and periodic data do not require a new algorithm

For short blocks, the same wrapper uses the plain table walk. That walk also reads only the table and writes exactly \(n\) bytes.

For segmented blocks, P87 preserves the existing assembly semantics, including revisiting a segment cycle and stopping at the required block length. Its correctness therefore does not depend on assuming that every valid BWT has a single cycle or that every segment is assembled exactly once.  

### Error handling remains consistent

The code checks an invalid primary index **before** invoking in-place inversion. After inversion, RLE1 reads the selected destination, and CRC is calculated over the **expanded final output**, not over `ll` or `rle`.

An RLE1 error or CRC mismatch truncates the caller’s output back to its previous length, just as before. On the next block, MTF clears and rewrites the scratch last-column buffer. Its previous contents are not needed for recovery. 

**My conclusion:** I found no unresolved algorithmic obstacle to overwriting `ll` at this boundary. The source-level transformation is well justified, and the patch introduces no new `unsafe` block.

## 4. The memory benefit is more precise than “about 0.9 MB saved”

There are three different quantities to keep separate.

### A. The eliminated allocation

For a fresh `Scratch<Segmented>`, `rle` starts as an empty vector. The new segmented path always succeeds through `invert_in_place`, and RLE1 then reads `ll`. Nothing in that path needs to allocate `rle`.

That property persists across blocks: it is an invariant of this implementation, rather than a fortunate outcome on one input. The added tests already check zero `rle` capacity across a change from level 1 to level 9.  

The directly removed reservation is therefore:

\[
\text{the capacity that the old implementation would retain in }rle.
\]

That may exceed the largest block’s actual length because of vector growth. Rust does not promise a particular growth factor, so “exactly 900,000 bytes per worker” is not a universal allocation claim. [doc.rust-lang.org](https://doc.rust-lang.org/std/vec/struct.Vec.html)

The `rle` **field** remains in `Scratch` for other inverse-BWT implementations. P87 avoids its backing allocation for `Segmented`; it does not remove every byte of metadata associated with the field.

### B. Distinct storage touched during a block

Ignoring cache-line rounding, the old path touches two separate \(n\)-byte regions for the last column and reconstructed stream. P87 uses one region for both roles.

Thus the logical reduction in their combined address footprint is approximately:

\[
n \text{ bytes per worker}.
\]

For four full 900,000-row blocks, that is approximately **3.6 MB less distinct intermediate storage** across the four workers. This is derived storage arithmetic, not a measurement of cache occupancy.

### C. Peak RSS

Peak RSS includes allocator behaviour, page residency, other buffers and when their peaks coincide. It need not fall by exactly the eliminated vector capacity.

We now have direct evidence that RSS did fall in the production candidate’s CI run. But its roughly 6.4% four-thread reduction should remain a measurement of that configuration, not a promised percentage for the i9 or every workload. 

**The strongest portable claim is that a redundant intermediate allocation is avoided.** The exact process-memory benefit remains configuration-dependent.

## 5. Why storage reuse can help even though the same bytes are copied

There is a useful theoretical result here, followed by an important limit.

### A limited cache-model result

Consider an idealised model with:

- A fixed sequence of memory references.
- A fully associative cache using least-recently-used replacement, starting empty.
- Corresponding lines of `ll` and `rle` mapped to the same line identity.
- All other addresses unchanged.

For any reference, define its *reuse distance* as the number of distinct lines accessed since the preceding reference to that line.

Merging the two buffers cannot increase that distance. The previous reference to the merged line may become more recent, and mapping two line identities to one cannot increase the number of distinct intervening identities.

Consequently, **this address coalescing cannot increase cache misses in that idealised LRU model**.

That is a genuine theoretical argument for the direction of the locality benefit. It does not depend on guessing a particular number of requests in flight.

However, real execution does not satisfy all those assumptions. Caches have limited associativity and different replacement behaviour; allocation changes can move other objects; prefetching and execution order can change. The model therefore supports the mechanism, **not a guarantee of faster execution on real hardware**.

### A concrete possible traffic saving

Suppose a cache line containing the last column remains cached and modified after construction has read it. P87 can overwrite that line with reconstructed bytes before its eventual eviction.

In that case, the processor need not write the obsolete last-column version to memory separately from the reconstructed version. It may also avoid obtaining a different destination line for the assembly store. Intel’s documentation describes the relevant read-for-ownership and locality effects for cached writes. [Intel CDRD](https://cdrdv2-public.intel.com/671488/248966-Software-Optimization-Manual-V1-048.pdf)

An illustrative favourable scenario can therefore save approximately one line read and one line write per reused line. Across a block, that is on the order of \(n\) bytes in each direction.

**That is a scenario, not a fixed saving.** If the last-column lines have already been evicted, the opportunity is different. Other buffers’ traffic can also change indirectly.

The diagnostic’s approximately **0.90 fewer DRAM-read bytes and 0.87 fewer write bytes per decoded byte** are consistent with a buffer-sized effect. They do not identify which lines generated the difference or prove that this traffic reduction caused the CPU saving. 

This resolves the apparent paradox: **the program can issue essentially the same useful loads and stores while the memory system performs less expensive work servicing them.**

## 6. The original diagnostic and the production candidate answer different questions

The diagnostic deliberately allocated and prefaulted **both** buffers in both arms. Its reuse arm kept `rle` allocated but stopped writing it after initialisation. Both arms used one compiled execution path, with the destination chosen at runtime. 

That design was useful because it isolated the consequences of changing the destination while controlling allocation and code differences.

The production candidate does something additional: **it never creates the unnecessary `rle` backing allocation for `Segmented`**. It also removes the diagnostic’s runtime selection machinery.

Therefore:

- The diagnostic gives evidence about address reuse under its controlled setup.
- The production candidate tests the actual engineering package: address reuse, avoided allocation and the resulting compiled code.

The production version is not mathematically guaranteed to be faster than the diagnostic reuse arm. Avoiding an allocation changes subsequent placement and initialisation behaviour; code layout can change too. Performance-measurement research specifically identifies memory placement and binary differences as sources of variability that must be considered. [School of Computing - University of Kent](https://www.cs.kent.ac.uk/pubs/2012/3233/)

Nevertheless, the newly recovered CI result is encouraging because it evaluates that production package rather than merely repeating the controlled diagnostic.

## 7. A newly identified reporting limitation: CI’s stage table does not exercise P87

This needs an explicit annotation in the record.

At the candidate pin, `examples/phases.rs` constructs separate vectors named `ll` and `walked`. It builds from `ll`, assembles into `walked`, and runs RLE1 from `walked`.

It does **not** call the new `invert_in_place` method. 

Consequently, the CI report’s stage-level changes—including its reported RLE1 slowdown—cannot be interpreted as effects of P87’s buffer reuse. Those timings describe the separate-buffer stage harness in the two builds.

This does not invalidate the whole-decoder benchmark, which exercises the changed production path. It means:

> **Use the CI whole-decoder results as supporting evidence. Do not use its current stage table to explain where P87 gains or loses time.**

There is no need to expand P87 into a stage-instrumentation project to resolve this. An annotation is sufficient unless phase attribution becomes necessary for the final decision.

## 8. What I would tighten before production validation

The implementation does not need redesign. I would concentrate on a small set of explicit contracts and differential checks.

**First, compare the same segmented implementation with and without reuse.** The new integration tests often compare `Segmented` against `Plain`, which is valuable independent output validation. But the most direct test of this transformation is `Segmented::invert` into a separate vector versus `Segmented::invert_in_place`, starting from the same last column and counts. That holds the algorithm constant and changes only the storage lifetime. The existing two methods make this possible without introducing another performance candidate.  

**Second, make the successful in-place contract explicit.** A `true` result should mean that `ll` contains exactly the bytes the separate-output method would produce, with the corresponding output length. A `false` result must leave the inputs and relevant kernel state suitable for the fallback call. The current documentation already promises no change on `false`; spelling out the success condition would make future implementations safer to review. 

**Third, check the lifetime invariant directly.** Around an inversion call, record the destination pointer, capacity and length; establish that the existing capacity suffices and that `rle` remains unallocated. Exercise short and long blocks, increasing and decreasing stream levels, periodic data, and scratch reuse after an error. The current tests cover much of this already; the remaining task is to make the particular invariant visible rather than relying only on final output hashes.

**Fourth, audit the actual production binary.** The intended change should not introduce a new per-row branch, a duplicate table construction or a different traversal kernel. The new selection is expressed at block level in generic code, but source shape alone does not establish the compiler’s final output. The original diagnostic’s code-equivalence checks do not automatically certify this different production binary.  

These are bounded preparation tasks, not reasons to explore another implementation family.

## 9. The remaining empirical question can be made much smaller

The necessary engineering question is:

> **Against the shipped decoder, does this production change improve the primary four-core workload, preserve the required controls, and avoid a memory regression?**

It does **not** require first proving a specific cache mechanism or resolving the original diagnostic’s 0.10 CPU-s/GB interaction threshold.

The original protocol deliberately separated mechanism from engineering. Its engineering rule required a resolved primary improvement in both throughput and CPU cost, passing no-harm controls, and passing RSS checks. The mechanism could remain inconclusive. 

### The main design issue is precision in the controls

The original i9 diagnostic’s controls decoded only 8–10 MB, producing short measurements with intervals too wide for the required 1% no-harm limits. That was a design limitation, not evidence that P87 necessarily harmed them. 

A simple calculation shows why “use another twenty rounds” is not an adequate plan. Under an independent normal approximation,

\[
N \approx \left(\frac{1.96\,s}{h}\right)^2,
\]

where \(s\) is the standard deviation of paired log ratios and \(h\) is the desired confidence-interval half-width.

At an illustrative 6–8% paired variability, merely reaching a nominal 1% half-width takes approximately **140–250 independent pairs**. That is not a powered design for passing every control, and correlation could make the requirement worse. The calculation illustrates why measurement duration and variance need attention before choosing a round count.

The better preparation is to size longer, representative control workloads and establish their expected precision. Repeatedly walking one already-warm block would answer a different question.

Also, the existing no-harm rule is directional: **CPU must not rise beyond its tolerance, and throughput must not fall beyond its tolerance**. It is not a requirement that beneficial changes remain within a symmetric ±1% band. That distinction should be preserved exactly. 

For an optimization of this size, I would keep the final validation focused on normal decoder timing, aggregate process CPU, correctness and memory. Another extensive counter campaign is not a prerequisite.

## 10. What is now settled, and what remains open

| Question | Assessment after this review |
|---|---|
| Is the old last column needed when P87 overwrites it? | **No, in the reviewed `Segmented` path.** Construction has transferred the necessary information into owned table storage. |
| Does inverse-BWT output require a larger destination than `ll` already holds? | **No.** Its length is the original BWT-row count. |
| Does P87 remove the assembly copy? | **No.** It changes its destination and reuses storage. |
| Can `Segmented` avoid the separate `rle` allocation throughout scratch reuse? | **Yes, by the reviewed control flow.** |
| Is there a defensible theoretical locality mechanism? | **Yes**, including a non-increasing-miss argument under an explicitly simplified cache model. |
| Is the production candidate wholly unmeasured? | **No.** The existing CI run supplies positive production-shaped evidence. |
| Is the exact i9 production speedup established? | **Not yet.** |
| Are all required no-harm conditions established? | **Not yet.** |
| Must the physical cause of the saving be isolated before promotion? | **No.** A valid engineering result can stand without that stronger causal claim. |

**My updated judgment is that P87 is theoretically ready for a bounded production decision.** Its remaining uncertainty is predominantly empirical, and the existing CI evidence strengthens the case for completing that decision.

I would preserve the current implementation, add the focused contract and differential checks, annotate the misleading stage attribution, and finish a properly sized production-validation protocol. I would not spend further research effort trying to derive an exact speedup from cache capacity or traffic totals.

The supporting reanalysis is available as an independent-ci-audit.json[independent CI audit](sandbox:/mnt/data/p87_review/independent-ci-audit.json), with its audit_ci.py[read-only reproduction script](sandbox:/mnt/data/p87_review/audit_ci.py) and the p87-ci-0c121c5.zip[original CI artifact](sandbox:/mnt/data/p87-ci-0c121c5.zip). No new i9 run, candidate compilation or decoder benchmark was performed for this review.
`````
