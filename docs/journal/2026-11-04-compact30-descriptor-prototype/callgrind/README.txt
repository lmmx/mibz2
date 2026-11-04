Callgrind cross-check for prototype 2 (untimed instruction simulation; not a perf counter, not a timing).

Toolchain: rustc 1.97.0 (2d8144b78, LLVM 22.1.6), cargo build --locked --release --example c30walk,
CARGO_ENCODED_RUSTFLAGS= (empty), CARGO_TARGET_DIR=target/rust-1.97.0.
Binary: target/rust-1.97.0/release/examples/c30walk, sha256
695ca5dd2ba9c2615c6d812694533d7520e88152fba56ba191f17c22f770d136
Valgrind: valgrind-3.22.0
Corpus: corpora/wikidata-20261026.bz2, sha256 cf13f4394e4ea0a38a0e9e3bc5271b2dce717c8f98fe3a0b8a5186c84216b8fe

Invocation (each builder alone in its own process, two runs each, in the order v2 run 1, d run 1, v2 run 2, d run 2):
  valgrind --tool=callgrind --cache-sim=yes --dump-instr=yes --callgrind-out-file=only-$W-run$R.out \
    target/rust-1.97.0/release/examples/c30walk ctor2d-count --only $W corpora/wikidata-20261026.bz2
  with W in {v2, d}. Each process decodes the corpus, then builds its 434 blocks with that builder's
  build::<false> only, and prints an FNV-1a digest of every output (identical for v2 and d: 4efb1c13ae58340f).

Files:
  only-*-run*.callgrind.gz     raw profiles (positions: instr line; events: Ir Dr Dw I1mr D1mr D1mw ILmr DLmr DLmw)
  only-*-run*-functions.txt    cgparse.py: self and inclusive Ir/Dr/Dw of build::<false>, refill, finish, and each callee
  only-*-run*-library-sites.txt sites.py: every library call from build::<false>, per call site
  classes-run*.txt             classes.py: build::<false>'s and refill's own instructions by exact execution count
  cgparse.py                   header-aware parser (reads positions: and events:, handles '*', '+n', '-n')
Commands:
  python3 cgparse.py PROFILE 'Ctor2d?>::build::<false>|ctor2d?::W<false>>::(finish|refill)' --callees
  python3 sites.py PROFILE
  MINX=1 python3 classes.py BINARY V2_PROFILE D_PROFILE   (from the worktree root; profiles gunzipped first)
