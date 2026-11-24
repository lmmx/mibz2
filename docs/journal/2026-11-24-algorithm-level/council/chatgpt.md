# Council answer: ChatGPT Pro

*Recorded verbatim, as Louis pasted it into the Coordinator's session on 24 November 2026. The member answered [BRIEF.md](BRIEF.md) at fd406da9 without seeing the other answers. Its inline citation links did not survive the paste, so each `⁠￼` marks where one was. Its section and table citations name the sources (M = MONODROMY record, N = Neon item, F = frozen target, J = journal, P = shared specification, ME = METHOD). The maths is in LaTeX as pasted.*

---

1. Definitions

[Argued — principal finding.] The intended problem is an existence question about a uniform, exact, natively acquired computational representation of inverse BWT whose proved cost law escapes a restricted old mechanism. The sources also impose a frozen numerical target and an empirical utility requirement. These are distinct obligations. In particular, the sources establish a scoped derivation floor, but do not yet establish a machine-execution floor in the same currency. A faithful formalisation must expose that missing comparison theorem instead of treating it as already proved. Journal §§21–23, 32⁠￼; MONODROMY 103 §2⁠￼; 108 §§2–4⁠￼.

1.1 Source and status conventions

[Taken from source — provenance.] I use the four mibz2 commits specified by the brief:

Source group	Commit
Master, charter, collected essay readings, published article's source	c282b422b4a1ae6ffde4c153ffe65a01a0088013
Formulation journal and its three appendices	2a99cd517ffdb370bedf8583b6130b3c3df2fe75
MONODROMY records and frozen theorem	f86a4d3b3b5d819c010c6eb27a4856acbd87a90a
Neon records	cc3b76f32adb1b50d07a520c67b2760b4c73b884

The source set includes all fifteen top-level MONODROMY records dated November 24, including record 110, and all seventeen research/neon/n*.md files present at the Neon pin. N-0, N-16 and N-17 are represented within those records rather than by separate files. The published article was read through its publishing source at the pinned master commit. No other council member's answer was consulted. Brief⁠￼; published source⁠￼; N-2⁠￼; N-12⁠￼; N-13⁠￼; N-14⁠￼.

[Taken from source — version limitation.] The brief supplies no immutable version for the doomslide article or the Mario repository. For Mario, the inspected snapshot is 1541b15c2a6541aa0b54cc10fa93634ad4142855. I do not identify that snapshot with the earlier SSL material described in the mibz2 journal. The distinction matters: the journal discusses a conditional constructive reachability programme, whereas the inspected repository develops a conditional no-A/no-flight result and leaves the WMotR completion theorem unfinished. Journal §§2, 23⁠￼; Mario README⁠￼; WMotR goal⁠￼.

[Argued — reporting convention.] Here, proved means that a mathematical argument is supplied here or identified precisely in a source; it does not silently mean proof-assistant checked. Taken from source includes measurements and proof-checking claims that I have not reproduced. Argued covers proposed definitions, modelling choices and their justification. Conjectured denotes an unproved mathematical assertion. Definitions recommended below are marked argued because choosing a definition is not proving a theorem.

1.2 Input domain: distinguish the kernel contract from G1's promise

[Taken from source.] The inverse-BWT kernel receives a byte column L, its counts, and the primary index p=\mathrm{origPtr}. Its actual size conditions are

1\le \ell\le 9,\qquad
1\le n=|L|\le 100\,000\ell\le 900\,000,\qquad
0\le p<n.

The alphabet is the ordered byte alphabet

\Sigma=\{0,\ldots,255\},\qquad
1\le |\operatorname{supp}(L)|\le \min(256,n).

There is no added sentinel symbol in the bzip2 interface. The 24-bit storage of origPtr does not enlarge the legal block bound. Shared specification §1.1⁠￼.

[Argued — definition.] Write \mathcal D_{\mathrm{ker}} for all marked columns satisfying those interface bounds. Write \mathcal D_{\mathrm{bz}}\subseteq\mathcal D_{\mathrm{ker}} for columns arising from legal bzip2 blocks. This separates an executable kernel contract from a promise about a compressed stream. It follows the project's requirement that kernels agree even on arbitrary columns, with corruption handled elsewhere. Shared specification §§1.1–1.2⁠￼.

[Argued — definition.] For i\in[n]=\{0,\ldots,n-1\}, define

C_L(a)=|\{j:L[j]<a\}|,\qquad
\operatorname{rank}_L(a,i)=|\{j<i:L[j]=a\}|,

\operatorname{LF}_L(i)
=C_L(L[i])+\operatorname{rank}_L(L[i],i),
\qquad
\Psi_L=\operatorname{LF}_L^{-1}.

Ranks are half-open. These are the shared specification's conventions. Shared specification §1.2⁠￼.

[Proved.] LF is a permutation: for each byte a, its occurrences map bijectively, in occurrence order, onto the consecutive range beginning at C_L(a); those ranges partition [n]. Consequently \Psi_L exists. This is the elementary argument behind the specification's LF_bijective obligation. Shared specification §1.2⁠￼.

[Argued — definition.] The promised domain for the scoped floor is

\mathcal D_{\mathrm{sc}}
=\{c\in\mathcal D_{\mathrm{ker}}:
\operatorname{LF}_c\text{ consists of one }n\text{-cycle}\}.

For a theorem specifically about legal bzip2 blocks, use
\mathcal D_{\mathrm{bz}}\cap\mathcal D_{\mathrm{sc}}. The single-cycle assumption must remain visible in every application of G1. MONODROMY 103 §§1–2⁠￼.

[Proved — domain distinction.] Valid BWT input does not imply a single LF cycle. For example, a constant text has a constant last column, for which LF is the identity. Thus "valid bzip2 block, single cycle" is an additional research promise, not a description of every block the decoder must handle. The project already specifies repetition of the marked cycle for multi-cycle columns. Shared specification §§1.1–1.2⁠￼; N-1⁠￼.

1.3 Output and correctness

[Argued — definition.] The required kernel output is the length-n byte string

\operatorname{decode}(c)[j]
=L\!\left(\Psi_L^{\,j+1}(p)\right),
\qquad 0\le j<n.

An admissible total decoder A must terminate and satisfy

\forall c\in\mathcal D_{\mathrm{ker}},
\qquad A(c)=\operatorname{decode}(c).
\tag{Correct}

On genuine BWT input this equals the original block's RLE1-coded text. RLE1 expansion and CRC checking lie outside the inverse-BWT specification. Shared specification §§1–2.2⁠￼.

[Proved.] On \mathcal D_{\mathrm{sc}}, the equivalent backward form is

\operatorname{decode}(c)[n-1-k]
=L\!\left(\operatorname{LF}_L^k(p)\right),
\qquad 0\le k<n,

because \Psi^{n-k}=\operatorname{LF}^k on the marked n-cycle. Shared specification, T1c⁠￼.

[Argued — strict reading.] Correctness means equality of every output byte. Producing a compressed description, an unordered multiset, a checksum match, or correct output merely on the benchmark is insufficient. A supplied-state diagnostic can omit acquisition for the diagnostic, but that omission cannot enter an end-to-end correctness-and-cost theorem. Frozen target §1a⁠￼; METHOD §§6–6a⁠￼; MONODROMY 105 §4⁠￼.

