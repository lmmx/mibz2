# Working backwards from the text: the Hutter Prize, an algorithm zoo, and where Open's cost lives

*20 November 2026. Notes from Louis's discussion (with ChatGPT, relayed to the Coordinator), and one
measurement the Coordinator ran to test its central idea. A pending lead for the research line
(FRONTIER.md), not an assignment.*

## 1. The discussion

**The Hutter Prize is the mirror image of mibz2.**
- **What it rewards:** the smallest self-extracting description of enwik9, a 1 GB Wikipedia
  extract. Decoding may take tens of hours on one core.
- **What we reward:** the least computation to reconstruct from a description that is already
  fixed (bzip2).
- **Louis's objection:** the prize's tolerance of slow decompression "seems backward". A short
  description whose use costs two days isn't obviously understanding.
- **The distinction it exposes:** *information complexity is not computational complexity*.
  Description length (Kolmogorov) differs from the work of producing the object from the
  description (time-bounded complexity; Bennett's logical depth).
- **Our own evidence of the same gap:** Neon's BSO (cycle 5) shrank the successor map's
  representation and lost 2–3× in time.
- **The useful framing:** a Pareto frontier over (size, decode time, peak memory). Hutter sits at
  one extreme; mibz2 at another, with size fixed.

**STARLIT (Margaritov, 2021 Hutter award).**
- **The move:** it reordered Wikipedia's articles by similarity (Doc2Vec vectors, a
  travelling-salesman-style tour), so an unchanged cmix predictor saw related material close
  together. The decoder undoes the order.
- **The lesson:** the win changed what the machine was given, not the machine itself.
- **Why it doesn't transfer directly:** bzip2's encoder has already fixed the order.
- **What does carry over:** the BWT has already grouped the text by right context, which is a
  reordering the encoder paid for. Does it expose an organisation the decoder doesn't yet exploit?

**Later winners (cmix descendants, a small transformer in the latest record).**
- **What they show:** complementary models are blended, each informative in its own contexts.
- **Note:** the leaderboard figures were relayed from ChatGPT and have not been checked here.

