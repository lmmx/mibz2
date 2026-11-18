# Reproduce Astra C

The [journal](../2026-11-18-astra-c-output-first.md) contains the proofs and verdict. The programs here are Python structural checks. Run with Python 3.10 or newer and assertions enabled.

## Inputs and outputs

The source base is `1edb29fdc3d42dfddfc3d60e46456b611323760c`. The data release is `2e4b34e086ac95e8218f1ee822a4652c8db5bb19`. It holds every 24th of the 233 large blocks of Wikidata stream SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. Zero is stream block 1. The inputs adapter reads the initial top bytes and widths through the end of those two arrays, coalesces the recorder's primary split, and supplies the primary separately. It reads no cut, bottom-order, rule, closure or output array. The original physical records consist of two bytes with a maximal count of 127. Reconstituting their count from the coalesced runs agrees with the archived census for every sampled block. The data archive and the dense sort-based oracle are testing conveniences, outside each proposed algorithm. [Input release](https://github.com/lmmx/mibz2/tree/2e4b34e086ac95e8218f1ee822a4652c8db5bb19) · [Archive reader](inputs.py) · [Pinned plain walk](https://github.com/lmmx/mibz2/blob/1edb29fdc3d42dfddfc3d60e46456b611323760c/src/bwt.rs#L673)

The two cursors receive a writable block buffer of known size. Each hit copies between disjoint memory views of that buffer. The fixture pre-fills it with a poison byte and compares every finished byte with dense inversion. The cursor and certificate see only the compressed column and primary. The buffer must remain readable for subsequent hits. Sending data to a write-only external sink while discarding the completed prefix would require retaining sources elsewhere.

Frozen checks three independent external digests. The last column hashes to `db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67`, the 899,931-byte marked output to `f87e5caa2d5f0df9d5958fe60901e2023ae04ac406603d4c1584c8bb4017fbd6`, and the 900,000 bytes after RLE1 to `d2584c09f1d643d6abff8a25513b63af4e05969c15c30256aed3a05c74281210`. The remaining nine raw hashes are calculated evidence, with independent dense checks of their input to RLE1. The corpus checks use pre-RLE1 bytes when recognizing punctuation. [Archived frozen extraction](../2026-11-18-open-return/evidence/checks.txt) · [Archived column digest](../2026-11-18-switch-elimination/evidence/checks.txt)

## Commands

From a checkout of this PR, the following reads only the specified data commit. Each comparison should be silent. An individual `.irec.xz` can be supplied to any real-block command to reproduce just its rows.

```sh
study=docs/journal/2026-11-18-astra-c-output-first
scratch=$(mktemp -d)
data=2e4b34e086ac95e8218f1ee822a4652c8db5bb19
git fetch --no-tags origin "$data"
git archive "$data" wikidata | tar -x -C "$scratch"
tapes="$scratch/wikidata"

python3 "$study/small.py" > "$scratch/small.json"
python3 "$study/section_small.py" > "$scratch/section-small.json"
python3 "$study/screen.py" "$tapes/0000.irec.xz" > "$scratch/frozen.csv"
python3 "$study/screen.py" "$tapes" > "$scratch/screens.csv"
python3 "$study/screen.py" --section "$tapes" > "$scratch/section-gate.csv"
python3 "$study/bare.py" "$tapes" > "$scratch/bare.csv"
python3 "$study/cursor.py" "$tapes" > "$scratch/cursor.csv"
python3 "$study/section.py" "$tapes" > "$scratch/section.csv"
python3 "$study/baseline.py" "$tapes" > "$scratch/cells.csv"
python3 "$study/ledger.py" operations > "$scratch/operations.csv"
python3 "$study/ledger.py" populations > "$scratch/populations.csv"
for file in small.json section-small.json frozen.csv screens.csv section-gate.csv bare.csv \
            cursor.csv section.csv cells.csv operations.csv populations.csv
do
    cmp "$study/$file" "$scratch/$file" || exit 1
done
```

## Counted work

Let $m$ be the number of capped input records, $r$ the number of coalesced runs and $g=\lceil n/64\rceil$. Both indexed variants scan $2m$ capped records. Each pass extracts the symbol and width, compares the symbol for continuation and accumulates width. The first updates a pair of 32-bit counts for each run. A 256-bucket prefix scan produces row and run-slot cursors. The second pass scatters $r$ eight-byte records, computing one signed translation per run, and a sentinel gives the final right boundary. Each record holds its start and byte in one word and its displacement in another. A sequential scan of these starts inserts $r$ start bits, writing $g$ 16-byte mask/rank entries including padding. This indexes the original one-step law everywhere, including other LF cycles. The compressed input can then be released.

For the on-demand stage, `operations.csv` counts the following units separately, which avoids assigning the same price to a dependent miss, an integer addition and a copied byte.

