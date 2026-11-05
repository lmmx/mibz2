# 2026-11-05: The symbol loop keeps the bit reader's state in registers: merged (#204)

Registered before any CI timing of this branch. The decision rule may be amended only in a
commit made before the first bench comment on this PR, and each amendment says what changed
and why.

## The change

The census found the Huffman bit buffer kept in memory, with a store-forward on every lookup.
Its pool was 0.5–4.1% of 4-thread wall.

**The cause, found in the listing.** `read_item` passes `&mut Bits` to cold calls
(`Huff::decode_entry_slow`, and `Bits::refill_slow` through `refill`). The reader's address
escapes, so LLVM keeps its state in memory. LLVM must also assume that the `out[n]` symbol
stores may alias it, so it reloads `buf` at every step.

**The change:**
- **`src/bits.rs`:**
  - `Bits` becomes `{ data, r: Reg }`. `Reg` is a `Copy` `{ pos, buf, live }` holding the
    refill, peek and skip logic, moved verbatim.
  - `Reg::refill_slow` is `#[cold] #[inline(never)]` and takes and returns the state by value.
  - Every public `Bits` method keeps its signature: there is no public API change.
- **`src/block.rs`:**
  - `Huff::decode` takes `(&mut Reg, &[u8])`. `decode_entry_slow` takes `Reg` by value and
    returns it, with the same logic.
  - `read_item`'s symbol loop works on a local `Reg` and writes it back once, on every exit
    path.
  - The 4-lookup loop is rotated so `buf` is not spilled across the back edge: the same
    refills at the same points.
- **Header parsing** is unchanged.

**New tests:**
- **`bits`:** a local state stepped as the loop steps it, compared with the slice's bits.
- **`block`:** 4000 fully written blocks, with real or corrupt codes up to 20 bits, flips and
  cuts. Each is compared with a lookup-at-a-time reference: symbols, error kind, end bit and
  `overran`.
- **`tests/decode.rs`:** 300 corrupted level-1 streams against libbzip2.

## Measured, untimed

All OBSERVED, with files in `2026-11-05-huffman-register/`.

- **The listing** (release, fat LTO, rustc 1.99.0):
  - **Master:** each 4-lookup iteration does 5 stores and 3 reloads of `buf` through `r15`,
    plus 4 read-modify-writes of `live`.
  - **The branch:** the 145-instruction loop has no stores except the `u16` symbol writes.
    `buf` stays in a register.
- **Instructions,** callgrind on the bench corpus:
  - 153.8 → 144.8 Ir per 4-lookup iteration (−5.9%);
  - for the scanner as a whole, 746.2M → 727.4M Ir (−2.5%), 3.56 → 3.47 Ir per decoded byte.
- **The forwards** leave the lookup chain: 4 a lookup iteration, about 0.94 a lookup.
- **Equivalence:**
  - an out-of-repo differential of master's `bits.rs` and `block.rs` against the branch, over
    52 files and 164,076 cases of intact and mutated blocks: 9.15 billion symbols compared,
    every case identical, all 11 reachable error kinds exercised;
  - the scanner-only driver gives the same 24,979,299 symbols and hash.
- **Tests:** `cargo test` passes in full. fmt and clippy are clean (clippy on rustc 1.97.0).

## The prediction, recorded before any run

The saving is about F cycles a lookup, where F is the store-forward latency on the runner, plus
2.5% of scanner Ir.

- **Where the access is not memory-renamed** (Intel Ice Lake 8370C, F about 4–5): the
  1-thread scanner stage is 15–25% faster, and `wikidata` 1-thread wall 1.5–3.5% faster.
- **Where the core may rename it** (AMD, and Golden Cove such as the 8573C): F is uncertain.
  The scanner cut is 0–25%.
- **4-thread wall:** −0.5% to −2.5%. 4-thread CPU falls by about the scanner CPU saved.
- **The other stages** (MTF, build, walk, RLE1, CRC) do not move: their code is untouched.

## The decision rule

- **Correctness (required):** CI green, and the same bytes on every corpus in every bench run.
- **Benefit (required):** in at least one run, `wikidata` 1-thread wall or CPU is faster
  beyond noise, or the bench's 1-thread scanner stage is faster beyond noise while the
  whole-decoder 1-thread point estimate is not slower. The same figure must not be slower in
  the other run.
- **What refutes the mechanism:**
  - a scanner-stage cut under 10% on an Intel runner (8370C or 8573C);
  - any other stage moving beyond noise, which would be an artefact.
- **No harm (required):** a wall row flagged slower is a harm signal, settled by one more run
  on the same head. Flagged again, it is a harm and the rule fails. At most three runs, naming
  each run's CPU.
- **Precondition:** rustc 1.99.1's listing of `read_item` in the CI build must not reintroduce
  the `buf` spill. The scanner stage row reads it indirectly.

## Result: the rule passed; merged as 264ae5bb

The registration above is as committed before any timing. Both bench runs are in
`2026-11-05-huffman-register/`.

| run | CPU | scanner stage | `wikidata` 1t wall | 1t CPU | 4t wall |
|---|---|--:|--:|--:|--:|
| 42057920798, attempt 1 | EPYC 9V74 | −5.0% (−6.3, −3.6) **faster** | +1.0% (−0.7, +2.6) | +1.0% (−0.7, +2.6) | not significant |
| 42057920798, attempt 2 | Xeon 8370C | −12.9% (−13.9, −11.9) **faster** | **−2.7% (−4.3, −1.1) faster** | **−2.7% (−4.3, −1.1) faster** | −1.3% (−5.6, +3.3) |

- **Benefit:** met in run 2. The same figure was not flagged slower in run 1.
- **Refuters, not hit:**
  - the Intel scanner cut is 12.9%, against the 10% floor;
  - no other stage moved beyond noise in either run.
- **No harm:** no wall row was flagged slower.
- **Correctness:** the same bytes on every corpus; CI green.
- **Against the prediction:**
  - The Intel scanner cut is below the predicted 15–25%.
  - 1-thread wall is within the predicted −1.5% to −3.5%.
  - On AMD the scanner gain is smaller and the whole decoder doesn't move. That is consistent
    with store-to-load forwarding being memory-renamed there (INFERRED).
- **Precision** (OBSERVED, harness-only A/A merges, k = 9): at 1 thread, run-to-run spread is
  about 0.7%, beyond one run's own interval. With it added, run 2's 1-thread result is about
  −2.7% (−4.8, −0.6).
- **Independent review:** no findings. A differential of master's `bits.rs` and `block.rs`
  against the head's compared `read_item` field by field, including `bit_pos` and `overran`:
  - 6,000 corpus trials and 100,000 synthetic blocks, truncated and bit-flipped, every error
    kind reached: 0 mismatches;
  - two injected write-back bugs were both caught.

