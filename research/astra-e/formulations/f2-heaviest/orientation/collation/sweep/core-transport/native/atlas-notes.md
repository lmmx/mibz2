# Forward atlas for a positive core

The atlas starts each occurrence at an original source in the nonempty interval U. An affine packet stores the original source ordinal, current physical rank, width and literal common prefix (up to eight bytes). Stable per-byte scatter constructs the original Ψ strips from capped L records. Consecutive caps with the same byte extend a single strip. At each permitted depth, every input-packet/strip intersection is charged once. Its Ψ image is clipped at the endpoints of U. If it would re-enter U, publish the current position and unchanged prefix. Otherwise append its byte and advance; publish survivors on reaching the cap. No packet is coalesced across a physical split in this accounting.

On the positive first-return arc W, precisely min(h, |W|−1) steps succeed, so even a one-byte return retains a nonempty backward core. Distinct source arcs have disjoint positive interior ranks. The reached ranks therefore form a disjoint union, stored as maximal half-open intervals of the forbidden set. If the primary lies at a successful depth, the same packet recovers the original source ordinal and the byte distance to it. None of the reached ranks lies in U, so the compressed root interval retains its full original mass.

Sort the terminal packets by physical rank and give each a starting ordinal in that order. For an ordinal v in the packet of owner o, its original source is o.base + v − o.v, and its source-leading prefix is o.prefix[..o.depth]. Adjacent terminal packets can share a physical target component while retaining separate owner records. For h=0 one identity owner covers U, the forbidden set is empty and there is no select table.

Atlas::bytes counts the allocated capacities of the two persistent section vectors and the owner vector. workspace_peak sums simultaneous temporary vector capacities for the select table, two active packet buffers, early/final leaves and the reached intervals. It excludes stack, caller input, allocator metadata and transient allocator copying. Adding the final persistent capacity to workspace_peak bounds their combined retained vector capacities during atlas preparation. Whole-process peak memory remains a separate measure.

The embedded native tests use independent stable row sorting, per-source explicit prefix walks and row-wise regrouping of actual translations and return tests. All binary columns through seven rows, all nonempty rank intervals, horizons 0/1/3/8 and all marks yielded 22,520 configurations and 145,448 verified marked atlases. The wide check covers singleton and whole-block intervals and mixed runs through caps 1, 8, 126 and 127. It verifies source and terminal ordinals, byte prefixes, all forbidden rows, intersection/band counts, and marked offsets. The complete test log reported two passing tests.

Run the atlas in isolation at the repository root:

```sh
python3 - <<'PY'
from pathlib import Path
s = Path('research/astra-e/formulations/f2-heaviest/orientation/collation/sweep')
section = (s / 'shift-compact/projection.rs').read_text().split('#[derive(Clone,Copy)]\nstruct Early')[0]
Path('/tmp/astra-atlas-contract.rs').write_text(section + '\n' + (s / 'core-transport/native/atlas.rs').read_text())
PY
rustc --edition=2021 --test -C opt-level=2 /tmp/astra-atlas-contract.rs -o /tmp/astra-atlas-contract
/tmp/astra-atlas-contract --nocapture
```

[Implementation and independent contract tests](atlas.rs)
