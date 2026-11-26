# Native reconstruction with a rank inside a single source bucket

A removed mark x occupies a source of type c=F[x]. Its predecessor type a=L[x] is outside the cut. Every source in the refined alias (a,c) survives in the same order as the a occurrences in the old incoming column restricted to bucket c. Its new base B′[id(a,c)] is known before publication. Add the number of these occurrences before x to obtain the new ordinal. The mark advances by the old spelling length of a inside its new word. A retained mark uses the previous deletion-prefix calculation unchanged. The ordinary move therefore needs no global predecessor rank. Only a possible fixed point with a=c can require the true global rank. [Derivation](PLAN.md) · [Every small cut](finite.json)

The bucket variant uses that identity and watches the ordinary prefix of emitted pieces until x′. The local variant also selects one old survivor slice in advance, and watches only that slice while emitting the new column. For x retained this slice was located during the graph overlay. For x removed, all of its predecessor's source bucket a survives and has new base B′[id(first[a])]. Therefore x′ minus this base is its old offset inside bucket a. One local slice scan locates it. [Selective emission proof](SELECTIVE.md)

| Work across ten original byte columns | Delayed global rank | Bucket rank | Bucket rank, single watched slice |
|---|---:|---:|---:|
| Ordinary cuts / moved marks | 250 / 90 | 250 / 90 | 250 / 90 |
| Original bootstrap histogram visits | 690,484 | 690,484 | 690,484 |
| Global rank prefix runs visited | 1,017,929 | 0 | 0 |
| Earlier slices visited in the destination bucket of a moved mark | 0 | 1,705 | 1,705 |
| Slices visited to locate a moved representative in its own bucket | 0 | 0 | 6,212 |
| Emitted pieces checked to capture the next mark | 1,065,610 | 1,065,610 | 250 |

Every real possible fixed mark occurred at the first raw run. The ten checks had no preceding raw runs to inspect. On an arbitrary column equal source and incoming types can still require work. The native exhaustive oracle includes such nonfixed rows and forced normalizations. A selected old slice can also cross an ancestry boundary, and its head may coalesce with the previous output run. Independently constructed dense examples include both within one selected slice. The real count of one watched piece per cut is specific to this panel. [Adversarial cases](witness.json) · [Native cases and binaries](check.json) · [Every input's work](counts.csv)

| Complete local marked byte inverse comparison | Thread CPU, 95% cluster interval | Wall time, 95% cluster interval |
|---|---:|---:|
| Bucket rank / delayed global rank | 0.9891 [0.9667, 1.0121] | 0.9859 [0.9580, 1.0146] |
| Bucket rank with single watched slice / bucket rank | 1.0475 [1.0218, 1.0738] | 1.0517 [1.0188, 1.0856] |
| Bucket rank / dense stable byte inverse | 0.9074 [0.8628, 0.9542] | 0.9009 [0.8494, 0.9556] |

Removing over a million global-rank prefix visits left a close comparison with the delayed route. Removing the observed mark check across almost all emitted pieces then increased the measured stage time in this implementation. The four matched arms charge all mark bookkeeping, the cut and grammar, rotation and output. The bucket route had the lowest point estimate among these four arms. Both bucket-local routes reuse the original per-bucket offsets and the supplied compact source-base vector. The extra local descriptors are fixed-size. Maximum fresh RSS was 10,272 KiB for bucket, 10,292 KiB each for delayed and single watched slice, and 8,096 KiB for dense. This local inverse comparison has a different endpoint and machine from a whole-worker comparison with mibz2 0.5.6. No CI or i9 result is attached. [1,600 validated samples](pairs.csv) · [40 fresh RSS processes](rss.csv) · [Per-tape paired results](summary.json)

**Reproduction.** The algorithm, native object equivalence checks and resource counts are frozen at b6f2ecf. The first commands build the pinned emitters and recover the ten native source tapes. Gates compare all complete old RLEs, all source-word IDs, the complete grammar DAG, the true ordinal and byte offset, and the materialized result at every step against the earlier checked algorithm. The ordinary and full audit builds separately check all 66,712 small cases per new arm and ten libbzip2-oracled full native output hashes per arm.

```bash
d=$(dirname "$(rg --files | rg '/cut/unseeded/native/published/bucket/build.py$')")
tmp=$(mktemp -d)
python3 -B "$d/../../build.py" "$tmp/base"
python3 -B "$d/../../check.py" "$tmp/base/run" research/astra-e/dense/target/wikidata corpora/wikidata.bz2 "$tmp/inputs" "$tmp/ordinary"
python3 -B "$d/../build.py" "$tmp/base" "$tmp/published"
python3 -B "$d/build.py" "$tmp/published" "$tmp/bucket"
python3 -B "$d/../../replay.py" "$tmp/base" "$tmp/published" "$tmp/bucket" | diff - "$d/../../binary-replay.json"
python3 -B "$d/finite.py" | diff - "$d/finite.json"
python3 -B "$d/witness.py" | diff - "$d/witness.json"
taskset -c 3 nice -n 15 python3 -B "$d/panel.py" gate "$tmp/bucket" "$tmp/inputs" "$tmp/gates" > "$tmp/check.json"
diff "$tmp/check.json" "$d/check.json"
diff "$tmp/gates/counts.csv" "$d/counts.csv"
taskset -c 3 nice -n 12 python3 -B "$d/panel.py" measure "$tmp/bucket/run" "$tmp/inputs" "$tmp/gates"
python3 -B "$d/panel.py" report "$tmp/gates"
python3 -B "$d/panel.py" report "$d" | diff - "$d/summary.json"
```

The next independent integration would carry the same local-rank identity into the lower-memory raw-RLE cut publisher. A matched factorial against the older seeded route should preserve arbitrary orientations, complete original inverse bytes, normalized multiple cycles, working capacity and fresh peaks. The rank theorem needs positive compact frequencies, stable source alias (a,c), and access to the owning old bucket. Publishing a second ancestry map solely for the mark would erase the intended memory advantage. Complete worker scoring and target-machine evaluation would still be needed for a production claim.
