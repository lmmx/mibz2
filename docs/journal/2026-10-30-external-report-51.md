# External research report 51: what the campaign salvages (verbatim)

A relayed message from the maintainer, unattributed in the relay, on what to keep from the
round.

**What it says:**

| Finding | What survives | The message's judgement |
|---|---|---|
| The one-pass BWT build (#114) | Shipped. Measured about 6% faster in the build stage on one CI runner | A concrete success |
| P74, the register-held MTF | The whole checkpoint MTF stage 19–33% faster on Wikidata, across two runners | The strongest unrealised opportunity |
| P74's online switch (option C) | A way to avoid the failed selector's whole-block census | Worth preserving, not yet proven |
| N1, partial depth | Fewer dependent lookups, with less construction than full Ψ⁸ | Mathematically valuable; economics doubtful |
| N1's compact navigation table | A possibly smaller working set during the walk | A useful idea; the cache benefit is unmeasured |
| The benchmarking methodology | Same-binary comparisons, negative controls, layout checks, CPU identification, retained-memory accounting | Durable engineering value |

**The recommendation:**
- **Keep:**
  - the shipped one-pass build;
  - P74's verified kernel;
  - option C's design notes, as a dormant candidate: no implementation now, revisited
    only if an end-to-end profile shows MTF is still worth attacking;
  - the corrected bench infrastructure.
- **Archive:** N1's construction and partial-depth approach, subject to the final
  correctness result. No further implementation on theoretical promise alone.
- **Change strategy:** future optimisation only against a freshly measured end-to-end
  bottleneck on the intended four-core hardware.
- **If one unshipped idea is kept,** it is P74 with a bounded online switch, not N1. Not
  ready for production.
- **A principle:** eliminating work is more dependable than replacing it with a faster
  operation that needs expensive preparation. P74 paid in classification and N1 in
  construction; the one-pass build removed preparation.

**Checked by the coordinator:** #114 (6a341f6) is in the v0.2.1 tag, so "shipped" holds.

## The message, as received

Yes. We found several things worth salvaging. But I would distinguish between discoveries
that can improve the shipped decoder, techniques worth retaining for future work, and
ideas that are interesting but probably uneconomic.

The campaign hasn't delivered the major new decompression architecture we were seeking. It
has, however, produced some real engineering value.

What I would salvage

Finding	What survives	My judgment
One-pass BWT construction (#114)	A shipped improvement, measured about 6% faster in its build stage on one CI runner	Concrete success
P74 register-resident MTF	19–33% faster complete MTF stage on Wikidata across two runners	Strongest unrealized opportunity
P74 online switching (Option C)	A possible way to avoid the failed selector's expensive whole-block census	Worth preserving, not yet proven
N1 partial-depth Ψ	Fewer dependent lookups with less construction than full Ψ⁸	Mathematically valuable; economics doubtful
N1 compact navigation representation	Potentially smaller active working set than the T-vector	Useful idea, but cache benefit unmeasured
Benchmarking and performance methodology	Same-binary comparisons, negative controls, code-layout checks, CPU identification and retained-memory accounting	Durable engineering value

The most salvageable unrealized idea is P74

I wouldn't discard register-resident MTF.

We demonstrated something significant: the complete checkpoint-aware MTF stage can execute
approximately 19–33% faster on Wikidata, with correctness checks across the relevant
alphabet sizes.

That isn't a hypothetical speedup.

What failed was making that specialized fast path economical across different
distributions.

The preregistered whole-block selector chose correctly, but its census imposed a 6–7%
regression on controls that never executed P74.

That's a particularly useful failure because it tells us exactly what needs to change.

The subsequent analysis of Option C found that a 4,096-token prefix agreed with
whole-block classification on all 70 examined blocks. That's encouraging, although a small
sample and not a correctness or performance guarantee for arbitrary distributions.

My long-term preference would be to retain Option C as a dormant candidate: no new
implementation now, but sufficient documentation to revisit it if an end-to-end profile
shows MTF remains worth attacking.

What I would salvage from N1

Less than I originally hoped.

Its most valuable result is the partial-depth construction principle:

\Psi^{2a}(x)=\Psi^a(\Psi^a(x)).

When a record's image remains inside one existing interval, its displacement and emitted
bytes can be composed without splitting that record.

That is potentially reusable algorithmic machinery for compressed permutation traversal,
even if this particular decoder cannot afford to build and retain the tables.

The variant is the only N1 configuration I would still consider economically plausible.
Full no longer looks attractive given its construction and memory requirements.

I would preserve the partial-depth proof, checker and cost models, but I wouldn't spend
more engineering effort on N1 unless the pending correctness and retained-memory analysis
gives us a credible reason to do so.

One thing I would not overlook

The shipped decoder's one-pass construction is an actual result. It reduces the cost of
building the conventional representation without requiring a new traversal algorithm or a
workload classifier.

That kind of optimization—removing a pass while retaining a proven data structure—may
ultimately be more productive than replacing the structure.

There is a broader principle here:

Eliminating work is generally more dependable than replacing it with a theoretically
faster operation that requires expensive preparation.

P74 suffered from classification overhead. N1 is suffering from construction overhead.
The one-pass build succeeded by reducing preparation instead.

That is a useful direction for future profiling, although it isn't by itself a new
optimization proposal.

My research recommendation

I would conclude this round with three outcomes:

* Keep: the shipped one-pass build, P74's verified register-resident kernel, the
  online-switch design notes and the corrected benchmarking infrastructure.
* Archive: N1's mathematical construction and partial-depth approach, subject to the final
  correctness result, without continuing to fund implementation on theoretical promise
  alone.
* Change strategy: undertake future optimization only against a freshly measured
  end-to-end bottleneck on the intended four-core hardware.

If I had to select one unshipped research idea to preserve, it would be P74 with a bounded
online switch, not N1. It has already demonstrated substantial stage-level gains, and its
remaining obstacle is better localized.

But I wouldn't call even that ready for production.

The campaign has produced one concrete shipped optimization, one experimentally validated
fast-path specialization, and a set of increasingly rigorous limits on what further
improvements can economically achieve.

That's less than the breakthrough we wanted, but substantially more than nothing.
