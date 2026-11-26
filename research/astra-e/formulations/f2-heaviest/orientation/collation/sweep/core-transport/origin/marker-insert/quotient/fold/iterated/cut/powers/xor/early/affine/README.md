# Selective contraction reproduces the Astra 3 exchange

Astra 3 already defined this pair-word section, both compact physical rank orders and marked exclusion at `9cd25610131b66924eaa75813d0f649db79f354f`. The first new construction enumerates only the cross-color contacts and reconstructs other arcs as complement gaps. The second answers a compressed successor query directly from the original band and a shared hit index. It omits construction of both new tilings. [Selective constructor proof](PROOF.md) · [Direct routing proof](IMPLICIT.md)

Across all 338 historical hash-colored generations of ten real tapes, the selective constructor produced the identical canonical maps, image orders, marked ranks, dictionary word IDs and sizes as the original source. The frozen tape also matched every row of the historical published 34-round CSV. Its advancing comparisons fell from 404,091 to 263,090 and its temporary packets from 304,796 to 276,856. The output still contains 271,899 canonical bands across those rounds. The real audit independently expanded the protected original byte horizon at the first two and last generations of each tape. [Per-generation counters](panel.csv) · [Differential against pinned source](check.py)

| Ten-tape sum | Historical full-overlay constructor | Selective constructor |
|---|---:|---:|
| Comparisons advancing one or both interval lists | 5,325,647 | 3,476,202 |
| Temporary packets before canonical coalescing | 4,010,154 | 3,647,701 |
| Canonical output bands | 3,585,500 | 3,585,500 |

On the first generation, the implicit object answered all 6,598,608 row-to-row successor and word queries identically to the explicit exchange. It uses 224,293 cross records and 522,544 nonempty original image owners across ten tapes. Of these row queries, 3,238,537 need no secondary hit lookup, 1,519,226 have exactly one candidate hit, and 1,840,845 have a longer hit slice. The largest such slice has 876 records. Under a binary search, the cumulative upper bound is 7,975,096 secondary comparisons, plus the primary owner lookup per query. These refer to one lookup per physical residual row, before any extra marked-cycle repetitions or output. Building this implicit index still consumes old band data and both per-owner bases. [Physical row checks and slice geometry](implicit-real.csv)

The independent tiny reference comparison covered 38,694 weighted interval-exchange, color and mark cases. A separate direct-routing check covered 255,750 individual successor/word queries in 48,828 cases, including repeated source words on disconnected bands, empty hit sets, adjacent compact hit boundaries, multiple cycles and an arbitrary protected row. [Explicit audit](tiny.json) · [Implicit audit](implicit-tiny.json)

These results reduce work in the prior structural screen. The untouched canonical output volume in the explicit route and the primary rank lookup in the implicit route remain material costs. There is no native or target timing for these two constructors. No further complete canonical contraction has been inferred to beat a measured inverse.

Generate the original input.txt and cut.bin files using the earlier powers README. With `a` set to this directory, run the following from the repository root. The verifier reads the original oracle with git show and requires its immutable commit locally.

```bash
python3 -B "$a/check.py" tiny
python3 -B "$a/implicit_check.py" tiny
python3 -B "$a/check.py" "$s/checked" "$s"
python3 -B "$a/implicit_check.py" "$s/checked" > "$s/implicit-real.csv"
```
