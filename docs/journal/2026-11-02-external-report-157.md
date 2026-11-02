# External research report 157: the guardian on reports 154–156: the 30-entry layout first for Astra; P87's allocation-ledger test failure; wake placement as a testable policy; two of the coordinator's readings too strong (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** reports 154–156 and the coordinator's reconciliation of report 152.

**Evidence cutoff:** 2 November 2026. Independent checks of the address identities, the
storage bound and PR #128's CI; no new run.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
**This was productive theoretical work. The third report, in particular, changes what I would ask Astra to assess tomorrow.** We now have more than a proposal to optimize the existing 14-bit extraction: we have a concrete way to remove that extraction sequence while retaining a substantial storage advantage.

The three reviews have also made the candidates more distinct:

| Candidate | What the new work adds | Remaining decision |
|---|---|---|
| **P87** | A stronger correctness and storage argument, plus previously overlooked production-candidate CI measurements | Does it pass a properly designed production validation on the target configuration? |
| **Wake placement** | One specific policy and a better definition of recoverable capacity | Does eliminating worker-on-worker placement actually improve whole-decoder performance? |
| **Compact navigation** | A new direct-access layout, coordinate transformation and conservative storage bound | Does the simpler access primitive survive independent review, implementation and the construction budget? |

**I would keep those distinctions rather than force all three into one ranking.** P87 is closest to a deployment decision. Wake placement is now a specific systems hypothesis. The new compact layout is the most consequential architectural development in this batch.

## 1. The 30-entry layout deserves Astra’s attention first

The important advance is not merely changing 14-bit fields to 16-bit fields.

It is the combination of three changes:

**Use two field values to identify exceptional cases, eliminating the start bitmap. Reduce the header to four bytes. Relabel rows so the wider fields remain directly addressable.**

That produces the exact layout:

```text
One aligned 64-byte record

Bytes  0 ................................ 59 | 60 ...... 63
      ┌─────────────────────────────────────┬─────────────┐
      │ 30 directly loadable 16-bit offsets │ 32-bit     │
      │                                     │ header      │
      └─────────────────────────────────────┴─────────────┘

      30 × 2 bytes                          + 4 bytes
      = 64 bytes
```

Under the researcher’s proposal, `0xFFFE` identifies a segment start and `0xFFFF` identifies RAW or parking. Ordinary offsets use the remaining values. This moves information previously held in the bitmap into the field the walker already reads.

**That is a structural improvement to the access operation—not another attempt to persuade the compiler to emit a better variable shift.**

### The relabelling is what makes it interesting

Simply using 30 entries per record would normally introduce division or equivalent arithmetic to locate a record. The proposed coordinate system avoids that:

\[
\phi(r)=32\left\lfloor\frac r{30}\right\rfloor+(r\bmod30).
\]

For an encoded row \(x=\phi(r)\):

\[
\begin{aligned}
\text{offset address}&=\text{records}+2x,\\
\text{header address}&=\text{records}+2(x\mathbin{\&}\sim31)+60.
\end{aligned}
\]

The address gaps correspond to the header’s two halfword positions. All links remain in encoded coordinates, so navigation does not convert back to original row numbers at every step.

**I independently checked these address identities across all 900,000 live rows**, including the inverse mapping, avoidance of header positions, same-line placement of field and header, and the 20-bit label limit. The arithmetic checks out. That is a check of the proposed coordinate scheme—not decoder correctness or performance.

It replaces this recurring operation sequence:

```text
slot → multiply by 14 → byte offset and bit offset
     → word load → variable shift → mask
```

with:

```text
encoded row → directly addressed halfword load
```

The header load and packed-token addition remain. It is still a two-table-load lookup, not the shipped table’s single-entry load.

### The storage bound is a substantive improvement

I also independently reproduced the numerical bounds.

For a full block:

\[
G=30{,}000,\qquad M=959{,}997,\qquad
\left\lfloor\frac{M}{65{,}534}\right\rfloor=14.
\]

Using the researcher’s argument:

\[
R\leq15\sigma-1,
\qquad
\text{bytes}\leq64(G+1)+128R.
\]

The resulting figures are:

| Assumption | Maximum RAW groups | Maximum logical table bytes | Reduction versus 3.6 MB |
|---|---:|---:|---:|
| At most 98 used characters | 1,469 | **2,108,096** | **41.4%** |
| Any byte alphabet, at most 256 characters | 3,839 | **2,411,456** | **33.0%** |

