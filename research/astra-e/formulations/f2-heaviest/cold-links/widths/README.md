# Reverse the ownership of unresolved returns

Stop the first backward trie as soon as its FM interval has width at most t. A cut [a,b) issues b−a placeholder occurrences. Stable child projection sends the jth placeholder to original cutoff row a+j. Even when all occurrences follow the same known suffix, their unexamined prefixes can spell distinct words and end at different source slots. The small witness in [small.json](small.json) has two different hidden words behind one cut of width two. Every complete first-return type has all its occurrences on the same side of the cut. Grouping each cut's distinct unknown tails by their proper backward prefixes reproduces exactly the eager rank work omitted by that cut.

The [missing-source-set theorem](../) survives arbitrary t. E is the complement of the known hot source ranks. Its cardinality equals the number of cold occurrences, which can exceed the numbers of cuts and hidden word types. B=D∪E, with D the heaviest discovered hot token's source bucket, stops every unknown backward transition. Each terminal placeholder carries one unresolved endpoint. The independent partial FM construction compared every macro with the explicit trace on each block and threshold. Real choices of t preserve the same heaviest D; an input with no hot word can retain just E.

## Resolve from the known source instead

Mark each **cutoff row** in original byte-rank coordinates. Starting at a cold source in E, its positive forward Ψ orbit reaches the cutoff of its own raw arc before any other cutoff. Each such row lies on a first-return segment ending at its associated section destination. These first-return segments partition the section-covered cycles. If the root itself is cut, the positive next section visit supplies the cutoff. Thus one knows where all cold walks begin and where they may stop before matching sources to destinations. The distance is exactly the deferred backward tail length. At the reached cutoff the rest of the word and section destination are already known.

For a realizable grouping, start one affine source interval per contiguous component of E. An original L run maps by LF to one contiguous interval in F order. Split L runs at cutoff-set boundaries before reversing these translations. Every resulting Ψ piece carries the byte, displacement and whether its landing row is a cutoff. At most two new original seams per cold group suffice. Locate the first descriptor of an active interval, scan intersecting descriptors, and continue from their translated subintervals until a cutoff. Source offset and width recover the endpoint pairing for every row in a terminal span. A sorted list of output spellings alone cannot pay for these operations. The ideal distinct proper-prefix count ignores disconnected Ψ images and is recorded separately. The reference constructs the table by sorting; a linear implementation can stable-bucket original runs by byte, then build its rank directory. Relative to the original LF representation this still incurs another constructed or reordered directional index. [Exact interval program](forward.py)

Forward ownership allows direct forward output even with cyclic clipping. Invert complete B macros using their known sources. Index each dangling macro's **continuation** by its cutoff row, with its saved first-level known suffix and following hot macro bytes in forward order. From an E source, emit Ψ bytes to its first cutoff, then emit that continuation and enter its now-known destination B row. From a hot B source, emit the complete forward macro. The marked primary emits raw Ψ bytes up to its first B visit, or up to n on a cycle without B. All output is clipped at n. This completely removes the one-lap reversal used by the initial backward diagnostic, including on marked periods that do not divide n. This separate writer uses the partial links and raw Ψ alone. It receives no table of completed cold links. [Writer](writer.py) · [Forty byte-exact real outputs](forward-marks.csv)

## Work and limits

Across nine admitted inputs, thresholds 1, 2, 4 and 8 produced 22,168, 25,559, 28,539 and 32,659 unresolved occurrences. At t=8 these belonged to 13,721 cut groups, of which 5,686 concealed multiple full words. The 46,552 first-level hot rank queries at t=8 replace 751,521 complete first-level queries, but individual cold tails then require 970,175 original queries. The marked forward writer can clip the initial or final cold arc; it used 970,072 of these, in addition to 251 initial raw queries over nine inputs. Unlimited independent processing of cold sources is a separate schedule from this serial writer.

| t | Hot first queries | Grouped Ψ tail lookups | Ψ intersections | Hot + Ψ lookup waves, eight lanes | B rows | Second FM queries |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 153,096 | 432,345 | 451,643 | 74,029 | 242,665 | 22,773 |
| 2 | 104,614 | 464,718 | 486,538 | 71,900 | 246,056 | 18,986 |
| 4 | 72,558 | 494,336 | 517,758 | 71,688 | 249,036 | 16,529 |
| 8 | 46,552 | 514,059 | 538,722 | 70,869 | 253,156 | 14,848 |

The former full first trie needed 94,236 eight-lane lookup waves. The alternative two-stage wave count publishes real hot children after parent completion, then releases independent Ψ roots and releases their children when the previous lookup completes. Counting one unit per intersected Ψ piece instead gives 67,703 eight-lane waves for the t=8 cold pass alone. Ordinary second induction retains 220,497 rows. At t=8, retaining each completed cold tail once per terminating affine span stores 739,206 raw bytes. The corresponding hot second-word dictionary uses 214,783 bytes, with a further 397,819 raw hot-continuation bytes in dangling macros. Translation tables, root projection, cutoff directories, temporary spellings, mark transfer and output still apply. The tests cover 24,060 exhaustive marked-column/threshold combinations plus all forty real outputs. [Full data](real.csv) · [Native-shape hot scheduling](hot-waves.csv) · [Pooled counts](summary.txt)

At t=8 the directional tail table has 563,238 cells over 559,121 original runs across the nine admissions. Only 4,117 additional seams are needed, but every required record and the rank index must still be constructed or reused. Hot first queries plus grouped Ψ lookups total 560,611, compared with 553,516 original-rank lookups in the earlier learned whole-run successor on these blocks. Including the respective second-stage lookups yields 575,459 versus 608,027; the operations differ in latency and all setup, ordering and byte costs remain. The promising structural change is the known-source forward formulation and direct streaming. More width tuning is unlikely to decide throughput. A bounded native test should price construction or reordering of the Ψ descriptor table, grouped cold tail resolution and byte staging together against the saved original discovery work, before considering a full worker. [Earlier learned baseline](../../../f2d/refined/summary.txt)

Reproduce from the worktree root. The input export and capped-run extraction are specified in the [one-width study](../) and [LF-power study](../../../f2d/refined/).

```sh
d=research/astra-e/formulations/f2-heaviest/cold-links/widths
export PYTHONDONTWRITEBYTECODE=1
python3 "$d/small.py" | diff - "$d/small.json"
python3 "$d/study.py" /tmp/f2-cold-input/export | diff - "$d/real.csv"
python3 "$d/waves.py" /tmp/f2-capped | diff - "$d/hot-waves.csv"
python3 "$d/writer.py" /tmp/f2-cold-input/export | diff - "$d/forward-marks.csv"
python3 "$d/summarize.py" | diff - "$d/summary.txt"
```
