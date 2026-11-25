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

## The full matrix's analysis program

`analysis/ideal_matrix.py` (all six parts, SECMRD), written before any data to `../registration.md` with clarifications 1–8. Slice A's `ideal_parts.py` stays as it ran. It reads `just bench-ideal-matrix`'s bundle and gives:
- **the effects:** Jain's sign table on the 32 fraction arms of I = +SECMRD alone, the 12 added arms never fitted. That is the 6 main effects and the 15 two-factor interactions, each with its 95% t-interval over the blocks, plus the allocation of variation (the three-factor pairs together, and the residual);
- **the ceiling table, read from the measured arms:** each part's room alone, its room with each other part ideal, and its share of the floor;
- **the floor;**
- **the labels,** as slice A's. MASKED names the part Q that masks P;
- **the VOID and INCOMPLETE checks;**
- **reported beside them:**
  - each oracle's handover, from its own ledger entry in the traces of the part alone ideal;
  - the room net of handover;
  - the stores and peak RSS;
  - the clock split;
  - the unpinned link.

`analysis/test_ideal_matrix.py` checks it on synthetic runs with known effects: 28 checks, run in CI. Among them:
- the fraction's 31 effect columns are orthogonal;
- q_SE recovers the 2²'s log(0.85)/4.

**How the binary implements clarifications 5–7:**
- **Each handover is its own ledger entry:**
  - S: hand_S, the scan oracle's symbols handed to the scanner's issue;
  - E: hand_E, the supplied codes' delivery and consumption;
  - C: hand_C, the built structures copied in;
  - M: hand_M, the run records or column copied in, nested in A5;
  - R: hand_R, the final bytes copied in, nested in A9.
- **D has no handover.** Its oracle discards the blocks, and its work is the reader's copy to the caller, copy_out (within A10).
- **The removed-part VOID reads only each part's own activities:** S A2+A3, E A7, C A6a+A6b+A6c, M A5, R A9, D copy_out.
- **The handover in the net figure is taken per thread:** the summed busy time over the scanner's one thread for S, or over the k workers for C, M, R and E. (wall − handover)/all real is then a geometric mean over the blocks.
- **All six stores are recorded in every arm,** so every arm holds about 505 MiB on wikidata ×4.
- **The check pass reruns the arm's own modes,** with emission's check mode where E is ideal and delivery's check mode always. It compares the size and SHA-256 of the blocks in order against the record pass's. Every oracle appears in arms without R, whose replay would otherwise mask what lies upstream of it.
- **A timed decode fails in the harness** if an ideal part was not wholly replayed (S, M and R every block, C every open block, D every byte discarded and none copied) or a real part was replayed.
