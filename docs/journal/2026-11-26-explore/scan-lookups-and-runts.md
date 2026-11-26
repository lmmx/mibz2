# The scan's lookups a symbol, and the corpus's runt blocks

An `explore` session (26 November 2026), started cold from the corpus and the code: the
pinned corpus rebuilt here from Wikimedia's dump (`bench/wikidata.py`, SHA-256
`82f65ace…9915` checked), then two standalone counts that share no code with the decoder.
Nothing in `src/` changed. Two findings: one sizes the scanner's next lever and corrects the
premise it was conjectured on; the other explains a corpus statistic that has been quoted
without its cause.

**Tools** (both in `examples/`, research only):
- `huff_lookup_census FILE.bz2`: reads each block's header, selectors and code lengths with
  its own bit reader, decodes the symbols bit by bit as libbzip2 does, and counts the root
  table lookups each 50-symbol group would need under a `B`-bit root table holding up to `K`
  whole codes an entry. Untimed; 1.3 s on the corpus.
- `scan_runts FILE.bz2 [PASSES]`: times `stream::Scanner::next` a block, best of 9 passes,
  on this session's VM (4-vCPU Xeon @ 2.1 GHz, 260 MiB L3; not the i9).

## 1. The scan already decodes 1.8 symbols a lookup: the multi-symbol lever is about 15%, not 13-17% of the wall

**The premise being tested.** Record 129 (`docs/journal/2026-11-24-algorithm-level.md`,
the fourth course mining) read §57's 1.41-1.51 ns a symbol as "about 7 cycles, one
first-level cache load's latency a symbol". From that it named the lever, several symbols a
lookup, put the floor at 2.6 symbols a lookup (mean bits / 10), and conjectured 13-17% off
the best wall. §75 then made multi-symbol decoding "the first engineering lead for the top
speed". No frontier claim carries it; `surface.tsv`'s row `S` names only the pooled scan.

**What the shipped decoder does.** `src/huff.rs` builds a 10-bit root table whose entries
hold one *or two* whole codes (the pair builder), plus a second level past 10 bits. The
production group loops (`decode_group_win`, `decode_group_slice`) are branchless on the
count: every entry writes two `u16`s and advances by 1 or 2, and branch only on the rare
slow entry. So the pairs are in the 1.41 ns figure.

**The census** (wikidata corpus, 431 blocks, 24,979,299 symbols: the same count `Scanner`
gives, so the reader agrees with the decoder):

- mean code length 3.749 bits; RUNA/RUNB 39.4% of symbols; codes of at most 5 bits 86.0%,
  longer than 10 bits 0.68%;
- lookups a symbol, greedy within each 50-symbol group (a code past `B` bits costs two):

| B \ K | 1 | 2 | 3 | 4 |
|---|---|---|---|---|
| 10 | 1.0068 | **0.5556** (shipped) | 0.4876 | 0.4712 |
| 11 | 1.0037 | 0.5322 | 0.4536 | 0.4326 |
| 12 | 1.0016 | 0.5184 | 0.4114 | 0.3859 |
| 13 | 1.0005 | 0.5111 | 0.3847 | 0.3553 |

- the shipped table's lookups spread over most of it: in the 552 tables of the blocks with at
  least 100,000 symbols, the entries taking 90% of a table's lookups number 742 of 1,024 at
  the median (p10 591, p90 817), and 99% take 959. A Huffman stream's 10-bit windows are
  nearly uniform, as compressed data should be, so the root table's working set is the whole
  table.

**What it means.**
- **The scan costs about 13 cycles a lookup, not 7 a symbol at one load's latency.** §57's
  1.41 ns a symbol (k = 1, 5.2-5.3 GHz) over 0.5556 lookups a symbol is 2.54 ns, 13.2-13.4
  cycles, a lookup. The dependent step (table load at about 5 cycles, then the shift and the
  next peek) is roughly 7-8 cycles. About 5 cycles a lookup go elsewhere in A3: the group's
  264-byte window, the output's copy, the refill every five steps, the selectors' switch.
  That share is a second lever, untouched by multi-symbol entries.
- **The multi-symbol ceiling at the shipped table size is 12-15% of A3's lookups,** K = 3 or
  4 at B = 10 (0.5556 → 0.4876 or 0.4712). Record 129's floor of 2.6 symbols a lookup
  (0.375 lookups) isn't reachable: whole codes and group edges hold even K = 4 at B = 13 to
  0.355, and B = 13 needs 8 times the table.
