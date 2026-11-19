# F2d. Which parts of delimiter construction must be paid?

The tabulation uses the F2b quote constructor at 92aeb21 and the F2c two-level decoder at 31565705f292b3b1dc3d8200eb65cd880e238bc7. The proposed object is the same ordered BWT of delimiter-ended phrases. A certificate for an entire L run can carry many unary FM refinements through one translation. The motivation is the measured cost of first-level discovery. [Two-level phase measurements](../f2c/phase-summary.csv)

## Bounds for the present constructor

Let r be the number of maximal L runs, n the number of rows, q the number of nonempty suffix intervals ending at their first quote, and b the number of explicitly retained trie nodes after unary contraction. Distinct suffix cylinders are disjoint by prefix-freeness, both within and across depths. Hence q ≤ n. Each interval lookup costs one 64-row rank/popcount query after constructing the run/page index in O(r + n/64 + capped input length). The overlay of q disjoint intervals with r runs has at most q+r−1 pieces: give each interval its first piece; each subsequent piece crosses a different run boundary. F2b pays q first-run lookups, q−b unary spelling bytes, the surviving selector processing, and the materialized dictionary spelling length D. Mark transfer costs at most one marked period. [Earlier invariant and implementation](../f2b/)

At a materialized node let R(v) denote its output run count, and merge consecutive selectors for the same child. A terminal emits this node's own word. Distinct nonterminal children own disjoint word IDs because their next byte differs at the same depth. Therefore consecutive, distinct selectors cannot coalesce their output labels. Within one child selector, its clipped child stream is already run-length encoded. Every fragment consumed by the current stable merge is exactly one output run at v. Thus its total number of fragments is exactly ΣR(v) over materialized nodes, including the root. Any constructor explicitly storing all those run streams has an Ω(ΣR(v)) entry-output cost. A constructor outputting only the root has just the Ω(R(root)) explicit-output bound from this argument. Frozen costs 18,994 pieces across nodes versus 6,133 at the root. All 228 real admissions cost 5,390,514 versus 1,786,472. These observations leave room for producing only the root. The full computation includes token-link construction, spelling and output after these costs. [Native counts](../f2b/geometry.csv)

There is also a worst-case input lower bound. Choose a delimiter d below the data alphabet and L = x₀…xₘ₋₁ dᵐ. Alternate disjoint pairs of data symbols by position parity, choosing freely within each pair. All m data entries are distinct maximal runs from their neighbors, so r=m+1. The section rows (L=d) are m+j. LF(m+j)=j by occurrence rank; LF(j) lies back in the section. The token column in section order therefore spells xⱼd in entry j, with m token runs. Independent swaps between positions in the same parity class give different ordered outputs at unchanged byte histogram. Even with that histogram supplied, a balanced class of Θ(m) binary choices has exponentially many possible orders and one queried data-run value gives one bit, requiring Ω(r) probes in this record-query model; explicit output requires Ω(r) run writes as well. The finite test verifies all 1,020 columns at m=2…9 and 3,868 count-preserving swaps. This establishes neither an Ω(q) enumeration bound on Wikidata nor a lower bound on a marked-only decoder that need not expose the entire token BWT. [Executable witness](chains.py) · [Receipt](lower-bound.txt)

## Whole-run jump invariant

On a nonquote maximal run i, LF(x)=x+δᵢ. If its entire image fits in run j, j is the next run for *every* subinterval of i. The initial discovery interval must be wholly inside i for a unary extension. In a chain with k certified transitions, the cumulative displacement is the sum of the translations and the exact reverse-trie edge spelling is the list of source run bytes in traversal order. A quote terminates the chain before its byte is consumed. When a full image meets multiple runs, its current source step still emits its byte; the destination of the actual subinterval then needs a lookup. A cached final run can be a quote. The existing terminal/branch test must still run there.

The structural program generates each real FM discovery path, then builds bounded jumps lazily using only run image queries available to an implementation. An independently materialized path checks each displacement, spelling, and stop. On frozen, of 66,871 unary visits, 55,219 admit whole-run successor certificates. Learning the successor on the first actual lookup and reusing it saves 19,543 later lookups without an extra setup query. An eight-byte descriptor skips 50,558 unary loop iterations, but building descriptors itself takes 48,096 run steps, including 38,748 lookup queries. Its remaining live discovery queries number 15,341, so total lookups fall from 70,560 to 54,089. On the nine density-admitted systematic inputs the same comparison is 751,521 versus 583,482, with 529,019 descriptor-build steps, 540,397 skipped loop iterations, and 116,771 distinct descriptors. Thus a drop in the live dependent path can nearly disappear in total instruction work. [Census, widths 2/4/8/16](jumps.csv)

## Reproduction

The ten stream ordinals are pinned in the extractor, including the out-of-density control at stream 41. Use the canonical Wikidata corpus and pinned repository release toolchain.

```sh
study=research/astra-e/formulations/f2d
tmp=$(mktemp -d)
cargo build --locked --release --manifest-path bench/moonshot/Cargo.toml --target-dir "$tmp/build"
rustc --edition=2021 -O "$study/extract.rs" --extern mibz2="$tmp/build/release/deps/libmibz2.rlib" -L "dependency=$tmp/build/release/deps" -o "$tmp/extract"
"$tmp/extract" corpora/wikidata.bz2 "$tmp" > "$tmp/inputs.csv"
python3 "$study/chains.py" --small | diff - "$study/lower-bound.txt"
python3 "$study/chains.py" "$tmp" | diff - "$study/jumps.csv"
```
