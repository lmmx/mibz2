# Native alternating prefix and core publication

The alternating representation publishes the depth-eight first object in 0.349 [0.327, 0.372] times the cost of recovering original words. It hands the writer a run-encoded column over 2m ordinal rows, with independent prefix and core byte slices. On ten systematic tapes the second writer then visited 214,969 rank intervals versus 107,637 on the original token object. The complete paired inverse was 1.003 [0.973, 1.035] times the original depth-eight publication and 1.888 [1.806, 1.975] times the named Compact control. The registered whole-worker admission was missed. [Rule fixed before timing](REGISTER.md) · [Every paired observation](phase.csv)

## Construction, mark, and exact scope

The first m sources are physical cut ranks V, whose outgoing edges emit positive core C and reach old-source phase U. The next m sources are U in original rank order and emit prefix P on their way to V. Prefix weight zero is permitted. Every cycle alternates with a positive core, so output still advances. The destination column first receives the incoming prefix IDs in V order and then the existing core runs from incoming U. For each ID, its j-th destination occurrence still comes from its contiguous source base plus j. The existing packed eight-lane core constructor supplies the untyped core root directly, without forming original full words or follower pairs. [Mathematical construction](../README.md) · [Native publication](publish.rs)

Sort the atlas bands by original source. Each consecutive run of equal literal prefixes forms a prefix bucket. Each band retains its original physical position. With a forced core winner of mass f, slice each prefix bucket into the minimum number of consecutive pieces with mass at most f. All aliases of one bucket reuse its physical byte slice. Native construction records the first alias and consumed width for each physical band, then emits split runs while scanning the bands in V order. Equal-frequency ties select a core because V comes first. Unforced arms omit the winner histogram and split. Published core dictionaries and runs stay intact.

An original primary equal to both phase rows starts on V, ensuring a positive outgoing token. A primary in a positive prefix recovers its U source and byte offset from the original source or the atlas's hidden-mark receipt. Other intersecting marked cycles reach V by the retained LF walk. A disjoint marked cycle uses its original byte fallback. If the chosen second delimiter misses the marked cycle, the first-level fallback consumes the same alternating tokens. Empty prefix edges advance the ordinal even when they emit no bytes. [Mark implementation](mark.rs) · [Independent dense verifier and targeted counterexamples](oracle.rs)

Cap zero, four and eight with forced winner, and four and eight with the stock winner, passed all 228 admissions. All 1,140 real objects matched an independent stable Ψ oracle on every phase partner, byte spelling, first return, marked ordinal, skip and full output. All entered the second writer. On arbitrary binary and ternary columns the two policies passed 466,960 marked configurations, including 124,224 disjoint marks. The forty nonzero-depth systematic objects agree with the independent lift on band, core, prefix, alias and run counts, marked coordinates, winning mass, and the specified core winner's second vocabulary and compiled bytes. [Tiny cases](small.txt) · [All real objects](admissions.csv) · [Independent construction checks](crosscheck.py)

## Complete cost on the same input

Each measured arm receives the same original capped runs, byte counts, named frequent interval and original primary. The shared interval-selection cost is excluded. The panel rotates eight arms through every position on the same core, with separate warmed storage. Each of 32 paired totals sums ten tapes. The table gives mean per-thread CPU milliseconds per ten blocks. Reported ratio intervals use the paired log-ratios across the 32 totals. The complete inverse includes the verified byte output and the stock second-stage integer column, which uses pages for 2m rows and a residual link per winning occurrence.

| Arm | Setup and mark | First discovery | First merge | Publication | Second discovery and spelling | Second total | Complete |
|---|---:|---:|---:|---:|---:|---:|---:|
| Named Compact | 3.517 | 8.948 | 2.384 | 0 | 2.854 | 4.542 | 19.412 |
| Original publication h8 | 18.221 | 7.427 | 2.245 | 3.896 | 2.888 | 4.582 | 36.384 |
| Physical follower h8 | 19.150 | 7.308 | 2.197 | 2.768 | 4.418 | 6.186 | 37.625 |
| Forced core h0 | 4.171 | 9.166 | 2.310 | 0.217 | 4.513 | 6.427 | 22.304 |
| Forced core h4 | 10.461 | 8.447 | 2.257 | 0.784 | 4.913 | 6.715 | 28.674 |
| Forced core h8 | 18.513 | 7.533 | 2.284 | 1.361 | 4.970 | 6.806 | 36.509 |
| Stock winner h4 | 11.136 | 8.614 | 2.263 | 0.771 | 4.780 | 6.597 | 29.391 |
| Stock winner h8 | 18.445 | 7.351 | 2.242 | 1.344 | 4.914 | 6.657 | 36.049 |

Against the physical follower h8, forced-core h8 spends 0.495 [0.458, 0.536] as long in publication. Against original publication h8 the first object including setup/discovery/merge costs 0.934 [0.905, 0.964]. Compared with named Compact the complete h0 and h4 ratios are 1.152 [1.113, 1.192] and 1.484 [1.429, 1.541]. Giving h8 the measured named setup cost changes its complete ratio against named Compact to 1.111 [1.059, 1.165]. This substitution is a bound on the observed stage budget. [Full analysis and reciprocal controls](summary.txt)

For forced-core h8 on the ten blocks the writer compiles 5,525,748 second bytes and uses 31,358 residual links. The original token object compiles 5,525,525 second bytes and uses exactly the same number of links. Nonetheless each traversal now alternates across an extra token phase. Second discovery/spelling costs 1.734 [1.625, 1.850] times original h8, with a final links-and-emission ratio of 1.023 [0.930, 1.126]. Across all 228 admissions, forced h8 records 4,771,288 second rank lookups and 124,954,436 compiled bytes, compared with 2,360,866 and 125,949,883 for named Compact. The consistently added work is a rank halfstep and its token copy even when the final spelling bytes stay nearly unchanged. A direct paired-transition construction would have to price how it computes that transition as well as its relation to the original forward atlas.

Every memory arm ran in a fresh process on the complete 228-admission sequence, checking each output against an independent marked hash. Capped h0/h4/h8 peaked at 8,816/11,144/12,064 KiB, versus 8,544 KiB for named Compact, 13,060 KiB for old h8 publication and 12,556 KiB for physical h8. Stock h4/h8 peaked at 10,668/11,628 KiB. Capped h8's retained decoder capacity including output peaked at 9,145,358 bytes. Even the largest publication saving does not meet the single-worker local time and memory gates. These observations establish no CI or target-machine win. [Process high-water and retained capacities](rss.csv)

## Reproduction

The native publication and harness are at 1351ce6, independent marked correctness at 2f23666, paired stage/geometry/RSS receipts at 77f4e69, and the mathematical/ABI reference at 3a7b000. The generator checks the frozen packed constructor and second-stage hashes. Use the existing Rust 1.99.1 research library, [original-cap exporter and input hashes](../../native/README.md), and original dense mark hashes.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/two-section/native
out=$(mktemp -d)
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
python "$d/../../../../clipped/native/tapes.py" research/astra-e/dense/target/wikidata "$out/ten.bin"
"$out/check"
"$out/check" check "$out/ten.bin" > "$out/ten.csv"
python "$d/crosscheck.py" "$out/ten.csv"
"$out/check" check /tmp/astra-token-all/admissions.bin > "$out/admissions.csv"
nice -n 8 taskset -c 3 "$out/price" price "$out/ten.bin" 32 > "$out/phase.csv"
bash "$d/memory.sh" "$out" /tmp/astra-token-all/admissions.bin /tmp/astra-clipped-final/expected.csv 2 > "$out/rss.csv"
python "$d/report.py" "$out"
```
