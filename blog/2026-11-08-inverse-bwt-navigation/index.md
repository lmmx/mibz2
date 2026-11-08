[mibz2 · Research blog](<https://lmmx.github.io/mibz2/blog/index.md>)

# The mathematical structure of inverse-BWT navigation

The inverse BWT's next-row map is one shift per run of equal bytes, so it can be stored per run rather than per row. Whether that beats mibz2's table is unproven.

8 November 2026 · 24 min read · [Open question](<#13-final-assessment>)

To undo the [Burrows–Wheeler transform](<https://en.wikipedia.org/wiki/Burrows–Wheeler_transform>) (BWT), a bzip2 decoder follows a chain through each block: from one row of the transform's sorted matrix to the next, emitting one byte at each step. The inverse Burrows–Wheeler transform in mibz2, our bzip2 decoder, is conventionally implemented using a table containing one navigation entry per BWT row, which we call the T-vector. Each entry is four bytes, holding the next row and the byte to emit. At the maximum bzip2 block size of 900,000 bytes, the conventional four-byte T-vector occupies 3.6 MB. The inverse BWT is the expensive part of decoding a block, because each step's address comes from the entry just read; mibz2 walks many such chains at once so that their memory reads overlap. [An earlier post](<https://lmmx.github.io/mibz2/blog/2026-11-02-revisiting-space-time-tradeoff-bwt-inversion/index.md>) works the transform through on a small example.

This post asks what kind of map that table encodes. The navigation function is not an arbitrary permutation. It has three layers of mathematical structure:

1. It is a stable-sort permutation, equivalently a collection of monotone occurrence-position sequences.
2. It is a *piecewise translation* with at most \\(r\\) pieces, where \\(r\\) is the number of equal-character runs in the BWT last column.
3. For a genuine cyclic BWT, its cycle decomposition is constrained by the periodicity of the original string.

These properties establish substantially more than nomenclature. They lead to exact representation formulas, space bounds, algorithms for arbitrary successor queries, and constructive results about multiple navigation steps.

Most importantly, **there is already a published theory of data structures for precisely this kind of permutation**.

[Nishimoto and Tabei (2021)](<https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICALP.2021.101>) established constant-time navigation using \\(O(r)\\) machine words for *move structures*, provided the current interval identifier is maintained. More recently, [Brown and Langmead (2026)](<https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/html/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.html>) established \\(O(n)\\)-time BWT inversion with \\(O(r)\\) additional working space from a run-length BWT representation. Those are genuine algorithmic results, not performance conjectures. They provide a direct theoretical alternative to mibz2's \\(O(n)\\)-entry T-vector.

**What is not proved** is that any of them beats mibz2's 32-lane, four-byte-per-row implementation in elapsed CPU cycles. Everything in this post is mathematics and published theory: nothing in it has been built or measured in mibz2, and it changes nothing in the decoder.

The post derives the consequences for mibz2, including what the theory tells us about compact30 (our research table, which packs thirty rows into each 64-byte record), which approaches have provable guarantees, how multi-character traversal can exploit the same structure, and which combinatorial identities do not materially help. [Section 1](<#1-establishing-the-exact-object>) defines the map and works it on `banana`. Sections [2](<#2-first-consequence-this-is-a-select-problem-not-a-general-permutation-problem>) to [4](<#4-third-consequence-rigorous-succinct-space-bounds>) derive its structure: a select problem, \\(r\\) translated intervals, and the space bounds that follow. [Section 5](<#5-fourth-consequence-move-structures-already-solve-the-theoretical-navigation-problem>) is the published theory of move structures, and [section 6](<#6-fifth-consequence-compact30s-representability-can-be-characterised-exactly>) applies the interval view to compact30. [Section 7](<#7-sixth-consequence-multiple-bwt-steps-admit-a-small-interval-representation>) covers several steps at once, [section 8](<#8-seventh-consequence-cycle-structure-distinguishes-actual-bwt-images>) the cycle structure, and [section 9](<#9-what-does-the-original-permutree-thesis-contribute>) what the permutree thesis this question started from contributes. Sections [10](<#10-stronger-correctness-guarantees-and-proof-obligations>) to [13](<#13-final-assessment>) turn the results into checks, a map of the design space, and an assessment.

## 1\. Establishing the exact object

### 1.1 Definitions and orientation

Let

\\\[ L=L\[0\]L\[1\]\\cdots L\[n-1\] \\\]

be the BWT last column of a block, and let

\\\[ \\Sigma=\\{c\_1\<c\_2\<\\cdots\<c\_\\sigma\\} \\\]

be its distinct characters, ordered by byte value. Write \\(n\_c\\) for the frequency of character \\(c\\), and define

\\\[ C(c)=\\sum\_{d\<c}n\_d. \\\]

The first column of the conceptual sorted BWT matrix is

\\\[ F=c\_1^{n\_{c\_1}}c\_2^{n\_{c\_2}}\\cdots c\_\\sigma^{n\_{c\_\\sigma}}. \\\]

Define zero-based rank and select by

\\\[ \\operatorname{rank}\_c(L,j)=|\\{i\<j:L\[i\]=c\\}| \\\]

and

\\\[ \\operatorname{select}\_c(L,k)=\\text{position of the }(k+1)\\text{st occurrence of }c. \\\]

The usual backward LF mapping is

\\\[ \\operatorname{LF}(j)=C(L\[j\])+\\operatorname{rank}\_{L\[j\]}(L,j). \\\]

mibz2 uses the inverse direction, which we shall consistently call \\(\\Psi\\):

\\\[ \\boxed{\\Psi=\\operatorname{LF}^{-1}} \\\]

and therefore

\\\[ \\boxed{\\Psi(C(c)+k)=\\operatorname{select}\_c(L,k).}\\tag{1} \\\]

For each row \\(i\\), mibz2 needs to recover the next row \\(\\Psi(i)\\) and emit the byte \\(F\[i\]\\), while preserving the bookkeeping required by its segmented traversal: mibz2 cuts each block's walk into segments that start at known rows, walks them in parallel and joins them. That is the mathematical operation being optimised.

mibz2 currently uses 32 interleaved traversal lanes, each following one segment, so that 32 independent memory reads can be in flight at once ([32 lanes, 8.6 chains](<https://lmmx.github.io/mibz2/blog/2026-10-31-room-walk-overlap/index.md>)), and it supports parallel work across blocks. Its compact30 research ([Building a smaller bzip2 decoding table](<https://lmmx.github.io/mibz2/blog/2026-11-04-compact30-constructor/index.md>)) stores thirty rows per 64-byte record, hence the name, with exceptional RAW records that hold full entries, and attempts to reduce the working set without sacrificing memory-level parallelism.

### 1.2 Example: banana

For the cyclic BWT of banana:

\\\[ L=\\texttt{nnbaaa},\\qquad F=\\texttt{aaabnn}. \\\]

The navigation permutation is

\\\[ \\Psi=\\begin{pmatrix}0&amp;1&amp;2&amp;3&amp;4&amp;5\\\\3&amp;4&amp;5&amp;2&amp;0&amp;1\\end{pmatrix}. \\\]

| Character | Domain of \\(\\Psi\\) | Destinations |
| --- | --- | --- |
| a | \\(0,1,2\\) | \\(3,4,5\\) |
| b | \\(3\\) | \\(2\\) |
| n | \\(4,5\\) | \\(0,1\\) |

The original row is \\(I=3\\). Emitting \\(F\[i\]\\) and following \\(\\Psi\\) produces the rows

\\\[ 3\\to2\\to5\\to1\\to4\\to0 \\\]

and the output banana. The six successor values are determined by three simple translations:

\\\[ \\Psi(i)=\\begin{cases}i+3,&amp;0\\le i\<3,\\\\i-1,&amp;i=3,\\\\i-4,&amp;4\\le i\<6.\\end{cases} \\\]

This example has only three runs in \\(L\\), and three translated intervals in \\(\\Psi\\). That correspondence generalises exactly.

## 2\. First consequence: this is a select problem, not a general permutation problem

Equation (1) gives a data-structure reduction:

\\\[ \\boxed{\\text{arbitrary }\\Psi\\text{-query}\\longrightarrow\\text{character-bucket identification}+\\text{select query}.} \\\]

Given a row \\(i\\), determine the unique character \\(c\\) satisfying

\\\[ C(c)\\le i\<C(c)+n\_c. \\\]

Then compute

\\\[ \\Psi(i)=\\operatorname{select}\_c(L,i-C(c)). \\\]

This decomposes the problem into familiar operations on indexed sequences. It means mibz2 can, in principle, represent navigation without storing an explicit successor for every row.

### 2.1 The output character is already known

A T-vector entry conventionally combines the navigation destination and a character. But the character associated with row \\(i\\) is simply \\(F\[i\]\\), which is determined entirely by the histogram:

\\\[ \\boxed{F\[i\]=c\\quad\\text{whenever}\\quad C(c)\\le i\<C(c)+n\_c.}\\tag{2} \\\]

For a byte alphabet, the character sequence in table order has at most 256 constant intervals and at most 255 changes.

This is an unconditional property of the inverse-BWT table, regardless of whether \\(L\\) is repetitive. **Consequence for mibz2:** the character field does not need to be independently stored in every navigation entry. It may be amortised over groups of rows, or recovered from a bucket directory. This is an important distinction from run-based compression: character sharing remains useful even for highly irregular BWT input.

### 2.2 An exact bound on mixed-character records

Suppose we divide table rows into fixed groups of \\(g\\) consecutive positions. Because \\(F\\) contains only \\(\\sigma\\) character buckets, there are at most \\(\\sigma-1\\) bucket boundaries. Every group containing more than one output character must contain at least one of those boundaries. Thus

\\\[ \\boxed{N\_{\\mathrm{mixed}}\\le \\sigma-1.}\\tag{3} \\\]

For compact30, \\(g=30\\) and \\(\\sigma\\le256\\), so \\(N\_{\\mathrm{mixed}}\\le255\\). At maximum block size,

\\\[ N\_{\\mathrm{groups}}=\\frac{900{,}000}{30}=30{,}000. \\\]

Hence at most 255 groups can require exceptional handling *solely because their output characters differ*: at most 0.85% of groups, affecting at most 7,650 rows.

This does not limit the number of RAW groups caused by widely separated navigation destinations, but proves that character-bucket crossings are a small, alphabet-bounded issue. **Practical gain:** output-character sharing can be designed around a strict worst-case bound; it does not depend on how repetitive the source text is.

## 3\. Second consequence: the permutation consists of exactly \\(r\\) translated intervals

Let \\(r\\) be the number of maximal equal-character runs in \\(L\\), and \\(r\_c\\) the number of runs containing character \\(c\\):

\\\[ r=\\sum\_{c\\in\\Sigma}r\_c. \\\]

### 3.1 The interval theorem

Consider a run of character \\(c\\) beginning at position \\(p\\), with length \\(m\\):

\\\[ L\[p:p+m\]=c^m. \\\]

Let

\\\[ k=\\operatorname{rank}\_c(L,p) \\\]

be the number of earlier occurrences of \\(c\\). The run corresponds to the following interval of rows in \\(F\\):

\\\[ a=C(c)+k. \\\]

By the stable-sort identity, for every \\(0\\le t\<m\\),

\\\[ \\Psi(a+t)=p+t. \\\]

Therefore

\\\[ \\boxed{\\Psi(i)=i+(p-a),\\qquad a\\le i\<a+m.}\\tag{4} \\\]

The entire run is represented by one constant displacement. Every character occurrence belongs to exactly one run. Their corresponding intervals partition the domain of \\(\\Psi\\):

\\\[ \\boxed{\\Psi\\text{ admits a partition into }r\\text{ intervals, on each of which it is a translation.}}\\tag{5} \\\]

In combinatorics, this is a *discrete interval-exchange permutation*. This identification is more operationally useful than membership in a weak-order lattice, because it tells us exactly how to evaluate the permutation.

### 3.2 A constructive representation

For each interval \\(t\\), store its starting position \\(a\_t\\) and first destination \\(p\_t\\). Order the intervals by their starting positions:

\\\[ 0=a\_0\<a\_1\<\\cdots\<a\_r=n. \\\]

Then

\\\[ \\boxed{\\Psi(i)=p\_t+(i-a\_t)\\quad\\text{for}\\quad a\_t\\le i\<a\_{t+1}.}\\tag{6} \\\]

No per-row destination array is required.

For \\(n\\le900{,}000\\), each position fits in 20 bits. The basic representation uses approximately

\\\[ B\_{\\mathrm{interval}}\\approx40r\\text{ bits} \\\]

for two 20-bit values per interval, plus indexing and metadata. Compare this with

\\\[ B\_{\\mathrm{T}}=32n\\text{ bits}. \\\]

Ignoring indexing overhead, the interval representation is smaller whenever \\(40r\<32n\\), or

\\\[ \\boxed{\\frac rn\<0.8.} \\\]

This is only a storage comparison for basic descriptors, not a complete performance or working-memory bound. A practical move structure needs additional information to locate the next interval. Nevertheless, it proves there is a potentially much smaller, exact representation whenever the BWT contains sufficiently long runs.

### 3.3 The exact count of consecutive navigation entries

Inside any character bucket, define

\\\[ \\Delta(i)=\\Psi(i+1)-\\Psi(i). \\\]

Because the bucket represents increasing occurrence positions, \\(\\Delta(i)\\ge1\\). Moreover, \\(\\Delta(i)=1\\) if and only if the two corresponding occurrences of that character are adjacent in \\(L\\). A run of length \\(m\\) contributes \\(m-1\\) such pairs, giving

\\\[ \\boxed{\\\#\\{i:\\ i,i+1\\text{ lie in the same character bucket},\\ \\Psi(i+1)=\\Psi(i)+1\\}=n-r.}\\tag{7} \\\]

This is an equality, not an average-case statement. There are \\(n-\\sigma\\) adjacent pairs within buckets, so the exact fraction with consecutive destinations is

\\\[ \\boxed{f\_{\\mathrm{consecutive}}=\\frac{n-r}{n-\\sigma}.}\\tag{8} \\\]

This gives the BWT run count a precise interpretation as a measure of local navigation irregularity.

Two concepts must not be conflated:

- The number of *descents* of \\(\\Psi\\) (positions where \\(\\Psi(i)\>\\Psi(i+1)\\)) is at most \\(\\sigma-1\\).
- The number of failures of consecutive translation **within buckets** is exactly \\(r-\\sigma\\).

The second quantity is relevant for run-based navigation compression.

### 3.4 A sparse-correction representation

Let the positions of character \\(c\\) in \\(L\\) be

\\\[ x\_{c,0}\<x\_{c,1}\<\\cdots\<x\_{c,n\_c-1}. \\\]

Define excess gaps

\\\[ g\_{c,j}=x\_{c,j}-x\_{c,j-1}-1,\\qquad j\\ge1. \\\]

Then

\\\[ \\boxed{\\Psi(C(c)+k)=x\_{c,0}+k+\\sum\_{j=1}^{k}g\_{c,j}.}\\tag{9} \\\]

Within runs, excess gaps are zero. A positive excess gap occurs exactly when consecutive occurrences of \\(c\\) belong to different runs. The number of positive gaps is

\\\[ \\boxed{\\sum\_c\\\#\\{j:g\_{c,j}\>0\\}=r-\\sigma.}\\tag{10} \\\]

Thus the navigation map can be described by one starting destination per character, the positions of \\(r-\\sigma\\) nonzero corrections, their magnitudes, and bucket boundaries.

The query becomes predecessor or prefix-sum over sparse corrections. This is mathematically equivalent to the interval representation, but suggests sparse bitvectors, indexed delta streams, or two-level directories.

**What we have gained:** an exact structural complexity parameter for the map. The important variable is principally \\(r\\), the number of BWT runs, rather than \\(n\\), descents, or Coxeter length (the inversion count, section 9).

## 4\. Third consequence: rigorous succinct-space bounds

### 4.1 Entropy of the stable-sort permutation

Fix the histogram \\((n\_1,\\ldots,n\_\\sigma)\\). The number of possible last-column sequences is

\\\[ M=\\frac{n!}{\\prod\_c n\_c!}. \\\]

There is a bijection between those sequences and permutations increasing within corresponding character buckets. An arbitrary member of the family requires at least

\\\[ \\boxed{B\_{\\min}=\\left\\lceil\\log\_2\\frac{n!}{\\prod\_c n\_c!}\\right\\rceil}\\tag{11} \\\]

bits in a fixed-length representation. Stirling's approximation gives the leading term

\\\[ \\log\_2 M\\approx nH\_0(L), \\qquad H\_0(L)=-\\sum\_c\\frac{n\_c}{n}\\log\_2\\frac{n\_c}{n}. \\\]

For a byte alphabet \\(H\_0(L)\\le8\\). Thus the stable-sort permutation with fixed histogram contains at most approximately eight bits per row, generally less.

**This bounds representation information, not access latency.** A four-byte T-vector stores redundancy deliberately for cheap successor retrieval.

### 4.2 Elias–Fano gives a constructive representation

Every character bucket contains a strictly increasing sequence of positions. Elias–Fano encoding is designed for such sequences.

For character \\(c\\), a standard Elias–Fano representation uses approximately

\\\[ n\_c\\left\\lceil\\log\_2\\frac{n}{n\_c}\\right\\rceil+2n\_c \\\]

bits before indexing overhead. Summed over all characters,

\\\[ \\boxed{B\_{\\mathrm{EF}}\\lesssim2n+\\sum\_c n\_c\\left\\lceil\\log\_2\\frac{n}{n\_c}\\right\\rceil.}\\tag{12} \\\]

For an approximately uniform byte distribution this is about \\(10n\\) bits, or approximately 1.125 MB for a 900,000-byte block, versus 3.6 MB for the T-vector.

Elias–Fano supports indexed access to elements without decoding all predecessors; theoretical constant-time access depends on suitable select indexing. It provides a formal alternative even when \\(r\\) is large. It does not exploit long runs as aggressively as a run-based representation, since its general bound depends on symbol frequencies.

### 4.3 A separate information bound from run count

Count length-\\(n\\) words over an alphabet of size \\(\\sigma\\) with exactly \\(r\\) runs. Choose positive run lengths in \\(\\binom{n-1}{r-1}\\) ways, the first character in \\(\\sigma\\) ways, and each subsequent character in \\(\\sigma-1\\) ways:

\\\[ \\boxed{N(n,r,\\sigma)=\\sigma(\\sigma-1)^{r-1}\\binom{n-1}{r-1}.}\\tag{13} \\\]

The bits required to enumerate this broader family are

\\\[ \\boxed{B\_{\\mathrm{run}}= \\left\\lceil\\log\_2\\sigma+(r-1)\\log\_2(\\sigma-1)+\\log\_2\\binom{n-1}{r-1}\\right\\rceil.}\\tag{14} \\\]

For \\(r\\ll n\\), this is \\(O(r\\log(n/r)+r\\log\\sigma)\\).

This counts all words with specified run count, not only valid BWT images or fixed-histogram words. It is an achievable representation-size benchmark for the larger family, not a tight bound for bzip2 blocks.

### 4.4 Two independent measures of compressibility

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

## 5\. Fourth consequence: move structures already solve the theoretical navigation problem

The interval-exchange permutation in Section 3 is precisely the sort of permutation handled by *move structures*, introduced by Nishimoto and Tabei. Their 2021 result establishes \\(O(1)\\)-time move queries with \\(O(r)\\) words of space after balancing represented intervals.

### 5.1 Carry the interval identifier

An ordinary query is \\(i\\mapsto\\Psi(i)\\). An interval representation must locate the interval containing \\(i\\), potentially requiring an expensive predecessor search. Instead retain

\\\[ \\boxed{(i,t)} \\\]

where \\(t\\) identifies the interval. The next row follows directly:

\\\[ j=p\_t+(i-a\_t). \\\]

We then need the interval \\(t'\\) containing \\(j\\). A move structure stores information about the destination interval of each image's beginning and advances through descriptors to locate \\(j\\). Unbalanced intervals may require multiple advances. The crucial theorem is that intervals can be split into \\(O(r)\\) balanced pieces, guaranteeing a constant number of advances:

\\\[ \\boxed{(i,t)\\xrightarrow{O(1)}(\\Psi(i),t')}\\tag{15} \\\]

using \\(O(r)\\) words in the word-RAM model.

Carrying \\(t\\) is essential. This is not one-memory-access arbitrary-row lookup.

### 5.2 Why it matters to mibz2

mibz2 already maintains per-lane state. A move structure adds a run/interval identifier to each lane, giving state

\\\[ (\\text{row},\\text{interval},\\text{segment state}). \\\]

A fresh segment start may require predecessor search once. For \\(P\\) segment starts initialised with binary search, seeding costs \\(O(P\\log r)\\); later transitions maintain interval state. The structure therefore removes general predecessor queries from the hot path.

It does not remove memory dependencies: a move query may load several descriptors. It does change the algorithmic problem from repeated searches to constant-time state transitions.

### 5.3 The 2026 result: linear-time inversion in run-bounded space

Brown and Langmead's *Bounding the Average Move Structure Query for Faster and Smaller RLBWT Permutations* (SEA 2026) studies *length capping*. (RLBWT is the run-length BWT: the last column \\(L\\) stored as its \\(r\\) runs, each a character and a length.) Set the average interval length to \\(n/r\\); for constant \\(c\\), split until every interval is of length at most

\\\[ \\boxed{L\_{\\max}=c\\frac nr.}\\tag{16} \\\]

The number of additional intervals is approximately at most \\(r/c\\), so \\(r'\\le r+r/c\\), up to rounding.

Their Theorem 2 bounds aggregate work over all \\(n\\) distinct positions, not strict worst-case cost per query. The number of interval advances is bounded by a constant multiple of \\(n\\):

\\\[ \\boxed{T\_{\\mathrm{all\\ moves}}=O(n).}\\tag{17} \\\]

For a single-cycle permutation, full traversal visits every position once. They also show \\(O(r)\\)-time/space construction of a capped structure for LF or its inverse from RLBWT. Their Theorem 9 establishes

\\\[ \\boxed{\\text{BWT inversion time}=O(n),\\qquad\\text{additional working space}=O(r).}\\tag{18} \\\]

This is a complexity theorem for inversion, not merely compressibility.

### 5.4 How directly does it apply to bzip2?

The theorem starts from run-length BWT. mibz2 reconstructs the byte sequence \\(L\\) after MTF (undoing bzip2's move-to-front coding, the stage before the inverse BWT), so discovering the runs involves examining \\(n\\) bytes. From the decoded sequence the construction-plus-inversion asymptotic cost is \\(O(n)\\) with \\(O(r)\\) navigation storage, when the input sequence and output buffer are accounted for separately. This does not make the entire decompressor \\(O(r)\\) total memory.

The single-cycle amortisation argument applies directly to primitive strings. Periodic cyclic BWTs may have several cycles; segmented traversal needs to preserve applicable assumptions. The balanced 2021 structure gives a per-move guarantee independent of single-cycle amortisation, at additional construction complexity.

### 5.5 Established result

There exists a mathematically established family of navigation structures whose storage scales with BWT runs rather than rows and that supports efficient repeated successor traversal without a fresh general predecessor search on each step.

The relevant works are [Nishimoto and Tabei (2021)](<https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICALP.2021.101>) and [Brown and Langmead (2026)](<https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/html/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.html>). Brown and Langmead's [Orbit library](<https://github.com/drnatebrown/orbit>) implements move-structure variants, but is not a verified drop-in replacement for mibz2's Rust T-vector.

## 6\. Fifth consequence: compact30's representability can be characterised exactly

### 6.1 The ordinary-record condition

compact30 groups thirty consecutive rows into 64-byte records and uses a label transform

\\\[ \\varphi(i)=32\\left\\lfloor\\frac i{30}\\right\\rfloor+(i\\bmod30) \\\]

or

\\\[ \\boxed{\\varphi(i)=i+2\\left\\lfloor\\frac i{30}\\right\\rfloor.}\\tag{19} \\\]

Two unused labels per group correspond to the record header. For a complete group \\(G=\[a,a+30)\\), ordinary-record conditions are a shared output character and labelled successor offsets fitting in 16 bits. Since \\(\\Psi\\) and \\(\\varphi\\) increase within a bucket, the endpoints suffice.

Ignoring special padding and sentinel cases:

\\\[ \\boxed{F\[a\]=F\[a+29\]} \\\]

and

\\\[ \\boxed{\\varphi(\\Psi(a+29))-\\varphi(\\Psi(a))\\le65{,}533.}\\tag{20} \\\]

This bounds *encoded successor-label span*, not necessarily raw row-index difference. It captures monotonicity used by the existing builder.

### 6.2 Gap-based interpretation

Suppose the group lies entirely in bucket \\(c\\), with first destination \\(x\\) and last destination \\(y\\). Between the thirty occurrences,

\\\[ y-x=29+\\\#\\{j:x\<j\<y,\\ L\[j\]\\neq c\\}. \\\]

Therefore

\\\[ \\boxed{\\Psi(a+29)-\\Psi(a)=29+h,}\\tag{21} \\\]

where \\(h\\) is the count of intervening non-\\(c\\) characters. The labelled span is

\\\[ \\begin{aligned} W={}&amp;29+h\\\\ &amp;+2\\left(\\left\\lfloor\\frac y{30}\\right\\rfloor-\\left\\lfloor\\frac x{30}\\right\\rfloor\\right). \\end{aligned}\\tag{22} \\\]

Apart from bucket crossings, compact30 RAW groups arise from too many intervening characters among thirty occurrences. It is occurrence-gap arrangement, not absolute destination position, that matters.

### 6.3 Guaranteed ordinary groups

If a complete 30-row group lies inside one translation interval,

\\\[ \\Psi(a+t)=p+t,\\qquad0\\le t\<30. \\\]

The raw span is exactly 29, and the labelled span at most 31. The group is guaranteed to fit an ordinary compact30 record. Its offsets form a precomputable pattern, precisely the bulk-emission case in the existing constructor, where the builder writes a whole record from a precomputed pattern because one run of \\(L\\) fills the group.

### 6.4 Exact formula for pure groups

Let translation intervals be \\(I\_t=\[a\_t,b\_t)\\). For group width \\(g\\), the number of complete aligned groups contained in interval \\(I\_t\\) is

\\\[ G\_t=\\max\\left(0,\\left\\lfloor\\frac{b\_t}{g}\\right\\rfloor-\\left\\lceil\\frac{a\_t}{g}\\right\\rceil\\right). \\\]

Therefore

\\\[ \\boxed{G\_{\\mathrm{pure}}=\\sum\_{t=1}^{r}G\_t.}\\tag{23} \\\]

For compact30,

\\\[ \\boxed{R\_{\\mathrm{pure}}=30G\_{\\mathrm{pure}}.}\\tag{24} \\\]

This is computable from BWT run boundaries and character ranks without building the entire navigation table.

Every complete group not inside one translation interval crosses a boundary. There are at most \\(r-1\\) such boundaries, each inside at most one aligned group:

\\\[ \\boxed{N\_{\\mathrm{nonpure}}\\le r-1.}\\tag{25} \\\]

The bound may be loose and non-pure groups can still be ordinary records. But when \\(r\\ll n/g\\), almost all complete groups are guaranteed pure.

### 6.5 Consequence for construction

BWT runs determine destination intervals. Record boundaries can be enumerated in row order, with endpoint spans found by sequential sweeps through run descriptors. Once runs are known, group classification can take

\\\[ \\boxed{O\\left(r+\\frac ng+\\sigma\\right)} \\\]

time without materialising every successor. A scan of \\(L\\) can then scatter final entries into classified records. This gives an \\(O(n+r+n/g)\\)-time construction strategy with appropriate run, record and cursor storage.

It **does not prove** lower instruction count or fewer writes than the fused builder. It proves that record classification need not be interleaved with every per-row scatter: structure permits classification at the interval and group level.

## 7\. Sixth consequence: multiple BWT steps admit a small interval representation

### 7.1 Powers of the navigation permutation

Conventional traversal visits \\(i,\\Psi(i),\\Psi^2(i),\\ldots\\). A multi-step transition computes \\(\\Psi^k(i)\\) while emitting the \\(k\\) intermediate characters:

\\\[ W\_k(i)=F\[i\]F\[\\Psi(i)\]\\cdots F\[\\Psi^{k-1}(i)\]. \\\]

The desired operation is

\\\[ \\boxed{i\\longmapsto\\left(W\_k(i),\\Psi^k(i)\\right).}\\tag{26} \\\]

For a general permutation this could need \\(n\\) stored entries; an interval-exchange permutation gives a better bound.

### 7.2 Bound on pieces of \\(\\Psi^k\\)

Start with a partition \\(P\\) into \\(r\\) translated intervals, each with constant output byte; there are \\(r-1\\) boundaries. Refine it so that two successive steps follow fixed source intervals. Splits occur at preimages of original boundaries. Since \\(\\Psi\\) is bijective and translates each existing piece, images of the pieces are disjoint intervals partitioning the destination domain. Each original boundary introduces at most one new split across the whole refined partition. Each further step adds at most \\(r-1\\) boundaries. Thus

\\\[ \\boxed{N\_k\\le\\min\\{n,\\ 1+k(r-1)\\}.}\\tag{27} \\\]

On every refined interval, \\(\\Psi^k\\) is a translation and \\(W\_k\\) is constant.

For two and four steps respectively:

\\\[ \\boxed{N\_2\\le2r-1,\\qquad N\_4\\le4r-3.} \\\]

This bound becomes uninformative when \\(k(r-1)\\) approaches \\(n\\).

### 7.3 Multi-character transition table

Each interval stores its starting row, displacement to \\(\\Psi^k(i)\\), and the emitted \\(k\\)-character word. With interval identifier, output retrieval and destination computation are straightforward. An explicit representation uses

\\\[ \\boxed{O\\left(kr(\\log n+k\\log\\sigma)\\right)}\\tag{28} \\\]

bits when \\(k(r-1)\\ll n\\). For fixed small \\(k\\), that is \\(O(kr\\log n)\\).

This proves a compact representation for the *function*, not constant-latency arbitrary interval lookup. Multi-character navigation need not inherently require an \\(n\\)-entry jump table.

### 7.4 Implication for mibz2

Kärkkäinen, Kempa and Puglisi's 2012 work studied reducing cache misses through multi-character BWT navigation, including two characters per lookup. mibz2 cites it as [prior art](<https://lmmx.github.io/mibz2/concepts/prior-work/index.md>). mibz2 has also built such a table: the pair walk of [PR \#34](<https://github.com/lmmx/mibz2/pull/34>), whose entries hold the row two steps on, was measured and dropped, because each step cost about twice as much and the table took about three times as long to build ([Why 32 couriers beat one](<https://lmmx.github.io/mibz2/concepts/couriers/index.md>)). The interval view suggests representing \\(k\\)-step transitions by run-sensitive translated intervals instead of a complete super-alphabet table.

The number of intervals increases at most linearly with \\(kr\\). For repetitive blocks it can be substantially below \\(n\\), though it may become large for run-rich blocks. Interval identification, segment boundaries and output ordering remain essential. A mathematical \\(k\\)-step operation does not imply one cache miss or CPU instruction.

## 8\. Seventh consequence: cycle structure distinguishes actual BWT images

### 8.1 Cycle count

Let \\(\\operatorname{cycles}(\\Psi\_L)\\) be the number of permutation cycles, and \\(m\_1,\\ldots,m\_r\\) the maximal run lengths in \\(L\\). A characterisation due to Likhomanov and Shur, reported in a survey of the bijective BWT, states

\\\[ \\boxed{L\\text{ is a valid cyclic BWT image}\\iff\\operatorname{cycles}(\\Psi\_L)=\\gcd(m\_1,\\ldots,m\_r).}\\tag{29} \\\]

For primitive input, the permutation is one cycle. Periodic inputs may have several. We must not assume every bzip2 traversal visits all distinct rows in one cycle.

### 8.2 Necklaces and exact enumeration

The BWT identifies cyclic rotations, so its images correspond naturally to *necklaces*. With counts \\(n\_1,\\ldots,n\_\\sigma\\) and \\(g=\\gcd(n\_1,\\ldots,n\_\\sigma)\\), Burnside's lemma gives

\\\[ \\boxed{N\_{\\mathrm{BWT}}=\\frac1n\\sum\_{d\\mid g}\\phi(d)\\frac{(n/d)!}{\\prod\_c(n\_c/d)!}.}\\tag{30} \\\]

The number of primitive necklaces is

\\\[ \\boxed{N\_{\\mathrm{primitive}}=\\frac1n\\sum\_{d\\mid g}\\mu(d)\\frac{(n/d)!}{\\prod\_c(n\_c/d)!}.}\\tag{31} \\\]

Here \\(\\phi\\) is Euler's totient and \\(\\mu\\) the Möbius function.

### 8.3 Coprime-frequency simplification

When \\(g=1\\), all source strings with that histogram are primitive:

\\\[ \\boxed{N\_{\\mathrm{BWT}}=\\frac1n\\frac{n!}{\\prod\_c n\_c!}.}\\tag{32} \\\]

The BWT restriction reduces the stable-sort family by a factor of exactly \\(n\\); the information saving is

\\\[ \\boxed{\\log\_2n\\text{ bits}.}\\tag{33} \\\]

At \\(n=900{,}000\\), this is approximately 19.78 bits for the entire block. Identifying the original rotation requires a primary index with roughly the same amount of information.

Cycle structure is useful for validity checks, periodicity, testing and traversal assumptions, but it is not a promising independent source of major storage savings. The stable-sort restriction accounts for most of the representational reduction.

## 9\. What does the original permutree thesis contribute?

Tamayo Jiménez studies weak-order lattices, permutree congruences, pattern-avoiding permutations, and related geometry. The inverse-BWT permutation belongs to a classical parabolic quotient of the symmetric group.

Let

\\\[ J=\\{s\_i:\\ i,i+1\\text{ lie in the same character bucket}\\}. \\\]

Then

\\\[ \\boxed{\\Psi\\in W^J}, \\\]

where \\(W^J\\) consists of permutations with no right descents at positions in \\(J\\).

### 9.1 Weak-order inversion statistic

The Coxeter length is

\\\[ \\ell(\\Psi)=\\\#\\{(i,j):i\<j,\\ \\Psi(i)\>\\Psi(j)\\}. \\\]

Stable sorting implies

\\\[ \\boxed{\\ell(\\Psi)=\\\#\\{(i,j):i\<j,\\ L\[i\]\>L\[j\]\\}.}\\tag{34} \\\]

This gives an exact algebraic rank, but it should not be confused with BWT run count.

### 9.2 Large inversion count can coexist with easy navigation

For \\(L=b^ma^m\\), \\(a\<b\\),

\\\[ \\Psi(i)=\\begin{cases}i+m,&amp;0\\le i\<m,\\\\i-m,&amp;m\\le i\<2m.\\end{cases} \\\]

There are just \\(r=2\\) runs but

\\\[ \\boxed{\\ell(\\Psi)=m^2}. \\\]

Thus quadratic Coxeter length coexists with two translated intervals. This is a valid cyclic BWT image of an alternating periodic word.

Weak-order geometry measures distance from identity under adjacent transpositions; it does not measure cost of successor queries.

### 9.3 Meets and joins

The parabolic-quotient family has weak-order meets and joins, but mibz2 evaluates one fixed permutation rather than optimising over permutations. Meets and joins need not preserve valid BWT images. The permutree thesis does not directly imply a faster decoder; its relevance is the algebraic setting, while useful data structures come from stable sorting, succinct sequences and run-bounded interval permutations.

## 10\. Stronger correctness guarantees and proof obligations

The mathematical properties support independent invariants:

| Invariant | Exact property | What it checks |
| --- | --- | --- |
| Permutation | \\(\\Psi\\) bijective on \\(\[0,n)\\) | No missing or duplicated destinations |
| Stable-sort identity | Equation (1) | Correct correspondence with \\(L\\) |
| Bucket monotonicity | \\(\\Psi(i+1)\>\\Psi(i)\\) within buckets | Counting-sort ordering |
| Run structure | Exactly \\(n-r\\) within-bucket adjacent pairs advance by one | Correct run correspondence |
| Interval translation | Equation (4) on every run | Correct interval descriptors |
| compact30 classification | Equation (20) | Ordinary/RAW representation decisions |
| BWT cycle validity | Equation (29) | Valid cyclic-BWT structure |
| Multi-step navigation | Equation (26) | Correct jump destinations and emitted words |

These diagnose internal errors more precisely than end-to-end decoded-output comparison.

### 10.1 Independent oracle

Given \\(L\\) and its histogram, construct the reference mapping through (1). For every row \\(i\\) verify

\\\[ \\Psi\_{\\mathrm{candidate}}(i)=\\operatorname{select}\_{F\[i\]}(L,i-C(F\[i\])). \\\]

This is a complete equivalence check, stronger than traversal from the primary index alone (which need not visit all distinct rows for periodic blocks). For \\(k\\)-step representations verify

\\\[ \\Psi\_{\\mathrm{candidate}}^{(k)}(i)=\\Psi^k(i) \\\]

and

\\\[ W\_{k,\\mathrm{candidate}}(i)=W\_k(i). \\\]

These are development oracles, not obligatory checks in production decoding.

## 11\. How the results constrain the design space

| Representation | Space for navigation | Access complexity | Main consideration |
| --- | --- | --- | --- |
| Conventional T-vector | \\(32n\\) bits | One direct lookup | High footprint |
| Packed destinations | About \\(20n\\) bits, plus symbols/metadata | Direct addressing | Bit extraction |
| Per-character Elias–Fano | About \\(nH\_0(L)+O(n)\\) bits | Indexed select | More dependencies |
| Simple run descriptors | \\(O(r\\log n)\\) bits | \\(O(\\log r)\\) predecessor | Repeated searches |
| Balanced move structure | \\(O(r)\\) words | \\(O(1)\\) with interval state | Interval transitions |
| Length-capped move structure | \\(O(r)\\) words | Amortised \\(O(1)\\) over distinct full-cycle visits | Coverage assumptions |
| \\(k\\)-step intervals | \\(O(kr(\\log n+k\\log\\sigma))\\) bits | Interval-dependent | Multi-character operation |

The key distinction is *random row access* versus *stateful successor navigation*. mibz2 predominantly needs the latter. Every lane follows a path and maintains state, making move structures relevant.

### 11.1 Constant-time transitions

The 2021 theorem establishes that compact representation does not imply logarithmic lookup time: \\(O(r)\\)-space stateful navigation can have \\(O(1)\\) worst-case move time. That removes a theoretical objection, not the hardware concern about several serial descriptor loads.

### 11.2 Amortised guarantees

The 2026 result bounds cumulative descriptor advances by \\(O(n)\\) for visits to distinct rows, rather than requiring each move to be worst-case constant. The mathematical argument can be set-based, not necessarily requiring one serial traversal order. mibz2's exact segmented and periodic-block behaviour still needs to meet the proof assumptions.

### 11.3 Construction

Given \\(L\\) and counts, its runs can be enumerated in \\(O(n)\\) time. Navigation intervals arise directly from runs; a full pointer array is unnecessary. The length-capped structure can then be built in \\(O(r)\\) time from RLBWT, giving an \\(O(n+r)=O(n)\\) path starting from decoded \\(L\\). Linear asymptotic construction does not mean lower constant cost than mibz2's fused counting-sort builder.

### 11.4 Memory versus critical-path latency

mibz2's compact30 work reduced footprint and benefited concurrent traversal, but direct construction remained expensive; the [4 November post](<https://lmmx.github.io/mibz2/blog/2026-11-04-compact30-constructor/index.md>) says bulk emission covered 74.3% of rows on the main Wikidata corpus. The [7 November pipelining work](<https://lmmx.github.io/mibz2/blog/2026-11-07-pipelined-teams/index.md>) overlaps a block's construction and traversal with other stages of neighbouring blocks: a team's leader runs the next block's MTF and finishes the previous block while its helpers build and walk the current one. Three resources remain separate:

\\\[ \\boxed{\\text{navigation storage},\\quad\\text{construction work},\\quad\\text{critical-path memory dependencies}.} \\\]

A representation may improve one and worsen another. The mathematical space/time bounds do not equate them.

## 12\. What this means for the direction of mibz2 research

### 12.1 Run-based navigation is a real alternative algorithm family

mibz2 need not choose only between the complete T-vector and packed per-row T-vectors. Move structures store interval transitions and support repeated navigation. When \\(r\\) is small they offer a theoretical reduction in live navigation storage, with established constructions and query guarantees.

### 12.2 Compact30 exceptions can be bounded and predicted

Character crossings are bounded by 255 groups; pure-run groups can be counted from (23); ordinary eligibility is expressed by (20). This enables interval/group-level classification and verification rather than opaque per-row heuristics.

### 12.3 Multi-step navigation has a run-sensitive bound

The pair \\((\\Psi^k,W\_k)\\) has at most \\(1+k(r-1)\\) translated, constant-output intervals. This establishes a compact *theoretical* representation of multi-character decoding transitions when \\(kr\\ll n\\), without guaranteeing CPU speedup.

### 12.4 Cycle structure is mainly for correctness

The restriction to valid BWT permutations adds characterisations and tests but removes little representation information beyond stable-sort structure in coprime-frequency cases. It should not be expected to supply major independent memory savings.

### 12.5 Weak-order geometry provides context, not the central optimisation

Parabolic quotients explain increasing buckets and inversion structure, but the objects the decoder actually evaluates are

\\\[ \\boxed{\\operatorname{select}\_c(L,k),\\quad\\text{translated intervals},\\quad\\text{move queries},\\quad\\Psi^k.} \\\]

These admit operational data structures and meaningful complexity bounds.

## 13\. Final assessment

| Gain | Established mathematical result | Value to mibz2 |
| --- | --- | --- |
| Stable-sort representation | \\(\\Psi\\) is a collection of character-select sequences | Succinct sequence structures |
| Run-bounded representation | \\(\\Psi\\) has \\(r\\) translated intervals | Space scaling with \\(r\\) |
| Move structures | \\(O(1)\\) stateful moves in \\(O(r)\\) words | Alternative to full T-vector |
| Linear-time inversion | \\(O(n)\\) time and \\(O(r)\\) additional RLBWT working space under assumptions | End-to-end theoretical feasibility |
| Fixed-record identities | Exact character crossings, spans and pure-group bounds | Formal compact30 analysis |
| Multi-step interval bound | At most \\(1+k(r-1)\\) pieces for \\(k\\)-step navigation | Compact multi-character transitions |

The permutree thesis identifies a legitimate algebraic neighbourhood, but the directly applicable algorithmic theorems come from succinct data structures and run-length BWT.

**What is proved:** the inverse-BWT navigation problem supports representations and algorithms with run-bounded space, efficient repeated traversal, and (in the RLBWT model) asymptotically optimal inversion time.

**What is not proved for mibz2:** that these beat its 32-lane, four-byte-per-row implementation in elapsed CPU cycles once table construction, segmented starts, emitted characters and shared-cache behaviour are counted.

The central research question changes from *how much can we compress the existing navigation table?* to:

**What is the least expensive stateful navigation representation for a BWT interval-exchange permutation, while preserving high memory-level parallelism across simultaneous traversals?**

The most consequential result to investigate further is the move structure, especially Brown and Langmead's 2026 length-capping construction. It is the closest existing theoretical answer to the problem mibz2 solves.

---

**Sources:**

- [Daniel Tamayo Jiménez, *Combinatorics of Permutreehedra and Geometry of s-Permutahedra*](<https://arxiv.org/abs/2310.19732>), PhD thesis (2023): weak order and parabolic quotients.
- [Takaaki Nishimoto and Yasuo Tabei, *Optimal-Time Queries on BWT-runs Compressed Indexes*](<https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICALP.2021.101>), ICALP 2021, LIPIcs 198, 101:1–101:15: constant-time move structures.
- [Nathaniel K. Brown and Ben Langmead, *Bounding the Average Move Structure Query for Faster and Smaller RLBWT Permutations*](<https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/html/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.html>), SEA 2026, LIPIcs 371, paper 9: length capping, and inversion in \\(O(n)\\) time and \\(O(r)\\) space.
- [Hideo Bannai, Dominik Köppl and Zsuzsanna Lipták, *A Survey of the Bijective Burrows–Wheeler Transform*](<https://drops.dagstuhl.de/storage/01oasics/oasics-vol131-manzinis-festschrift/html/OASIcs.Manzini.2/OASIcs.Manzini.2.html>) (2025), in the Festschrift for Giovanni Manzini, OASIcs 131: cyclic BWT and necklace enumeration.
- [Juha Kärkkäinen, Dominik Kempa and Simon J. Puglisi, *Slashing the Time for BWT Inversion*](<https://researchportal.helsinki.fi/en/publications/slashing-the-time-for-bwt-inversion/>), DCC 2012, pp. 99–108: multi-chain and multi-character inversion.
- [*Building a smaller bzip2 decoding table: from 49 instructions to 12*](<https://lmmx.github.io/mibz2/blog/2026-11-04-compact30-constructor/index.md>) (4 November 2026): compact30 architecture and builder evidence.
