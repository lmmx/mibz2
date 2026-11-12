# External research report 96: approving the walker's implementation preparation; W4 refined (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 18:54 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** the walker decoder-trial protocol draft.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Worked for 8s

I approve the implementation-preparation phase for the unrolled BWT walker, with one refinement to the structural audit. The draft is coherent, the scope is appropriately narrow, and it preserves the independence of the P74-C trial.

I've reviewed both documents at `ea19199`.

### What looks right

The proposed `SegmentedUnrolled` kernel is a particularly clean design.

Rather than adding another per-block dispatch or modifying `Plan`, Claude can select the new inverse-BWT implementation through the existing `StreamDecoder::with_inverse` mechanism.

That gives us a meaningful comparison:

| Component                        | Shipped       | Experimental            |
| -------------------------------- | ------------- | ----------------------- |
| Scanner and feeder               | Same          | Same                    |
| Worker count and buffer slots    | Same          | Same                    |
| MTF                              | Shipped plain | Shipped plain           |
| RLE1, CRC and block construction | Same          | Same                    |
| Inverse-BWT walker               | Existing      | Constant-index unrolled |

The plan also correctly excludes the cooperative walker used by larger team configurations. We're testing the actual four-core `Segmented` decoder, not claiming an optimization of every parallel architecture.

### My one concern: structural gate W4

The protocol currently requires the normalized machine code of the two decoder instantiations to differ only in their inverse calls.

That may be unnecessarily restrictive.

Because `Segmented` and `SegmentedUnrolled` are separate Rust monomorphizations, LLVM can legitimately alter register allocation, inlining, code layout and surrounding instructions—even when the source-level implementation is equivalent.

We've already seen this phenomenon with P74-C.

The important distinction is between:

* Functional identity: the two modes use the same scanner, scheduling, MTF, RLE1, CRC and memory-management behaviour.

* Machine-code identity: every instruction outside the walker is identical.

We need the first. Requiring the second could create another structural stop for a compiler difference that is an ordinary consequence of integration.

I would change W4 before implementation to require the same stage functions, verified source-level data flow and an audited report of any surrounding machine-code differences. Material differences should trigger review before timing; byte-for-byte equivalence should not be mandatory.

This still protects against accidentally benchmarking a different MTF or decoder pipeline, without making the experiment depend on LLVM choosing identical layouts for separate generic instantiations.

One other detail: the four-core affinity restriction should cover the entire process, not just the workers. The draft already includes that, so no change is needed.

### My recommended decision

I approve the preparation and implementation of the off-by-default `SegmentedUnrolled` full-decoder research arm according to the draft at ea19199.

Please make one protocol refinement before committing the registration: revise W4 so it requires source-level functional equivalence of scheduling, MTF, build, RLE1, CRC and buffering, together with an exact-binary audit reporting any surrounding code-generation differences. Do not demand identical normalized machine code outside the inverse call, because separate Rust monomorphizations can legitimately produce different layout and register allocation. If a difference changes execution semantics or introduces a material uncontrolled benchmark difference, stop for review.

Keep `SegmentedUnrolled` isolated to additions in `src/bwt.rs`, preserve the existing production defaults, and keep P74-C entirely separate.

You may implement the research kernel, correctness tests, harness, freeze and local dry run up to timing. Preserve the protocol, input hashes and all generated evidence.

Do not start exploratory CI timing, execute the i9 trial, merge, or enable the kernel by default. Report the exact frozen SHA, audit and correctness findings, and any deviations for review before the next authorization.

### What comes after that

If the dry run passes, the next decision will be whether to authorize exploratory CI timing. Then, after the existing P74-C i9 experiment, we can run the unrolled walk's own four-core decoder trial.

The registered stage result gives this experiment a strong justification: the unrolled walker was approximately 12% faster in the counted Zen 3 stage run, with an independent Zen 4 replication.

But the final shipping question is still whole-decoder throughput, CPU consumption and memory.

This is exactly the right next step: integrate a successfully measured optimization into the real decoding path, without changing anything else.
````
