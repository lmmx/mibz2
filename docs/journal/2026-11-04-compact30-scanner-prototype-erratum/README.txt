Prototype 1 erratum evidence (charter §6.1 row 1; review of PR #146, comment 6630265636).
Untimed instruction simulation (callgrind), not a perf counter, not a timing.

Toolchain: rustc 1.97.0 (2d8144b78, LLVM 22.1.6), cargo build --locked --release --example c30walk,
CARGO_ENCODED_RUSTFLAGS= (empty), CARGO_TARGET_DIR=target/rust-1.97.0, at 7f74e9f (ctor2s.rs unchanged).
Binary: target/rust-1.97.0/release/examples/c30walk, sha256
b36885a7c5d3e38c178886727ac14d6119a1835bba5e64704bc180861050ad53
ctor2s's build::<false>, refill and finish, and v2's build::<false>, are identical, after address
normalisation, to the listings archived at 7f74e9f (0 differences each).
Valgrind: valgrind-3.22.0
Corpus: corpora/wikidata-20261026.bz2, sha256 cf13f4394e4ea0a38a0e9e3bc5271b2dce717c8f98fe3a0b8a5186c84216b8fe

Invocation (run 1, then run 2, one after the other):
  valgrind --tool=callgrind --cache-sim=yes --dump-instr=yes --callgrind-out-file=run$R.out \
    target/rust-1.97.0/release/examples/c30walk ctor2s-count corpora/wikidata-20261026.bz2
  ctor2s-count builds each of the 434 blocks with v2's build::<false> and then ctor2s's, in one
  process, and checks that the outputs are identical.

Files:
  run{1,2}.callgrind.gz          raw profiles: positions instr line; events Ir Dr Dw I1mr D1mr D1mw ILmr DLmr DLmw
  run{1,2}-functions.txt         cgparse.py: self and inclusive Ir/Dr/Dw of both build::<false>, refill, finish, per callee
  run{1,2}-library-sites.txt     sites.py: every library call from each build::<false>, per call site
  run{1,2}.console, .stdout      valgrind's summary and the harness's output
  cgparse.py                     header-aware parser: reads positions: and events:, handles '*', '+n', '-n', '(id) name',
                                 and .gz input
  sites.py                       per-call-site table (needs positions: instr line)
  parser-test-archived-profile.txt  cgparse.py on the profile archived at 7f74e9f
                                 (callgrind/per-instruction.callgrind.gz), and the original parser's crash on it
Commands:
  python3 cgparse.py PROFILE 'Ctor2s?>::build::<false>|ctor2s?::W<false>>::(finish|refill)' --callees
  python3 sites.py PROFILE