- **Wider roots lose the cache.** Entries holding 3 or 4 symbols need 8 bytes (4 × 9-bit
  symbols, a count, a length), so a K = 4 table is 8 KiB at B = 10 and 32 KiB at B = 12. A
  block's six tables are all hot (§57: 438 switches in 1,163 selectors), and the census shows
  each table's whole extent is touched. That is 48 KiB at B = 10 and 192 KiB at B = 12,
  against the i9's 32 KiB L1d. Only a K = 3 entry packed into 4 bytes keeps today's 24 KiB.
- **The table builds grow with them,** and the runts (section 2) pay that in full: 198 blocks
  that each decode about 42 symbols would each build two tables of 2^B entries. A wider
  build must be gated on the block's selector count (`n_sel × 50` bounds its symbols before
  any is decoded).
- **Sized by §57's own numbers:** if a lookup's cost stayed at 13 cycles, K = 3 at B = 10
  would take 12.2% off A3, about 9.9% off the scanner (A3 is 81% of it). By the bottleneck
  migration in `leads/course-opportunities.md` (a 7% scanner cut gives 3.96% at 8 workers),
  that is roughly 5-6% of the wall at 8 workers. The upper bound, K = 4, gives about 6-7%.
  Neither is 13-17%. That is still above PERFORMANCE-POLICY's 3% line, so it is a lever
  worth a probe. Claim `multi-symbol-huffman-table` (idea) carries it, with its stop test.

## 2. The 198 non-Open blocks are runts the dump's compressor leaves

The corpus has 431 blocks. Every census since #933 reads "233 Open blocks out of 431" with
the Open rows at 99.996%. The cause wasn't recorded:

- **197 blocks decode to fewer than 1,000 bytes** (mean 39, median 22, max 274), and one
  more to 1,183. The other 233 decode to 898,817-900,000 bytes. Every block of real size is Open. The
  198 non-Open ones are runts, and they hold 0.03% of the symbols.
- **They complete 900,000-byte input chunks.** In 186 of the 197, the full block before the
  runt and the runt together decode to exactly 900,000 bytes. Of the 11 others, three open a
  window (it was cut mid-chunk), six follow another runt, and two follow a block of exactly
  900,000. 46 blocks decode to exactly 900,000 bytes with no runt after them.
- **The cause, by inference:** Wikimedia's compressor cuts its input every 900,000 bytes and
  compresses each chunk into blocks of one stream (no stream header appears in any 1 MiB
  window). bzip2's level-9 block holds at most 899,981 RLE1 bytes (`100000 × 9 − 19`), so a
  chunk whose RLE1 form doesn't shrink below that spills its last few dozen bytes into a
  block of its own. Which tool writes the dump wasn't checked. The same windowing of a
  dated dump makes this the dump's structure, not the corpus builder's.
- **What they cost now: almost nothing.** On the VM the scanner spends 0.255 ms of 46.6 ms
  (0.55%) on the 198 runts, 1.29 µs a runt. A runt's job, wake-up and in-order hand-off sits
  inside the 5 ms of plumbing that `ideal-parts/results/floor` closed.
- **Where they matter:**
  - *Per-block statistics:* a median or mean over "431 blocks" is a median over two
    populations. Censuses should report over the 233 full blocks (as #933 and #951 already
    do for Open), and counts like `SLOTS_PER_THREAD × threads` hold about 54% as many full
    blocks as places.
  - *The READY_BYTES floor doesn't bind on this corpus.* `Shared::full` stops the scanner
    when 12 MiB is ready *and* at least `threads + 2` places are in flight. 12 MiB ready
    means at least 14 full blocks ready, about 27 places with their runts. That is past the
    floor at every count to 20, so the runts never shrink it.
  - *Fixed per-block costs scale with them:* any change that adds a per-block cost to the
    scan (wider tables, section 1) pays it 431 times for 233 blocks' worth of data.

## What was not done

- No timing on the i9, and no prototype of a wider table: the lookup counts are exact, but
  the cycles a lookup are inferred from §57's figure and this census, not measured together.
- The VM's scan figures (1.83-1.88 ns a symbol in the larger blocks) aren't converted to
  cycles. The VM's clock under load is unknown and it has no PMU (as #951 found).
- The dump's compressor isn't identified.
