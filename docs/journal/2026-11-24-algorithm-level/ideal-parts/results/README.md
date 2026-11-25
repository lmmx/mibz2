# Results

Empty until the first run. Each run gets a section here: its bundle (evidence commit and SHA-256), its analysis program (`analysis/`), its tables, and its reading against `../registration.md`.

## The analysis program

`analysis/ideal_parts.py` (slice A, S x E), written before any data to `../registration.md`. It
reads bench-ideal-parts' bundle (the plan and runs.jsonl), and gives:
- the effects, Jain's sign table on log wall;
- the allocation of variation;
- the ceiling table: each part's room, its room with the other ideal, its share of the floor;
- the floor;
- the labels (ROOM, NO ROOM, MASKED, UNRESOLVED; INTERACTS, ADDITIVE, UNRESOLVED);
- the VOID and INCOMPLETE checks;
- reported beside them: the oracle's memory and handover, the clock split, the unpinned link and
  peak RSS.

`analysis/test_ideal_parts.py` checks it on synthetic runs with known effects. The recipe's
report phase runs it, and it writes `ideal-parts.tsv` into the bundle.

## Slice A's preflight (not evidence)

`just bench-ideal-parts dry` at 0cb6afb1, on a 4-CPU container (no PMU, not the i9): two blocks, a 2 s pre-warm. It is archived as `dry/run-20261125T100951Z.tar.gz` (sha256 255ea22c…5b6a88d0).
- Every phase from checks to run passed.
- 144 decodes. Every check pass was byte-identical to its record pass, and every trace's identities held.
- The pin held in 32 of 32 pinned traces.
- The removed parts read 0.00% of their real busy time at every count.
- The report was VOID on the machine alone. The clock split is empty here (no PMU).

**Since the preflight:** the handover counts the supplied blocks alone. It had been summing the walk's own time in the arms with emission real.
