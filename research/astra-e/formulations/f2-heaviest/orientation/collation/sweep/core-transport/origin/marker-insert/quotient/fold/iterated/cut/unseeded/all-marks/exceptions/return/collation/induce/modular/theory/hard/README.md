# Two original runs can force a linear live modular grammar

The number of original L runs does not bound the raw modular contractor's useful join count independently of n. With just two runs, a primitive rotation can force Θ(n) strict contractions and Θ(n) live concatenations at fixed radix. A second primitive two-run rotation with the same radix has only Θ(log n) live joins. A cyclic-power example of length n=t² separates Θ(√n log n) raw contractions from Θ(log n) live joins. These are costs of the frozen right-tip representation. The hard spelling itself admits logarithmic compression.

Use zero-based mark zero throughout. For L=1^(n−b)0^b the stable LF is x↦x+b modulo n, a single cycle exactly when gcd(n,b)=1. In general there are gcd(n,b) equal lanes. This last column is the BWT of the corresponding single cyclic binary text or its power. The number of zero and one occurrences is fixed by the F buckets, and the marked output is independently obtained by reading F(x) before applying x↦x−b modulo n. All formulas below hold for integer radix q≥2; the native checks use q=2 and q=4.

## Hard primitive

Write n=qm+1, b=q, m≥2 and retain ranks congruent to zero. Let A denote the first L run and B the last. Use the subscript j for the original source residue. The reordered top group 0 is A₀ of width m followed by B₀ of width one. Each other top group j is Aⱼ of width m−1 followed by singleton Bⱼ. The corresponding bottom group is the singleton from B at source residue j+1 modulo q, followed by Aⱼ. LF keeps the source residue inside A and sends the last q source rows into ranks 0,…,q−1.

Remove groups j=q−1 down through 1. In each, the top singleton first faces Aⱼ of width m−1. The smaller-top splice removes it from immediately after Aⱼ and puts it in exactly the same position. This executes m−2 strict unit contractions. The next unit compares equal widths, removes the outgoing singleton, and installs its path in the still-present incoming rank of the preceding group. One further unit equal contraction removes the last rank of group j. The singleton coming from the higher eliminated groups may have a longer word by then, but still has width one. Its name differs from the outgoing singleton until the retained group is reached. Hence round one has

    strict = (q−1)(m−2), equal = 2(q−1), self = partial = 0.

There were only 2q progressions. The surviving normalized group contains a straight width-m piece and one wrap singleton, and its map is rotation by +1 on m+1 ranks. For m=q^s, a +1 rotation on qM+1 has top A of width qM and one wrap B. Each nonzero residue now removes exactly one width-M equal pair. This leaves another +1 rotation on M+1. The s such rounds remove s(q−1) equal pairs; the last size-two exchange takes one equal contraction. The complete count is

    total contractions = total joins = live joins = (q−1)(m+s)+1.

Every intermediate join from this primitive exchange is live. At each nonself contraction the joined word is attached to a positive-width arc. Changing that arc's word later uses its old word as a child in the new join. Removing it as a whole still attaches the join to a positive-width survivor. A self contraction would discard an interval mapping identically to itself, impossible while a single-cycle exchange still has an unremoved rank. Inducing a single cycle on a nonempty section preserves one cycle. Thus each recorded join has a descendant at the final marked rule.

At q=4, n=65,537 the first layer had eight pieces and 49,146 strict unit contractions. Its entire tower produced 49,174 reachable joins.

## Easy primitive with the same two runs

Take b=1 and n=q^d. On a +1 rotation of size qM with M≥2, the top residue groups before the last each contain A of width M. The final group contains A of width M−1 followed by the singleton B. On the bottom, group zero contains B then the final A of width M−1; every nonzero group contains one width-M A. At the right, first compare the wrap singleton with the preceding width-M A. A strict contraction moves this singleton in the top order and leaves width M−1 at the bottom tip. One equal contraction then removes the remaining M−1 rows of that group. The next nonzero group has the same configuration. After q−1 such pairs the normalized +1 exchange has size M and two bands. At size q the discarded rows require q−1 equal singleton contractions. Consequently

    strict = (q−1)(d−1), total contractions = total joins = live joins = (q−1)(2d−1).

For q=4 and n=65,536 this gives 45 live joins. The adjacent hard size 65,537 and this easy size differ by one row and share the same original run count, radix and mark; their rotations have different offsets.

## Periodic families distinguish construction from output

For n=qm and b=q each residue is invariant. Its exchange has A of width m−1 and B of width one over bottom B then A. In each deleted group, m−2 strict contractions and one equal reduce the isolated lane to one self loop, which is deleted. No join from that group can be reached by the marked lane. The retained exchange is +1 on m. For m=q^s the combined counts are

    total contractions = (q−1)(m+2s−1)
    all joins = (q−1)(m+2s−2), live joins = (q−1)(2s−1).

Now set t=q^s, n=t², b=t. This is one periodic binary text with t physical LF cycles of length t. For the first s modular rounds both mass and offset are divisible by q. Each removed residue has two bands of widths (t−1)u and u with u=b_current/q. Their difference decreases in units of u. It takes exactly t−2 strict contractions, one equal and one self to remove the whole residue, regardless of u. The retained exchange divides both widths by q, preserving their ratio. Once b_current=1, the final marked interval is the easy primitive rotation on t ranks. Hence

    total contractions = (q−1)(st+2s−1)
    strict = (q−1)(s(t−2)+s−1)
    all joins = (q−1)(s(t−1)+2s−1), live joins = (q−1)(2s−1).

For q=4 and t=1,024, the complete native tower contracted 15,387 times against 27 live joins. The long periodic output then repeats a 1,024-byte marked lap.

## What the witness leaves open

The repeated raw join is exactly a power. If the top tail a is smaller than bottom tail b and already immediately follows b in the top ordering, the strict splice leaves both tails in place. Write wa,wb for their widths. It is possible to batch

    h = min(floor((wb−1)/wa), floor(remaining/wa))

strict updates with a single width change and the word b^h·a on the short arc. The final possible equal or partial comparison remains separate. For a larger top tail and a bottom tail that already follows it in the bottom order, the corresponding spelling is b·a^h. Binary powering over a persistent grammar needs O(log(h+1)) joins; a native power word could store the repetition count directly. The proof follows by iterating the unchanged pair of names and their constant short width. This covers all the hard primitive family’s m−2 repetitions per discarded residue. It changes the unit of work, and no speedup on general Wikidata follows from the family.

The frozen native implementation is `modular.hpp` at `5d240588`. The executable below independently checks every emitted byte by direct inverse rotation, traverses the final grammar for reachability, and records the native layer counts. The formula checker compares 82 whole first layers, 58 full tower formulas and 1,513,510 decoded bytes. The remaining non-q-power m trials still check primitivity, first-layer counts and the marked output. [Native source](counts.cpp) · [Raw counts](counts.csv) · [Formula checker](formulas.py) · [Receipt](checks.json)

```sh
c++ -std=c++20 -O3 -Wall -Wextra counts.cpp -o /tmp/astra-mod-hard
/tmp/astra-mod-hard | diff - counts.csv
python3 -B formulas.py | diff - checks.json
```