1.4 Structural parameters

[Argued — definition.] Let the maximal linear runs of L[0:n) be

R_j=[a_j,b_j),\qquad j=1,\ldots,r.

Do not merge the first and last runs across the boundary. Define

J_1(c)=\{a_j:b_j-a_j=1\},\qquad
h_1(c)=|J_1(c)|.

Thus h_1\le r\le n. Let m(c) denote the number of rows retained by the specified Open implementation, including its treatment of the primary row; m is not interchangeable with h_1. MONODROMY 96 §§0–2⁠￼; 102 §3⁠￼; shared specification, O1⁠￼.

[Proved — edge cases.] A single cycle need not have h_1>0. For L=\texttt{bbaaa},

\operatorname{LF}=[3,4,0,1,2],

which is a single cycle, while both runs have width greater than one. At the other extreme, n=1 gives h_1=1, so an integer charge bounded by h_1/2 must be zero. A fractional improvement theorem therefore needs an explicit performance domain and zero-case policy; correctness still covers these cases. These conclusions follow directly from the definitions above.

1.5 Computation and accounting

[Argued — recommended model.] Use a fixed bounded-word machine \mathsf M, with an explicit memory layout and finite elementary instruction set. For the concrete decoder, fix 64-bit words and the permitted SIMD instructions. For asymptotic statements, define a separate uniform word-RAM family with w(n)=\Theta(\log n). Do not use the finite bzip2 size cap to conceal unbounded tables or enormous constants. This adds precision required by the no-advice clause and record 108's instruction-set requirement. Frozen target §1a.7⁠￼; MONODROMY 108 §2⁠￼.

[Argued — definition.] An instruction set consists of:

1. Typed operands of bounded size.
2. Exact state-transition semantics.
3. Explicit memory accesses.
4. A fixed operational tariff.
5. Rules for lowering any later high-level operation into these elementary transitions.

Ordinary arithmetic, comparisons, bit operations, branches, loads and stores are permitted with stated widths. A high-level operation may be mathematically new, but naming it does not make its execution one elementary instruction. Arbitrary-length copying, sorting, inversion and arbitrary-precision arithmetic require their actual implementations and costs. MONODROMY 107 §4⁠￼; 108 §2⁠￼.

[Argued — definition.] For the complete execution trace \tau_A(c), retain at least

\mathcal C_{\mathsf M}(A,c)
=
\bigl(
W,\ Q_{\mathrm{read}},\ Q_{\mathrm{write}},
\operatorname{span},\
d,\ D_{\mathcal H},\
P,\ S_{\mathrm{peak}}
\bigr).

Here W is elementary work; Q is actual traffic; span is dependency depth; d is the frozen dependent-load count; D_{\mathcal H} is the dependent-miss count under a specified memory model; P is the pass measure; and S_{\mathrm{peak}} is maximum simultaneously live auxiliary storage. Construction, recognition, conversion, consumption, runtime verification, fallback and allocation initialisation all enter this trace. Frozen target §§1–1a⁠￼; MONODROMY 103 §3⁠￼; 107 §4⁠￼.

[Argued — pass convention.] A pass must denote a specified traversal of a specified logical sequence, with bounded work per visited record and an explicit access schedule. The pass account must include input and workspace traversals, partial traversals, repeated scans and output—not only scans whose source variable happens to be named L. A pass is not a unit-cost instruction, and arbitrary work between passes remains charged in W,Q. In particular, an O(r) traversal cannot be declared negligible merely because r is often small: r can equal n. MONODROMY 98 §2⁠￼; 108 §§2–4⁠￼.

[Argued — unresolved numerical convention.] The sources do not specify a complete trace-to-pass function that makes the frozen K=2 unambiguous. "One working pass plus output" does not decide how to count multiple arrays, partial scans, fused traversals or accesses to values updated earlier in the same sweep. Therefore the exact function P_\Pi(\tau), including record units and fusion rules, must be fixed as part of the specification package \Pi. I retain K=2; I do not silently choose a convenient interpretation that makes a candidate fit. Record 110 directly prevents interpreting one sweep as necessarily one synchronous composition layer. Frozen target §1⁠￼; MONODROMY 108 §3⁠￼; 110 §§2–3⁠￼.

[Argued — memory convention.] Measure auxiliary state in bytes, including all simultaneously live indices, dictionaries, labels, plans, stacks, caches implemented by the algorithm, and writable input copies. Excluding the final n-byte output allocation does not make its traffic free. Any use of that allocation as workspace must be declared and charged. The frozen condition is the pointwise ceiling

S_{\mathrm{peak}}(A,c)\le S_{\mathrm{Open}}(c),

for the pinned baseline and required performance domain. A generic O(r)-word statement is not a substitute. Frozen target §§1, 3⁠￼; N-1⁠￼.

1.6 Resolve record 108 by keeping its two currencies separate

[Taken from source.] Record 108 adopts the strict convention: a build pass incurs a charge for each row whose record it uses to determine something about that row; a pure fold to O(1) words per block is exempt from that charge. Its adopted conclusion concerns the known or proposed mechanisms. Its broader "no by definition" discussion is labelled argued, not proved. MONODROMY 108 §§3–4⁠￼.

[Argued — definition.] Call that frozen charge P_{108}, reserving c_{\mathcal R} for record 103's precisely scoped rigid-calculus charge. Pure folds remain free only in P_{108}; their arithmetic and accesses remain charged in \mathcal C_{\mathsf M}. The number and width of the fold's result words must be fixed uniformly, so that "one word" cannot encode an entire column.

[Argued — necessary qualification.] The phrase "determine something about that row" is not, by itself, a complete operational semantics. A formal version needs a fixed rule/effect system assigning attributable charge tokens to executions, including bulk operations and the fold exception. The frozen pseudo-code's count of events with singleton subjects does not implement the later rule that one bulk event can carry many row charges. Until that system is supplied, P_{108} is a specified intention with an incomplete formal interface, not a machine-instruction count. Frozen target §§1a–2⁠￼; MONODROMY 107 §4⁠￼; 108 §2⁠￼.

[Proved — distinction.] These ledgers cannot generally be identified. A fold may have P_{108}=0 while executing many machine operations. Conversely, one bounded SIMD instruction can perform several operations that the provenance convention charges separately. Therefore neither P_{108}=W nor "one provenance event equals one hardware instruction" follows from the definitions. Frozen target §1a.4⁠￼; MONODROMY 108 §§2–4⁠￼.

2. What "a new basis" means formally

2.1 A basis is an executable representation with a cost theorem

[Argued — definition.] Relative to a baseline mechanism class \mathcal B, a proposed basis is specified by existentially quantified data

\bigl(\mathcal Z,\operatorname{Rep},
\operatorname{Build},\operatorname{Use},\theta,f\bigr).

These are placeholders in the question, not a proposed object or construction:

* \mathcal Z is a precisely defined representation space.
* \operatorname{Rep}(c,z) states its mathematical relationship to the input and required output.
* Build acquires z from the permitted native input.
* Use produces the required bytes, with every further input access exposed.
* \theta(c) is an independently defined structural parameter, possibly a tuple.
* f gives a proved bound in the fixed cost model.

