# The plan: what we are doing, and why it would matter

*Written 26 November 2026, at Louis's request: "we need to write down durably what we are planning to do and why it would matter, so that when we are half way in doing it we dont get confused about what we are doing it for."*

**Read this first when picking up any thread below.** Every thread names:
- the question it answers;
- the number it could move;
- the files it would touch;
- the test that stops it.

If a piece of work can't say which row of this page it serves, it's a side-quest: write it down in that thread's folder and leave it.

## 0. The goal, in one paragraph

We want to find **where new mathematics would make the decoder faster**, rather than guessing. So we first *characterise* the decoder by measuring how fast it would be if each part were perfect (the ideal-parts matrix, `ideal-parts/`). Then we aim the mathematics (Antigravity, MONODROMY) at the parts with real room, at the worker counts that matter. Only after that do we engineer anything.

The order is **characterise → find the room → aim the mathematics → engineer**. Each step is a separate PR and a separate decision.

## 1. Where we stand (the matrix, in plain terms)

`ideal-parts/results/full/README.md` has the numbers. Read "ideal" as *this part costs nothing*: its output is handed over, pre-computed. The room is therefore a ceiling, not a forecast. A real idea captures some fraction of it.

The decoder has one scanner thread reading the compressed bits, k workers rebuilding blocks, and a reader putting the blocks back in order. Its parts:
- **S**, the scan: Huffman decoding, serial, one thread (`src/huff.rs`, the scanner loop in `src/parallel.rs`);
- **M**, MTF/RLE2 (`src/runs.rs`, `src/mtf.rs`);
- **C**, construction: building Open's structures from the BWT column (`src/open.rs` `Kernel::construct`, `src/open/endpoint.rs`). It has three sub-phases:
  - A6a, the first pass (runs, split, admission);
  - A6b, setup, the interval-exchange induction and the dictionary;
  - A6c, retained-edge wiring;
- **E**, emission: writing the text out of those structures (`src/open.rs` `emit*`, `src/open/outq.rs`);
- **R**, RLE1, the CRC and output (`src/rle.rs`, `src/crc.rs`);
- **D**, delivery: the in-order reader (`src/read.rs`, the reader in `src/parallel.rs`).

Walls are on wikidata ×4, with the scanner pinned and the clock pre-spun.

| workers | today's wall | best single part made free | the pair S + C made free | everything free (the floor) |
|---|---|---|---|---|
| 1 | 1,306 ms | C: 756 ms (−42%) | 751 ms | 391 ms, mostly the oracles' own cost |
| 6 | 243 ms | C: 183 ms (−25%) | 143 ms (−41%) | 94 ms |
| **8 (what ships)** | **195 ms** | C: 184 ms (−6%); S: 189 ms (−3%) | **125 ms (−36%)** | 112 ms |
| 10 | 196 ms | S: 180 ms (−8%) | 149 ms (−24%) | 142 ms |

**What this says:**
1. **At one or a few workers, construction (C) is the cost.** It is 80% of the explainable variation at 1 worker, and making it free takes 42% off.
2. **At 8 workers, which is what ships, no single part has much room**, because two parts take turns being the bottleneck. Speed up the scanner and construction binds; speed up construction and the scanner binds. Together they have 36%. This is the S × C interaction, the largest in the matrix.
3. **Emission (E) is third:** 16% at 1 worker, and more net of its oracle's copying.
4. **MTF, RLE1/CRC and delivery have little room of their own.** Delivery matters only once the scanner is free.
5. **Above 8 workers, memory bandwidth becomes a third bottleneck** (Antigravity's second pass, journal §90). So 8–9 workers is the target, not 10.
6. **The floor is not yet measured.** The 112 ms at 8 workers is mostly the oracles' own copying, not the decoder's. Thread 1 below measures it.

## 2. The threads, in priority order

| # | thread | question | why it matters (the number it could move) | touches | stop test | owner | state |
|---|---|---|---|---|---|---|---|
| 1 | **The floor, measured** (`ideal-parts/`, step 7) | What does the decoder cost when every part is free *and* the oracles cost nothing? | It decides whether the plumbing (queues, locks, copies) is worth any thought. It's 14–19 ms by arithmetic, but unmeasured. | `src/ideal/` only, off by default | Characterisation: it ends when the run is reported | Antigravity builds (`ideal-parts/antigravity/prompt-build-zero-copy.md`); the Pathfinder writes the recipe; the workstation runs it | prompt ready |
| 2 | **C: where its time goes, and the induction order** (`rauzy-census/`) | Of construction's time, how much is the induction steps? Would a cheaper induction order (1.812 → 1.279 steps per run measured, floor 1.220) cut *time*, and can the order be chosen while decoding? | C is the largest room below the knee and half of the 36% at 8 workers. A6b is 216 of 1,306 ms at 1 worker. If time follows steps, the known order saves about 60 ms there (5%). The census tells us whether it's more, or less. | `examples/rauzy.rs`, new census code; no `src/` changes | Stop if (steps' share of A6b) × (step cut) < 3% of the 1-worker wall, or if no rule decides the order from what's already decoded | Antigravity (`rauzy-census/prompt.md`) | prompt ready |
| 3 | **S + C together: pooling the scan** | Can the serial scan be dissolved into the worker pool (a cross-block parallel scan), so S and C stop taking turns? | The 36% at 8 workers needs *both*; Antigravity estimates 130–145 ms against 195 ms. | `src/parallel.rs`, `src/huff.rs` | Antigravity's kill test (IDEAL-PARTS-2 §5, proposal 2), after thread 1 | not started: waits on thread 1 | design only |
| 4 | **C's other sub-phases** (A5 + A6a fold; A6c wiring) | Can MTF/RLE2's output feed construction directly (no 127-cap interface), and can A6c's scattered writes go? | Antigravity puts this at 48–61% of C's ceiling together, more than the induction order alone. | `src/runs.rs`, `src/open.rs` | Antigravity's kill tests (IDEAL-PARTS-2 §5, proposals 1 and 3) | not started: waits on thread 2's time split | design only |
| 5 | **E: forward emission lanes**, jointly with A6c | Can emission run as several forward lanes from anchors, instead of a two-ended walk? | E's room, about 16% at 1 worker; it unlocks A6c. | `src/open.rs` `emit*`, `src/open/outq.rs` | IDEAL-PARTS-2 §5, proposal 3 | not started | design only |
| 6 | **Porting research software** (`leads/research-software.md`) | Would a fast port of network calculus, uiCA or surface_dynamics answer a question we can't answer now? | Proved upper bounds (network calculus); a cost model for kernel search (uiCA). | none: offline tools | Each has a 1–2 day test with the existing tools first; port only on PASS | Antigravity (the network-calculus and uiCA tests: part 2 of `rauzy-census/prompt.md`) | prompt ready |

**Not now:** production defaults, tunings, and fixes. A thread that suggests one writes it down in its folder and carries on (`ideal-parts/README.md`'s task box).

## 3. How a thread ends

Every thread ends written up: a result in its folder, and a journal section, a line in this table's state column, and the next decision. A negative result is a result. It removes a row from §2, and that's progress.
