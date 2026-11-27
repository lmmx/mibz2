claim: six-gbs-architecture
by: antigravity
outcome: done
evidence: docs/site/blog/2026-11-27-six-gbs-decoder-architecture.md

## Summary
Published `docs/site/blog/2026-11-27-six-gbs-decoder-architecture.md` under standing blog-publishing authority, synthesizing five freshly merged primary studies on `origin/master` into an end-to-end 6 GB/s bzip2 decoder architecture.

## Primary sources cited (all merged on `origin/master`)
- `#1057` (`7f5a1c6a`, PASS): uniform BWT-row sampled anchors (`S = 16`, bit-31 tag on unmodified SIMD `Reader::wire`, 4-lane dynamic chase, `−31.0%..−35.8%` `A6c+A7`, `97.4%` of L3-cold oracle, `+208..+235 ms` 1W proj.).
- `#1045` (`e61ec127`, PASS / scoped STOP): `fused_4l_tile4096_vec` zero-scratch in-place tile compaction + 64 KiB PCLMULQDQ CRC32 + GF(2) combine (`−53.4%` writes, `+31.8..+49.9 ms` 1W proj.); 1-lane, inline FSM, and stitched 4-lane (`74.9%` output moved) stopped cleanly.
- `#1058` (`28c4c8c4`): 160-byte inline vectored `ReadyBlock` (`0.86×–1.01×` across 6 consumers, RSS `1.017×` with `+16 KiB` tile headroom, `0/431` Wikidata overflows).
- `#1048` (`5c57286d`) + `#1055` (`3a2b7702`, PASS): register-spill diagnosis of `#1010` + 2 KiB L1d packed run batch + `OutQ16` 496/512 guard zone (`6.22×` fewer `push_slow`, `−12.1%` base / `−19.1%` native `A5+A6a`).
- `#1056` (`72286cff`): discrete-event replay of slack-only hybrid scanning (`91.6%`-idle `mibz2-read` AVX2 `Finder`, `0.00 ms` `pool_copy`, Policy A jump-ahead) reaching `131.8 ms` (`6.36 GB/s`) at 8W with `0.997×–0.999×` at `k = 6..7`.

## Site & mechanical checks
- Added inline `.bf-fig` SVG pipeline architecture diagram and coupled 8-worker trajectory chart, social card (`docs/site/assets/social/blog-2026-11-27-six-gbs-decoder-architecture.png` + `cards.json`), `docs/site/blog/index.md` card, and `zensical.toml` nav + `Optional` llms entries.
- Verified strict `zensical build --clean --strict`, `check_site.py`, `check_render.py`, `check_prose.py` (`dek: PASS`, reading ease 50, body 8% flagged), zero agent names in `docs/site/`, and `frontier.py check`.
