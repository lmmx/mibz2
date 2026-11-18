<!-- Reader's note (added on committing): this is the recovery document the ChatGPT Pro session
produced after its original session ended without a handoff, on 18 November 2026. It is preserved
verbatim below. It is a reconstruction from that session's reasoning and terminal transcript, not
independently reproduced evidence: the programs, CSVs and PDF it names were not recovered. The
brief that started the session is docs/journal/2026-11-18-pro-investigation-discussion.md §0, and
the discussion alongside it is in that file. Independent assessment by MONODROMY and the
Researcher is pending. -->

# mibz2 — recovered investigation state

**Recovery date:** 18 November 2026. **Status:** a reconstruction from the previous turn's recoverable reasoning and terminal-output transcript, **not** a rerun of the research or a verification of files from the missing workspace.

## Access audit

- The preceding turn's accessible reasoning/terminal transcript survives. It contains draft report extracts, commands, hashes, experimental rows, and descriptions of complete tests. Its last visible shell invocation edited `handoff/report.tex`, updated `handoff/README.md`, copied the oracle program and two CSVs into the handoff, invoked `sh typeset/build.sh`, invoked `pdftotext`, then tried to list PDF page counts. **No corresponding command result is present in the transcript**; completion/success/failure cannot be established.
- The prior shell workspace was `/workspace/scratch/e5fd3d75ec08` and contained a `repo/` checkout and private experiment directories including `handoff/`, `analysis/`, `textorder/`, `structure/`, and `typeset/`. That tree **is not available in the present filesystem**. The active `/mnt/data` contains the original `/mnt/data/pro-attachments.zip`, not the prior experiment outputs. No experiment, builder, or oracle job is running in the current process namespace. This is not proof that the old process ended successfully: the namespaces are different.
- The original nine-file user attachment is intact, including `packet.md`, `reading-list.md`, two Astra investigation journals, discussion, `open-return-proof.md`, README, and `derive_runs.py`. It does not contain the new work described below.
- The transcript mentions a 12-page typeset `typeset/report.pdf` *before* the final oracle-page edit. There is no accessible PDF, LaTeX source or final archive to inspect. No Git commit or pushed PR for the new work is attested in the recovered transcript.

## Principal new mathematical direction

The previous investigation switched from labelled return grammars to **ordered terminal copies**. It distinguished (a) the BWT row/rank order, (b) the actual forward text order, and (c) the order in which data is made available for replay. The essential claim was that constant-time rank-neighbour byte equalities can be turned into nonoverlapping substring-copy rules, provided their destinations and dependencies are obtained. Instead of explicitly spelling a word for every return path, an independent terminal-order construction was reported to reconstruct the complete pre-RLE1 output from literal seed bytes and copied substrings.

A recovered proof sketch of the rank-neighbour identity: within any maximal BWT run, the stable-order LF map obeys `LF(x+1)=LF(x)+1` for adjacent rows `x,x+1` in that run. Hence, whenever both adjacent trajectories have not been interrupted by a cut, applying LF to the two rows preserves adjacency and equality of the emitted byte (their `L` values agree). Iterating supplies equal output fragments and a copy relation. The proof's **byte equality is unconditional under its stated same-run itinerary conditions**; the *availability of its source in forward emission* is a separate property and must not be inferred from the equality. The previous work claimed a stricter well-foundedness principle for its particular decomposition: if a source byte remained unknown, its representative BWT rank was strictly smaller. The exact full induction and the code that enforced this condition are **not currently accessible** and require auditing before this becomes a stand-alone theorem.

The resulting descriptions were reported to be small, but discovery and coordinate placement cost more than Cells. This is a **negative performance verdict for the constructions tested**, not a lower bound for every copy-based method.

## Recovered results and checks (transcript evidence only)

