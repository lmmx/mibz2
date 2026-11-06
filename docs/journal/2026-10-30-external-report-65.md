# External research report 65: option C reduced to five falsifiable obligations (verbatim)

A relayed message from the maintainer, unattributed in the relay.

**Decision rule:** advance P74-C ⇔ state equivalence ∧ (bounded or explicitly empirical
risk) ∧ measured net stage gain. A stage pass is not deployment evidence. Prove handoff
correctness and bound monitoring exposure first. If either needs a costly per-token
mechanism, terminate C without CI.

**The coordinator's deduction, sent to the analysis to verify or refute:**
- **The claim:** threshold evasion defeats frequency-ratio rules, but not a debt rule with
  one-sided conservative weights. With w(q) ≥ the true excess of every high-rank token and
  α ≤ the true saving of every low-rank token, D̂_t ≥ D_t on every prefix. So "switch when
  D̂ > B" bounds R(x) ≤ (B + H + M(n))/T_T(n) for every stream, with no exposure premise.
- **The cost:** false switches on Wikidata, from the conservatism.
- **The premise:** per-token bounds that hold regardless of context. Mispredicts and
  store-forwarding may not be boundable, which would make the bound empirical.

## The message, as received

P74-C: reduce the problem to five falsifiable obligations.

1. Replace rank-frequency prediction with bounded-loss switching. Define . A fitted cannot
bound : cache dependencies, run emission and branch history violate rank-only additivity.
Use it for policy selection, never for a safety proof.

2. Derive the adversarial floor. For explicit tokens, initial commitment , switching delay ,
maximum excess cost per token, and handoff :

R_{\max}\le
\frac{\delta_{\max}(p+W)+H+M(n)}
{T_T(n)}

only if total unfavorable exposure is bounded by . Threshold evasion invalidates that
premise. Without an exposure bound, worst-case regression approaches the unswitched P74
penalty. Require a proof of bounded exposure or explicitly abandon universal safety.

3. Test whether monitoring is actually free. Disassemble both kernels. Count extra
fast-path µops, branches, register spills and loop-carried dependencies. Kill any policy
requiring per-token debt maintenance unless its measured cost fits the Wikidata gain and
the control budget. Fallback-only updates plus sparse checks are preferred, not presumed
free.

4. Make switching transactional. Handoff only after completing an MTF token and its RLE2
effects. Prove equality of list, input cursor, pending-run state, output cursor and
checkpoint counters. Use a separate-state oracle; inject switches at every legal boundary
on exhaustive small inputs. No reverse switch.

5. Preregister a strict stage gate. Same binary; paired measurements; CPU/topology
recorded. Wikidata checkpoint gain ≥5%, CI excludes zero, gain exceeds 2× layout noise.
Random/records plain and checkpoint upper 95% regression bounds ≤3%. Adversarial
correctness mandatory; fix adversarial performance limits before measuring. No threshold
sweep or post-result tuning.

Decision rule:

\boxed{
\text{Advance P74-C}
\iff
\text{state equivalence}
\land
\text{bounded or explicitly empirical risk}
\land
\text{measured net stage gain}
}

One further hard constraint: a stage pass is not deployment evidence. Full-decoder
throughput, CPU-s/GB and RSS remain separate gates.

Priority: first prove handoff correctness and bound monitoring exposure analytically. If
either requires a costly per-token mechanism, terminate C without CI.
