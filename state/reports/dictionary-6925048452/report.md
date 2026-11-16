# Cells mathematical dictionary: independent proof check

Assignment: [6925048452](https://github.com/lmmx/mibz2/pull/162#issuecomment-6925048452). Reviewed the supplied text, not a subsequent blog PR. Specification pin: `a2da49638ac54d7763de1c167bd054f67ffdc763`.

**REVISE, narrowly.** The return-tower mathematics is sound. The important empirical correction is that the certificate census is censored: its 33% is a certified lower bound, not an exact unrestricted 256-byte survival fraction. Make the categorical composition convention and graph incidences explicit; strengthen the marked-tower proof as below. The three headline assessments are substantively fair. Exact page/theorem numbering in the supplied expanded Ghrist edition remains unverified in this session.

No decoder timing, implementation, merge, adoption decision or new authority. Independent calculations below concern finite mathematical models and already committed counts.

## 1. Corrections before publication

### R1. State the certificate census's actual observable

Record 28's [source, lines 1119–1146](https://github.com/lmmx/mibz2/blob/25481a03d4fae5aa6405ffebb8a5b343efe9b8bc/docs/journal/monodromy/2026-11-16-certificates/stopwalk.rs.txt#L1119) searches `0..64.min(mm-k0)`, with no wrap at the end of the recorded inverse walk. A start enters the 256-byte numerator only if it reaches that threshold within this search and its certificate still has width at least two. The denominator includes every recorded start, including those whose threshold was never reached.

The aggregate is 0.333566 from rounded per-block values, weighted by starts, over 30 sampled blocks at beta=1.5. That supports:

> At least about one third of sampled starts are certified to share a whole-word itinerary covering 256 bytes, within a 64-step search that does not wrap past the recorded walk's end.

It does **not** establish that all other starts have singleton certificates at 256 bytes. This is a measurement-scope correction, not a retraction of the positive witness.

Counterexample to treating the routine as an uncensored estimator: let F(x)=x+1 mod 1000, with intervals [0,999) and [999,1000), and unit word lengths. This is inverse LF for `L=b a^999`. At 256 steps, 744/1000 starts share the long interval itinerary. A 64-step scan records zero 256-byte successes. `check.py` reproduces 0.744 versus 0.0. No hash or timing is involved. End truncation introduces another source of censoring.

### R2. Fix the categorical conventions, including loops and repeated operands

**Equation (8):** path labels in traversal order are correct. Specify the target's composition convention. With the usual delooping convention `v ∘ u = vu`, traversing labels u and then v gives uv, so the displayed assignment targets `B((Sigma*)^op)`. Alternatively explicitly define the chronological convention `v ∘ u := uv` and retain the displayed notation. Example: edges labelled `ab`, then `c`, emit `abc`, whereas ordinary right-to-left monoid composition gives `cab`. This is a convention repair, not a failure of the word-label construction.

**Position sheaf (§5.3):** on a loopless graph, the stated maps give exactly the stated constraint. For a loop x→x, the ordinary face poset has only one vertex-to-edge incidence. It cannot carry two independently specified endpoint maps. The text explicitly includes the one-vertex closure, so use the endpoint-incidence category (two arrows even on a loop), or subdivide every loop into two edges through an auxiliary vertex. For subdivision, distribute increments h and 0; both constraints then impose p(x)-h=p(x). On the selected cycle modulo d, final h=d makes this constraint tautological, as it should; the construction must still preserve endpoint incidences to state the general modulus criterion correctly.

**Grammar sheaf (§4):** correct as a set-valued relation construction. Specify *three distinct, role-labelled incidence edges per production*: result, left operand, right operand. For N→UU, the two operand edges must remain separate even though they meet the same symbol vertex. Fix a terminal c by giving its symbol vertex the singleton stalk {c}, with the inclusion into each edge stalk; equivalently prescribe the terminal section as a boundary condition. The incidence equations are precisely w_N=w_U w_V. A finite acyclic grammar with fixed terminals therefore has a unique global evaluation (induction in production order). No monoid homomorphism of concatenation is being asserted.

### R3. Give the marked theorem its general proof; distinguish bibliography checked here

The theorem in §6.1 is correct. Its four hypotheses do not explicitly require singleton initialization, while its proof chooses it. Either put singleton isolation and the update rules among the hypotheses, or use the shorter general proof in §2 below. Do not treat equal byte values as the marked occurrence.

The supplied `EAT.pdf` is not attached or available in this workspace. Public author-hosted material corresponds to edition 1.0, with different numbering. I checked the substance independently and against available primary material; I cannot certify the submitted expanded-edition page numbers, Theorem 8.24, Equation 11.3 or Example 7.37. Keep those locators attributed to the edition-holder's check, or verify them against that exact edition before presenting this as a fully checked bibliography. No book pages are reproduced here.

## 2. Statement-by-statement proof findings

| Statement | Disposition and derivation |
|---|---|
| LF formula and reference output, §1.1 | Correct. Stable occurrence ranking is a permutation; within a run it is a translation. Inverse LF before emission gives the stipulated output, including repeat/truncate semantics on an arbitrary column. The real-interval extension uses integer translations and half-open intervals; irrational/minimal IET conclusions are not inherited. |
| (1), word orientation | Correct. For a return x,πx,…,π^(h−1)x, storing its reverse puts bytes in inverse-LF order. At the induced inverse edge T(x)→x the first emitted byte is L[π^(h−1)x], the last L[x]. |
| Tower partition and (2) | Correct. For each z in the saturation, choose its nearest backward visit to D, allowing time zero when z∈D. This gives a unique (x,j), 0≤j<h(x). Hence the levels are disjoint, cover precisely the saturation, and have total size sum(lambda h). On a single cycle this equals n while D is nonempty. |
| Suspension conjugacy (3) | Correct: eta is the bijection just constructed. Away from a tower roof, eta(x,j+1)=π eta(x,j); at the roof, eta(T(x),0)=π^h(x). The base map alone is not conjugate to the original permutation. |
| Suffix step, (4), §1.4 table | All three cases are correct. K⊂I_B lies in the retained section because I_B and I_A are disjoint. Its first old return lands in Q; its second in J_A, disjoint from Q⊂J_B. Thus it returns after exactly two old returns. Reversing the B-then-A itinerary gives w_A w_B. Top win places B after the shortened A in image order; bottom win renames the suffix of I_B as A after B in domain order; equality deletes A and transfers its image position to B. |
| A=B closure | Correct. Equal terminal domain/image intervals imply translation zero, so each base row is fixed by the induced map and represents one original cycle. Different base rows can represent different cycles. Dropping an unmarked branch reduces the saturation. |
| Dictionary; finite substitutions; (5) | Correct with versioned symbols, occurrence identities and an unrotated root. Initialize P(root)=0 and other reachable nodes at infinity. Each parent's occurrences translate its child positions by fixed offsets. The minimum therefore propagates in reverse topological order. This is not the cyclic phase computation. |
| Morse matching (6), acyclicity and boundary | Correct on the stated saturation, with regular subdivision where needed. A matched path follows outgoing LF edges through removed vertices and terminates at D; it cannot cycle. Eliminating those vertex-edge pairs transports the head of critical e_x to T_D(x), giving boundary T_D(x)−x with consistent orientations. A one-row loop needs an explicit subdivided matching; the row/critical-edge dictionary then refers to the corresponding contracted path. Removing an entire component is not a homology-preserving operation on the original full graph. |
| (7), non-homomorphic multiplication | Correct for the byte alphabet's noncommutative free monoid: take distinct a,b,c,d to witness acbd≠abcd. Identity incidence maps only synchronize words; they do not concatenate them. The critique says the proposed monoid-valued sheaf was not specified, not that no richer one can exist. |
| Path functor (8) | Correct after declaring chronological composition or using the opposite monoid, as R2. This is additional ordered path data, not ordinary chain-module coefficients. |
| (9)–(11), period and modular exactness | Correct on the selected cycle, with D meeting it. Return excursions partition its d rows; the sum of h is d. The integer circulation is −d≠0, excluding an integer/real potential on the closed graph. Modulo d it vanishes. Define p by integrating −h from one anchor; consistency follows on closure. Dividing increments by d yields integral period −1 in R/Z. Keep edge lifts/winding as well as vertex values. |
| `abab`, modulus n | Correct: LF for `bbaa` is (0 2)(1 3). Integrating −1 round either two-cycle gives −2, nonzero modulo 4. Modulo the selected d=2 there is no obstruction. Output occurrences can instead be distinguished on a cover. |
| §5.3 existence iff cycle increments vanish | Correct on a permutation graph with endpoint incidences preserved. Necessity is telescoping. For sufficiency, propagate from one anchor round each component; vanishing circulation closes it. There are exactly q choices per unanchored component over Z/q, and one after an anchor. On a general graph require every *signed closed-walk* sum to vanish; checking only directed cycles would be insufficient. |
| (12), marked closure | Correct; see the general proof below. Singleton isolation is sufficient, not necessary for the theorem. The actual writer still needs its own proof. |
| (13), dual transform | Correct for top win: old lambda_A=new lambda_A+new lambda_B, old lambda_B=new lambda_B; new h_B=old h_A+old h_B. Thus M=I+E_AB and h_new=M^T h_old. For bottom win use I+E_BA. For equality the survivor column has ones in rows A and B; transpose sums their heights. Volume is unchanged by each concatenating step. |
| (14), step bounds | Correct for one n-cycle and completion by the stated elementary steps: final m=1 and one label; precisely r0−1 equality deletions, no unmarked component closures. Each ordinary step removes at least one row. These assumptions exclude early stops, budgets, label coalescing and multi-component discards. |
| Primitive family `(ab)^k a` | Correct for k≥1 and the **unrefined** run partition. Widths k+1,k undergo one strict subtraction, then k−1 unit subtractions and one equality: S=k+1=(n+1)/2. This proves the claimed non-O(r polylog n) behavior. Retaining all successively growing literal words can cost quadratic bytes. Singleton refinement is a different step census. |
| (15)–(16), batching | Correct provided the same ordered pair wins strictly q consecutive times. Inducting on q gives left multiplication w_A^q w_B for top wins and right multiplication w_A w_B^q for bottom wins. Width stays positive with every strict comparison; equality ends the batch. Binary powering takes O(log q) extra concatenation rules. This does not assert such streaks are common or profitable. |
| (17), itinerary cylinder | Correct for a fixed partition and translations, with word labels constant on each interval. Necessity and sufficiency follow by x_j=x+s_j and membership in every I_ij. Empty intersections are allowed. Byte-word equality or net-displacement equality alone cannot substitute for an interval itinerary. Later refined bases are contained in these cylinders, not necessarily maximal. |
| Output and representation lower bounds, §6.6 | Correct in the explicitly stated byte-traffic model. Outputting n bytes does not prove n dependent navigation loads, existing grammar size, or existing memory layout optimal. |

**General proof of (12).** At closure choose the base x of the tower containing r*. The first return satisfies T(x)=x; its least positive return time h is therefore exactly the original cycle length d. Write r*=π^v(x), 0≤v<d. Equation (1) says E[t]=L[π^(d−1−t)x]. The marked occurrence has index j=d−1−v. Substitution gives E[(j+1+k) mod d]=L[π^(v−1−k)x]=L[π^(−1−k)r*], the required reference output for every k. This proof requires the occurrence/tower coordinate, not the byte value, and works even without singleton refinement.

For the implementation's singleton proof: initially only its one-row letter contains the distinguished terminal. A width-one letter cannot be a strict winner, so its word cannot be retained and duplicated by winning. A losing marked letter transfers into the width-one composite; equality retains a width-one survivor, transferring the mark if A is deleted. All other cases leave it unchanged. At marked closure the width is therefore one. This closes the induction omitted from the short paragraph.

## 3. Count trace and independent aggregation

Source pins: construction and companion at `4599b7633eb6b38b4994b678d8cd0e2095d1a387`; records 21,23,27,28 at `25481a03d4fae5aa6405ffebb8a5b343efe9b8bc`. The construction journal is byte-identical at these two pins. All paths below are relative to `docs/journal/monodromy/` at the latter pin unless specified.

| Claim | Source and check |
|---|---|
| Original 27,439,723; S/r=1.812; S/n=.1309 | `2026-11-10-rauzy-veech.md` §§0,2,11 gives r=15,141,602, n=209,697,662, 233 blocks. Division gives 1.812207387 and .130853738. These are traced counts, not a new replay of all original blocks. Its §6.2 distinguishes Cells' 27,439,756: 33 extra steps after marking. Retain **original/unrefined** in the caption. |
| 33%, 5.0 bytes/run at beta1.5 | Record21 `2026-11-16-stop-walk-reopened.md`; raw `2026-11-16-stop-walk-reopened/beta_1.5_wikidata.jsonl`. Sum alloc=228,714,675; sum saved=75,747,258; 5.002592064 bytes/original run and 33.11866980%. 232/233 blocks stop; a non-stopping block saves zero. |
| 4.89 bytes/run,32%,30 wider-ID blocks at beta1.2 | Record27 `2026-11-16-compact-tail.md`; raw `2026-11-16-compact-tail/tail_wikidata.jsonl`, filtered to switch1.2 (233 rows). Ledger saving = alloc−made_at_switch−join_width*t−8*frozen, with join_width4 or6 according to IDs. Sum=74,058,616; 4.891068726 bytes/run,32.38035163%,30 wider-ID blocks. This verifies the specified word-storage ledger, not measured peak RSS or allocation capacity. |
| About one third at256B | Record28 `2026-11-16-certificates.md`; raw `2026-11-16-certificates/certificates_wikidata_every8.jsonl`. Beta1.5 has30 rows. Start-weighting the rounded fractions gives .333566191. R1 states the essential censoring qualification. |
| m=.672r; q_live=.1212r; m/q≈5.54 | Record23 `2026-11-16-two-chains.md` §2 gives both, over232 stopped blocks. The preceding beta1.5 raw rows independently give m/r=.671922912 when the denominator includes only stopped blocks (all233 instead give .663936484). The live-letter .1212 is traced to record23; that entry contains the counter source but no corresponding per-block output file was present in the pinned tree. I did not independently recompute that raw total. The ratio of the stated rounded quantities is5.544554. |

Every requested figure is traceable to a named record; the live-letter raw-total limitation and certificate censoring are explicit. The prose correctly keeps prototype emitter counts separate from whole-decoder timings, and treats the mark offset as an uncounted/unexercised obligation in the early prototype. None of these counts certifies all origins or periodic input.

`counts.py` reproduces these aggregations from the three pinned JSONL files; `count-results.json` records the result. Do not pool the 232-stopped cohort, all233-block cohort, and every-eighth-block sample without naming the denominator.

## 4. Ghrist: fair correspondences, limited bibliographic verification

**Morse:** the proposed auxiliary model is exact, after the regular-complex/loop qualification. It yields homology, not ordered bytes. The author's publicly indexed [EAT edition1.0 chapter7, §7.8 p150, Theorem7.23](https://www2.math.upenn.edu/~ghrist/EAT/EATchapter7.pdf) states the acyclic matching and Morse-homology result. That supports the substantive verdict, but does not verify the other edition's numbering. The graph boundary above is independently derived.

**Sheaves:** the rejection of an unspecified monoid-valued sheaf is fair. A diagram of stalks and morphisms is missing from that assertion; noncommutative multiplication fails the needed homomorphism test. A set-valued relation sheaf supplies a valid alternative, with the ports and boundary conditions in R2. [Curry–Ghrist–Nanda, *Discrete Morse Theory for Computing Cellular Sheaf Cohomology*, §2](https://people.maths.ox.ac.uk/nanda/source/DMSheaf_Final.pdf) provides primary support for regular-CW cellular diagrams. Its module-valued computational theorem must not be transferred to the nonlinear word relation merely by naming it a sheaf.

**Circle coordinates:** the modular integration argument is exact; negative orientation and modulus d are essential. [de Silva–Vejdemo-Johansson, *Persistent Cohomology and Circular Coordinates*, §2.1](https://arxiv.org/html/0905.4887v1#S2.SS1) supplies the primary circle-map/cohomology construction. A real cocycle with integral periods integrates modulo integers. Smoothing by a coboundary changes vertex potentials while preserving the cohomology class, so it does not preserve the decoder's prescribed intermediate positions. This corroborates the mathematical assessment without certifying the supplied book's particular Example7.37 reference.

No novelty or optimality theorem follows from any of these correspondences. In particular, an exact homological reduction does not prove that this reduction order is the cheapest decoder.

## 5. Is building a sheaf useful here?

**As an executable specification, plausibly. As a new decoder mechanism, not yet.** The position sheaf is a system of modular difference constraints. On a known cycle, solving it is the existing weighted prefix/ranking computation; an anchor removes the one-dimensional freedom. The grammar sheaf's unique section is ordinary SLP evaluation. An explicit string-valued solution could reproduce the quadratic retained-word cost the article correctly warns against.

A useful falsifiable application is a checker for independently generated segment summaries: check endpoint phase differences, loop sums and port order without trusting the production writer. It may catch a wrong modulus, reversed concatenation or duplicated/omitted incidence. That is a verification benefit, not a prediction of faster decoding. A runtime proposal would need a different representation or evaluation schedule and a count of work it removes; the generic sheaf object alone supplies neither. Preserve this distinction in the article.

## 6. Validation and disposition

`check.py`, written independently from the supplied equations, exhausts every ternary column of lengths1–6 and every origin: **6,015 inputs,26,331 states,16,011 concatenations,135,191 tower-point checks and385,896 itinerary-cylinder checks**. It compares the update table with direct LF first returns, checks reversed byte/marker words, tower coverage and suspension transitions, marked closure output, one-cycle step bounds and volume identities. Separate assertions cover noncommutative batching, modular sections including loops, a repeated-operand grammar relation, and the census-cap counterexample. All pass. This is finite validation alongside the proofs, not a proof by enumeration or a production safety review.

Expected: a faithful dictionary separating dynamics, labels and storage. Found: that core survives; the strongest correction concerns the measured certificate observable, and categorical notation needs explicit conventions. The mathematical gap is closed by the derivations above; exact expanded-edition bibliography and the live-letter raw total remain limited as stated. Next useful work is the scoped text correction and source-locator check, not another decoder benchmark or generic sheaf implementation.

**Disposition: REVISE R1–R3; no STOP of the mathematical line.**
