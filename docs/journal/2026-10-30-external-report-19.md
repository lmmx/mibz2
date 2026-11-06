# External research report 19: research burn-down round 15, the literature cluster (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 18 (`2026-10-30-external-report-18.md`).

**What it contains:** source-verified triage dispositions for the register's literature
rows.
- **#28:** Petri's thesis is from 2013, not 2024.
- **#29:** Alanko 2020 belongs to the Wheeler-graph and tunnelling family.
- **#30:** Navarro's survey has two complementary parts.
- **#31:** Lazy LZ is about constructing an LZ parse, not decoding one.
- **#32:** the two Kempa titles are one research lineage, a preprint and its published
  version.
- **#33:** the bijective BWT is a different transform.
- **A one-shot amortisation inequality,** C < Qδ, separating the indexing literature from
  decoding a single block.

The researcher states these are triage dispositions, not cover-to-cover reviews.

**Cross-checks:**
- **#28–#33 and #36:** an independent read-only literature agent is checking these,
  together with N1's precedent check (T4). Its findings are reconciled in
  `2026-10-30-research-portfolio.md` §9 when it reports.
- **The researcher's next cluster** (#37, #38, #43, #45, #47, #48, #51, #58) already has
  dispositions in the ledger's latest version, §9, published after the branch the
  researcher read.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text and is kept as received.

## Round 15, as received

````text
## Burn-down round 15: the six unresolved literature questions

I went back to the original [42-paper triage](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-27-literature-triage.md?utm_source=chatgpt.com) and checked the primary publication records. This immediately exposed a bibliographic error and established several useful dispositions.

### First finding: #28 is misdated by eleven years

The register lists Scalable Succinct Indexing for Large Text Collections as “Petri (2024).”

The located original thesis is by Matthias Petri, July 2013, at RMIT University.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://jsc.github.io\&sz=32)

jsc.github.io



That matters because it's not recent evidence of a new CPU-friendly inverse-BWT structure. It is older succinct-index research, and its direct relevance must be assessed through its actual algorithms rather than the apparent publication date.

\#28 disposition: bibliographic error resolved. No demonstrated new inversion mechanism; a detailed algorithm audit remains separate.

### Second finding: #30 is a genuine two-part survey, not duplicate bibliography

Navarro's Indexing Highly Repetitive String Collections has two complementary parts.

Part I examines repetitiveness measures, compression paradigms and direct access to compressed strings. Part II concentrates on the indexing mechanisms built on those foundations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://users.dcc.uchile.cl\&sz=32)

users.dcc.uchile.cl+1



For mibz2, that division is useful:

- Part I helps assess whether BWT run count \\(r\\) is sufficient to predict a decoder representation's cost.
- Part II is the relevant place to look for rank/select, locating and shortcut data structures.

But neither part is itself an algorithm that decodes a standard bzip2 block with less work.

\#30 disposition: bibliographic identity resolved; retain as background and a route to primary algorithm papers, not as an independent high-priority decoder candidate.

### Third finding: #33 is explicitly about a different transform

Bannai, Kärkkäinen, Köppl and Piątkowski's Indexing the Bijective BWT is a genuine 2019 paper. Its principal result is a self-index for the bijective BWT, using backward search.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://drops.dagstuhl.de\&sz=32)

drops.dagstuhl.de



That distinction is decisive for our research boundary.

Mibz2 cannot substitute a bijective BWT for the conventional cyclic BWT encoded in existing `.bz2` blocks. The paper's indexing algorithms therefore do not directly provide a faster decoder.

\#33 disposition: primary mechanism identified; no direct transfer established. Low priority unless a specific internal permutation technique proves independent of the transform variant.

### Fourth finding: #32 contains two titles for one research lineage

The two Kempa entries are not two unrelated 2017 algorithms.

Closing in on Time and Space Optimal Construction of Compressed Indexes is associated with arXiv:1712.04886. The published SODA 2019 work is Optimal Construction of Compressed Indexes for Highly Repetitive Texts. Both are by Dominik Kempa and concern building compressed indexes efficiently from repetitive text.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.researchgate.net\&sz=32)

researchgate.net+2



This matters because we should not count them as independent evidence for two construction techniques.

More importantly, the paper's sampled-index approach does not provide a cheap, direct replacement for mibz2's sequential inverse-BWT walk. Its suffix-array access mechanism may involve multiple LF steps and index queries. That is reasonable when building a reusable index; it is an unattractive cost model when materializing one 900 KB decompressed block.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.researchgate.net\&sz=32)

researchgate.net



\#32 disposition: bibliographic duplication substantially resolved; no independent decoder shortcut established. This is a strong candidate to mark answered in the register, subject to confirming the title history.

