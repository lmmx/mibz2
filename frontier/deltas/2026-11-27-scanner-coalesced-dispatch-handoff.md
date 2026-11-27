claim: scanner-coalesced-u8-i9-run
by: codex
outcome: partial
evidence: recipe #1105 squash cb4afa18333a3e82eb2a500d8ae155f4d3e996d5, reviewed head 5aeb37a42a90336f1023d8d9ecc4d6e2844ff7ea, review 6084494699, benchmark run 45953896071, evidence c71c4a56c145508e9186d5efa567c1c7b0b46719, #1084, #1086

The coalesced scanner recipe is integrated. The six Actions checks and legacy pre-commit.ci status succeeded on the reviewed head. Local wiring against both pinned builds exercised all 1,724 Wikidata ×4 blocks, with 932 eligible construction replays per arm at eight and ten workers, and both phase producers checked 838,800,784 decoded bytes. The six generated corpora also passed independent libbz2 parity. These are correctness and wiring results. [Independent review](https://github.com/lmmx/mibz2/pull/1105#pullrequestreview-6084494699) · [CI benchmark](https://github.com/lmmx/mibz2/actions/runs/45953896071)

The seam-window queue integration (#1106) released an admission slot. A participant with supported workflow-dispatch access can take `scanner-coalesced-u8-i9-queue`. Verify its atomic lease, the exact recipe head and green checks, then recheck `bench/workstation-queue.txt` and its history, both signal tags, and the evidence branch. A pre-existing run is reconciled before any mutation. The last inspection found no coalesced recipe row, tag or result. The physical run requires a ready tag at the reviewed head and a queue row on master.

```sh
gh workflow run run-signal.yml --ref master -f signal=ready -f recipe=bench-scanner-coalesced -f sha=5aeb37a42a90336f1023d8d9ecc4d6e2844ff7ea
git ls-remote --tags origin refs/tags/ready/bench-scanner-coalesced refs/tags/hold/bench-scanner-coalesced
```

Queue row after verifying the ready SHA:

```
bench-scanner-coalesced    bench/scanner-coalesced    Frozen #1034 keep/revert comparison, ten paired rounds, seven corpora, pinned scanner, serial phases and construction replay at eight and ten workers (#1105).
```

The standing 9 November authorisation covers this registered attempt. No ready signal, queue insertion or workstation verdict is recorded by this handoff. The separate four-thread CI trade-off in #1084 remains open.
