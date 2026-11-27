# Run-only blocks: the ready cap does not bound memory

Explore, 27 November 2026, master 1a61f5b, 4-vCPU Xeon @ 2.1 GHz VM (15 GiB), rustc 1.99.1,
`cargo build --release --example bzcat`. Off the map: every memory and admission figure in
the frontier is priced on wikidata, whose blocks decode to about 1 MB. A bzip2 block of runs
only decodes to at most ~46 MB (900,000 RLE1 bytes, 5 bytes for each 259 output bytes).

## Corpora (bzip2 1.0.8 -9, not checked in)

| file | input | .bz2 | note |
|---|---:|---:|---|
| text | 211.2 MB | 53.1 MB | repo docs+src concatenated, line-number varied; ~1 MB blocks |
| zeros | 2147.5 MB | 1,522 B | ~46 blocks of ~46.6 MB each |
| mixed | 1090.5 MB | 10.8 MB | 10 x (4 MB of text, 100 MB of zeros) |

Best of 3 runs, `bzcat THREADS [SLOTS] < f.bz2 > /dev/null`; wall, `ru_maxrss` and
`ru_minflt` from `wait4`. The default `bzcat` window is 2 x threads; `BzReader::parallel`
and `decode_parallel` use `SLOTS_PER_THREAD` = 4 x threads (src/read.rs:119, src/lib.rs:887).

| corpus | t | slots | wall ms | GB/s | max RSS MiB | minflt |
|---|---:|---:|---:|---:|---:|---:|
| text | 4 | 8 | 553-561 | 0.38 | 99-103 | 26.7k |
| text | 4 | 16 | 518 | 0.41 | 110 | 29.0k |
| text | 8 | 32 | 590 | 0.36 | 148 | 42.5k |
| zeros | 1 | 1..16 | 778-863 | 2.5-2.8 | 149-150 | 38.0k |
| zeros | 2 | 4 | 463 | 4.64 | 296 | 75.5k |
| zeros | 4 | 4 | 426 | 5.04 | 457 | 116.7k |
| zeros | 4 | 8 | 402-413 | 5.2-5.3 | 591 | 151.0k |
| zeros | 4 | 16 | 513 | 4.18 | 591 | 151.0k |
| zeros | 4 | 32 | 497 | 4.32 | 943 | 241.1k |
| zeros | 8 | 32 | 597 | 3.60 | 1,273 | 325.6k |
| zeros | 16 | 64 | 685 | 3.14 | 1,818 | 532.4k |
| mixed | 4 | 8 | 405-427 | 2.6 | 583-653 | 161.5k |
| mixed | 4 | 16 | 429 | 2.54 | 702 | 181.7k |

## What it shows

- **Memory follows the window, not the 12 MiB cap.** At t = 4, run-only RSS grows from 457
  to 943 MiB as slots go 4 to 32, about 46 MB a slot. A 1,522-byte input takes 1.27 GB at
  the library's default window for 8 threads, and 1.82 GB at 16 (where the stream's own 46
  blocks are the limit), a ~840,000x amplification. Text at the same settings: 99-148 MiB.
- **Why (src/parallel.rs):** `Shared::full` (l.402) applies `READY_BYTES` only when the
  *scanner* issues a block, and only as a floor of `threads + 2` in flight. Run-only blocks
  scan in microseconds, so the scanner fills the whole window with jobs before any block is
  ready; workers never consult the cap and decode every issued block into its own buffer
  (`Vec::with_capacity(n + n/16 + 64)`, then grown to ~46 MB by RLE1). The pool
  (`free_out`, l.961/972) then keeps up to `slots` such buffers until the decoder drops.
  The module doc's bound (slots x ~46 MB) holds; the cap's byte bound does not.
- **It also costs time:** at t = 4 the 16- and 32-slot windows are 24-28% slower than 8
  slots on zeros (513/497 vs 402 ms; CPU 1.37 to 1.61-1.68 s), in line with the extra
  first-touch faults (151k to 241k pages, ~370 MB more zeroed by the kernel).
- **At t = 1 the floor is three buffers** (~149 MiB at every window): worker, ready, reader.

## Not shown

No i9 run; no Python-level RSS; 4 vCPU oversubscribed at t = 8/16 (RSS is the point there,
not wall). Wikidata's admission is untouched: its ready bytes never reach the cap (explore
2026-11-26-explore-2). Lever and stop test: claim `ready-cap-worker-gate`.
