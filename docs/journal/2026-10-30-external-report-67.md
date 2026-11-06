# External research report 67: the debt bound corrected (verbatim)

A relayed message from the maintainer, unattributed in the relay, on the coordinator's
conservative-debt deduction (report 65).

**What it decides:**
- **Conservative weights remove threshold evasion,** not monitoring overshoot or initial
  losses. With checks every W tokens:
  R(x) ≤ (B + W·d̂⁺_max + H + M(n))/T_T(x). This needs the probe monitored under the same
  rule, a valid cost decomposition, and no penalty in the shipped suffix.
- **Open:**
  - context-dependent excess, which a per-rank bound needs proved;
  - false-switch economics, measured as the switch position and the fast-path bytes
    retained;
  - a positive lower bound on T_T for short blocks.
- **Kill test:** conservative debt maintained with no extra low-rank-path operations and no
  premature switching on Wikidata. If either fails, the formal-bound version is
  uneconomic, and only an explicitly empirical policy remains.

**The coordinator's action:** the correction and both kill tests were passed to the running
analysis.

## The message, as received

The debt argument is promising, but its bound needs correction. Conservative weights can
eliminate threshold evasion; they cannot eliminate monitoring overshoot or initial losses.

Let , with context-dependent costs, and suppose for every transition. Then

D_t=\sum_{i\le t}d_i\le\hat D_t.

If switching occurs immediately when , then at the trigger:

D_t\le B+\hat d_t.

With checks only at intervals of tokens:

\boxed{R(x)\le
\frac{B+W\hat d_{\max}^{+}+H+M(n)}
{T_T(x)}}

provided the initial probe is monitored under the same rule, the cost decomposition is
valid, and the post-switch shipped suffix incurs no additional penalty.

Three hard problems remain:

* Context-dependent excess cost. Rank does not determine . Branch prediction, overlapping
  stores and cache state affect execution. Per-rank conservative weights require proven
  bounds on these effects; a 256-byte list fitting L1 does not establish them.
* False-switch economics. Conservative weights may force P74 off Wikidata before
  recovering useful gains. Measure the switch position and retained fast-path byte share,
  not merely classification agreement.
* Denominator and overhead. A 3% bound relative to requires a known positive lower bound on
  shipped runtime for the relevant block class. Short blocks can make fixed , and
  disproportionately expensive.

Stronger formulation: use a conservative upper envelope of cumulative excess cost,
incorporating state-dependent uncertainty. If that envelope is too pessimistic to preserve
Wikidata's gain, abandon formal worst-case safety and preregister an empirical gate.

Decision: continue the read-only analysis. The key falsification test is whether
conservative debt can be maintained without extra low-rank-path operations and without
premature switching on Wikidata. If either fails, the formal-bound version of Option C is
uneconomic; only an explicitly empirical policy remains viable.
