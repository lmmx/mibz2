# Admitted Wikidata BWT columns

Six exact BWT last columns from the bench corpus, `corpora/wikidata.bz2` (SHA-256 `82f65ace…`; the windows of `wikidata-20261019-all.json.bz2` in `bench/wikidata.json`). They span the Cells regime at run-count r/n 0.057–0.143. `block-<id>.L` is the raw column, one byte per row, as the decoder holds it after Huffman, MTF and RLE2 and before RLE1. `<id>` is the block's index among the corpus's blocks of at least 65,536 rows, as the MONODROMY records use it; block 0 is the frozen column.

`manifest.json` gives each column's n, r (maximal runs), r/n, SHA-256, primary index (bzip2's `orig_ptr`, as given), which kernels admit it, and its provenance (window, block in window, block in corpus).

The LF convention is stable rank: LF(i) = C[L[i]] + #{j < i : L[j] = L[i]}. To recover the block's bytes before RLE1, start at Ψ(primary index), where Ψ = LF⁻¹, and read L[x] while following Ψ. On block 0 this spells `412157,"id":"Q5412157"},…`.