### Fifth finding: #31 is about constructing an LZ parse, not decoding one

Kärkkäinen, Kempa and Puglisi's 2016 Lazy Lempel-Ziv Factorization Algorithms is an actual journal paper about LZ77 factorization and related string algorithms.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://researchportal.helsinki.fi\&sz=32)

University of Helsinki Research Portal



The important distinction is between:

- Finding repeated substrings while constructing an LZ parse.
- Reusing repeated output during inverse-BWT reconstruction.

The former may use suffix or BWT-based data structures to locate matches. But a standard bzip2 block does not supply an LZ parse. Constructing one during decoding incurs additional work that the Precopy family was already intended to avoid.

\#31 disposition: the proposed transfer is indirect and overlaps the existing Precopy question. No independently economical work-reducing decoder mechanism has been identified from the verified paper description. A full algorithm audit would be needed before claiming the paper contains nothing transferable.

### Sixth finding: #29 belongs to the Wheeler-graph/tunnelling family

Jarno Alanko's 2020 doctoral thesis Space-Efficient Algorithms for Strings and Prefix-Sortable Graphs contains four constituent papers.

The University of Helsinki identifies its topics as shortest common superstrings, compact variable-order Markov models, Wheeler automata, and tunnelling on Wheeler graphs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.helsinki.fi\&sz=32)

University of Helsinki



The last two are relevant to our mathematical interests, but their principal purpose is indexing and compressed graph representation.

For mibz2, the useful question is whether they supply a cheaper way to recognize or construct translation intervals. Our current N1 work already builds such intervals from the BWT's run structure and composes them without general sorting.

I found no directly established alternative that reduces the construction traffic we now know is N1's central weakness.

\#29 disposition: thesis scope identified, overlap with the existing tunnelling and compressed-index family established; no new independent decoder candidate justified by the available evidence.

## What this does to the six-question literature cluster

| Question          | Finding                                                              | Recommended disposition                                       |
| ----------------- | -------------------------------------------------------------------- | ------------------------------------------------------------- |
| #28 Petri         | Thesis is from 2013, not 2024                                        | Bibliography corrected; retain technical audit as conditional |
| #29 Alanko        | Thesis primarily covers compact string structures and Wheeler graphs | Deprioritize; overlap established                             |
| #30 Navarro I/II  | Two complementary surveys, both published in 2021                    | Bibliographic identity answered; use as references            |
| #31 Lazy LZ       | LZ parsing construction, not an available bzip2 decoding primitive   | Deprioritize under P04/Precopy                                |
| #32 Kempa ×2      | Preprint/published research lineage, not two independent hypotheses  | Resolve duplicate; no new decoder candidate                   |
| #33 Bijective BWT | Different transform and indexing objective                           | Deprioritize for ordinary `.bz2`                              |

These are source-verified triage dispositions, not six completed cover-to-cover technical reviews. I'd mark #30 and the duplication aspect of #32 answered, correct #28's bibliographic record, and retain narrowly scoped conditions for any deeper reading where source mechanisms remain unexamined.

The literature cluster has substantially shrunk without yielding another high-priority implementation proposal. That's a legitimate outcome.

## A more useful scientific deduction from these papers

There is a recurring divide between the indexing literature and our decoder problem.

An index is built once and queried repeatedly, so its setup cost can be amortized across many searches.

Mibz2 reconstructs an entire block once.

For a proposed auxiliary structure, let \\(C\\) be its construction cost, \\(\delta\\) the saving per query, and \\(Q\\) the number of queries.

The structure pays only when

\\[ \boxed{C\<Q\delta.} \\]

For N1, \\(Q\\) is approximately \\(n/8\\), adjusted for segment termination. For one-byte navigation structures, \\(Q\\) may approach \\(n\\).

This simple inequality connects the literature work to the new 233-block census.

A succinct index with excellent query time can still lose badly on one-shot inversion if it performs multiple construction passes. Conversely, our Ψ⁸ representation may pay despite more bytes of construction traffic because it avoids many dependent queries.

The literature's compressed-space guarantees alone do not settle that comparison.

### Current burn-down direction

We've now addressed all six outstanding literature items at the bibliographic and mechanism-classification level, while preserving the distinction between source triage and complete algorithm verification.

The next questions I would work on are the remaining information-theoretic and CPU layout questions #37, #38, #43, #45, #47, #48, #51 and #58. These are still important because they could reveal whether a cheaper compact representation exists independently of N1's heavy builder.

The most important discipline is to avoid treating an open measurement as an open mathematical mystery—or treating an old index construction as a decoder breakthrough without accounting for its one-block setup cost.

The 84-question inventory is getting less ambiguous. We have not created another backlog, and I haven't modified the repository or launched any experiments.
````
