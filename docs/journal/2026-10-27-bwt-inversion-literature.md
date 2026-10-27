# 2026-10-27: BWT inversion, the literature after "Slashing the Time" (research branch)

**This is literature collection only.** There is no implementation, benchmark or agent work here.
- **Its aims:** an inventory of the works that cite, or follow from, Kärkkäinen, Kempa & Puglisi (2012); the citation relationships between them; and a reading queue for mining them for directions.
- **What it is for:** the fast bz2 decoder (`2026-10-27-compute-frontier.md`, #194), and the terminology a standalone decoder library should use.
- **The question that steers the reading:** does later research improve inverse-BWT traversal for existing bzip2 streams? That covers LF-mapping traversal, memory locality, independent chains and memory-level parallelism, pointer chasing and prefetching, cache-aware layouts, parallel inversion, alphabet expansion (the super-alphabet), and exploiting repetition (Precopy).

The data is in `2026-10-27-bwt-inversion-literature/`, built to be extended:
- **`works.csv`:** one row per distinct work, with a stable key, its title, authors and year as shown, the venue as shown, its first capture, Scholar's "Cited by" count, a status, and notes.
- **`edges.csv`:** one row per citation relationship (citing → cited), with the capture it came from and how far it is verified.

A work cited from several lists has **one** row in `works.csv` and one edge per list.

## Ground rules

- **A Scholar listing is a lead, not a verified citation.** Every edge from a Scholar "Cited by" page is `scholar-listing` until the citing work's bibliography has been checked, then `verified`, or `false-positive`.
- **Nothing is pruned for relevance.** Tangential works, surveys, theses, talks and possible duplicate versions are all kept. Only the obvious false positive is marked as such, and it too stays in the data as an audit trail.
- **ChatGPT's earlier interpretations are recorded separately** (§ "Unverified claims"), not mixed into the inventory. Some were speculative and some ChatGPT itself withdrew.
- **Titles, authors, years and venues are as Scholar showed them,** truncations ("…") included. Nothing has been filled in from memory.

## Captures

| id | what | how captured | completeness |
|---|---|---|---|
| C0 | Scholar "Cited by" of the root `kkp2012-slashing` | the user's Scholar lookup, as ChatGPT transcribed it (the raw paste is not in this record) | 16 entries: 10 on page 1, 6 on page 2. **Re-capture the raw page** for exact venues and "Cited by" counts. |
| C1 | "Cited by" of `gnp2017-fast-locating-rlbwt` | the user's manual Scholar paste, 2026-10-27 | 1 result, complete |
| C2 | "Cited by" of `baier2018-undetected-redundancy` | the user's manual Scholar pastes, 2026-10-27, page 1 ("About 19 results") and page 2 ("Page 2 of 19 results") | 10 + 9 = **19, complete** (page 2 alone was pasted first) |
| C3 | "Cited by" of `baier-dede2019-tunnel-planning` | the user's manual Scholar paste, 2026-10-27 | 6 results, complete |
| C4 | "Cited by" of `kmk2012-grammar-precompression` | the user's manual Scholar paste, 2026-10-27 | 5 results, complete |

## The root

**`kkp2012-slashing`:** J. Kärkkäinen, D. Kempa, S. J. Puglisi. *Slashing the Time for BWT Inversion*. Data Compression Conference (DCC) 2012, pp. 99–108. DOI 10.1109/DCC.2012.18, as given; to verify.

**Abstract** (as supplied by the user):
> Inverting the Burrows-Wheeler transform (BWT) is a bottleneck in BWT-based decompressors. The state-of-the-art inversion algorithm runs in linear time but is slow in practice due to CPU-cache misses. For more than a decade these cache misses have been thought to be inherent to BWT inversion. We show how to reduce the number of cache misses by a factor of nearly two, and simultaneously the cost of cache misses by another factor of two, obtaining a consistent speed up by a factor of 2.3-4. We can do even better if the data is highly repetitive. We describe an algorithm that achieves an asymptotic reduction in cache misses in theory and is the fastest algorithm in practice for such data.

**Techniques, as ChatGPT summarised them from the PDF. Each is to be verified against the paper:**
- **The super-alphabet:** precompute LF² = LF[LF[i]] with the two characters, so each dependent lookup yields two characters. This is the "nearly 2× fewer misses".
- **Several traversals on one thread,** eight in ChatGPT's account, attributed to out-of-order execution hiding the misses. This is the "cost of a miss halved".
- **Precopy:** for highly repetitive data, copy already-reconstructed text with sequential accesses instead of walking again. Per ChatGPT: 156.49 → 14.57 ns/char on an artificial repetitive set, but slower than the combined method on English and DNA.
- **Reported figures, per ChatGPT:** English 117.3 → 39.1 ns/char, on an Intel Core 2 Duo with a 4 MB L2.

## The inventory

### First generation: Scholar's "Cited by" of the root (C0)

| key | work | year | status |
|---|---|--:|---|
| `navarro2021-indexing-ii` | Navarro: Indexing Highly Repetitive String Collections, Part II: Compressed Indexes | 2021 | lead |
| `kempa2017-optimal-construction` | Kempa: Optimal Construction of Compressed Indexes for Highly Repetitive Texts | 2017 | lead |
| `kkp2016-lazy-lz` | Kärkkäinen, Kempa, Puglisi: Lazy Lempel-Ziv Factorization Algorithms | 2016 | lead (the root's authors) |
| `navarro2020-indexing` | Navarro: Indexing Highly Repetitive String Collections | 2020 | lead; possibly a version of the 2021 entry |
| `baier2018-undetected-redundancy` | Baier: On Undetected Redundancy in the Burrows-Wheeler Transform | 2018 | lead; its list is C2 |
| `daykin-watson2017-indeterminate` | Daykin, Watson: Indeterminate String Factorizations and Degenerate Text Transformations | 2017 | lead |
| `daykin2018-survey-orderings` | Daykin et al.: A Survey of String Orderings and Their Application to the Burrows–Wheeler Transform | 2018 | lead (survey) |
| `daykin2016-bbort` | Daykin et al.: Binary Block Order Rouen Transform | 2016 | lead |
| `baier-dede2019-tunnel-planning` | Baier, Dede: BWT Tunnel Planning Is Hard but Manageable (DCC 2019) | 2019 | lead; also in C2; its list is C3 |
| `kmk2012-grammar-precompression` | Kärkkäinen, Mikkola, Kempa: Grammar Precompression Speeds Up Burrows–Wheeler Compression | 2012 | lead; its list is C4 |
| `gnp2017-fast-locating-rlbwt` | Gagie, Navarro, Prezza: Fast Locating with the RLBWT | 2017 | lead; its list is C1 |
| `baierje2021-ambroxol` | J. E. Baier: Charakterisierung der inhibitorischen Wirkung von Ambroxol … (medical dissertation) | 2021 | **false positive** |
| `burrows-blocksorting` | Burrows: Block-Sorting Compression | n/a | lead (foundational) |
| `petri2024-scalable-succinct` | Petri: Scalable Succinct Indexing for Large Text Collections | 2024 | lead; year as transcribed |
| `ferragina-manzini2014-bwt` | Ferragina, Manzini: Burrows-Wheeler Transform (encyclopedia entry) | 2014 | lead (background) |
| `kempa2017-closing-in` | Kempa: Closing in on Time and Space Optimal Construction of Compressed Indexes | 2017 | lead; possibly related to the other Kempa 2017 entry |

That is 16 entries: 15 apparently scholarly works and 1 false positive.

### Second generation, grouped by the list that produced them

**C1: citing `gnp2017-fast-locating-rlbwt`** (1)
- `sadakane-navarro2017-computation-compressed`: Sadakane, Navarro, *Computation over Compressed Structured Data*, 2017, a Shonan meeting report.

**C2: citing `baier2018-undetected-redundancy`** (19; Scholar's "Cited by" count in brackets)

| key | work | year |
|---|---|--:|
| `fan2023-spectrum-tilings` | Fan, Khan, Pibiri, Patro: Spectrum preserving tilings enable sparse and modular reference indexing [20] | 2023 |
| `brown2026-colbwt` | Brown, Shivakumar, …: Col-bwt: Pangenomic seed chaining with maximal matches improves read classification [4] | 2026 |
| `brown2025-chain-statistics` | Brown, Shivakumar, Langmead: Improved pangenomic classification accuracy with chain statistics [5] | 2025 |
| `cotumaccio2025-wheeler-languages` | Cotumaccio, D'Agostino, …: Wheeler graphs and Wheeler languages [6] | 2025 |
| `alanko2019-tunneling-wheeler` | Alanko, Gagie, Navarro, Seelbach Benkner: Tunneling on Wheeler Graphs (DCC 2019) [28] | 2019 |
| `bannai2019-bijective-bwt` | Bannai, Kärkkäinen, …: Indexing the bijective BWT [22] | 2019 |
| `gagie2026-reportable-shortcuts` | Gagie: Reportable Shortcuts with RLBWTs (arXiv:2609.06596) | 2026 |
| `goga-balaz2022-pfp-tunnelled` | Goga, Baláž: Prefix-free parsing for building large tunnelled Wheeler graphs (arXiv:2206.15097) [9] | 2022 |
| `prezza2021-subpath-survey` | Prezza: Subpath queries on compressed graphs: A survey [7] | 2021 |
| `samofalov-polak2026-ternary` | Samofalov, Polak: Ternary Data Categorization for Evaluating the Impact of Data Transforms on Image Compressibility | 2026 |
| `bonizzoni2026-leaky-tunnels` | Bonizzoni, Cozzi, Gagie, Gao, …: Patching leaky tunnels in BWTs (arXiv) | 2026 |
| `langmead2025-wheeler-primer` | Langmead: We Are What We Index; a Primer for the Wheeler Graph Era (invited talk) | 2025 |
| `alanko2025-beyond-wheeler` | Alanko, Biagi, Equi, …: Graph Indexing Beyond Wheeler Graphs [1] | 2025 |
| `baier2022-edge-min-debruijn` | Baier, Büchler, Ohlebusch, Weber: Edge minimization in de Bruijn graphs [5] | 2022 |
| `baier-dede2019-tunnel-planning` | Baier, Dede: BWT tunnel planning is hard but manageable [6] (also first generation) | 2019 |
| `mustafa2022-petabase-thesis` | Mustafa: Algorithms for efficient sensitive search and sample comparison on petabase-scale genomics data (thesis) [5] | 2022 |
| `alanko2020-space-efficient-thesis` | Alanko: Space-Efficient Algorithms for Strings and Prefix-Sortable Graphs (thesis) [1] | 2020 |
| `baierje2021-ambroxol` | (false positive, as in C0) | 2021 |
| `gagie2018-relaxing-wheeler` | Gagie, Gourdel, Manzini, Navarro, Simpson: Relaxing Wheeler graphs for indexing reads ([CITATION] only) [1] | 2018 |

**C3: citing `baier-dede2019-tunnel-planning`** (6; every one is also in C2)
- `brown2026-colbwt`, `brown2025-chain-statistics`, `gagie2026-reportable-shortcuts`, `goga-balaz2022-pfp-tunnelled`, `bonizzoni2026-leaky-tunnels`, `baierje2021-ambroxol` (false positive).

**C4: citing `kmk2012-grammar-precompression`** (5)

| key | work | year |
|---|---|--:|
| `jez2015-fully-compressed-pm` | Jeż: Faster fully compressed pattern matching by recompression (ACM TALG) [110] | 2015 |
| `jez2015-approx-grammar` | Jeż: Approximation of grammar-based compression via recompression (TCS) [94] | 2015 |
| `deng2022-fm-grammars` | Deng, Hon, Köppl, …: FM-indexing grammars induced by suffix sorting for long patterns (DCC 2022) [11] | 2022 |
| `jez2020-recompression-talk` | Jeż: Recompression: Technique for word equations and compressed data (talk) [1] | 2020 |
| `cankaya-darwish2013-star-encoding` | Cankaya, Darwish: Improving compression performance with a star encoding front end [1] | 2013 |

### Works in more than one list

| key | lists |
|---|---|
| `baier-dede2019-tunnel-planning` | C0 (cites the root), C2 (cites Baier 2018) |
| `brown2026-colbwt`, `brown2025-chain-statistics`, `gagie2026-reportable-shortcuts`, `goga-balaz2022-pfp-tunnelled`, `bonizzoni2026-leaky-tunnels` | C2 and C3 |
| `baierje2021-ambroxol` | C0, C2, C3: one false positive repeated, most likely an author-name collision ("Baier") in Scholar |

**Totals:** 42 records in `works.csv` and 47 citation edges:
- 1 root;
- 15 first-generation works;
- 23 new second-generation works (17 from C2, 5 from C4, 1 from C1; the other 2 in C2 were already in C0), so 38 scholarly works besides the root;
- 1 false positive;
- 2 unverified leads from ChatGPT's topical search (below).

### Possible duplicates and versions (check before merging)

- `navarro2020-indexing` against `navarro2021-indexing-ii`: a preprint and journal "Part II"? The 2020 entry might correspond to Parts I and II together.
- `kempa2017-optimal-construction` against `kempa2017-closing-in`: two Kempa 2017 titles on optimal construction of compressed indexes. Possibly a preprint and a thesis, or two versions.

Both stay separate records until checked. Scholar showed them as separate entries.

## Unverified claims and leads (from ChatGPT's earlier messages)

These are recorded so they can be checked, not relied on. ChatGPT itself later withdrew its shortlist, its "no one has improved the 2012 techniques" conclusion and its "second-generation findings".
- **A publication venue:** `kempa2017-optimal-construction` was "published at SODA 2019".
- **`kempa-kociumaka2023-collapsing`** (Kempa, Kociumaka, *Collapsing the Hierarchy of Compressed Data Structures: Suffix Arrays in Optimal Compressed Space*, FOCS 2023) was found by topical search. **It is not shown to cite the root or any captured work.**
- **`cache-friendly-bw-inversion`:** the Helsinki group's publication page reportedly links a *Cache-Friendly Burrows-Wheeler Inversion*. It is a possible **precursor** (a backward reference) of the root and high priority to verify. It would be the "state of the art" the root improves on, or that work's immediate predecessor.
- **A characterisation of Baier 2018:** tunnelling "reduces compressed size by 8–16% on average". It changes the compressed representation, so on its own it would not speed up decoding existing `.bz2` files.
- **A characterisation of the grammar-precompression paper:** it "reduces the data passed through the transform"; that is, it also changes the compression pipeline.
- **The root's technique summary** above (the super-alphabet, eight traversals, Precopy, the ns/char figures).

## What came from us, and what from the literature

To keep attribution straight in the journal and in any library:

| finding | source |
|---|---|
| bz2 decode is ~half the split's CPU; the inverse-BWT walk is ~60% of the decode | **ours**, measured (#194) |
| the walk is latency-bound: K independent chains through one block, 28.5 → 4.4 c/B | **ours**, by direct experiment (#194); consistent with the root's account of multiple traversals hiding cache misses |
| interleaving whole blocks per core fails (K tables outgrow L2) | **ours** (#194) |
| the joined walk: random segment starts, a stop bitmap, joined in order (12 lanes) | **ours**; an implementation of multi-traversal for existing bzip2 blocks with no extra metadata. pbz2 (below) does the same thing independently. |
| PCLMULQDQ CRC, AVX2 RLE1 and AVX2 MTF | **ours**; not from this literature |
| several traversals per thread to hide miss latency | **literature** (the root, 2012); we rediscovered it empirically |
| the super-alphabet (LF², two characters per lookup) and Precopy | **literature** (the root); not yet tried by us |

**Software prior art** (from the bake-off, 2026-10-27):
- **pbz2 0.2.0** (Rust, MIT, `#![forbid(unsafe_code)]`) already walks the inverse BWT in interleaved chains: `walk_in_pieces`, 16 lanes, 128 evenly spaced starts, a page arena, then joined.
- **libbzip2** (Seward) is the format's reference implementation; its `tt` array is what our journals call the T-vector.
- Neither is in Scholar's graph. A library should cite both alongside the root.

## Terminology (for the journal and a future library)

- **BWT, inverse BWT (iBWT):** the transform and its inversion. In bzip2 the inversion runs per block, and **origPtr** (`orig_ptr`) is the row of the original string.
- **LF mapping (last-to-first):** the permutation inversion walks. The literature writes LF(i); libbzip2 builds it as the `tt` array, the **T-vector** in our notes. Use "LF mapping" in a library's documentation, and mention `tt` for bzip2 readers.
- **Super-alphabet:** treating q consecutive symbols as one symbol so each LF step yields q characters (q = 2 means LF²). The root's term; use it rather than inventing one.
- **Precopy:** the root's name for copying already-decoded repeated text instead of walking it.
- **Memory-level parallelism (MLP):** the computer-architecture term for overlapping independent cache misses. The root describes the effect through out-of-order execution. "Interleaved traversals" or "multiple traversals" names the algorithm, "MLP" the mechanism.
- **RLBWT:** the run-length compressed BWT. It is central to the indexing branch (the "r-index" line of work).
- **Tunnelling (Baier):** collapsing parallel runs ("tunnels") in the BWT for a smaller representation. It generalises to **Wheeler graphs**.
- **Bijective BWT:** a variant with no end marker or origPtr; it needs its own inversion.
- **Grammar compression, SLP, recompression:** grammar-based representations (the Jeż branch).
- **Prefix-free parsing (PFP):** a parsing step for building BWTs of very repetitive collections.

## Citation-network assessment (from listings only; counts are discovery signals, not relevance)

- **Keystones:**
  - `baier2018-undetected-redundancy` has 19 citers, and the whole **tunnelling → Wheeler-graph** branch hangs from it.
  - Navarro's surveys (`navarro2020-indexing`, `navarro2021-indexing-ii`) are likely hubs for the compressed-indexing branch. Their own citers aren't captured yet.
- **Same-author clusters:**
  - Kärkkäinen, Kempa, Puglisi and Mikkola: the root, `kkp2016-lazy-lz`, `kmk2012-grammar-precompression`, `kempa2017-*`, `bannai2019-bijective-bwt` (Kärkkäinen), `alanko2020-space-efficient-thesis` (Helsinki). This cluster is the most likely place for inversion engineering to have continued.
  - Gagie, Navarro and Prezza: RLBWT and Wheeler graphs; Gagie appears in five captured works across C1–C3.
- **Bridges:**
  - `baier-dede2019-tunnel-planning` cites both the root and Baier 2018, linking the inversion paper to the tunnelling branch.
  - `bannai2019-bijective-bwt` connects Kärkkäinen's group to the tunnelling citers.
- **Branches in the captures so far:**
  1. **Inversion engineering** (the root; the possible precursor `cache-friendly-bw-inversion`). The only one directly about decoding. Its descendants are not yet captured.
  2. **Tunnelling and Wheeler graphs** (Baier, Alanko, Gagie, Bonizzoni, Goga and Baláž, Cotumaccio, Prezza, Langmead). These are indexing and representation; for our question their relevance is indirect at best (see the hypotheses).
  3. **Compressed indexing for repetitive texts** (Navarro, Kempa, Gagie–Navarro–Prezza, Petri).
  4. **Grammar compression** (the grammar-precompression paper → Jeż, Deng et al.).
  5. **BWT variants** (Daykin et al.; the bijective BWT).
  6. **Applications in genomics** (Brown et al., Fan et al., Mustafa).
- **No captured citing work is, from its title or snippet, about inverting a standard BWT faster.** That is a statement about these captures, not a conclusion about the field.

## Hypotheses to check while reading (ours; unverified)

1. **Tunnels as copy opportunities.** A tunnel is a block of BWT rows that LF maps in parallel, so the text it stands for repeats. On a standard bzip2 block, would *detecting* tunnel-like structure while building the T-vector predict where Precopy can copy instead of walk? This connects branch 2 to the root's Precopy.
2. **Runs in the BWT give consecutive LF targets.** Within a run of equal characters, LF(i+1) = LF(i) + 1. That is the property RLBWT indexing exploits. Could an inversion use run boundaries to batch or prefetch LF steps? `gagie2026-reportable-shortcuts`, which takes shortcuts over long matching stretches, may hold a usable idea.
3. **Wikidata JSON is highly repetitive** (17.5× in bzip2, against ~3.5–4× for English), which puts it nearer the root's "highly repetitive" regime, where Precopy won. Whether that holds per 900 kB block is an empirical question; the queued follow-up's Precopy screen decides it.

## Reading and précis queue

Assessed from titles, snippets and citation context only. **Nothing here has been read in full.** "Applicability" means to inverting existing bzip2 blocks, as against building new formats or indexes.

### Full technical précis (ranked)

1. **`kkp2012-slashing` (the root).** Verify every technique and figure ChatGPT reported. Extract:
   - the exact super-alphabet layout (entry size, how q is chosen, construction cost);
   - the number of traversals and how their starting points are chosen;
   - when Precopy wins;
   - the test machines and data.

   Applicability: **direct**.
2. **`cache-friendly-bw-inversion` (to locate first).** The probable baseline or precursor: what "state of the art" meant, and which locality tricks came before the root. Applicability: **likely direct**.
3. **`baier2018-undetected-redundancy`.** The structure of BWT redundancy (tunnels) and how it is detected; test hypothesis 1. Applicability: **indirect** (a representation change), possibly a Precopy predictor.
4. **`gnp2017-fast-locating-rlbwt`.** LF over runs and the run-length structure; test hypothesis 2. Applicability: **indirect**.
5. **`gagie2026-reportable-shortcuts`.** Shortcutting long matching stretches in RLBWT steps; test hypothesis 2. Applicability: **indirect, speculative**.
6. **`navarro2021-indexing-ii` / `navarro2020-indexing`.** A survey map: look for any section on BWT inversion or decompression speed, standard terminology, and references to later inversion work. Applicability: **a discovery hub**.
7. **`kkp2016-lazy-lz`.** The root's authors on factorising repetitive text. Look for parsing used to copy rather than recompute, an idea related to Precopy. Applicability: **indirect**.
8. **`kmk2012-grammar-precompression`.** The same group, about reducing BWT work on repetitive data. Check what it says about inversion time and whether it reuses the root's inversion. Applicability: **indirect** (a format change).

### Targeted scan (ranked)

1. **`kempa2017-optimal-construction` / `kempa2017-closing-in`:** look for LF or inversion engineering sections and cache-aware construction; check whether the two are the same work.
2. **`alanko2020-space-efficient-thesis`** (Helsinki, from the root's group): its related-work chapter, for inversion engineering after 2012.
3. **`bannai2019-bijective-bwt`:** inversion of the bijective variant, and any traversal techniques that transfer.
4. **`daykin2018-survey-orderings`:** its references to inversion algorithms for BWT variants.
5. **`baier-dede2019-tunnel-planning`, `bonizzoni2026-leaky-tunnels`, `alanko2019-tunneling-wheeler`, `goga-balaz2022-pfp-tunnelled`:** how tunnels are found, which bears on hypothesis 1.
6. **`petri2024-scalable-succinct`:** engineering for cache-efficient succinct structures, and LF access patterns at scale.
7. **`burrows-blocksorting`, `ferragina-manzini2014-bwt`:** terminology and the canonical inversion statement, for library documentation.
8. **`prezza2021-subpath-survey`, `langmead2025-wheeler-primer`:** maps of the Wheeler-graph branch; check them for any decompression angle.
9. **`deng2022-fm-grammars`:** grammars with the FM-index; scan for any decoding speed-up.

### Bibliography only, for now

`daykin-watson2017-indeterminate`, `daykin2016-bbort`, `sadakane-navarro2017-computation-compressed`, `fan2023-spectrum-tilings`, `brown2026-colbwt`, `brown2025-chain-statistics`, `cotumaccio2025-wheeler-languages`, `alanko2025-beyond-wheeler`, `baier2022-edge-min-debruijn`, `mustafa2022-petabase-thesis`, `gagie2018-relaxing-wheeler`, `samofalov-polak2026-ternary`, `jez2015-fully-compressed-pm`, `jez2015-approx-grammar`, `jez2020-recompression-talk`, `cankaya-darwish2013-star-encoding`, `kempa-kociumaka2023-collapsing` (unverified lead).

**False positive:** `baierje2021-ambroxol`.

## Next captures that would help most

1. **The root's own "Cited by", raw** (C0 was a transcription), with exact venues and counts. Check that page 2 really had 6.
2. **The "Cited by" of the inversion-engineering cluster:** `kkp2016-lazy-lz`, `kempa2017-optimal-construction`, `kempa2017-closing-in`, `navarro2020-indexing`, `navarro2021-indexing-ii`.
3. **`cache-friendly-bw-inversion`:** find it, its "Cited by", and the root's own reference list (backward references, to see what "state of the art" meant in 2012).
4. **Beyond Scholar:** search the cited-by lists for "BWT inversion", "inverse BWT", "LF mapping" with "cache" or "parallel", "bzip2 decompression", and GPU/SIMD inversion. Practitioner work (decoders, GPU papers) may cite the root without being well indexed.
5. **Software:** pbz2's README or docs for any references it cites.

To add a capture: give it the next id (C5…), add new works to `works.csv` (one row per distinct work; reuse an existing key when the work is already there), and add one `edges.csv` row per listing.