The proof uses the ordering of successors within a first-column character bucket. The existing builder’s stable counting scatter supplies precisely that ordering: it processes input positions in order and advances a separate destination cursor for each character. 

The bound’s structure looks sound under its stated assumptions: bucket-crossing groups are bounded by the number of bucket boundaries, and wide homogeneous groups consume disjoint spans within each bucket.

**This is stronger than assuming the old 1.9% exception rate carries over.** It means the design can retain a meaningful logical storage advantage even under a conservative exception count.

But the qualifications still matter:

- It bounds **logical table storage**, not peak RSS or retained vector capacity.
- It does not bound the execution cost of exceptional visits.
- It does not establish the proposed roughly 20-instruction loop.
- A bounded-memory direct constructor is not necessarily a cheap constructor.

Those are the remaining questions—not reasons to dismiss the mathematical progress.

### Is this another endless format variation?

**Not inherently.** It addresses the specific cost exposed by the failed implementation: extracting individual packed fields during dependent navigation.

The justification is substantially better than “try a slightly different bit width.” It gives us an explicit trade: surrender some compression to remove multiplication, variable shifting and masking from the ordinary lookup.

Nevertheless, I would treat it as **one new, separately identified design**, not permission for a family of width, group-size and prefetch sweeps. The frozen FOR-14 result remains unchanged, and `k`/`j` remain useful compiled comparators.

## 2. P87 has stronger evidence—but I found one integration issue

The researcher is right to correct our earlier description of the production candidate as wholly untimed.

I checked the published CI benchmark report for candidate `0c121c5` against `13e9854`. It reports the quoted four-thread Wikidata results: **wall time −1.9%, CPU cost −2.3%, and lower RSS**. It also explicitly identifies the configuration as **four logical CPUs on two physical cores with SMT**, not four physical i9 cores. 

So the accurate position is:

> **P87 has positive diagnostic evidence on the i9 and positive production-candidate evidence on a different CI configuration. Target-machine production validation remains outstanding.**

I confirmed the published report; I have not repeated the researcher’s separate raw-artifact audit.

### A concrete issue before production validation

The associated Rust test job was not completely green.

In Actions test run `41352595637`, job `124985814761`, I found this failure:

```text
trace::tests::ledger_matches_a_workers_buffers

left:  0
right: 140004
```

The source assertion checks `rle` capacity against the old allocation expectation. The reviewed source still expects that buffer to exist in its allocation ledger test. 

That is consistent with P87 successfully avoiding the allocation, rather than an output-corruption failure. **But the ledger and its test must be reconciled with the new storage lifetime; the failure should not simply be waived.**

This is a bounded integration task, not an argument for redesigning P87.

The researcher’s other distinction is also important: **P87 removes a backing allocation, not the assembly copy.** Its source-level correctness argument and simplified cache argument support the transformation, while the exact throughput and RSS effects remain empirical.

I would regard P87 as close to a production-validation decision once those contracts, accounting and controls are finished. I would not require it to explain the original memory-system penalty first.

And I would **not widen the no-harm tolerances because the earlier controls were imprecise**. Longer, representative measurements are the appropriate way to resolve that uncertainty unless the project independently changes its product requirements.

## 3. Wake placement is now a testable policy, not a broad aspiration

The researcher has made a useful transition from “improve scheduling” to:

> Four decoding workers each receive a distinct singleton CPU mask; scanner, feeder and reader retain access to the original four-core set.

