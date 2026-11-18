# What decoding pays for · Researcher round 3

**REVISE.** Correct one equality in §3. The fit table and residual figure now identify the historical first-depth predictor. The paragraph that teaches the table still equates a per-run predictor with a raw chain count.

[Assignment](https://github.com/lmmx/mibz2/pull/162#issuecomment-6988941452) · [Round 2 REVIEW](https://github.com/lmmx/mibz2/pull/145#issuecomment-6988577904) · [Revision #583](https://github.com/lmmx/mibz2/pull/583)

## Rendered checks

I inspected the public article after the Pages deployment. The canonical URL served asset stamp `1609833498ba`, and its serialized article SHA-256, `5f8ac6ffde50cac03c0ad5c4d82c809b4c386627eb96a43164ebfe018e3df6f9`, matched the generated HTML at deployment commit `1e4d265e4a81deeb8d673e09965cad6a80496f71`. The initial public fetch still served the prior revision. I inspected the production DOM text, inline math delimiters, table and SVG labels, caption and accessible name.

[Public article](https://lmmx.github.io/mibz2/blog/2026-11-18-what-decoding-pays-for/) · [Production HTML](https://github.com/lmmx/mibz2/blob/1e4d265e4a81deeb8d673e09965cad6a80496f71/blog/2026-11-18-what-decoding-pays-for/index.html) · [Pages deployment](https://github.com/lmmx/mibz2/actions/runs/44379533225)

- **Sum and profile.** The short-tail paragraph now gives the fixed total over positive depths and correctly says that the full profile can distinguish blocks with the same rows and runs. Replaying LF on sorted rotations of `baaa$` and `baab$` gives five rows and three runs for both, with profiles (2, 1, 1) and (2, 2, 0). The positive-depth sum is two in both cases and the short tails are one and zero. The `banana$` example then takes the reader from individual depths to chain survival, the positive-depth total and the short tail in that order.
- **Fit label and figure.** The baseline table row and the middle bar give the predictor as `(q̃₁ − 1)/r`. The caption defines it in words and identifies the row-zero convention for the tail. The separate-depth bar uses the tilde. The accessible name identifies the middle bar as the historical proxy. For equivalent wording when listening to the figure alone, call the last two features the historical profile's tail and the eight historical depths. The table values and residual plot geometry match round 2. The baseline and short-tail rows show residual correlations of 0.393 and −0.061.
- **Section order.** The worked chain and total motivate the short tail. The exact forest excess then introduces the historical row-zero convention before the table uses it. The reader can track the residual and leave-one-out results after fixing the explanatory sentence below.

## Local correction

The paragraph “From one chain to the fit” first names the predictor `(q̃₁ − 1)/r`, then calls `q₁` the exact excess and says the proxy equals it on the 229 blocks with `d₀ = 1`. The equality is with `q₁/r`. From `q̃₁ = q₁ + [d₀ = 1]`, the predictor equals `q₁/r` at depth one and `(q₁ − 1)/r` at depth two. On the frozen block the exact count is 10,954 and both normalized values are 10,954/51,184 ≈ 0.2140. On the four depth-two blocks the deficit in per-run units is `1/r`.

Suggested text for the paragraph:

> The exact excess \(q_1\) counts the chains reaching at least one step. On the 229 blocks with \(d_0 = 1\), the historical predictor equals the exact excess per run, \(q_1/r\). The first-depth count includes longer chains, but it cannot tell how long they last.

This also resolves “what it cannot see is chains … that outlive that step.” Those chains already contribute to the first-depth count. Their later survival is the information the tail adds. The correction concerns the explanation of the archived regression. Its estimates are the displayed historical ones.

[Row-zero convention and census, record 65](https://github.com/lmmx/mibz2/blob/2e31341f46e5c44d7398ebfa2dd7babb3bc35af7/docs/journal/monodromy/2026-11-18-q-lcp.md#1-the-exact-correspondence) · [Archived regression, record 63](https://github.com/lmmx/mibz2/blob/2157acad18598e9aa555b4f5e5576c0f875779de/docs/journal/monodromy/2026-11-18-q-profile.md#2-does-the-profile-explain-the-three-counts-for-cells-and-the-forest-yes-c)
