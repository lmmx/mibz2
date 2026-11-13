# 2026-11-10 · MONODROMY 2 · The generalized-Lyndon literature read against the induction: 32 Scholar lines, a bibliography, what was already in print, and the steps as the tree's own count

Labels: **P** proved here · **C** counted (exact for its input) · **M** measured · **Conj**
conjecture · **K** known, cited with its place. "Entry 1" is
`2026-11-10-rauzy-veech.md`; its notation is used without restating it. No clock in this
entry: nothing here was timed.

## 0. Result

- **Asked (Louis, 10 Nov):** two Google Scholar result lists, reached from the references of
  Crochemore–Russo's *Cartesian and Lyndon trees* through Dolce–Restivo–Reutenauer's *On
  generalized Lyndon words*, to be recorded verbatim, made an itemised bibliography, and
  mined.
- **Verbatim:** `2026-11-10-lyndon-literature/scholar-results.txt`, 32 result lines as pasted.
- **Bibliography (§2):** the 32 lines are **21 works** (§1 maps each line: conference,
  journal, reprint and proceedings-copy lines of one work are merged, each merge from the
  sources); 27 more works were fetched to state things exactly. Every field is from a page
  fetched on 10 November; what could not be fetched is said, item by item.
- **For the decoder's speed: nothing.** No paper of the lists inverts a transform in fewer
  than n steps of LF, builds a Lyndon tree, array or grammar from a column, or works from
  the run-length form in time that depends on r alone (§3.4, §3.1).
- **Already in print, narrowing entry 1's claims (§5):**
  - the right-end induction, read in the text, is the **Schützenberger–Melançon rewriting**
    on the circular word (Melançon 1992; Reutenauer 2006, §3.4): merge a rise u < v whose
    right word is a largest word present. Every right-end step is such a rise (**C**);
  - what entry 1's §2 does with "a column that is no BWT" is the inversion of the
    **extended / bijective BWT** by its cycles, largest first (Gil–Scott; Kufleitner 2009,
    Theorem 8);
  - that every letter's word is a Lyndon word follows from two lemmas on next and previous
    smaller suffixes (as Ellert 2022 states them).
  What stays entry 1's: the rewriting done on the run-length column, one rise at all its
  occurrences at once in O(1), in S steps; the counts; the cells form.
- **Derived while reading (§4; P + C, brute force, `lyndon_induction_tests.py.txt`):**
  - **Theorem 5: the steps are the tree's.** For every order of right-handed steps, steps =
    the inner nodes of the derivation tree that have a hinge below them. It contains entry
    1's Theorem 3, gives the floor r₂ − 1 a second proof, and says that **no order which
    leaves the Lyndon tree is cheaper than the right end**: every saving entry 1 counted
    (1.279 r against 1.812 r) changes the grammar;
  - **the induction inverts the extended and the bijective BWT in S steps**, with the Lyndon
    tree of every factor: the only run-length inversion in print takes n steps;
  - **the left end is the right end for the inverse alphabet order** (the Lyndon tree,
    inverse order, of the greatest rotation): entry 1's 2.137 r against 1.812 r is a fact
    about the text, not about the ends;
  - **the left Lyndon tree is no order of steps** (smallest case `aabbc`), and where it can
    be counted it costs about n nodes over the hinges, not 1.8 r;
  - **the alternating BWT is the same exchange with every interval flipped;** the induction
    with a flip bit inverts it, and its tree is not the Galois words' standard tree;
  - on a two-run column `b^k a^l` the right end is **Euclid by subtraction** on (k, l).
- **Usable by the production kernel (§6, ranked):** a proved refusal after the order stage
  (S ≥ r₂ − 1, and r₂ is free there); the two-run columns as the division rule's unit test;
  a validity criterion from the run lengths' gcd (Likhomanov–Shur) for telling a corrupt
  stream from a periodic one. None of them moves the clock on Wikidata.

## 1. The lists

`2026-11-10-lyndon-literature/scholar-results.txt` holds the two lists as pasted, the first
above the dash line (A1 to A22 here), the second under the heading "On generalized Lyndon
words" (B1 to B10). Between them the message carried its three requests, which are §0's
first item and are not in the file. A line's number is its place in its list.

| line | title as pasted | work |
|---|---|---|
| A1 | Algorithms for Galois words: Detection, factorization, and rotation | [HKYS24] |
| A2 | Some variations on Lyndon words | [DRR-var] |
| A3 | Block sorting-based transformations on words: beyond the magic BWT | [GMRRS18] |
| A4 | [HTML] Linear construction of a left Lyndon tree | [BC] |
| A5 | Reprint of: ω-Lyndon words | [PZ20] |
| A6 | [HTML] Generalized Lyndon factorizations of infinite words | [BW19] |
| A7 | [PDF] Constructing the bijective BWT | [BKKP] |
| A8 | V-Words, Lyndon Words and Galois Words | [DMS] |
| A9 | [HTML] Generalised Nyldon words | [FHN25] |
| A10 | Some variations on Lyndon words (invited talk) | [DRR-var] |
| A11 | Lyndon words formalized in Isabelle/HOL | [HS21] |
| A12 | From the Lyndon factorization to the canonical inverse Lyndon factorization: Back and forth | [BDZZ26] |
| A13 | Generalized Lyndon factorizations of infinite words | [BW19] |
| A14 | Synchronous Combinatorial Games | [Wols26] |
| A15 | V-Words, Lyndon Words and Substring circ-UMFFs | [DMS] |
| A16 | Left Lyndon tree construction | [BC] |
| A17 | String Rearrangement Inequalities | [LZJ22] |
| A18 | String Rearrangement Inequalities and a Total Order Between Primitive Words | [LZJ22] |
| A19 | [HTML] Reprint of: Generalized Lyndon factorizations of infinite words | [BW19] |
| A20 | Generalized Lyndon Factorizations | [BW19] |
| A21 | Block Sorting-Based Transformations on Words: Beyond the Magic BWT | [GMRRS18] |
| A22 | [PDF] Local and Global Regularities in Infinite Words | [Postic20] |
| B1 | [BOOK] Hopf algebras in combinatorics | [GR-Hopf] |
| B2 | Constructing and indexing the bijective and extended Burrows–Wheeler transform | [BKKP] |
| B3 | A survey of the Bijective Burrows-Wheeler transform | [BKL] |
| B4 | [HTML] Inverse Lyndon words and inverse Lyndon factorizations of words | [BDZZ18] |
| B5 | Constructing the bijective and the extended Burrows-Wheeler transform in linear time | [BKKP] |
| B6 | [HTML] The alternating BWT: an algorithmic perspective | [GMRRS20] |
| B7 | Computing longest Lyndon subsequences and longest common Lyndon subsequences | [BIKKP24] |
| B8 | [HTML] On the longest common prefix of suffixes in an inverse Lyndon factorization and other properties | [BDZZ20] |
| B9 | Simple linear time algorithm for sorting strings in omega-order with applications | [LZJ22] |
| B10 | Lyndon words versus inverse Lyndon words: Queries on suffixes and bordered words | [BDZZ20] |
| (heading) | On generalized Lyndon words | [DRR-gen] |

## 2. Bibliography

Every field is from a page fetched on 10 November 2026 by the reader named in §3 (the
"record from" of each item); nothing is filled from memory. "Not verified" is the
reader's own list of what it could not reach.

### 2.1 The works behind the 32 lines (21)

- **[DRR-gen]** Francesco Dolce, Antonio Restivo, Christophe Reutenauer. *On generalized Lyndon words.*
  Theoretical Computer Science, 2019.
  Versions and what was read: Journal: Theoretical Computer Science 777 (July 2019) 232-242. Preprint: arXiv:1812.04515v1 [cs.DM], 11 Dec 2018, 16 pp. (the version read; it already thanks the referee).
  DOI `10.1016/j.tcs.2018.12.015`; arXiv `1812.04515`.
  Scholar lines: the heading of list B.
  *What it is:* Re-proves by word combinatorics that generalized Lyndon words (one alphabet order per position) factorize the free monoid; characterizes first and last factors; Galois words. No tree and no BWT in it.
  *Record from:* Crossref API record of the DOI (fetched: authors, journal, volume 777, pages 232-242, issued 2019-07); arXiv abs page 1812.04515 (fetched). DBLP could not be used: it answered with a bot wall.
  *Not verified:* Journal text not read (sciencedirect.com returned 403); all locators below are pages of arXiv v1.
- **[HKYS24]** Diptarama Hendrian, Dominik Köppl, Ryo Yoshinaka, Ayumi Shinohara. *Algorithms for Galois Words: Detection, Factorization, and Rotation.*
  35th Annual Symposium on Combinatorial Pattern Matching (CPM 2024), LIPIcs vol. 296, pp. 18:1-18:16, Schloss Dagstuhl, 2024.
  Versions and what was read: One work, two versions: arXiv:2403.02636 (v1 5 Mar 2024, v2 23 Apr 2024, 16 pp., 'accepted to CPM 2024'); CPM 2024, LIPIcs 296, 18:1-18:16. No journal version found on Crossref.
  DOI `10.4230/LIPIcs.CPM.2024.18`; arXiv `2403.02636`.
  Scholar lines: A1.
  *What it is:* Linear-time, constant-extra-space algorithms to test whether a word is Galois, to compute the Galois factorization online, and to find the Galois rotation; no inversion, no tree.
  *Record from:* DROPS landing page meta tags (title, authors, pages, DOI, conference); arXiv API entry; the arXiv v2 PDF.
  *Not verified:* The arXiv metadata gives the DOI as '10.4230/LIPIcs.CPM.2024.1'; DROPS gives 10.4230/LIPIcs.CPM.2024.18 (used here). LIPIcs volume number 296 read from the DROPS PDF path and from the reference list of DMS-VWORDS, not from a volume field. Read arXiv v2, not the LIPIcs PDF.
