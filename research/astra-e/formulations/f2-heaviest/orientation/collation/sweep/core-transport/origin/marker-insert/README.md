# A distinguished suffix marker with no atlas

The original first-word BWT can supply the exact moved cut section to the ordinary second writer. Insert an extra marker only on an occurrence whose outgoing word ends in the selected suffix at the specified cut. Every other occurrence keeps one transition. The resulting weighted BWT needs m+s ordinal rows for m original words and s selected occurrences. Its compressed column is constructed in two scans of the existing first-word runs. It uses neither a forward prefix atlas nor a new byte-level core discovery. All sixty actual Wikidata calls to the existing Rust second writer returned the independent marked bytes. [Constructor, transport and native ABI](insert.py) · [Real runs and checks](wikidata.csv)

## The two pieces of the column

For a core C choose those original IDs W whose shortened core equals C. Write W=P·C for each selected ID. Put the s marker source ranks first, ordered by the original **destination** of these occurrences. Put all original U source ranks after them, retaining their original order. If a U source was selected, it emits P and moves to the marker ordinal corresponding to its old destination. That marker emits C and moves to the old destination in the U half. Unselected sources emit their entire original word and follow their old successor in U.

A source ID W either always enters the marker block or never enters it. Within that ID its old destinations increase, so its new destinations do as well. All marker successors increase by their chosen order. Thus each ID retains a contiguous stable source bucket, and the transformed object is an ordinary weighted BWT. Its first incoming subsequence is the old incoming RLE filtered to selected IDs, in order. Its remaining subsequence is the old incoming RLE with each selected ID replaced by the one marker. Place the old source bases at offset s, and the marker base at zero. An empty P is safe because its successor emits a positive C. Other marked cycles still contain some positive-weight transition on every lap. For a nonempty fixed section the old marked token and byte skip suffice. If the skip falls in a selected suffix, one ordered scan of old runs finds its filtered destination rank and subtracts the prefix length. Otherwise offset the marked source by s and retain the skip. A marked cycle disjoint from U uses the original fallback.

The marker frequency is s. When C is an actual mode of the shortened cores, s is at least every individual old ID's mass. The earliest-source tie rule of the unchanged second writer then chooses the marker. A fixed C need not be a mode on arbitrary input. The weighted object still decodes exactly for another chosen delimiter or a disjoint-cycle fallback. In the paper-style reference, source-leading words are used. For quote and a comma cut, native Compact stores W with its leading quote moved to the end. Its selected word ends in comma then quote. Truncating its existing dictionary slice by two and sharing a single comma-quote marker preserves the physical comma cut and the original byte stream. The representation's retained buffers, dictionary setup, marks and actual second work are priced in the native study. [Native protocol](native/PLAN.md)

## Exact RLE accounting and a sharp bound

Partition the selected old IDs into any number of nonempty classes. All words in a class can split off the same suffix, represented by one marker. Assign separate contiguous marker source blocks in any fixed order. The above stable proof works for each class. Emit one filtered original-ID stream per class, then the complete substituted stream. The proof requires only fixed class membership and spelling per source ID. An abstract marker can have zero length if the complete original words remain positive and their spellings are retained in the original ID.

For class j let aⱼ be its number of *old runs*. An island is a maximal consecutive subsequence of old runs in that class. Write kⱼ for the number of islands, and gⱼ for the number of consecutive pairs of islands whose facing old IDs are identical. Filtering coalesces exactly those pairs, giving aⱼ−gⱼ runs in that class's stream. Substituting the marker for every class member gives kⱼ runs in the full stream instead of aⱼ. The class streams use disjoint alphabets. None of their old IDs occurs in the substituted stream. Therefore the exact new count is **R + Σ(kⱼ−gⱼ)**. For each nonempty class its excess is one plus its number of unequal island boundaries. For one class the islands are precisely the contiguous components of the selected landing rows in U. One selected old ID adds exactly one run regardless of its frequency.

For one class, k≤min(a,R−a+1), so R+1 ≤ R_new ≤ R+ceil(R/2). Alternating selected IDs A and B with an unselected X between every selected pair attains the upper bound. Appending one final X gives the even-R construction. For multiple classes the general upper bound is 2R, attained if every old run has a different selected color and label. These are bounds on run representation; page tables may still grow with the number of ordinal rows. If every selected word has at least two original bytes, Σ f(W)|W| ≥ m+s, hence m+s is at most the number of original rows on cycles meeting U. The quote/comma selection has this property. [Class construction and independent edge verifier](classes.py) · [Finite witnesses](class-small.json)

The small one-class test covered 21,054 configurations and 116,558 marks. It verified 87,146 marks entering U, including 25,074 suffix marks; 29,412 marks missed U. Some configurations have an absent selected class and some correctly take the old fallback. The multi-class test verified all 214,155 transformed edges on 22,896 colored token objects. Every unchanged or split arc recovered its exact old successor and bytes. [Marked checks](small.json)

| Ten-block sum | Old R | New R | Selected occurrences | Island count | Equal bridges | Second actually used |
|---|---:|---:|---:|---:|---:|---:|
| Quote h4 | 84,317 | 84,947 | 282,172 | 3,479 | 2,849 | 10/10 |
| Quote h8 | 84,317 | 85,519 | 288,648 | 3,687 | 2,485 | 10/10 |
| Quote h12 | 84,317 | 85,204 | 299,675 | 3,322 | 2,435 | 10/10 |
| Frequent, each h4/h8/h12 | 66,373 | 66,383 | 31,358 | 17 | 7 | 10/10 |

Factoring every shortened core at quote h8 increased R from 84,317 to 124,906. It changed the sum of per-ID literal lengths from 1,077,638 to 1,051,197 bytes. Selecting only the comma gave 85,519 runs. These are logical sizes before vector capacity, the m+s page index, grouping/classification costs, and the second-stage dictionary. [All 120 one-class and all-class comparisons](class-wikidata.csv)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert
python3 "$d/insert.py" /tmp/astra-token-all/libtoken.so | diff - "$d/small.json"
python3 "$d/insert.py" /tmp/astra-token-all/libtoken.so research/astra-e/dense/target/wikidata | diff - "$d/wikidata.csv"
python3 "$d/classes.py" | diff - "$d/class-small.json"
python3 "$d/classes.py" research/astra-e/dense/target/wikidata | diff - "$d/class-wikidata.csv"
```

The shared native library uses the [checked second-writer build and its release dependency path](../../follower/README.md).
