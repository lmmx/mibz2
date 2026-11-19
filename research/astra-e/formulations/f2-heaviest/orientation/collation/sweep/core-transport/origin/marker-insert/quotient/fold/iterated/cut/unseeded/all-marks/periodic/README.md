# All-primary coordinates with periodic and heterogeneous cycles

The gcd g of maximal input run widths gives an exact g-way lift of the stable BWT permutation. After division, a single representative orbit of size n/g certifies that every original primary spells a rotation of one primitive circular tape. On a general column the same gcd is exactly the gcd of its distinct necklace multiplicities. The single-orbit check matters. On L=001100, primary 2, g=2, the quotient is 010 with two cycles. Repeating its first three marked bytes gives 010010, while the correct bytes are 010101. [Proof](PROOF.md) · [Minimal counterexample](small.json)

For general multi-cycle inputs, ordinary cuts and same-type power reductions carry integer offsets without a spelling grammar. Save a terminal owner and weight for every fixed component, including the components omitted by power normalization on a single-mark decoder. An old row at step j of a maximal run starts at byte offset j times the old type weight inside its macro source. Reverse these additions, then scatter original F bytes by terminal and offset. The independent dense test checked 111,438 primary ranks across 11,469 columns, including 10,128 heterogeneous columns, 31,712 ordinary cut steps, 3,721 power steps and 39,507 fixed terminals. It also regenerated and checked the genuine cyclic BWT of every enumerated word. A sparse all-primary publisher for the power descriptors would still need its complete construction and storage price. [Numerical implementation](numeric.py) · [Exhaustive checks](check.py)

The compulsory count of distinct circular base tapes for heterogeneous primary outputs is characterized exactly by taking the shortest different primitive necklaces. Over a fixed alphabet its worst-case order is n/log n. The complete normalized numerical atlas can require linearly many bits even on primitive binary input. The dense exhaustive search confirms the sharp dictionary extremum at every binary size through twelve. [Proof and counting assumptions](PROOF.md) · [Exact maxima and attaining columns](extremal.csv)

Four binary runs suffice to force the maximum possible number of contiguous modular-affine pieces, even with a different fitted slope on each piece. For L=b a b a^(n−3), n even, that number is n/2. The permutation advances by two except at two crossing edges. The arithmetic all-row formula instead separates even and odd ranks. More generally a base rotation with h residue lanes gives one contiguous phase arc per lane exactly when each lane has at most one exceptional outgoing edge. The lane permutation and d-length phase formulas then give every component and offset from O(h) metadata. An explicit four-run primitive word-column realizes a single merged cycle at all h,d≥2. [General theorem](PROOF.md) · [34,406 arbitrary permutation/shift checks and 1,155 four-run families](interleave.json)

All ten original Wikidata blocks have g=1 and a full representative orbit. Their sum is 8,999,915 raw marked bytes, with every output authenticated against the previous libbzip2-backed inverse. None of these blocks meets the necessary half-mass condition for a nontrivial one-splice residue decomposition. Their best single shift covers only 1.85–3.01% of rows. These weights independently reproduce Astra B's strongest LF translation error counts with the inverse sign convention. Astra B already treated general exception graphs; the one-splice characterization supplies the sharper h-lane admission test and closed-form coordinate where it applies. The real contiguous-slope study belongs to the sibling backwards/slopes record. No speed or target-memory improvement is inferred. [GCDs and orbit hashes](real.csv) · [Independent shift reconciliation](shift_mass.csv)

Reproduce the structural tests from the repository root. Generate the ten checked original input.txt and cut.bin pairs with the powers verification instructions, then set `inputs` to that directory.

```bash
p=$(dirname "$(rg --files | rg '/all-marks/periodic/PROOF.md$')")
python3 -B "$p/check.py" | diff - "$p/small.json"
python3 -B "$p/extremal.py" | diff - "$p/extremal.csv"
python3 -B "$p/interleave.py" | diff - "$p/interleave.json"
python3 -B "$p/real.py" "$inputs" | diff - "$p/real.csv"
python3 -B "$p/shift_mass.py" "$inputs" | diff - "$p/shift_mass.csv"
```
