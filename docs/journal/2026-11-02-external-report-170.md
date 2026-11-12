# External research report 170: the researcher acknowledges Astra's current-row F convention as the review baseline, and sets out its review of the prototype (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with report 171.
- **Source:** the independent researcher of reports 138–156.
- **In reply to:** the maintainer's handoff of Astra's refinement (report 165) and its
  approval (reports 166 and 168).

**Evidence cutoff:** 2 November 2026, before the researcher had the prototype's artifacts
(PR #134).

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
Acknowledged. **The review baseline is now Astra’s current-row F-emission convention**, not my earlier carried-byte proposal: plain encoded-row lane state, header `(encoded_base << 8) | F`, RAW entries `(encoded_successor << 8) | F[row]`, and mandatory first segment at **`φ(orig_ptr)`**.

**I see no immediate contradiction that invalidates Claude’s authorized prototype.** The supplied identity gives, for every step \(t\),

\[
F[\Psi^t(\mathrm{orig\_ptr})]
=
L[\Psi^{t+1}(\mathrm{orig\_ptr})].
\]

Thus emitting current-row F from the primary row matches emitting L from its successor. This equivalence does not require a single permutation cycle. It does **not**, by itself, establish that the segmented implementation handles starts, parking, resumption and tails correctly; those remain implementation-review obligations.

Once you provide Claude’s artifacts, the review will address the three specified questions: semantic correctness across transitions, the actual compiled dependency and state-traffic costs, and any **specific** missed opportunity suggested by the BLAKE3 study. I’ll distinguish scalar-reference guarantees from segmented-test coverage, and static code-generation improvements from measured performance.

The conclusion will be whether the prototype **meets its compile-and-correctness objectives, needs one specific repair, or exposes an architectural obstacle for Astra**. FOR-14’s frozen negative result remains unchanged, and `k`/`j` remain untimed.
`````
