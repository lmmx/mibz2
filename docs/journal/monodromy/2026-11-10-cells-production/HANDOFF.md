# Cells kernel into production: where the work stood when it was stored (10 Nov 2026, 17:55Z)

Stored because the session neared a usage limit. Nothing here is a result; it is what the
next session needs to go on. The record of the kernel itself is
`../2026-11-10-rauzy-veech.md`; of the literature task, `../2026-11-10-lyndon-literature.md`.

## The assignment

QUEUED 6774426061 on #145, from the Coordinator, priority 1 (Louis, relayed: "the Rauzy–Veech
work must reach the release pipeline"; then, verbatim from the Coordinator's session: "yes I
would very much like to see this shipped with production comparison ASAP"):

- a production PR that puts the cells kernel into `src/` as an inversion route for admitted
  blocks, on the paths CI measures; the one-thread `Segmented` kernel first, the team path
  in a second PR;
- admission: the free r/n test first, then the cells budget; a block that fails either
  keeps today's route, byte for byte; the PR says what a refused block costs;
- gates (PERFORMANCE-POLICY.md): the CI bench decides the merge (`wikidata` and `text`
  faster beyond noise, no primary-corpus loss); `random` and `records` weighed under rule
  3, so the admission must keep them on their old route;
- safety (B5): differential tests against the shipped decoder; a fuzz mode that forces the
  new route, with its reach counted; Miri on the unsafe parts; mutation-killed admission
  tests; the Guardian's review before merge;
- size: smallest first; the kernel behind a switch that is off by default, then the default
  flipped in a second PR on its bench;
- milestones on #145; the first is a design note, before any code.

Agreed with the Coordinator (session messages, 17:33Z to 17:46Z): the literature PR first
(docs only, merged on green), then slice one; a runtime switch preferred, so the i9 A/B is
one build. An i9 census of the example is already queued by the Coordinator
(`just bench-cells-vs-prod` at c79ff7d3, rule in LOG 6774716029); nothing to build for it.

## What is in hand

- `prod_call-path.json`, `prod_safety-tests.json`: two of four read-only maps of the
  production path, as their readers returned them (facts with path:line at master 1a871bb).
  The other two (the bench gates; what the example kernel lacks for `src/`) had not
  returned.
- From the call-path map, the facts the design starts from:
  - every block on the paths CI measures goes through `decode_block<I: bwt::InverseBwt>`
    (`src/lib.rs:332`); the kernel is a type parameter (`Scratch<I = bwt::Segmented>`,
    `src/lib.rs:278`), not a configuration value;
  - the records route (#324) branches at the top of `decode_block` (`src/lib.rs:341-362`):
    `reads_runs() && runs::takes_runs(b)`, `runs::front`, the `orig_ptr < n` check,
    `InverseBwt::invert_runs(&s.runs, n, b.orig_ptr, &mut s.ll, k)`, then `finish`;
  - **the narrowest place for a try-and-fall-back is inside `Segmented::invert_runs`
    (`src/bwt.rs:152-165`), before `runs::fill`**: it has the records, n, orig_ptr, the
    output and the kernel's state; no team path calls it; its present body is the fallback
    and clears `out` itself, so a refusal has nothing to undo;
  - two constraints: the records are private to module `runs` (no accessor), and a refusal
    must not surface as `invert_runs == false` (`decode_block` then reaches a
    `debug_assert!(false)`, `src/lib.rs:361`);
  - no cargo feature, environment variable or builder option selects a route today; routes
    are forced by the type parameter (`decode_with::<Plain>`), `Kernels::scalar()`, `Plan`,
    and in tests by editing `Symbols.max_len` round the admission edge. So a runtime switch
    would be the first of its kind; a second `InverseBwt` type (cells, then `Segmented`'s
    body) is the shape the code already has.

## Design decisions already taken (to go in the design note)

1. The right-end sweep as measured: by the literature entry's Theorem 5 no other order of
   steps makes the same grammar in fewer steps.
2. Admission in three tests: r/n (free, before the order stage); r₂ − 1 against the cells
   budget (proved floor S ≥ r₂ − 1, counted in the order stage); the cells budget inside
   the induction. Break-even on this container: S ≈ 0.16 n on Wikidata and JSON
   (entry 1, §8 item 4). A refused block must leave `out` untouched: the kernel writes to
   its own buffer until `write_cells`.
3. The kernel needs for `src/`: an `unsafe fn` boundary with its invariants stated; a
   buffer bound that is not 128 n bytes; no x86-only intrinsic on the path (CI builds
   aarch64 and Windows) and nothing Miri rejects; the decoder's own records.
4. Evidence to move with it: the complement-before-write check, `--arbitrary` as a test,
   the region-end assertion, the two-run columns `b^k a^l` (closed form: Euclid).

## Next steps, in order

1. Finish the literature PR (its fifth reader; see that entry's §3.7).
2. Read the two missing maps or redo them (`map-production-path`: bench-gates, kernel-gap).
3. Post the design note on #145 (route with `coord.py`).
4. Slice one in a scratch worktree off master; onto the branch when the literature PR has
   merged. One branch only: `claude/sharp-meitner-cx0v78`.