| Event | Work included |
|---|---|
| Direct byte | Derive page and bit from the row. Read rank and mask, restrict the mask to bits through this bit, popcount, and add the preceding rank to address the piece. Read its tagged start and displacement, emit its byte and add the displacement. A byte used to build a certificate reads the adjacent start, shifts both bounds, takes a max and min, and tests the stopping rule and output limit. The unanchored prefix needs only the current record. |
| Linked phrase query | One page-head lookup and one 16-byte record test for every traversed link, including all failed interval comparisons. A match supplies a length, source and exit. On a publication, clip to each affected page, read and rewrite its head, store its record, and compare append capacity. Doubling moves every old node once per growth and the line trace includes both sides. |
| Section query | A subtract and one indexed eight-byte load/test. On a hit unpack length/source/shift, clamp at the output limit, copy and advance. A publishing miss makes one extra bound intersection for the terminal section, packs the slot, and writes each covered ordinal once. Section starts have been known since the 256-bucket scan. |
| Output destination and source | The cursor and length give each destination and its successor. Every hit's source was saved only after bytes had been emitted. Both the source read and the destination write are charged, and all miss bytes are stored. `direct_steps + copy_source_bytes = n`. A hit truncated at the end has no exit update. |
| Downstream | The inverse emits the entire RLE1-coded string. The normal next stages examine $n$ marked bytes, write $u$ raw bytes and checksum $u$ raw bytes. These three logical stream lengths are in the per-block ledger. The comparison's cache trace ends at the last marked byte. The production RLE1 kernel uses overlapping vector loads, so logical stream lengths are not a native load count. Frozen has $n=899{,}931$, $u=900{,}000$. |

The linked layout needs the input, $8r+4$ bytes of addressed base records, $16g$ bytes of rank pages, a 2,048-byte histogram, $4g$ bytes of heads, 16 bytes per allocated linked slot and $n$ bytes of output. During growth, the old and new node arrays coexist. The section replaces the heads and linked array with $8f$ explicitly cleared bytes for a byte of frequency $f$. The index array accumulates the reduced return map during emission. Output, input and temporaries have different lifetimes. The footprint columns count the *distinct lines actually addressed*, including relocation. They do not assume that all those lines are simultaneously live.

The deterministic-successor screen gives chain contraction perfect byte sources and free access to their spelling. Building it would still need the base run laws, the intersections of two sorted tilings, the forced successor links, a way to read the concatenated labels, and $n$ destination bytes. Even under those concessions it must choose a successor at every nonfinal split-piece visit. The screen reports that count and the number of split pieces. The histogram-only screen charges raw selection scans and their exact aligned per-query line spans. Its scalar cursor and counting formula agree on every full-period case among 3,586 small marked columns. [Exact screens](screens.csv) · [Histogram-only counts](bare.csv)

## Cache and comparison assumptions

All reported fills are deterministic simulations of indexed byte addresses. Line size is 64 bytes, with fully associative LRU, write allocation, dirty eviction and a final dirty drain. A read fill is a fill triggered by a data read. Transfers are fills plus dirty writebacks. The four capacities are independent cold models of the same trace, with 512 lines at 32 KiB, 4,096 at 256 KiB, 32,768 at 2 MiB (one tenth of 20 MiB), and 46,811 at one seventh of 20 MiB. They do not simulate actual scheduling, associativity, prefetch, memory-level parallelism, other decoder stages, OS zeroing, allocator bookkeeping or a contended shared cache. Capacity estimates assume recycled backing buffers. The new index clear, input reads, internal node relocation, steady-state output writes and baseline position-table fill are charged. Python object overhead and verification traffic are outside the packed-layout model.

For Cells, `baseline.py` replays the verified source-level induction field references, including demand reads forming eager prefetch addresses. It remaps them to the release layout with aligned base allocations, 20/8-byte top/bottom slots, a zero debug gap and the correct eager/lazy budget. Original-run input scans and histograms are charged in their respective phases. The first scan's histogram uses a register accumulator for the prefix sums. Repeated loads of a lookahead cap within one already scanned group are represented by that group's span. The same cache continues through the marked spine, position fill and owner scan, including queue records, arena payloads, output aliases and source/destination copying. Long copies touch their source span before the destination. The induction field counts and writer work are asserted against the published evidence. Different vector copy orders, allocation alignments, branch costs and cache replacement can change the ratios. Gate comparisons include the eight actual eager and nine actual lazy admissions. The 233-block census supplies only those gate flags and the cross-check of counts. [Induction trace model](../2026-11-18-cells-endpoint-sweep/traffic.py) · [Gate and writer receipts](../2026-11-18-cells-live-reuse/blocks.csv) · [Pinned output and CRC path](https://github.com/lmmx/mibz2/blob/1edb29fdc3d42dfddfc3d60e46456b611323760c/src/parallel.rs#L1019)

The `extra_read_premium` in `populations.csv` divides transfer savings by excess demand-read fills where both are positive. It gives the change of sign in an explicitly additive cost model. The four cache levels must not be summed as simultaneous DRAM costs. Native speed and shared bandwidth at 7–10 workers remain questions for independent measurement.
