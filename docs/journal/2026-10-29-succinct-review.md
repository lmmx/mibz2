# The succinct-structures review: prior art, theory, designs and a census of ψ (29 October 2026; #79, research only)

This entry keeps a long discussion from being lost: where it came from, what was claimed, what was checked and how, what was corrected, and what was decided. Nothing in it changes production code.

**Results:**
- one measurement: the census in §9, untimed and deterministic (`examples/psi_census.rs`);
- its output, in `2026-10-29-succinct-review/` (`psi-census.md`, and `psi-census.jsonl`, one line a block).

## 0. Provenance, and how to read this entry

**Who took part:**
- The user.
- Two external chatbots whose messages the user relayed into the working session. They are named here as the user named them:
  - the **research chatbot**, which did the succinct-structures research and later read Seward's paper;
  - the **guardian chatbot**, which reviewed each proposal for experimental discipline.
- This repository's agent, which checked claims against code, papers and measurements, and kept the GitHub record.

**Where the record already lives:** the issues and PRs below are the record as it happened. This entry consolidates them.
- **#79:**
  - the link-only milestone;
  - the prior art;
  - Seward's lineage;
  - the refinements;
  - the ψ = select extension.
- **#81, #91, #92 and #93.**

**What is not in the repository:** the research chatbot's original long research log, the starting point of §1. It covered:
- the Rust succinct ecosystem;
- rank/select, the FM-index and wavelet structures;
- bzip2's small mode;
- 30 open questions.

The user shared it in the session, and only its conclusions are kept here.

**Every claim below carries one of four marks:**
- **read**: checked against a primary source or code by the agent;
- **measured**: ours, with the machine named;
- **relayed**: from a chatbot's reading, not checked here;
- **hypothesis**: untested.

## 1. Where it started, and the first verdict

**The research chatbot's starting question:** could succinct data structures (rank/select, FM-index, wavelet trees and matrices, compressed permutations) make mibz2's inverse BWT faster?