The required semantic obligations are

\operatorname{Build}(c)=z\Longrightarrow\operatorname{Rep}(c,z),

\operatorname{Rep}(c,z)
\Longrightarrow
\operatorname{Use}(c,z)=\operatorname{decode}(c).
\tag{Representation}

This formalises the interface demanded by record 107 and the three conditions in channel entry 14. MONODROMY 107 §4⁠￼; channel #14⁠￼.

2.2 Condition A: a new object relative to the old mechanism

[Argued — definition.] "New" must be relative to a declared \mathcal B. It cannot mean merely a new type name, a different coordinate notation, a shorter correctness proof, or a macro whose executed expansion is the old mechanism. The object must support a semantic interface and a proved cost separation that the old mechanism cannot supply in the same currency. MONODROMY 103 §2⁠￼; 105 §§1–3⁠￼; 107 §4⁠￼.

[Argued — strict interpretation of the current requirement.] For the present campaign, the governing object cannot merely be a collection of individual row states or rigid interval-translation facts under another name. Row indices and intervals may still occur as metadata. Prohibiting their occurrence anywhere would exclude essentially every implementation and would also defeat the historical sanity check. The prohibition concerns the mechanism responsible for the claimed saving. Channel #14⁠￼; journal §32, R1 and R6⁠￼.

[Proved — consequence of a separation.] Suppose every implementation admitted to \mathcal B costs at least F(c), while the candidate costs strictly less than F(c) on a nonempty promised family. Then the candidate cannot admit a cost-preserving reduction into \mathcal B on that family. This supplies an operational distinction that syntactic nonmembership alone cannot supply. The proof is immediate contradiction between the two inequalities; §3 specifies the required comparison for G1.

2.3 Condition B: a new structural parameter that helps on the hard family

[Argued — definition.] Require a whole-execution cost bound

C^*_{\mathsf M}
\bigl(\operatorname{Build};\operatorname{Use},c\bigr)
\le f\bigl(\theta(c),n,w,\sigma\bigr),
\tag{Cost law}

where C^*_{\mathsf M} is a fixed compared cost component defined in §3. The bound includes acquisition. The parameter must have mathematical meaning independent of defining it to equal the candidate's running time or its measured saving. Its recognition or computation, if the decoder needs it, is also charged. Channel #14⁠￼; journal §32, R2–R4⁠￼; METHOD, G3–G4⁠￼.

[Argued — quantitative meaning of "small".] On the declared performance family, "small" must imply the required inequality, for example

f(\theta(c),n,w,\sigma)
\le \tfrac12\kappa h_1(c),

with the currency and \kappa fixed beforehand. A small-looking object, a favourable average, or six low descriptor counts is insufficient. An asymptotic claim such as f=o(h_1) is stronger and must be stated separately; the sources do not establish it. Frozen target §§1, 1a.7⁠￼; MONODROMY 108 §4⁠￼.

[Argued — hard-family definition.] Use record 103's text-generated family as the starting point:

T=S[U_1,\ldots,U_m],
\qquad
U_i\overset{\mathrm{iid}}{\sim}\operatorname{Unif}(\mathrm{Hex}^k),
\qquad
c=\operatorname{BWT}(T,p).

The admissible scaffolds, slot placement, alphabet, lengths, single-cycle conditioning and treatment of repeated fills must be declared. "Hexadecimal," "novel token" and "independent random content" are different predicates. A novelty-conditioned theorem must specify that conditioning; it cannot inherit an independence claim without checking it. MONODROMY 103 §4⁠￼; N-8⁠￼.

2.4 Condition C: native acquisition without the old mechanism

[Argued — definition.] Build starts from the permitted encoding of L, the primary index and deterministic information already available at that interface. It receives no decoded text, suffix array, orbit positions, run labels, output-order plan or other supplied capability unless the cost of producing it is included in the same trace. Frozen target §1a.3⁠￼; FRONTIER, "The target"⁠￼; METHOD §6⁠￼.

[Argued — strict interpretation.] The acquisition proof must show that the new state is not obtained by a rowwise LF/Ψ walk hidden in a helper, conversion or preliminary decoding stage. Merely changing the traversal's name or the storage order does not discharge this obligation. A trace property establishing native acquisition and a complete cost account are both necessary: the syntactic absence of a call named walk proves neither. Channel #13–14⁠￼; MONODROMY 103 §2⁠￼.

[Argued — fallback rule.] A total decoder may have a fallback outside its promised improvement family. Every fallback and recognition cost counts, and the promised inequalities must hold for the actual combined decoder on the required inputs. A supplied-state consumer remains a conditional result until this acquisition obligation is discharged. Frozen target §1a.3⁠￼; METHOD §§6–6a⁠￼.

2.5 Sanity check: LF walk → Rauzy–Veech

[Taken from source.] The historical transition supplies the intended pattern: the pointwise walk uses row states; the interval method uses labelled translations and first-return paths on intervals; its construction operates on run/endpoint structure rather than first decoding the text row by row. The shared specification identifies the preservation, first-return, endpoint-removal and spelling obligations. Shared specification, O2–O12⁠￼; MONODROMY 104 §2⁠￼.

[Proved — mathematical part of the check.] If I lies within one run, LF is a translation on I. Consequently a valid interval fact can represent the same transformation for every row of I using shared data. Composition preserves the relevant orbit segment and emitted word when its compatibility conditions hold. This establishes a genuine change from individual point obligations to shared interval obligations; it does not rely on renaming point states. MONODROMY 103, Lemma A⁠￼; 104 §2⁠￼.

[Taken from source; argued qualification.] The reported cost advantage is governed by run/interval structure and the actual induction and retained-link counts, rather than by performing one pointwise navigation step for every original row. This meets the intended relative definition on the favourable instances documented by the programme. However, the sources do not justify replacing the full cost law by an unconditional theorem "RV takes O(r) total time." Output remains length n, and actual induction-description size and construction work must remain in the bound. FRONTIER, "The target"⁠￼; shared specification, O10–O12⁠￼; N-3⁠￼.

[Argued — crucial relativity.] The historical comparison is against the pointwise-walk mechanism, whose stipulated navigation count is n. The current comparison is against the larger rigid calculus \mathcal R, which already contains interval composition. Thus Rauzy–Veech satisfies the historical new-basis test while failing the current request to escape G1. A definition requiring the historical transition to beat h_1 would be testing the wrong floor. MONODROMY 99 §1⁠￼; 103 §2⁠￼; 104 §2⁠￼.

3. What "lowering the floor" means

3.1 The floor actually established

[Taken from source — proved in prose.] Record 103's \mathcal R uses fixed-orientation rigid facts

(I,h,t,w)

asserting that every x\in I follows the same length-h word and satisfies

\operatorname{LF}^h(x)=x+t.

Primitive facts, mirrors and restrictions are free in its accounting. A singleton-result composition is charged once per built DAG node, at its join row; each executed output-chain link is charged at its start row. DAG reuse is not unfolded and charged again. Mixed-orientation derivations are outside the stated theorem. MONODROMY 103 §1⁠￼.

