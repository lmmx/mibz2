Callgrind cross-check for the U1/U2 trial (untimed instruction simulation; not a perf counter, not a timing).

Toolchain: rustc 1.97.0 (2d8144b78, LLVM 22.1.6), cargo build --locked --release --example c30walk,
CARGO_ENCODED_RUSTFLAGS= (empty), CARGO_TARGET_DIR=target/rust-1.97.0.
Binary: target/rust-1.97.0/release/examples/c30walk, sha256
3ce86859be086ee1c47bce4ab7426ceffad3e7ffe6755c3a162f32d4ceea050f
(the listings in ../listings are from this binary; a clean rebuild reproduces it)
Valgrind: valgrind-3.22.0
Corpus: corpora/wikidata-20261026.bz2, sha256 cf13f4394e4ea0a38a0e9e3bc5271b2dce717c8f98fe3a0b8a5186c84216b8fe

Invocation (each builder alone in its own process; order v2 run 1, su run 1, v2 run 2, su run 2):
  valgrind --tool=callgrind --cache-sim=yes --dump-instr=yes --callgrind-out-file=only-$W-run$R.out \
    target/rust-1.97.0/release/examples/c30walk ctor2su-count --only $W corpora/wikidata-20261026.bz2
  W in {v2, su}. Each process decodes the corpus, builds its 434 blocks with that builder's build::<false>
  only, and prints an FNV-1a digest of every output (identical: 4efb1c13ae58340f).

Files:
  only-*-run*.callgrind.gz       raw profiles (positions: instr line; events: Ir Dr Dw I1mr D1mr D1mw ILmr DLmr DLmw)
  only-*-run*-functions.txt      cgparse.py: self/inclusive Ir, Dr, Dw of build::<false>, refill, finish, per callee
  only-*-run*-library-sites.txt  sites.py: every library call from build::<false>, per call site
  classes-run*.txt               classes.py: own instructions by exact execution count, v2 -> ctor2su
  cgparse.py, sites.py, classes.py  the parsers (header-aware; .gz input)
Commands (from the worktree root):
  python3 cgparse.py PROFILE 'Ctor2s?u?>::build::<false>|ctor2s?u?::W<false>>::(finish|refill)' --callees
  python3 sites.py PROFILE
  MINX=1 python3 classes.py BINARY V2_PROFILE SU_PROFILE '<c30walk::ctor2::Ctor2>::build::<false>' \
    '<c30walk::ctor2su::Ctor2su>::build::<false>' '<c30walk::ctor2::W<false>>::refill' '<c30walk::ctor2su::W<false>>::refill'