**The first review verdict** (the agent's, accepted by the user and both chatbots):
- **Keep rank/select and wavelet structures out of the walk.** Each step would become several *dependent* operations. Our walk is throughput-bound, with 32 interleaved lanes and misses overlapped. The measured elasticity of throughput to walk instructions is 0.16 (3x3) and 0.32 (4x1) (`2026-10-28-cooperative-inversion.md` §12).
- **The one candidate:** a *link-only* T-vector. The output byte is dropped from the entry and recovered from the sorted first column F. That keeps navigation a direct permutation, with one random load a step.
- **Test it where residency binds,** at and past the knee (independent workers at 4–8, 4x2, 5x2), not at 3x3.

**The order the user set, kept throughout:**
1. #78's MTF spill fix (done: a mixed result, §15.4 of the coop journal);
2. #81, the one-pass build (single-core test done, cooperative test next);
3. only then #79's compact-table work.

One controlled experiment at a time. No implementation of a succinct representation yet.

## 2. The link-only milestone (#79), and what was already measured

**Today's entry** at row j is `(i << 8) | L[j]`:
- bits 8–30 hold the link (the i with LF(i) = j);
- bit 31 is the segment START flag;
- bits 0–7 hold the row's last-column byte.

**The byte is redundant:** F is sorted, so F[j] is the c with C[c] ≤ j < C[c+1]. And `L[ψ(k)] = F[k]`, so a walk can emit F[row] and move to ψ(row).

**The variants, kept apart:**

| | entry | byte | build | what it isolates |
|---|---|---|---|---|
| A, today | `(i<<8) \| L[j]` | in the entry | init pass, then read-modify-write | the reference |
| A′, first-column packing (bz2ngrs) | `(i<<8) \| F[j]` | in the entry | store-only: c is the byte being scattered | the store-only build alone |
| B, link-only 4 bytes | `i` (and the flag) | from F | store-only | the recovery cost alone (B against A′) |
| C, link-only 3 bytes | 20-bit link + flag in 24 bits | from F | store-only, narrow stores at boundaries | the 25% footprint cut |

**The correction:** the build saving is not link-only's. A′ gets the store-only scatter while keeping the byte. So B's case rests on cheap recovery, and C's on residency alone.

**What #91 (the compact pilot, published in this review) had already measured** (VM, 8 rounds, **measured**):
- its `t` arm *is* B: ψ only, the byte from an n/32 window table (`FTable`) plus a step past bucket boundaries, a store-only build with production's run stores;
- **the build:** t1 against p1 1.030 (1.007–1.049); t4 against p4 1.001 (0.857–1.091). So the link-only, store-only construction benefit on the VM is about 3%;
- **the walk:** 0.826 and 0.794. That conflates three things, so **the recovery cost is not yet isolated**:
  - the loop form (`t` has no room passes; #77 measured those at 1.03–1.06);
  - the start test;
  - the F lookup;
- **the packed 20-bit `c` arm:** build 0.15× `t`, walk 0.74× `t`;
- **the split ll16 + ll4 `b` arm:** build 0.43×, walk 0.77×;
- **the VM's 32 MB L3 holds every table,** so no residency effect could show there.

**The missing control:** an `e` arm, the pilot's lane walk with the byte embedded (A′). Then `t` against `e` is the recovery cost under the same loop.

## 3. Prior art

| source | what it is | what it found | mark |
|---|---|---|---|
| Seward, "Space-time tradeoffs in the inverse B-W transform", DCC 2001 (doi 10.1109/DCC.2001.917175) | `basis`, `bw94`, `mergedTL`, `indexF`, `tree`, `treeopt` | §5 below | the paper wasn't read by the agent: **relayed** (the research chatbot read it), with the parts below **read** in secondary sources |
| Kärkkäinen & Puglisi, "Cache Friendly Burrows-Wheeler Inversion" ([PDF](https://www.cs.helsinki.fi/u/tpkarkka/publications/ccp2011.pdf)) | Seward's algorithms re-implemented for large n, plus `copy` | `indexF` "substantially more competitive than Seward did". Table III at 50 MB, speed against `mtl` (merged, 5n bytes): XML 0.88, DNA 1.02, ENGLISH 0.86, PROT 0.92, SOURCE 0.74. Their `indexF` uses 4n bytes, their `mtl` 5n, since n > 2^24 means the link and byte can't share a word. Future work: "An interesting avenue for future work is parallel inversion... explicit, with threads... or implicit, by programming to the out-of-order execution of the CPU" | **read** |
| libsais 2.12.1 (`libsais_unbwt_*`) | `P`: 4 bytes a row, ψ² links only. Two bytes recovered a step: `c = fastbits[p >> shift]`, then `while (bucket2[c] <= p) c++`. `fastbits` is 2^17 + 1 u16 entries (shift 3 at 900 k rows); `bucket2` holds bigram starts. `decode_1…8` interleave up to 8 chains, plus OpenMP | the fastest inverse BWT we have measured: 1.13× production on one i9 core, 1.08× at s4x1, 1.20× our 3x3 (coop journal §7.5). README: "bi-gram LF-mapping... decoding of two symbols simultaneously"; "sensitive to fast memory and software prefetching" | **read** (source and README) |
| libbzip2 1.0.8, `-s` | `decompress.c` 530–550: ll16 holds L, then `SET_LL(i, cftabCopy[L[i]]++)` overwrites it with LF links in place. A **pointer reversal** (a whole dependent walk) then turns them into forward links. The output walk does two loads a step plus `BZ2_indexIntoF`, an 8-step binary search over 257 bucket starts | "2.5 bytes per block byte... albeit at about half the normal speed" (bzip2(1)). That half pays for the extra traversal as well as the search and split loads, not for link-only storage as such | **read** (source, man page) |
| Lauther & Lukovszki, ESA 2005 | about 1.625 bytes per input byte in linear time; a tunable space–time variant | — | abstract **read** |
| Kärkkäinen & Puglisi, ESA 2010 (medium-space inversion) | "a smooth space-time trade-off" | — | citation only |
| Kärkkäinen, Kempa & Puglisi, "Slashing the Time for BWT Inversion", DCC 2012 | several symbols per random access | 2.3–4× | abstract via search; PDF 404 |
| Barbay & Navarro, STACS 2009; TCS 2013 (expanded) | permutations as shuffled increasing subsequences, or as runs | Theorem 3.11: 2n(1+H) + o(n log σ) + O(σ log n) bits, O(1 + log σ) access | **relayed** (the guardian checked the theorem) |
| Nishimoto & Tabei, ICALP 2021 | move structure: constant-time LF in space proportional to the BWT's runs, by splitting intervals so that the *next* one is found in bounded work | — | **relayed**. Our move-structure walk was a measured negative (+90–108% in the room walk, −59–63% as one chain) |
| Barbay, Gagie, Navarro & Nekrich 2010, alphabet partitioning | nH₀ + o(n)(H₀+1) bits; access and rank O(log log σ), select O(1) | — | **relayed** |
| Elias–Fano; partitioned Elias–Fano (Ottaviano & Venturini, SIGIR 2014); sdsl's Ψ encodings | sorted lists in about 2 + log(U/n) bits an element; partitioned; sampled gap codes for Ψ | — | from memory, not checked |

**What the prior art changes:**
- Link-only storage with recovery from F is 25 years old. An expert modern implementation, libsais, uses a form of it and is the fastest we've measured. libsais stays at 4 bytes a row and spends L2-sized tables to halve the steps.
- What remains untested:
  - byte-aligned 24-bit links;
  - single-symbol recovery from an L1-sized table;
  - residency at the i9's knee;
  - all of these inside our cooperative, interleaved walk.

## 4. Two early points, settled

- **"Is it new?"** No. Seward's `indexF` (2001) is the 4-byte link-only design, and libsais is a modern relative. The claim worth testing is performance, not novelty: whether cheap recovery plus a smaller table moves the space–time trade-off when several blocks share an L3 (the user and both chatbots agreed).
- **"Is `indexF`'s 'reasonable decompression speed' a euphemism, or the best of both worlds?"** Partly a euphemism, and a trade, not both worlds:
  - In the research chatbot's reading of Seward's tables, `indexF` keeps less than half of `mergedTL`'s throughput for 37.5% less memory.
  - "Reasonable" meant usable on 8 MB machines. It is a point on his frontier, not off it.
  - The frontier moves with hardware: by 2011 `indexF` ran at 0.74–1.02× (above), because misses grew relatively more expensive than computation.
  - For mibz2, B (4-byte link-only) cannot be on the frontier: same footprint, more work. Only C can be.
  - "Best of both worlds" should be read as **peak against peak**: C's best absolute throughput at 6–10 workers against today's best. That is #79's own pass rule.

## 5. Seward's paper, as the research chatbot read it (relayed)

- **§2:** the T-vector, with L[j] = F[T[j]]. Seward assumes the compressor reverses its input; bzip2 doesn't, which a footnote calls a design flaw.
- **§3.2, `bw94`:** stores occurrence ranks and adds C[L[i]] in the walk (a deferred base). The idea folds counting and numbering into one scan. It doesn't help #81: our counts come from the MTF stage, and `bw94` walks LF, which would need the reversal.
- **§3.3, `mergedTL`:** the link and the byte in one word.
- **§4, `indexF`** ("T overwrites L"): link-only, 20-bit links, a binary search over the cumulative counts. It may need a pointer reversal to give bzip2's output direction (confirmed in libbzip2's source, §3).
- **§5, `tree` and `treeopt`:** frequency trees recover links on demand. `treeopt` compresses the nodes, and its speed depends strongly on the alphabet (good on ecoli's 4 symbols, poor on near-uniform 256).
- **§6.3, Table 3:**

| algorithm | instructions/byte | memory refs/byte | simulated misses/byte | MB/s (K6-III 400 MHz) | memory (1 M block) |
|---|--:|--:|--:|--:|--:|
| `bw94` | 18.00 | 9.00 | 1.63 | 1.93 | |
| `mergedTL` | 20.00 | 8.00 | 0.88 | 3.10 | 4 MB |
| `indexF` | | | | (below half of `mergedTL`'s) | 2.5 MB |
| `treeopt-1M` | | | | | 2 MB |
| `treeopt-500k` | | | | (about 27% below `treeopt-1M`) | 1.5 MB |

  `mergedTL` executes 11% more instructions and is about 61% faster, with about half the misses.

**The mechanisms differ, and they were kept apart:**
- Seward's merge cuts misses per step: two arrays become one.
- C keeps one access a step and cuts footprint, which matters only where capacity binds.
- B adds an access a step, the opposite direction from the merge. It is only cheap if that access hits L1.
- What §6.3 supports is that instructions are cheap next to misses.

## 6. Theory: what the table actually holds

**Information (read, standard):**
- Given the counts n_c, the stable permutation and L determine each other. So the table carries log₂(n!/∏ n_c!) ≈ nH₀(L) bits, at most 8 a row.
- Not every arrangement is a valid bzip2 transform: the walk from `orig_ptr` must reproduce the text, and periodic inputs split ψ into equal cycles. But the valid ones are about a 1/n fraction, so the bound is tight to O(log n) bits (the research chatbot raised the point; the agent checked the size).
- Today's 32 bits a row are mostly a materialised answer: Seward's "cache of navigation answers".

**Direction (read, from `src/bwt.rs`):**
- The build writes i at slot k = C[c] + rank_c(L, i) = LF(i). So T[LF(i)] = i, and **the table holds ψ = LF⁻¹**, not LF.
- ψ is increasing over each F bucket's rows: ρ ≤ σ *contiguous* runs.
- For j in bucket c, ψ(j) = select_c(L, j − C[c]). **The walk needs select, not rank.**
- The table is therefore **an inverted index of L**: each byte's sorted positions, concatenated in F order. LF instead has shuffled increasing subsequences and needs rank.
- So the research chatbot's blocked-rank idea (a flat directory of per-block counts) is LF-oriented. The guardian noted it is not a drop-in for our ψ walk.

**The cost model** (agreed by all three):
- per step: bits a row, instructions, **dependent-load depth**, memory traffic;
- per design: construction cost, metadata footprint, and **peak throughput at the best concurrency**;
- what matters is which work can overlap the current miss and which must follow it.

The per-step access classes, in order of fit for 32 interleaved lanes:
1. one random load and arithmetic;
2. one random load and an independent L1 lookup;
3. independent loads that can overlap;
4. dependent loads in series.

The compressed-permutation results (Barbay–Navarro, CSA Ψ encodings, Nishimoto–Tabei) reach their access bounds through samples, rank/select or interval state: class 4. A theoretical O(1) bound doesn't mean one load.

**The research question:** how close to nH₀ can a ψ representation get while staying in class 1 or 2?

## 7. Recovering F's byte

- **The window table** (`FTable`): `tab[j >> k]` holds the byte at the window's start, corrected by `while j >= C[b+1] { b += 1 }`. The lookup's address is the current row, so it is off the link chain.
- **The bound:**
  - the fraction of rows in windows that hold a boundary is at most (σ − 1)·2^k/n: 0.91% at k = 5 and σ = 256 for a full block;
  - since the walk visits each row once, the number of steps that take the correction equals the rows past a window's first boundary, exactly;
  - the research chatbot proposed a uniform flag in the entry. Packing the flag beside the byte in a u16 saves the `cum` load in the common case.
- **The L1 constraint:** at 900 k rows, k = 5 is 27.5 KB, most of a 32–48 KB L1D. k = 7 is 6.9 KB.
- **Measured (§9):** at k = 5, 0.29% of rows are in mixed windows and 0.15% are stepped for, and 93% of mixed windows hold one boundary. So the correction is nearly free. The open question is the lookup's instruction cost and L1 occupancy inside the walk, which the `e` arm measures.

## 8. The designs on the table (all unscheduled; hypotheses)

1. **C, 24-bit link-only** (the agent):
   - **reads:** `read_unaligned::<u32>` at 3j, masked, with the allocation padded. A 4-byte read crosses a line for 3/64 positions (4.69%), an exact 3-byte read for 2/64. The alternative is 21 entries a line, with `j / 21` on the chain;
   - **writes in the parallel scatter:** a 4-byte store spills into the next entry. That is safe inside one (thread, bucket) sub-range, where destinations ascend. At each sub-range's last entry it would corrupt another thread's entry, or the next bucket's (a data race, UB), so those take narrow stores;
   - **residency, corrected here (§9):** the operative table is 3.6 MB, not the 1.9 MB first posted on #79. If the knee is set by table bytes (about 14.4 MB), it moves from 4.0 to 5.3 full blocks.
2. **Line-blocked Elias–Fano ψ, 16 bits a row** (the agent):
   - 32 rows to an aligned 64-byte line: base 20 + ℓ 4 + byte 8 + start mask 32 + unary high part 64 + 32·ℓ low bits (ℓ ≤ 12) = 512 bits;
   - a step: line = j >> 5, r = j & 31, high = tzcnt(pdep(1 << r, unary)) − r, and the low bits by a shift;
   - the byte comes from the header, so no F table is needed;
   - exceptions are the lines that cross a bucket boundary and the spans too wide.

   **The guardian's corrections, accepted:**
   - one line is one fetch but several loads, with decoding between the fetch and the next address. The chain lengthens by about 15 cycles, and prefetching waits on it;
   - full-line *ordinary* stores still read the line for ownership; only non-temporal stores avoid it, and they bypass the cache. The agent had overstated this;
   - 10 full-size tables at 16 bits are about 18 MB, close to the 20 MB L3. The knee moves, but it isn't removed.
3. **The FOR-14 / EF-12 hybrid** (the research chatbot): the same 64-byte record, in one of three formats:
   - **FOR-14:** 64-bit header plus 32 × 14-bit offsets from the base, for spans below 2^14, so ψ(j) = base + extract₁₄(slot);
   - **EF-12:** 64-bit unary high part plus 32 × 12-bit lows, for spans below 33 · 2^12 (about 135 k);
   - **an exception record.**

   Its point is to keep the common case to a shift, mask and add. The record address j >> 5 needs no F lookup.
4. **Blocked rank** (the research chatbot): L as bytes, plus per-block cumulative counts. 256 × 32-bit counters cost 1024/B bytes a row: 4 at B = 256, 1 at B = 1024, with a long local count. It is LF-oriented (§6), so it is not for our ψ walk.
5. **Alphabet-specialised recovery** (the research chatbot): a constant for σ = 1, comparisons for 2–4, and so on. It waits until a general lookup is measured.
6. **20 or 21-bit dense packing:** measured in #91's `c` arm, at 0.15× the build. It comes after C, and only if C is encouraging.

## 9. The census of ψ (measured; untimed, exact; `examples/psi_census.rs`)

**Method:**
- for every block of the bench corpus (`corpora/wikidata.bz2`, SHA-checked), ψ by a counting sort;
- every 32-row group classified, with rows counted rather than groups:
  - FOR-14 (span < 2^14);
  - EF-12 (span < 33 · 2^12);
  - crossing a bucket boundary;
  - span too wide;
- the F windows of 2^k rows that hold a boundary.

It runs in 1.3 s on the VM. `psi-census.md` is the output, `psi-census.jsonl` the per-block lines.

**The corpus, first:**
- 431 blocks: **233 full blocks** (899,809–900,000 rows, mean 899,990) and **198 tiny ones** (1–1,185 rows, about 0.03% of the rows).
- So per-block means over the corpus, such as the "mean block 486 KB, mean table 1.9 MB" in earlier entries and on #79, mix two populations. **The working tables are all 3.6 MB.**
- The tiny blocks' origin isn't established here.

**The records** (full blocks; the tiny ones change nothing at this precision):

| class | rows |
|---|--:|
| FOR-14 | **98.11%** |
| EF-12, not FOR-14 | 1.40% |
| crossing a bucket boundary | 0.29% (under the 0.91% bound) |
| span wider than EF-12 | 0.21% |
| all exceptions | **0.49%** |
| bits a row, exceptions stored at 24 bits | **16.12** |

**Rows by their group's span:**
- **77.6%** of rows are in groups of span exactly 31: 32 consecutive positions, a run of 32 equal bytes in L (the run stores' case);
- 91.1% are in spans below 2^10, 98.4% below 2^14, and 99.6% below 2^16.

**F windows:**

| k | mixed windows | rows in them | rows stepped | mixed windows with 1 / 2 / 3 / 4+ boundaries |
|--:|--:|--:|--:|--:|
| 4 | 0.14% | 0.14% | 0.07% | 17,766 / 548 / 129 / 419 |
| 5 | 0.29% | 0.29% | 0.15% | 17,530 / 762 / 169 / 352 |
| 6 | 0.56% | 0.56% | 0.29% | 16,755 / 961 / 283 / 346 |
| 7 | 1.08% | 1.07% | 0.56% | 15,776 / 1,065 / 411 / 436 |
| 8 | 2.06% | 2.04% | 1.06% | 14,661 / 1,040 / 528 / 615 |

**What it says:**
- **The hybrid's common case is nearly universal on this corpus.** 98.1% of rows decode as FOR-14 and 0.49% are exceptions, for an effective 16.12 bits a row. So the guardian's question, whether the corpus permits the simple case, is answered yes for Wikidata.
- **A sharper common case appears:** 77.6% of rows are in run groups, where ψ(j) = base + (j & 31). A record format for these needs one add after the fetch, which is less decode than today's shift. The run-length structure behind the move structure shows up here locally, with fixed addressing and no interval search. This is a hypothesis about design, not speed.
  - **Exact for steps too:** a full inversion visits each row once, so 77.6% of the walk's *steps* fall in run groups. The share of *time* is another matter: a format test and its branch still sit on every step (the guardian's caution).
  - **A two-level variant it suggests** (a hypothesis, not yet analysed): run groups carry no per-row data. So an 8-byte descriptor per 32-row group (base, byte, format, start mask: 2 bits a row) could serve them alone. Payload records, at 14 bits a row, would be kept only for the 22.4% of non-run groups. That is about 2 + 0.224 × 14 + exceptions ≈ 5 bits a row, at the price of a second, dependent load on about 22% of steps. It is a different point on the frontier from the fixed 16-bit record, and would face the same dependency-path audit.
- **F recovery:** at k = 5 the correction loop runs for 0.15% of rows, and 93% of mixed windows need one step.
- **What it doesn't say:** anything about speed. That needs the decode's dependency path (§10).

## 10. Decisions, open questions, and the sequence

**The sequence, unchanged:**
1. **#81's cooperative test:** a fused-scatter arm on the given-counts build, with its predictions written down first, then the i9. Its single-core test is done: 1.027×, below the 1.03 floor, with no end-to-end change (#93, not merged).
2. **#79's audit:**
   - read the generated code of #91's F lookup;
   - add the `e` arm to isolate the recovery cost;
   - port the winner to the room walk.

   Only then C, at 24 bits.
3. **The line-blocked designs** (EF, FOR-14/EF-12, run records) stay sketches until a written dependency path for one step shows the decode can sit on the chain. It must give the number of cycles from the line's arrival to the next address, and prefetching must be able to wait for it. The census now gives their sizes. A prototype comes after that analysis, not before.
4. **The MTF lead** (coop journal §15.4): keep the list's front 32 bytes in a register. It is a separate, later experiment.

**Open questions:**
- What is the minimum redundancy above nH₀ for class 1 or 2 access to ψ? (Theory; literature first.)
- Does a run-record format (ψ = base + r) let the walk's common step be *cheaper* than today's while halving the table?
- How does an alphabet-partitioned select (Barbay–Gagie–Navarro–Nekrich) map onto the cost classes?
- Where do the corpus's tiny blocks come from, and does any bench figure average over them where it shouldn't?

## 11. The two chatbots' final conclusions (stored verbatim), and what the census has answered

At the user's request ("State your conclusions, I'll have them recorded"), both chatbots wrote a final statement. They are kept verbatim and unedited beside this entry:
- `2026-10-29-succinct-review/conclusions-guardian.md`: the guardian chatbot's "Succinct-Structures Research Review — Final Conclusions";
- `2026-10-29-succinct-review/conclusions-research.md`: the research chatbot's "Research conclusions: Succinct Data Structures for High-Performance Inverse BWT".

Both are that chatbot's view, not measurements. They agree with each other and with this entry on:
- the facts: ψ = LF⁻¹ is select over 256 sorted occurrence lists, and the information is ≈ nH₀;
- the prior art;
- the cost model (dependency depth, not operation count, with peak throughput at the best concurrency);
- the candidates: 24-bit C first, the 16-bit FOR-14/EF-12 hybrid as an unscheduled sketch;
- the sequence: #81's cooperative test, then #79's audit.

**What the census (§9), run after they were written, has answered:**
- **The guardian's §8 "next research measurement" and the research chatbot's question 2 are measured for this corpus:**
  - FOR-14 holds 98.11% of rows, EF-12 1.40%, and exceptions 0.49% (boundary crossings 0.29%, spans too wide 0.21%);
  - the effective size is 16.12 bits a row with exceptions stored at 24 bits.

  By the guardian's own rule ("if ordinary records rarely qualify... deprioritise"), the hybrid is *not* deprioritised. Its next step is the exact bit layout and the dependency-path audit, and only then a microbenchmark.
- **The variation with alphabet and block size is not tested.** This corpus's working blocks are all full, with σ between 65 and 128. Another corpus is needed for that.
- **The F-bucket boundary distribution,** part of the guardian's "#79 audit", is measured: at k = 5, 0.15% of rows are stepped for, and 93% of mixed windows hold one boundary. What remains of the audit is the lookup's generated code, and the `e` arm.
- **One thing neither foresaw:** 77.6% of rows are in 32-row groups that are a single run in L, where ψ = base + r. A run-record format would decode with one add after the fetch.
- **The research chatbot's confidence table** marks "the proposed 16-bit hybrid records are practical on real BWTs" as unproven. Their *size* is now shown for this corpus (98.6% of rows fit a 64-byte record). Their *speed* remains unproven.

**Marks to keep in mind when reading them:**
- Seward's paper was read by the research chatbot, not by the agent ("directly established by the paper" is relayed).
- The mibz2 measurements they cite (the 14.4 MB knee, the ψ² / Precopy / move-structure negatives, #81's 1.027×) are ours, from the journals and CI.

## 12. Phase II: the research agenda and the coordinator's priorities (both stored verbatim)

After the census, two more texts were relayed to be recorded. Both are kept verbatim:
- **`2026-10-29-succinct-review/agenda-phase-2-research.md`:** the research chatbot's "Phase II: From compressibility to navigability", its five questions, and a proposed **Ψ representation atlas**:
  - (I) the structure of the translation population;
  - (II) the simplest 64-byte record;
  - (III) formats without branching;
  - (IV) the next pointer without a second dependent access;
  - (V) behaviour on other input distributions.

  New references: Gog, Navarro & Petri, *Improved and Extended Locating Functionality on Compressed Suffix Arrays*; Ottaviano & Venturini's partitioned Elias–Fano; Ferragina & Pucci's generalised Elias–Fano. **Not yet read by the agent.**
- **`2026-10-29-succinct-review/coordinator-phase-2.md`:** the coordinator, formerly called the guardian.
  - **The roles:** the research chatbot explores; the agent validates against the repository, builds controlled experiments, inspects machine code and records; the coordinator gates; the user decides.
  - **The order:** #81's cooperative test, then #79's audit, then the 16-bit fixed-line hybrid, then sub-16-bit work as theory only.
  - **The rule:** every experiment needs a falsifiable hypothesis, a matched control, explicit criteria and a stopping point. Negative results stay negative.

**How the agenda maps onto the work:**

| agenda item | where it stands |
|---|---|
| #81's cooperative test | built, checked and pre-registered (coop journal §16, #95), waiting for the i9 |
| the atlas's questions I, II and V (group structure per block, exact layouts and budgets, other inputs) | measurement only: the census extended (`psi_census`, next), not a prototype |
| (Ψ(j), segment marker(j), F[j]) as the record's required triple | adopted as the definition of a record's function |
| III–IV (branching, the dependency path) | analysis first, written down before any prototype |
| a prototype of the hybrid record | held, per the coordinator, until #81's cooperative result is in |

## 13. The atlas, measured: census v2 (untimed, exact; `examples/psi_census.rs`)

The census now classifies four record formats at three group sizes, gives quantiles per full block, measures L's runs, and runs on other inputs (`psi_census --inputs`). Outputs: `psi-census.md` (the bench corpus, superseding §9's table, with v1's figures within 0.2 points) and `psi-census-inputs.md`.

**The record family at 16 bits a row,** with G rows in 2G bytes:
- a 64-bit header: base 20, byte 8, format 2, and the segment-start mask;
- a payload of 16G − 64 bits;
- the formats:
  - **RUN**: span exactly G − 1, so ψ = base + slot, with no payload;
  - **FOR-(16 − 64/G)**: 14 bits at G = 32;
  - **EF-(14 − 64/G)**: 12 low bits and a 2G-bit unary high part;
  - **exceptions**: a group that crosses a bucket, or whose span is too wide. Its entries are stored whole, 32 bits each, for 16 + 32 bits a row.

**The required triple** (atlas §6): (ψ(j), start(j), F[j]).
- Ordinary records hold all three: ψ from base and payload, the start bit from the mask, F[j] as the header's byte.
- Groups that cross a bucket boundary have no single byte, so they're exceptions, and their 32-bit entries carry the byte.

### 13.1 The bench corpus

| full blocks | G | RUN | FOR | EF | crossing | too wide | exceptions | bits a row |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| 233 | 16 | 81.95% | 15.74% | 1.39% | 0.14% | 0.78% | 0.92% | 16.29 |
| 233 | **32** | **77.39%** | **20.73%** | 1.40% | 0.28% | 0.21% | **0.49%** | **16.16** |
| 233 | 64 | 72.18% | 25.50% | 1.73% | 0.55% | 0.04% | 0.59% | 16.19 |

**Per full block at G = 32** (min / p10 / median / p90 / max):

| measure | quantiles |
|---|--:|
| RUN | 63.7 / 72.2 / 78.0 / 80.9 / 88.0% |
| RUN or FOR-14 | 97.8 / 98.0 / 98.1 / 98.2 / 98.5% |
| exceptions | 0.4 / 0.5 / 0.5 / 0.5 / 0.6% |
| bits a row | 16.14 / 16.14 / 16.15 / 16.17 / 16.19 |

So the fit is not an aggregate artefact: every full block is within 0.05 bits of the mean. G = 32 is the best of the three. Smaller groups catch more runs but cross more boundaries (G = 16 doubles the exceptions); larger groups lose runs.

**L's runs** (= ψ's translation runs, §13.3):
- there are 15.1 M runs over 209.7 M rows, n/r = 13.8;
- weighted by rows, they are long: 51% of rows are in runs of at least 1,024, and 19% of rows in runs of 2^11–2^12;
- the Wikidata BWT is highly run-structured, which is why RUN dominates.

**The F windows:** unchanged from §9.

### 13.2 Other inputs (atlas V; each compressed by libbzip2 at level 9; G = 32)

| input | n/r | RUN | FOR-14 | EF-12 | exceptions | bits a row |
|---|--:|--:|--:|--:|--:|--:|
| random bytes, 2.7 MB | 1.0 | 0.00% | 99.11% | 0.00% | 0.89% | 16.29 |
| a 4-letter alphabet, random, 2.7 MB | 1.4 | 3.44% | 96.42% | 0.09% | 0.04% | 16.01 |
| English prose (this journal), 0.5 MB | 2.3 | 8.86% | 87.58% | 2.62% | 0.94% | 16.30 |
| Rust source, 0.4 MB | 3.1 | 22.29% | 74.09% | 2.57% | 1.05% | 16.34 |
| an ELF binary, 3.0 MB | 3.4 | 34.84% | 54.73% | 8.61% | 1.83% | 16.58 |
| already compressed (the corpus's bz2 bytes), 2.7 MB | 1.0 | 0.00% | 99.05% | 0.07% | 0.88% | 16.28 |
| periodic, period 1000, 2.7 MB | 892 | 96.60% | 0.24% | 1.15% | 2.01% | 16.64 |

**What it says:**
- **The 16-bit fixed record fits every input tested,** at 16.0–16.6 bits a row, with exceptions at 2.0% or less.
- **The worst case for crossings is reached** by random bytes and compressed data: σ = 256 gives 255 boundaries, 0.89% of rows against the (σ − 1)·G/n bound of 0.91%.
- **Wide spans** need a byte rarer than about 1 in 4,224 rows (32 occurrences spread over more than 135 k rows). They peak at 1.2% (periodic) and are near zero on random data.
- **The format mix is workload-specific.** RUN dominates Wikidata (77%) and periodic data (97%). FOR-14 dominates random, compressed and prose (88–99%). EF-12 matters only for the ELF binary (8.6%) and prose (2.6%). **The RUN fast path is Wikidata's; FOR-14 is the general one.**

### 13.3 Two conclusions for the design (agent's reading)

1. **ψ's translation runs are exactly L's runs.** This settles the research chatbot's question I.
   - In c's bucket, consecutive rows hold c's consecutive occurrences in L.
   - So ψ(j + 1) = ψ(j) + 1 exactly when those two occurrences are adjacent in L, that is, inside one run of c.
   - So the number of translation runs is r, the BWT's run count (the r of the r-index literature), and their lengths are L's run lengths.
   - A fixed group is RUN when it lies inside one run of L.
2. **With fixed records, RUN costs no less space than FOR-14:** a RUN group is FOR-14 with offsets 0..31. So **one FOR-14 extraction can serve every ordinary group but EF's**, which bears on the atlas's question III. With RUN counted as FOR, that path covers:
   - 98.1% of the corpus's rows;
   - 99.1% of random bytes, 99.9% of the 4-letter alphabet, 99.05% of compressed data;
   - 96.4% of prose and of source, 96.8% of periodic data;
   - 89.6% of the ELF binary.

   A separate RUN fast path would skip one payload load and a shift and mask, but adds a format branch. Whether that pays is a codegen and measurement question (atlas IV), which the prototype would decide. So the first prototype should have **one common path** (FOR-14, RUN folded in) plus EF and exceptions, with a RUN fast path as its variant.

**Still open:**
- the dependency path's cycles, from the line's arrival to the next address (atlas IV);
- the build's cost;
- the first-priority reading: Gog, Navarro & Petri on Ψ list encodings.

The prototype stays held until #81's cooperative result is in (§12).

## 14. Phase II's handoff checked: prior art read, the designs' exact sizes, and a one-step dependency analysis

Two more texts are stored verbatim:
- `2026-10-29-succinct-review/phase-2-findings-research.md`: the research chatbot's Phase II findings and technical handoff, with the **two-tier design**;
- `2026-10-29-succinct-review/coordinator-phase-2-verdict.md`: the coordinator's verdict. Its instruction:
  - keep three candidates: 24-bit links, the 16-bit fixed line, and the two-tier design;
  - before any implementation, count the two-tier design's storage exactly, per block, partial groups included, and account for its directory's residency and dependency cost;
  - #81 stays first;
  - no compact-representation implementation is authorised yet.

### 14.1 The two new prior-art claims, read at their sources

**Gog, Navarro & Petri, *Improved and Extended Locating Functionality on Compressed Suffix Arrays*** (preprint, 14 January 2015; an early partial version at SEA 2014). **Read.**
- They encode Ψ as "a sequence containing σ monotonically increasing runs". They compare Elias-δ, OptPFor and their modification of partitioned Elias–Fano, in blocks of 128 with stored block starts.
- **Uniform blocks** come from partitioned EF: a block "containing all the values in between two block representatives" costs zero bits. They apply the idea to OptPFor too, and note it "was not used in the original CSA implementation of Sadakane or in previous versions of sdsl". This is the RUN format's prior art.
- Blocks with a run transition "we store... as a prefix-sum encoded sequence". This is our boundary-crossing case.
- On a byte alphabet, partitioned EF answers Ψ[i] "in around 200 nanoseconds, which is roughly the cost of two memory accesses and thus close to optimal" (english.2108MB, Pizza&Chili). That is a persistent index with a top level plus a block, two dependent accesses: the cost our designs are trying to avoid.

**Groot Koerkamp, `cacheline-ef`** (GitHub README). **Read.**
- "packs chunks of 44 sorted 40-bit values into a single cacheline": a 4-byte base (the 32 high bits of the smallest value), 44 low bytes, and 16 bytes of unary high bits. That is 11.6 bits a value, with a span of up to 256 · 84.
- It "only requires reading a single cacheline per query", against about 3 for plain Elias–Fano, and offers `index` and `prefetch`. It is described in the PtrHash paper (arXiv 2502.15539).
- It is the prior art for one-line-per-query EF. Our fixed-line records specialise the idea: FOR-14, the byte and the start flags beside EF.

The EF-12 bound the research chatbot corrected to 135,167 is what `psi_census` already used (span < 33 · 2^12).

### 14.2 The designs' exact sizes, counted (census v3)

`psi_census` now counts groups by class and gives each design's exact bytes:
- **fixed line:** 64 bytes a group, plus an exception's 32 entries stored whole at 4 bytes each;
- **two-tier:** 8 bytes a group, plus 64 bytes for each FOR or EF group, plus an exception's 128 bytes.

The RUN share was always a share of *groups*: rows in actual 32-row groups, each block's partial last group counted as its own. So the coordinator's distinction between eligible rows and complete groups was already respected, and here it is counted directly.

**The corpus's 233 full blocks, G = 32:**

| design | bits a row | a full block's table |
|---|--:|--:|
| today's T-vector | 32.00 | 3.600 MB |
| 24-bit links | 24.00 | 2.700 MB |
| fixed-line records | **16.16** | **1.818 MB** |
| two-tier | **5.70** | **0.641 MB**: descriptors 225 KB, payloads 398 KB, exception entries 18 KB |

**Per full block** (min / p10 / median / p90 / max):

| measure | quantiles |
|---|--:|
| fixed line, bits a row | 16.14 / 16.14 / 16.15 / 16.17 / 16.19 |
| two-tier, bits a row | 3.98 / 5.13 / 5.59 / 6.53 / 7.89 |
| two-tier steps that need a second dependent load (rows not in a RUN group) | 12.0 / 19.1 / 22.0 / 27.8 / 36.3% |

So the illustrative 5.6 bits a row was close, and the exact figure is 5.70. But the two-tier design's second load is not a fixed 22%: it ranges from 12% to 36% of steps by block.

**Other inputs, G = 32** (`psi-census-inputs.md`):

| input | fixed line | two-tier | two-tier steps with a second load |
|---|--:|--:|--:|
| random bytes | 16.29 | 18.14 | 100% |
| a 4-letter alphabet | 16.01 | 17.46 | 97% |
| English prose | 16.30 | 16.73 | 91% |
| Rust source | 16.34 | 14.59 | 78% |
| an ELF binary | 16.58 | 12.72 | 65% |
| already compressed | 16.28 | 18.14 | 100% |
| periodic | 16.64 | 2.86 | 3% |

**The two-tier design is Wikidata-shaped.** Without runs it is larger than the fixed line, and every step takes two dependent loads. With runs it is far smaller. A decoder could choose per block at build time, since the RUN share is known then, at the cost of two walk loops.

### 14.3 The directory's residency (analysis)

The i9-10900K has 32 KB L1D and 256 KB L2 per core, and 20 MB of shared L3.
- **A full block's 225 KB directory does not fit L1.** On its own it would nearly fill a core's L2. But the two-tier design's working set is 641 KB a block, and its accesses are random across the block.
- **In the cooperative walk,** every thread of a team touches the whole block, so each core's L2 would see the whole 641 KB: mostly misses to L3.
- **On one core,** the directory alone at 88% of L2, competing with the payloads, output lines and walk state, should hit L2 often but not reliably.
- **So, unless measured otherwise, model both of its loads as L3-latency.** Its gain is capacity: 0.641 MB a block, so ten blocks are 6.4 MB, well inside the L3. The fixed line's ten blocks are 18.2 MB, near the L3's capacity, and today's are 36 MB, past it.

### 14.4 One step's dependency path, from the row known to the next row known (analysis; latencies assumed)

**The assumed latencies,** typical for this core and *not measured here*:

| operation | cycles |
|---|--:|
| L1 hit | about 5 |
| L2 hit | about 14 |
| L3 hit | about 42 |
| `pdep`, `tzcnt`, `imul` | 3 each |
| shift, add, `lea` | 1 |
| a branch mispredict | about 17, plus the younger work it flushes |

| design | address (from the row) | dependent memory levels | after the data arrives | format branch on Wikidata | chain, L3-resident | instructions a step (estimate) |
|---|---|--:|---|---|--:|--:|
| today | `t + 4·row` | 1 | `>> 8` | none | about 43 | about 17 (measured: 16.8 in the walk) |
| 24-bit | `t + 3·row` | 1 (4.7% of reads split across lines) | mask | none | about 44 | about 19, plus the F lookup off the chain |
| fixed line, RUN folded into FOR-14 | `r + 64·(row >> 5)`; the payload word's offset `(row & 31)·14 / 8` also comes from the row | 1: the header and payload words are independent loads from one line | shift, mask, add base | EF and exceptions only (1.9% of rows): predictable | about 45 | about 25 |
| fixed line, separate RUN path | the same | 1 | an add (RUN) or shift/mask/add (FOR) | **RUN 77% against FOR 21%, interleaved: unpredictable** | about 45, plus mispredicts | about 23 |
| two-tier | `d + 8·(row >> 5)` | **1 for RUN, 2 for FOR or EF** | an add (RUN); payload address, load, shift/mask/add (FOR) | **RUN 78% against the rest: unpredictable** | about 43 (RUN), about 90 (FOR), plus mispredicts | about 20 (RUN), about 30 (FOR) |

**What it says:**
- **The fixed line, with RUN folded into FOR-14, adds only about 2–3 cycles to the chain** at equal residency.
  - Every payload word's address comes from the row alone, so the line's loads issue together and the decode is a shift, mask and add.
  - Its cost is about 8 more instructions a step, which at the measured elasticity (0.16–0.32) is a few percent. Its gain is residency: the knee from about 4 to about 8 full blocks.
- **A RUN fast path is a mistake in an interleaved walk on Wikidata.** RUN and FOR groups alternate unpredictably, so a format branch mispredicts on about a fifth of steps. Each mispredict flushes younger instructions, including other lanes' issued loads and prefetches, which cuts memory-level parallelism. This firms up §13.3's recommendation: one common FOR-14 path.
- **The two-tier design cannot avoid that branch.**
  - Making it branchless means a conditional move that waits for the payload load on *every* step, which makes every step two loads.
  - So it pays mispredicts on roughly 12–36% of steps, plus a second dependent L3 load on the same steps.
  - At equal residency, its mean chain is about 0.78 × 43 + 0.22 × 90 ≈ 53 cycles, against today's 43.
  - It should therefore lose inside the L3 and can win only where today's table spills to DRAM, past the knee. There its tables (6.4 MB for ten blocks) stay resident while today's (36 MB) don't.
- **The ranking this predicts** (a hypothesis):
  - at or below the knee: today ≈ fixed line > two-tier;
  - past the knee: the fixed line, then perhaps the two-tier, can beat today. The two-tier design's advantage grows with concurrency and with RUN share.
  - Best-at-best-concurrency is the criterion (§4), so the fixed line is the first candidate to measure. The two-tier design is the second.

### 14.5 What follows, in the coordinator's order

1. **#81's cooperative test** (#95), waiting for the i9.
2. **#79's audit:** the F lookup's generated code, and the `e` control arm.
3. **The isolated navigation cost** the coordinator ranked next, as a matched pointer-chasing harness:
   - the real ψ of the corpus's blocks, in each representation: today's 32-bit table, 24-bit links, the fixed line with RUN folded into FOR-14 (and the RUN-branch variant), and the two-tier design;
   - walked by the same interleaved lane loop, at 1–10 concurrent blocks;
   - reporting steps a second and MB/s per core, with the build measured separately;
   - the predictions in §14.4, pre-registered when it is built.

   It is not authorised yet. It comes after #81's result.
4. **Only if that justifies it,** a decoder prototype with construction, exceptions and concurrency.