[Proved — G1's argument.] For c\in\mathcal D_{\mathrm{sc}}, an output chain covers every row. A rigid fact whose path meets a singleton run has a singleton domain. For each x\in J_1(c), either:

* x begins its output-chain link, yielding a charge at x; or
* descending the fact's derivation reaches a singleton composition whose join row is x.

Every charged event has one attribution row, so distinct singleton rows require distinct events. Hence

\forall D\in\mathcal R,\qquad
c_{\mathcal R}(D,c)\ge h_1(c).
\tag{G1}

More strongly, the charge attributed to any X\subseteq J_1(c) is at least |X|. This is the row-preserving provenance theorem of record 103, not a general machine lower bound. MONODROMY 103 §2⁠￼.

[Proved — meaning of escape.] A valid lower bound cannot be beaten inside the same class in the same currency. Therefore "lowering G1's floor" must mean finding a successful computation outside the class covered by G1, while preserving an honest comparison of costs. It cannot mean refuting G1 by introducing another derivation satisfying all its hypotheses.

3.2 The missing operational comparison

[Argued — required specification.] Before considering a candidate, fix:

* an operational old class \mathcal A_{\mathrm{old}};
* a trace extraction E into valid \mathcal R-derivations;
* a nonnegative whole-trace cost functional C^*_{\mathsf M};
* a constant \kappa>0, with its dependence on word width or SIMD width stated.

Then prove

\forall B\in\mathcal A_{\mathrm{old}}\ \forall c\in\mathcal D_{\mathrm{sc}},
\quad
E(\tau_B(c))\in\operatorname{Valid}_{\mathcal R}(c),

C^*_{\mathsf M}(B,c)
\ge
\kappa\,c_{\mathcal R}\!\left(E(\tau_B(c))\right).
\tag{Bridge}

The comparison must cover every implementation admitted to the old operational class. A semantic expansion into LF facts is insufficient. MONODROMY 103 §2⁠￼; 107 §4⁠￼.

[Proved.] G1 and Bridge imply

\forall B\in\mathcal A_{\mathrm{old}},
\qquad
C^*_{\mathsf M}(B,c)\ge\kappa h_1(c).

The inequality direction matters. A compiler satisfying only
C_{\mathsf M}(\operatorname{Compile}(D))\le\kappa c_{\mathcal R}(D)
is an upper-bound result and does not transfer G1 into a machine lower bound.

[Argued — present status.] The sources do not supply a fully fixed C^*_{\mathsf M} and Bridge for the operational class envisaged by R4. Consequently that operational floor-separation problem is presently a parameterised theorem schema. Selecting its tariff and proving Bridge are prerequisites, not details that may be chosen after a favourable candidate appears. Journal §32, R4⁠￼; MONODROMY 103 §2⁠￼; 108 §2⁠￼.

3.3 No work disappears into a pass

[Argued — accounting rule.] C^*_{\mathsf M} must be a fixed functional of the complete execution, invariant under moving code between named phases or wrapping it in a macro. Every transition omitted from C^*_{\mathsf M} remains in the full ledger \mathcal C_{\mathsf M}, with the exclusion justified explicitly. This is the precise replacement for "no cost moved off the ledger." Frozen target §1a.3⁠￼; MONODROMY 108 §§2–4⁠￼.

[Proved — compulsory-output caution.] In a byte-traffic currency, materialising the output costs at least n written bytes, while h_1\le n. A total-byte-work target below h_1/2 would therefore be impossible for an elementary reason unrelated to BWT structure. In a word-instruction currency the output bound is different, which is why the unit must be stated.

[Argued — baseline subtraction.] If the comparison removes a common cost B(c), require an actual decomposition

W(A,c)=B(c)+C^*_{\mathsf M}(A,c).

The baseline must be fixed independently of the candidate. In particular, separate lower bounds W\ge B and W\ge\kappa h_1 do not imply W\ge B+\kappa h_1: the work can overlap. Acquisition fused into a scan must not disappear through this subtraction. MONODROMY 108⁠￼; round-two review, cost-accounting discussion⁠￼.

3.4 Relation to the frozen target and whole-decoder utility

[Argued — strict reading.] Retain

P_{108}(A,c)\le\tfrac12h_1(c)

as a separate frozen obligation. An operational improvement that violates it may be an important result under a revised execution-based question, but is not a frozen yes. Conversely, satisfying the provenance inequality does not prove an instruction, traffic or timing improvement. Frozen target §1a and residual⁠￼; MONODROMY 108 §4⁠￼.

[Taken from source.] The frozen utility condition requires a whole-decoder win against master. The supplied-codes experiment reported emitter ratios 0.53 at one worker and 0.55 at ten, but whole-decoder ratios 0.838 and 0.988, with the latter interval [0.957,1.020]. That experiment did not establish a ten-worker whole-decoder win. Journal §29⁠￼; frozen target §7⁠￼.

[Argued — scope correction.] That diagnostic constrains the tested consumer and pipeline. It does not prove that every improved inversion mechanism is useless at ten workers, nor identify a universal bottleneck outside the kernel. A different acquisition procedure can change construction, traffic and interaction with other stages. The final utility test must pin the baseline commit, machine, compressed inputs, worker count and statistical decision rule. Journal §29⁠￼; METHOD, G4 and experimental gates⁠￼.

4. The main theorem's shape

4.1 Fix the specification before quantifying over the decoder

[Argued — definition.] Let

\Pi=
\bigl(
\mathsf M,\mathcal D_{\mathrm{ker}},
\mathcal F,\mathscr G,
P_{108},C^*_{\mathsf M},
\mathcal A_{\mathrm{old}},E,\kappa,
K,S,\mathcal H,\delta,\varepsilon,
\text{resource budgets}
\bigr)

be fixed first. Take the frozen values K=2 and \delta=1/2. Here \mathcal F is the exact validation set; \mathscr G is the declared collection of text-generated distributions; and \varepsilon is the allowed resource-bound failure probability. No currency or input family may be chosen retrospectively to suit the decoder. Frozen target §§1–1a⁠￼; MONODROMY 103 §4⁠￼.

[Argued — quantifier warning.] The question must not be

\exists A\,\exists C^*:
\text{\(A\) is cheaper under \(C^*\)}.

An arbitrary candidate-dependent tariff could manufacture that conclusion. The machine, charge classification, old-class coverage and comparison factor belong in \Pi, outside the existential candidate quantifier.

4.2 The mathematical success predicate

[Argued — proposed exact schema.] Once \Pi is instantiated, the strict mathematical target is

\exists A,\mathcal Z,\operatorname{Rep},
\operatorname{Build},\operatorname{Use},\theta,f
\quad
\mathsf{Success}_{\Pi}
(A,\mathcal Z,\operatorname{Rep},
\operatorname{Build},\operatorname{Use},\theta,f),
\tag{Main}

where \mathsf{Success}_{\Pi} is the conjunction below. This packages the source obligations without supplying any construction. Frozen target §§1–4⁠￼; channel #14⁠￼; journal §32⁠￼.

Premise	Required statement	Present standing
Specification	Correct holds on every c\in\mathcal D_{\mathrm{ker}}.	Argued: required target; not supplied for a new decoder.
Representation	The two implications in Representation hold, and A executes their acquisition and consumption.	Argued: required target.
Native acquisition	The complete build starts from the native interface and satisfies the declared exclusion of hidden rowwise acquisition.	Argued: required target.
Structural cost law	The whole computation satisfies Cost law, with an independently defined \theta.	Argued: required target.
Old floor	G1 applies to extracted old traces, and Bridge holds for the declared operational old class.	Proved in source for scoped G1; argued/open obligation for the general operational bridge.
Validation margin	On every c\in\mathcal F, both P_{108}(A,c)\le h_1(c)/2 and the declared operational separation hold.	Argued: strict target, not established.
Uniform-family margin	For each specified distribution in \mathscr G, the joint margin and resource event has probability at least 1-\varepsilon.	Argued: target; \mathscr G,\varepsilon still need completion.
Resources	Pass, peak-state, dependent-load, dependent-miss and full-ledger budgets hold on the declared performance domain.	Argued: target; some model parameters remain unspecified.
Uniformity	One program; no corpus answers, per-block advice or hidden acquisition tables.	Taken from source: required by clause 7.

[Argued — probability statement.] For each allowed scaffold and size tuple g, the probabilistic condition should be written explicitly as

\Pr_{c\sim\mathcal G_g}
\!\left[
\begin{array}{l}
P_{108}(A,c)\le h_1(c)/2,\\
C^*_{\mathsf M}(A,c)\le\kappa h_1(c)/2,\\
\operatorname{Resources}_{\Pi}(A,c)
\end{array}
\right]\ge 1-\varepsilon_g.
\tag{Family}

Correctness is not inside this probability: it remains universal. The strict default is a deterministic decoder. Any algorithmic randomness requires a separate seed quantifier and a zero-error policy. Frozen target §1a.7⁠￼; MONODROMY 103 §4⁠￼.

[Argued — distinction between target versions.] Requiring both margins is the strict reconciliation of the frozen target with the later operational ambition. If the programme instead elects to replace P_{108}, that produces a new version of the target. It must not be described as discharging the old one. The sources' explicit residual labels support preserving this distinction. Frozen target, residual⁠￼; MONODROMY 108 §4⁠￼.

4.3 Premises are the agenda, not the conclusion

[Argued — Mario-style form.] The useful main theorem is a conditional composition result:

H_{\mathrm{spec}}\land H_{\mathrm{rep}}
\land H_{\mathrm{native}}\land H_{\mathrm{cost}}
\land H_{\mathrm{bridge}}\land H_{\mathrm{resources}}
\Longrightarrow \mathsf{MathematicalSuccess}_{\Pi}.

Each premise has its own exact statement and status. A proved composition theorem with an unproduced acquisition premise remains a conditional result. The checked statement, its definitions, residual hypotheses and non-vacuity are all part of the audit. Journal §23⁠￼; Mario invariant harness⁠￼; Mario trust model⁠￼.

[Taken from source; argued correction.] The existing N/R/W decomposition is one sufficient proof architecture. It is not a necessary form for every successful decoder. If used, its margin must include the split, shared construction, both streams, wide-row work, glue, conversion and fallback. Its state bound must concern actual simultaneous liveness; its pass bound must concern the combined execution schedule. Frozen target §§3–4⁠￼; MONODROMY 102 §3⁠￼; 103 §3⁠￼.

[Proved — logical limit.] From

P_N\land P_R\land P_{\mathrm{glue}}\Longrightarrow Q

and \neg P_N, one cannot infer \neg Q. Therefore refuting a stream premise, a supplied representation or one acquisition procedure cannot establish the programme's no. MONODROMY 103 §3⁠￼; frozen target §6⁠￼.

4.4 What a meaningful "no" must quantify over

[Argued — definition.] A no for the instantiated mathematical problem is

\forall A\in\mathcal A_{\mathrm{admissible}}(\Pi),
\qquad
\neg\exists
(\mathcal Z,\operatorname{Rep},
\operatorname{Build},\operatorname{Use},\theta,f)
\;\mathsf{Success}_{\Pi}(\cdots).
\tag{No}

The admissible class must include every permitted global computation and every mixture of permitted mechanisms. A finite list of separately unsuccessful families does not establish this universal statement. MONODROMY 106 §§1–3⁠￼; 107 §§2–3⁠￼; 108 §4⁠￼.

[Argued — useful narrower no.] For the finite validation condition alone, a count obstruction can have the form

\forall A\in\mathcal A_{\mathrm{admissible}},
\quad
\operatorname{Correct}(A)\land
\operatorname{Resources}_{\Pi}(A)
\Longrightarrow
\exists c\in\mathcal F:
P_{108}(A,c)>h_1(c)/2.

For a probabilistic target, the corresponding no must exceed the stated failure probability on an allowed distribution. An expectation bound is not automatically a per-block or high-probability obstruction. MONODROMY 102 §1⁠￼; 103 §4⁠￼.

[Conjectured — source, not established.] The records conjecture near-one per-random-row charges for certain bounded-pass settings. Neither that conjecture nor the failure of currently tested routes supplies No for unrestricted admissible native decoders. MONODROMY 98 §3⁠￼; 100 §3⁠￼; 108 §4⁠￼.

4.5 Engineering acceptance remains a separate assertion

[Argued — definition.] A complete programme yes additionally requires the specified empirical predicate

\operatorname{Utility}_{\mathrm{protocol}}
(\operatorname{Rust}(A),\operatorname{baseline}).

This includes the whole-decoder result, not merely the inversion kernel or supplied-state consumer. The implementation's correspondence with the mathematical algorithm is another explicit obligation. A correctness theorem about an abstract construction does not establish the Rust implementation's equivalence or its measured speed. Shared specification §0⁠￼; frozen target §7⁠￼; METHOD⁠￼.

5. Check against the sources

5.1 What the two motivating sources contribute

[Taken from source.] Doomslide contrasts accepted results with understanding that can generate further results: the Hearers preserve assertions, the Learners their reasons; Archimedes separates mechanical discovery from subsequent demonstration; Hilbert's twenty-fourth problem asks about relationships between proofs. Boat I offers oracle-produced instances, Boat II a recurrence generating further instances, and Boat III an anthyphairetic argument exposing structure. This supplies a methodological demand for an intelligible generative mechanism, not a theorem that a better BWT decoder exists. 

[Argued — match and addition.] Sections 2–4 preserve that demand by requiring a representation, executable acquisition, preserved semantics and a cost law. They add an execution model and a comparison theorem because the essay does not provide either. The collected mibz2 readings' proof-complexity analogy is useful, but a short schema, short representation and cheap execution remain different properties. Collected readings §§3–4⁠￼; MONODROMY 105 §1⁠￼; 107 §4⁠￼.

[Taken from source — Mario scope.] The inspected Mario capstone is a Qed conditional result about finite sequences of completed linked-Clight execute_mario_action calls. Its conclusion is non-flight at the resulting memory state, under initial-state, input and residual hypotheses. The linked-program, memory, external-call and return-value assumptions remain material. The WMotR completion theorem is explicitly not started. Linked capstone⁠￼; frame semantics⁠￼; WMotR goal⁠￼.

[Argued — lesson.] The relevant precedent is therefore the disciplined composition of conditional results, with model boundaries exposed. It is not evidence that a checked theorem automatically establishes the intended real-world reachability or impossibility claim. The same discipline requires mibz2 to expose acquisition, operational charging, implementation correspondence and empirical utility separately. Mario trust model⁠￼; journal §23⁠￼.

5.2 Corrections and contradictions within the records

[Taken from source; argued adjudication.] The following table identifies material contradictions, superseded claims and incompatible scopes. An earlier assertion is not retained merely because it remains quoted in the journal or frozen draft.

Issue	Conflict in the source set	Strict disposition
G1's domain	Record 96 says every column; N-1 and records 102–103 require single-cycle orbit coverage.	Use record 103's single-cycle theorem. M96⁠￼, N1⁠￼, M103⁠￼.
What c_1 counts	Record 98's singleton-rule formulation, record 102's all-links rule, record 103's DAG attribution and record 108's bulk-read convention are not one unchanged definition.	Name c_{\mathcal R} and P_{108} separately; state compatibility obligations. M98⁠￼, M102⁠￼, M103⁠￼, M108⁠￼.
Open equals h_1	Earlier comparisons say Open meets the floor exactly; record 102 corrects its total charge to at least m.	"Exactly one per singleton row" does not mean total cost h_1. Wide-row link charges remain. M96 §4⁠￼, M102 §3⁠￼.
Orientation	The frozen fact type has positive h and a mirror constructor; record 103 requires one orientation throughout.	Mixed-orientation coverage requires a separate argument. Do not silently infer it for every two-ended implementation. F §2⁠￼, M103 §1⁠￼.
Sorting and list ranking	Record 98 assigns zero count; record 99 declares the families inside \mathcal R; record 103 withdraws that family-wide claim.	Classify concrete executions through cost-preserving simulations. M98⁠￼, M99⁠￼, M103 §2⁠￼.
E1 closure	Record 99 closes class facts generally; records 103 and 107 leave bulk, packed and implicit uses open.	The scalar Gd frontier count closes that implementation only. M99⁠￼, M104 §1a⁠￼, M107 §1⁠￼.
Randomness model	Record 96 assumes independence in the column; record 103 relocates randomness to text fills.	Do not transfer the earlier probabilities automatically. Equality of observed rates is not a proof of independence. M96 §§3–4⁠￼, M103 §4⁠￼.
Sampling convention	Record 96 uses a geometric run-width law in a row-sampled discussion.	Use record 103's size-biased correction when sampling rows. M103 §3⁠￼.
Expectation versus guarantee	The local-key estimate was used as a per-block stream coefficient before repair.	Keep it distributional; prove a concentration or pointwise statement if required. M100 §4⁠￼, M102 §1⁠￼.
Theorem N	Record 106 applies the rigid interior argument to non-rigid word facts; its local-key class also admits constant keys.	Theorem N is withdrawn. Only the repaired \Lambda' theorem remains. M106⁠￼, M107 §§1–2⁠￼.
Flip blindness	Record 101's treatment of observed records overlooks changed translation information; later coefficients change again.	Use complete observations and N-18's access-specific coefficients: one for raw bytes/point LF values, two for full run records in its disjoint-span argument. M101 §3⁠￼, M107 §1⁠￼, M108 §1⁠￼, N18⁠￼.
Which U is bounded	An all-flip intersection was treated as the relevant indistinguishability set.	U depends on the complete observation; adding observations can enlarge the set of fixed coordinates. M107 §1⁠￼, N18⁠￼.
Order-sensitive summaries	Record 107 says it has no event floor beyond histograms; N-18 and record 108 recover a semantic fibre floor inside \Lambda'.	The general fibre statement survives; a useful quantitative bound on its fixed rows remains missing for richer observations. M107 §3⁠￼, M108 §1⁠￼, N18⁠￼.
Short proofs	Record 105 suggests an o(|S|)-size schema as the escape criterion.	A constant-size formula can still need one evaluation per row. Require execution costs. M105 §1⁠￼, M107 §4⁠￼.
Gap counts and positions	Record 104 identifies one-step gap-rank data with the aggregate positional problem.	Record 107 withdraws that identification: one-step destinations do not by themselves determine orbit positions. M104 §1c⁠￼, M107 §1⁠￼.
Unit-price evidence	The first account uses only the 5–14% lanes result and predicts no frontier benefit.	The later branchless-append result is 0.55–0.78 of emitter time; the prediction is withdrawn. Equal provenance counts do not determine time. ER §4⁠￼, M104 §1b⁠￼, M107 §1⁠￼.
Novel content cannot be copied	Record 102 denies a copy source for novel random nibbles.	Record 108 withdraws the inference: short substrings can recur inside otherwise novel tokens. This does not prove cheap acquisition. M102 §2⁠￼, M108 §1⁠￼, N12⁠￼.
One pass means depth two	Record 108's lenient analysis uses that premise; record 110 uses same-sweep updates.	One sweep is not necessarily one synchronous layer. Record 110 remains a charged near miss, not a yes. M108 §3⁠￼, M110 §§2–3⁠￼.
Finite family size	The frozen Finset is described as 232 distinct blocks plus six admitted blocks, while records repeatedly say 238.	N-2 identifies the six among the 232. Distinguish a set of distinct columns from a list with repeated validation entries. F §1⁠￼, N2⁠￼.
The frozen no	Its prose leaves fine-input global E2 open, but its displayed union omits global E2 and then calls the open cell empty.	That theorem cannot establish emptiness of the omitted class. F §6⁠￼.
Mario's completed goal	The brief's wording and some journal analogies can suggest a completed level-impossibility result.	The inspected repository establishes a conditional intermediate theorem; Goal 2 remains open. B⁠￼, J §23⁠￼, Mario⁠￼, Goal2⁠￼.

5.3 What the measurements establish

[Taken from source — measurements, not rerun.]

Finding	Defensible interpretation
About 65% of h_1 is novel hex	N-8's cost-weighted figure is 64.61% under its maximal-hex-token definition on six blocks. Restricting token length changes the figure substantially. This is a tokenisation-dependent measurement, not a randomness theorem. N8⁠￼.
Repeated long tokens are often wide	Across the cited 3,645 repeated 40-character SHA-1 occurrences, only three bytes contribute singleton rows. This supports that measured long-repeat case; it does not exclude useful short copies. N8⁠￼, M108 §1⁠￼.
Supplied copy plans have low descriptor counts	N-12 reports approximately 0.54–0.64 descriptors per h_1 on five blocks and 0.48 on the sixth. Acquisition is excluded. Descriptor count is not automatically P_{108} or machine instruction count. N12⁠￼.
The half target remains demanding	Five of those six supplied-plan ratios exceed 0.5 even before acquisition. Thus the shorthand ratio proves neither the frozen margin nor a general new-parameter theorem. N12⁠￼.
Fibre thresholds constrain a narrow class	N-18's reported q_{50} values range from 6,903 to 14,120 on six blocks, using witnessed flip families. Their consequence depends on the observation language and \Lambda' output restriction. N18⁠￼, M108 §1⁠￼.
Cache results are model-specific	N-15 simulates an edges-only cache model; its per-worker L3 approximation is not a full shared-cache execution theorem. N15⁠￼.

[Argued — additional corrections.] Several screening conclusions must remain narrower than their headings:

* N-12's repaired acyclic parse is evidence about that greedy unidirectional scheme, not all macro representations or hybrids. Its acquisition cost being \Omega(n) does not alone violate a constant-pass budget. N-12⁠￼.
* N-13's literal differential-SA entropy does not bound all arithmetic or implicit representations. An algorithmic O(\cdot) upper bound cannot be substituted for a lower bound proving budget failure. N-13⁠￼; round-two review⁠￼.
* N-14's delimiter-and-hole experiment does not quantify over every return hierarchy. N-14⁠￼.
* N-10's large suffix-array jumps do not by themselves prove a near-one affine-segment count; steep affine sequences can have large jumps. N-10⁠￼; round-two review⁠￼.
* N-18's finite enumerations do not prove an asymptotic vanishing fraction of fixed coordinates. Counting many observation collisions also does not determine how many particular coordinates remain fixed. N-18⁠￼.
* N-3's Sturmian aside labels approximately forty diagram edges as much smaller than h_1 while specifying r=2. Since h_1\le r, that comparison cannot be correct; the historical saving there is against row-scale work. N-3⁠￼.
* With an R-share q=h_{1R}/h_1, free N, and no other charges, the half target requires a_R\le 1/(2q), not automatically a_R\le 1/2. Adding glue and wide-row charges tightens that inequality. N-11⁠￼; frozen margin⁠￼.

5.4 The strongest surviving fibre statement

[Argued — definition from the repaired sources.] For a complete observation \iota, define its fibre at c, with n,p fixed, by

\operatorname{Fib}_{\iota}(c)
=
\{c'\in\mathcal D_{\mathrm{sc}}:\iota(c')=\iota(c)\}.

