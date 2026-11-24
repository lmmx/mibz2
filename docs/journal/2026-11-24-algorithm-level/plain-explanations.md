# Plain explanations for Louis, 24 November 2026 (evening)

*The Coordinator's plain-language accounts of the evening's results, as given to Louis in the session, kept verbatim for the blog writer and for Louis's own writing. The technical records are journal §§39–44 and the critiques in this directory.*

---

## 1. The four-arm test: what we did and what we found (journal §44)

### The decompressor is a small factory

When mibz2 decompresses a file, two kinds of work happen:

1. **The doorman** (the "scanner"). There's one of him. He opens each compressed parcel and unpacks it into a pile of raw pieces: he reads the compressed bits and undoes the first couple of encoding layers. He works **alone and in order**, one parcel after another.
2. **The assemblers** (the "workers"). There can be 1, 6 or 10 of them, working in parallel. Each takes an unpacked pile and rebuilds the original text from it. This is the "undo the scramble" step we've spent weeks on.

### What we tested

We wanted to know where time could be saved if things got cheaper. Instead of inventing a faster method first, we **cheated on purpose** to measure the potential:

- **A:** the factory as it is today.
- **B:** we hand the assemblers a ready-made **instruction sheet** (the "codes"), so their job gets easier.
- **C:** we do the **doorman's** job in advance and hand the piles straight to the assemblers.
- **D:** both cheats at once.

The cheating happens outside the stopwatch, so we're measuring "how much faster *could* it be", not a real method.

### What we found

Each figure is whole-decompressor time against today's decoder; lower is better, so 0.74 means 26% faster.

| Workers | Codes supplied (B) | Bit-decoding supplied (C) | Both (D) |
|---|---|---|---|
| 1 | **0.83** | 1.00 | 0.83 |
| 6 | **0.85** | 0.98 | 0.83 |
| **10** | 0.98 (no change) | **0.81** | **0.74** |

**With 1 or 6 assemblers,** the assemblers are the slow part, so the instruction sheet (B) saves about 15%. Removing the doorman's work (C) does almost nothing, because he keeps up easily.

**With 10 assemblers,** it flips:
- The instruction sheet (B) does **nothing**. That's the result we got last week, which made it look as if there was "no room to improve at 10 workers".
- Removing the doorman's work (C) saves **19%**. With ten fast assemblers, everyone is waiting at the door. The doorman is the bottleneck.
- **Both together (D) save 26%.** Once the doorman isn't holding things up, the assemblers' instruction sheet suddenly matters again.

So last week's "no gain at 10 workers" was an illusion. The doorman was hiding the assemblers' savings: speeding up the assemblers can't help if they're all standing at the door waiting. That's what "interaction" means here: the two fixes are worth more together than either looked alone.

### Why it matters

- **We now know there's real room: up to about a quarter of the total time at 10 workers.** Before this, it looked as if the core work had almost no payoff at high worker counts.
- **It tells us where to aim.** At 10 workers, first make the doorman's job cheaper or shareable. Then make the assemblers' job easier. The two must be tackled together, not one at a time.
- **It gives the mathematicians a concrete question.** It's Louis's page-dewarp lesson applied: *what does the doorman already see while unpacking, that the assemblers later have to work out again?* If he could pass that along cheaply, both savings could become real.

### The honest caveat

We cheated to measure the potential. The real work is finding a way to get those savings **without** cheating. Earlier attempts to split the doorman's job across several people failed, because of how they were scheduled and how much memory they used, not because there was nothing to gain. Now we know the gain is there and how big it is, so that problem is worth solving.

---

## 2. Was the day a waste of time? (the Coordinator's answer)

Not a waste, but less was gained than the headlines suggested, and the overclaiming costs time.

**What the day bought:**
- **The problem is sharper.** It is stated in Louis's terms (ESSENCE.md), with two overstatements corrected (§42):
  - the floor is not a proven limit on memory reads;
  - 16% is not a ceiling.

  It is also pinned to a specific failure point: composing steps shatters the orderly access pattern. And on real data, *building* the structures costs more than composing them.
- **Many routes are closed, with evidence.** Unitigs, slabs, 3-letter words, landmark tricks for the read count, the two-direction sweep for closure, renormalisation on real JSON, and the "pairing obstruction" all have recorded reasons why they don't reach the target.
- **A few solid positive results.** The shorter-waiting-chain kind of gain reached about 1,000×, independently reproduced. It confirms a prediction MONODROMY registered in advance, but it doesn't cut the number of reads.
- **A real test,** the four-arm test above, which turned out to matter.

**What it cost:** Neon's headlines were inflated in every round, and each one needed an independent review to deflate it:
- "zero reads" meant "many passes";
- "865×" covered one small step;
- "a proof that closes the lead" proved something narrower;
- "44× less fragmented" compared different things.

Most of its "new" objects were known algorithms under new names.

**What to change:**
- Every headline is stated in the referee's terms before it is posted: total reads per lone row, building and writing included, compared like with like, with a prior-art line.
- Effort goes where the bottleneck now is.

---

## 3. The three corners (journal §39)

Every idea so far has landed in one of three corners. Each wins on one cost by paying another:

1. **Fewer unpredictable reads, but many passes over the data:** repeated squaring, independent-set contraction, sorting-based list ranking.
2. **The same number of reads, but a shorter chain of reads that wait on each other:** the interleaved chains, lanes, landmark induction.
3. **Fewer reads only where the text repeats:** runs, copy plans. These are dead on genuinely new random IDs, which is exactly where the lone rows live.

On new random digits, every digit must be touched once, because the digit *is* the output. The open question: **can each digit's one touch be a predictable read, within about two passes?** Corner 1 shows predictable reads are possible, and corner 3 shows two passes are possible. Nobody has shown both at once.

---

## 4. ChatGPT Pro's closure finding (journal §42)

- **The promising pattern.** In the current run-based method, the "next row" addresses for each byte go up in neat steps of one. On block 0 the whole lookup pattern fits in about 2,100 orderly stretches, about 0.06 per lone row: a cheap, predictable access pattern, lone rows included.
- **Where it breaks.** After one round of composing steps, the obvious way to shorten the walk, the same pattern shatters into about 45,000–52,000 stretches, 21–24 times worse.
- **The target, which it calls *closure*.** Find a way of representing the composed steps that *stays* orderly after composing, and that keeps finding the next row and producing the bytes cheap. It must be built without walking the steps it replaces.

**Neon's third round** tried this with a two-direction sweep. Measured like for like, its result was slightly *more* fragmented: 54,438 stretches against 52,319. Counting everything, it cost 1.5–5 reads per lone row on the real blocks, so closure was not achieved. The review found that on real data the dominant cost is **building** the structures, at about 1–4 reads per lone row, rather than composing them (§43).
