# Project the entire RLE1 exception set from monochrome word cylinders

**Object.** Instead of decoding a transducer on every dictionary phrase or classifying every candidate output byte, reconstruct a sparse set of original BWT row coordinates whose bytes can affect a count. The remaining bytes can move directly to final output. This is a union of delimiter-programme word cylinders, evaluated in the original byte alphabet.

Let Ψ advance the stable marked row by one, emitting its F byte before advancing. For each byte b, the FM cylinder I(b⁴) consists exactly of the rows whose next four emissions are b. This holds for arbitrary stable-match permutations and repeated visits to a short cycle. Define H as their union. Define D as the union, over j from zero through four, of Ψʲ(H). The positions of every complete four-literal-plus-count unit are in D. Mark all five even when the count is zero, equals b, or belongs to several overlapping windows. A final incomplete group of four is contained as far as the input extends. A row in D need not be in an actual group. The conservative union permits false positives from count-byte interpretation, overlap, cyclic extension past finite block end and components not visited by the mark.

**Boundary proof.** Parse consecutive visits to D in their original order. Maximal spans outside D can be copied literally. A true count group cannot straddle such a span, since all its positions are in D. At the beginning of a D span, a parser state of four from earlier positions is impossible, as its preceding four equal rows and the current row would all be in the same D span. A smaller live count can agree with initial input literals, but reaching four before a symbol change would put at least one preceding outside-D literal into a true four-run. Thus starting D spans from an empty state gives exactly the same bytes until the first symbol change, and the same state thereafter. If the block ends first, both parsers agree on output and validity: attaining an unfinished four from a preceding outside-D literal has the same contradiction. Apply this at every finite split. A mark inside a D span starts with the ordinary empty parser state. Repetition until n bytes and a final truncation retain their specified semantics.

**Transport to an existing section.** Seal D by retaining D∪Ψ(D) in addition to the existing section and the primary. Every source in D is now retained and has a retained successor, so its compressed outgoing edge is a singleton literal. Transport a source x to its retained row by count/rank in the section. Encode the dangerous singleton's byte with its label. Every other return edge spells only ordinary literal output, since a path cannot cross an internal retained row. Splitting some previously thick runs costs work and may affect the label bit width. The CRC remains over actual final bytes, as in the baseline.

**Recognition cost.** The first native screen indexes maximal runs by character with source position, byte rank and run length. A pass forms the buckets and histogram. Three stable backward extensions per present symbol find each I(b⁴). At most six binary rank queries per character apply. Each occurrence then takes five Ψ selects. Sorting and deduplication orders the protected source and endpoint ranks for the constructor; the worst case is dense. A fallback threshold can be applied *before enumerating* a big H. The all-run diagnostic index reached 2.37 MB of live vector capacities on these ten successive blocks. This is a real cost. A smaller possible index uses the fact that the nested b-prefix intervals all lie inside the b bucket and that the first three Ψ advances from I(b⁴) remain in that bucket.

**Exact screen.** Every mark of all binary last columns of length 1–10 passed dense sorting, four-byte membership, independent stock RLE1 output/error agreement across D boundaries and n-byte truncation. On ten systematic Wikidata blocks every potential four-run actually triggered a count, and the sets did not overlap. Frozen has 34 such starts, 170 protected rows and 204 sealed rows including exits. Considering the original thin-run and primary classification, 75 new retained rows are needed. The other nine blocks add from zero to 452 retained rows. The ten observed exact counts should not be generalized to arbitrary compressed data. [Raw block counts](geometry.csv) · [Exhaustive witness](small.txt)

```sh
d=research/astra-e/formulations/f2-rle
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
rustc -O --edition=2021 "$d/probe.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/probe"
"$scratch/probe" small
"$scratch/probe" corpora/wikidata.bz2
```

The first complete worker hook retains the primary, the original thin rows and the sealed window rows. The output array tags only dangerous singleton residual labels, reserving enough extended label space in wide mode. It limits H to 1,024 occurrences before materialization and otherwise uses the stock gated path. The forced native writer matched 18,434 arbitrary marked columns, including 1,680 unfinished-run refusals. It used the sparse route on 227 of 228 large density admissions and independently matched 204,297,794 coded bytes through decoded output and CRC. The other 900,000-row admission exceeded the cap. Over 20 paired rounds on nine systematic admissions, sparse/stock opend thread CPU was 1.250 [1.185,1.319]. The median summed recognition time of 3.91 ms alone exceeded opend's 1.10 ms stock RLE1 time. End-to-end worker and memory measurements have a separate scope. [Census](native.csv) · [Raw observations](phases.csv) · [Combined phase accounting](summary.txt)

```sh
python3 "$d/prepare.py"
rustc -O -C codegen-units=1 --edition=2021 "$d/measure.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/native"
"$scratch/native" small
"$scratch/native" corpora/wikidata.bz2 "$scratch/phase.csv"
```

## Subsequent constructions and independent checks

| Reconstruction | What survived the next test |
| --- | --- |
| [Diagonal rank](diagonal/) and [ownership in the existing section](saturated/) | Sparse recognition survived; constructing or tracing every protected singleton was expensive. |
| [Rank from existing strips](compiled/) and [bundled interval owners](bundled/) | Evaluating every start from existing strip/queue geometry removed an extra cap scan. Bundling eliminated all 66 return-owner budget falls in the original corpus. |
| [Combined narrow endpoint and one-edge lookahead](combined/) | The independent five-arm CI run beat opend at workers 1, 2 and 4. Against the frozen narrow/ahead predecessor it was 1.0046, 1.0136 and 1.0142 in paired wall time; W2/W4 intervals excluded parity on the slower side. [CI report](../../ci/evidence/combined-word-actions-44436152031.md) |
| [Count-only substitutions](count-only/), [arbitrary marked cuts](marked-components/) and [streamed edit lists](slices/) | Positivity of every corrected component's mass certifies full marked coverage. The sparse-slice worker removed corrected spelling copies, finished directly on 227/228 admissions and passed the full gate. [Official gate](../../results/v1-rle-slices-gate/score/summary.txt) |
| [Cyclic BWT of fully corrected bytes](decoded-column/) and [simultaneous predecessor update](decoded-column/local-update/) | Updating all counts together reduced stream 134 from 23,887 sequential moves to 470. Every corrected rotation agreed with the independent suffix order. |
| [Sparse corrected-BWT worker](decoded-column/backbone/) | 224/228 admitted blocks wrote final bytes directly. All real outputs, stored CRCs, native adversaries and 338 official cases passed. Separate original navigation and corrected cap scattering alone cost 8.58 times the stock parser they replaced. The continuation is to test whether both full scans can be removed. |