Let U_\iota(c) contain the rows whose orbit position is identical throughout that fibre. If observations are adaptive, \iota must include the complete observable transcript, not just an advantageous subset of answers. N-18, Lemma 18.1⁠￼; MONODROMY 108 §1⁠￼.

[Proved — semantic necessity, with restricted cost consequence.] A sound rule receiving only \iota(c) cannot assign a definite position to a row outside U_\iota(c): the same observation would force the same answer on two inputs requiring different answers. Inside the repaired \Lambda', where only rigid facts write bytes at their own rows' positions,

c_1\ge h_1-|J_1\cap U_\iota|.

The second conclusion uses the output restriction, not indistinguishability alone. N-18, Lemma 18.1⁠￼; N-19, Proposition 19.1⁠￼; MONODROMY 107 §2⁠￼.

[Argued — limit.] Taking \iota=L makes the fibre singleton, but does not provide a cheap decoder. Information sufficiency is different from efficient extraction. N-19's examples outside \Lambda' demonstrate why the restricted output rule cannot silently become a theorem about all exact inversion algorithms. N-19⁠￼; MONODROMY 108 §§1–2⁠￼.

6. Critique of the current formulation and corrected statements

6.1 Frozen target

[Argued — corrections.]

Frozen item	Defect	Corrected statement
Input and correctness	"Valid" is used for both legal BWT input and single-cycle columns.	Correctness covers \mathcal D_{\mathrm{ker}}; G1 carries the explicit \mathcal D_{\mathrm{sc}} promise. P⁠￼, M103⁠￼.
c_1, clauses 1–2	Singleton-subject counting, DAG charging and distinguished-input charging are conflated.	Define c_{\mathcal R} exactly as in record 103; separately formalise P_{108} with attributable charge tokens for every admitted rule. F⁠￼, M103⁠￼, M108⁠￼.
Free aggregates	"Aggregate" is treated as though it were a semantic class of functions.	Exempt only the specified bounded-word fold computation from P_{108}, while charging its actual execution. No arbitrary observation function is free. M108 §2⁠￼.
K=2	Pass units and composition rules are incomplete.	Supply an executable trace-to-pass account for all input/workspace traversals, with full work and traffic bounds alongside it. F §1⁠￼, M108⁠￼, M110⁠￼.
Memory	The conclusion substitutes O(r) for the exact Open ceiling.	State the pointwise peak bound S_{\mathrm{peak}}\le S_{\mathrm{Open}}; state any asymptotic theorem separately. F §§1, 3⁠￼.
No advice	A fixed finite program can still contain a giant lookup table on a bounded domain.	Fix program/static-data restrictions and, for asymptotics, a uniform extension to unbounded n. Charge static lookup resources explicitly. F §1a.7⁠￼, M103 §3⁠￼.
Probability	"In probability" lacks failure rate, parameter quantifiers and conditioning.	State Family, including scaffold quantifiers, \varepsilon_g, single-cycle conditioning and the separate correctness guarantee. F §1a.7⁠￼, M103 §4⁠￼.
Dependent loads and misses	The model is not fully specified; per-block misses depend on other workers and cache state.	Retain clauses 6/6b, but define the memory layout, replacement policy, initial state and concurrent context. A per-block miss function needs those arguments. F §1a.6–6b⁠￼, N15⁠￼.
Split/additivity	The displayed proof architecture omits some acquisition costs and treats component budgets as the complete execution.	Charge the actual combined trace, including split, shared work, W, glue and fallback. Additivity requires complete attribution; it does not prove streamwise lower bounds by itself. F §§3–4⁠￼, M103 §2⁠￼.
No theorem	Its quantified class omits the advertised open mechanism.	Quantify over the complete admissible class, or label the result a partial no for the stated subclass. F §6⁠￼, M107 §3⁠￼.
Utility	Count success and empirical acceptance are too easily merged; "master" floats.	Keep mathematical and empirical predicates separate and pin the baseline and decision protocol. F §7⁠￼, ME⁠￼.

