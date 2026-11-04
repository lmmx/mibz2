The compact30 combined build, assembly 1 (charter §6.3; journal
docs/journal/2026-11-04-compact30-combined-build.md). Untimed: no benchmark, perf counter,
timing, i9 or workstation use. Callgrind is an untimed instruction simulation.

Source: examples/c30walk/ctor2c.rs, plus a mod line and the ctor2c- dispatch in
examples/c30walk.rs.
  c4e45d30  the profiled source (every listing and profile here)
  589d810e  counted-only counters for the paths the components' counters miss
  591c84f7  the n1 counter moved after U2 (counted-only), so MU1 is reported at U2
Toolchain: rustc 1.97.0 (2d8144b78, LLVM 22.1.6), cargo build --locked --release --example
c30walk, CARGO_ENCODED_RUSTFLAGS= (empty), CARGO_TARGET_DIR=target/rust-1.97.0.
  profiled binary (c4e45d30): sha256 c21ec7416937a6494a355c8683e0efd88141c977f96f37dc70aad25ede8230dd
  counted-counter binary (591c84f7): sha256 bc5fdae60e6b8ddc1ed2502b81b95d2db1dcc1bf6c505a121bcc7de7fcfb9808
Valgrind 3.22.0. Miri 0.1.0, nightly rustc 1.101.0 (16d1ffac1, 2026-10-26).
Corpora: corpora/*.bz2 as 2026-11-02-compact30-census/corpora.sha256; the primary is
wikidata-20261026.bz2 (cf13f439...).

suites/      the release suites on the profiled binary (corpora 44 records, adversarial 200,
             synthetic 54,287 cases, quick 869), every check exact, exit 0
counted/     the same suites on the counted-counter binary, and sequential.jsonl: the counted
             build alone in the --only c process's order (ctor2c-count --counted), the ledger's
             frequencies (output digest 4efb1c13ae58340f)
listings/    the profiled binary's ctor2c build::<false>, refill, finish (the leaf), finish_slow,
             and v2's build::<false>, refill and finish from the same binary
callgrind/   the four raw profiles (each builder alone: v2 run 1, c run 1, v2 run 2, c run 2),
             their consoles and stdout, and the binary's sha256
identity/    asmid_c.py, profiled against counted-counter binary, build::<false>'s closure (53
             functions, 4,509 instructions: identical); control/ (build::<true>: DIFFERENT);
             kept-comments/ (the unmodified asmid.py: only GCC_except_table992/993 comment
             labels differ)
ledger.json, ledger.txt   comb_replay.py's output: every changed class, its counter expression,
             counted value against profile frequency, listing ranges, and the interval
floor.json   floor.py: O, F, pass conditions (i) and (ii), v3's §7(b) and §7(c)
overlap-classes.json      overlap.py: the scanner's and descriptor's isolated class tables side by
             side, and the B2 classes both credit
harness-callsite.asm      callsite.py: c30walk::main around both build::<false> calls (S62-F04)
gate2/       paths.txt (every class, B2 -> scanner -> descriptor -> combined), stack.txt (dynamic
             stack traffic), row-loops.txt, and ndiff.py diffs of each function against B2 and
             each component's isolated listing
mutations/   each mutant's diff against the committed source, its build and its console: M1, M2,
             DM1, DM2, V3M (release, quick suite); MU1, MU3 (Miri on scanner:controls, default and
             -Zub-checks=no); MU2 on the U1 form (variant-a.diff), and the unmutated variant a
miri/        one console per registered class of the union, summary.txt, toolchain.txt
rebuild/     the profiled commit rebuilt clean in another worktree: identical .text, .rodata,
             .data.rel.ro and .eh_frame (sections.txt) and normalised closure (identity.json);
             the whole-binary hash differs by the embedded build path (binaries.sha256)
tools/       every script used; comb_replay.py's parser is the S62 review's, copied unchanged

Commands (from the worktree root, with corpora/ linked to the census corpora):
  CARGO_ENCODED_RUSTFLAGS= CARGO_TARGET_DIR=target/rust-1.97.0 cargo +1.97.0 build --locked --release --example c30walk
  E=docs/journal/2026-11-04-compact30-combined-build; B=target/rust-1.97.0/release/examples/c30walk
  # suites
  $B ctor2c-check OUT.jsonl corpora/{mixed-independent,period-3,random,records,text-l1,text-l5,text,wikidata-20261026,wikidata-fitted,wikidata-heldout,zeros}.bz2
  $B ctor2c-adversarial OUT.jsonl; $B ctor2c-synthetic; $B ctor2c-synthetic --quick
  # the counted record
  $B ctor2c-count --counted corpora/wikidata-20261026.bz2 > sequential.jsonl
  # profiles (at c4e45d30's binary; about 10 minutes each)
  sh $E/tools/cg.sh PROFILED_BINARY OUT_DIR corpora/wikidata-20261026.bz2
  # listings
  python3 $E/tools/listing.py PROFILED_BINARY OUT.asm '<c30walk::ctor2c::Ctor2c>::build::<false>'   (and refill, finish, finish_slow, v2's)
  # identity of the uncounted closure, profiled against counted-counter binary
  python3 $E/tools/asmid_c.py PROFILED_BINARY $B OUT '<c30walk::ctor2c::Ctor2c>::build::<false>' \
    --archived "<c30walk::ctor2c::Ctor2c>::build::<false>=$E/listings/ctor2c-build-false.asm" \
    --archived "<c30walk::ctor2c::W<false>>::refill=$E/listings/ctor2c-refill-false.asm" \
    --archived "<c30walk::ctor2c::W<false>>::finish=$E/listings/ctor2c-finish-false.asm" \
    --archived "<c30walk::ctor2c::W<false>>::finish_slow=$E/listings/ctor2c-finish-slow-false.asm"
  python3 $E/tools/asmid_c.py PROFILED_BINARY $B OUT '<c30walk::ctor2c::Ctor2c>::build::<true>'   # control: DIFFERENT
  # the ledger, the floor and §7 (archives only; no Rust is compiled or run)
  python3 $E/tools/comb_replay.py $E/callgrind $E/counted/sequential.jsonl ledger.json --listings $E/listings
  sh $E/tools/fetch_archives.sh . ARCH            # the components' listings and profiles, by git show from their pins
  python3 $E/tools/overlap.py ARCH/scanner ARCH/descriptor overlap-classes.json
  python3 $E/tools/floor.py ledger.json $E/listings/ctor2c-finish-false.asm $E/listings/ctor2c-build-false.asm overlap-classes.json floor.json
  python3 $E/tools/gate2.py $E ARCH OUT_DIR      # paths.txt and stack.txt
  python3 $E/tools/ndiff.py ARCH/b2/ctor2-build-false.asm $E/listings/ctor2c-build-false.asm --label B2,comb
  python3 $E/tools/callsite.py PROFILED_BINARY $E/callgrind/only-v2-run1.callgrind.gz $E/callgrind/only-c-run1.callgrind.gz '<c30walk::ctor2c::Ctor2c>::build::<false>'
  # mutants (restore with git checkout examples/c30walk/ctor2c.rs after each)
  python3 $E/tools/mutate.py M1|M2|DM1|DM2|V3M; build; $B ctor2c-synthetic --quick      # must fail
  python3 $E/tools/mutate.py MU1|MU3|MU2|variant-a
  CARGO_TARGET_DIR=target/miri [MIRIFLAGS=-Zub-checks=no] cargo +nightly miri run --example c30walk -- ctor2c-synthetic --miri scanner:controls
  # Miri over the union of the registered classes, one class per process
  sh $E/tools/miri.sh OUT_DIR            # or with class names, e.g. scanner:epoch v3:span
The tools import each other: run them by path (tools/ is put on the import path by Python).
