# A compound marker induces the second return directly

Can a second-level delimiter remove first-level construction? On all ten systematic blocks, the most frequent quote-ended token in F2c is the two bytes colon, quote. Its occurrence requires a preceding quote. The corresponding three-byte marker P is therefore quote, colon, quote. The experiment fixes this choice in advance and builds one return section of the original byte BWT. There is no first-level quote-token BWT, dictionary, alphabet histogram or token-link table on this route.

## Return to a word interval

Let Ψ take a row forward in its cyclic sequence of F bytes, with LF its inverse. The FM interval S = I(P) consists of the rows starting with a nonempty byte word P. Occurrence matching is stable, so Ψ to the power |P| strictly preserves order on S, including among equal infinite periodic continuations. It maps S bijectively to the rows immediately following an occurrence of P. These claims hold on each cycle of an arbitrary valid last column and on their union.

To label the rows of S by their preceding return word, seed the F2 trie at I(P) with an empty output spelling. At a node I(uP), all rows have u immediately preceding the destination marker, and none of the intermediate backward steps met S. Each nonempty left extension gives I(cuP). Its prefix fixes at least |P| bytes, so this whole child is either inside or disjoint from S. At the first hit, record the word cu, including the terminal byte c. Otherwise continue in that child. One FM bucket for a fixed terminal word occupies I(cuP) inside S. The bucket starts at its left endpoint minus the beginning of S. This supplies its token order, including when ordinary lexicographic collation of the spellings alone would leave an ambiguous prefix. All earlier F2 stable inverse merges apply.

Two distinct surviving nodes cannot have prefix-compatible cylinder words. If one word were a proper prefix of another, the longer path would contain the shorter word's occurrence of P before its destination. That is an intermediate hit of S, which terminates the path. Thus the real node intervals remain pairwise disjoint across all depths. For q nonterminal intervals and r maximal L runs, the run/interval overlay has at most q+r−1 pieces, irrespective of how wide P is. An interval in a single nonterminal L run can still be contracted on discovery. Explicit spelling, interval-location work, ordering, peak vectors and output are charged separately.

Every path from S returns in finite time because LF is a permutation. Rewind the actual primary until it first reaches S, or returns to itself. The elapsed number of bytes is the offset into the first outgoing return word. Expand exactly n output bytes, including a truncated last phrase on a short cycle; a marked cycle disjoint from S uses the existing fallback. The proof allows overlapping occurrences, including a return shorter than |P|.

## Composition of sections

Let d be a one-byte delimiter and w a first-level token ending in d with no other d. The section following every occurrence of token w equals the rows following the byte word P=dw. Indeed, the first d immediately precedes the token, and its next encountered d is w's terminator. This remains true when w=d and adjacent pairs overlap. Ψ to the power |P| conjugates the return permutation on I(P) to the return permutation on those after-occurrence rows and preserves ranks. Inducing first on d and then on token w gives the same after-occurrence permutation by transitivity of first returns. Thus the rank-successors agree exactly.

Output words shift with the choice of boundary. If X labels a direct return of length t and Y its after-occurrence return, X P = P Y, both being the same t+|P| consecutive bytes starting at the earlier occurrence. The equation remains valid for t<|P|. It gives a bijection between return labels and preserves the sequence of label changes in section rank order. Byte lengths and token-BWT run counts must agree between the two constructions for a common marker. Building either intermediate alphabet is unnecessary for exactness. Finding the most frequent w on a general block would still cost work. This candidate always uses the stated three-byte pattern.

## Reproduce

Use the repository toolchain, canonical Wikidata corpus, and the candidate source at this milestone. The first command forces the direct constructor through every binary pattern of length one to four, every binary column of lengths one to nine, and every primary against an independent dense stable sort.

```sh
study=research/astra-e/formulations/f2-word
export MOONSHOT_CANDIDATE="$PWD/$study/candidate.rs"
cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::arbitrary_marks_and_overlaps -- --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV=/tmp/f2-word-geometry.csv cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::corpus -- --ignored --nocapture
bash research/astra-e/score.sh compound-marker "$MOONSHOT_CANDIDATE" --with-opend
```

## Native results

The forced small test passed 98,080 covered marks and 147,740 exact refusals, including patterns longer than a marked cycle. The corpus route accepted 228/233 large blocks and agreed with Segmented over 205,197,794 inverse bytes. Direct and nested constructions agreed on both the number of token rows and total dictionary bytes on every admission. Nine systematic admissions also agreed on distinct word and token-run counts. On frozen both have 24,165 rows, 5,060 runs, 2,950 distinct words and 146,183 dictionary bytes. [Property test](property.log) · [All block observations](geometry.csv) · [Cost comparison](phase-summary.txt)

A 21-round three-arm rotation on those nine blocks checked every output against Segmented outside the timed region. Summing the per-block median thread times, direct construction reduced residual and output work from 5.68 to 2.01 ms. Discovery and spelling increased from 15.34 to 20.52 ms. The complete inverse took 28.92 ms direct, 26.61 ms nested and 19.93 ms for LazyCells. Frozen direct discovery took 100,644 interval lookups. Exact section composition removes a whole level, but a longer byte-level search erased that saving in this construction. This leaves the cost of locating branching return cylinders as the next mathematical question. [Raw phase times](phase-cpu.csv)

```sh
bash "$study/price.sh" /tmp/f2-word-price "$PWD/bench/moonshot/target/release/deps" > /tmp/f2-word-phases.csv
python3 "$study/summarize.py"
```
