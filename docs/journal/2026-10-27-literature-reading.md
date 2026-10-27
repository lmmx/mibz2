# The literature phase: what the sources say, and the next campaign

The deliverable of the research brief in
[2026-10-27-literature-triage](2026-10-27-literature-triage.md), written after issues
#6–#10 were resolved experimentally. It covers the paper-by-paper findings, the
hypotheses they give, an overlap check against this campaign, the cheapest experiment
for each hypothesis, and the references.

## What this campaign tested (the overlap check's baseline)

Each result below is from the paired CI bench (10 rounds, 1 and 4 threads).

| PR | change | Wikidata, 1 / 4 threads | outcome |
|---|---|--:|---|
| #21 | the walk's next entry prefetched, 32 lanes, 512 segments | (released in 0.1.3) | kept |
| #24 | up to two Huffman symbols a lookup, four lookups a refill | −2.3% / −4.7% | kept, 0.1.3 |
| #23 | the T-vector built in four interleaved quarters (#7) | +12.1% / +10.9% | rejected |
| #25 | the MTF head in two registers (#8) | −1.6% / +1.0% on top of #24; random +11% | rejected |
| #27 | a plain walk for blocks with few runs (#9, step 1) | ±0.3%; pattern +21% / −19% | rejected |
| #29 | LF² as `LF[LF[i]]`: a gather pass and an 8-byte pair table (#6) | +9.3% / +20.7% | rejected |
| #30 | Precopy by a backward neighbour search, each byte's row recorded (#9) | +118% / +118% | rejected |

The sources show that #29 and #30 tested constructions their papers do not use. Sections
1 and 2 below give the published constructions. Neither carries the cost that sank our
versions.

## 1. Paper-by-paper findings

### Kärkkäinen, Kempa & Puglisi (2012), "Slashing the Time for BWT Inversion" (DCC 2012): read in full

- **Super-alphabet (Mtl-sa, §3, Figure 2).**
  - Figure 2 gives the naive `ComputeLF2: LF2[i] ← LF[LF[i]]`, a random gather.
  - The "Reducing space" paragraph of §3 says the implementation does not do that. *ComputeLF2 is the same as ComputeLF but with L replaced by LL and C replaced by CC, which stores the counts for pairs of characters*: one counting-sort pass over character pairs, with σ² counters.
  - Its stated preprocessing cost is O(σ_T + (n + σ²)/B) cache misses, sequential except for the σ² counters.
  - The pair array LL (2 bytes a row) is stored interleaved with LF2 (4 bytes a row), so the walk takes one miss per two characters.
  - #29 built the Figure 2 version: a gather pass (~10 c/B locally) and 8-byte entries.