Linux affinity constrains the CPUs on which each thread may run. Under successfully verified distinct singleton masks, one decoding worker cannot be placed on another decoding worker’s assigned CPU. That gives the policy a clear property to test. [man7.org](https://man7.org/linux/man-pages/man2/sched_setaffinity.2.html)

But those are **not exclusive cores**: support threads remain eligible to execute there. Nor is the policy necessarily work-conserving. A ready worker can be stuck behind support work on its assigned core while another permitted core is idle.

That is why the researcher’s capacity-aware accounting is a genuine improvement:

\[
H(t)=\min(I(t),R(t),Q(t))
\]

under the stated simplifying assumptions, with eligibility matching required when masks differ.

It avoids treating several waiting workers as several recoverable cores when only one CPU—or one job—is available.

**The 5.65% observation remains useful evidence, not a speedup budget.** I would prefer this refined accounting of the existing traces before implementing a placement policy.

The condition-variable analysis is also useful, but should stay separate. A shared condition variable serves different predicates; waking only one unspecified waiter can wake the wrong role. The researcher’s counterexample to a blind `notify_all()`→`notify_one()` substitution is sound under the stated state sequence. Rust’s API requires the predicate to be checked under the mutex and rechecked after wake-up; notifications do not select a waiter by application predicate. [Rust Documentation](https://doc.rust-lang.org/std/sync/struct.Condvar.html)

**Do not combine affinity, notification changes and queue replacement into one experiment.** That would discard the interpretability the new policy provides.

## 4. Claude’s additional reading is useful, but two conclusions remain too strong

### The 36% stage/full-decoder gap is not a measured wake-placement opportunity

As described in Claude’s note, the comparison changes both **measurement boundary** and **placement**: pinned stages plus MTF versus an unpinned full decoder.

That gap includes work omitted by the stage harness as well as possible pipeline and placement effects. It cannot be assigned to wake placement, and the dispatch study’s 5.65% cannot simply be treated as a measured subset of the same numerical gap.

It is a reason to investigate the integration boundary—not an established 36% recoverable gain.

### One L3-miss share will not cleanly separate capacity from request servicing

The additional pending-cycle comparison is interesting. It supports the descriptive statement that demand-associated outstanding misses occupy a larger share of the prefetched kernels’ execution under concurrency.

It does **not** establish that every extra cycle is a load stall. Intel’s pending-cycle event records cycles with misses outstanding, not cycles during which no useful execution occurs. [GitHub](https://raw.githubusercontent.com/intel/perfmon/main/SKL/events/skylake_core.json)

Likewise, a falling `fb_full` rate **per cycle** need not mean fewer such events per row: the denominator has grown. Both normalizations matter.

Most importantly, measuring `MEM_LOAD_RETIRED.L3_MISS` under four copies would not, by itself, provide an unbiased account of all navigation traffic. It counts retired loads missing L3; Intel’s mixed offcore-read event includes demand and prefetch traffic, which is a different population. [GitHub](https://raw.githubusercontent.com/intel/perfmon/main/SKL/events/skylake_core.json)

So I would not commission a new capture on the premise that **one L3-hit ratio settles capacity versus service latency**. It could add evidence, but not the clean discriminator Claude’s wording suggests.

The new layout offers a more direct engineering question anyway: **can we make the lookup much cheaper while keeping a provably smaller table?**

## 5. What I would do next

This changes tomorrow’s Astra brief.

Previously, the immediate question was whether the compiled `k` and `j` changes justified a measurement. Now there is a new competitor that removes the remaining extraction sequence rather than optimizing it.

**I would ask Astra to review the 30-entry design before choosing the next compact-navigation experiment.** Not to reopen the whole portfolio, and not to implement anything.

The review should settle four precise points:

1. **Encoding correctness:** two sentinels, RAW parking, carried-byte orientation, encoded segment starts, partial groups and restoration.
2. **Storage proof:** the disjoint-span argument, boundary-group count and every byte omitted from the logical-size calculation.
3. **Execution feasibility:** whether direct halfword addressing and packed-state addition plausibly compile into the intended cheap path without hidden conversions or dependent metadata.
4. **Investment choice:** whether to measure existing `k`/`j` first, prepare one compile-only prototype of the 30-entry design, or stop compact-navigation work.

The new design should not automatically win merely because its paper instruction count is lower. But **it deserves to be considered before we spend the next experiment answering a narrower question about an extractor we may be able to eliminate.**

Meanwhile, P87 has a small, identifiable completion task—including its allocation-ledger failure—and wake placement has a specific policy plus an existing-trace accounting task. Neither needs a new i9 run simply to advance that preparation.

## Bottom line

**The researcher has materially improved all three candidates, not just restated the shortlist.**

P87 is better supported empirically and more precisely justified. Wake placement now has a provable intervention property and a less inflated opportunity model. Compact navigation has gained a genuinely different access primitive with a conservative storage bound.

For your larger-gain objective, **the 30-entry compact layout is the most important new item to put before Astra**. P87 remains the most mature modest improvement; wake placement remains the strongest distinct utilization hypothesis.

Nothing needs running on the workstation now. The next decision is whether the new representation survives a focused adversarial review strongly enough to earn **one bounded implementation step**.
`````