| Investigation/quantity | Recovered observation | Scope or qualification |
|---|---|---|
| Full decoder correctness | All 431 Wikidata blocks, their block CRCs, and 12 combined stream CRCs passed; 75,736 marked small/random tests matched conventional inverse. | Reported prior executions; test logs/executables not accessible now. |
| Return dictionary | 61,393 slots; 3,448 literal roots; 2,976 slice instructions. | Frozen block, reported. |
| A second, dictionary-free route | Placed 96,770 rows by scalar first returns and recovered 803,161 other bytes with 37,675 logical append copies. | Frozen block; dependency scheduling reportedly valid, but construction cost unfavorable. |
| Full-text terminal-order replay | 51,184 run-head seed bytes; 32,862 intervals; 64,258 copies, of which 32,670 singletons. | Frozen block; recovered command and expected output checksum. |
| Copy dictionary directly | 5,868 initial translated intervals compiled into 11,588 copies over 61,393 bytes, including 6,614 singleton copies. Median copies/representative tracing length 0.190 across 233 large blocks. | Reduces descriptor size but still costs construction. |
| Boundary phases | 17,578 distinct sample requests recover 67,252 distinct head/tail ranks on frozen block; median 19,594 requests over 233 large blocks. | Recovered as reported measurements. |
| Phase-sampled total inverse | 6.043 ms sampling+head enumeration, 0.846 ms forming exchange; 8.942 ms total versus 2.366 ms labelled control (45 frozen paired rounds). | Same test host/experiment; not comparable to i9 production baseline. |
| Direct open section at just the sampled ranks | 84,718 scalar steps on 40,871 complementary strips. | Fewer rank values were not a cheap way to obtain the required positions. |
| Coarsest fixed-order labelled translation partition | Retained subcolumn `v` has 47,333 runs, partition has 47,544 intervals; dropping return words leaves 47,544. Removing spelling saved 0–4 boundaries, median one, over 233 large blocks. | Obstruction to the specific labelled-partition merge, not to changing coordinates. |
| Delete isolated ranks before induction | Reduced induction steps on all 233 large blocks; median reduction 19%. Two methods of restoring them still expanded nearly the entire dictionary and showed no reliable whole-inverse speedup. | Negative trial, previous run only. |
| Text-oracle minimum append-copy parse | **11,145** copies for the same 51,184-byte run-head seed, only **33 singletons**, maximum copy length **4,819**. Another seed containing all 256 ascending bytes: **20,068** copies, **917** singletons. | This algorithm **is supplied the final decoded text as an oracle**; not an RLBWT constructor. |
| Witnesses for the oracle parse | 6,124 copies source wholly in the seed (42,627 bytes); 5,021 source in previously decoded output (857,304 bytes); no seed/output-seam crossings. | 899,931 target bytes exactly. |
| Oracle vs fixed terminal-order replay | On ten prespecified Wikidata sampled blocks, observed fixed-program/oracle ratio 4.79–6.75, median 5.72. Oracle copies/head-cluster count 0.929–1.040, median 0.998. | Oracle-discovery cost uncharged in these ratios. |
| LF-chain/rank-order seed | An alternative seed computable without sampled text positions reportedly gave 11,130 oracle copies on frozen block. | Oracle still needs the text; exact generating program not accessible. |
| Short-run simple rank-layer shortcut | 347,803 copy operations even with addresses provided. | Reported earlier in progress messages; not promising. |

The archived oracle program `textorder/append_oracle.cpp` reportedly built a suffix automaton of available seed+decoded prefix, greedily took the longest *nonoverlapping* previously available substring, and compared against an independent exact dynamic program on **20,475** small input/seed pairs. Its extracted output after an additional instrumentation pass included `seed_only_ops`, `seed_only_bytes`, `cross_ops`, and `cross_bytes`. A saved-looking CSV header and frozen row appeared in the terminal transcript, but these files themselves are not accessible here.

### Copy-optimality lemma (recovered proof; independent of measurements)

**Model:** A target `T` is emitted in increasing text order after a fixed initial seed `D`; a copy instruction appends a substring wholly present in `D + T[:t]` at the current output position `t`, with no source/destination overlap; each copy costs one regardless of length. Assume every required byte is available as a singleton somewhere in the seed, so parsing can finish.

**Claim:** A greedy parser taking the *longest* legal copy at each output position minimizes the number of copy instructions.

**Argument:** At a common starting position, take an optimal first phrase of length `a` and a longest legal phrase of length `b>=a`. The greedy phrase covers the entire first optimal phrase and possibly further optimal phrases. Discard fully covered phrases. If the greedy phrase ends partway into an optimal phrase, the remaining suffix can be copied by shifting that phrase's previous source forward by the same offset; its source was available at the original phrase's start, hence remains available now. This gives a parse of no greater phrase count after the greedy first phrase. Iterate from the new shared boundary. Thus greedy is optimal. The claim is **not** that the oracle parse can be cheaply found from BWT input.

A related admissibility condition for an output-only source at text position `t` and length `ell` is the existence of another occurrence starting at `j<=t-ell`, under normal text-end restrictions. The previous notes proposed using the lexicographic interval of rotations and its minimal text position to test this. Obtaining such intervals and minima from the RLBWT was explicitly left as a discovery-cost problem.

## Failed or insufficient approaches

