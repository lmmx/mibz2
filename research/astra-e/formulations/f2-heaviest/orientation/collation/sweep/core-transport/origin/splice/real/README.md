# Independent real J and H splice objects

The primary panel uses each of the ten systematic, 24-admission-spaced Wikidata tapes. For the frequent section it uses h=8 and the pinned 19-byte first-return spelling's suffix. For quote it uses h=12 and a comma core. All input, J fixture and dense H oracle SHA-256 values are recorded by case. The normalized source archive is 616 KiB and retains the original marked last columns as coalesced byte runs. The generated objects are reproducible from those columns. [Initial 20 receipts](primary.csv) · [Source archive](sources.xz) · [Data generator](generate.py)

**Independent construction.** Let Ψ be the original stable forward permutation, obtained by sorting physical original row numbers stably by last-column byte. On physical source row x, traverse Ψ once and append L[Ψ(x)] before testing the positive return. J is the complete physical first-hit interval Z(C) from the original run index. To find H independently, enumerate the original arcs starting in U, find their cut d=min(h,|word|−1), and keep each actual physical cut position whose suffix equals C. The generator checks these positions against the pinned run-index hole construction. It next walks every outgoing J arc and every outgoing H arc separately in the original byte permutation. Therefore the dense H successor, associated outgoing byte word and marked byte output are determined without contracting J edges. [Pinned hole construction](../../holes.py)

The J type IDs are in true physical source order. For any specified return word, start with the destination interval J and take the byte-restricted LF image for each successive symbol in reverse spelling order. This sends an interval to an interval of no larger width by stable byte rank. Exclude J at each proper intermediate step. An interval whose width is at most |J| cannot straddle both ends of J, so this exclusion leaves at most one interval. Intersect with J on the final step. The surviving original source set for that word is thus contiguous. Monotonicity within its common spelling cylinder preserves its source order at the corresponding destinations, including tied rotations. For source i with type t and dense partner k, place label t at incoming slot k. The generator checks that stable selection on the resulting token BWT recovers every original J partner, including ties within a type. Incoming is stored as maximal (type,width) pairs. Holes are ascending zero-based J source ranks. The frequent panel has total J mass 31,367 and nine holes, leaving 31,358 H sources. Quote has 299,698 and 23, leaving 299,675. Each policy covers 8,999,915 original coded output bytes.

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

## Extended independent replay

The six fixed section/depth choices (frequent and quote, each at h4, h8 and h12) give 60 cases. The extended generation from the normalized source pack reproduced the original twenty files byte-for-byte. Across the sixty, Ψ has period n on every original mark, and every J and H row belongs to the corresponding marked return cycle. The binary verifier read 993,194 J ranks, independently contracted their 28,625 hole ranks and compared the successor and full outgoing word at every one of 964,569 H ranks with the dense original-domain oracle. J and H marked output matched all 53,999,490 original bytes; two J marks landed on holes. A three-row nonprimitive fixture places the initial byte inside a compound H word. Separate fixed points exercise the uncovered-mark fallback. [Sixty input and oracle digests](extended.csv) · [Independent binary replay](verify.py) · [Results](replay.txt)

```bash
mkdir -p /tmp/splice-extended
python3 "$d/generate.py" "$d/sources.xz" /tmp/splice-extended full > /tmp/splice-extended/cases.csv
cmp /tmp/splice-extended/cases.csv "$d/extended.csv"
python3 "$d/verify.py" /tmp/splice-extended "$d/extended.csv"
```

## True native source convention

The direct-native property and comma kernels use trailing-marker words. All J words of one case start with the same byte c, and every H compound does also. Replace each word cY by Yc. Concatenating rotated source words around a cycle shifts its full byte sequence forward by one. The source type order, destination labels, widths and all successors stay the same. To preserve the first output byte, subtract one from a positive natural skip. For a zero natural skip, move to the preceding source in that section and start at the last byte of its rotated word. A disjoint-source mark keeps its sentinel. This also explains why concatenation commutes with rotation for H words here. The fixture conversion rotates each dense-original H word once, independently of J contraction. [Converter](rotate.py) · [Trailing-format case digests](rotated.csv) · [Complete trailing-format dense replay](rotated-check.txt)

An independent export ran the original native fixed-literal first and second decoders at source pin `8ec1f2efb6e84b5a1626d92aa42039af79934e8a`. It requested the actual first-object snapshot from arm 4 for frequent h8 and arm 3 for quote h12. All twenty native snapshots agreed exactly with the converted J objects on each source base in order, all 52,927 type spellings (7,183,214 bytes), all 87,345 incoming run bases and widths, marked source and skip, and every one of the 17,999,830 final coded bytes. All twenty natural skips happened to be positive. A separate skip-zero test checks the required wrap to the previous source's last byte. Native export executes against the parent decoder library exposing the moonshot `byte_counts` accessor. [Raw export digests and counts](native-match.csv) · [Exact comparator](native_compare.py) · [Compiler input and native binary hashes](native-manifest.txt)

```bash
mkdir -p /tmp/splice-native-rotated
python3 "$d/rotate.py" /tmp/splice-primary /tmp/splice-native-rotated "$d/primary.csv" > /tmp/splice-native-rotated/cases.csv
cmp /tmp/splice-native-rotated/cases.csv "$d/rotated.csv"
python3 "$d/verify.py" /tmp/splice-native-rotated "$d/rotated.csv"
cargo build --release --lib --features moonshot
python3 "$d/native_export.py" /tmp/splice-native-export "$PWD/target/release/deps/libmibz2.rlib"
python3 "$d/native_compare.py" /tmp/splice-native-export /tmp/splice-native-rotated "$d/rotated.csv" > /tmp/splice-native-match.csv
cmp /tmp/splice-native-match.csv "$d/native-match.csv"
```