**Louis's synthesis: an algorithm zoo, and working backwards.**
1. **A zoo of block- or region-specific primitives.**
   - **The idea:** a repertoire of reconstruction methods rather than one universal frontier.
     Open and Cells already form a small zoo, with Open on and Cells as the general fallback.
   - **The condition each member must meet:** C_recognise + C_chosen < C_baseline, paid for over
     all blocks, the fallbacks included.
   - **Headroom first:** an oracle that picks the best member per block, with hindsight, bounds
     the gain before any selector is built (as with LIMIT, #657).
2. **Working backwards.**
   - **The direction:** start from the structure we believe the text has (JSON, schema-level
     likeness, templates) and ask what traces of it survive, cheaply readable, before inversion.
   - **The aim:** not to overfit to Wikidata or cheat, but to learn what kind of algorithm could
     win from structural knowledge.
3. **The opposite of prediction is certification.**
   - **Why:** cmix predicts the next byte, but an exact decoder can't output a likely byte.
   - **What we want instead:** cheap, exact structural certificates that determine many output
     bytes at once.
   - **Neon's width-1 theorem binds only part of this:** online interval certificates. Richer
     certificates are open.

## 2. The measurement: where Open's cost lives, in the text's own terms

**Question.** Working backwards needs to know which structures of the *text* own the expensive
rows of the *column*. If JSON skeletons are already cheap, the skeleton idea has nothing left to
win.

**Method** (`costmap.py`, run on the six admitted blocks, about a minute in total).
1. **Invert each block,** mapping every row to the text position of the byte it holds.
2. **Classify each text byte by regex:**
   - GUIDs;
   - 40-hex hashes (sha1);
   - entity ids (Q/P/L…);
   - other numbers;
   - JSON keys;
   - other letters (text);
   - the rest (structure).
3. **Attribute the rows** in thin runs (width < 16, a proxy for Open's thin side at LIMIT=16) and
   in width-1 runs to those classes.

**Result** (share of rows, %):

| block | thin rows / n | GUID: thin rows | GUID: width-1 rows | GUID: all bytes | sha1: thin | JSON keys: thin | JSON keys: all bytes |
|---|---|---|---|---|---|---|---|
| 0 | 9.8 | 31.0 | 66.8 | 3.0 | 22.3 | 1.0 | 37.6 |
| 2 | 12.2 | 22.7 | 61.8 | 2.8 | 22.0 | 1.8 | 37.0 |
| 85 | 17.2 | 27.1 | 64.9 | 4.7 | 25.0 | 2.4 | 37.8 |
| 143 | 21.5 | 23.2 | 53.6 | 5.0 | 9.1 | 3.1 | 34.2 |
| 207 | 15.0 | 30.8 | 56.9 | 4.6 | 7.7 | 4.0 | 33.4 |
| 210 | 22.4 | 37.9 | 67.1 | 8.5 | 6.2 | 2.6 | 35.9 |

*Measured (`costmap.py`). The classes are regex approximations; "thin" is a run-width proxy,
not Open's exact retained set.*

**What it says.**
- **The JSON skeleton is already free.**
  - JSON keys are 33–38% of the bytes but 1–4% of the thin rows.
  - Repeated keys and delimiters sit in wide runs, which Open (and Cells before it) already
    collapses.
  - So "reconstruct the shared skeleton collectively" is roughly what the current kernel does.
- **The cost lives in random identifiers.**
  - Statement GUIDs are 3–8.5% of the bytes but 54–67% of the width-1 rows.
  - GUIDs plus sha1 hashes are 31–53% of the thin rows.
- **Two different kinds of random string:**
  - **GUIDs are unique.** Their content is about 122 random bits with no copy source, so every
    hex digit after the first few sits in a width-1 run. Neon's width-1 theorem applies to them
    directly. No certificate or copy method can source them from earlier output.
  - **sha1 reference hashes repeat.** One reference is shared by many statements, so their rows
    are thin but rarely width-1 (1.5–8% of the width-1 rows). These are copies: once one
    occurrence is decoded, the rest are spelled. This is Astra C's territory (output-sourced
    copies), concentrated in a class we can now name.
- **This is not Wikidata-specific overfitting.** Random identifiers (UUIDs, hashes, keys) inside
  repetitive structured text are the generic shape of JSON logs, APIs and dumps.

**The questions it opens for the research line.**
1. **The GUID wall.**
   - **Its nature:** the width-1 rows of unique random tokens behave like a random permutation
     segment. Each dependent step is a likely cache miss, and the content carries nothing to
     exploit.
   - **The question:** is there an operation on *random* segments that isn't a walk? For
     example: their contexts become unique after about log₁₆ n ≈ 5 characters, so locally they are
     a sorted list of random strings. Or is this the true floor, which would be a scoped
     obstruction worth proving?
2. **The copy class.**
   - **The question:** how much of the thin side is repeated random strings (hashes, repeated
     ids), and what does an exact copy mechanism cost when charged for its discovery?
   - **The constraint:** the discovery has to come from what the decoder already holds, not from
     the decoded output.
3. **A zoo, measured.**
   - **The idea:** split thin rows by class and ask whether one primitive per class, each
     recognised cheaply, beats Open's single treatment.
   - **First step:** the oracle headroom, before any recogniser is built.

## 3. Sources

- The Hutter Prize rules and leaderboard: prize.hutter1.net and hutter1.net (relayed; not
  re-checked here).
- STARLIT: Margaritov's repository and write-up (relayed).
- Bennett, *Logical depth and physical complexity* (1988).
- Kempa and Kociumaka, *Resolution of the Burrows–Wheeler Transform conjecture* (2020), r = O(z log² n). This explains why repetitive structure yields few, wide runs.
