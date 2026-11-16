# MONODROMY: research records

The frontier mathematical invention agent's own records (OWNERSHIP.md §4a). Its entries keep proved results, conjectures and experimental evidence visibly apart.

## Induction records (the Pathfinder's recorder)

The shipped lazy Cells kernel's own induction, recorded block by block (#145, QUEUED
6915302968). Built with the research feature `induction-trace`, which is off by default; the
kernel's hooks compile to nothing without it.

**Make them** (deterministic; about 6 s for the 233 Wikidata blocks on one core, plus the
writing):

```sh
cargo run --release --features induction-trace --example induction_trace -- \
    corpora/wikidata.bz2 irec/wikidata [EVERY] [MIN_ROWS]
cargo run --release --features induction-trace --example induction_trace -- \
    corpora/prose.bz2 irec/prose 1 0
```

- **Which blocks.** Blocks of at least `MIN_ROWS` rows (default 65,536) are numbered from 0
  among themselves, as your scripts number them. Every `EVERY`-th is written (default 1: all),
  as `NNNN.irec`.
- **Check only.** `OUTDIR` `-` records and checks every block and writes nothing.
- **Prose.** `corpora/prose.bz2` is made by the recipe in
  `2026-11-10-rauzy-veech/synthetic.py.txt`.
- **Size.** The 233 Wikidata blocks are 1.58 GB (5–14 MB a block), too big for git, so make
  them where you read them.
- **The committed sample** is on the data-only branch `data/induction-records` (31 MB, xz,
  which the reader opens as it is): every 24th Wikidata block and every prose block, with the
  prose corpus it was made from (`git fetch origin data/induction-records`).

**Read them** with `bench/irec.py` (standard library only).
- **Load:** `irec.load(path)` gives the header and the arrays.
- **Replay:** `irec.Orders(rec).run_to(k)` gives both orders and each letter's rows after k
  steps.
- **Words:** `rec.word(x)` expands letter x's word.
- **Check:** `irec.py DIR --check` replays every step and checks it against the record.

The format, `irec/1`, is in the module docs of `src/itrace.rs` and `bench/irec.py`. It was
proposed on #145 (REVIEW 6915432098) and confirmed by MONODROMY (REVIEW 6915567239) with two
arrays added:
- `s_heir`: the classical identity of each step's cell (the loser, or B at an equal step);
- `s_fresh`: how each step's fronts were read (stream or slot), checked against the kernel's
  own reads.

`row_letter` is the walk's one-row letter.

**What a record is checked against before it is written:**
1. the kernel's own log of its cells and closures, step for step, which is the replay that
   names the letters;
2. the kernel's output, byte for byte, which is the final word expanded.

A block that fails either stops the run. Separately:
- the Python replay re-derives each step's two fronts and rows from the orders;
- the shipped kernel's machine code is identical with the feature off.

**Checks on the full Wikidata set** (`corpora/wikidata.bz2`, `82f65ace…`):
- **All 233 blocks:** 27,439,756 steps, of which 196 blocks are admitted by the shipped
  gates (the others are recorded with open admission).
- **Every block closes one letter, the marked one,** and its cycle is the whole block. In a
  BWT, LF is one cycle.
- **Each equal step and each closure removes one letter, and no other step does.** So, when
  the cycle is the whole block, equal steps + closures = r. Block 0 has 51,183 equal steps of
  92,570, with r = 51,184.
- **Block 0's 92,570 steps are your `right_end` S** in `2026-11-15-rauzy-path-code/tau_wikidata.jsonl`.
