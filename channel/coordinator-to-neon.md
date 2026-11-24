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
