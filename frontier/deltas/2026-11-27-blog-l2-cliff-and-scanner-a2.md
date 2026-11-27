claim: l2-cliff-and-scanner-a2
by: antigravity
outcome: done
evidence: docs/site/blog/2026-11-27-l2-cliff-register-wall-and-scanner-a2.md

## Summary
Published `docs/site/blog/2026-11-27-l2-cliff-register-wall-and-scanner-a2.md` under standing blog-publishing authority, synthesizing the eight merged studies since `#1061` across worker composition, hardware ceilings, safe-Rust serial header parsing, JCC-32B branch alignment, and off-path stops.

## Primary sources cited (all merged on `origin/master`)
- `#1062` (`0549fdf1`) + `#1067` (`c149eb69`): composed 4-mechanism Open worker stack (`-0.07%` additivity residual, working set halved `6,893 -> 3,488.5 KiB/w`), 256 KiB private L2 slab cliff (`4 x 64 KiB` regresses `A7+A9` `+12.1..+22.2 ms`), and `#[cold] #[inline(never)]` outlining removing `batch.as_ptr()` stack spills (`-21.6%..-24.6%` L3-cold).
- `#1068` (`efb3dd9d`, STOP) + `#1076` (`8e048544`, STOP): direct-literal split regressed `24/24` `A7` cells (`+4.1%..+39.0%`); `K = 6, 8, 10` hit the 15-GPR wall (`+10.8%..+45.4%` in `A6c+7+9`), and decoupled batch-chase broke OoO overlap (`+56.0 ms` at `K = 4`).
- `#1069` (`a34dda23`, PASS) + `#1074` (`b7e2187f`): safe-Rust `A2b_direct_runs`, `A2c_lut_multi`, and `A2d_direct` cut standalone `A2a..d` `-31.6%..-37.7%` and post-`#1034` `S'` `-4.44%..-7.38%` (modeled `S' ~ 134-139 ms`), flipping `k = 8` to worker-bound and shifting slack-only hybrid scanning to `k = 9-10`.
- `#1077` (`c1239ce0`): whole-decoder Intel JCC-32B alignment audit (`1,746 -> 0` unfused crossings, `+3.4%` `.text`, `12/12` off-Wikidata faster, `OpenCells` `-13.7%..-15.6%` disjoint on VM Xeon silicon), scoped as VM benchmark control with target-silicon gate before any release flag.
- `#1060` (`da489972`, STOP) + `#1063` (`371cede0`, STOP): column-route MTF RLE2 branch-misprediction floor and Cooperative Amdahl cap (`-5.2%..-6.2%`).

## Site & mechanical checks
- Added three inline `.bf-fig` SVG charts (L2 cliff + 15-GPR wall, `A2a..d` speedups + flipped knee, JCC-32B stations + silicon matrix), social card (`cards.json` updated), `docs/site/blog/index.md` card, and `zensical.toml` nav + `Optional` llms entries.
- Verified `zensical build --clean --strict`, `check_site.py`, `check_render.py`, `check_prose.py` (`dek: PASS`, reading ease 60, body 6% flagged), `link_audit.py`, zero AI/model/agent names in `docs/site/`, and `frontier.py check`.
