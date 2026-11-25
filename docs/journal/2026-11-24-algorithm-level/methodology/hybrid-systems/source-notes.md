# Sources and coverage

## Book

Rafał Goebel, Ricardo G. Sanfelice and Andrew R. Teel, *Hybrid Dynamical Systems: Modeling, Stability, and Robustness*. Princeton University Press, 2012. Print ISBN 9780691153896; DOI [10.1515/9781400842636](https://doi.org/10.1515/9781400842636).

- [Authors' book page](https://hybrid.soe.ucsc.edu/hybridsystemsbook).
- [Authors' bibliographic record](https://hybrid.soe.ucsc.edu/biblio/2012/hybrid-dynamical-systems-modeling-stability-and-robustness).
- [Public Chapter 1 sample](https://hybrid.soe.ucsc.edu/sites/default/files/preprints/65.pdf).
- [Authors' errata](https://hybrid.soe.ucsc.edu/publications/errata-hsbook).

The public PDF has 23 pages and contains Chapter 1, not the entire book. Its text and mathematical diagrams were read for the introductory précis. The précis is deliberately concise; the longer decoder discussion is independent analysis.

The private full book and chapter files were supplied in the earlier conversation. Their file references remain available, but their contents could not be retrieved in this working session. A complete nine-chapter reading therefore remains open. Reattaching the original PDF or chapter ZIP is sufficient to continue it. Chapter titles in the index follow the previously supplied chapter files; a pending row means no full reading is certified here.

The repository contains original notes and links. It does not contain the private PDF, chapter PDFs, page images, extracted book text, or private attachment identifiers.

## Decoder baseline

All code and repository observations in these notes were checked at commit [`9c38ad6c18237a1b923f9b762a1f634864d3b8a4`](https://github.com/lmmx/mibz2/commit/9c38ad6c18237a1b923f9b762a1f634864d3b8a4), on `docs/journal-algorithm-level`. The notes are a dated interpretation of that state.

Primary repository sources:

- [Experiment programme](../../briefs/experiment-programme.md): stage 1 completed; scanner work queued; stage 2 and subsequent recalibration defined.
- [Structure and function](../../leads/structure-and-function.md): the existing design-space and dynamical work.
- [Stage-1 report](../../math-targets/data/character-stage1/report.txt), [per-count measurements](../../math-targets/data/character-stage1/per_k.tsv), [contention](../../math-targets/data/character-stage1/contention.tsv), [public API](../../math-targets/data/character-stage1/api.tsv) and [instrumentation cost](../../math-targets/data/character-stage1/cost.tsv).
- [`src/parallel.rs`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs): admission, queues, ready buffers, reader and workers.
- [`src/stream.rs`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/stream.rs) and [`src/block.rs`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/block.rs): scanner and current intermediate representation.
- [`src/lib.rs`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/lib.rs) and [`src/read.rs`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/read.rs): worker transformations and public output interface.

The stage-1 report describes 840 grid decodes and additional cost, counter, memory-controller and API observations. It is diagnostic and reports `PERTURBED`. In particular, reader activity is now measured; the older review's missing-reader limitation must not be repeated as a current fact. Values copied into these notes are summaries, not a new independent dataset or a new adoption verdict.

## Reading protocol for the remaining chapters

Read each chapter completely, including examples, qualifications and notes; inspect mathematical notation against the page where extraction is ambiguous. Reconcile the author's errata. Write an original explanation, with printed section/page references and explicit theorem hypotheses. Then write the separate application: proposed decoder change, model assumptions, observable prediction, cheapest disproof, and whether adopting it changes a production default.

The collection must not be labelled complete until all nine chapter rows are marked read and their notes have been checked against the supplied full text.
