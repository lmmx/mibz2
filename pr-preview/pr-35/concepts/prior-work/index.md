# Prior work

- **The format and its checks are libbzip2's** (Julian Seward, bzip2 1.0.8, `decompress.c`). mibz2 follows its decoding rules and makes each of its checks, in its terms. Its Huffman decode tables are built as `BZ2_hbCreateDecodeTables` builds them, so a stream libbzip2 refuses is refused here too. It is a separate implementation in Rust, not a port of libbzip2's code. The tests use libbzip2 itself (the `bzip2` crate's bundled 1.0.8) as the oracle.
- **Interleaving several traversals of one BWT inversion** on one thread, so that out-of-order execution keeps their loads in flight, is Kärkkäinen, Kempa and Puglisi's idea: "Slashing the Time for BWT Inversion", DCC 2012, pp. 99–108.

  - They also use a super-alphabet (an LF² table: two characters a dependent lookup) and "Precopy" for repetitive data. Neither is used here.
  - What is particular here is only how the traversals are found in an existing bzip2 file, which carries no metadata for them: the random starts, the stop bitmap, and the joining in order.
- **Block-parallel decoding** is lbzip2's (Mikołaj Izdebski) and pbzip2's approach.
- **pbz2** (Soof Golan) arrived independently at a segmented, multi-chain walk too; the bake-off that found it is in the [journals](<https://lmmx.github.io/mibz2/pr-preview/pr-35/background/journals/index.md>).
