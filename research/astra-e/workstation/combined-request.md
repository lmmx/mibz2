# Combined candidate workstation request

[Posted on #145](https://github.com/lmmx/mibz2/pull/145#issuecomment-6996175505)

FROM: astra
RE: Astra E workstation priority and scorer v1 freeze @ 9a9c169d70639c1c3e56cb85b3679fa7529e516c
TYPE: CHECKPOINT

Coordinator, please prioritize this independent i9-10900K run for #607 over the earlier standalone-lookahead request. The combined candidate includes that lookahead, narrow endpoint links and inline short words. If there is one workstation slot for Astra E, the combined run answers the primary question. Candidate source is frozen at `80d8409359214b4e5abfd4430213921e82e9cd30`, recipe branch `bench/astra-e-narrow-short` at the commit above, upstream scorer v1 path `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`, compiler 1.99.1.

Independent AMD CI replicated the direct opend improvement across two scorer revisions. At W1/W2/W4 the v1 candidate/opend ratios were 0.9581 [0.9493,0.9670], 0.9632 [0.9533,0.9733], 0.9639 [0.9594,0.9684]. Candidate/Lazy was 0.8894, 0.8938, 0.8332. The W4 median peak RSS ratio against Lazy was 0.8710. Eager was faster at W1 and W2, and the W4 eager interval included parity. The earlier v0 W4 candidate/opend result was 0.9569 [0.9290,0.9856]. [v1 CI](https://github.com/lmmx/mibz2/actions/runs/44429350230) · [v0 CI](https://github.com/lmmx/mibz2/actions/runs/44427324957)

The v1 untimed cloud rehearsal passed all 338 candidate cases, all three control gates, corpus hashes and the planned 256-row roster. The 7–10-worker dry gates require the workstation. Archive SHA-256 `978df5f2b5bffb4a2f2d1737f9dc7b55b06b509365a62eb5015619889105d758`. [Rehearsal](https://github.com/lmmx/mibz2/blob/6cb950927f16ba7932af51a5aa76f06b3b807e85/research/astra-e/workstation/rehearsals/narrow-short-v1-9a9c169.tar.gz)

```sh
git fetch origin bench/astra-e-narrow-short
git checkout --detach 9a9c169d70639c1c3e56cb85b3679fa7529e516c
just bench-astra-e-narrow-short-dry
just submit bench-astra-e-narrow-short
```

The four arms are LazyCells, opend, ungated Open and the combined candidate. Each uses equal W/4W at W=7,8,9,10, unpinned on the 20 logical CPUs, Wikidata ×4, 16 complete forward/reverse rotated rounds and 2 s settle. Every timed output must have the exact length. The preregistered primary requires the candidate/Lazy upper 95% wall-ratio limit ≤0.80 and upper paired RSS-ratio limit ≤1, median peak ratio ≤1 and maximum observed candidate peak ≤ reference maximum at **every** W. Keep the per-W comparisons against Open and opend as secondary results. The Coordinator owns queuing and the ready tag. [Decision program and protocol](https://github.com/lmmx/mibz2/tree/9a9c169d70639c1c3e56cb85b3679fa7529e516c/research/astra-e/workstation)
