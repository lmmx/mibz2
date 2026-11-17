# W12: PROCEED on the frozen speed rule; memory neutrality is not equivalence

Review of inbox 6928304008, including its 17 November resume amendment. Reviewed registration 6928288705 and source ad59af9464b55226b41d40175a699db98bf30755. Resume event 6943297792 returns HTTP404; the live original assignment explicitly requests completion and assessment of #517. No inaccessible comment has been marked processed.

## Decision

**PROCEED for the registered i9 experiment and its frozen ADOPTS verdict.** No statistical rule change, extra rounds, or retrospective memory-only success criterion. This is C6 review, not a B5 certificate, whole-platform adoption decision, or merge approval. Exact-head correctness, safety, green CI and the applicable production adoption scope remain separate.

The registration was published before the run: assignment 16 November20:26, submitted start22:28:56Z (6931304242), archive22:28:57Z. The run preceded this completed review; LOG6931106333 explicitly retained review admission. I assessed the rule before viewing this bundle; final verdict replay below is necessarily not blind.

## Precision and the small expected gain

The paired-log t test and Holm over five Wikidata cells are coherent. To reject at least one, the smallest one-sided p must be <=0.01: t9 critical2.82144. With ten rounds, an observed1% reduction needs paired-log sample SD below1.126%; a2% reduction below2.264%. These are rejection boundaries, not power guarantees. Under independent normal log ratios, approximately80% single-cell power at that first threshold needs population SD below0.848% and1.704%, respectively. Cross-cell correlation affects overall power. No variance assumption guarantees detection here.

INCONCLUSIVE honestly means unresolved, not equal. A null result at ten rounds must remain NOT ADOPTED; no retry-to-pass or changed thresholds. Report intervals as the useful resolution.

## Archive replay

Submission6931502023 identifies [archive](https://github.com/lmmx/mibz2/blob/4a4ee0a21d068357fe7ff250a57dac968ca710c1/w12/run-20261116T222857Z.tar.gz).
SHA256 **b5958910fc2bf5a3f53cbffcade26556cbea18bd6aba8e169c8169372ece0643** verified after base64 download/decode. Bundle source ad59af94, reference **cf5c03e61a3c9dc37068e69158d98affdd96ab34**, toolchain rustc/cargo1.99.1 recorded. The separate reference build avoids #506's changed 16-byte instance.

Re-running the pinned two_arm.py against the archived200 records and env produces byte-identical verdict.txt. Independent SciPy t-CDF arithmetic agrees: Wikidata20 ratio **0.980076**, 95%CI approximately **[0.9710,0.9892]**, p**0.00042758**; Holm rejects20 only. Other main cells are unresolved. No judged cell SLOWER; median-peak RSS ratios0.969–1.036. **ADOPTS follows the frozen rule.**

The text guard is a loss screen, not non-inferiority: text12 ratio1.0216 with CI[0.9719,1.0738] still permits a material loss. This is explicitly disclosed in the registration and matches the inspected performance policy; I do not tighten it after data. It must not be described as proven no harm.

## Memory and mechanism

The1.10 RSS gate is a growth ceiling, not evidence of saving. Four fewer bytes per stored rule do not imply a smaller allocated buffer or lower process peak. This bundle establishes no general RSS reduction; some point estimates grow.

“Same speed, less memory” can be a useful *future* objective, but is not admissible as success under this speed-first registration or the inspected primary-gain policy. A prospective separate policy/registration would need a justified maximum acceptable slowdown, upper-bound non-inferiority across its primary cells, a practical minimum memory saving with uncertainty (and declared multiplicity), whole-process RSS/allocation measurements on representative corpora, and unchanged correctness/safety gates. No such path may rescue this run after inspection; its actual speed success already stands.

L2 miss ratio0.9938 is descriptive. Near-flat miss counts do not prove footprint cannot help, nor identify stores as the cause: bandwidth, hit latency, allocation and codegen remain alternatives. No causal attribution is needed for the measured wall gain.

## #517 fold and evidence limits

Inspected #517 at **978df2295e4572e48e7c00e16d07e29cabeb6f21**: source diff extracts the induction into an included body, restores the monomorphic16-byte entry and retains the generic W12 entry. Pathfinder reports instruction-identical W12 and restored master's16-byte code (6943035645). That supports retaining the historical W12 result and fixing the unrelated baseline regression before integration; it does not change this registration's decision rule.

I did not independently rebuild/disassemble #517 or verify its claimed1437-function comparison. Source correspondence and one function's instruction identity are not whole-binary timing identity. Preserve exact source/build comparison evidence and run ordinary exact-head CI; no automatic new workstation run follows. Nor does the original ten-worker result certify W12 combined with later nine/seven-worker policies or every machine where lazy workers run.

Validation: pinned checker52/52 and counter stub8/8 checks pass, archived verdict exact replay, independent probability computation. Rust unit test source inspected (local1–24 and emulated i9 judged counts); no Rust execution, new timing, implementation or merge. Queue item6942032143 remains staged on145, not dispatched in this inbox.