- **Interleaved parts (§3, "Reducing cost of cache misses").** The text is cut into p parts walked in one thread, with p = 8. A footnote assumes the parts' end positions are stored in the compressed file, which bzip2 does not do. mibz2's pseudo-random flagged segment starts (#4, #5) cover the same ground without that metadata.
- **Precopy (§4, Figure 3).**
  - An *s-run* is a maximal range of rows sharing a suffix of length s. Lemma 8: inside an s-run, LF^k(j) = LF^k(i) + (j − i) for k ≤ s.
  - The s-runs are found from the LCS recurrence (Lemma 5, Corollary 6) in O(rs + n/B) misses.
  - At the first visit to an s-run, the walk records its output position. At a later visit it copies s characters from that position and jumps with LF^s(p) = powLF[lf + s] + (p − pf).
  - The implementation note: the run's information is *encoded within LF[b..e]*. LF values inside a run are consecutive, so most of them need not be stored. No extra array is needed.
- **Results.** A 2.3–4× speedup over Seward's mergedTL from the super-alphabet and the parts together. Precopy is fastest on highly repetitive data, with n/r of 50 or more.

### Kärkkäinen & Puglisi (2011), "Cache Friendly Burrows-Wheeler Inversion" (CCP 2011): read in full

- **The copy algorithm (§IV).**
  - Two access paths j and j + 1 stay one row apart while L[x] = L[x + 1] along them: a *chain*.
  - The walk detects a chain **forward**, by comparing L[j] and L[j + 1] at each step. That is the adjacent entry, so no extra miss.
  - At the chain's end it writes (k, i, ℓ) into T[j], T[j + 1] and T[k]. Those are entries the walk has already consumed, and nothing is allocated per byte.
  - It marks j + 1. A later arrival at j + 1 copies ℓ characters from S[i..] and jumps to k + 1.
- **Space and results.** Space is 6n bytes: the output is kept for copying, which mibz2's segment buffers already do. Results: 1.0–1.4× faster than mergedTL on Pizza&Chili texts, and 1.8× on a doubled English text.
- **The contrast with #30.** #30 searched **backward** for an already-walked neighbour, which forced a per-byte record of rows (4 B a byte, about 13 c/B locally). Forward detection needs no such record.

### libsais 2.12.1 (Grebnov), `src/libsais.c`, the `unbwt` routines: the code read

A production inverse BWT for the single-primary-index case. Its chain mode applies from `UNBWT_CHAIN_MIN_SIZE` = 64 kB, which is bzip2's case.

- **biPSI (`libsais_unbwt_calculate_biPSI`):**
  - P is ψ² and holds rows only, 4 bytes each, the same footprint as mibz2's T-vector.
  - It is built in one counting-sort pass: `P[bucket2[(T[LF(i)] << 8) + T[i]]++] = i`, with 65,536 bigram buckets after a bigram histogram pass (`compute_bigram_histogram`).
  - **No characters are stored.** The two characters a step decodes are the row's bigram bucket: `c = fastbits[p >> shift]; while (bucket2[c] <= p) c++;`. `fastbits` holds 2^17 u16 (256 kB), and `bucket2` 65,536 u32 (256 kB).
  - Both tables sit in L2 per core.
- **Runs in the build (`LIBSAIS_UNBWT_RUN_LANES` = 16, `store_rows`).** When 16 consecutive L symbols are equal, and the 16 rows they map to share their bigram, the 16 rows are written with one vector store and the counters advance by 16. This is run-aware LF in the build (KP 2011's Lemma 3).
- **Chains (`libsais_unbwt_decode_chains`, `calculate_biPSI_marks`):**
  - 24 chains (`UNBWT_CHAINS`) walk in one thread, each writing 2 bytes a step and prefetching `P[q]`.
  - Chain starts are hashed pseudo-random rows, one per interval of 2^shift rows (`libsais_unbwt_mark_row`, a 0x9E3779B9 hash).
  - The build marks the entries whose two-row step would pass over a start (`UNBWT_MARK`, the successor kept in the entry), so a chain stops exactly there.
  - This is mibz2's flagged segmented walk, with ψ² in place of ψ.
- **The convention gap.** libsais inverts a suffix BWT with `$` removed (its builds skip the primary index row, `t != 0`). bzip2's is the cyclic rotation BWT with origPtr. The structures carry over (no row to skip), but libsais cannot decode bzip2's blocks as it is.

### Baier (2018), "On Undetected Redundancy in the BWT" (CPM 2018): read §1–3

- **Tunnelling** shortens runs in the BWT to encode it smaller. It is a new format, so it does not apply to decoding existing .bz2 files.
- Its tunnels (blocks of parallel LF paths) are the same structure as KKP's s-runs and KP's chains, which Precopy exploits.
- Baier & Dede (2019, tunnel planning) choose which tunnels to cut for compression. That is also format-side, and was not read further.

### Gagie, Navarro & Prezza (SODA 2018 / JACM 2020), RLBWT locating, and Gagie (CPM 2026): abstracts and structure only

- These papers locate pattern occurrences in O(r) space, with toeholds and φ over sampled suffix-array positions.
- For inverting a plain BWT they add nothing past the run lemma (LF(i) = LF(i − 1) + 1 inside a run, KP 2011 Lemma 3), which libsais's build and both copy algorithms already use.
- No inversion speedup found. The briefing's entry "Reportable shortcuts with RLBWTs" (Gagie 2026) did not turn up under that title. The closest is Gagie's "Merging RLBWTs adaptively" (CPM 2026), a construction paper.

### Not read in full, for the reason given

| work | why not read in full |
|---|---|
| Petri (2024) | Succinct indexing: its compact layouts trade random-access speed for space. |
| Alanko (2020, thesis) | Prefix-sortable graphs. |
| Navarro (2020, 2021) | Surveys of repetitive indexes. |
| Kempa (2017, both) | Index construction. |
| Bannai et al. (2019) | The bijective BWT. |
| KKP (2016) | Lazy LZ parsing. |

Their abstracts and section structure show no inversion technique that the three sources above don't already cover. If a hypothesis below needs one of them, it says so.

### Practitioner decoders, outside the citation graph

- **indexed_bzip2** (Knespel): block-parallel decoding with prefetching and caching, sharing rapidgzip's architecture. Its README reports 6.1× over Python's bz2 at 12 threads, but gives no block-finder figures. Its block finder bears on #10, which stays deferred.
- **pbz2 0.2.0:** the speculative magic scan and the fused front end (#8 step 3, #10) were read in the bake-off (2026-10-27-bz2-bakeoff.md).

## 2. Hypotheses, ranked

Ranked by expected gain on the Wikidata blocks, with cost and likelihood. The local per-stage costs at 0.1.3, in c/B at one thread:

| stage | c/B |
|---|--:|
| walk | ~12 |
| build | ~6 |
| scanner | ~3 |
| MTF | ~2 |
| Huffman | ~1.5 |

### H1. ψ² as libsais builds it, in the flagged segmented walk (reopens #6)

- **The design:**
  - Build P = ψ² in one bigram counting sort (a bigram histogram pass, prefix sums over 65,536, then the scatter).
  - Recover each step's two bytes from a fastbits table and a `bucket2` scan.
  - Keep the 32-lane flagged walk with prefetch. Stop a segment at a start reached directly (START, as now), or passed over: the build flags the entry at LF(start), as libsais's marks do.
- **Expected:** half the dependent loads at the same 4 B per row. The walk could go from ~12 toward ~6–7 c/B. The build gains a histogram pass (~1–2 c/B), and its scatter spreads over the bigrams in use rather than 256 counters.
- **Risks:**
  - The bigram scatter's write streams: as many as a block has distinct bigrams, likely a few thousand on JSON.
  - The fastbits scan length on skewed bigram distributions.
- **Cost:** about 300 lines, one module. Likelihood: high, since it's production-proven.

### H2. Precopy as KP 2011 does it: forward chains, per-chain metadata (continues #9)

- **The design:**
  - While a lane walks row x, compare x's byte with x + 1's (the adjacent entry). With ψ there is also a bucket-boundary condition, F[x] = F[x + 1]: a 112 kB bitmap, or a flag bit in the entry.
  - Count the chain's length. At its end, write (k, out position, ℓ) into entries already consumed, and mark j + 1 (the VISITED encoding of #30 can carry it).
  - A lane arriving at a marked row copies ℓ bytes from the recorded segment position and jumps to k + 1.
  - Copies stop at segment starts, as walks do.
- **Expected:** #9's offline analysis found 65% of Wikidata's bytes copyable in runs of 32 B or more, with a mean of 200 B. If walk order cuts that toward the 44% #30 saw, it still replaces about half the walk's misses with sequential copies.
- **The cost per walked step:** one compare with the neighbouring entry. Per chain: 3 stores, then 2 loads at its reuse.
- **Interaction with H1:** chains along ψ² need both rows of each pair to stay parallel, which makes H2 harder. Do H2 on ψ first, then decide whether they combine.
- **Cost:** moderate. Likelihood: medium-high on Wikidata, nil on random and records (which must not regress: the check is one compare).

### H3. Runs in the build, as libsais does it (an alternative to rejected #23 for #7)

When 16 consecutive last-column bytes are equal, the 16 rows they map to are consecutive (the run lemma). Write them with one vector store, and the counter advances by 16.

- **Expected:** a cut to the build's ~6 c/B on run-heavy blocks only. Wikidata's mean run is short: 1–19 bytes per run, from #27's screening.
- **Cost:** small. Likelihood: low-medium for Wikidata, high for pattern, runs and onebyte.

### H4. The fused front end, pbz2's (#8 step 3, for memory)

Decode Huffman → RLE2 → MTF straight into the T-vector's (or P's) words. This removes the symbols buffer (1.8 MB a slot) and the last-column copy (0.9 MB a worker).

- **Expected:** peak RSS down by roughly 2.5 MB per slot and worker, about 15–20 MB of 4 threads' 71 MB. Speed roughly neutral.
- **Cost:** moderate. Likelihood: high for RSS.

### H5. The parallel scanner (#10)

This stays deferred: the scanner binds only from about 10 workers. The sources (pbz2, indexed_bzip2) add nothing that changes that.

## 3. Overlap check

| idea | the source's construction | ours so far | status |
|---|---|---|---|
| interleaved traversals | KKP §3 parts (stored ends) | #4, #5: pseudo-random flagged starts | done: our version needs no metadata |
| super-alphabet | KKP §3 pair counting sort; libsais biPSI + fastbits | #29: LF[LF[i]] gather, 8 B entries | **untested as published: H1** |
| Precopy / copy | KP §IV forward chains; KKP §4 s-runs in LF | #30: backward search, per-byte rows | **untested as published: H2** |
| run-aware build | libsais RUN_LANES store_rows | #23: four interleaved quarters | **untested: H3** |
| walk choice per block | (none: our idea) | #27 | rejected |
| tunnelling | Baier 2018 | | format change, n/a |
| RLBWT locate | Gagie–Navarro–Prezza | | index, n/a |

## 4. Cheap experiments, one per hypothesis

Each is the smallest measurement that could reject its hypothesis before any decoder code is written.

- **H1: two block statistics, then the build alone.**
  - First, the distinct bigrams per Wikidata block (the scatter's write streams) and the mean fastbits scan length. Both come from one pass over each block's last column.
  - Then the bigram build on mibz2's own blocks, timed against `build`.
  - The walk's gain is the paper's and libsais's design: it shows only in an implementation, and is not cheaply estimated apart from one.
  - A reference figure, `libsais_unbwt` on a suffix BWT of each block's decoded text, would show how fast a mature implementation walks a comparable BWT. It is not an A/B, because its transform, primary-index convention and preprocessing differ from bzip2's. It is optional, and it decides nothing.
- **H2: chain statistics.** In walk order (32 lanes, 512 segments), count the chains KP's forward rule finds and their mean length, and the share of bytes they cover. This is the same scratch harness as #9's analysis, which measured backward reuse in text order.
- **H3: run coverage.** The share of each block's last column in 16-aligned uniform windows whose target rows share a bigram. If it is under ~10% on Wikidata, H3 is for the synthetic corpora only.
- **H4: memory.** Peak RSS from the existing `rssest` harness with the symbols buffer and the last-column copy accounted separately, giving the upper bound on what fusion saves.

## 5. References

- J. Kärkkäinen, D. Kempa, S. J. Puglisi. Slashing the Time for BWT Inversion. DCC 2012, pp. 99–108. §3 "Reducing space" (the pair counting sort for LF2), §3 parts, §4 Precopy (Lemma 8, Figure 3, the implementation note on encoding in LF). https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf
- J. Kärkkäinen, S. J. Puglisi. Cache Friendly Burrows-Wheeler Inversion. CCP 2011. §IV (chains, the forward check of L[j] = L[j + 1], (k, i, ℓ) in T[j], T[j + 1], T[k]), Tables II–III. https://www.cs.helsinki.fi/u/tpkarkka/publications/ccp2011.pdf
- I. Grebnov. libsais 2.12.1, `src/libsais.c`: `libsais_unbwt_calculate_biPSI` (the bigram build and RUN_LANES), `libsais_unbwt_calculate_fastbits`, `libsais_unbwt_mark_row` and `calculate_biPSI_marks` (chain starts and marks), `libsais_unbwt_decode_chains` (24 chains, 2 bytes a step, prefetch). https://github.com/IlyaGrebnov/libsais
- U. Baier. On Undetected Redundancy in the Burrows-Wheeler Transform. CPM 2018, LIPIcs 105, 3:1–3:15. arXiv:1804.01937.
- T. Gagie, G. Navarro, N. Prezza. Optimal-Time Text Indexing in BWT-runs Bounded Space. SODA 2018; journal version JACM 2020 (arXiv:1809.02792).
- T. Gagie. Merging RLBWTs adaptively. CPM 2026, LIPIcs 369, 16 (arXiv:2511.16953).
- M. Knespel. indexed_bzip2. https://github.com/mxmlnkn/indexed_bzip2
