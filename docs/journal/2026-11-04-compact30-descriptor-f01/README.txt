S62-F01 remedy for prototype 2, the descriptor arrays (descriptor journal §10): counted-only counters
for every changed path, the uncounted listing's identity, the counts, and the ledger replayed with
counted frequencies. Also S62-F04's caller boundary. Nothing was timed; no benchmark, no perf.

Toolchain: rustc 1.97.0 (2d8144b78, LLVM 22.1.6), cargo build --locked --release --example c30walk,
CARGO_ENCODED_RUSTFLAGS= (empty), CARGO_TARGET_DIR=target/rust-1.97.0 (the prototype's recipe).
Pinned binary (29dd071f, rebuilt clean in this worktree): sha256
695ca5dd2ba9c2615c6d812694533d7520e88152fba56ba191f17c22f770d136, equal to the prototype's archived binary.
Counted-counter binary (this commit's source): sha256
5b1fed926fa711cd833e1d5f1f9e96331dcac424911746d219110ddd487e4bcb.
Corpus: corpora/wikidata-20261026.bz2, sha256 cf13f4394e4ea0a38a0e9e3bc5271b2dce717c8f98fe3a0b8a5186c84216b8fe.

identity/   identity.json: asmid.py, pinned binary against the counted-counter binary, from
            <c30walk::ctor2d::Ctor2d>::build::<false> through its transitive callee closure
            (52 functions, 4,132 instructions, all identical), and this binary's build::<false>,
            refill and finish against the prototype's archived listings (identical).
            control-build-true.json: the same tool on build::<true>, which the counters change
            (DIFFERENT, as it must be).
            ctor2d-{build,refill,finish}-false.asm: this binary's listings.
counted/    sequential.jsonl: `ctor2d-count --counted`, the counted build alone in the order and
            arenas of the profiled `ctor2d-count --only d` process (one Ctor2d, one pair of
            arrays, the 434 blocks in order); its output digest, 4efb1c13ae58340f, is the
            profile run's. This is the ledger's record: the arena-regrowth and reserve paths
            depend on arena history, which the three-thread check harness does not share
            (its primary record differs from this one in exactly bitmap_in_place,
            bitmap_reserves, rec_grows, rec_grow_frees, raw_grows and snaps_reserves; every
            other field is equal).
            The release suites on the counted-counter binary, all exit 0: corpora (11 corpora x 4
            configurations), adversarial, synthetic, synthetic-quick. Every pre-existing field of
            every jsonl record, and every console after removing the 27 new fields, equals the
            prototype's archive.
ledger.json, ledger.txt   f01_replay.py's output: every changed frequency class, its counter
            expression(s), the counted value against the profile frequency, and the interval.
harness-callsite.asm      callsite.py's output: c30walk::main around both build::<false> calls,
            each instruction with its own count in the v2-alone and d-alone profiles (S62-F04).

Commands (from the worktree root; the pinned binary is a clean build of 29dd071f copied aside):
  CARGO_ENCODED_RUSTFLAGS= CARGO_TARGET_DIR=target/rust-1.97.0 cargo +1.97.0 build --locked --release --example c30walk
  B=target/rust-1.97.0/release/examples/c30walk; F=docs/journal/2026-11-04-compact30-descriptor-f01
  P=docs/journal/2026-11-04-compact30-descriptor-prototype
  python3 $F/asmid.py PINNED_BINARY $B OUT '<c30walk::ctor2d::Ctor2d>::build::<false>' \
    --archived "<c30walk::ctor2d::Ctor2d>::build::<false>=$P/listings/ctor2d-build-false.asm" \
    --archived "<c30walk::ctor2d::W<false>>::refill=$P/listings/ctor2d-refill-false.asm" \
    --archived "<c30walk::ctor2d::W<false>>::finish=$P/listings/ctor2d-finish-false.asm"
  $B ctor2d-count --counted corpora/wikidata-20261026.bz2 > $F/counted/sequential.jsonl
  $B ctor2d-check $F/counted/corpora.jsonl corpora/{mixed-independent,period-3,random,records,text-l1,text-l5,text,wikidata-20261026,wikidata-fitted,wikidata-heldout,zeros}.bz2
  $B ctor2d-adversarial $F/counted/adversarial.jsonl; $B ctor2d-synthetic; $B ctor2d-synthetic --quick
  python3 $F/f01_replay.py descriptor $P $F/counted/sequential.jsonl ledger.json
  python3 $F/callsite.py PINNED_BINARY $P/callgrind/only-v2-run1.callgrind.gz $P/callgrind/only-d-run1.callgrind.gz \
    '<c30walk::ctor2d::Ctor2d>::build::<false>'
The replay and the call-site listing read archives only (the prototype's raw Callgrind profiles,
taken on the pinned binary); neither compiles nor runs Rust.
