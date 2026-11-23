[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-797/blog/index.md>)

# Correctness is uniform, cost is not

Every route in mibz2's bzip2 decoder is exact on every block, but cost varies from block to block and so we hunt for speed in many places.

22 November 2026 · 12 min read · [Research in progress](<#7-what-is-proved-measured-and-open>)

mibz2 is a bzip2 decoder in Rust, and most of its time goes on inverting the Burrows–Wheeler transform (BWT) of each block. Real compressed data offers that inversion no single way in. A block of a Wikidata dump holds a JSON skeleton, copies of text already decoded, and random identifiers that no local structure predicts. The proportions change from block to block. So the decoder is fast through several exact constructions, each cheap on a different structure, and its mathematics is as varied as its data.

**The takeaways:**

1. **Correctness is uniform and cost is not.** A correctness proof needs one route; speed on real data needs many. Every route mibz2 ships returns exactly the reference output on every valid block. What differs between them is cost, and cost depends on the block and the machine.
2. **Each fast route bets on a structure.** The bet is a cost hypothesis: when the block has this structure, this construction is cheaper. Finding structures worth betting on is the research.
3. **When the data refuses a hypothesis, there are three responses:** weaken it; certify it more cheaply; or change the representation so it is no longer needed. Refusals are where the next construction comes from.
4. **Global changes of representation are rare.** We have had one. Below it we pursue local constructions on purpose, and they may yet reveal a general one. We do not know how complete the family is.
5. **Scope:** one decoder, bzip2's inverse BWT, with Wikidata as the main corpus. One workstation judges it: an i9-10900K with a 20 MiB L3. Nothing here is proved in Lean yet, and the literary analogy in §5 is not evidence.

## 1\. One proof, two fates

[lean-zip](<https://github.com/kim-em/lean-zip>) is a DEFLATE implementation in Lean whose round trip, `inflate (deflateRaw data level) = .ok data`, rests on "more than 1,100 theorems". Its README calls the result a ratchet: "the obligation `inflate (deflate x) = x` either still holds or the build goes red". So agents can optimise it hard without breaking it. It also says where it stands: its decoder trails "the C-grade decoders (miniz\_oxide by \~1.45×, libdeflate by \~2.5×)". The theorem guarantees every optimisation is correct. It does not say which will be fast.

mibz2 has the same gap. Two recent changes are both byte-exact on every block. The i9 judged them very differently, at 10 threads on four copies of the Wikidata benchmark:

Two exact changes on the i9 at 10 threads: CPU time against wall time

Measured on the i9-10900K, Wikidata ×4, 16 paired rounds each, 95% intervals. The construction stack (runs open-r5/run-20261122T051443Z and, on a later master, run-20261122T103311Z) saved CPU but not wall time. The block reader (parscan-c37/run-20261122T082138Z, decision 2; and block-reader-on-wormhole/run-20261122T093349Z, on top of \#739) saved wall time.

The construction stack cut 5% of CPU time in the inverse BWT, and its 10-thread wall time did not move. The block reader shortened the serial stage that feeds the workers and saved less CPU. It cut wall time by 7% alone, and by 10% on top of \#739. With ten workers sharing one cache and one memory bus, saving work and saving time come apart. Correctness certifies that a change is admissible. Its cost lives in the data and the machine.

## 2\. What heterogeneity looks like in a Wikidata block

Heterogeneity is measurable, and it shows at three scales.

**Within a block.** Most of a Wikidata block is a repeated JSON skeleton of keys and punctuation, which bzip2 already turns into long runs, cheap to invert. A smaller share is repetition at a distance: `sha1` references and repeated strings, copies of text already decoded. Then there are the statement identifiers. They are random 122-bit GUIDs, and the birthday bound means their contexts shatter after about four or five hex digits. The [classical-tools audit](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-22-classical-tools-audit.md>) measured six blocks. There they are 53.6–67.1% of the width-1 rows the Open kernel retains, and 28.7% of its emission time. Three structures, three costs, in one block.

**In the decoder's own terms.** The dispatch's gates are written in one structural statistic of a block, r/n: run records per row. So the heterogeneity can be read off the gates themselves:

Where each block's r/n falls among the gates

One dot per run-route block, from the per-block census of the open-admission journal (docs/journal/2026-11-19-open-admission/blocks.jsonl), coloured by today's route: green, Open; blue, Segmented. Larger dots are full blocks of 100,000 rows or more. Whether the runs corpus's two blocks fit the dense band's cap was not measured; Wikidata's five did.

Wikidata's 233 full blocks lie between r/n 0.059 and 0.206, all inside Open's range. Five of them are in the dense band. Its 150 tiny blocks lie between 0.40 and 1.0 and go to `Segmented`. By count, Open takes barely half the corpus's 431 blocks. By rows, it takes 99.996%. Text sits just past the dense band, at 0.29–0.33. The periodic `pattern` and `onebyte` sit below the sparse gate, at 0.008.

**Between corpora.** One exact change gains differently on different data. The block reader of \#745 on CI's paired bench, at 4 threads:

One exact change, seven corpora: the block reader's wall time at 4 threads

CI's paired bench on \#745 (10 rounds, a 4-core Xeon runner, not the i9). Green: faster, the interval excluding no change. Grey: no change shown. The bz2 ratio is each corpus's output over its compressed size. Every run of both builds gave the same output on every corpus.

The change speeds up the serial stage that reads the compressed stream. The barely compressible `records` and `random` gain 6–7%. The `runs` and `onebyte` corpora, compressed hundreds or hundreds of thousands of times, show nothing. That fits the serial stage's share of the work growing with the compressed bytes, though the bench does not measure that share per corpus.

**Between families of data.** A hindsight oracle chose, per block, the fastest of thirteen kernel configurations over a research build: eleven settings of one Open construction, and two walks. On the 431 Wikidata blocks it would save 3.17% over that build's fixed setting. On six non-Wikidata blocks it would save 12.8–13.1%. Both are single-threaded, on a research host. The same family of kernels has a different headroom on different data.

## 3\. Exact routes, each betting on a structure

The [proof programme](<https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/PROOF-PROGRAMME.md>) (§5.1) reads the shipped dispatch from `src/` on master. Here it is for `OpenCells`, the kernel workers hold at 8 threads and above:

The per-block dispatch: exact routes behind cost gates

The dispatch of OpenCells (src/lib.rs, src/runs.rs, src/open.rs, src/bwt.rs on master). Below 8 workers the kernel is Cells, and the cooperative teams use their own walks; PROOF-PROGRAMME.md §5.1 lists them all. Research candidates are not shown.

A block's records are read when enough of its symbols are runs. On that route, the Open kernel takes the blocks its count gates admit. It also takes a dense band up to r/n 0.28, under a cap on retained rows ([\#703](<https://lmmx.github.io/mibz2/pr-preview/pr-797/blog/2026-11-21-a-memory-gain-is-a-gain/index.md>)). Everything refused goes to `Segmented`, the walk that accepts every block.

The figure's point is its two kinds of line. Each box is a theorem of one shape: for route i, Pᵢ(x) ⇒ Fᵢ(x) = R(x). Here R is the reference, the walk of the LF permutation on a valid block. For every shipped route, Pᵢ is only the entry condition, a valid block. Each is tested against R on arbitrary columns, not just on Wikidata. The gates are not part of any correctness argument. Each chooses for speed or for memory. Correctness is uniform across the routes; the structure each bets on is a cost hypothesis.

The Open kernel is the route where the unit of computation changes. The LF walk visits one row at a time, and on a block of runs most of those visits are predictable. Open treats the thin runs as strips and computes first-return maps between them, reached through Rauzy–Veech induction. The walk then jumps from strip to strip rather than from row to row. Its [proof](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-18-open-return/proof.md>) is in the journal. Inside it, two more bets: a width certificate (below) and repetition of a short marked cycle.

## 4\. When the data refuses

Three refusals from the research lines show where the work is.

- **A solver that never fires.** A pure-arithmetic solver for two-letter Sturmian blocks inverts them in O(log n) steps by the Euclidean algorithm, and it is exact on 5,627 Sturmian BWTs. On Wikidata, `gcd(w) = 1` on every block and 0.00% of Rauzy–Veech steps have a quotient of 2 or more. The hypothesis is sound, cheap to test, and true of no real block.
- **The GUID wall.** No predicate on local structure picks the GUID rows out for a shortcut, because random identifiers have none. The escape would be to store them other than as BWT rows, which bzip2's format does not allow.
- **The copy sector.** Three independent mechanisms found the repeated `sha1` and text rows and cut the hops spent on them by 16–58%, byte-exact. All three lose 1.14–2.64× end to end, because finding a copy costs more than emitting it saves. A saved hop costs 10.1–10.5 ns, against 5.79 ns to emit a row. The structure is real and common; recognising it is the obstruction, on the research host.

Each refusal has three possible answers, and each has an example:

- **Weaken the hypothesis.** Open's correctness never needed the density line r/n \< 0.15. A cap checked by the first pass replaced it, which admitted the dense band up to 0.28 (\#703).
- **Certify it more cheaply.** The fast elimination loop needs a block without long spirals. Rather than detect spirals, one comparison checks that the widest strip is at most 32,768 rows. No elimination step widens a node, so the widest strip bounds every spiral's length.
- **Change the representation.** Open itself: the first return to the thin runs needs no assumption on the block, where earlier routes were gated by density and budget.

The first two add or move a branch of the dispatch. Only the third removes branches, and it is what keeps the decoder from becoming a tree that grows with every regime found.

We propose to keep a map of these moves. Each entry records the hypothesis moved from, the move and the result. It also records the witness, the status and the measured effect. The obstructions go in it too, each with its scope. A failed implementation is not an impossible construction. The copy sector lost on one machine against one base; only an exact obstruction with a witness rules a construction out, and only within its scope.

## 5\. The stutter

The process in §4 has a shape, and an essay of Gilles Deleuze's names it well. In "He Stuttered", Deleuze separates a character who stutters in speech, which a writer can report, from a writer who makes the language itself stutter. In the second case the variation is not laid over a fixed language from outside. It comes from within the language's own system, putting it in continuous variation and disequilibrium. His examples include Melville, Kafka, Beckett, the poet Gherasim Luca and Charles Péguy. Péguy moves from a word to its near-synonym, with a slight shift of meaning each time.

The mapping to a decoder is close:

- **The fixed thing said** is the conclusion, = R. It never moves.
- **The reported stutter** is a special case bolted onto a uniform decoder. A branch notices a regime and handles it on the side, while the decoder's objects stay what they were.
- **The language stuttering** is a construction whose mathematical objects change with the data's structure. Rows become strips and returns; a hypothesis returns weakened, certified, or replaced by a representation that does not need it, and each return changes the question.

The difference in what drives the variation matters. For Deleuze it is the writer, working the language. Here it is the data's resistance, worked by us. The program is the language the proof is written in, so a proof that follows the data must vary with it too.

The analogy has limits, and they should be plain. Deleuze wrote about literature and language. He made no claim about algorithms or proofs, and we claim none for him. The analogy names the shape of the process. It is not evidence that the process works: the evidence is in §§1–4.

## 6\. Local and global

Open is the one global change of representation so far. It is a classical idea, the induced map of Poincaré and Kakutani, reached through Rauzy–Veech induction. Everything since has been bound to a regime: the dense band, the width certificate, the block reader.

Local constructions can lead to a global one. In mathematics a different proof of a solved case has opened new routes before. The Riemann Hypothesis for curves over finite fields was proved first by Weil. Then came Stepanov's elementary method, completed by Bombieri, and it was that method's descendants that travelled ([Two centuries of tools](<https://lmmx.github.io/mibz2/pr-preview/pr-797/blog/2026-11-22-two-centuries-of-tools/index.md>)). Inverse BWT has the same structure. The LF walk, cycle decomposition and first-return induction are different constructive proofs that the BWT is invertible, and each gives a different decoder. The proof programme names seven points, L0 to L6, where one step of the proof can be replaced. Open replaced the classical route at L3, L4 and L5.

What we cannot yet say is how complete the family is. The oracle's 3.17% bounds only the alternatives it chose between, over a research build rather than master. Those were eleven settings of one construction's two parameters, and two walks. It says that family is nearly saturated on Wikidata, and nothing about a construction outside it. The 10-thread plateau points the same way. Every construction gain of the last twelve research cycles left DRAM traffic at 10 workers unchanged, at 4.91 bytes per output byte.

## 7\. What is proved, measured and open

**Proved:** nothing in Lean yet. The plan is the specification first, the LF walk, then the Open kernel as a conditional route, then the dispatch lemma that joins the routes. **Measured:** the i9 figures in §1, CI's per-corpus bench in §2, and each route's byte-exact parity against the reference. **Open:** whether the local constructions we keep finding are pieces of one representation we have not yet seen, or whether real data needs many routes for good.

---

**Sources:** [lean-zip's README](<https://github.com/kim-em/lean-zip>); Gilles Deleuze, "He Stuttered", in *Essays Critical and Clinical*, trans. Daniel W. Smith and Michael A. Greco (1997), « Bégaya-t-il… » in *Critique et clinique* (1993).

**Research records:** [the classical-tools audit](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-22-classical-tools-audit.md>); [the proof programme](<https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/PROOF-PROGRAMME.md>), §5 for the routes, the hypothesis map and the oracle's scope; [Open's first-return proof](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-18-open-return/proof.md>); the i9 evidence bundles named in the first figure, on the `evidence` branch; CI's bench comment on \#745.
