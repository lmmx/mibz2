[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-206/blog/index.md>)

# Huffman decoding in mibz2: keeping the bit buffer in a register

The Huffman decoder wrote its bit buffer to memory and read it back at every lookup. Kept in a register, the stage ran 13% faster on Intel and 5% on AMD.

5 November 2026 · 13 min read · [Research result](<#8-what-the-benchmark-measured>)

A bzip2 block stores its contents as Huffman-coded symbols: short bit patterns for common symbols and longer ones for rare ones, packed end to end with nothing between them. Before anything else can happen to a block, a decoder has to read those symbols back, one code at a time. It can't start on a code until it knows where the one before it ended. In mibz2 one thread, the scanner, does this for every block in the file, while the workers do the rest. On CI's runners it is about a tenth of the time a one-thread decode takes.

This post is about a change to that loop which leaves the Huffman codes, the decoding tables and the bits read exactly as they were. What changed is where the decoder keeps three numbers: its place in the input, the 64 bits it holds, and how many of them are still unread. They had been written to memory and read back at every table lookup. Kept in registers, the scanner stage ran 12.9% faster on an Intel Xeon 8370C runner and 5.0% faster on an AMD EPYC 9V74 runner. A one-thread decode of the Wikidata sample ran 2.7% faster on the Intel runner, and no change was detected on the AMD one. The Intel cut fell short of the 15–25% predicted for it. The AMD cut was smaller, within the wider range predicted for AMD, whose cores may already hide part of the cost that was removed.

[Section 1](<#1-where-huffman-decoding-sits-in-the-decoder>) places the stage in the decoder, and [section 2](<#2-the-bit-reader-and-its-table>) explains the bit reader and its table. Sections [3](<#3-what-the-census-found>) and [4](<#4-why-the-compiler-kept-the-buffer-in-memory>) are how the stored buffer was found and why the compiler put it there, and [section 5](<#5-the-change-the-state-as-a-value>) is the change. Sections [6](<#6-what-was-predicted-before-any-timing>) to [8](<#8-what-the-benchmark-measured>) give what was predicted, how correctness was checked, and what the benchmark measured, run by run.

## 1\. Where Huffman decoding sits in the decoder

A bzip2 block goes through several stages on its way back to bytes. mibz2 splits them between two kinds of thread, as the [post on block dispatch](<https://lmmx.github.io/mibz2/pr-preview/pr-206/blog/2026-11-01-block-dispatch/index.md>) describes:

Where Huffman decoding sits in mibz2

Measured: the base build's one-thread stage costs on the Intel Xeon 8370C runner, in cycles per byte of output at the runner's reported clock, 11.28 c/B in all (the second benchmark run of section 8). At one thread the stages run one after another on one CPU; at four, the scanner's thread runs beside the workers.

- **The scanner** reads each block's header and decodes its symbols. The header says which byte values occur, gives the block's Huffman tables, and says which table each stretch of symbols uses. This part is sequential: a block ends with an end-of-block symbol, so the next block's start isn't known until every symbol before it has been decoded.
- **A worker** takes a block's symbols and does the rest. It undoes the move-to-front coding and its run-length coding (MTF and RLE2), builds the inverse BWT's navigation table and walks it, then expands bzip2's first run-length coding (RLE1) and checks the block's CRC.

At one thread every stage runs on the same CPU, one after another, so the scanner's share of the time is its share of the decode: on the Intel runner, 1.19 of 11.28 cycles per output byte, about 10.5%. At four threads it runs alongside the workers. All four of the runner's CPUs are then busy, so the scanner's CPU time still costs wall time, at about a quarter of its own rate. That comes from [the binding census](<https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-binding-census.md>), our survey of what limits the decoder on CI's runners.

## 2\. The bit reader and its table

The scanner reads the stream through a small bit reader, [`Bits`](<https://github.com/lmmx/mibz2/blob/66a9dedd/src/bits.rs#L6-L14>). Its state is three numbers:

| Field | Type | What it holds |
| --- | --- | --- |
| `pos` | `usize` | the next byte of input to load |
| `buf` | `u64` | the stream's next bits, the first of them in the most significant bit |
| `live` | `u32` | how many of `buf`'s top bits are valid |

Three operations change it:

- **refill** loads the eight bytes at `pos` and ORs them into `buf` just below the live bits. It then advances `pos` by as many whole bytes as fit, so that at least 56 bits are live afterwards. Near the end of the input it loads a byte at a time instead.
- **peek** reads the top few bits of `buf` without consuming them.
- **skip** consumes \\(n\\) bits: it shifts `buf` left by \\(n\\) and takes \\(n\\) from `live`.

The bit buffer, and one table lookup

Conceptual, drawn just after a refill. The divisions in the bar are bytes. A refill leaves 56 to 63 bits live; four lookups use at most 40 of them.

Decoding a code uses a lookup table, as most fast Huffman decoders do. A block has between two and six Huffman tables, and each group of 50 symbols names the table it was coded with. The last group stops at the end-of-block symbol, so it may be shorter. For each table, mibz2 builds a [lookup table of 1,024 entries](<https://github.com/lmmx/mibz2/blob/264ae5bb/src/block.rs#L45-L72>), one 32-bit entry for each possible value of the next ten bits. Ten bits hold most codes but not all, because bzip2's codes can be up to 20 bits long. So an entry says one of four things:

- the ten bits start with one whole code: here are its symbol and its length;
- they start with two whole codes, both inside the ten bits: here are both symbols and their combined length;
- the code is longer than ten bits: decode it the slow way, a bit at a time, as libbzip2 does;
- no valid code starts here: the block is corrupt.

On the Wikidata sample, about 83% of lookups decode a pair ([census](<https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-binding-census/serial-stages.md>)). The fast loop refills, makes four lookups, then refills again. It runs while at least eight symbols remain in the group, because four lookups can produce eight, and the group's last few symbols go through a slower loop.

One lookup is four steps:

1. take the top ten bits of `buf`;
2. load the table entry they index;
3. take the code's length from the entry;
4. shift `buf` left by that length.

Step 1 of the next lookup needs the result of step 4 of this one. However many instructions the processor could run at once, it can't start a lookup before the last one has finished its shift. So the time a lookup takes is the latency of this chain: the time each step waits for the one before it. Any step added to the chain is paid on every lookup.

## 3\. What the census found

The [binding census](<https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-binding-census.md>) asked what limits today's decoder on CI's runners. One of its lines of inquiry took the stages that run in sequence, and read the scanner's machine code. In the four-lookup loop, each lookup ended by storing `buf` to memory, and the next began by loading it back.

A store followed by a load from the same address costs few instructions, and it doesn't wait for the cache: the processor hands the stored value straight to the load. That hand-over is called store-to-load forwarding. It still takes time, about 4–5 cycles on Intel's Ice Lake cores such as the 8370C's, and here it sat on the lookup chain. The census estimated a lookup at about 13–15 cycles with the reload, against about 8–9 with `buf` in a register. These figures are estimates, from the listing, the instruction counts and the measured cycles, not counts of events: CI's runners expose no hardware performance counters.

One lookup's chain, before and after

Estimated, not measured: the census's reading of the listing for Ice Lake. The measured saving is in section 8.

Across the four runner models the census measured, the scanner took 6.9–10.3% of the four-thread wall time (7–10% in its summary) and 11–14% of the one-thread time. It estimated that removing the forward would recover 0.5–4.1% of the four-thread wall, and 1.7–5.6% at one thread, and called it "The cheapest and most certain mechanism on this front."

## 4\. Why the compiler kept the buffer in memory

Nothing in the Rust asked for a store. The loop calls the reader's methods, which are small and marked `#[inline(always)]`, so their bodies are compiled into the loop. Two calls are not inlined: the slow path for codes longer than ten bits, and the slow refill near the end of the input. Both are marked `#[cold]`, and on master before the change both took the reader by reference, as `&mut Bits` ([`decode_entry_slow(e, br)`](<https://github.com/lmmx/mibz2/blob/66a9dedd/src/block.rs#L343>), [`refill_slow(&mut self)`](<https://github.com/lmmx/mibz2/blob/66a9dedd/src/bits.rs#L49-L50>)).

Passing a reference to a function that isn't inlined lets the reader's address escape. As far as the compiler (LLVM) can tell, that function might read or change the reader, so the reader's fields have to be in memory, at that address, whenever such a call can happen. The calls are rare, but one can happen at any lookup. A second constraint follows from the first. The loop writes each decoded symbol through a pointer into the output, `out[n]`, and once the reader's address has escaped, LLVM must assume such a write might change it too. So it loads `buf` again after the writes ([registration](<https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-huffman-register.md>)).

This is one of the four steps, from master before the change, as the [archived listing](<https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-huffman-register/asm-excerpt.txt>) gives it (the bench binary, rustc 1.99.0, in Intel syntax: destination first). `r15` holds the reader's address:

```text
ab240  mov    rsi,QWORD PTR [r15+0x18]                              bits.rs:74
ab244  mov    rax,rsi                                               bits.rs:74
ab247  shr    rax,0x36                                              bits.rs:74
ab24b  mov    edx,DWORD PTR [r9+rax*4]                              block.rs:326
ab24f  mov    eax,edx                                               block.rs:331
ab251  shr    eax,0xb                                               block.rs:331
ab254  and    eax,0x3                                               block.rs:331
ab257  je     ab450 <mibz2::block::read_item+0x12c0>                block.rs:331
ab25d  mov    ecx,edx                                               block.rs:332
ab25f  and    ecx,0x1f                                              block.rs:332
ab262  shl    rsi,cl                                                bits.rs:81
ab265  mov    QWORD PTR [r15+0x18],rsi                              bits.rs:81
ab269  sub    DWORD PTR [r15+0x20],ecx                              bits.rs:82
```

| Instructions | What they do |
| --- | --- |
| `mov rsi,QWORD PTR [r15+0x18]` | load `buf` back from the reader in memory: the forward |
| `mov`, `shr rax,0x36` | its top ten bits (a shift right by 54), the index |
| `mov edx,DWORD PTR [r9+rax*4]` | load the table entry |
| `mov`, `shr`, `and`, `je` | the entry's symbol count: none means the slow path |
| `mov`, `and ecx,0x1f` | the bits to consume, the entry's low five bits |
| `shl rsi,cl` | consume them |
| `mov QWORD PTR [r15+0x18],rsi` | store `buf` back to the reader |
| `sub DWORD PTR [r15+0x20],ecx` | take them from `live`, in memory |

Over a four-lookup iteration the registration counts five stores and three reloads of `buf`, and four read-modify-writes of `live` in memory: about one forward on the chain per lookup.

## 5\. The change: the state as a value

The change, [\#204](<https://github.com/lmmx/mibz2/pull/204>), moves the three fields into a small struct that can be copied, `Reg`, and has the loop hold one in a local variable ([`src/bits.rs`](<https://github.com/lmmx/mibz2/blob/264ae5bb/src/bits.rs#L6-L24>)):

```rust
pub struct Bits<'a> {
    data: &'a [u8],
    r: Reg,
}

#[derive(Clone, Copy)]
pub(crate) struct Reg {
    pos: usize,
    buf: u64,
    live: u32,
}
```

- **The logic moves unchanged.** Refill, peek and skip become methods of `Reg`.
- **The cold paths take the state by value** and return the state after: `decode_entry_slow(&self, e: u32, r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg)`, and `refill_slow(mut self, data: &[u8]) -> Reg`. No call receives the state's address, so the compiler can keep its fields in registers, and pass them into a cold call and back like any other arguments.
- **The loop works on a copy.** `read_item` takes the state out of the reader before the symbol loop, with `let (data, mut r) = br.parts();`, and writes it back once, with `br.set(r)`, on every way out of the loop.
- **The loop is rotated.** The refill moves from the top of the four-lookup loop to its foot. With the refill at the top, the state was carried from one pass of the loop to the next in memory, which cost a store and a reload every refill. The refills happen at the same points as before:

```rust
if left >= 2 * STEPS {
    r.refill(data);
    loop {
        for _ in 0..STEPS {
            step!(true);
        }
        if left < 2 * STEPS {
            break;
        }
        r.refill(data);
    }
}
```

`Bits`'s public methods keep their signatures, and the block header is parsed as before. This is the same step in the new build, from the same listing:

```text
ab582  shr    rcx,0x36                                              bits.rs:71
ab586  mov    rdx,QWORD PTR [rsp+0x28]                              bits.rs:71
ab58b  mov    edx,DWORD PTR [rdx+rcx*4]                             block.rs:335
ab58e  mov    r9d,edx                                               block.rs:340
ab591  shr    r9d,0xb                                               block.rs:340
ab595  and    r9d,0x3                                               block.rs:340
ab599  je     ab853 <mibz2::block::read_item+0x1643>                block.rs:340
ab59f  mov    ecx,edx                                               block.rs:341
ab5a1  and    ecx,0x1f                                              block.rs:341
ab5a4  shl    rdi,cl                                                bits.rs:78
```

`buf` is in `rdi`, and the step ends at `shl rdi,cl`, with no store. The loop's only stores now are the symbol writes. It still loads values that don't change during the loop from the stack, such as the table's address (`[rsp+0x28]`). Nothing in the loop stores to them, so those loads wait on no forward, and their addresses don't depend on the chain.

## 6\. What was predicted, before any timing

Before any CI timing, the change was registered: its mechanism, the measurements that need no timing, a prediction and a decision rule, committed before any CI timing of the branch ([registration](<https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-huffman-register.md>)).

Instruction counts from callgrind, which counts every instruction a program executes, on the bench corpus:

| Instructions | Before | After | Change |
| --- | ---: | ---: | ---: |
| Per four-lookup iteration | 153.8 | 144.8 | −5.9% |
| The whole scanner | 746.2 million | 727.4 million | −2.5% |
| Per byte of output | 3.56 | 3.47 | −2.5% |

The forwards also leave the lookup chain: four in each iteration, about 0.94 a lookup on average. The registration then priced the saving at about \\(F\\) cycles a lookup, where \\(F\\) is the store-to-load forwarding latency on the runner, plus the 2.5% of the scanner's instructions:

| Figure | Predicted |
| --- | --- |
| Scanner stage, one thread, Intel Ice Lake (8370C), \\(F\\) about 4–5 | 15–25% faster |
| Scanner stage, one thread, AMD and Intel Golden Cove (8573C) | 0–25% faster |
| Wikidata wall, one thread, Intel | 1.5–3.5% faster |
| Wikidata wall, four threads | 0.5–2.5% faster |
| MTF, table build, walk, RLE1, CRC | unchanged: their code is untouched |

The AMD range is wide because some cores can rename a store and a later load of the same address: they recognise the pair and hand the value over without the forward's latency. Whether CI's AMD runners do that for this code wasn't known.

The decision rule had four parts:

- **Correctness, required:** CI green, and the same bytes on every corpus in every bench run.
- **Benefit, required:** in at least one run, the Wikidata one-thread wall or CPU faster beyond noise, or the one-thread scanner stage faster beyond noise with the whole decoder's one-thread point estimate not slower; and the same figure not slower in the other run.
- **Refutation:** a scanner cut under 10% on an Intel runner, or any other stage moving beyond noise, would refute the mechanism.
- **No harm, required:** a wall figure flagged slower is settled by one more run on the same head. Flagged again, the rule fails. At most three runs, with each run's CPU named.

## 7\. Is it still the same decoder?

The change has to give exactly the same symbols, and fail in exactly the same way on corrupt input. Before timing:

- **A differential** ran master's `bits.rs` and `block.rs` against the new ones, outside the repository, on 52 files and 164,076 cases of intact and mutated blocks. It compared 9.15 billion symbols, every case was identical, and all 11 error kinds that can be reached were exercised.
- **A scanner-only driver** gives the same 24,979,299 symbols, with the same hash.
- **New tests:**

  - the reader's state stepped as the loop steps it, compared with the input's bits;
  - 4,000 fully written blocks, with real or corrupt codes up to 20 bits, bit flips and cuts. Each is compared with a reference that decodes one lookup at a time: the symbols, the error, the end bit and whether it read past the input;
  - 300 corrupted level-1 streams, compared with libbzip2.

In both benchmark runs, every corpus decoded to the same bytes with both builds, and CI passed.

## 8\. What the benchmark measured

The CI benchmark decodes every corpus with the base build and the PR's build in alternating order, ten paired rounds a run. It runs on a GitHub-hosted runner with four virtual CPUs: two physical cores, each running two hardware threads (SMT siblings). The CPU model varies from run to run. It ran twice on \#204's final head, 53cc38db: once on an AMD EPYC 9V74 (Zen 4) and once on an Intel Xeon 8370C (Ice Lake). Each change below is the PR's build over the base: the geometric mean of the rounds' paired ratios, with its 95% interval in percent beneath it. Negative is faster. "Wall" is the elapsed time to decode the Wikidata sample.

| Figure | Predicted | Intel 8370C | AMD 9V74 |
| --- | --- | ---: | ---: |
| Scanner, one thread | Intel: −15% to −25%<br>AMD: 0 to −25% | −12.9%<br>(−13.9, −11.9) | −5.0%<br>(−6.3, −3.6) |
| Scanner, cycles a byte |  | 1.19 → 1.04 | 1.18 → 1.12 |
| Wall, one thread | Intel: −1.5% to −3.5% | −2.7%<br>(−4.3, −1.1) | \+1.0%<br>(−0.7, +2.6) |
| Wall, two threads | none | −2.5%<br>(−4.6, −0.3) | −1.3%<br>(−4.0, +1.5) |
| Wall, four threads | −0.5% to −2.5% | −1.3%<br>(−5.6, +3.3) | −0.3%<br>(−3.2, +2.6) |
| Other stages | no change | none moved | none moved |

**On Intel, the stage cut missed its prediction.** It was 12.9%, against 15–25%. That is above the 10% the rule set as the refutation, so a saving of the predicted kind was there, but smaller than the model priced it. By our arithmetic, which is derived rather than measured, the stage saved 0.15 cycles per output byte at the runner's reported clock. At the census's 0.0646 lookups per byte, that is about 2.3 cycles a lookup, against the 4–5 the model priced. With no hardware counters on CI, the benchmark can't say which part of the model was wrong.

**On AMD, the cut was smaller still, as the mechanism allowed.** It was 5.0%, about 0.9 cycles a lookup by the same arithmetic, and inside the 0–25% registered for a core that may rename the forward. That fits renaming, but no counter shows it.

**One-thread decoding got faster on Intel, and no change was detected on AMD.** On Intel the whole one-thread decode was 2.7% faster, inside the predicted 1.5–3.5%. The scanner's own saving explains about half of that point estimate: 0.15 of 11.28 cycles per byte, 1.3%. The rest is within the noise of the other stages, none of which moved beyond noise. On AMD the one-thread decode was +1.0% (−0.7%, +2.6%), with no change detected either way.

**One run's interval understates how much a figure moves between runs.** Each interval above describes the spread between one run's ten rounds. A/A controls, benchmark runs on merges that left the decoder alone, show the one-thread figure also moving by about 0.7% from run to run. With that spread added, the Intel one-thread result is −2.7% (−4.8%, −0.6%): still faster.

**At four threads, no change was detected on either runner.** The prediction, 0.5–2.5%, is smaller than the benchmark can resolve at four threads, where the intervals here are 6–9 points wide. At four threads the scanner thread's CPU time fell from 0.15 to 0.12 seconds on Intel and from 0.09 to 0.08 on AMD, out of about 1.6 and 1.1 seconds in all. No figure was flagged slower in either run.

Under its rule, the change passed. Correctness held. The Intel run showed the benefit, with the one-thread wall and CPU both faster beyond noise, and the AMD run's same figure was not slower. The Intel stage cut was above the refutation threshold, no other stage moved, and nothing was flagged as a harm.

PR \#204 met its registered decision rule and was merged on 5 November 2026, as 264ae5bb.

## 9\. What it shows

The Huffman codes, the tables and the bits consumed are the same as before, and the scanner runs only 2.5% fewer instructions. What changed is the chain each lookup waits on. On Intel, 2.5% fewer instructions gave a 12.9% shorter stage, because what was removed, the forward, sat on that chain. Elsewhere in mibz2 the opposite has happened: in the [post on memory-level parallelism](<https://lmmx.github.io/mibz2/pr-preview/pr-206/blog/2026-11-02-memory-level-parallelism/#4-cutting-instructions-helped-the-walk-but-not-four-worker-throughput>), removing a third of the walk's instructions made no difference to four workers. An instruction count predicts time where the instructions sit on the critical path.

The pattern is easy to recreate. A loop carries a value from one iteration to the next, and also passes a reference to it to a function that isn't inlined, such as a cold path that takes `&mut self`. The value's address escapes, so it lives in memory, and every iteration pays a store and a reload. Passing the state by value and taking it back keeps it in registers. The census found two other store-to-load chains, in MTF's list and in the table build's byte cursors. Those are different: there the data lives in memory because the algorithm keeps it there. The bit buffer had no reason to be in memory, only a reference that let it escape.

A decode that takes more symbols a lookup is being prototyped separately, and would combine with this change. The census estimated that a wider table, now, would take at most about another 10% off the scanner, which is under 1% of the four-thread wall.

---

**Sources:**

- the change: [\#204](<https://github.com/lmmx/mibz2/pull/204>), and its [registration](<https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-huffman-register.md>), committed before any CI timing;
- the listings: [`asm-excerpt.txt`](<https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-huffman-register/asm-excerpt.txt>), and the instruction counts per address in [`cg/`](<https://github.com/lmmx/mibz2/tree/264ae5bb/docs/journal/2026-11-05-huffman-register/cg>);
- the code before: [`src/bits.rs`](<https://github.com/lmmx/mibz2/blob/66a9dedd/src/bits.rs>) and [`src/block.rs`](<https://github.com/lmmx/mibz2/blob/66a9dedd/src/block.rs>) at 66a9dedd; after: [`src/bits.rs`](<https://github.com/lmmx/mibz2/blob/264ae5bb/src/bits.rs>) and [`src/block.rs`](<https://github.com/lmmx/mibz2/blob/264ae5bb/src/block.rs>) at 264ae5bb;
- the scanner's share and the estimates: [the binding census](<https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-binding-census.md>) and its [serial stages](<https://github.com/lmmx/mibz2/blob/264ae5bb/docs/journal/2026-11-05-binding-census/serial-stages.md>);
- the two benchmark runs: [run 42057920798](<https://github.com/lmmx/mibz2/actions/runs/42057920798>), attempts 1 (AMD EPYC 9V74) and 2 (Intel Xeon 8370C), reported on [\#204](<https://github.com/lmmx/mibz2/pull/204>).
