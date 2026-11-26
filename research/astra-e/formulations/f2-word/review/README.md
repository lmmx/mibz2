# Independent review of the direct word section

Let Ψ advance one byte-row position, emitting the first-column byte F at the current row. Fix a byte d. A first-level boundary is the row just after any d, and a first-level token runs from a boundary through its next d inclusive. For a token w, let P = dw and m = |P|. S is the interval of rows whose next m bytes spell P. B consists of the first-level boundaries whose incoming token is w. These definitions apply separately on every Ψ cycle, including cycles shorter than m.

**Order and returns.** The map H = Ψᵐ sends S onto B. At any hit it steps over the opening d and the complete token w. At a point of B, rewinding m bytes reaches the unique corresponding opening d. Each single Ψ step is strictly increasing within an F-byte bucket by stable occurrence matching. All starting rows of S read the same m bytes, so H preserves rank at every step on their images. In particular, rank i in S corresponds to rank i among B in original row order. F2c can equivalently express that rank inside the first-token F bucket of w by stable LF selection on its incoming token.

If the next hit after s has gap h and is s′, then the next hit of B after H(s) has the same gap h and is H(s′). Advancing any fixed number of steps along a permutation commutes with Ψ. Any earlier B hit would rewind by m to an earlier S hit. Thus the two induced successor arrays agree entry for entry in rank coordinates.

**Words and marks.** Write A for the h bytes from s to s′ and Q for the h bytes from H(s) to H(s′), with the destination row excluded in each case. Reading the same length-(h+m) path gives A·P = P·Q. If h ≥ m, A begins with P, and Q is the remaining suffix followed by P. If h < m, the common path has two copies of P at distance h and is h-periodic throughout. In both cases Q is A rotated left by m modulo h. The circular byte text 010102, with d = 0 and w = 10, has an overlapping return A = 01 and Q = 10. Equal source and target spellings would be a bug on that edge.

P has d only at its first and last bytes. Any proper occurrence overlap must therefore place the next start on the final d, giving gap m−1. A chain of such overlaps can span many tokens. A Ψ cycle omitting P has no B hit either and must fall back. For a covered primary, rewind within its own cycle to the preceding start row. Its byte distance is the initial skip in the outgoing A-word. F2c instead uses the corresponding offsets in its post-occurrence coordinate. Following successive words, skipping once and truncating at n gives the marked result even when the cycle length does not divide n.

**Executable checks.** The reviewer constructs dense Ψ independently by stable sorting arbitrary last-column indices. It compares FM prefix bounds with literal byte lookahead, computes first-level token edges by a dense walk, and feeds their incoming-token runs to the recorded F2c Python constructor with reversed dictionary IDs. For every arrow it checks the whole rank bijection, destination, both words and rotation. For every primary it checks the reconstructed n bytes or the required fallback.

All binary columns of lengths 1–9 and ternary columns of lengths 1–6 gave 2,114 columns, 9,360 candidate sections and 14,209 checked arrows. Of 64,231 marked cases, 23,662 admitted marks lay on cycles whose lengths do not divide n. The test verified 21,350 missing-marker fallbacks and 8,349 overlapping arrows. A separate input with three lifted orbits, 65 repeated 18-byte tokens and five unrelated singleton cycles verified 192 endpoint overlaps. A one-row orbit also passed when P = dd required reading twice around that orbit. F2c's second-level constructor is pinned by its digest in checked.json and source commit 31565705f292b3b1dc3d8200eb65cd880e238bc7.

```sh
python3 -B research/astra-e/formulations/f2-word/review/conjugacy.py > /tmp/f2-word-review.json
diff -u research/astra-e/formulations/f2-word/review/checked.json /tmp/f2-word-review.json
```

The review establishes the correspondence of the mathematical sections and checks F2c's executable token table. The direct Rust constructor's independent byte tests and measurements are in the parent study.
