# The pooled scan: the prototype as built

*26 November 2026, the Pathfinder. This is the prototype the registration (`registration.md`, #926, with #770's cause, #930) asks for before its kill test. It sits behind the research feature `pooled-scan`, on bench/ideal-matrix-next (#909 with #921). With the feature off, or its mode unset, the decoder compiles and runs as shipped.*

## What it is

**The files:**
- `src/pooled_scan.rs`: the mode, the counters and the finder (#317's, ported);
- `src/parallel/pooled.rs`: the chain thread and the workers;
- `stream::Scanner::next_hinted`, the hook;
- `tests/pooled_scan.rs`;
- `examples/pooled_scan.rs`;
- the harness's `P` arm (`examples/character.rs`).

**How it works:**
- **The chain thread** (in the scanner's place) holds the input as the scanner does. It finds every block and stream-end magic in it as it arrives, and hands each block candidate's bytes (to the next magic, plus 16) to the workers in bit order. The blocks given a place and not yet read, plus the candidates not yet resolved, stay under the shipped window.
- **A worker** reads its candidate's item (the symbols, at level 9's bound) and reports where it ended and the CRC it stores. Then it decodes the block end to end, as a worker does now, and publishes it by its ordinal.
- **The chain** walks the stream with `Scanner::next_hinted`, a copy of `next` with one hint at each block position. There it takes a worker's read only where all of these hold:
  - the read began at exactly the chain's bit;
  - the stream is at level 9;
  - the read gave a block within its bytes;
  - the candidate's ordinal is the block's place.

  Anywhere else it reads the item itself and hands the block on, as the shipped scanner does.
- **The reader** is the shipped reader.

## Where it departs from the registration's §3, and why

1. **The chain runs on the chain thread, not in the reader.** The stream's framing (headers, ends, the combined CRC, every fallback read) needs the input, which the chain thread holds; the reader holds only decoded blocks. On the pooled path the chain thread reads no symbols. It waits only for each block's report (an end bit and a CRC), which a worker makes as soon as its read is done. So this is not #770's serialised scan (§2): the scan runs on all k workers at once.
2. **A candidate's bytes are copied** (the compressed input once over: 47.4 MB on wikidata ×4), not lent from the input's chunks. Lending them needs a rope of shared chunks; the copy is simpler for a prototype. Its cost on the i9 is estimated, not measured: about 5 ms of the chain thread, at memcpy speed.
3. **Below level 9, and after a mismatch, the shipped path.** A candidate is read at level 9's bound, since its stream's level isn't known until the chain reaches it. Level 9 is the only level where that read is the sequential decoder's, both in the original pointer's bound and in MTF's bound on rows. So in a stream below level 9 the chain reads every block itself.
   - The same holds once a candidate's ordinal stops matching its place (a false or missed magic): the rest of that decode runs as the shipped scanner does, which keeps the ideal oracles' block indices right.
   - #317's production obligation (decode at level 9's bound and check the rows against the stream's) is left for a production version.
4. **At 1 worker, the shipped path,** as the registration says: the mode applies only from 2 workers.

## Exactness (checked)

`tests/pooled_scan.rs`, 9 tests, all passing:

| test | what it checks |
|---|---|
| `the_corpora_at_levels_9_and_1` | **the pooled path itself.** Every corpus at levels 9 and 1, at 2, 4 and 8 workers: the bytes equal the input. At level 9, every block comes from a worker's read (accepted = blocks, 0 fallbacks, 0 rejected, 0 degraded); at level 1, the shipped path. |
| `streams_of_mixed_levels` | streams at levels 9, 9, 1 and 9, then an empty stream |
| `any_chunking` | input fed 1 byte to 8 MiB at a time, read 1 byte to 16 MiB at a time |
| `false_and_missed_magics` | the test hooks add a false candidate after every n-th block, or drop every m-th real one: the bytes still exact, and the chain's paths for both reached |
| `truncations_are_the_shipped_decoders` | cuts at every 1/53 of the file, at each block's first bytes and over the last 12 bytes: the output before the error, and the `Error { kind, block, bit }`, equal to the shipped parallel decoder's, and the kind equal to the sequential decoder's |
| `mutations_are_the_shipped_decoders` | 240 cases of 1–3 random bit flips, compared the same way |
| `headers_ends_and_trailing_data` | empty input, a header only, a bad level, not bz2, trailing bytes, an empty stream, one byte |
| `one_worker_is_the_shipped_path` | nothing handed out at 1 worker |
| `the_reader_stops_early` | the decoders cancel and drop without a hang |

**Also checked:**
- **The finder** (`pooled_scan::tests`) agrees with the bit-by-bit finder at every cut, with AVX2 and without. Under Miri it passes on the scalar path.
- **On wikidata ×4 at 2 and 4 workers** (`examples/pooled_scan.rs`, this 4-CPU container): 1,724 of 1,724 blocks came from workers' reads, with the shipped decoder's bytes.
- **The harness's arms** none, P, C, PC, SC and S run with every check pass byte-identical. P and PC are fully pooled in the timed decode and in the check pass.

**Two limits to note:**
- **The character study's flow identities don't hold in a P arm's trace.** In the pooled pipeline a block is read and decoded before the chain gives it a place, so the shipped pipeline's points (issued, enqueued, taken by a worker) don't come in its order. The ledgers' activities and waits still hold, which is what the registration's reported figures need: the workers' A2 and A3, and their idle share. So the kill test's recipe should exempt the P arms from the flow identities and say so.
- **Timing here means nothing.** On the 4-CPU container the scanner never binds. The kill test runs on the i9.
