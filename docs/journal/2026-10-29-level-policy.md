# A level-aware team cap: does the policy leave cores idle on small-block files? (pre-registered)

- **Measurement first:** no decoder change. A policy change, if this indicates one, gets its own PR and A/B.
- **Prompted by** the knee journal's footprint run (`2026-10-29-libsais-knee.md`, §2–3). At bzip2 -3 (1.2 MB tables), one core a block kept scaling to 8 blocks, where full blocks turn at 4.

## The question

`Plan::for_threads` caps teams of 2 at round(0.7 × L3 / 3.6 MB): 4 teams on the i9's 20 MB L3, at any thread count from 8 up. 3.6 MB is a full 900 K-row block's table.
- **A stream at a lower level has smaller blocks:** level n holds n × 100 K rows, so its tables are about n × 0.4 MB.
- **The cap then leaves cores idle,** and the footprint run says those cores could be used.
- **The level is in the stream's header** (`BZh1`–`BZh9`), known before any block is decoded.

**On the native corpus the cap is right** (the release run, `2026-10-29-cooperative-decoder.md`):
- At 10 threads, 5 teams of 2 (18 MB of tables) ran at 0.912× the policy's 4 teams.
- At 20 threads, 10 teams ran at 0.66×.

## What is run

`just bench-level-policy`: the release-gate harness (`bench/coop_decoder.py`, `examples/ab.rs`), as the release ran it.
- **The decoder:** the whole decoder, file to hashed output, every CRC checked.
- **Rounds:** 20, interleaved, unpinned, back to back.
- **Arms:** `mibz2-workers` (T workers), `mibz2-coop2` (T/2 teams of 2) and `mibz2-auto` (today's policy), at T = 4, 6, 8, 10 and 20.
- **Three corpora, all the bench corpus's content:**
  - bzip2 -3 (1.2 MB tables);
  - bzip2 -6 (2.4 MB);
  - the native corpus, as this session's control.

**The comparisons that matter:**
- **At 10 threads:** `mibz2-coop2` (5 teams of 2) against `mibz2-auto` (4 teams of 2), paired by round.
- **At the default, 20 threads:** the best arm against `mibz2-auto`.
- **At -3:** whether `mibz2-workers` at 8–10 beats both team arms.

## Predictions (fixed now)

**From capacity:**
- **At -6:** 5 teams hold 12 MB of tables, below the 14.4 MB that 4 full ones held at the knee. So 5 teams of 2 should beat 4, up to 5/4 if each team keeps its pace.
- **At -3:** 5 teams hold 6 MB, so the same applies.
- **One core a block at -3** reached 8 blocks (9.6 MB) before turning, in the inversion alone. In the full decoder, `mibz2-workers` at 8 or 10 may compete with teams.
- **On native:** 5 teams lose to 4 again (about 0.91×), as in the release run.

**What can take the gain:**
- the second limit at 10 cores (knee journal §3, reading 5);
- the decoder's stages outside the inversion (scanner, Huffman, MTF), which don't shrink with the table;
- a -3 file's 3× as many blocks, with their per-block overheads.

## The decision rule (fixed now)

**A level-aware cap is proposed** if, at -3 or -6, either `mibz2-coop2` at 10 threads, or the best arm at 20, beats `mibz2-auto` at 20 under all of these conditions:
- a paired geometric mean ≥ 1.05, with its 95% lower bound > 1.00;
- peak RSS ≤ 1.15× `mibz2-auto`'s;
- CPU-seconds ≤ 1.5× its;
- the same output hash on every run.

**The candidate rule, written now so the data can't choose it:**
- The cap becomes round(0.7 × L3 / (level × 0.4 MB)), still at most half the physical cores. On the i9 that is 4 teams at levels 8–9, 5 at 7 and below.
- The level is taken from the first stream's header. A file whose later streams have larger blocks keeps the first stream's plan, a cost the PR would measure.
- **If workers win at -3 instead,** the candidate becomes workers below some level, chosen from this run and confirmed by the PR's own A/B.

**Otherwise** today's cap stands for every level, and the result is recorded here.

**A policy PR would be merged only after its own A/B,** which also checks two things:
- the native corpus is unchanged (same plan, so paired ≈ 1.00);
- the 4-core pass is unchanged (workers below 6 physical cores, as today).

## The command (the i9)

`just bench-level-policy`: about 10–15 minutes.
- It writes one `coop-decoder-*.md`, `.json` and `.jsonl` a corpus. Upload those, and the console.
