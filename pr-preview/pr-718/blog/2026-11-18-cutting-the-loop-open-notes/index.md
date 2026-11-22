← [Cutting the loop open: decoding a block along shared paths](<https://lmmx.github.io/mibz2/pr-preview/pr-718/blog/2026-11-18-cutting-the-loop-open/index.md>)

# Technical notes: cutting the loop open

These notes hold the detail that [the main post](<https://lmmx.github.io/mibz2/pr-preview/pr-718/blog/2026-11-18-cutting-the-loop-open/index.md>) leaves out to keep its argument moving. They use its notation. \\(L\\) is the block's last column, of \\(n\\) rows, and \\(f\\) is LF. \\(S\\) is the set of kept rows, with \\(m = |S|\\) and the primary row \\(p\\) among them, and \\(E = f(S)\\) is the set of entrances. After the cut, the rows not kept form \\(H\\) wide strips, and \\(\\sigma\\) counts the distinct bytes in the block. Every statement here is from the project's [proof](<https://github.com/lmmx/mibz2/blob/f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71/docs/journal/2026-11-18-open-return/proof.md>), checked by the two verifications cited at the end.

## The bundle bound

Partition the row line into the \\(H\\) wide strips and the \\(s\\) intervals of consecutive kept rows. That makes \\(H + s\\) pieces, with \\(H + s - 1\\) internal boundaries. Let \\(e\\) be the number of intervals of consecutive entrances. The entrance intervals can also be cut wherever the first-column byte changes. Then every entrance in a bundle is reached from a kept row with the same letter, so a bundle has one kept letter as well as one word.

**Bound.** There are at most \\(e + H + s - 1\\) bundles. With the byte cuts, there are at most \\(3H + \\sigma\\).

**Proof.** Represent a gap between two neighbouring entrances by the row on its right. Inside one strip, a gap moves to the gap between the two rows' images, since the strip shifts both by the same amount. This map of gaps is injective, because LF is. Its image lies inside a strip's image, so it never produces a gap inside \\(E\\), and two distinct entrance gaps start disjoint gap paths. A gap between two different bundles must meet one of the \\(H + s - 1\\) internal boundaries before both sides reach the same interval of kept rows. Charge it to the first boundary it meets. Since the gap paths are disjoint, each boundary takes at most one charge. Add one bundle for the left end of each entrance interval. Finally, \\(E\\) is the complement of the strips' images, so it has at most \\(H + 1\\) intervals, and the byte cuts add at most \\(\\sigma - 1\\); so \\(e \\le H + \\sigma\\). Likewise \\(s \\le H + 1\\). Together, \\(e + H + s - 1 \\le 3H + \\sigma\\). ∎

**Tight cases.** Both verifications found columns with one wide strip that attain the bound. The column `110020000011` at threshold 2, with primary 9, has \\(H = 1\\) and \\(\\sigma = 3\\), and six bundles: \\(3 \\times 1 + 3\\). It attains the finer bound too, with \\(e = 4\\) and \\(s = 2\\), since \\(4 + 1 + 2 - 1 = 6\\). The column `abbbbdddaaaaaaaacccddddaabbbcccc` at threshold 4, with primary 21, has \\(H = 1\\) and \\(\\sigma = 4\\), and seven bundles: \\(3 \\times 1 + 4\\). One verification checked the bound on 19,717 run-structured cases, with no violation. On the frozen block, at \\(t = 16\\), there are 3,204 bundles against \\(3 \\times 3{,}448 + 89 = 10{,}433\\).

## One strip's heights

The bound limits the number of bundles, not their heights. Take a single wide strip of \\(w\\) rows that LF moves by \\(d \\ne 0\\). There are \\(\\min(w, |d|)\\) rows entering it. When \\(0 \< |d| \\le w\\), each entering path advances by \\(d\\) a step until it leaves the strip, so the paths' heights are \\(\\lfloor w/|d| \\rfloor\\) and \\(\\lceil w/|d| \\rceil\\). These are the quotients of a Euclidean division of \\(w\\) by \\(|d|\\).

The two extremes are a shift by one row and an image disjoint from the strip. A shift by one gives one path of height \\(w\\): a bundle of width 1, so the traced letters \\(U\\) equal the area \\(A\\). A disjoint image gives one bundle of width \\(w\\) and height 1, so \\(U = 1\\) for \\(A = w\\). Two twelve-row columns with the same run widths and byte counts, `220021021110` and `110010202221`, show both behaviours at threshold 2. In the first the strip's displacement is \\(-3\\), and three rows cross in parallel for one traced letter. In the second it is \\(+1\\), and one row takes three steps along the strip.

This is why neither the bundle count nor a small \\(H\\) bounds \\(U\\) by a multiple of \\(H\\). The same holds for the endpoint constructor's step count \\(K\\) below. It removes a common suffix one step at a time, without dividing, so a thick strip with a small displacement costs about \\(w/|d|\\) steps.

**More kept rows can mean more to trace.** On the frozen block, \\(U\\) falls at every threshold swept, but that is not a rule. The column `110000` with primary 2 has \\(f = \[4, 5, 0, 1, 2, 3\]\\) and two loops, 2 → 0 → 4 → 2 and 1 → 5 → 3 → 1. At \\(t = 1\\) only the primary is kept. Its one open path, 0 → 4 → 2, traces two letters, and the other loop is never cut. At \\(t = 2\\) rows 0, 1 and 2 are kept. That cuts the second loop open. The nonempty paths, 4 → 2 and 5 → 3 → 1, are one and two steps long and lie in separate bundles, so \\(U\\) rises from 2 to 3.

## The two constructors of the open paths

**The tracer.** Start at the leftmost entrance not yet covered, and follow its path through the wide strips. At each strip, and at the final interval of kept rows, note the distance from the path's row to that piece's right-hand edge. Count the starting entrance interval's edge as well. The smallest of these distances is the width of the bundle: every row in it follows the same strips to consecutive exits. Advance by that width and repeat. The tracer needs only the wide strips and the intervals between them.

**The endpoint constructor.** Build an auxiliary interval exchange on \\(m + n\\) points: a prefix of \\(m\\) terminal points, followed by a shifted copy of the \\(n\\) rows. Its arrows are:

1. terminal \\(k \\to m + e\_k\\), with the empty word;
2. \\(m + x \\to m + f(x)\\) on the \\(H\\) wide strips, with their bytes;
3. \\(m + s\_i \\to i\\), with the empty word.

The terminals of each kind join up into intervals, so at most \\(3H + 2\\) intervals start the exchange. Its first return to the prefix is exactly \\(P\\), with the words \\(w\_k\\), because the two empty arrows delimit each path. Now remove rows from the right-hand end until only the prefix is left. If the last domain strip is \\(X\\) and the last image strip is \\(Y\\), remove the common suffix of their widths. The shorter strip's record keeps the joined word, the reverse word of \\(X\\) then that of \\(Y\\). The prefix boundary can never be crossed, and the number of live strips never increases. On the frozen block, at \\(t = 16\\), the exchange starts from 7,786 intervals and takes 11,215 steps, making 7,334 rules that each join two words. The word dictionary is then expanded by copying the rules already expanded.

**Agreement.** The two constructors gave identical bundle and word-length counts at \\(t = 16\\) on all 431 Wikidata blocks and the six prose blocks.

**The queued decoder.** As the run records arrive, each narrow row appends its kept index to a queue for its byte. Consecutive entries of a queue are then wired to consecutive exits of a bundle. That is the stable sort of the main post's section 2, done during the pass that reads the runs. A packed table, of four bytes per kept row when the row and the word's code fit in one word and eight otherwise, holds each kept row's inverse return and its word. On the frozen block it is 359,496 bytes. Output follows the marked loop, copying each word's bytes forwards.

## The narrow–wide switch cut

Colour each arrow narrow or wide, by the run it leaves. A *switch row* is a row whose incoming and outgoing arrows have different colours. Narrow-to-wide switches are the rows of \\(E \\setminus S\\), and wide-to-narrow switches the rows of \\(S \\setminus E\\). Since \\(|E| = |S|\\), the two counts are equal:

\\\[h = |E \\setminus S| = |S \\setminus E| = m - |E \\cap S|.\\\]

The count can be taken before any word is known, by intersecting two tilings of the row line into runs. On the frozen block at \\(t = 16\\), \\(h\\) = 15,278 in both directions, as one verification recomputed. Keeping only the \\(2h = 30{,}556\\) switch rows, plus the primary if needed, gives a smaller kept system than the 89,874 narrow rows. But its construction traverses those 30,556 arrows one at a time, and traces 58,581 more letters to find the narrow side's words. In the repeated inverse-only measurement, its emitter took 0.511 ms against the queued decoder's 1.018 ms, and its constructor 6.771 ms against 1.857 ms. One verification corrected a detail of the convention. The program reclassifies a wide primary row as narrow before counting, so on `111000` at threshold 2 with primary 0 it finds switch rows \\(\\{0, 3\\}\\), not \\(\\{0\\}\\).

## What the two verifications checked

Both verifications rebuilt the construction from the proof, not from the project's code.

| Claim | First verification | Second verification |
| :--- | :--- | :--- |
| Restriction lemma | proved | proved |
| Factorisation, spelling, independence | proved; 6,000 random cases against a plain LF walk | proved; 30,557 small column and kept-set pairs, at every kept mark |
| Stochastic complement | proved, with the closed loops in \\(C\\) deleted | proved, with the closed loops in \\(C\\) deleted |
| Bundle bound | proved; tight; 0 violations in 19,717 cases | proved; tight |
| Endpoint constructor | proved; step count not recomputed | proved |
| Switch identity | proved; 15,278 both ways on the frozen block | proved; marking convention corrected |
| Frozen-block counts | recomputed exactly at \\(t\\) = 1, 16, 256 | checked as recorded at \\(t = 16\\) |
| Medians over 233 blocks | taken from the records | re-derived: 88.6689% and 5.4943% |
| Timings | not re-run | paired ratios recomputed from the raw rounds |

The project's own checks, separately, compare each constructor with a plain decoder on 198,790 column, primary and threshold combinations. They also check the destination and word of every kept row's arrow on 26,135 more, and compare complete streams with libbz2.

---

**Sources:** [the proof](<https://github.com/lmmx/mibz2/blob/f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71/docs/journal/2026-11-18-open-return/proof.md>), [the research journal](<https://github.com/lmmx/mibz2/blob/f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71/docs/journal/2026-11-18-astra-investigation-2.md>), [the checks](<https://github.com/lmmx/mibz2/blob/f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71/docs/journal/2026-11-18-open-return/evidence/checks.txt>), the first verification's [record](<https://github.com/lmmx/mibz2/blob/7747fddd11684e7959cb75f8de19a5b26138aef1/docs/journal/monodromy/2026-11-18-open-return-verification.md>), and the second verification's [review](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6978935386>) and [report](<https://github.com/lmmx/mibz2/tree/432a6ce03857b21e57b03ff8d97094d0f96eff97/state/reports/open-return-6978698021>).
