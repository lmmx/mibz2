# Production port evidence: master c4b69de3 → perf/compact30-default b4786f66

These are the raw Callgrind profiles behind PR #155 and channel CHECKPOINT 6641512290, archived
for replay (Guardian D015).

- **Input:** `wikidata-independent.bz2`, 11965711 bytes, sha256 `7f5c9a0044e5a30589bf343b9b31e698849b2acc69461a55c97b550193128b45`. Blocks 0 (54 bytes), 1
  (899,971 bytes) and 41 (900,000 bytes).
- **Driver:** `driver/`. It is the same source for both arms, and only the `mibz2` path
  dependency differs: master's worktree or the branch's. The release profile is fat LTO,
  `codegen-units = 1`, with line tables. The toolchain is Rust 1.97.
- **Command:** `valgrind --tool=callgrind --toggle-collect=c30cg::measured c30cg cg FILE BLOCK
  REPEATS`. REPEATS is 1 (`-r1`) or 2 (`-r2`, warm second decode).
- **Outputs:**
  - `cg/{base,branch}-b{0,1,41}-r{1,2}.{out,console,stdout}`;
  - `.stdout` holds the decoded length and the XOR of the CRCs. r1's CRCs match between the arms;
    r2's are 0 by construction, the same value XORed twice.
- **Mapping for block 1, r1** (`callgrind_annotate --inclusive=yes`):

| scope | master symbol | master | branch symbol | branch |
|---|---|---:|---|---:|
| whole | `decode_block` | 25,751,794 | `decode_block` | 29,530,459 |
| build | `bwt::build` | 5,429,062 | `c30::Ctor::build` | 4,369,204 |
| walk | `bwt::room_segments` | 15,426,892 | `c30::C30::room` | 20,207,700 |

  The PR's and the CHECKPOINT's whole figures are PROGRAM TOTALS, which run 16 above
  `decode_block` in each arm. The rest of `decode_block` is MTF, CRC and assembly: master
  4,895,840, branch 4,953,555.
- **Local timing** (shared VM, indicative only): `runab.py`, raw rounds in `times.tsv`.
- **Scope limit:** this covers one block. It is not a corpus-wide result.
