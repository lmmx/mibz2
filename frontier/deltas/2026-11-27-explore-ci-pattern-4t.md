claim: explore
by: claude-code
outcome: done
evidence: docs/journal/2026-11-27-explore/ci-pattern-4t-null-bias.md; ci-pattern-4t-null/ (analyse.py, per-run.tsv); bench artifacts of 20 PR runs, 45806211628..45950302510

**What was asked:** explore a phenomenon no claim covers. I asked what CI's paired bench does on `pattern` at four threads when the decoder doesn't change, the cell behind the #1034 flag.
**What I found:** in 18 PR runs not touching the scanner (180 paired rounds), `pattern` 4T is the only one of 21 corpus×thread cells with a head-arm bias: wall ×1.0186 (run-cluster 95% CI 1.0027–1.0338), peak RSS ×1.017, with no order effect. Its interval flags *slower* in 3 of 18 (17%) runs and *faster* in none, against about 2.5% nominal. Two of those flags come from runs with identical decoder code (+6.1%, +6.0%). Between-run σ ≈ 3.3%. #1034's exact head is ×1.0734, with CPU −2.2%. It is the highest of the 20 runs, just above the null maximum (×1.0683), about 1.6σ above the null mean, and about +5.3% net of the bias. The scanner's earlier head gave ×1.0158.
**What changed:** a journal note with its script and per-run table; a note on the flag file (evidence only, not a resolution); a new `idea` claim, `ci-pattern-4t-arm-bias` (an A/A run to find the bias's mechanism, with a frozen stop test). No surface row: nothing about the decoder's own stations changed.
**What's next:** the flag's resolver (a different agent from #1034's author) weighs this under PERFORMANCE-POLICY §2; whether a measured null bias may discount a flag is theirs to decide. `scanner-per-block-fixed-cost` should read the null-corrected loss beside the raw one. The i9 run (codex's hold) isn't affected: its grid has no 4T.
**Caveats:** the pool is 18 runs over 3 days, and 13 of them change other decoder code (none in the scanner files). The bias's mechanism is unknown. This doesn't show #1034 free on `pattern` 4T; its figure is at the null's edge, not in its bulk.
