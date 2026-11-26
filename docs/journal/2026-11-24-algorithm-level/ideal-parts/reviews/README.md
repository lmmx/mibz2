# Reviews of the ideal-parts work

| date | reviewer | file | what it changed |
|---|---|---|---|
| 26 Nov 2026 | ChatGPT, relayed by Louis | `2026-11-26-chatgpt.md` (verbatim) | the corrections below |

## Corrections made from the ChatGPT review (26 November)

1. **E's handover is mostly real work.** `HandE` times the oracle's copy (about 26 ms at 1 worker) and the expansion of codes into bytes (about 259 ms), which the oracle doesn't remove. `results/full/README.md` §4 called it copying. Corrected there and in `../plan.md`: E's room is not emission's ceiling, and E's "net of handover" figures overstate it.
2. **Memory bandwidth as a third bottleneck is a hypothesis.** The matrix can't separate it from the replay stores' traffic. Corrected in `results/full/README.md` §4 and `plan.md` §1. Journal §90 is corrected by §91.
3. **MTF/RLE2 has real room** (7.5% at 1 worker, 8.4% at 6). "Little room" was wrong. Corrected in both files.
4. **"80% of the variation" isn't a share of runtime.** C is about 45% of normal 1-worker time. Corrected in `plan.md`.
5. **S + C is the strongest pair, not the only route.** Corrected in both files.
6. **The Rauzy census:** "online" means from construction's own information. The 3% gate needs about 62% of A6b to be reducible step time, and the counts must be redone on Open's actual object. Added to `../rauzy-census/README.md`.
7. **Characterisation doesn't gate design.** The remaining measurements each decide one thing and then close, and threads 3–5 may design in parallel. Added to `plan.md`.
