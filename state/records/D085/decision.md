## D085 — Conditional-window inference needs a model; run16's literal test fails and its archive conflicts

**G1/G2, A:** REVISE; qualification freeze remains held on the inference justification. Window conditioning is a possible prospective estimand, not a proof that the present t bound is conservative. **B:** condition 3 is unmet as registered (FAIL-MEASUREMENT / not admissible under its all-conditions rule). Additionally HOLD the claimed numerical count pending reconciliation of conflicting archived totals. Neither finding stops the research line or routine design/safety work.

Inbox [6759768735](https://github.com/lmmx/mibz2/pull/161#issuecomment-6759768735); receipt b25b0ce08459c2a55c16f08f835b993a02e722f4. Authority: docs/team SPEC, SPEC.GUARDIAN, MODE, RULES and PROTOCOL at a2da49638ac54d7763de1c167bd054f67ffdc763. Reviewed #316 at 190eb39e1c7ec1394b2413241596ca366df3ad04 and #335 at 313558570e454e98a3dd861a8b0d6cb813a286c2; registration 4c67631e35274caf41e35e6f4500b297b387d8d2.

### A. Direction of bias: a conditional theorem, not a general guarantee

Write the B=24 batch means as Y_b = theta + g_b + epsilon_b, conditioning on a specified fixed path g. The window target is theta_w = theta + mean(g). Require E(epsilon | g)=0. If Sigma is the conditional residual covariance and P=I-11'/B, then:

- Var(mean(Y) | g) = 1' Sigma 1 / B².
- E[s_Y²/B | g] = (g'Pg + tr(P Sigma)) / [B(B−1)].

Their difference is [B g'Pg + B tr(Sigma) − 1' Sigma 1 − (B−1)1' Sigma 1] / [B²(B−1)], or equivalently g'Pg/[B(B−1)] + tr(Sigma)/[B(B−1)] − 1'Sigma1/[B(B−1)]. No sign is guaranteed for general Sigma.

For **iid homoskedastic Gaussian residuals**, Sigma=sigma²I: the expected excess is s_g²/B ≥0. More strongly, mean error is independent of the residual norm, whose scaled square has a noncentral chi-square distribution. Its noncentrality from fixed g makes the denominator stochastically larger than the central one. The usual one-sided t bound is conservative for theta_w under these assumptions. Thus the proposed direction is correct **in that model**, not from conditioning alone.

Counterexample to an unconditional assertion: g=0 and epsilon_b=Z, with Z a centred Gaussian residual common to all batches. The batch sd is zero while the mean error is Z. At the null boundary a negative Z passes half the time. Adding small iid noise makes this nonsingular without fixing coverage. If Z is included in the conditioned path instead, that example disappears—but the remaining noise class and the meaning of the new target must then be specified, rather than assuming all problematic variation away.

Local mathematical probes (seed 6759768735, 100,000 draws, B24, t(.95,23)) gave boundary rejection 5.122% for iid Gaussian residuals, 2.822% with a fixed linear path spanning −1 to 1, and 50.028% with an unconditioned common N(0,1) residual plus iid sd .01. These are counterexample/model checks, not CI measurements or a replication of the pilot calibration. The formula, not their Monte Carlo precision, establishes the distinction.

**What must be fixed before freeze:** the target as a latent conditional expected log ratio over a pre-defined window and weighting, exactly what path is fixed, what remains random, its conditional mean/covariance/distribution assumptions, and the basis for those assumptions. The realised sample mean itself is an observed descriptive quantity, not a latent effect needing this CI. Sequential head/base measurements also do not observe both potential outcomes at the same instant: treatment/order interaction, carryover and idle-gate selection must fit the target/model. Deterministic +−−+ order cancels constant/linear order terms, not arbitrary path-by-order effects. Randomised order with a justified design-based analysis is an option to investigate, not an automatic repair for interference.

The draft's calibration demonstrates vulnerability across the examined classes; it does not prove that no scientifically defensible single-dispatch class could ever exist. Conversely, a mean-free 600-round menu cannot validate the unrestricted residual assumptions needed here. More rounds can average a fixed finite-memory process asymptotically; they do not guarantee coverage uniformly over increasingly slow alternatives. Avoid turning a finite-budget failure into an impossibility theorem.

**Condition 7 is not automatic replication.** §2.6 explicitly triggers a full second dispatch only after a slower wall flag. It settles recurrence of those flags, excludes its data from condition 2, and leaves non-repeated/invalid-confirmation flags unresolved. Many accepted cells therefore get no second-run evidence. Keeping it unchanged does not provide generalisation of every window-conditioned NI result or repair invalid first-dispatch size.

**Authority:** C6 delegates prospective experiment design, acceptance and adoption. Choosing a narrower window estimand does not inherently require Louis. But making it sufficient for the standing service-default route is a substantive policy change, not a wording correction: identify affected conditions and the lost runner/generalisation claim, justify the deployment-risk bridge, obtain Researcher review/PROCEED and Guardian admissibility review, commit the decision record and log it before qualification data. A frozen criterion's relaxation, or amendment of RULES/STEWARDSHIP outside the delegated policy class, remains D3/D1. No such adoption is enacted here. Existing freeze REVISE remains until the concrete model/policy is reviewed; routine drafting and calibration continue.

Independent dispatches are an alternative, not guaranteed independent merely because run IDs differ. Freeze sampling units, dependence assumptions, number/cost, between-dispatch estimator, failure handling and power. No dispatch or spending authorised by this review.

### B. The registered witness and a new evidence inconsistency

The exact condition 3 requires vector-store instructions inside share whose **debug line is in bwt.rs's run16**, ≥10,000,000 executions each in the new arm and none in the old. The report explicitly says that literal test reads zero in both arms. The registration says all conditions must hold for the count to be read and each arm is counted once. Its disclosure clause requires deviations to be reported; it does not grant automatic substitution of a failed evidence predicate.

Therefore **unmet as registered**, not a passed condition “with deviation”. Record the invalid measurement alongside the original result; preserve the numerical observation and corrected interpretation separately under B4. This is a defective witness, not evidence that run16 never executed or that the construction is falsified. Instruction identity can be a stronger scientific witness, but cannot silently rewrite the frozen one. A prospective corrected instrumentation design needs a distinct bounded question, reviewed before new data, not a rerun to pass; existing profiles can be audited without collecting new measurements.

**The archived alternative witness is itself inconsistent:**

- share-window-instructions.txt lists the four stores at 0xd6988, 0xd698e, 0xd6994, 0xd699b, each with **21,480,166** executions, not the journal's **10,740,083**. The ratio is exactly two.
- share-count.txt gives new share self **2,845,867,142**, arcs **296,376**. Their sum / 209,700,196 = **13.572536 Ir/D**, whereas the journal says 6.786.
- Its old self **3,748,029,150** plus **296,376** implies **17.874688 Ir/D**, not 8.937.
- report-new.txt nevertheless puts the entire build stage at **1,458,357,039** Ir, smaller than the other file's purported share self cost. The two ledgers cannot both describe the claimed same accounting.

The factor two suggests duplicated input or aggregation, but its cause is not established. The loader globs both compressed and uncompressed matching profiles without deduplicating their identities; that is one concrete hypothesis to test, not a finding that duplication occurred. Call counts are 466 in the archive, so do not simply halve everything. Reconcile exact profile inventory/digests, thread/part identities, parser version, commands, denominators and binary identities. Recompute the existing evidence, record corrected files alongside originals, and explain precisely why the contradictory artifacts were produced. Do not label the falsifier PASS or FAIL from the larger archive total until accounting is reconciled. **This hold is independently material even if condition 3's alternative witness were accepted.**

The old-binary hash difference is **not itself a freeze violation**: this registration explicitly asks for a new build of master 27ed84df4, records each binary hash and supplies tolerances to reproduce #330; it does not require #330's exact executable. Keep the full hashes and pin the build. I inspected the archived hashes, but have not independently proven source identity or rebuilt either binary.

**Mutation stop/resumption:** a survived safety mutant is a B6 coverage stop, not evidence to hide. Adding a targeted test after it survives is the appropriate repair if the original mutant is then killed for the intended reason and the unmutated kernel passes. The source guard test uses 32 bytes of value 5, counts[4]=1/counts[5]=31, and requires refusal; it targets a 16-byte window at a bucket ending one slot short. The report says Miri kills M6 by an out-of-bounds read while the original passes. Record that scoped reported repair/resumption and retain the initial survival. Native survival is not automatically disqualifying when the relevant safety detector is Miri. I have not replayed Miri or verified raw kill logs; independent B5 review and reproducible baseline/mutant evidence remain outstanding. No production-safety discharge here.

### Evidence limits and protocol

Read the pinned rules, exact draft/result and frozen registration diff, scripts, source guard test, archived hashes, stage/share ledgers and instruction listing. Reproduced conditional-model calculations and archive arithmetic; did not replay compressed profiles, run native benchmarks, reproduce the pilot calibration, or certify the reported chronology fix/tooling identity. Those fixes do not resolve inference.

Migration checked before receipt: old head f9dab3d2b40739d89d613f86e8c2d67bec3159e4; manifest SHA256 60df88f62692909a122e174463e94c012b28e39ab08c6c8a765d41e762a72488. Pinned coord.py audit: five resolved historical findings and previously reported LOG 6757065549 on #162; no new routing defect. REVIEW routes to #145. No other-branch write, implementation, merge, measurement dispatch or automation change.