1. Run neighbours do give exact equalities, but the first whole-span dependency schedule was cyclic: of 10,954 spans covering 848,747 bytes, only 3,123 spans covering 5,063 bytes could be settled whole even with coordinates provided (from the *earlier* committed open-return investigation, distinct from the new copy schedule).
2. Compiling equalities to a copy log reduces the *number of output instructions*, not necessarily the effort to learn their destinations. Scalar phase recovery on the actual BWT was substantially slower than the labelled control.
3. Merging intervals merely because their word labels can be erased leaves almost all necessary translation boundaries intact.
4. Pruning isolated ranks saves induction work but adds restoration work. Both tested restorations failed to improve the full inverse reliably.
5. Choosing a seed with very few literal pieces (70 reported) increased copy count to 17,749 in one dictionary experiment. Seed count alone is a misleading surrogate.
6. Oracle-minimal copy parsing demonstrates substantial untapped *description-level* repetition. It says nothing yet about the computational cost of learning those copies from the compressed input.

## Recovered draft handoff and preservation gap

A `handoff/report.tex` draft had 12 pages in a successful *pre-final-edit* typesetting inspection, with these sections: Research handoff; The two orders and input contract; Recovering labels after scalar induction; Obtaining slices from runs; Compiling equalities into append copies; Applying replay directly to output; Which boundary phases must be recovered?; What the real blocks paid for; Removing isolated ranks before induction; How many append copies does the text require?; What the smaller descriptions rule out; Decision and continuation.

The final visible shell command inserted a new oracle-parsing section from `analysis/optimum-page.tex`, copied `textorder/append_oracle.cpp` to `handoff/code/`, copied `textorder/append-oracle-sample-detail.csv` and `textorder/append-oracle-frozen.csv` to `handoff/evidence/`, updated `handoff/README.md` with a frozen-block reproduction recipe, and attempted to rebuild the PDF. **There is no observed completion line for this command and no accessible updated PDF.** It is unknown whether the insertion duplicated a section, whether typesetting succeeded, or whether a final zip was produced.

The prior transcript records the following **expected** example checksums, not checks performed in this recovery session:

- `handoff/sample/0001-264045.L` (899,931-byte BWT column, primary 264045): SHA-256 `db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67`.
- The executable `textreverse-radix.cpp` at arm 16 with `--out` produced the pre-RLE1 stream with SHA-256 `f87e5caa2d5f0df9d5958fe60901e2023ae04ac406603d4c1584c8bb4017fbd6`.
- The previous draft README used `g++ -O3 -std=c++17 code/textreverse-radix.cpp -o replay`, then `./replay 16 sample/0001-264045.L 264045` and `./replay --out 16 sample/0001-264045.L 264045 | sha256sum`.

Other filenames mentioned in the prior workspace: `handoff/evidence/archived-build-small-checks.txt`, `textorder/append-oracle-sample.csv`, `textorder/append-oracle-frozen.csv`, `textorder/append-oracle-sample-detail.csv`, `typeset/report.pdf`, `typeset/with-oracle.txt`, `analysis/optimum-page.tex`, `handoff/README.md`, `handoff/report.tex`, `handoff/code/textreverse-radix.cpp`, `handoff/code/append_oracle.cpp`, and a `handoff/sample/` BWT file. **None has been recovered as an actual file** in the present container.

## Most consequential next test (proposed, not performed)

Grant the constructor the *ordered known literal values*, **16,917 gap endpoints and their translations** for free, then charge the actual effort to produce bytes at their final output addresses, including joins and copies. Compare with the reported **2.392 ms labelled inverse**. The previous constructor reportedly lost even its equation/compilation stages under this generous input assumption. A genuinely different direct interval-consumption/output strategy would need to beat the control with that oracle input first; any winning margin bounds the budget available to derive those endpoints from the RLE. This isolates the key unsolved challenge: computing copy witnesses and output coordinates cheaply from the **actual BWT input**, not compressing a fully known string.

**Scientific bottom line:** A proof of a small output copy description and a proof of cheap BWT-to-copy construction are different achievements. The first is supported by recovered proofs and reported exact experiments; the second was **not established**. No new general lower bound on inversion, memory traffic, or I/O was established.

## Preservation decision

This Markdown document is the newly saved recovery artifact. The original attached source packet remains available at `/mnt/data/pro-attachments.zip`. The previous turn's original new programs, native binaries, complete raw CSVs and final LaTeX/PDF cannot be exported because their backing workspace is inaccessible; the evidence and source snippets visible in the transcript are preserved here as far as their content allows. **Do not attribute this document the status of an independently reproduced experiment.**
