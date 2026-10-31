# P74-C full-decoder trial: the freeze record

**Status:** frozen before any timing. Every value below is [M], measured on the preparation
VM (Intel Xeon @ 2.80 GHz, 4 cores, no SMT, AVX2, rustc 1.90.0), unless labelled [D] or [E].
None of it is a timing.

**What fixes the run:** the protocol `2026-10-31-p74c-decoder-protocol.md`, `frozen.json`
and `bench/p74c-decoder-pin`.

## Corpora (frozen.json has every hash in full)

| # | Corpus | Role | Compressed SHA-256 | Decoded SHA-256 | Decoded bytes | Blocks | Bypassed | P74 to end | Handoffs |
|--:|---|---|---|---|--:|--:|--:|--:|--:|
| 1 | `wikidata-fitted` | secondary | `82f65ace0731962d…` | `3f075e30f4f8c734…` | 209,700,196 | 431 | 198 | 233 | 0 |
| 2 | `wikidata-heldout` | secondary | `7f5c9a0044e5a305…` | `cfda21b10a7f2898…` | 235,800,351 | 473 | 211 | 262 | 0 |
| 3 | `wikidata-20261026` | primary | `cf13f4394e4ea0a3…` | `47a2971ccc4a91e9…` | 207,000,624 | 434 | 204 | 229 | 1 |
| 4 | `random` | control | `10d444e6204289af…` | `00e14e00981a3970…` | 8,000,000 | 9 | 9 | 0 | 0 |
| 5 | `records` | control | `ff8b5f6ed3dd4445…` | `ff2de2d9a8112fa2…` | 9,600,000 | 11 | 11 | 0 | 0 |
| 6 | `mixed-independent` | control | `11639f3599a85391…` | `10bb5815c99a132b…` | 2,000,000 | 3 | 2 | 0 | 1 |
| 7 | `text` | control | `59236f2dff3afbd4…` | `53eefeed43ec0469…` | 12,278,087 | 13 | 0 | 12 | 1 |
| 8 | `split-q05` | adversarial | `080cb116c1b08c16…` | `3a685aa59cc92d14…` | 780,000 | 1 | 0 | 0 | 1 |
| 9 | `split-q25` | adversarial | `03c2b03861bdef0e…` | `3055b43ac3063635…` | 700,000 | 1 | 0 | 0 | 1 |
| 10 | `split-q50` | adversarial | `6de093665bdb104a…` | `55951d28b8e5597e…` | 600,000 | 1 | 0 | 0 | 1 |
| 11 | `split-q75` | adversarial | `90f2a6659ab93e41…` | `6088df249839df3c…` | 500,000 | 1 | 0 | 0 | 1 |
| 12 | `split-q90` | adversarial | `8d7e00b6c6b48f9b…` | `d78c6f9037e66b3e…` | 440,000 | 1 | 0 | 0 | 1 |
| 13 | `hug-eq-p32` | adversarial | `ced189b9752de23b…` | `d8c09c5b91175e2b…` | 380,000 | 1 | 0 | 1 | 0 |
| 14 | `hug-eq-p32-extra` | adversarial | `74c5805d157cf8c3…` | `8375e14b1144fde7…` | 380,003 | 1 | 0 | 0 | 1 |
| 15 | `hug-eq-p32-early` | adversarial | `9ee8e9c855465fe5…` | `69631166e402ca98…` | 380,003 | 1 | 0 | 0 | 1 |
| 16 | `hug-under-p33` | adversarial | `d7c2218c361946f8…` | `7fcf7ca508b640b4…` | 380,000 | 1 | 0 | 1 | 0 |
| 17 | `hug-over-p31` | adversarial | `df64a161cc9d8ef4…` | `4bd0d417efef1213…` | 380,000 | 1 | 0 | 0 | 1 |
| 18 | `rank-032` | adversarial | `26690df8f667b759…` | `7b57bc928922a048…` | 68,012 | 1 | 0 | 0 | 1 |
| 19 | `rank-063` | adversarial | `4d059055c22f985b…` | `12fe5613739aa51e…` | 68,012 | 1 | 0 | 0 | 1 |
| 20 | `rank-064` | adversarial | `173371a5440bbf75…` | `1d5b00ae06714dea…` | 68,012 | 1 | 0 | 0 | 1 |
| 21 | `rank-127` | adversarial | `ffb2fe1f43136878…` | `60d2ff3c7e224ca8…` | 68,012 | 1 | 0 | 0 | 1 |
| 22 | `rank-128` | adversarial | `38ca09d08afa7336…` | `6750513c0ae035d2…` | 68,012 | 1 | 0 | 0 | 1 |
| 23 | `rank-191` | adversarial | `0ecca8ea6922886a…` | `93b959f1987a362c…` | 68,012 | 1 | 0 | 0 | 1 |
| 24 | `rank-192` | adversarial | `2edfc79736fada86…` | `422b4b54b0c1b48e…` | 68,012 | 1 | 0 | 0 | 1 |
| 25 | `rank-224` | adversarial | `2ca82aa2d7f245b1…` | `c8338c7d19f315bc…` | 68,012 | 1 | 0 | 0 | 1 |
| 26 | `rank-252` | adversarial | `42800b168f416f09…` | `9261449ddb591a2f…` | 68,012 | 1 | 0 | 0 | 1 |
| 27 | `rank-255` | adversarial | `b394f05d26c50271…` | `9c398cf0b07b626e…` | 68,012 | 1 | 0 | 0 | 1 |
| 28 | `edge-l9-s16383` | adversarial | `eac7822fdf57edb6…` | `0a546e1a05ffab9a…` | 19,498 | 1 | 1 | 0 | 0 |
| 29 | `edge-l9-s16384` | adversarial | `1e9f3968a3023f17…` | `37f09e9f7083cdb0…` | 19,515 | 1 | 0 | 0 | 1 |
| 30 | `edge-l9-s16385` | adversarial | `2ed065e53a8c7e09…` | `9d3273443daae5f4…` | 19,504 | 1 | 0 | 0 | 1 |
| 31 | `edge-l1-s49999` | adversarial | `bb4a53a67276adb8…` | `080851eb1de0e455…` | 80,214 | 1 | 0 | 0 | 1 |
| 32 | `edge-l1-s50000` | adversarial | `7cc49ca5f6d2e32d…` | `c8b2030dec9ec29d…` | 80,208 | 1 | 1 | 0 | 0 |
| 33 | `edge-l9-s449999` | adversarial | `6bb04521d7c04858…` | `84f8c1521ea66952…` | 653,418 | 1 | 0 | 0 | 1 |
| 34 | `edge-l9-s450000` | adversarial | `df50de7f82e5d8ab…` | `efb933a60de5b845…` | 653,433 | 1 | 1 | 0 | 0 |

