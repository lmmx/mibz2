# Quotienting zero-prefix rows in the ordinary Compact quote root

At h4 the quotient removed 3,602,044 duplicate token rows across 228 admissions, kept the exact marked bytes and saved just 228 second-discovery rank queries against explicit sparse4. In the fixed 60-round panel all three depths missed the preregistered complete CPU criterion against ordinary named quote. The native source and exact small/ten gate are at `745abf7a5307b94a7acc29521384e7cb0da4dff6`, following the prospective plan at `a546a05adcf4d19bc85fcc7fdf8eb014b9a1d12f`. [Plan](PLAN.md) · [Frozen calls](phase.csv) · [Verified full admissions](admissions.csv)

## Column, marked rank and costs

Use the fast stock quote first root with its trailing-quote orientation. For a selected old word, the interned native byte spelling ends in comma followed by quote. The zero-prefix word Z consists of exactly those two bytes. Its z source rows already emit the full marker. Let b be the old Z source base, m the old quote frequency and s the selected mass. Replace Z's bucket with the s marker rows sorted by the old destination rank. A Z source embeds at its corresponding Z occurrence in this bucket, and keeps its old byte offset. The other selected words keep their old source order, an unchanged slice start, and a byte length reduced by two. Their additional marker rows fall in the gaps between successive Z occurrences.

The native publisher classifies the K original words, collecting inclusion bits in existing first-level scratch. Its first pass over the R old incoming records counts Z and non-Z selected mass and collects H records, one per selected non-Z incoming run. Each record stores the old prefix node, its run width and the number of preceding Z occurrences. Its gap in old coordinates is b plus that preceding-Z count. The second pass recolors selected IDs to Z and inserts these records at their gaps, splitting runs at insertion coordinates and coalescing equal neighboring labels. A per-word bit limits the update to each affected source base to one addition. The second pass needs no separate Z spool. The new ordinal space has m+s−z rows, the original dictionary ID count, and a single contiguous marker bucket using Z's old ID and bytes. When Z is absent the previous sparse insertion handles the block. When only Z is selected the original root and marked pair are the answer.

A Z marked source at index i in its bucket moves past the hazard widths at gaps up to and including i. For a mark inside a non-Z selected suffix, find its source occurrence number within that word. Walk the H records in order until it lands inside a record for that same ID. Its marker ordinal is b plus preceding Z and non-Z selected mass and the in-record offset. Subtract the trimmed prefix byte length from the skip. Other old rows after the Z bucket shift by s−z. A disjoint marked cycle uses the existing byte-level fallback. Construction and mark transfer take O(K+R+H) work, O(H) supplemental retained records of 12 bytes each, and the republished coalesced RLE in the existing alternate output vector. First discovery, emitted run writes, second-level scratch and final output are also included in the phase and memory ledgers. [Native publication and transfer](quotient.rs) · [Row-by-row independent reference](oracle.rs)

Recoloring selected non-Z runs can remove at most 2H run boundaries. Inserting their H records can add at most 2H boundaries, hence the final count is within 2H of R. The stock second-level root picks the largest mass, breaking ties by lower base. A marker at old base b can lose an equal-mass tie to an earlier ID. All 684 real quotient cases (228 admissions × three depths) selected Z. The small gate includes 600 mixed quotient/mark cases with a nonmarker tied winner. Each verified quotient reproduces the original bytes under the actual chosen second root. [Independent quotient theorem and real census](../README.md) · [All exact small cases](small.txt)

## Exact and resource checks

The native small gate tested all last columns containing a quote over a ternary alphabet through length six and a binary alphabet through length eight, with every primary at each depth. It compared all 24,906 ordered weighted dictionaries, maximal incoming runs, transferred marks/skips and complete output bytes. This included 4,971 marks on cycles outside the quote section. The ten actual full columns matched the dense oracle at all three depths. Each of the 1,368 arm/admission combinations then compared every marked output byte with independent dense LF. A separate process per arm checked frozen dense hashes while collecting peak RSS. The exported run tapes exercise inverse BWT before the outer RLE1 and stream CRC stages. [Ten whole objects](ten.csv) · [Full marked admission receipt](admissions.log) · [Streaming memory samples](rss.csv)

| Depth | Sum of old quote masses | Z rows eliminated | Selected non-Z runs H | New token rows | New first runs (old 84,317) | Second rank queries | Second dictionary bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 4 | 1,357,067 | 158,614 | 4,104 | 1,480,625 | 84,427 | 125,936 | 1,688,586 |
| 8 | 1,357,067 | 158,614 | 5,562 | 1,487,101 | 84,995 | 124,506 | 1,663,435 |
| 12 | 1,357,067 | 158,614 | 9,990 | 1,498,128 | 84,641 | 124,504 | 1,658,397 |

