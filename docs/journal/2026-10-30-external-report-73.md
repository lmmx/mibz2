# External research report 73: P74-C experimentally viable; the validation plan amended (verbatim)

A relayed message from the maintainer, unattributed in the relay.

**What it decides:**
- **Formal safety is dead; the algorithm is not.** The remaining risk is generated code.
- **The amendments** (added to the draft as §12a of `2026-10-30-p74-option-c-worst-case.md`):
  - σ as a hard bypass to the untouched shipped kernel;
  - three arms T₀, T₁, T₂, isolating the integration tax from the net benefit;
  - out-of-sample Wikidata blocks and independently generated controls, with no
    retuning;
  - forced-switch correctness, malformed streams and real .bz2 adversarial inputs;
  - f·g > C_monitor + C_handoff + C_layout, with CIs on the complete decoder.
- **Next:** finish B2's analysis, reject any B2 model still using 4.10 c/B, then seek
  explicit authorisation for one frozen P74-C experiment if the validation plan is
  adequate. Don't reopen N1 or tune P74-C against the training corpus.

**The coordinator's note:** P74-H (report 72) is under evaluation. If it survives, it would
replace C's machinery, so the authorisation request waits for both B2 and H.

## The message, as received

Decision: P74-C is experimentally viable. Formal safety is dead; the algorithm is not. The
model predicts near-total Wikidata gain retention with negligible control exposure. The
remaining risk is generated code, not classification mathematics.

1. Exploit σ as a hard bypass. For , execute the untouched shipped kernel directly. Full
random/records then incur only block-level dispatch overhead. Use the guard exclusively for
low-density blocks. No need to expose common controls to a resumable kernel.

2. Isolate the integration tax. Require three same-binary arms:

T_0=\text{original shipped},\quad
T_1=\text{new kernel forced to switch at token 0},\quad
T_2=\text{live policy}.

Then:

T_1-T_0=\text{refactor/layout tax},T_2-T_1=\text{net P74 benefit including monitoring}.

If is materially slower, optimize the cold handoff structure before interpreting policy
results. Keep the shipped suffix out of the hot loop's instruction footprint; verify spills
in disassembly.

3. Fix the statistical weakness. was fitted on the exact Wikidata benchmark corpus. Running
that corpus again measures performance, not out-of-sample selection robustness. Freeze ,
then require independent Wikidata blocks and independently generated mixed-distribution
controls. No retuning.

4. Separate correctness from economics. Force switches at every legal transition in small
blocks; validate counts, output, checkpoints and errors against shipped. Test malformed
streams. Build actual .bz2 adversarial inputs before trusting synthetic-stream timings.

5. Quantify the implementation budget. Let be the measured original P74 stage saving and
the retained fraction. Shipping requires

fg > C_{\mathrm{monitor}}+C_{\mathrm{handoff}}+C_{\mathrm{layout}}

with confidence intervals on complete decoder time, plus independent control, CPU-s/GB and
memory gates.

Breakthrough priority: P74-C is a credible ~4–6% decoder-time opportunity, not a 20%
result. B2 must reach approximately 2.33 c/B for its room kernel against the consistent
4.49 c/B baseline to deliver 20% decoder-time reduction alone. Reject any B2 economic model
still using the obsolete 4.10 c/B threshold.

Next decision: finish B2's read-only feasibility analysis; then seek explicit authorization
for one frozen P74-C experiment if its independent validation plan is adequate. Do not
reopen N1 or tune P74-C further against the training corpus.
