← [The mathematical structure of inverse-BWT navigation](<https://lmmx.github.io/mibz2/pr-preview/pr-380/blog/2026-11-08-inverse-bwt-navigation/index.md>)

# Technical appendix: the mathematics of inverse-BWT navigation

This appendix holds the derivations and side results that [the main post](<https://lmmx.github.io/mibz2/pr-preview/pr-380/blog/2026-11-08-inverse-bwt-navigation/index.md>) leaves out to keep its argument moving. It uses the main post's notation, defined in [its section 2](<https://lmmx.github.io/mibz2/pr-preview/pr-380/blog/2026-11-08-inverse-bwt-navigation/#2-why-does-every-run-give-one-shift>): the last column \\(L\\) of length \\(n\\), its \\(\\sigma\\) distinct characters, their frequencies \\(n\_c\\) and starting rows \\(C(c)\\), the first column \\(F\\), rank, select, the next-row map \\(\\Psi\\), and the run count \\(r\\). Equations (1) to (8) are the main post's. This appendix numbers its own (A1) onwards. Its sections can be read in any order. Nothing here has been built or measured in mibz2.

## 1\. Navigation as a select problem

Equation (1) of the main post gives a data-structure reduction:

\\\[ \\boxed{\\text{arbitrary }\\Psi\\text{-query}\\longrightarrow\\text{character-bucket identification}+\\text{select query}.} \\\]

Given a row \\(i\\), determine the unique character \\(c\\) satisfying

\\\[ C(c)\\le i\<C(c)+n\_c. \\\]

Then compute

\\\[ \\Psi(i)=\\operatorname{select}\_c(L,i-C(c)). \\\]

This decomposes the problem into familiar operations on indexed sequences. It means mibz2 can, in principle, represent navigation without storing an explicit successor for every row.

### 1.1 The output character is already known

A T-vector entry conventionally combines the navigation destination and a character. But the character associated with row \\(i\\) is simply \\(F\[i\]\\), which is determined entirely by the histogram:

\\\[ \\boxed{F\[i\]=c\\quad\\text{whenever}\\quad C(c)\\le i\<C(c)+n\_c.}\\tag{A1} \\\]

For a byte alphabet, the character sequence in table order has at most 256 constant intervals and at most 255 changes.

This is an unconditional property of the inverse-BWT table, regardless of whether \\(L\\) is repetitive. **Consequence for mibz2:** the character field does not need to be independently stored in every navigation entry. It may be amortised over groups of rows, or recovered from a bucket directory. This is an important distinction from run-based compression: character sharing remains useful even for highly irregular BWT input.

### 1.2 An exact bound on mixed-character records

Suppose we divide table rows into fixed groups of \\(g\\) consecutive positions. Because \\(F\\) contains only \\(\\sigma\\) character buckets, there are at most \\(\\sigma-1\\) bucket boundaries. Every group containing more than one output character must contain at least one of those boundaries. Thus

\\\[ \\boxed{N\_{\\mathrm{mixed}}\\le \\sigma-1.}\\tag{A2} \\\]

For compact30, \\(g=30\\) and \\(\\sigma\\le256\\), so \\(N\_{\\mathrm{mixed}}\\le255\\). At maximum block size,

\\\[ N\_{\\mathrm{groups}}=\\frac{900{,}000}{30}=30{,}000. \\\]

Hence at most 255 groups can require exceptional handling *solely because their output characters differ*: at most 0.85% of groups, affecting at most 7,650 rows.

This does not limit the number of RAW groups caused by widely separated navigation destinations, but proves that character-bucket crossings are a small, alphabet-bounded issue. **Practical gain:** output-character sharing can be designed around a strict worst-case bound; it does not depend on how repetitive the source text is.

## 2\. Consecutive entries and sparse corrections

### 2.1 The exact count of consecutive navigation entries

Inside any character bucket, define

\\\[ \\Delta(i)=\\Psi(i+1)-\\Psi(i). \\\]

Because the bucket represents increasing occurrence positions, \\(\\Delta(i)\\ge1\\). Moreover, \\(\\Delta(i)=1\\) if and only if the two corresponding occurrences of that character are adjacent in \\(L\\). A run of length \\(m\\) contributes \\(m-1\\) such pairs, giving

\\\[ \\boxed{\\\#\\{i:\\ i,i+1\\text{ lie in the same character bucket},\\ \\Psi(i+1)=\\Psi(i)+1\\}=n-r.}\\tag{A3} \\\]

This is an equality, not an average-case statement. There are \\(n-\\sigma\\) adjacent pairs within buckets, so the exact fraction with consecutive destinations is

\\\[ \\boxed{f\_{\\mathrm{consecutive}}=\\frac{n-r}{n-\\sigma}.}\\tag{A4} \\\]

This gives the BWT run count a precise interpretation as a measure of local navigation irregularity.

Two concepts must not be conflated:

- The number of *descents* of \\(\\Psi\\) (positions where \\(\\Psi(i)\>\\Psi(i+1)\\)) is at most \\(\\sigma-1\\).
- The number of failures of consecutive translation **within buckets** is exactly \\(r-\\sigma\\).

The second quantity is relevant for run-based navigation compression.

### 2.2 A sparse-correction representation

Let the positions of character \\(c\\) in \\(L\\) be

\\\[ x\_{c,0}\<x\_{c,1}\<\\cdots\<x\_{c,n\_c-1}. \\\]

Define excess gaps

\\\[ g\_{c,j}=x\_{c,j}-x\_{c,j-1}-1,\\qquad j\\ge1. \\\]

Then

\\\[ \\boxed{\\Psi(C(c)+k)=x\_{c,0}+k+\\sum\_{j=1}^{k}g\_{c,j}.}\\tag{A5} \\\]

Within runs, excess gaps are zero. A positive excess gap occurs exactly when consecutive occurrences of \\(c\\) belong to different runs. The number of positive gaps is

\\\[ \\boxed{\\sum\_c\\\#\\{j:g\_{c,j}\>0\\}=r-\\sigma.}\\tag{A6} \\\]

Thus the navigation map can be described by one starting destination per character, the positions of \\(r-\\sigma\\) nonzero corrections, their magnitudes, and bucket boundaries.

The query becomes predecessor or prefix-sum over sparse corrections. This is mathematically equivalent to the interval representation, but suggests sparse bitvectors, indexed delta streams, or two-level directories.

**What we have gained:** an exact structural complexity parameter for the map. The important variable is principally \\(r\\), the number of BWT runs, rather than \\(n\\), descents, or Coxeter length (the inversion count, [section 7](<#7-what-does-the-permutree-thesis-contribute>)).

## 3\. Space bounds

### 3.1 Entropy of the stable-sort permutation

Fix the histogram \\((n\_1,\\ldots,n\_\\sigma)\\). The number of possible last-column sequences is

\\\[ M=\\frac{n!}{\\prod\_c n\_c!}. \\\]

There is a bijection between those sequences and permutations increasing within corresponding character buckets. An arbitrary member of the family requires at least

\\\[ \\boxed{B\_{\\min}=\\left\\lceil\\log\_2\\frac{n!}{\\prod\_c n\_c!}\\right\\rceil}\\tag{A7} \\\]

bits in a fixed-length representation. Stirling's approximation gives the leading term

\\\[ \\log\_2 M\\approx nH\_0(L), \\qquad H\_0(L)=-\\sum\_c\\frac{n\_c}{n}\\log\_2\\frac{n\_c}{n}. \\\]

For a byte alphabet \\(H\_0(L)\\le8\\). Thus the stable-sort permutation with fixed histogram contains at most approximately eight bits per row, generally less.

**This bounds representation information, not access latency.** A four-byte T-vector stores redundancy deliberately for cheap successor retrieval.

### 3.2 Elias–Fano gives a constructive representation

Every character bucket contains a strictly increasing sequence of positions. Elias–Fano encoding is designed for such sequences.

For character \\(c\\), a standard Elias–Fano representation uses approximately

\\\[ n\_c\\left\\lceil\\log\_2\\frac{n}{n\_c}\\right\\rceil+2n\_c \\\]

bits before indexing overhead. Summed over all characters,

\\\[ \\boxed{B\_{\\mathrm{EF}}\\lesssim2n+\\sum\_c n\_c\\left\\lceil\\log\_2\\frac{n}{n\_c}\\right\\rceil.}\\tag{A8} \\\]

For an approximately uniform byte distribution this is about \\(10n\\) bits, or approximately 1.125 MB for a 900,000-byte block, versus 3.6 MB for the T-vector.

Elias–Fano supports indexed access to elements without decoding all predecessors; theoretical constant-time access depends on suitable select indexing. It provides a formal alternative even when \\(r\\) is large. It does not exploit long runs as aggressively as a run-based representation, since its general bound depends on symbol frequencies.

### 3.3 A separate information bound from run count

Count length-\\(n\\) words over an alphabet of size \\(\\sigma\\) with exactly \\(r\\) runs. Choose positive run lengths in \\(\\binom{n-1}{r-1}\\) ways, the first character in \\(\\sigma\\) ways, and each subsequent character in \\(\\sigma-1\\) ways:

\\\[ \\boxed{N(n,r,\\sigma)=\\sigma(\\sigma-1)^{r-1}\\binom{n-1}{r-1}.}\\tag{A9} \\\]

The bits required to enumerate this broader family are

\\\[ \\boxed{B\_{\\mathrm{run}}= \\left\\lceil\\log\_2\\sigma+(r-1)\\log\_2(\\sigma-1)+\\log\_2\\binom{n-1}{r-1}\\right\\rceil.}\\tag{A10} \\\]

For \\(r\\ll n\\), this is \\(O(r\\log(n/r)+r\\log\\sigma)\\).

This counts all words with specified run count, not only valid BWT images or fixed-histogram words. It is an achievable representation-size benchmark for the larger family, not a tight bound for bzip2 blocks.

### 3.4 Two independent measures of compressibility

The two measures

\\\[ \\boxed{H\_0(L)}\\qquad\\text{and}\\qquad\\boxed{r} \\\]

answer different questions. Low entropy means concentrated character frequency; low \\(r\\) means few constant BWT runs. A block can have low entropy and many short runs, or high entropy and few long runs.

| Property | Mathematical implication | Relevant structure |
| --- | --- | --- |
| Low \\(H\_0(L)\\) | Fewer stable-sort arrangements | Entropy-based sequence encodings |
| Low \\(r\\) | Few translated navigation intervals | Run descriptors and move structures |
| Small gaps between occurrences | Narrow destination spans | Base-plus-offset records |
| Small \\(\\sigma\\) | Few output-byte bucket boundaries | Shared-character headers |

There need not be one universally optimal compact representation.

## 4\. What the intervals say about compact30

mibz2's compact30 research ([Building a smaller bzip2 decoding table](<https://lmmx.github.io/mibz2/pr-preview/pr-380/blog/2026-11-04-compact30-constructor/index.md>)) stores thirty rows per 64-byte record, hence the name, with exceptional RAW records that hold full entries, and attempts to reduce the working set without sacrificing memory-level parallelism.

### 4.1 The ordinary-record condition

compact30 groups thirty consecutive rows into 64-byte records and uses a label transform

\\\[ \\varphi(i)=32\\left\\lfloor\\frac i{30}\\right\\rfloor+(i\\bmod30) \\\]

or

\\\[ \\boxed{\\varphi(i)=i+2\\left\\lfloor\\frac i{30}\\right\\rfloor.}\\tag{A11} \\\]

Two unused labels per group correspond to the record header. For a complete group \\(G=\[a,a+30)\\), ordinary-record conditions are a shared output character and labelled successor offsets fitting in 16 bits. Since \\(\\Psi\\) and \\(\\varphi\\) increase within a bucket, the endpoints suffice.

Ignoring special padding and sentinel cases:

\\\[ \\boxed{F\[a\]=F\[a+29\]} \\\]

and

\\\[ \\boxed{\\varphi(\\Psi(a+29))-\\varphi(\\Psi(a))\\le65{,}533.}\\tag{A12} \\\]

This bounds *encoded successor-label span*, not necessarily raw row-index difference. It captures monotonicity used by the existing builder.

### 4.2 Gap-based interpretation

Suppose the group lies entirely in bucket \\(c\\), with first destination \\(x\\) and last destination \\(y\\). Between the thirty occurrences,

\\\[ y-x=29+\\\#\\{j:x\<j\<y,\\ L\[j\]\\neq c\\}. \\\]

Therefore

\\\[ \\boxed{\\Psi(a+29)-\\Psi(a)=29+h,}\\tag{A13} \\\]

where \\(h\\) is the count of intervening non-\\(c\\) characters. The labelled span is

\\\[ \\begin{aligned} W={}&amp;29+h\\\\ &amp;+2\\left(\\left\\lfloor\\frac y{30}\\right\\rfloor-\\left\\lfloor\\frac x{30}\\right\\rfloor\\right). \\end{aligned}\\tag{A14} \\\]

Apart from bucket crossings, compact30 RAW groups arise from too many intervening characters among thirty occurrences. It is occurrence-gap arrangement, not absolute destination position, that matters.

### 4.3 Guaranteed ordinary groups

If a complete 30-row group lies inside one translation interval,

\\\[ \\Psi(a+t)=p+t,\\qquad0\\le t\<30. \\\]

The raw span is exactly 29, and the labelled span at most 31. The group is guaranteed to fit an ordinary compact30 record. Its offsets form a precomputable pattern, precisely the bulk-emission case in the existing constructor, where the builder writes a whole record from a precomputed pattern because one run of \\(L\\) fills the group.

### 4.4 Exact formula for pure groups

Let translation intervals be \\(I\_t=\[a\_t,b\_t)\\). For group width \\(g\\), the number of complete aligned groups contained in interval \\(I\_t\\) is

\\\[ G\_t=\\max\\left(0,\\left\\lfloor\\frac{b\_t}{g}\\right\\rfloor-\\left\\lceil\\frac{a\_t}{g}\\right\\rceil\\right). \\\]

Therefore

\\\[ \\boxed{G\_{\\mathrm{pure}}=\\sum\_{t=1}^{r}G\_t.}\\tag{A15} \\\]

For compact30,

\\\[ \\boxed{R\_{\\mathrm{pure}}=30G\_{\\mathrm{pure}}.}\\tag{A16} \\\]

This is computable from BWT run boundaries and character ranks without building the entire navigation table.

Every complete group not inside one translation interval crosses a boundary. There are at most \\(r-1\\) such boundaries, each inside at most one aligned group:

\\\[ \\boxed{N\_{\\mathrm{nonpure}}\\le r-1.}\\tag{A17} \\\]

The bound may be loose and non-pure groups can still be ordinary records. But when \\(r\\ll n/g\\), almost all complete groups are guaranteed pure.

### 4.5 Consequence for construction

BWT runs determine destination intervals. Record boundaries can be enumerated in row order, with endpoint spans found by sequential sweeps through run descriptors. Once runs are known, group classification can take

\\\[ \\boxed{O\\left(r+\\frac ng+\\sigma\\right)} \\\]

time without materialising every successor. A scan of \\(L\\) can then scatter final entries into classified records. This gives an \\(O(n+r+n/g)\\)-time construction strategy with appropriate run, record and cursor storage.

It **does not prove** lower instruction count or fewer writes than the fused builder. It proves that record classification need not be interleaved with every per-row scatter: structure permits classification at the interval and group level.

So compact30's exceptions can be bounded and predicted. Character crossings are bounded by 255 groups; pure-run groups can be counted from (A15); ordinary eligibility is expressed by (A12). This enables interval/group-level classification and verification rather than opaque per-row heuristics.

## 5\. Several steps at once

### 5.1 Powers of the navigation permutation

Conventional traversal visits \\(i,\\Psi(i),\\Psi^2(i),\\ldots\\). A multi-step transition computes \\(\\Psi^k(i)\\) while emitting the \\(k\\) intermediate characters:

\\\[ W\_k(i)=F\[i\]F\[\\Psi(i)\]\\cdots F\[\\Psi^{k-1}(i)\]. \\\]

The desired operation is

\\\[ \\boxed{i\\longmapsto\\left(W\_k(i),\\Psi^k(i)\\right).}\\tag{A18} \\\]

For a general permutation this could need \\(n\\) stored entries; an interval-exchange permutation gives a better bound.

### 5.2 Bound on pieces of \\(\\Psi^k\\)

Start with a partition \\(P\\) into \\(r\\) translated intervals, each with constant output byte; there are \\(r-1\\) boundaries. Refine it so that two successive steps follow fixed source intervals. Splits occur at preimages of original boundaries. Since \\(\\Psi\\) is bijective and translates each existing piece, images of the pieces are disjoint intervals partitioning the destination domain. Each original boundary introduces at most one new split across the whole refined partition. Each further step adds at most \\(r-1\\) boundaries. Thus

\\\[ \\boxed{N\_k\\le\\min\\{n,\\ 1+k(r-1)\\}.}\\tag{A19} \\\]

On every refined interval, \\(\\Psi^k\\) is a translation and \\(W\_k\\) is constant.

For two and four steps respectively:

\\\[ \\boxed{N\_2\\le2r-1,\\qquad N\_4\\le4r-3.} \\\]

This bound becomes uninformative when \\(k(r-1)\\) approaches \\(n\\).

### 5.3 Multi-character transition table

Each interval stores its starting row, displacement to \\(\\Psi^k(i)\\), and the emitted \\(k\\)-character word. With interval identifier, output retrieval and destination computation are straightforward. An explicit representation uses

\\\[ \\boxed{O\\left(kr(\\log n+k\\log\\sigma)\\right)}\\tag{A20} \\\]

bits when \\(k(r-1)\\ll n\\). For fixed small \\(k\\), that is \\(O(kr\\log n)\\).

This proves a compact representation for the *function*, not constant-latency arbitrary interval lookup. Multi-character navigation need not inherently require an \\(n\\)-entry jump table.

### 5.4 Implication for mibz2

[Kärkkäinen, Kempa and Puglisi's 2012 work](<https://researchportal.helsinki.fi/en/publications/slashing-the-time-for-bwt-inversion/>) studied reducing cache misses through multi-character BWT navigation, including two characters per lookup. mibz2 cites it as [prior art](<https://lmmx.github.io/mibz2/pr-preview/pr-380/concepts/prior-work/index.md>). mibz2 has also built such a table: the pair walk of [PR \#34](<https://github.com/lmmx/mibz2/pull/34>), whose entries hold the row two steps on, was measured and dropped, because each step cost about twice as much and the table took about three times as long to build ([Why 32 couriers beat one](<https://lmmx.github.io/mibz2/pr-preview/pr-380/concepts/couriers/index.md>)). The interval view suggests representing \\(k\\)-step transitions by run-sensitive translated intervals instead of a complete super-alphabet table.

The number of intervals increases at most linearly with \\(kr\\). For repetitive blocks it can be substantially below \\(n\\), though it may become large for run-rich blocks. Interval identification, segment boundaries and output ordering remain essential. A mathematical \\(k\\)-step operation does not imply one cache miss or CPU instruction.

So the pair \\((\\Psi^k,W\_k)\\) has at most \\(1+k(r-1)\\) translated, constant-output intervals. This establishes a compact *theoretical* representation of multi-character decoding transitions when \\(kr\\ll n\\), without guaranteeing CPU speedup.

## 6\. Cycle structure and necklaces

### 6.1 Cycle count

Let \\(\\operatorname{cycles}(\\Psi\_L)\\) be the number of permutation cycles, and \\(m\_1,\\ldots,m\_r\\) the maximal run lengths in \\(L\\). A characterisation due to Likhomanov and Shur, reported in [a survey of the bijective BWT](<https://drops.dagstuhl.de/storage/01oasics/oasics-vol131-manzinis-festschrift/html/OASIcs.Manzini.2/OASIcs.Manzini.2.html>), states

\\\[ \\boxed{L\\text{ is a valid cyclic BWT image}\\iff\\operatorname{cycles}(\\Psi\_L)=\\gcd(m\_1,\\ldots,m\_r).}\\tag{A21} \\\]

For primitive input, the permutation is one cycle. Periodic inputs may have several. We must not assume every bzip2 traversal visits all distinct rows in one cycle.

### 6.2 Necklaces and exact enumeration

The BWT identifies cyclic rotations, so its images correspond naturally to *necklaces*. With counts \\(n\_1,\\ldots,n\_\\sigma\\) and \\(g=\\gcd(n\_1,\\ldots,n\_\\sigma)\\), Burnside's lemma gives

\\\[ \\boxed{N\_{\\mathrm{BWT}}=\\frac1n\\sum\_{d\\mid g}\\phi(d)\\frac{(n/d)!}{\\prod\_c(n\_c/d)!}.}\\tag{A22} \\\]

The number of primitive necklaces is

\\\[ \\boxed{N\_{\\mathrm{primitive}}=\\frac1n\\sum\_{d\\mid g}\\mu(d)\\frac{(n/d)!}{\\prod\_c(n\_c/d)!}.}\\tag{A23} \\\]

Here \\(\\phi\\) is Euler's totient and \\(\\mu\\) the Möbius function.

### 6.3 Coprime-frequency simplification

When \\(g=1\\), all source strings with that histogram are primitive:

\\\[ \\boxed{N\_{\\mathrm{BWT}}=\\frac1n\\frac{n!}{\\prod\_c n\_c!}.}\\tag{A24} \\\]

The BWT restriction reduces the stable-sort family by a factor of exactly \\(n\\); the information saving is

\\\[ \\boxed{\\log\_2n\\text{ bits}.}\\tag{A25} \\\]

At \\(n=900{,}000\\), this is approximately 19.78 bits for the entire block. Identifying the original rotation requires a primary index with roughly the same amount of information.

Cycle structure is useful for validity checks, periodicity, testing and traversal assumptions, but it is not a promising independent source of major storage savings. The stable-sort restriction accounts for most of the representational reduction.

## 7\. What does the permutree thesis contribute?

[Tamayo Jiménez](<https://arxiv.org/abs/2310.19732>) studies weak-order lattices, permutree congruences, pattern-avoiding permutations, and related geometry. The inverse-BWT permutation belongs to a classical parabolic quotient of the symmetric group.

Let

\\\[ J=\\{s\_i:\\ i,i+1\\text{ lie in the same character bucket}\\}. \\\]

Then

\\\[ \\boxed{\\Psi\\in W^J}, \\\]

where \\(W^J\\) consists of permutations with no right descents at positions in \\(J\\).

### 7.1 Weak-order inversion statistic

The Coxeter length is

\\\[ \\ell(\\Psi)=\\\#\\{(i,j):i\<j,\\ \\Psi(i)\>\\Psi(j)\\}. \\\]

Stable sorting implies

\\\[ \\boxed{\\ell(\\Psi)=\\\#\\{(i,j):i\<j,\\ L\[i\]\>L\[j\]\\}.}\\tag{A26} \\\]

This gives an exact algebraic rank, but it should not be confused with BWT run count.

### 7.2 Large inversion count can coexist with easy navigation

For \\(L=b^ma^m\\), \\(a\<b\\),

\\\[ \\Psi(i)=\\begin{cases}i+m,&amp;0\\le i\<m,\\\\i-m,&amp;m\\le i\<2m.\\end{cases} \\\]

There are just \\(r=2\\) runs but

\\\[ \\boxed{\\ell(\\Psi)=m^2}. \\\]

Thus quadratic Coxeter length coexists with two translated intervals. This is a valid cyclic BWT image of an alternating periodic word.

Weak-order geometry measures distance from identity under adjacent transpositions; it does not measure cost of successor queries.

### 7.3 Meets and joins

The parabolic-quotient family has weak-order meets and joins, but mibz2 evaluates one fixed permutation rather than optimising over permutations. Meets and joins need not preserve valid BWT images. The permutree thesis does not directly imply a faster decoder; its relevance is the algebraic setting, while useful data structures come from stable sorting, succinct sequences and run-bounded interval permutations.

So weak-order geometry provides context, not the central optimisation. Parabolic quotients explain increasing buckets and inversion structure, but the objects the decoder actually evaluates are

\\\[ \\boxed{\\operatorname{select}\_c(L,k),\\quad\\text{translated intervals},\\quad\\text{move queries},\\quad\\Psi^k.} \\\]

These admit operational data structures and meaningful complexity bounds.

## 8\. Correctness checks and oracles

The mathematical properties support independent invariants:

| Invariant | Exact property | What it checks |
| --- | --- | --- |
| Permutation | \\(\\Psi\\) bijective on \\(\[0,n)\\) | No missing or duplicated destinations |
| Stable-sort identity | Equation (1) of the main post | Correct correspondence with \\(L\\) |
| Bucket monotonicity | \\(\\Psi(i+1)\>\\Psi(i)\\) within buckets | Counting-sort ordering |
| Run structure | Exactly \\(n-r\\) within-bucket adjacent pairs advance by one | Correct run correspondence |
| Interval translation | Equation (2) of the main post, on every run | Correct interval descriptors |
| compact30 classification | Equation (A12) | Ordinary/RAW representation decisions |
| BWT cycle validity | Equation (A21) | Valid cyclic-BWT structure |
| Multi-step navigation | Equation (A18) | Correct jump destinations and emitted words |

These diagnose internal errors more precisely than end-to-end decoded-output comparison.

### 8.1 Independent oracle

Given \\(L\\) and its histogram, construct the reference mapping through equation (1) of the main post. For every row \\(i\\) verify

\\\[ \\Psi\_{\\mathrm{candidate}}(i)=\\operatorname{select}\_{F\[i\]}(L,i-C(F\[i\])). \\\]

This is a complete equivalence check, stronger than traversal from the primary index alone (which need not visit all distinct rows for periodic blocks). For \\(k\\)-step representations verify

\\\[ \\Psi\_{\\mathrm{candidate}}^{(k)}(i)=\\Psi^k(i) \\\]

and

\\\[ W\_{k,\\mathrm{candidate}}(i)=W\_k(i). \\\]

These are development oracles, not obligatory checks in production decoding.

## 9\. The design space and the results

[Section 6 of the main post](<https://lmmx.github.io/mibz2/pr-preview/pr-380/blog/2026-11-08-inverse-bwt-navigation/#6-what-would-it-cost-mibz2-and-what-would-settle-it>) keeps three resources apart: navigation storage, construction work and critical-path memory dependencies. mibz2's compact30 work reduced footprint and benefited concurrent traversal, but direct construction remained expensive; the [4 November post](<https://lmmx.github.io/mibz2/pr-preview/pr-380/blog/2026-11-04-compact30-constructor/index.md>) says bulk emission covered 74.3% of rows on the main Wikidata corpus. The [7 November pipelining work](<https://lmmx.github.io/mibz2/pr-preview/pr-380/blog/2026-11-07-pipelined-teams/index.md>) overlaps a block's construction and traversal with other stages of neighbouring blocks: a team's leader runs the next block's MTF and finishes the previous block while its helpers build and walk the current one.

So mibz2 need not choose only between the complete T-vector and packed per-row T-vectors. Move structures store interval transitions and support repeated navigation. When \\(r\\) is small they offer a theoretical reduction in live navigation storage, with established constructions and query guarantees. The representations the theory offers, with what each costs:

| Representation | Space for navigation | Access complexity | Main consideration |
| --- | --- | --- | --- |
| Conventional T-vector | \\(32n\\) bits | One direct lookup | High footprint |
| Packed destinations | About \\(20n\\) bits, plus symbols/metadata | Direct addressing | Bit extraction |
| Per-character Elias–Fano | About \\(nH\_0(L)+O(n)\\) bits | Indexed select | More dependencies |
| Simple run descriptors | \\(O(r\\log n)\\) bits | \\(O(\\log r)\\) predecessor | Repeated searches |
| Balanced move structure | \\(O(r)\\) words | \\(O(1)\\) with interval state | Interval transitions |
| Length-capped move structure | \\(O(r)\\) words | Amortised \\(O(1)\\) over distinct full-cycle visits | Coverage assumptions |
| \\(k\\)-step intervals | \\(O(kr(\\log n+k\\log\\sigma))\\) bits | Interval-dependent | Multi-character operation |

The results, in one table:

| Gain | Established mathematical result | Value to mibz2 |
| --- | --- | --- |
| Stable-sort representation | \\(\\Psi\\) is a collection of character-select sequences | Succinct sequence structures |
| Run-bounded representation | \\(\\Psi\\) has at most \\(r\\) translated intervals | Space scaling with \\(r\\) |
| Move structures | \\(O(1)\\) stateful moves in \\(O(r)\\) words | Alternative to full T-vector |
| Linear-time inversion | \\(O(n)\\) time and \\(O(r)\\) additional RLBWT working space under assumptions | End-to-end theoretical feasibility |
| Fixed-record identities | Exact character crossings, spans and pure-group bounds | Formal compact30 analysis |
| Multi-step interval bound | At most \\(1+k(r-1)\\) pieces for \\(k\\)-step navigation | Compact multi-character transitions |

---

**Sources:** as listed in [the main post](<https://lmmx.github.io/mibz2/pr-preview/pr-380/blog/2026-11-08-inverse-bwt-navigation/#7-the-argument-in-brief>).
