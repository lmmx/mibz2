# Exact suffix obligations and packet costs for a frozen prefix policy

A prefix-closed set A contains the byte prefixes at which a source packet attempts one more forward step. An action at α costs its forward-run intersections even if the next rank lands in the section. In that case the terminal stays at its current row, retaining the last byte for backward discovery. For a positive original return W, let t_A(W) count successful steps and write W = P·C with |P| = t_A(W). Every occurrence of W makes the same decisions and C is nonempty.

**Suffix identity.** Let S contain every distinct proper suffix of the original words, including the empty suffix. A nonempty u in S is removed precisely when every W ending in u *strictly* has its predecessor action W[:|W|−|u|−1] enabled. A required action of length H or greater is unavailable when H steps are permitted, so such a u remains. Proof: write k=|W|−|u|>0. The same occurrence of u is still proper in C exactly when |C|>|u|, or t_A(W)<k. Since u is nonempty, the kth advance is short-safe. Reaching it is equivalent to enabling the action at depth k−1, since the policy also contains all its ancestors. If an original word equals u it supplied no such proper occurrence. The empty suffix survives every policy. Removing a repeated occurrence alone changes the count only if it was the last witness among the original words.

Duplicate prerequisite actions can be combined. If a suffix requires both α and one of its proper extensions, the shorter action can be omitted from the obligation list because the policy is prefix-closed. The exact saving is the sum of the indicators that a complete nonempty-suffix obligation set is contained in A. This is monotone and supermodular on prefix-closed sets. Two incomparable actions can have zero individual gain while their union removes a suffix.

For example, circular text 01210221 with marker 0 gives stable L = 11220210 and U = [0,2). Its return types are 0121 and 0221. Enabling the empty prefix and 0 leaves Q=2. Further enabling either 01 or 02 spends one extra forward intersection with Q unchanged. Enabling both removes suffix 1, with Q=1. A linear-price version can be solved as a maximum-weight closure: give each eligible suffix its unit benefit, add arcs requiring each predecessor-prefix action and each enabled action's parent, and assign negative per-action resource costs. For separate integer resource caps this only certifies the scalarized envelope. Under any fixed nonnegative prices, adding the priced budgets to the optimal scalarized value gives an upper bound on feasible suffix savings.

**Additive construction.** With uncoalesced source-affine packets, the input packets arriving at a prefix α depend only on its enabled ancestors. A competing sibling decision changes none of them. Record I_α, the number of input packets; F_α, their intersections with normalized select runs; and T_α, the number of fragments after testing the two section boundaries. T counts early fragments stopped in place as well as fragments carried to children. Every execution obeys

```text
F(A) = sum over enabled α of F_α
B(A) = 1 + sum over enabled α of (T_α − I_α).
```

The second identity telescopes the replacement of each live packet by its emitted or surviving fragments. An unreachable action has zero contribution. All costs can therefore be collected by one full-depth run on the priced input. Applying a literal policy to another input requires that input's own packets if pricing it; training coefficients cannot assert the transfer result. Source-active ranks over all attempted steps are disjoint because they stay in separate positive section-return arcs. Their images are disjoint since Ψ is injective. Each original select boundary can split a source packet at most once, and the two section endpoints can split at most two strict packet-image interiors. Thus B ≤ r+2 for r maximal original byte runs.

L = 01 with U = [0,2) illustrates the choice of accounting. Both returns have length one. With no action, (Q,F,B)=(1,0,1). Enabling the root attempts two select runs and leaves both terminal cuts in place, giving (1,2,2). Its normalized affine identity map would have one band. The telescoping identity prices the uncoalesced atlas required by the registered policy.

**Independent check.** The Python auditor computes forward strips directly as adjacent stable Ψ ranks with equal output byte and translation. It groups dense row steps by actual select-strip and section-membership changes, separately executes each tested policy instead of pricing it solely from training data, reconstructs every positive core backwards, then checks the exact original source, destination and complete word. It enumerates all binary stable columns through seven rows, every nonempty interval, maximum depths 0, 1, 3 and 5, and all effective prefix-closed policies at each configuration. Unreachable literals have no operational effect. The 22,520 section/depth instances cover 57,134 policies, 153,678 source arcs, 25,484 policies with charged short lookahead, and 7,418 instances with a depth-impossible suffix. All 5,164 ordered incomparable policy-pair checks passed; 496 showed strict complementarity. This audit uses synthetic inputs only.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/short-core/policy/audit
python3 -B "$d/check.py" > /tmp/astra-prefix-policy-audit.json
cmp "$d/exhaustive.json" /tmp/astra-prefix-policy-audit.json
```

[Independent enumerator](check.py) · [Exact counts and counterexamples](exhaustive.json) · [Frozen accounting convention](../REGISTER.md)
