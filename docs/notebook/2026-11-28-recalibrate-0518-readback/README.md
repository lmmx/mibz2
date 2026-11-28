# recalibrate-0518 readback (28 Nov 2026)

Bundle `recalibrate-0518/run-20261128T100510Z` (origin/evidence f8b21e4, sha256 51fb951b…f2, verified),
recipe branch `bench/recalibrate-0518` at b4d25f64, decoder = 7be50513, i9-10900K, rustc 1.99.1.
Registration: `docs/journal/2026-11-28-recalibration-0518-registration.md`.

## What the recipe did wrong (harness, not decoder)
1. Its verdict step calls `bench/two_arm.py` with the multi-reference interface (`--ref-at`, `--judge-at`,
   `--pair`, `--sparse`, `--primary`, `--check-workers`, `--unpinned`, `--rounds-at`, `--alpha`). Master's
   (and the recipe branch's) two_arm.py has none of them and requires `--ref`, so both families printed
   nothing and the console says VOID. The registration's "test_two_arm.py 52/52 PASS" is master's scorer.
   The interface exists only on the `bench/open-*` branches (open-port, a1dc5aeb, 157/157 own checks).
2. `ab run ... --unpinned FILES`: master's `ab` has no `--unpinned`; it took it as a corpus path, giving
   336 failed phantom rows (`corpus = "--unpinned"`, os error 2). The real rows ran with no affinity
   (`cpus = "all"`; runner `Cpus_allowed_list 0-19`), i.e. unpinned in fact.
3. `bench/counters.py --events imc` does not exist (choices: default, l2). The DRAM diagnostic is absent.

## Re-score with the open-port scorer, the recipe's exact arguments
- `pinned-verdict.txt`: every whole-record, topology, spinner and worker check passes (648/648).
  Holm (α 0.025, 14 cells) rejects only occ-8-32 at 10 (wikidata 0.911 [0.906, 0.916]), which is
  SLOWER on text (1.064 [1.020, 1.110]): A FINDING, NOT ADOPTED. THE CHOICE at 10, 8, 6, 4: none,
  the shipped trims stay (7/28, 6/24, 5/20, eager 3/12).
- `unpinned-verdict-strict.txt`: VOID under the open-port scorer's metadata check, which wants
  `--unpinned`'s explicit CPU list and gets `"all"` (1152/1152 records). Not overridden.
- `unpinned-diagnostic.txt` (decides nothing; the only change is `cpus` rewritten "all" -> 0..19, true
  of the runner's affinity): Holm rejects no cell of 25; THE CHOICE at every count: none, the
  shipped trims stay. Nearest: occ-10-64 at 16 0.962 [0.931, 0.995] p 0.013, memory 1.25;
  occ-10-48 at 12 0.977 [0.956, 0.998] p 0.017, memory 1.16 (both over the 1.10 gate).

## Against the frozen MONODROMY prediction
- Held: unpinned 8/32 not beaten by 9/36, 9/27, 10/x; unpinned 6 and 4 lazy beat one-fewer and eager;
  pinned 8/6/4 shipped trims hold.
- Missed: pinned 10, "7/28 ties or narrowly beats 8/32". 8/32 is 8.9% faster on wikidata-x4 (and 6.4%
  slower on text). The predicted downward shift of k* did not appear on wikidata at pinned 10.
