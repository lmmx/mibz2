# Independent marked-row verification

The terminal at `ef679d8d` passed all 26,999,745 real row/context pairs and all 30 marked outputs at depths 4, 8 and 16. Each row was checked through the actual one-byte update, unbounded group and a clipped group. Every emitted spelling matched step-by-step physical LF, in the correct output orientation, and every landing matched its original row and new context ID. The ten full marked inverses also matched the byte vectors independently identified in the historical original capped record and libbzip2 receipt. This is a component gate. It does not provide an end-to-end wall measurement or a new stream CRC computation. [Real rows](real.csv) · [Hashes](gate-manifest.json)

The reference stably sorts the original row indices on their last-column bytes to produce Ψ. Inverting that permutation gives LF. Independent prefix doubling on the pair (F, Ψ) recovers each physical context class without the candidate's constructor, and the gate checks all memberships before using its renumbering. Complete marked output follows F then Ψ for exactly n steps. Buffers have checked canaries.

The arbitrary-column panel confirmed 39,052 complete marked outputs. It included 15,044 cases where the physical marked cycle length did not divide n, of which 10,754 had nonconstant all-smooth marked cycles. Checks cover complete and clipped groups, each primary on all binary columns through length 8 and all ternary columns through length 6, and longer columns around cap-127 boundaries. Original high-byte alphabet values include 128 and 255. A minimal nondividing witness has last column (0,128,0), n=3, primary rank 1 and marked period two. The high-depth all-smooth path emits 0,128,0 in forward order. [Finite counts](finite.csv) · [Verifier](gate.cpp)

Run from this directory on the local clock's read-only original cap and raw corpus.

```sh
taskset -c 2 nice -n 12 python3 -B gate.py /workspace/work/astra-powers-original/capped "$(mktemp -d)"
```
