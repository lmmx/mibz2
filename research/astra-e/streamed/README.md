# Expand Open-Return output in bounded chunks

The stock run frontend first validates the symbols, block length and primary row. Under the `moonshot` feature only, it then offers the block to a kernel that can append fully expanded bytes. The default method returns `None`. The research candidate accepts precisely OpenGated's run-density route. On all other routes it uses OpenGated. The library checks the CRC of the new output suffix with the stock scalar or hardware kernel, and restores the caller's original length on RLE1 failure or CRC mismatch. The combined-stream CRC still consumes the checked return value through the ordinary path. All shipping `finish` code and allocator behavior are unchanged.

The original width-16 queued endpoint constructor supplies the residual permutation, its dictionary, and the primary. Follow successor edges, emit their words in original order, and stop after exactly `n` coded bytes. Each edge emits at least one byte. A short marked cycle naturally restarts at the primary, and its last word is truncated exactly where the byte count reaches zero. Unmarked cycles need no traversal. For full-period blocks the real dictionary word can also be larger than one staging chunk.

The candidate retains a 32 KiB byte vector per worker. It flushes at about 16 KiB, splitting very long words before appending. Each append is at most 16 KiB, with less than 16 KiB already staged, so the vector stays under 32 KiB of logical contents. Each flush passes a certified RLE1 prefix into stock `rle::unrle`, retains at most four bytes, and eventually feeds the final suffix, including a possible unfinished run. CRC uses the final contiguous output in one pass. The caller's preexisting output is excluded from the block CRC. The candidate reserves space for the ordinary minimum output estimate before flushing, matching stock's common-case reservation and leaving expansion growth to the same vector allocator.

## RLE1 boundary argument

A token has either one byte or five bytes consisting of four equal bytes and their count. If bytes immediately on either side of a proposed cut differ, it cannot split two of the four equal bytes. If the four bytes preceding the cut are not all equal, it also cannot separate a run from its count. Together these conditions certify a token boundary. The flush tests the last four interior cuts. If none passes, it starts at the already certified beginning, consuming locally decidable one- and five-byte tokens while at least five bytes remain. Hence every intermediate flush ends on a whole token. Only the final call can encounter four equal coded bytes with a missing count; stock then reports `RunUnfinished`. The fast condition is sufficient, and the parsing fallback covers ambiguous count values and long stretches of equal bytes.

## What the change costs

The emitting loop keeps a remaining-byte count and tests the chunk length. Words exceeding 16 KiB use multiple appends. Each coded byte still enters a staging vector and each flush calls stock RLE1. This removes the required block-sized intermediate from the selected computation, at the price of chunk-boundary recognition and more RLE1 calls. Previous nonselected blocks may leave the decoder's ordinary scratch capacity resident. Report measured whole-process peak RSS and wall time separately from the logical staging bound. The advisory cache model may observe this shorter-lived staging but cannot verify machine performance.

## Reproduction

Use Rust 1.99.1, the canonical scorer corpus, and the library's `moonshot` feature.

```sh
python3 research/astra-e/streamed/prepare.py
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%h origin/master -- bench/moonshot
python3 bench/moonshot/moonshot.py score research/astra-e/streamed/candidate.rs --with-opend
python3 docs/journal/2026-11-18-open-return/frozen.py corpora/wikidata.bz2 /tmp/astra-e-frozen.bz2
bash research/astra-e/streamed/check.sh /tmp/astra-e-frozen.bz2
```

The targeted checker uses the scorer build's compiled libraries. It exhausts short ternary strings including ambiguous count values, compares 3,200 long-word and chunk-boundary inputs to monolithic RLE1, and runs scalar and detected block decodes with preexisting caller output. It checks frozen output against libbzip2, a one-row marked cycle in a block with many LF cycles, wrong stored CRC, retry into the same vector and scratch, and a late unfinished RLE1 run. `check.txt` records the outcome. The full scorer gate and the independent CI comparison must measure the feature-enabled head candidate against the baseline's independently built standard kernel.