6.2 Channel entries #13–#14

[Argued — corrected reading of #13.] The acquisition question is appropriately directed at the missing premise, but "many rows per instruction" requires a fixed instruction semantics and charged lowering. The corrected requirement is:

\text{native acquisition}
+
\text{proved complete cost}
+
\text{a quantitative benefit on the declared family}.

Consumer-only descriptor counts do not discharge acquisition or the half-h_1 inequality. "O(1) passes" also does not imply the frozen value K=2. Channel #13⁠￼; MONODROMY 108 §4⁠￼; N-12⁠￼.

[Argued — corrected reading of #14.] The three conditions should be retained jointly:

1. An exactly specified representation whose saving mechanism is not a cost-preserving restatement of the current row/rigid-interval mechanism.
2. A proved acquisition-and-consumption law in an independently meaningful parameter, with a quantitative guarantee on the declared hard family.
3. Acquisition from the native interface, with hidden old-mechanism work excluded and all actual work accounted.

The source's statement that supplied copy plans already meet conditions 1–2 is a provisional assessment. The measurements establish a consumer description size on six blocks; they do not establish the required uniform cost theorem. Channel #14⁠￼; N-12⁠￼.

[Argued — progress versus success.] A named near miss is useful because it identifies an exact unmet premise. It is not a partial proof of existence merely because it has an attractive object. Likewise, a scoped obstruction is useful without becoming a universal no. This preserves the source programme's status distinctions while making their logic explicit. METHOD, result statuses⁠￼; MONODROMY 106 §4⁠￼.

6.3 Rubric R1–R7

[Argued — corrected rubric.] The rubric should evaluate the following obligations, with met / partial / unmet accompanied by the claim-status labels.

Criterion	Problem with the current wording	Corrected criterion
R1 — New object	Datatype novelty is easy to manufacture; a literal ban on all intervals defeats the historical analogy.	Give \mathcal Z,\operatorname{Rep} and its operational interface; identify the mechanism replaced. Demonstrate a cost distinction, not merely a name distinction.
R2 — New parameter	"Small where runs fail" is qualitative; six measurements do not define a family theorem.	Define \theta independently, prove the whole cost law, and state the exact pointwise or probabilistic smallness guarantee. Keep measured validation separate.
R3 — Acquisition	No visible LF loop and O(1) passes do not establish native cheap acquisition.	Audit the complete build and all dependencies from the native interface; state exact passes, work, traffic and state.
R4 — Floor lowered	G1 is a provenance theorem, while the rubric requests an execution-model conclusion.	Either compare in the explicitly extended provenance currency, or supply C^*_{\mathsf M} and Bridge before claiming an operational G1 separation.
R5 — Proof form	A conditional theorem can have uninhabited or unproduced premises.	Expose every residual, establish non-vacuity, and distinguish proved composition from existence of the required witnesses.
R6 — Not incremental	Syntactic exclusion from \mathcal R is insufficient; a different proof need not compute a different algorithm.	Establish the representation's semantic content and a quantitative separation from the declared old mechanism. A problem/format change is a separate target.
R7 — Utility	An emitter payoff can coexist with no whole-decoder gain.	State the supplied-state whole-decoder diagnostic and then the fully acquired implementation's result under a pinned protocol. Unknown utility remains unknown.

Source for the original seven criteria: journal §32⁠￼. The corrections follow the scoped membership requirement, execution-interface repair and charge-convention diagnosis. MONODROMY 103 §2⁠￼; 107 §4⁠￼; 108⁠￼.

[Argued — acceptance correction.] "R1–R3 met and R4 partial" can classify promising research progress. It cannot establish that a floor has been lowered. That claim requires a completed comparison theorem and a completed acquisition-inclusive upper bound in the same currency. A programme yes also requires the remaining frozen conditions and utility receipt. Journal §32, verdict⁠￼; frozen target §§1a, 7⁠￼.

7. Open choices

[Argued — recommendations.] The sources do not decide the following choices completely. These are specification decisions, not requests for permission and not proposed constructions.

Open choice	Options	Recommendation and reason
Status of the frozen target	Keep P_{108} as mandatory; replace it with an execution criterion; maintain both as separately named questions.	Maintain both explicitly. Preserve the frozen target for honest historical comparison, and version any operational reformulation. Otherwise a change of currency can masquerade as a solution. F⁠￼, M108⁠￼.
Operational currency C^*	Total work; a specified reconstruction component; work above a proved common baseline; a resource vector without scalarisation.	Use the full vector immediately; select a scalar only with explicit transition rules and a proved bridge. The sources do not justify one existing proxy as the operational h_1 floor. M103⁠￼, M107⁠￼, M108⁠￼.
Pass semantics	Input-read passes only; passes over every logical array; a fixed schedule plus byte/word traffic budgets.	Use a fixed traversal schedule plus exact traffic/work budgets, counting workspace traversals too. Specify partial-scan and fusion rules before applying K=2. This prevents both hidden passes and misleading synchronous-depth assumptions. M108⁠￼, M110⁠￼.
Old operational class	Fixed-orientation \mathcal R; an extension covering mixed orientation; a broader class of equivalent implementations.	Start with record 103's exact class and extend it only by proved simulations. Do not assert family-wide coverage from semantic expandability. M103 §§1–2⁠￼.
Hard-family quantifiers	Worst-case fills; expectation; high probability; a novelty-conditioned distribution.	Use universal correctness, per-block validation, and an explicit high-probability resource statement on a fixed text-generated family. State conditioning and failure rate; do not substitute token novelty for randomness. M103 §4⁠￼, N8⁠￼.
Meaning of "non-incremental"	Fixed fractional separation; asymptotic improvement; structural change plus measured utility.	Require structural change plus the fixed quantitative separation; state any asymptotic gain as an additional theorem. A factor-of-two target does not itself imply a new asymptotic complexity class, and "new mathematics" alone does not imply useful cost. J §32⁠￼, F §1⁠￼.
Small-h_1 cases	Apply the half target everywhere; exclude declared degenerate cases from performance claims; add an additive allowance.	Keep correctness universal and declare the performance family before candidate evaluation. Do not silently add an allowance to the frozen inequality. Explicitly state what happens at h_1=0 and n=1. F §1⁠￼; §1.4 above.
Finite bounds and no advice	Concrete bounded-domain theorem only; uniform asymptotic theorem only; both.	Use both. A concrete theorem should match bzip2 and the implementation; a uniform extension should prevent finite-domain lookup arguments from becoming the claimed mathematical basis. F §1a.7⁠￼, P §1⁠￼.
Algorithmic randomness	Deterministic; zero-error randomised; bounded-error decoding.	Default to deterministic. If randomisation is admitted, preserve exact output and state its resource quantifiers separately. CRC acceptance does not replace correctness. M105 §4⁠￼.
Memory and output workspace	Read-only input/write-only output; reusable input/output buffers; unrestricted workspace under the ceiling.	Allow declared reuse with complete accounting. Artificial buffer restrictions can exclude legitimate mechanisms, while unaccounted reuse can hide state and traffic. Keep the pointwise Open ceiling. F §§1–1a⁠￼.
Cache comparison	Simplified per-worker simulation; full concurrent model; empirical misses only.	Retain the frozen clause but distinguish modelled from measured claims. A shared-cache guarantee requires explicit concurrent context; the N-15 approximation cannot supply it alone. F §1a.6b⁠￼, N15⁠￼.
Output proof language	Restrict to \Lambda'; allow additional output rules with charged implementations.	Allow declared, sound extensions. Otherwise the admissible language would exclude the very mechanisms the programme is asking whether it can discover. Keep N′ explicitly scoped to \Lambda'. M107 §2⁠￼, N19⁠￼.
N/R/W decomposition	Mandatory decoder architecture; optional accounting/proof device.	Keep it optional. It is useful for attribution and diagnostics, but a no must quantify over decoders that do not use this decomposition. M103 §3⁠￼, F §6⁠￼.
Engineering threshold	Any statistically established whole-decoder win; the charter's 10% headroom gate; worker-specific acceptance.	Distinguish the pre-prototype headroom gate from final acceptance, and fix both protocols. Retain the default ten-worker result as the primary deployment question, with one-worker results reported separately. Do not turn one supplied-state STOP into a universal theorem. ME, G4⁠￼, J §29⁠￼.
Formal proof deliverable	Prose proof; executable formal statement; checked theorem plus implementation refinement.	First fix executable definitions and the conditional theorem with every residual exposed; then require checked discharge for any claimed formal result. Derivation data and execution traces must retain computational identity and cost; a proposition of correctness alone is insufficient. P §0⁠￼, M105 §1⁠￼, Trust⁠￼.
