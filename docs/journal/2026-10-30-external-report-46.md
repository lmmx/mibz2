# External research report 46: the guardian on the three N1 analyses (verbatim)

The guardian's message, relayed by the maintainer while the three read-only N1 analyses run
(external report 44).

**What it says:**
- **The scope is right** and stays within the research-only authorisation.
- **A correction to the record:** "P^6" and "P^12" are not equivalent notations for the
  partial-upgrade plans. They are a transmission or rendering artefact, not a mathematical
  identity. The plans are P^{2|4} (two-byte navigation, eligible unsplit intervals upgraded
  to four bytes) and P^{4|8} (four-byte navigation, upgraded to eight). The symbols were
  lost again in this relay; the bullets are kept as received. Report 44's note is
  corrected to say so.
- **The decisive outcomes:**
  - exact output across all tested segments, with no implementation mistake shared between
    the reference and the candidate;
  - whether the four-worker advantage survives plausible contention, without the
    unverified 1.3×;
  - whether the reconciled traffic and peak, temporaries included, stay acceptable.
- **The four-copy analysis should report where N1 loses,** not only where it wins: the
  boundary, not a favourable illustration.
- **No new experiment, no further instruction.** The risk is widening the scope before the
  results are read.

**The coordinator's actions:** report 44's note corrected; the correctness check told to
keep its reference independent of the candidate (the expected output is the original input
itself); the four-copy analysis told to headline the loss boundary. Neither changes scope.

## The message, as received

Claude has the right three investigations underway, and their scope remains within the
research-only authorization.

There is, however, one correction I'd make to its record before treating the findings as
authoritative.

Claude says it interpreted P^6 as and P^12 as . Those aren't equivalent mathematical
notations. The intended objects are partial-upgrade representations, not sixth- and
twelfth-power Ψ tables.

The correct notation is:

* : start with two-byte navigation and upgrade eligible unsplit intervals to four-byte
  navigation.
* : start with four-byte navigation and upgrade eligible unsplit intervals to eight-byte
  navigation.

Its interpretation of the intended candidates is correct, but the unusual notation should
be documented as a transmission/rendering issue, not a mathematical identity.

Otherwise, I like the plan—particularly the effort to deliberately introduce bugs into the
correctness checker. That is an effective way to establish that the checker tests the
relevant properties rather than merely agreeing with its own construction algorithm.

The decisive outcomes I'm looking for are:

* Whether the partial upgrades produce exact output across all tested segments, with no
  shared implementation mistake between the reference and candidate.
* Whether N1's four-worker advantage survives plausible contention assumptions, rather than
  requiring the unverified 1.3× slowdown.
* Whether the reconciled memory traffic and peak allocation remain acceptable, including
  temporary construction buffers.

I'd also ensure the four-copy sensitivity analysis reports where N1 loses, not only where
it wins. The boundary is more useful than a favorable illustrative scenario.

No new experiment is warranted while those checks are running. Once the three results
arrive, we can decide whether Gate B has a sufficiently specific and falsifiable
construction hypothesis.

For now, no additional instruction is necessary. Claude has everything it needs, and the
main risk would be expanding the research scope before reviewing what these analyses
actually establish.
