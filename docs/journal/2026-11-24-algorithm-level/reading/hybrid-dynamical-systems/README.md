# Hybrid Dynamical Systems: reading notes

Goebel, R., Sanfelice, R. G. and Teel, A. R., *Hybrid Dynamical Systems: Modeling, Stability, and Robustness*. Princeton University Press, 2012.

ChatGPT recommended it as the language for the decoder's "digital on analogue" dynamics (journal §67, `../../external/chatgpt-dynamics.md`). Louis has a private copy. **The book itself is not in this repository and must not be added:** the PDFs and the text extracted from them stay outside it. What is here is original writing: précis in our own words, an index of section titles and page numbers, and our interpretation.

## Layout

| path | what it is |
|---|---|
| `sections.tsv` | every section with its printed page, and the précis file that covers it |
| `precis/ch01-introduction.md` … `precis/ch09-conical-approximation.md` | one précis per chapter, section by section, naming each definition, theorem and example by number and printed page |
| `precis/extraction-notes.md` | places where the extracted text was garbled or the print seems wrong; check these against the book before relying on them |
| `relevance/` | **separate from the précis**: the Coordinator's interpretation for our work (see `relevance/README.md`) |

## How the précis were made

The chapters were split from Louis's copy and their text extracted on the command line. Each chapter was then read in full by one reader (three readers, three chapters each), who wrote its précis section by section in their own words. Proofs appear only as one-line strategies. The Coordinator read the contents, the preface and §1.1 directly, and wrote everything in `relevance/`. ChatGPT's own first reading is recorded separately (`../../external/chatgpt-hybrid-systems-book.md`).

## Where to start

Chapters 1, 2 and 4, then 6 and 8; 3 and 7 for the certificates; 5 as needed; 9 last. For the purpose of this project, start with `relevance/levers.md`.
