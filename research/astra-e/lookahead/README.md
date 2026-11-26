# Load one residual edge before expanding the current word

This candidate changes the instruction schedule of opend's emitter. At loop entry the current row and its packed edge are available. Decode the current label, compute the successor, then load the next edge before writing the current label. At the end of a marked cycle the closing load of the original mark is valid, and its value is unused. Every emitted label still uses the correct row, including lookup in the separate word-code array on wide blocks. The constructor, density gate, repetition rule, dictionary, number of marked hops and required heap storage remain as in opend.

The same Rust 1.99.1 release binary contains the original and experimental loops. The original dictionary copy calls `memcpy` at 0x38172, masks the successor at 0x3818c, tests for the mark at 0x38193, and reads the edge at 0x381ac. The candidate masks the successor at 0x2c3b9 and reads its edge at 0x2c3c5, before its literal/word branch at 0x2c3c9. The word arm jumps to 0x2c344 and calls `memcpy` at 0x2c357. The new machine-level order could overlap part of the dependent load's delay with word expansion. It adds a live register value and can require a stack spill. These observations establish a changed schedule. Native performance requires the paired scorer.

The compiled candidate decoded all 209,700,196 output bytes of the pinned Wikidata corpus through its CRC-checking 1-worker entry. The executable's identity and disassembly of both loops are in `assembly.txt`. The full scorer gate separately checks bytes against libbzip2.

```sh
python3 research/astra-e/lookahead/prepare.py
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%h origin/master -- bench/moonshot
python3 bench/moonshot/moonshot.py score research/astra-e/lookahead/candidate.rs --with-opend
python3 research/astra-e/lookahead/assembly.py bench/moonshot/target/release/moonshot
```