**Against the stage trial [M]:**
- `wikidata-fitted`, `wikidata-heldout`, `random`, `records` and `mixed-independent`
  reproduce its compressed hashes exactly.
- All 27 suite files reproduce its compressed and decoded hashes.
- `text` differs, as registered: it is built from this commit's own sources.
- Every decoded hash is `bzip2 -dc`'s.

## The different-dump corpus (`wikidata-20261026`)

**The order of events [M]:**
1. The positions were committed with the protocol before any byte was fetched
   (`different-dump/spec.json`, commit `29c478e`, pushed to `origin/research/p74c-decoder`).
2. The dump's size was confirmed by an HTTP HEAD: `Content-Length: 103565615653`,
   `Last-Modified: Tue, 27 Oct 2026 16:38:44 GMT`, ETag `"6ae0d394-181cfde225"`.
3. Each window was fetched behind the range guard (206, exact `Content-Range`, exact
   length).
4. The windows were built with the stage trial's `--windows`, unchanged.
5. The hashes were registered in `different-dump/windows.json`.

Nothing was tuned on the corpus.

| Window | Offset | SHA-256 |
|---|--:|---|
| d00 | 4,315,233,985 | `d18e5fdb8c62d0f88b675792400a724e63ed9acdb066418865232e895d6a8d35` |
| d01 | 12,945,701,956 | `1c5f95e476fa75c20bc2c851d5859ebf872a789d54a4133a7318a737fb33f341` |
| d02 | 21,576,169,927 | `fdeeece0d748018d3c6a6181f5a002c6d691cdd37ca91fffbafaaa4cc362f311` |
| d03 | 30,206,637,898 | `316b4ff073dd3b6df716052f49d7a4e49553a24b23f0973ffe1c917ecd6551f4` |
| d04 | 38,837,105,869 | `db63631fcfdf3931b10db29f8ba7b561526f65918a89f163fa4cdbd257f2e5b8` |
| d05 | 47,467,573,840 | `28e768be80ed3c586f02a012a5b40d0b7e154670ffdb4c9356b98a0c8cfcceaf` |
| d06 | 56,098,041,812 | `d42380ff1c24ac0e1fc6b2caed5030760a986b3d1ca7bfe93c0fd844ed02197c` |
| d07 | 64,728,509,783 | `7b01785f0bc84bb81a020b890ac11b93a0cab18bfc73d29ae281cf1ed05da234` |
| d08 | 73,358,977,754 | `bfd50cf980b2ee8cefc6ffd65cb33d572aa895ba72b4259ec98e7b957cda8a88` |
| d09 | 81,989,445,725 | `744d5b2c376b0bb5836c148a68b2a6bdfdc139a40bdf06114c8105223a57bbcc` |
| d10 | 90,619,913,696 | `4ff35590fa356a09cca858790957daa00fb2e817f707faf3f509f2b6cae562e4` |
| d11 | 99,250,381,667 | `3f4173b86106519998798324e5dde1c30cc995bdd2969f795c9af1480b710fd6` |

**The corpus:**
- compressed SHA-256 `cf13f4394e4ea0a38a0e9e3bc5271b2dce717c8f98fe3a0b8a5186c84216b8fe` (11,953,225 bytes);
- decoded SHA-256 `47a2971ccc4a91e958c5c8797f45ff7a50ae0d7e7cfac31152c8018d994b7a81` (207,000,624 bytes);
- 434 blocks, 25,232,396 symbols.

