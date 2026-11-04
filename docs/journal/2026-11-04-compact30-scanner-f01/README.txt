S62-F01 remedy for the U1/U2 trial (scanner journal §11): counted-only counters for every changed
path, the uncounted listing's identity, the counts, and the ledger replayed with counted
frequencies. Also S62-F04's caller boundary. Nothing was timed; no benchmark, no perf.

Toolchain: rustc 1.97.0 (2d8144b78, LLVM 22.1.6), cargo build --locked --release --example c30walk,
CARGO_ENCODED_RUSTFLAGS= (empty), CARGO_TARGET_DIR=target/rust-1.97.0 (the trial's recipe).
Pinned binary (49cb46b6, rebuilt clean in this worktree): sha256
3ce86859be086ee1c47bce4ab7426ceffad3e7ffe6755c3a162f32d4ceea050f, equal to the trial's archived binary.
Counted-counter binary (this commit's source): sha256
6b1f1d958f8ccd1f65d37c251a8a50a9bd621cd653bb8c036171ce09a9784d44.
Corpus: corpora/wikidata-20261026.bz2, sha256 cf13f4394e4ea0a38a0e9e3bc5271b2dce717c8f98fe3a0b8a5186c84216b8fe.

identity/   identity.json: asmid.py, pinned binary against the counted-counter binary, from
            <c30walk::ctor2su::Ctor2su>::build::<false> through its transitive callee closure
            (52 functions, 4,057 instructions, all identical), and this binary's build::<false>,
            refill and finish against the trial's archived listings (identical).
            control-build-true.json: the same tool on build::<true>, which the counters change
            (DIFFERENT, as it must be: the tool detects a change).
            ctor2su-{build,refill,finish}-false.asm: this binary's listings.
counted/    the release suites on the counted-counter binary, all exit 0: corpora (11 corpora x 4
            configurations; the primary record is wikidata-20261026.bz2, permille 0, extra false),
            adversarial, synthetic, synthetic-quick. Every pre-existing field of every jsonl record,
            and every console after removing the 19 new fields, equals the trial's archive.
ledger.json, ledger.txt   f01_replay.py's output: every changed frequency class, its counter
            expression, the counted value against the profile frequency, and the interval.
harness-callsite.asm      callsite.py's output: c30walk::main around both build::<false> calls,
            each instruction with its own count in the v2-alone and su-alone profiles (S62-F04).

Commands (from the worktree root; the pinned binary is a clean build of 49cb46b6 copied aside):
  CARGO_ENCODED_RUSTFLAGS= CARGO_TARGET_DIR=target/rust-1.97.0 cargo +1.97.0 build --locked --release --example c30walk
  B=target/rust-1.97.0/release/examples/c30walk; F=docs/journal/2026-11-04-compact30-scanner-f01
  T=docs/journal/2026-11-04-compact30-scanner-u1u2-trial
  python3 $F/asmid.py PINNED_BINARY $B OUT '<c30walk::ctor2su::Ctor2su>::build::<false>' \
    --archived "<c30walk::ctor2su::Ctor2su>::build::<false>=$T/listings/ctor2su-build-false.asm" \
    --archived "<c30walk::ctor2su::W<false>>::refill=$T/listings/ctor2su-refill-false.asm" \
    --archived "<c30walk::ctor2su::W<false>>::finish=$T/listings/ctor2su-finish-false.asm"
  $B ctor2su-check $F/counted/corpora.jsonl corpora/{mixed-independent,period-3,random,records,text-l1,text-l5,text,wikidata-20261026,wikidata-fitted,wikidata-heldout,zeros}.bz2
  $B ctor2su-adversarial $F/counted/adversarial.jsonl; $B ctor2su-synthetic; $B ctor2su-synthetic --quick
  python3 $F/f01_replay.py scanner $T $F/counted/corpora.jsonl ledger.json
  python3 $F/callsite.py PINNED_BINARY $T/callgrind/only-v2-run1.callgrind.gz $T/callgrind/only-su-run1.callgrind.gz \
    '<c30walk::ctor2su::Ctor2su>::build::<false>'
The replay and the call-site listing read archives only (the trial's raw Callgrind profiles,
which were taken on the pinned binary); neither compiles nor runs Rust.