Across the ten tapes the previous explicit markers paid 125,946, 124,516 and 124,514 second queries at the same depths and created the same second dictionaries. The number of second rank queries equals the edge vector length here. Each first probe of a node adds one node edge and each additional homogeneous probe adds one ordinary edge. Across 228 at h4, sparse4 and quotient4 also stored the same 6,398,991 marker links and 36,230,837 dictionary bytes. At h4/h8 three actual primary marks belonged to Z. At h12 those same three and one additional selected non-Z suffix transferred. [Prior sparse depth controls](../../native/ten.csv) · [Resource report](summary.txt)

## Frozen native comparison

The six arms used a common existing Compact first root, stock second induction and final byte writer on original caps. The source-identical named controls use independently warmed scratch through the same compiled call site. Each ten-block round summed the complete thread CPU times by arm. Each arm occupied every order position on every input within each six-round cluster. The table uses the geometric means of paired round ratios and two-sided t intervals over the ten six-round mean log ratios. The Xeon host had four cores of CPU quota. All 3,600 paid calls ran on CPU3 at nice +12 and checked every output. The scope is single-thread inverse-stage CPU. [Matched phase calls](phase.csv) · [Temporal clusters](clusters.csv) · [Script](report.py)

| Arm | Complete / named A | Complete / named B | Complete / sparse4 | Peak KiB on 228 | Largest retained bytes with output |
|---|---:|---:|---:|---:|---:|
| Named A | 1 | | | 8,204 | 8,473,542 |
| Named B | .99500 [.92628, 1.06881] | 1 | | 8,208 | 8,473,542 |
| Sparse4 | 1.02154 [.96570, 1.08060] | 1.02667 [.98587, 1.06916] | 1 | 8,328 | 7,943,286 |
| Quotient4 | 1.08796 [1.05282, 1.12428] | 1.09343 [1.00324, 1.19173] | 1.06502 [.98703, 1.14918] | 8,140 | 7,896,294 |
| Quotient8 | 1.00804 [.91871, 1.10606] | 1.01311 [.97311, 1.05475] | .98679 [.93888, 1.03714] | 8,160 | 7,901,390 |
| Quotient12 | 1.05616 [.98319, 1.13454] | 1.06147 [1.02902, 1.09495] | 1.03389 [.98151, 1.08907] | 8,308 | 7,934,870 |

Quotient4 publication took 1.25475 [1.12023, 1.40543] of sparse4 publication CPU. The combined downstream second stage took .95157 [.89138, 1.01583], and the complete post-root portion including publication took .98121 [.92198, 1.04424]. The common first-root time varied between those arms by 1.11035 [1.00145, 1.23109]; the whole h4 difference cannot be attributed entirely to quotient code. The smaller root saved memory and induced nearly the same second-index work. A subsequent design must change the cost of building or walking that second object to use the contraction for throughput. The registered whole-worker threshold was a full point ratio at most .98 with upper bound below 1 versus both named controls. None qualified for whole-worker promotion. [Phase totals and all contrasts](summary.txt)

## Reproduction

Use the original caps and reference hash manifests recorded by SHA-256, with the Rust 1.99.1 release library. The archive and manifest construction commands are linked from the preceding native study. [Environment and input/binary SHA-256](environment.txt) · [Cap export and independent hash generation](../../../../native/README.md)

```bash
source /workspace/work/env.sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/native
out=/tmp/astra-quotient-replay
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
"$out/quotient" | diff - "$d/small.txt"
"$out/quotient" geometry /tmp/astra-clipped/ten.bin > "$out/ten.csv"
diff "$d/ten.csv" "$out/ten.csv"
nice -n 12 taskset -c 3 "$out/quotient" price /tmp/astra-clipped/ten.bin > "$out/phase.csv"
"$out/quotient" admissions /tmp/astra-token-all/admissions.bin > "$out/admissions.csv"
for a in 0 1 2 3 4 5; do
  "$out/quotient" rss /tmp/astra-token-all/admissions.bin "$a" /tmp/astra-clipped-final/expected.csv > "$out/rss-$a.csv"
done
head -1 "$out/rss-0.csv" > "$out/rss.csv"
for a in 0 1 2 3 4 5; do tail -n+2 "$out/rss-$a.csv" >> "$out/rss.csv"; done
python3 -B "$d/report.py" "$out"
```