**Selector audit** (frozen policy, independent spelling, `p74c_decoder --audit`):
- 204 blocks bypassed;
- 229 P74 to the block's end;
- **1 handoff:** block 384 (135,665 symbols, 87,567 explicit, 2,012 of rank ≥ 32),
  switching at consumed symbol 1,963.

This is the first Wikidata handoff seen. The stage trial's two Wikidata corpora had none,
and still have none here (fitted 198/233/0, same-dump held-out 211/262/0). The runner's
actual-handoff audit (`audit_manifest_corpora`: the kernel's own handoff hook, both CP
values) checks it against the oracle on every block of every corpus.

## Arm identity (protocol §3a): source lines at the freeze commit

**Arm-dependent code**, all of it in `examples/p74c_decoder.rs`:
- `:326-330`: `arm_kernels`, the only arm-dependent code. Arm 1 is
  `base.with_mtf(MtfKernel::P74c)`; arms 0 and 2 are `base` (`Kernels::detect()`).
- `:516`: every arm makes the same call, `decode_once(&z, w, arm_kernels(arm), bytes)`.
- `:346-353`: `decode_once`, the same `StreamDecoder::planned(Plan::Workers(w), 2 * w, k)`,
  feeder and reader.

**The decoder** (`src/parallel.rs`):
- `:125`: `Plan::Workers(n)` → `with_inverse::<Segmented>`.
- `:168` and `:176`: the same workers and scanner.
- `:415` and `:443`: each worker calls `decode_block(&symbols, k, …)`.

**The one branch on the choice** (`src/lib.rs`):
- `:349-364`: `decode_block`'s `match (cooperative, k.mtf)`. `Shipped` reaches
  `mtf::mtf_checkpoints` (`:352`) and `mtf::mtf` (`:360`); `P74c` reaches
  `mtf::p74_c::mtf_checkpoints` (`:356`) and `mtf::p74_c::mtf` (`:364`).
- `:117`, `:139`, `:145` and `:159`: the field, `scalar()`'s and `detect()`'s `Shipped`,
  and `with_mtf`.

**Recorded per block, checked per child:**
- `mtf::selections`; `check_selections` (`examples/p74c_decoder.rs:414`);
- `tests/p74c_selection.rs`.

## Affinity proof (protocol §4a)

**Where it is taken:**
- the spawn snapshot (`examples/p74c_decoder.rs:357`), every decoder thread;
- the feeder's own record (`:363`);
- the end snapshot, untimed.

**What is asserted:**
- the child asserts w + 3 threads;
- the parent asserts every thread's allowed set is within the allocation (`:631-632`);
- the parent asserts its own affinity is the allocation.

**What `--check` saw [M]:** at every w = 1..4, on all 34 corpora, both arms, 4/5/6/7
threads at spawn.

## The frozen kernel and the structural audit [M]

**The text is unchanged from 8b6caf1:**
- `combined`'s text from its doc comment to the end of the file;
- `SHIFT`, `SWAPPED`, `BLEND`, and the `front!`, `moved!` and `fill!` macros.

What changed: the public entries (no `Arm`; production passes `start_shipped = false`),
the selection counters at the entries, and the imports.

`bench/p74c_decoder_asm_audit.py` on the integrated `examples/p74c_decoder` (rustc
1.90.0):

| Variant | Combined, here | Combined at the pin | Free reference | Gates G1-G4 |
|---|---|---|---|---|
| plain (the trial's) | 38 instr, 6 jcc, 0 RMW slots | 38, 6, 0 | 37, 6, 0 | all pass |
| checkpoints (not exercised by the trial) | 37, 6, **2** | 39, 6, 0 | 38, 6, 2 | all pass |

- **Plain:** the counts are identical to the pin's.
- **Checkpoints:** the integrated binary allocates registers differently.
  - The output index n and the output pointer go through stack slots on the low-rank
    cycle (`[rsp+0x28]`, `[rsp+0x70]`): Gate 1 Free's shape, which is within the
    registered gate (≤ Free's 2), not the pin's 0.
  - [D] The likely cause is the changed calling context (one caller, constant
    `start_shipped`). The kernel's source is unchanged, and per protocol §2 nothing is
    tuned.
  - The checkpoint kernel runs only under Cooperative/teams plans, which this trial
    doesn't measure.

## Correctness [M]

- **`cargo +1.90.0 test --locked --release`, the full suite, once:** 84 passed, 1 ignored
  (`audit_manifest_corpora`, which needs a file list; the runner runs it).
  - The frozen P74-C unit tests: 10, plus the ignored audit.
  - The ported Gate 1 tests: 8.
  - The public P74-C entries: 5.
  - The whole decoder under both settings: 6.
  - The selection test: 1.
- **`p74c_decoder --check`, all 34 corpora:** A and B at w = 1, 2, 3, 4 through the
  trial's decode, plus sequentially. Every output hash equals `bzip2 -dc`'s, and every
  block's selection is the arm's kernel's.
- **`bzip2 -t`:** passes on all 34.

