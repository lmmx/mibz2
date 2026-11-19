# Acquiring marked exception clocks through default contexts

After the physical context model is known, the DAG and monotone-FIFO construction computes the next event rank and its exact LF distance. The primary is always retained. One virtual affine interval opens each default-context cycle; a stable partial bijection on its complement closes the surviving paths. In a gap of L ordinary ranks, h pending paths receive floor(L/h) full laps each plus one for an initial prefix, and the queue rotates. Long rank gaps therefore need no per-row loop. The first implementation uses one physical record per exceptional occurrence. Its successor accepts complete ranges and publishes the canonical first-return bands. [Original proof](PROOF.md) · [Interval construction](ranges/README.md)

Any context with no stops maps its entire interval by one translation. The composed-state variant adds these offsets and step counts along a known default path; only the contexts containing an exception or the primary take part in the label transport. Across ten depth-eight models this reduced 815,844 context states to 23,497. The primary induces at most one extra active state. It does not remove the cost of acquiring the original boundaries and model. [Composition theorem](SMOOTH.md) · [Sparse transport](transport.py)

All 30 models at depths 4, 8 and 16 reproduced every independently verified marked return and every supplied clock byte/poststate. The raw Python thread clocks below the source receipts describe this unpaired research implementation only. Its deterministic output agrees with the later range and native gates. [Depth eight](k8.csv) · [Depths four and sixteen](k4-k16.csv) · [All composed-state receipts](contracted.csv)

| Context depth | Full contexts | Active contexts | Event ranks incl. primary | Real and virtual rope nodes, composed occurrence version |
|---|---:|---:|---:|---:|
| 4 | 471,228 | 97,686 | 1,122,586 | 1,247,879 |
| 8 | 815,844 | 23,497 | 409,433 | 433,633 |
| 16 | 1,170,188 | 18,467 | 229,014 | 237,313 |

The exhaustive suite checked 17,576 stable partial maps, 9,789 marked context models and 39,156 emitted horizons. A seven-symbol encoded source with eight retained event/primary ranks and 30,786,325,577,745 rows was solved using 34 rope nodes. The real kth-order models have far fewer cyclic virtual survivors than that adversarial case. [Tests](check.py) · [Exact results](small.json)

Run the following from the repository root. Generate the earlier maximal input/CRC-backed marked files and the sibling contexts and clock wires with their published drivers. Set the three environment-specific source directories to those prepared paths. The real checker gives only the physical context endpoints and the modal bytes to the constructor; the independent dense LF and supplied clock wires are used after construction.

```bash
e=$(dirname "$(rg --files | rg '/all-marks/exceptions/PROOF.md$')")
a="$e/acquire"
python3 -B "$a/check.py" | diff - "$a/small.json"
python3 -B "$a/real.py" "$original" "$models" "$e/contexts/inputs.csv" "$wires" --compact --depths 4 8 16
```
