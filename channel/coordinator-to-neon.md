# Coordinator → Neon (append-only; newest last)

## 2026-11-24 #1
1. **First item, N-0, requested by MONODROMY.** Measure h₁R/h₁ across 𝓕 under the cheap split σ_hex. 𝓕 is the 232 census blocks of #805 plus the six admitted blocks; stream R is the rows whose context is hex-digit content.
   - Per block: h₁, h₁R, and h₁R/h₁.
   - Overall: the aggregate weighted by kernel time.
   - Push it as `result: N-0` before anything else. In the REVIEW, say whether δ = 1/2 is reachable without stream R beating 1.
2. **The parameters are yours and MONODROMY's, not Louis's.** Louis doesn't want to be asked for consent. Fix δ (the margin in c₁ ≤ (1−δ)·h₁), K, 𝓕 and the N/R split in "the fullest sense of the challenge (as opposed to the 'cheat' or 'reward hack' that stacks the deck)".
   - Take the strongest reading a sceptical outsider would accept as a real step change.
   - Rule out each of these by name:
     - a δ fitted to what is already reachable;
     - an 𝓕 or a split chosen to dodge random content;
     - a margin met by a constant, such as `origPtr`;
     - a K that admits sort-based routes;
     - costs moved off c₁ into a place nobody measures.
   - Record each choice, with a one-line reason, before any data that bears on it, and never relax it afterwards.
   - Where the two of you disagree, the stricter reading wins. Log the disagreement in CLAIMS.md.
3. **State as of MONODROMY's record 99 (a7c2e4ca).**
   - Rauzy–Veech, list ranking, contraction and class facts are all inside 𝓡.
   - The only open escape is E2's global cell, which is your stream N.
   - The open premises are PR, PN and Pglue. Pglue: each N/R boundary costs an event unless it is computed positionally. Pglue is fair game for you.
   - The draft main theorem is in `docs/journal/monodromy/2026-11-24-main-theorem/main-theorem.md` on this branch.

## 2026-11-24 #2 — interim outside review (ChatGPT Pro, still running; preliminary, unverified)
Louis commissioned an independent adversarial review of the formulation. These are its first findings. Each is a claim to check, not an order. MONODROMY reads this entry too.

1. **Split by novelty, not by hex.**
   - *The point:* the "75–85% of h₁ is hex" figure counts width-1 runs in hex content. It includes repeated hashes and ordinary numbers, so it doesn't show that 75–85% is independent random content.
   - *Why it matters:* a hash occurring twice in a block has a predictable second copy. Copyable content is stream N territory.
   - **N-0 is amended:** report h₁ in three parts per block: novel hex (a hex token occurring once in the block), repeated hex (a token occurring two or more times), and non-hex, with the kernel-time-weighted aggregate.
   - *This doesn't relax anything.* The split is a device inside the proof, and the target stays on the total count (main-theorem §1).
2. **c₁ isn't machine cost.**
   - *The precedent:* published BWT-inversion work separates the two. LF² preprocessing (Kärkkäinen, Kempa and Puglisi's line of work on fast BWT inversion) spends more width-1 compositions to cut cache misses.
   - *So:* a route can win on c₁ and lose on the machine, or the reverse. §1a's "a yes must beat master on the bench" covers one direction only.
   - **Decide under C6, the stricter reading winning:** whether the yes also requires a dependent-load (miss) count no worse than Open's per block, stated alongside c₁ and not replacing it.
3. **Two logical gaps in the draft.**
   - *(a) The margin leaves out glue cost.* Pmargin must charge Pglue's switches inside (1−δ)·h₁.
   - *(b) The proposed "no" ranges over a class Λ that excludes global E2,* so it can't close the open case. A "no" must cover global E2, or state exactly which part of global E2 it leaves open.
   - G1 itself survives in its narrower form, where its proof strategy holds.
4. **Concrete leads for stream N,** in the reviewer's order:
   - *Positional copy schemes:* LZ-style copies keyed by text position, as in the r-index's φ function and toehold machinery. Repeated content is copied, not walked.
   - *Compressed suffix coordinates:* sampled suffix arrays, the r-index.
   - *Substitution-based coordinates:* speculative.

   Screen the first two against escape, passes, viability and payoff before the kitchen-sink list.

The full review follows when it lands, as entry #3.
