# Independent real J and H splice objects

The primary panel uses each of the ten systematic, 24-admission-spaced Wikidata tapes. For the frequent section it uses h=8 and the pinned 19-byte first-return spelling's suffix. For quote it uses h=12 and a comma core. All input, J fixture and dense H oracle SHA-256 values are recorded by case. The normalized source archive is 616 KiB and retains the original marked last columns as coalesced byte runs. The generated objects are reproducible from those columns. [Initial 20 receipts](primary.csv) · [Source archive](sources.xz) · [Data generator](generate.py)

**Independent construction.** Let Ψ be the original stable forward permutation, obtained by sorting physical original row numbers stably by last-column byte. On physical source row x, traverse Ψ once and append L[Ψ(x)] before testing the positive return. J is the complete physical first-hit interval Z(C) from the original run index. To find H independently, enumerate the original arcs starting in U, find their cut d=min(h,|word|−1), and keep each actual physical cut position whose suffix equals C. The generator checks these positions against the pinned run-index hole construction. It next walks every outgoing J arc and every outgoing H arc separately in the original byte permutation. Therefore the dense H successor, associated outgoing byte word and marked byte output are determined without contracting J edges. [Pinned hole construction](../../holes.py)

The J type IDs are in true physical source order. Identical source return words must occupy consecutive J slots. For source i with type t and dense partner k, place label t at incoming slot k. The generator checks that stable selection on the resulting token BWT recovers every original J partner, including ties within a type. Incoming is stored as maximal (type,width) pairs. Holes are ascending zero-based J source ranks. The frequent panel has total J mass 31,367 and nine holes, leaving 31,358 H sources. Quote has 299,698 and 23, leaving 299,675. Each policy covers 8,999,915 original coded output bytes.

Starting at the original primary, follow the inverse of Ψ until reaching the closest preceding source on each section. The reported mark is that source's ordered rank, and skip is the number of original bytes from its start. Marked decoding begins inside its outgoing word at skip, follows the section permutation cyclically, and truncates after exactly n bytes. When a marked cycle is disjoint from a section its mark is 0xffffffff and skip zero; in that case a decoder needs the original marked fallback. Independently following original Ψ from the primary produces the exact coded n-byte oracle. The generator checks both marked section traversals against it whenever their own marks exist.

## Binary records

Integers are unsigned little-endian u32. A case name is `OOOO-section-hD` with OOOO the four-digit tape ordinal. A `.jh` file has eight magic bytes `JHSPv1\0\0` and ten integers in this order.

| Header position | Value |
|---|---|
| 1–3 | Ordinal, policy (0 frequent, 1 quote), depth h |
| 4–7 | Original n, J mass, J mark, J byte skip |
| 8–10 | Source type count T, incoming run count R, hole count K |

The body holds T source words, each as (byte length, exact bytes), then R integer pairs (type ID, run width), then K integer J hole ranks. A `.oracle` starts with `JHORv1\0\0`, followed by four integers (H mass, H mark, H byte skip, n). It contains one record per ascending physical H row, each (successor rank, byte length, exact outgoing word). The final n bytes are the original marked output in byte order. The H map/words digest covers the concatenated per-row records alone. It is distinct from the full oracle and marked-output digests. The oracles intentionally contain enough bytes for exact equality checks.

```bash
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/splice/real
mkdir -p /tmp/splice-primary
python3 "$d/generate.py" "$d/sources.xz" /tmp/splice-primary > /tmp/splice-primary/cases.csv
```

The same generator accepts the original directory of `.irec.xz` files. The source pack records ordinal, n, original primary, the original archive digest and each true (byte,width) run. It excludes all induction arrays. To reproduce that pack from the initial archives, run `python3 "$d/sources.py" research/astra-e/dense/target/wikidata /tmp/sources.xz`.