- **[DRR-var]** Francesco Dolce, Antonio Restivo, Christophe Reutenauer. *Some variations on Lyndon words.*
  arXiv preprint; 30th Annual Symposium on Combinatorial Pattern Matching (CPM 2019), LIPIcs 128, as an invited talk, 2019.
  Versions and what was read: (1) arXiv:1904.00954v1 [cs.DM], 1 Apr 2019, 16 pp. (2) "Some Variations on Lyndon Words (Invited Talk)", CPM 2019, LIPIcs vol. 128, pp. 2:1-2:14, Schloss Dagstuhl, 2019. Both PDFs read; the statements carry the same numbers in both (Theorem 36 is on arXiv p. 12 and LIPIcs p. 2:11).
  DOI `10.4230/LIPIcs.CPM.2019.2`; arXiv `1904.00954`.
  Scholar lines: A2, A10.
  *What it is:* Compares words by u^ω against v^ω; the prefix standard permutation of a Lyndon word; Theorem 36: its decreasing Cartesian tree is the left Lyndon tree (Viennot's left standard factorization).
  *Record from:* DROPS landing page of the DOI (fetched: title, authors, volume 128, pages 2:1-2:14); arXiv abs page (fetched).
  *Not verified:* No journal version was looked for beyond these two.
- **[GMRRS18]** Raffaele Giancarlo, Giovanni Manzini, Antonio Restivo, Giovanna Rosone, Marinella Sciortino. *Block Sorting-Based Transformations on Words: Beyond the Magic BWT.*
  Developments in Language Theory (DLT 2018), eds. M. Hoshi, S. Seki, Lecture Notes in Computer Science vol. 11088, Springer, Cham, pp. 1-17, 2018.
  Versions and what was read: One work, one published version: DLT 2018, LNCS 11088, pp. 1-17 (first online 5 Aug 2018 per Crossref). Open post-print at the Palermo repository (same text, its own page numbers). No arXiv version found (arXiv API title search returned nothing). The books.google hit is a copy of the same proceedings chapter.
  DOI `10.1007/978-3-319-98654-8_1`.
  Scholar lines: A3, A21.
  *What it is:* Defines the generalized transforms BWT_K (alphabet order depending on the position mod k), shows all are invertible in O(n^3), bounds their runs by 2 rho(w), and proves BWT and ABWT are the only rank-invertible ones.
  *Record from:* Springer landing page meta tags (title, authors, pages, DOI, 'vol 11088'); Crossref records for the chapter and the book (editors, ISBN 978-3-319-98653-1 / -98654-8); footnote of the post-print.
  *Not verified:* The ordinal '22nd' and the conference city are only in the Scholar line, not checked on the publisher page. Read the post-print, not the LNCS pages: theorem numbers and page numbers quoted are the post-print's.
- **[BC]** Golnaz Badkobeh, Maxime Crochemore. *Linear construction of a left Lyndon tree (arXiv and PSC title: Left Lyndon tree construction).*
  Information and Computation, 2022.
  Versions and what was read: (1) Journal: Information and Computation 285 (May 2022), article 104884. (2) arXiv:2011.12742v1 [cs.DS], 25 Nov 2020, 18 pp. (the version read). (3) Prague Stringology Conference 2020, pp. 84-95, as the arXiv version's footnote and its reference [1] give it.
  DOI `10.1016/j.ic.2022.104884`; arXiv `2011.12742`.
  Scholar lines: A4, A16.
  *What it is:* Builds the left Lyndon tree of a Lyndon word online, left to right, in O(n) letter comparisons from Duval's periodicity; its nodes are made in the order of the prefix ranks, which is the postorder.
  *Record from:* Crossref API record of the DOI (fetched: volume 285, article 104884, 2022-05); arXiv abs page (fetched).
  *Not verified:* Journal text not read (403). The PSC 2020 data are copied from the arXiv PDF, not from a PSC page.
- **[PZ20]** Mickaël Postic, Luca Q. Zamboni. *ω-Lyndon words.*
  Theoretical Computer Science 809 (original); reprinted in Theoretical Computer Science 834, 2020 (arXiv 2019).
  Versions and what was read: Versions: (1) arXiv:1907.01072, v1 1 Jul 2019, v2 9 Jul 2019, math.CO, 9 pp. (2) Original: Theoret. Comput. Sci. 809 (February 2020) 39-44, DOI 10.1016/j.tcs.2019.11.004. (3) 'Reprint of: ω-Lyndon words', Theoret. Comput. Sci. 834 (September 2020) 60-65, DOI 10.1016/j.tcs.2020.05.037 (Crossref 'associatedlink' points to the original). Read in full: arXiv v2.
  DOI `10.1016/j.tcs.2019.11.004`; arXiv `1907.01072`.
  Scholar lines: A5.
  *What it is:* Every infinite word factors uniquely as a non-increasing product of ω-Lyndon words, for any total order on infinite words satisfying a lexicographic-like condition; nothing on trees, BWT or algorithms.
  *Record from:* Crossref API records of both DOIs; arXiv API record; arXiv v2 PDF
- **[BW19]** Amanda Burcroff, Eric Winsor. *Generalized Lyndon factorizations of infinite words.*
  WORDS 2019 (LNCS, Springer); Theoretical Computer Science 809; reprinted in Theoretical Computer Science 834, 2019 (conference), 2020 (journal and reprint).
  Versions and what was read: ONE work, four versions: (1) arXiv:1905.04746, v1 12 May 2019, v2 20 Jun 2019, 14 pages, 1 figure. (2) Conference: in 'Combinatorics on Words, 12th International Conference, WORDS 2019, Loughborough, UK, September 9-13, 2019, Proceedings' (eds. R. Mercaş, D. Reidenbach), Lecture Notes in Computer Science, Springer, pp. 106-118, DOI 10.1007/978-3-030-28796-2_8 (first online 9 Aug 2019); the books.google line is this chapter. (3) Journal: Theoret. Comput. Sci. 809 (February 2020) 30-38, DOI 10.1016/j.tcs.2019.11.003. (4) 'Reprint of: ...', Theoret. Comput. Sci. 834 (September 2020) 66-74, DOI 10.1016/j.tcs.2020.05.040. Read in full: arXiv v2.
  DOI `10.1016/j.tcs.2019.11.003`; arXiv `1905.04746`.
  Scholar lines: A6, A13, A19, A20.
  *What it is:* Proves the Dolce-Restivo-Reutenauer conjecture: every infinite word has a unique non-increasing factorization into finite and infinite generalized Lyndon words; no trees, no BWT, no algorithm.
  *Record from:* Crossref API records of the four DOIs (chapter, book, article, reprint); arXiv API record; arXiv v2 PDF
  *Not verified:* LNCS volume number of the WORDS 2019 proceedings (Crossref gives none; DBLP and Springer pages were not reachable).
- **[BKKP]** Hideo Bannai, Juha Kärkkäinen, Dominik Köppl, Marcin Piątkowski. *Constructing and indexing the bijective and extended Burrows–Wheeler transform (earlier: "Constructing the Bijective BWT"; "Constructing the Bijective and the Extended Burrows–Wheeler Transform in Linear Time"; "Indexing the Bijective BWT").*
  Information and Computation 297, article 105153, 2024.
  Versions and what was read: ONE work in five versions. (1) IPSJ SIG Technical Report "Constructing the Bijective BWT", 175th SIG-AL, vol. 175 no. 19, pp. 1-6, Nov 2019 (author's BibTeX; PDF koeppl.github.io/bin/paper/algoken19bbwt.m.pdf, header "IPSJ SIG Technical Report"). (2) arXiv:1911.06985 v1 "Constructing the Bijective BWT" (16 Nov 2019); v2 "Constructing the Bijective and the Extended Burrows-Wheeler Transform in Linear Time" (22 Apr 2021). (3) CPM 2021, LIPIcs 191, pp. 7:1-7:16, doi 10.4230/LIPIcs.CPM.2021.7; its front page names arXiv:1911.06985 as full version. (4) CPM 2019 "Indexing the Bijective BWT", LIPIcs 128, pp. 17:1-17:14, doi 10.4230/LIPIcs.CPM.2019.17. (5) Inf. Comput. 297:105153 (issued March 2024): footnote on p. 1 says parts were presented at CPM 2019 and CPM 2021; author's BibTeX: "Journal version of" both. New in (5): section 4.6 Inversion, Theorem 4.21 (index in O(r) words), Table 6 (runs of BBWT against BWT). No inversion section in (2), (3) or (4) (grep of the three texts).
  DOI `10.1016/j.ic.2024.105153`; arXiv `1911.06985 (construction half only; v1 16 Nov 2019, v2 22 Apr 2021)`.
  Scholar lines: A7, B2, B5.
  *What it is:* Linear-time BBWT/eBWT construction by a modified SAIS, an FM-index on the BBWT with O(|P| lg |P|) backward steps, and (journal only) inversion by walking LF cycle by cycle.
  *Record from:* api.crossref.org/works/10.1016/j.ic.2024.105153; arxiv.org/abs/1911.06985; drops.dagstuhl.de/entities/document/10.4230/LIPIcs.CPM.2021.7 and .../LIPIcs.CPM.2019.17; dkppl.de/bibtex.bib
  *Not verified:* Publisher page not reached (403): the journal text was read in the author's copy, so theorem and page numbers are that copy's. Which koeppl.github.io file the Scholar line points to was not seen; algoken19bbwt.m.pdf is the only PDF with that title found there. Author's BibTeX records an erratum: Algorithm 2 calls the Lyndon factorization ascending, it is descending.
- **[DMS]** Jacqueline W. Daykin, Neerja Mhaskar, W. F. Smyth. *V-Words, Lyndon Words and Galois Words (extended version of: V-Words, Lyndon Words and Substring circ-UMFFs).*
  Conference version: Combinatorial Optimization and Applications (COCOA 2023), eds. W. Wu, J. Guo, LNCS vol. 14461, Springer, pp. 471-484. Extended version: arXiv preprint., 2023 (conference, 15-17 Dec 2023; first online 9 Dec 2023; the LNCS volume is print-dated 2024); 2024 (arXiv).
  Versions and what was read: ONE work in two versions. (1) 'V-Words, Lyndon Words and Substring circ-UMFFs', COCOA 2023, LNCS 14461, pp. 471-484. (2) 'V-Words, Lyndon Words and Galois Words', arXiv:2409.02757v1 [cs.DS], 4 Sep 2024, 30 pp.; its first-page footnote: 'A preliminary version of this paper was presented at COCOA 2023. The main new contributions in this extended version are Section 7 and Section 8.' Also an SSRN preprint with the arXiv title, DOI 10.2139/ssrn.5193979 (Crossref: posted-content, 2025). No journal article found on Crossref.
  DOI `10.1007/978-3-031-49611-0_34`; arXiv `2409.02757`.
  Scholar lines: A8, A15.
  *What it is:* UMFF / circ-UMFF theory: border-free minimal conjugates under any total order form a circ-UMFF; Galois words do not form a UMFF, the border-free ones do; a structure theorem for binary border-free Galois words.
  *Record from:* Springer landing page meta tags and Crossref (chapter and book records) for the conference version; arXiv API entry and PDF for the extended version; Crossref for the SSRN record.
  *Not verified:* The COCOA chapter is paywalled: only its landing page was fetched; the extended arXiv version was read in full. The SSRN page was not opened. COCOA's Crossref record gives the city as 'Hawai, HI' (sic).
- **[FHN25]** Pamela Fleischmann, Annika Huch, Dirk Nowotka. *Generalised Nyldon words.*
  Theoretical Computer Science 1046, article 115320, 2025.
  Versions and what was read: Theoret. Comput. Sci. 1046 (August 2025), article 115320 (OpenAlex: hybrid open access, CC BY-NC-ND, published 2025-05-14). Preprint: SSRN, DOI 10.2139/ssrn.5182616 (2025). No arXiv record: arXiv API queries 'au:Fleischmann AND au:Huch AND au:Nowotka' and 'all:Nyldon AND au:Fleischmann' return no such paper. ABSTRACT ONLY (from OpenAlex): ScienceDirect and SSRN answered 403. A different, earlier text by the same authors was read: 'Infinite Nyldon words', Theorietag 2023 technical report pp. 25-28.
  DOI `10.1016/j.tcs.2025.115320`.
  Scholar lines: A9.
  *What it is:* Abstract only: Nyldon words for a position-dependent alphabet order have a unique non-decreasing factorization, form a right Hall set and 'coincide with the anti-Lyndon words'.
  *Record from:* Crossref API and OpenAlex API records of the DOI (abstract from OpenAlex)
  *Not verified:* Everything beyond the abstract; arXiv identifier (none found).
- **[HS21]** Štěpán Holub, Štěpán Starosta. *Lyndon Words Formalized in Isabelle/HOL.*
  Developments in Language Theory, 25th International Conference, DLT 2021 (LNCS, Springer); Archive of Formal Proofs entry 'Lyndon words', 2021.
  Versions and what was read: Paper: in 'Developments in Language Theory, 25th International Conference, DLT 2021, Porto, Portugal, August 16-20, 2021, Proceedings' (eds. N. Moreira, R. Reis), Lecture Notes in Computer Science, Springer, pp. 217-228 (first online 6 Aug 2021). Formal development: AFP entry 'Lyndon words' (session Combinatorics_Words_Lyndon, theories Lyndon and Lyndon_Addition), 24 May 2021, BSD licence, updated to v1.12 on 19 Nov 2025. No arXiv record found (arXiv API 'au:Holub AND ti:Lyndon', 'au:Starosta AND all:Isabelle'). Read: the AFP proof outline (11 pp.), not the paper (Springer page behind a cookie wall; OpenAlex says closed access).
  DOI `10.1007/978-3-030-81508-0_18`.
  Scholar lines: A11.
  *What it is:* Machine-checked basics of Lyndon words: unborderedness, suffix characterisation, concatenation, longest Lyndon suffix, existence and uniqueness of the Lyndon factorization, existence of a factorization into two Lyndon words; no tree, no Hall set, no BWT.
  *Record from:* Crossref API records of the chapter and the book; isa-afp.org entry page and outline.pdf
  *Not verified:* LNCS volume number; the paper's own text (only the AFP outline was read).
- **[BDZZ26]** Paola Bonizzoni, Clelia De Felice, Rocco Zaccagnino, Rosalba Zizza. *From the Lyndon factorization to the canonical inverse Lyndon factorization: back and forth.*
  Computability 15(1-2), pp. 152-176, 2026.
  Versions and what was read: Journal: Computability vol. 15, issue 1-2, pp. 152-176, issue dated May 2026, published online 5 Aug 2026, SAGE (Crossref). Preprint: arXiv:2404.17969v1, 27 Apr 2024. Numbers below are arXiv v1's.
  DOI `10.1177/22113568251406824`; arXiv `2404.17969 (v1 read)`.
  Scholar lines: A12.
  *What it is:* Gives both directions between CFL_in(w) (Lyndon factorization for the inverse order) and ICFL(w): CFL_in(w) is the concatenation of the 'unbordered border' decompositions NB(m_i) of the ICFL factors, and ICFL is computed locally inside the maximal prefix-chains of CFL_in; closes by naming BWT variants built on inverse Lyndon words as open.
  *Record from:* Crossref record of the DOI; arXiv API record and arXiv v1 PDF (read: abstract, sections 1, 8, 9 statements, 10).
  *Not verified:* SAGE page not fetched (403); journal numbering not checked. The arXiv record does not itself carry the journal reference: the link arXiv <-> Computability is by identical title and authors only.
- **[Wols26]** Richard Wols. *Synchronous Combinatorial Games.*
  Master thesis, Utrecht University (Mathematical Sciences); advisor W. Kets, 2026.
  Versions and what was read: Utrecht University student theses repository, handle 20.500.12932/51048, deposited 12 Feb 2026, open access CC-BY-NC-ND (repository API record). Chapter 7 'Absolute-Lyndon words', section 7.1, pp. 85-86 of the PDF.
  Scholar lines: A14.
  *What it is:* A game-theory thesis; it cites Dolce-Restivo-Reutenauer's generalized Lyndon words once, in a survey paragraph of Lyndon-word variants said to have no direct connection with its own 'absolute-Lyndon words' (binary words whose rotations pair with complemented Lyndon words), and conjectures, without construction, a bijective-BWT variant from those. Nothing for the mission.
  *Record from:* Repository API record (metadata) and the thesis PDF (abstract and chapter 7 read).
- **[LZJ22]** Ruixi Luo, Taikun Zhu, Kai Jin. *String Rearrangement Inequalities and a Total Order Between Primitive Words (conference); extended journal version: Simple linear time algorithm for sorting strings in omega-order with applications.*
  IJTCS-FAW 2022 (Frontiers of Algorithmic Wisdom), LNCS 13461, pp. 206-218; Acta Informatica 63, article 17, 2022 (conference; first online 1 Jan 2023), 2026 (journal).
  Versions and what was read: Conference: Frontiers of Algorithmic Wisdom, IJTCS-FAW 2022, eds. Minming Li, Xiaoming Sun, LNCS vol. 13461, Springer, pp. 206-218, copyright 2022, first online 01 Jan 2023 (Springer chapter page; Crossref). Preprint: arXiv:2204.11213v1, 24 Apr 2022. Journal: Acta Informatica vol. 63, issue 2, article 17, published 15 Apr 2026, open access CC BY-NC-ND (Springer article page; issue number from Crossref); its publisher's note says it is an extended version of the IJTCS-FAW 2022 paper, which is why the lines are merged.
  DOI `10.1007/978-3-031-20796-9_15 (FAW); 10.1007/s00236-026-00530-5 (Acta Informatica)`; arXiv `2204.11213 (v1, the conference text; read in full). No arXiv record of the journal version was found (arXiv API searches by each author, 10 Nov 2026).`.
  Scholar lines: A17, A18, B9.
  *What it is:* Sorts n words by their infinite powers (omega-order) in O(L), L the total length, by reducing to an ordinary sort of the powers M_i = A_i^(deg+2); smallest/largest concatenation of n words is the omega-sorted order; the journal version adds O(L+sigma) and O(L+n log n) variants and a section saying the extended BWT sorts all conjugates, which this algorithm does not.
  *Record from:* Springer chapter page and Crossref (FAW); arXiv PDF v1 read in full; Springer article page of the Acta version, read through a fetch tool that returns a model's rendering of the page (three passes, with short verbatim quotes on the eBWT section); Crossref record.
  *Not verified:* Acta received/accepted dates not shown; the Acta lemma numbers are as relayed by the fetch tool, not read in the raw PDF (direct download met a browser check and was not forced). FAW conference dates/place not fetched. The books.google page itself was not opened; it is matched by title, authors and venue string.
- **[Postic20]** Mickaël Postic. *Local and Global Regularities in Infinite Words.*
  Ph.D. dissertation, 2020.
  Versions and what was read: Not read. The Mathematics Genealogy Project page (id 299538, fetched) gives: Ph.D. 2020, dissertation title, advisors Luca Zamboni and Gabriele Fici. No PDF found: math.univ-lyon1.fr/~postic/ is 404, HAL API has no record, theses.fr reset the connection.
  Scholar lines: A22.
  *What it is:* Thesis whose section 1.3.1 is 'Generalized Lyndon words' (per the Scholar snippet only); its Lyndon part is presumably the Postic-Zamboni paper, which I read instead.
  *Record from:* mathgenealogy.org/id.php?id=299538
  *Not verified:* University (the Scholar line and the task say Lyon; the genealogy page I parsed does not name it), URL, page count, and all content: thesis not fetched.
- **[GR-Hopf]** Darij Grinberg, Victor Reiner. *Hopf Algebras in Combinatorics.*
  arXiv preprint (lecture notes), 2020 (v7; v1 2014).
  Versions and what was read: arXiv:1409.8356v7, 28 Jul 2020, 282 pages (v1 30 Sep 2014). Read: Chapter 6 only, sections 6.1 (pp. 166-177) and 6.6.1 (pp. 205-210) in full; section 6.2 headings only.
  arXiv `1409.8356`.
  Scholar lines: B1.
  *What it is:* Chapter 6 proves the Chen-Fox-Lyndon theorem and the (right) standard factorization from scratch and states the Gessel-Reutenauer bijection with its link to the BWT and to the bijective BWT; it deliberately avoids Hall sets.
  *Record from:* arXiv API record; arXiv v7 PDF
- **[BKL]** Hideo Bannai, Dominik Köppl, Zsuzsanna Lipták. *A Survey of the Bijective Burrows-Wheeler Transform.*
  The Expanding World of Compressed Data: A Festschrift for Giovanni Manzini's 60th Birthday (eds. Ferragina, Gagie, Navarro), OASIcs 131, Schloss Dagstuhl, 2025.
  Versions and what was read: pp. 2:1-2:26 (one version found; author copy koeppl.github.io/bin/paper/festschrift25bbwt.pdf)
  DOI `10.4230/OASIcs.Manzini.2`.
  Scholar lines: B3.
  *What it is:* History, construction, indexing, runs r_B against r, other bijective variants, six open problems; inversion only by LF cycles; no Lyndon tree, grammar or interval exchange.
  *Record from:* drops.dagstuhl.de/entities/document/10.4230/OASIcs.Manzini.2 (meta tags) and the PDF, read in full
  *Not verified:* Scholar's other four versions not identified.
- **[BDZZ18]** Paola Bonizzoni, Clelia De Felice, Rocco Zaccagnino, Rosalba Zizza. *Inverse Lyndon words and inverse Lyndon factorizations of words.*
  Advances in Applied Mathematics 101, pp. 281-319, 2018.
  Versions and what was read: Journal: Adv. Appl. Math. 101 (Oct 2018) 281-319 (Crossref). Preprint: arXiv:1705.10277v7 [cs.FL], submitted 29 May 2017, v7 17 Dec 2017 (arXiv API; its record carries the journal reference and DOI). All theorem numbers below are those of arXiv v7.
  DOI `10.1016/j.aam.2018.08.005`; arXiv `1705.10277 (v7 read)`.
  Scholar lines: B4.
  *What it is:* Defines inverse Lyndon words (greater than every proper suffix), shows they are the sesquipowers of anti-Lyndon words (Lyndon words for the inverse alphabet order), and defines the canonical inverse Lyndon factorization ICFL(w): unique, linear time, a grouping of the Lyndon factorization for the inverse order.
  *Record from:* Crossref API record of the DOI (authors, volume, pages, date); arXiv API record and the arXiv v7 PDF (read: abstract, sections 1-4, 6.1, 7).
  *Not verified:* The Elsevier page was not fetched (403); the journal's own theorem numbering was not checked against arXiv v7.
- **[GMRRS20]** Raffaele Giancarlo, Giovanni Manzini, Antonio Restivo, Giovanna Rosone, Marinella Sciortino. *The Alternating BWT: An algorithmic perspective.*
  Theoretical Computer Science 812, pp. 230-243, 2020.
  Versions and what was read: Journal: Theor. Comput. Sci. vol. 812, pp. 230-243, April 2020 (copyright line 2019). Preprint: arXiv:1907.02308v1 [cs.DS], 4 Jul 2019, 28 pp., headed 'Preprint submitted to Elsevier'. It is the full version of GMRRS-DLT18 (the DLT paper says details 'will be given in the full paper'; this paper calls the DLT paper 'a preliminary version'), but it is a separate publication with new sections 6-7, so it is listed as its own work.
  DOI `10.1016/j.tcs.2019.11.002`; arXiv `1907.02308`.
  Scholar lines: B6.
  *What it is:* The ABWT's LF map reverses the order inside each letter's bucket (eq. (3)); backward search works; BWT and ABWT are the only rank-invertible BWT_K; ABWT computed in linear time via difference covers and a linear-time Galois-rotation algorithm.
  *Record from:* Crossref record of the DOI (title, authors, volume, pages, date); arXiv API entry and the PDF itself.
  *Not verified:* The journal text was not fetched (ScienceDirect answered 403). Read arXiv v1 in full. Numbering cross-checked only indirectly: GMRRS-IC23 cites 'Theorem 5.9' and 'Theorem 4.4' of the journal paper and DMS-VWORDS cites its 'Lemma 7.3' and 'Proposition 7.4', which match the arXiv numbering.
- **[BIKKP24]** Hideo Bannai, Tomohiro I, Tomasz Kociumaka, Dominik Koeppl, Simon J. Puglisi. *Computing Longest Lyndon Subsequences and Longest Common Lyndon Subsequences (journal); conference title: Computing Longest (Common) Lyndon Subsequences.*
  Algorithmica 86(3), pp. 735-756; conference IWOCA 2022, LNCS 13270, pp. 128-142, 2022 (conference), 2024 (journal issue; online 6 May 2023).
  Versions and what was read: Conference: Combinatorial Algorithms (IWOCA 2022), LNCS, pp. 128-142, online 29 May 2022 (Crossref; volume 13270 as given on the Algorithmica page). Preprint: arXiv:2201.06773v1, 18 Jan 2022; also Research Square preprint DOI 10.21203/rs.3.rs-2118135/v1 (2022, Crossref). Journal: Algorithmica vol. 86, issue 3, pp. 735-756, published online 6 May 2023, print March 2024, open access (Springer page, Crossref).
  DOI `10.1007/s00453-023-01125-z (Algorithmica); 10.1007/978-3-031-06678-8_10 (IWOCA)`; arXiv `2201.06773 (v1 read: abstract, sections 1-3)`.
  Scholar lines: B7.
  *What it is:* Dynamic programming for the longest subsequence of T that is a Lyndon word: O(n^3) time and O(n) space offline, O(n^3) time and space online in the journal version, O(n^4 sigma) for the common variant; no BWT, no Lyndon tree. Nothing for the mission.
  *Record from:* Crossref records of both DOIs; arXiv API and PDF v1; Springer article page (abstract, extension note) through the fetch tool.
  *Not verified:* IWOCA LNCS volume number is second-hand (Algorithmica page's reference).
- **[BDZZ20]** Paola Bonizzoni, Clelia De Felice, Rocco Zaccagnino, Rosalba Zizza. *Lyndon words versus inverse Lyndon words: queries on suffixes and bordered words (conference, arXiv) = On the longest common prefix of suffixes in an inverse Lyndon factorization and other properties (journal).*
  LATA 2020, LNCS 12038, pp. 385-396; journal version Theoretical Computer Science 862, pp. 24-41, 2020 (conference), 2021 (journal).
  Versions and what was read: Conference: Language and Automata Theory and Applications (LATA 2020), eds. Leporati, Martin-Vide, Shapira, Zandron, LNCS vol. 12038, Springer, pp. 385-396, first online 25 Feb 2020 (Springer chapter page). Journal: Theor. Comput. Sci. 862 (Mar 2021) 24-41 (Crossref). Preprint: arXiv:1911.01851v1, 2 Nov 2019, titled as the LATA paper; the arXiv record gives journal-ref 'Theoretical Computer Science, 2020' and the TCS DOI, which is why the two Scholar lines are merged as one work. Numbers below are arXiv v1's (the Springer page shows the bound as 'Proposition 6', Section 6, in the LATA text).
  DOI `10.1007/978-3-030-40608-0_27 (LATA); 10.1016/j.tcs.2020.10.034 (TCS)`; arXiv `1911.01851 (v1 read)`.
  Scholar lines: B8, B10.
  *What it is:* Proves that in ICFL(w) a nonempty border of a factor is never a prefix of the next factor, and that the longest common prefix of any two factors of w is at most M = max |m_i m_{i+1}| over consecutive ICFL factors; also how the Lyndon factorization of a factor of w reuses that of w.
  *Record from:* Springer chapter page (LATA volume, pages, editors, abstract, section list); Crossref record of the TCS DOI; arXiv API record and arXiv v1 PDF (read: abstract, sections 1, 3, 7, 8, 9).
  *Not verified:* The TCS text was not readable (ScienceDirect 403): that it is the journal version of the LATA paper rests on the arXiv record's journal-ref/DOI, not on the TCS page. LATA conference dates and place not shown on the page fetched.

### 2.2 Fetched to state things exactly (27)

- **[CR]** Maxime Crochemore, Luís M. S. Russo. *Cartesian and Lyndon trees (arXiv title: Cartesian trees and Lyndon trees).*
  Theoretical Computer Science, 2020.
  Versions and what was read: Journal: Theoretical Computer Science 806 (February 2020) 1-9. Preprint: arXiv:1712.08749v1 [cs.DS], 23 Dec 2017, 12 pp. (the version read).
  DOI `10.1016/j.tcs.2018.08.011`; arXiv `1712.08749`.
  *What it is:* Proposition 1 (u < v iff uw < w) makes the Lyndon tree the Cartesian tree of suffix ranks, and the Lyndon table the next-smaller table of the ranks; O(n) after suffix sorting; runs.
  *Record from:* Crossref API record of the DOI (fetched); arXiv abs page (fetched).
  *Not verified:* Journal text not read (403); locators are pages of arXiv v1. The DOI was first seen in the reference list of arXiv:1904.00954 and then confirmed at Crossref.
- **[HR]** Christophe Hohlweg, Christophe Reutenauer. *Lyndon words, permutations and trees.*
  Theoretical Computer Science, 2003.
  Versions and what was read: Theoretical Computer Science 307(1) (September 2003) 173-178.
  DOI `10.1016/S0304-3975(03)00099-9`.
  *What it is:* A note: the tree (standard bracketing) of a Lyndon word is the completed increasing tree of its suffix standard permutation.
  *Record from:* Crossref API (fetched: authors, volume 307, issue 1, pages 173-178, 2003-09); Semantic Scholar API record of the DOI (fetched: date 2003-09-26, and the article's opening paragraphs).
  *Not verified:* Full text NOT obtained: sciencedirect.com 403, core.ac.uk 403 (challenge page), web.archive.org 429 and resets. So the theorem's number and exact wording in the original are not read; what is reported is the article's own opening paragraphs as the Semantic Scholar API returns them, and the restatements in CR, DRR-var, BC, Ellert22 and BE23.
- **[Reut06]** Christophe Reutenauer. *Mots de Lyndon généralisés.*
  Séminaire Lotharingien de Combinatoire, 2006.
  Versions and what was read: Séminaire Lotharingien de Combinatoire 54 (2006), Article B54h, 16 pp. (received 6 May 2005, accepted 14 March 2006, final 20 April 2006). DRR cite it as "54:Art. B54h, 16, 2005/07".
  *What it is:* Generalized Lyndon words form a Hall set with a standard factorization (Th. 2.2), hence a bracketing and a Lie basis; section 3.4 states the circular Schützenberger–Melançon rewriting that finds the Hall conjugate of a word.
  *Record from:* The journal's page for B54h and its PDF (both fetched; read sections 1, 2, 3.1-3.4, 3.7, 3.8 and the references).
  *Not verified:* Sections 3.5 and 3.6 (quadratic forms, Markoff numbers) only skimmed.
- **[Ellert22]** Jonas Ellert. *Lyndon Arrays Simplified.*
  30th Annual European Symposium on Algorithms (ESA 2022), LIPIcs 244, 2022.
  Versions and what was read: LIPIcs vol. 244, pp. 48:1-48:14.
  DOI `10.4230/LIPIcs.ESA.2022.48`.
  *What it is:* Simplified O(n) Lyndon array on general alphabets; its section 2 states the next/previous-smaller-suffix lemmas in the form used below.
  *Record from:* DROPS landing page (fetched) and PDF.
  *Not verified:* Only section 2 (p. 48:3: Definition 2, Lemmas 3 and 4) was read.
- **[BE23]** Hideo Bannai, Jonas Ellert. *Lyndon Arrays in Sublinear Time.*
  31st Annual European Symposium on Algorithms (ESA 2023), LIPIcs 274, 2023.
  Versions and what was read: LIPIcs vol. 274, pp. 14:1-14:16.
  DOI `10.4230/LIPIcs.ESA.2023.14`.
  *What it is:* The succinct 2n-bit Lyndon array (the Lyndon tree's information) of a packed string in O(n / log_σ n) time on a word RAM.
  *Record from:* DROPS landing page (fetched) and PDF.
  *Not verified:* Abstract and introduction only (pp. 14:1-14:3).
- **[GilScott]** Joseph Yossi Gil, David Allen Scott. *A Bijective String Sorting Transform.*
  arXiv preprint, 2012.
  Versions and what was read: arXiv only; the survey (p. 2:2) says it was never published in a journal or proceedings
  DOI `10.48550/arXiv.1201.3077`; arXiv `1201.3077 (v1, 15 Jan 2012)`.
  *What it is:* Defines the transform on the Lyndon factorization; inverse by Match + MultiThread: follow each cycle from the smallest unvisited row, n steps.
  *Record from:* arxiv.org/abs/1201.3077 and the PDF, read in full
  *Not verified:* Section 4 prints the factor order as ω0 < ω1 < …, which contradicts its own section 5 (cycle of row 0 gives the last factor); read as a slip.
- **[MRRS07]** Sabrina Mantaci, Antonio Restivo, Giovanna Rosone, Marinella Sciortino. *An extension of the Burrows–Wheeler Transform.*
  Theoretical Computer Science 387(3), 2007.
  Versions and what was read: pp. 298-312 (Nov 2007)
  DOI `10.1016/j.tcs.2007.07.014`.
  *What it is:* eBWT of a multiset of words; abstract: a bijection between words and finite multisets of conjugacy classes of primitive words.
  *Record from:* api.crossref.org/works/10.1016/j.tcs.2007.07.014; abstract from iris.unipa.it/handle/10447/26745
  *Not verified:* Full text NOT read (ScienceDirect 403, Elsevier API 401, no open copy on CORE). Theorem 20, Remark 12, Definition 4, Proposition 15, Section 3 are known here only as cited by BKKP-bbwt and Koeppl20. First names from the survey's reference list.
- **[GR93]** Ira M. Gessel, Christophe Reutenauer. *Counting permutations with given cycle structure and descent set.*
  Journal of Combinatorial Theory, Series A 64(2), 1993.
  Versions and what was read: pp. 189-215 (Nov 1993)
  DOI `10.1016/0097-3165(93)90095-P`.
  *What it is:* Source of the bijection between words and multisets of primitive necklaces; here stated only through CDP05, Kufleitner09 and the survey.
  *Record from:* api.crossref.org/works/10.1016/0097-3165(93)90095-P
  *Not verified:* Full text NOT read (publisher 403/401; author's page TLS failure and 503). No theorem number of the paper itself is given in this report.
- **[CDP05]** Maxime Crochemore, Jacques Désarménien, Dominique Perrin. *A note on the Burrows-Wheeler transformation.*
  Theoretical Computer Science 332(1-3), 2005.
  Versions and what was read: arXiv v1 17 Feb 2005 (read); journal pp. 567-572 as in the survey's reference [21]
  DOI `10.1016/J.TCS.2004.11.014`; arXiv `cs/0502073`.
  *What it is:* BWT as a special case of the Gessel-Reutenauer bijection; states the general case: cycles of the permutation correspond to the Lyndon factors of the word.
  *Record from:* arxiv.org/abs/cs/0502073 and its PDF; journal data from the survey's reference list
  *Not verified:* Journal volume/pages not checked at the publisher.
- **[Kuf09]** Manfred Kufleitner. *On Bijective Variants of the Burrows-Wheeler Transform.*
  Proceedings of the Prague Stringology Conference 2009, 2009.
  Versions and what was read: arXiv v1 3 Aug 2009, 15 pages (read); PSC 2009 pp. 65-79 as in the survey's reference [47]
  arXiv `0908.0239`.
  *What it is:* Proof that the bijective BWT is invertible: cycles of the standard permutation ordered by smallest element spell the Lyndon factors; removing a cycle leaves the BWTS of the remaining factors.
  *Record from:* arxiv.org/abs/0908.0239 and its PDF
  *Not verified:* PSC pagination from the survey only.
- **[Koeppl20]** Dominik Köppl, Daiki Hashimoto, Diptarama Hendrian, Ayumi Shinohara. *In-Place Bijective Burrows-Wheeler Transforms.*
  31st Annual Symposium on Combinatorial Pattern Matching (CPM 2020), LIPIcs 161, 2020.
  Versions and what was read: pp. 21:1-21:15; arXiv:2004.12590 (27 Apr 2020)
  DOI `10.4230/LIPIcs.CPM.2020.21`; arXiv `2004.12590`.
  *What it is:* BBWT built and inverted in place in quadratic time; RLBBWT <-> RLBWT in O(n lg r / lg lg r) time and O(r lg n) bits by reading one cycle at a time and deleting it from the run lengths.
  *Record from:* drops.dagstuhl.de/entities/document/10.4230/LIPIcs.CPM.2020.21; arxiv.org/abs/2004.12590
  *Not verified:* arXiv's related-DOI field reads 10.4230/LIPIcs.CPM.2020.23; DROPS and the PDF give .21.
- **[BBIK]** Golnaz Badkobeh, Hideo Bannai, Tomohiro I, Dominik Köppl (conference version: Badkobeh, Bannai, Köppl). *Bijective BWT Based Compression Schemes.*
  Theory of Computing Systems 70, article 42, 2026.
  Versions and what was read: SPIRE 2024, LNCS 14899, pp. 16-25, doi 10.1007/978-3-031-72200-4_2 (3 authors); arXiv:2406.16475 (24 Jun 2024, "slightly extended version"); Theory Comput. Syst. 70(3):42, online 3 Jul 2026, 16 pages (4 authors; adds Theorem 4)
  DOI `10.1007/s00224-025-10235-w`; arXiv `2406.16475`.
  *What it is:* r_B = O(z log^2 n); r = 2 with r_B = Ω(log n); Lyndon factorizations of all rotations in O(n) from the right and left Lyndon trees; optimal rotation in Õ(nh).
  *Record from:* api.crossref.org for both DOIs; arxiv.org/abs/2406.16475; author copy of the journal PDF (read)
  *Not verified:* Springer landing page not read (login redirect); journal text from the author's copy.
- **[BIN25]** Hideo Bannai, Tomohiro I, Yuto Nakashima. *On the Compressiveness of the Burrows-Wheeler Transform.*
  36th Annual Symposium on Combinatorial Pattern Matching (CPM 2025), LIPIcs 331, 2025.
  Versions and what was read: pp. 17:1-17:15
  DOI `10.4230/LIPIcs.CPM.2025.17`.
  *What it is:* r = Ω(r_B log n) on a family; r_B and r within O(log^4 n) of each other; LF on Fibonacci words as a rotation of Zeckendorf digits.
  *Record from:* drops.dagstuhl.de/entities/document/10.4230/LIPIcs.CPM.2025.17
- **[CLPRS25]** Davide Cenzato, Zsuzsanna Lipták, Nadia Pisanti, Giovanna Rosone, Marinella Sciortino. *BWT for string collections.*
  arXiv preprint (the survey cites it as OASIcs 131, pp. 3:1-3:29, 2025), 2025.
  Versions and what was read: arXiv v1 1 Jun 2025 (read: sections 2 and 3)
  arXiv `2506.01092`.
  *What it is:* Definition of the omega-order with exponents, of the EBWT of a multiset, and of its inversion from one index per cycle.
  *Record from:* arxiv.org/abs/2506.01092
  *Not verified:* OASIcs pagination from the survey's reference [18] only.
- **[Kempa19]** Dominik Kempa. *Optimal Construction of Compressed Indexes for Highly Repetitive Texts.*
  arXiv preprint; related DOI is the SODA 2019 proceedings, 2017.
  Versions and what was read: abstract only
  DOI `10.1137/1.9781611975482.82`; arXiv `1712.04886`.
  *What it is:* Abstract: BWT, PLCP, LZ77 and also Lyndon factorization in O(n/log_σ n + r polylog n), from the text.
  *Record from:* arxiv.org/abs/1712.04886
  *Not verified:* Only the abstract was read; venue name not on the page fetched.
- **[LL25]** Felipe A. Louza, Arnaud Lefebvre. *In-Place BWT and Lyndon Array Construction in Constant Space.*
  arXiv preprint, 2025.
  Versions and what was read: abstract only (v1, 24 Dec 2025)
  arXiv `2512.20869`.
  *What it is:* Abstract: Lyndon array from suffix ranks kept during in-place BWT construction, by next-smaller-value; quadratic time; from the text.
  *Record from:* arxiv.org/abs/2512.20869
  *Not verified:* Only the abstract was read.
- **[GMRRS23]** Raffaele Giancarlo, Giovanni Manzini, Antonio Restivo, Giovanna Rosone, Marinella Sciortino. *A new class of string transformations for compressed text indexing.*
  Information and Computation 294, article 105068, 2023.
  Versions and what was read: Background, fetched to answer question (b). Journal: Inf. Comput. vol. 294, art. 105068, Oct 2023. arXiv:2205.05643v2 (v1 11 May 2022). Preliminary version: R. Giancarlo, G. Manzini, G. Rosone, M. Sciortino, 'A New Class of Searchable and Provably Highly Compressible String Transformations', CPM 2019, LIPIcs 128, 12:1-12:12, DOI 10.4230/LIPIcs.CPM.2019.12, arXiv:1902.01280 (four authors).
  DOI `10.1016/j.ic.2023.105068`; arXiv `2205.05643`.
  *What it is:* Context-adaptive BWTs are invertible in O(sigma n^2); those with 'local orderings' have an order-preserving LF on rows with equal first and last symbols, invert in linear time, admit an r-index, and are an ordinary BWT over pairs; a linear-time lower bound on runs over all these variants.
  *Record from:* Crossref (journal record); DROPS meta tags (CPM 2019 record); arXiv API entries and the arXiv v2 PDF.
  *Not verified:* Read arXiv v2 (sections 2-6), not the journal PDF.
- **[GRR12]** Ira M. Gessel, Antonio Restivo, Christophe Reutenauer. *A bijection between words and multisets of necklaces.*
  European Journal of Combinatorics 33(7), pp. 1537-1546, 2012.
  Versions and what was read: Background named in the task. One version found: Eur. J. Combin. 33(7), 1537-1546, Oct 2012. Full text NOT obtained (ScienceDirect 403; the Palermo repository copy is restricted).
  DOI `10.1016/j.ejc.2012.03.016`.
  *What it is:* Abstract only (seen through a search-result summary): replacing the standard permutation by the co-standard one in Gessel-Reutenauer's map gives a bijection whose inverse sorts by the alternating order and whose image is a special set of multisets of necklaces, not all primitive.
  *Record from:* Crossref record of the DOI.
  *Not verified:* Everything beyond the bibliographic record. What its Section 3 proves is taken from Elizalde-25, section 2.2, a secondary source.
- **[Eliz25]** Sergi Elizalde. *A bijection for descent sets of permutations with only even and only odd cycles.*
  arXiv preprint [math.CO], 2025.
  Versions and what was read: arXiv:2503.09972v2, 7 Apr 2025. Used only as a secondary source for the statement of GRR-EJC12, Section 3.
  arXiv `2503.09972`.
  *What it is:* Restates the Gessel-Reutenauer bijection and the alternating variant of Gessel-Restivo-Reutenauer, with the exact description of the image of the latter.
  *Record from:* The arXiv PDF.
  *Not verified:* Venue of any published version not looked up.
- **[CPS19]** Émilie Charlier, Manon Philibert, Manon Stipulanti. *Nyldon words.*
  Journal of Combinatorial Theory, Series A 167, 2019.
  Versions and what was read: J. Combin. Theory Ser. A 167 (October 2019) 60-90; arXiv:1804.09735, v1 25 Apr 2018, v2 30 Jan 2019, 28 pages. Read: arXiv v2 (page numbers below are v2's).
  DOI `10.1016/j.jcta.2019.04.002`; arXiv `1804.09735`.
  *What it is:* Nyldon words (Grinberg's reversal of the Chen-Fox-Lyndon recursion) factor the free monoid uniquely and are a right Lazard = right Hall set; the paper's sections 10-12 are an accessible statement of left/right Lazard sets, Viennot sets, both standard factorizations of Lyndon words and Melançon's algorithm.
  *Record from:* Crossref API record; arXiv API record; arXiv v2 PDF
- **[PR18]** Dominique Perrin, Christophe Reutenauer. *Hall sets, Lazard sets and comma-free codes.*
  Discrete Mathematics 341(1), 2018.
  Versions and what was read: Discrete Math. 341(1) (January 2018) 232-243; arXiv:1609.05438v1, 18 Sep 2016. Read: arXiv v1 sections 1, 2, 7, 8 (page numbers below are v1's).
  DOI `10.1016/j.disc.2017.08.034`; arXiv `1609.05438`.
  *What it is:* States Hall sets and Lazard sets in Viennot's (left) convention, shows the Lyndon words as a left Lazard set whose trees are the LEFT standard factorizations, and gives the left version of Melançon's algorithm on a cyclic word.
  *Record from:* Crossref API record; arXiv API record; arXiv v1 PDF
- **[Mel92]** Guy Melançon. *Combinatorics of Hall trees and Hall words.*
  Journal of Combinatorial Theory, Series A 59(2), 1992.
  Versions and what was read: J. Combin. Theory Ser. A 59(2) (March 1992) 285-308; Zbl 0761.05033. NOT READ: ScienceDirect and core.ac.uk answered 403. Known here only through the zbMATH review and through what Reutenauer 2006 s.3.4, Perrin-Reutenauer s.8, Charlier et al. s.12 and Remark 60, and Garg Theorem 3.18 attribute to it.
  DOI `10.1016/0097-3165(92)90070-B`.
  *What it is:* Per the zbMATH review: new combinatorial proofs of the unique factorization into Hall words by a rewriting system, and an order on the free monoid under which Hall words behave like Lyndon words.
  *Record from:* Crossref API record; zbMATH Open API record Zbl 0761.05033
  *Not verified:* All statements inside the paper (definitions of rise / legal rise, confluence theorem, the circular-word algorithm, any step count).
- **[Reut93]** Christophe Reutenauer. *Free Lie Algebras.*
  London Mathematical Society Monographs, New Series 7, Clarendon Press, Oxford, 1993.
  Versions and what was read: xvii + 269 pp. (zbMATH Zbl 0798.17001). Chapter page ranges from Crossref: 0 Introduction 1-13; 4 Hall bases 84-104; 5 Applications of Hall sets 105-126; 7 Circular words 154-175. NOT READ: only the publisher's chapter abstracts (Crossref) and the zbMATH review by D. Krob. Theorem numbers used below are as cited in papers I read: Cor. 4.5, Cor. 4.7, Th. 4.9 (Reutenauer 2006), Prop. 4.1, Theorem 4.18 (Perrin-Reutenauer), Theorem 7.20 (Grinberg-Reiner).
  DOI `10.1093/oso/9780198536796.001.0001`.
  *What it is:* Chapter 0: Lazard elimination and Lazard sets; chapter 4: Hall trees, a confluent rewriting system on standard sequences, and Lazard sets = Hall sets (Viennot); chapter 5: Lyndon words as a Hall set; chapter 7: circular words and the bijection words <-> multisets of primitive necklaces.
  *Record from:* Crossref API records of the book and of chapters 003.0001-003.0009; zbMATH Open API record
  *Not verified:* Every in-book statement and its numbering beyond the abstracts, the review and the citations named.
- **[Viennot78]** Gérard Viennot. *Algèbres de Lie libres et monoïdes libres. Bases des algèbres de Lie libres et factorisations des monoïdes libres.*
  Lecture Notes in Mathematics 691, Springer, 1978.
  Versions and what was read: Lecture Notes in Mathematics 691, Springer, Berlin-Heidelberg-New York, 124 p. (zbMATH Zbl 0395.17003). Chapters per Crossref include 'Bases des algèbres de Lie libres' pp. 9-61 and 'Factorisations des monoïdes libres et familles basiques des algèbres de Lie libres' pp. 94-112. NOT READ; used only as cited (p. 14-15 for the left standard factorization, by Perrin-Reutenauer and Dolce-Restivo-Reutenauer; Theorems 51-52 of Charlier et al.).
  DOI `10.1007/BFb0067950`.
  *What it is:* Origin of Lazard sets, Hall sets in the general sense, regular (Viennot) factorizations and the left standard factorization.
  *Record from:* Crossref API record of the book and two chapters; zbMATH Open API record
  *Not verified:* All content (not fetched).
- **[Garg21]** Swapnil Garg. *New results on Nyldon words and Nyldon-like sets.*
  Advances in Applied Mathematics 131, article 102249, 2021.
  Versions and what was read: Adv. in Appl. Math. 131 (October 2021) 102249; arXiv:1908.04056v3 (29 Oct 2022). Scanned: abstract, section 3 (Melançon's algorithm), headings of sections 5-6.
  DOI `10.1016/j.aam.2021.102249`; arXiv `1908.04056`.
  *What it is:* Restates Melançon's contraction algorithm for any right Hall set (conjugate and factorization versions) and studies how long the right Lazard procedure takes to produce the Nyldon words.
  *Record from:* Crossref API record; arXiv API record; arXiv v3 PDF
- **[BDRZZ24]** Paola Bonizzoni, Clelia De Felice, Brian Riccardi, Rocco Zaccagnino, Rosalba Zizza. *Unveiling the Connection Between the Lyndon Factorization and the Canonical Inverse Lyndon Factorization via a Border Property.*
  MFCS 2024, LIPIcs 306, 31:1-31:14, 2024.
  Versions and what was read: 49th International Symposium on Mathematical Foundations of Computer Science (MFCS 2024), LIPIcs vol. 306, article 31, pp. 31:1-31:14, published 23 Aug 2024 (DROPS page). arXiv:2406.18473v2.
  DOI `10.4230/LIPIcs.MFCS.2024.31`; arXiv `2406.18473`.
  *What it is:* ICFL is the unique inverse Lyndon factorization with the border property, and is obtained in linear time by compacting factors of CFL_in (abstract only).
  *Record from:* DROPS document page; arXiv API record. Abstract only.
  *Not verified:* Body not read.
- **[DNSZZ26]** Clelia De Felice, Pietro Negri, Manuel Sica, Rocco Zaccagnino, Rosalba Zizza (arXiv:2609.23401), earlier version by Pietro Negri, Manuel Sica, Rocco Zaccagnino, Rosalba Zizza (arXiv:2603.17537). *The Inverse Lyndon Array (2609.23401); The Inverse Lyndon Array: Definition, Properties, and Linear-Time Construction (2603.17537).*
  arXiv preprints (comment on 2609.23401: 'SOFSEM conference'), 2026.
  Versions and what was read: arXiv:2609.23401v1, sections 1-3 read (Lemmas 2, 6, 7, 8, Corollary 1); arXiv:2603.17537v1 skimmed (Lemma 7, Lemma 10).
  arXiv `2609.23401v1 (20 Sep 2026); 2603.17537v1 (18 Mar 2026)`.
  *What it is:* For the inverse setting, the Lyndon array for the inverse order equals the distance to the next GREATER suffix, the next/previous-greater-suffix edges are non-crossing, and the inverse Lyndon array is that distance plus an LCE (border) term; O(n) on general ordered alphabets; ICFL from it in O(n). No tree is named.
  *Record from:* arXiv API records and PDFs.
  *Not verified:* Whether the two arXiv records are versions of one paper (author lists differ) is my reading of titles and abstracts; no proceedings record fetched.


## 3. In print (K), by what it bears on

Read by five sub-agents, one cluster of works each, every statement with the place it was
read at.
Keys are §2's. "Not in print" means: not in the works read, which §7 bounds.

**3.1 The tree.**
- The tree of a Lyndon word, its standard bracketing, is the increasing tree of its suffix
  standard permutation: [HR] (its opening paragraphs; the body was not obtained). Restated
  with a proof from suffix ranks: [CR] Proposition 1 and §4 (the Lyndon factorization is the
  left-to-right minima of the ranks; the longest Lyndon factor at i ends at the next smaller
  rank).
- Always for the suffixes of a word. **No work read states it for rotation ranks or for
  the rows of a BWT, builds a Lyndon tree, array or grammar from a BWT or a run-length BWT,
  or builds one below linear time except from the packed text** (O(n / log_σ n): [BE23],
  abstract and introduction). [Koeppl20] §4.3 gets the Lyndon *factorization* from a
  run-length BWT by Duval's algorithm on n forward steps.
- [CR] §6: the tree labelled by ranks is "richer" than the Lyndon tree; the labels seem hard
  to recover without sorting the suffixes.
- The right Lyndon tree gives the Lyndon factorization of every suffix as the right
  children hanging off a leaf's path to the root: [BBIK] §4, Theorem 3 and its proof. So a
  stretch's forward minima (entry 1, §5.5) count Lyndon factors.

**3.2 The right end, read in the text, is the Schützenberger–Melançon rewriting.**
- [Reut06] §3.4, crediting [Mel92] and Schützenberger: take the letters of w as a circular
  sequence of Hall words. A *rise* is a pair of neighbours h_i < h_{i+1}; it is *legal* when
  h_{i+1} is at least every word of the sequence. Merge a legal rise; repeat. What is left
  is the Hall conjugate of w (a power of it if w is not primitive), with its tree. The same
  algorithm: [PR18] §8 (its left form), [CPS19] §12 and Algorithm 3, [Garg21] Theorem 3.18.
  [Mel92] itself was not obtained.
- Lyndon words are the Hall set in question: the right Lazard set that removes the greatest
  element, in decreasing order, and the left Lazard set that removes the least ([CPS19]
  Definition 47, Examples 48–49; "the choice of the words … determines the total order",
  Remark 50). Hall sets and Lazard sets coincide (Viennot; [Reut93] Theorem 4.18, as cited).
- The tree is the standard bracketing, a basis of the free Lie algebra: [GR-Hopf] Theorem
  6.1.30, Definition 6.1.38; [Reut06] Corollaire 2.3 for every generalized order.
- Steps of such rewritings are counted in print only as: k − 1 eliminations for the k Lyndon
  words of length ≤ n; Algorithm 3's worst case O(n²) ([CPS19] Remark 58, Open Problem 59,
  §12). **No work read takes all occurrences of a rise at once, relates a count to the runs
  of a BWT, or gives a fewest-steps form.**

**3.3 Every piece is a Lyndon word.** With next[i] and prev[i] the nearest smaller suffixes
to the right and left of i: x[i .. next[i]) is the longest Lyndon prefix of the suffix at i,
and x[prev[i] .. i) is a Lyndon word ([Ellert22] §2, Lemmas 3 and 4, credited there to
Hohlweg–Reutenauer, Franek et al. and Bille et al.). Entry 1's pieces at prefix m are of one
or the other kind.

**3.4 Any column: the extended and the bijective BWT.**
- Every word is the image of exactly one multiset of aperiodic necklaces: sort all rotations
  of all necklaces by their infinite powers, take last letters; the cycles of the inverse
  standard permutation spell the necklaces ([GR-Hopf] Definitions 6.6.12, 6.6.26, Theorem
  6.6.29, Remark 6.6.31, crediting [GR93] and [Reut93] Theorem 7.20; [CDP05] §4). The BWT is
  the one-necklace case; the bijective BWT is the multiset of T's Lyndon factors ([BKL] §2.1;
  [Kuf09]).
- **Inversion by cycles, in print:** the cycles taken by increasing smallest row spell the
  Lyndon factors last to first ([GilScott] §5, Algorithms 5.1–5.2; [Kuf09] Lemma 7 and
  Theorem 8, whose proof deletes a cycle and finds a valid transform left; [BKKP] §4.6). All
  of them walk LF or FL for n steps. Run-length to run-length: [Koeppl20] Theorem 3, n
  backward steps, O(n lg r / lg lg r). **Neither the survey (2025) nor the journal article
  (2024) has an inversion in fewer than n steps.**
- A column is the BWT of some word iff the number of cycles of its standard permutation
  equals the gcd of its run lengths (Likhomanov–Shur, as [BKL] p. 2:4 states it).
- Runs: r_B is "roughly equal" to r on the standard corpora ([BKKP] Table 6); r_B = O(r log⁴
  n) and r = O(r_B log⁴ n), with families a factor log n apart both ways ([BIN25]; [BBIK]
  Theorem 2).
- The bijective BWT has two runs iff T = b^k a^l or T is a power of the Lyndon rotation of
  a standard word ([BKL] Theorem 24).
- The survey's open problems ([BKL] p. 2:22; six, unnumbered) and the journal article's
  ([BKKP] §7): construction in O(n / log_σ n); matching in O(m) rank operations; which
  rotation has the fewest runs; a bijective BWT on other factorizations (generalized Lyndon,
  Galois, inverse Lyndon, Nyldon). None asks for a faster inversion.

**3.5 The left tree.** The left standard factorization (the longest proper prefix that is a
Lyndon word) is Viennot's and Širšov's ([DRR-var] Proposition 26; [PR18] Example 1). Its
tree is the decreasing Cartesian tree of the proper prefixes ranked by their infinite powers
([DRR-var] Theorem 36), built in O(n) letter comparisons, the ranks being the left-to-right
postorder ([BC] Proposition 4, Theorem 6). [BC] §7 asks: "Is there a tight relation between
the left Lyndon trees and the right Lyndon trees?" (Entry 1's §5.6 credited "another tree" to
Dolce–Restivo–Reutenauer: the factorization is Viennot's; theirs is the Cartesian form.)

**3.6 Other orders.**
- A generalized lexicographic order takes an alphabet order for each position; its Lyndon
  words are a Hall set, with standard factorization "v the proper suffix of least v^∞"
  ([Reut06] Théorème 2.2; [DRR-gen] Definitions 4 and 12, Theorem 14). Galois words are the
  alternating case. They can be bordered and are not closed under ordered product (121, 3,
  1213: [Reut06] §3.8), so they are no Viennot factorization.
- Of the transforms that sort by a periodic sequence of alphabet orders, **only the BWT and
  the alternating BWT are inverted from ranks alone** ([GMRRS20] Definition 5.2, Theorem 5.9). In the alternating BWT
  equal letters lie in L in the reverse of their order in F ([GMRRS20] Proposition 3.2(3b),
  equation (3)). A second family that works: orders that depend on the last k symbols of
  the common prefix ([GMRRS23] §4, Lemma 17).
- Galois words: detection, factorization and rotation in O(n) time ([HKYS24]). Circ-UMFFs
  hold one rotation of every primitive word, all border-free ([DMS] Theorem 22); Galois
  words are not one ([DMS] §7). A reader's caution: [DMS] Theorem 46 ("for Galois u, v with
  uv primitive, u < v iff uv is Galois") is contradicted by aba, c (the example [Reut06] and
  [HKYS24] print), 80 pairs over three letters to length 5.
- ω-Lyndon words for any order on infinite words with a lexicographic-like condition
  ([PZ20]); generalized Lyndon factorizations of infinite words ([BW19]); Nyldon words, a
  right Hall set for the other extremal Lazard choice ([CPS19]); generalised Nyldon words
  ([FHN25], abstract only). No tree, transform or algorithm in them.
- Formalized in Isabelle/HOL: the basics, the factorization's existence and uniqueness, the
  existence of a factorization into two Lyndon words; no tree ([HS21], from the AFP
  outline).

**3.7 Inverse Lyndon words, sorting in the infinite order, Lyndon subsequences.**
- An anti-Lyndon word is a Lyndon word for the inverse alphabet order: the greatest
  rotation of a primitive word ([BDZZ18] Definition 2.3). Inverse Lyndon words are the
  non-empty prefixes of powers of anti-Lyndon words; they may be bordered and need not be
  primitive ([BDZZ18] Definition 3.1, §4.1). The canonical inverse Lyndon factorization is
  unique, linear-time, and groups the Lyndon factors for the inverse order ([BDZZ18]
  Definitions 4.2 and 4.3; back and forth: [BDZZ26] Proposition 8.2). No tree of an
  anti-Lyndon word, no standard factorization of one, and no greatest rotation as a root
  is in these works; the nearest is the Lyndon array for the inverse order as distances to
  the next greater suffix ([DNSZZ26] Lemma 2, a 2026 preprint).
- The longest common prefix of two factors of w is at most the longest product of two
  consecutive factors of that factorization ([BDZZ20] Proposition 7.5, Lemma 9.6). The
  reader's answer on its use here: none. A step compares two lengths, not two strings; and
  on block-like words the bound is of the order of n (12,966 of 20,000 bytes on a text
  sample whose longest repeat is 34; its count).
- [BDZZ26] §10 names as open "new bijective variants of the Burrows Wheeler Transform,
  based on multisets of inverse words".
- Sorting n words by their infinite powers in O(L) time, L their total length ([LZJ22]):
  A^ω < B^ω iff AB < BA; on Lyndon words that order is the lexicographic one. It sorts the
  given words, not their rotations, and its authors say it is not the extended BWT's sort.
- A second-hand note: [DNSZZ26]'s introduction says Louza, Smyth, Manzini and Telles (JDA
  50, 2018, which entry 1 cites for the walk) get the Lyndon array "as a byproduct of the
  inversion of the bijective" BWT. If so that is the nearest work to §4.2, by a walk of n
  steps; the paper itself was not reread here.
- Nothing for the mission in the longest Lyndon subsequence ([BIKKP24]: O(n³) dynamic
  programming) or in [Wols26], which cites the root paper in one background sentence.

## 4. Derived while reading (P + C; `lyndon_induction_tests.py.txt`, about two minutes)

Notation is entry 1's (§1, §2): π = LF on a column L; letters with a top interval I_α, an
image J_α, a length λ_α and a word w_α; a right-handed step at a coincidence takes A (the
top interval ending there) and B (the image ending there) and names N → w_A w_B. A *hinge*
is a text position whose byte a run-start row of L emits (entry 1, Theorem 3).

**4.1 Theorem 5 (P; C): the steps are the tree's, not the order's.** In any sequence of
right-handed steps, from any column:

- **(a)** every letter, at every moment, has exactly one row whose occurrence of the
  letter's word holds a hinge: its first row;
- **(b)** so each step makes exactly one inner node above a hinge, and
  **steps = the inner nodes of the derivation forest that have a hinge leaf below them.**

*Proof.* A row x of a letter α stands for one occurrence of w_α in the text: the bytes the
rows x, πx, …, π^{|w_α|−1}x emit. At the start a letter is a run and its word one byte: the
occurrence at x holds a hinge iff x is the run's start, its first row. A step, with the
cell's rows read as rows of I_B: the occurrence of w_A w_B at x is B's at x and then A's at
π_m x.
- Top wins (λ_A > λ_B). The new letter is all of I_B, carried onto the last λ_B rows of
  I_A, which do not include A's first row. Its occurrence at x holds a hinge iff x is B's
  first row: one row, the first. A keeps its first rows, its first among them.
- Bottom wins (λ_A < λ_B). The new letter is the last λ_A rows of I_B, which do not include
  B's first row, carried in order onto all of I_A. Its occurrence at x holds a hinge iff
  π_m x is A's first row, iff x is the cell's first row. B keeps its first row.
- Equal. The new letter is I_B carried in order onto I_A: B's first row goes to A's first
  row. One row holds a hinge (two of them).
A step makes one new inner node an occurrence, that is, a row of its cell; by (a) one of
them is above a hinge. Nothing in this used where the coincidence lies. ∎

- **C:** every path of right-handed steps on the BWT of every Lyndon word over 2 letters to
  length 11, 3 to 7 and 4 to 6: 1,865 words, 5,255,519 states, 2,792,616 final states. The
  number of steps is a function of the state, and equals the final tree's inner nodes above
  a hinge on every one.
- **It contains entry 1's Theorem 3** (the right end: S = the Lyndon-tree nodes on the
  hinges' paths), with a proof that needs neither §5.5's count nor Theorem 1.
- **Equal steps are the nodes with a hinge on both sides; a cut step has its hinges on one
  side.** r hinges need r − 1 of the first kind in any tree of one cycle. So the freedom of
  entry 1's Theorem 4 (steps = r₂ − 1 + U) is all in the second kind: U + (r₂ − r) nodes
  that hang hinge-free text onto a hinge's path. r₂ − r is the number of stretches of more
  than one row (entry 1, §5.5: 0.220 r on Wikidata): the floor is one such node a stretch,
  and it is met when a stretch's inside is bracketed whole, as the other occurrences of
  steps whose hinged occurrence lies elsewhere, before it is hung on.
- **Corollary (P): the Lyndon tree costs S whatever the order.** No order of right-handed
  steps that leaves the Lyndon tree takes fewer steps than the right end. So every saving
  entry 1 counted (1.279 r against 1.812 r) changes the grammar: it is not the Lyndon tree
  any more, and nothing that relies on Lyndon words (Theorem 2's corollary, the Lyndon SLP)
  carries over to it. **C:** on 725 Lyndon words (binary to 10, ternary to 7) the fewest
  steps over all orders that leave the Lyndon tree is the right end's count on each; the
  fewest over all orders is 2.7% less in sum (4,087 against 4,201; 4,071 with left-handed
  steps allowed too).
- **Mixed hands have no such formula (C).** With steps at the start of an interval allowed
  too, the count is neither tree's-nodes-above-run-starts nor above-run-ends (249,156
  (word, tree) pairs: 227,286 and 227,286; the larger of the two on 246,384). Left-handed
  steps alone are the mirror: the hinges are the bytes the run-END rows emit.

**4.2 Corollary A (P; C): the induction inverts the extended and the bijective BWT.**
Entry 1's §2 runs the induction on "a column that is no BWT" and drops what closes
unmarked. What it drops:

- Every column L is the extended BWT of exactly one multiset M of primitive necklaces
  (**K:** §3.4): row x is a rotation ρ_x of a word of
  M, the rows in the order of the infinite words ρ_x^ω, and π carries a row to the row of
  the rotation one place earlier. The cycles of π are the necklaces.
- A letter closes (A = B) when it is the last on both orders: I_A = J_A = [m − λ_A, m). Each
  of its rows is then the only row of its cycle left in [0, m), so it is the least row of
  that cycle: ρ_x is the least rotation of its necklace, a Lyndon word, and by Invariant I
  w_A is that word. Cycles close by decreasing least row.
- **So the closed letters, in the order they close, are the Lyndon words of M, largest
  first, each once with λ = its multiplicity; and the tree of each is its Lyndon tree.**
- **On the bijective BWT of T** (**K:** the extended BWT of T's Lyndon factors, §3.4) **the closed
  words, in order, with their multiplicities, are the Lyndon factorization of T:** the
  induction inverts the BBWT, and writes T front to back as it closes.
- **C:** all 18,030 columns over 2 letters to length 12 and 3 to 8 (15,965 with more than
  one word; 44,674 closed letters, 8,767 of several points): every closed word a Lyndon
  word with its Lyndon tree, non-increasing in the infinite order and in the lexicographic
  order alike, and the column the extended BWT of what closed. The same 18,030 words as
  texts: the induction on the BBWT returns Duval's factorization, in its order.
- For the decoder this says what a corrupt stream decodes to: the n bytes round the Lyndon
  word of the cycle the stream's row lies on. For the kernel it says that `write_cells`
  with every closed letter written, not only the marked one, is a BBWT inverter in S
  cells. Not built; no clock.

**4.3 Corollary B (P; C): the left end is the Lyndon tree for the inverse order.** Turn the
rows over (x ↦ n − 1 − x) and the alphabet with them: L' = the complement of L reversed is
the BWT, for the inverse order, of the same primitive T, and π' is π turned over. The
left-end induction on L is the right-end induction on L'. By Theorem 1 its tree is the
Lyndon tree, for the inverse order, of the greatest rotation of T. **C:** 17,732 primitive
words. So entry 1's 2.137 r against 1.812 r (§5.6) is a fact about Wikidata's text under an
order and its inverse, not about the two ends: for the inverse order the ends change
places.

**4.4 The left Lyndon tree is not an order of steps (C).** The left standard factorization
(the longest proper prefix that is a Lyndon word) gives another tree of the same Lyndon
word. Over every order of steps, at either end of an interval, on the BWT of every Lyndon
word over 2 letters to length 10 and 3 to 7 (725 words): the left tree differs from the
right on 613, and no order leaves it on 77 of those. Smallest: `aabbc` (L = `caabb`), left
tree (((a(ab))b)c). Where an order does leave it, it takes fewer steps than the right end
on one word of 314. Its bracketing follows what lies to the left of a factor; a cell is a
set of rows, and rows are sorted by what lies to the right.

**4.5 The right end is Lazard's elimination of the largest letter (K, §3.2; C).** Take the
largest letter z of the cyclic word and replace every block x zⁱ (x ≠ z) by one new letter
bracketed ((x z) z)…; repeat on the new alphabet, the new letters compared as words. What
is left is the right Lyndon tree (**C:** 9,335 Lyndon words over 2 letters to length 14, 3
to 9, 4 to 7). The induction does the same eliminations a cell at a time: the rows of the
largest letter are the last rows, and a step removes the largest of them.

**4.6 Every right-end step is a legal rise (C).** On the BWT of all 17,732 primitive words
over 2 letters to length 12 and 3 to 8, at each of the 133,817 right-end steps: every
letter's word is a Lyndon word, w_A < w_B, and w_B is a largest of the words present. So
the right end is §3.2's rewriting with LF's row order choosing the rise: the one at the
largest row. A reader adds (its simulator, not here): w_B never increases along the
induction, and a maximal stretch of steps with one w_B = z is one Lazard elimination of z,
cut into a step for each pair of letters it joins.

**4.7 Two runs: Euclid (C).** On the column `b^k a^l`, 1 ≤ k, l ≤ 40 (1,600 columns): one
letter closes, of gcd(k, l) points, its word of length (k + l)/gcd(k, l); the steps are the
sum of the partial quotients of k/l, Euclid's algorithm by subtraction. With entry 1's
division rule (§8 item 5) it must be the number of partial quotients, and the word the
Lyndon rotation of a standard word ([BKL] Theorem 24): a unit test for that rule with a
closed form on both sides. A reader notes that the rule would not change what the grammar
is: one elimination of z makes the letters y z^k for every k at once, and y z^k bracketed
((y z) z)… is already a node chain of the Lyndon tree.

**4.8 What the readers counted beyond the papers** (their programs are in
`readers-code.tar.gz`, not rerun here; each count is one sub-agent's, checked here only where
§4.1 to §4.7 say so).
- **The left tree at size.** Its nodes over the hinges against the right tree's: 2,518
  against 1,944 on a JSON-like text of 6,000 bytes (r = 804) by one reader; 29,011 against
  8,581 on one of 30,000 by another (0.97 n against 1.9 r), who gives the reason: every
  unbordered prefix of the least rotation is a node of the left spine, and most prefixes are
  unbordered. Mine, on four inputs of 300 to 400 bytes: 294 / 291, 260 / 262, 178 / 146,
  399 / 386. Not smaller in sum anywhere; larger as n/r grows.
- **By steps on any cell, not only at a coincidence, the left tree is reachable** (the left
  Lazard order: least piece first; 8,730 words, one reader), and the count over the hinges
  holds there too. About one step in twenty of that order is away from a coincidence.
- **The Hall orders the literature names, as orders of steps** (Lyndon right and left, the
  inverse order's two, Nyldon and Nyldon for the inverse alphabet; 3,600 short words, one
  reader): 31,686 to 32,053 steps in sum against a floor of 27,325. None is 1% under the
  right end.
- **The alternating BWT.** LF reverses every run ([GMRRS20] says so as a formula; three
  readers checked it, 89,502 runs in one count). The induction with a flip bit a letter (the
  loser lands before the winner when the winner is flipped; the loser's flip is XORed with
  the winner's) ends on the least rotation in the alternating order on every word tried
  (67,319 in the largest count), and on any column gives back the multiset of the
  alternating bijection, a flipped closed letter of k points standing for squares. Its tree
  is the Cartesian tree of the alternating rotation ranks, which is the Galois words'
  standard tree on 7 of 1,375 binary Galois words: `abb` gives a(bb), and bb is no Galois
  word. The Hall order compares pieces by their own infinite powers; the induction compares
  the rotations that begin at them; for the lexicographic order the two agree on the
  largest cut, for the alternating order they do not. On five Wikidata chunks of 900,000
  bytes the alternating transform has the BWT's runs to 0.1% and 1 to 5% more steps.
- **Another order of the first letter** (the F-buckets as blocks of rows in another order,
  the runs cut at the seams: at most σ − 1 letters more; a generalized order in §3.6's
  sense with only the first position changed). Entry 1 counted bucket orders on 8 blocks
  and found them worse (§5.6). Two readers recounted: on two Wikidata chunks, byte order
  1.811 r and 1.810 r; reversed, by frequency, random, buckets reversed in place: 1.80 r to
  2.20 r, the best gain 0.4% on one chunk; on three small inputs 0.945 to 1.29 of the byte
  order's count. Closed at depth one.

## 5. What this changes in entry 1

Entry 1 stays as merged, with one dated paragraph added to its §10 that points here. The
corrections:

| entry 1 says | it should say |
|---|---|
| §10, new: "§5.6's: that this SLP is the Lyndon tree above the run starts" | the bottom-up making of the Lyndon tree of the least rotation, by merging on the circular word a largest word into its left neighbour, is the Schützenberger–Melançon rewriting (§3.2). New: its run on the run-length column, LF's row order choosing the rise, a rise taken at all its occurrences at once in O(1), and the count |
| §2, "A column that is no BWT … drop it from both orders and go on" | that is the inversion of the extended BWT by its cycles, largest first; in print with n steps (§3.4). New: in S steps, with each factor's tree (§4.2) |
| §5.6 Theorem 2, "every letter's word at every moment is a Lyndon word" as its own count | it follows from two lemmas in print (§3.3) |
| §5.6, "The left standard factorisation gives another tree (Dolce–Restivo–Reutenauer 2019)" | the factorization is Viennot's and Širšov's; DRR give its Cartesian form (§3.5). It is no order of steps at coincidences (§4.4) |
| §5.6, "left end 2.137 r" as an order of steps of its own | the right end for the inverse alphabet order (§4.3) |
| §5.6 Theorem 3, "P from §5.5 and Theorem 1" | a case of Theorem 5 (§4.1), which needs neither |
| §8 item 9, "Whether another order of the alphabet makes fewer cells" | counted for the first letter by entry 1 and two readers: no (§4.8) |

Nothing in the works read anticipates steps = r₂ − 1 + U, the count over the hinges, a
count of S on any corpus, or the cells form.

## 6. Ideas, ranked by what they are worth to the decoder

**To use in the production kernel** (entry 1, §8 items 3 to 5):
1. **A proved refusal after the order stage.** S ≥ r₂ − 1 (entry 1, Theorem 4), and r₂ = 2r
   less the coincidences, which the order stage can count as it lays the two orders out (not built). So
   a block whose r₂ − 1 exceeds the cells budget is refused before any cell, soundly. On
   Wikidata r₂/n tracks S/n a little better than r/n does (correlation over the 233 blocks
   0.991 against 0.983; S/r₂ 1.14 to 1.62 a block, S/r 1.29 to 2.08; **C**). Neither sees a
   periodic block (S/r₂ in the hundreds). Cost: a counter. *Kind: new, from Theorem 4.*
2. **The two-run columns as the division rule's unit test** (§4.7): steps must fall from
   the sum of the partial quotients of k/l to their number. Cost: the rule (not built) and
   thirty lines. *Kind: a known characterization ([BKL] Theorem 24) put to use.*
3. **A validity criterion for free** (§3.4): with g the gcd of the run lengths, a column is
   a BWT iff LF has exactly g cycles. So when g = 1, the first letter that closes unmarked
   proves the stream corrupt; when g > 1 a periodic block is told from a corrupt one by
   counting what closes. The decoder must still write what the walk would. Falsifier: a
   valid block that closes a letter early with g = 1. Cost: a gcd over the records. *Kind:
   known result to implement (Likhomanov–Shur), as a diagnostic only.*

**To record, not to build:**
4. **The kernel as an inverter of the bijective and the extended BWT** (§4.2): write every
   closed letter, not only the marked one. The literature has no run-length inversion under
   n steps. Not bzip2's format. Falsifier: a published inversion in o(n) steps. Cost: a
   writer loop and a census of S on bijective transforms of real text. *Kind: new idea.*
5. **Run-length bijective BWT to run-length BWT without n steps:** the factors' Lyndon
   grammars from the run list, then a Lyndon grammar → run-length BWT construction (entry
   1, §10: Olbrich, ESA 2025). Falsifier: that construction needing a reduced grammar the
   induction does not give. Cost: days. *Kind: new idea, a reader's.*
6. **The division rule in Lazard's form** (§4.7): N → w_A (w_B)^q is a run-length coding of
   nodes the Lyndon tree already has, so Theorems 1 and 2 survive it. A reader counted the
   steps it would take away on small inputs: 40 of 1,944 on a JSON-like text, 81 of 5,267
   on prose, 499 of 501 on `(ab)^500 a`. Entry 1 counted 94 steps of 27.4 million in such
   streaks on Wikidata (its §6.4). *Kind: related approach; it bears on periodic blocks only.*
7. **The run-minimization yardstick** ([GMRRS23] Theorem 27: the fewest runs any
   context-adaptive order gives, in O(n) for a constant alphabet): how far a block's r is
   from the least any sort order could give. A decoder cannot choose the order. *Kind:
   known algorithm; of use to an encoder.*

**Closed by this reading:**
8. The left Lyndon tree as an order of steps (§4.4, §4.8): not at coincidences, and not
   cheaper.
9. Other Hall orders, Nyldon included, as orders of steps (§4.8): within 1% of the right
   end.
10. Another order of the first letter, or buckets reversed in place (§4.8): at most 0.4%.
    The untried variant is depth two (the σ² pair buckets); a reader prices it at forty
    lines and expects nothing.
11. The alternating order (§4.8): the same runs, more steps, and a tree that is no Hall
    tree.
12. A machine-checked Theorem 2 on top of [HS21]: weeks; the standard factorization is not
    in the formal development yet.
13. The bound on common prefixes from inverse Lyndon factorizations, the linear sort in
    the infinite order, Lyndon subsequences (§3.7): no use in the induction.

**What no work read offers:** a way to make the Lyndon tree cheaper than by the induction.
Theorem 5 says where the cost is: one node above a hinge a step, whatever the order.

## 7. Limits

- **Not obtained** (publisher walls; each is cited only through the works that restate it):
  the body of Hohlweg–Reutenauer 2003; Melançon 1992; Reutenauer's *Free Lie Algebras*;
  Viennot 1978; Gessel–Reutenauer 1993; Gessel–Restivo–Reutenauer 2012; Mantaci et al.
  2007; the journal texts of the Elsevier and Springer versions (the arXiv or author
  versions were read, and locators are theirs); Fleischmann–Huch–Nowotka beyond its
  abstract; Postic's thesis; the paper of Holub–Starosta (its formal outline was read).
  Melançon 1992 should be read before §3.2 is leaned on: whether it already merges all
  equal rises at once, with a cost, is not known here.
- **The readers' tests are theirs.** §4.8 reports them with their sizes; only §4.1 to §4.7
  were run here, and only those are in the file beside this entry.
- **The brute force is small:** words to length 14 at most; nothing in §4 was run on a
  bzip2 block. Theorem 5 and its corollaries are proved; §4.4 and §4.8 are counts.
- **"Not in print" is five readers on one day**, on the two lists and what they cite. It is
  not a search of the field.

## Reproduce

```text
python3 docs/journal/monodromy/2026-11-10-lyndon-literature/lyndon_induction_tests.py.txt
```
About two minutes; no dependency; its output as run here is `lyndon_induction_tests.out.txt`
(T1 to T7; the figures of §4.1 to §4.7). Beside it: `scholar-results.txt` (the lists,
verbatim), `readers/` (the five readers' reports as they returned them: bibliography,
findings with their places, verdicts on five conjectures, ideas) and
`readers-code.tar.gz` (their own test scripts and outputs, as left in the session's
scratch; none was rerun here).
