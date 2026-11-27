claim: none
by: antigravity
outcome: done
evidence: docs/site/blog/2026-11-27-hardware-verdicts-and-measurement-traps.md

- Published `docs/site/blog/2026-11-27-hardware-verdicts-and-measurement-traps.md` with three inline `.bf-fig` SVG charts, social card (`docs/site/assets/social/blog-2026-11-27-hardware-verdicts-and-measurement-traps.png` + `cards.json`), `docs/site/blog/index.md` card, `zensical.toml` nav + `llms` entries, and `frontier/blog/hardware-verdicts-and-measurement-traps.toml`.
- Covers merged evidence since `#1099`: physical i9-10900K `NOT ADOPTED` verdict on `scanner-seam-window` (`#1118`, `c0b2341c`) and reopened `k = 8` scanner headroom (`#1133`), the `1,522 B -> 1.27 GB` run-only window blowup and `runt-exempt-window` premise catch (`#1094`, `#1103`), the 24-run (240-pair) dissection of `#1034`'s 4T `pattern` CI spike into bimodal `ready`-backlog RSS bursts (`R² = 0.706`) and a `p = 0.0114` Round-0 cold-start bias (`#1108`, `#1111`, `#1137`), station-uneven JCC 32B alignment (`#1107`), merged feature-gated 4-lane forward emission (`#1110`), and the `--depth 1` shallow-runner recipe invariant (`#1122`, `#1126`, `#1127`).
- Verified locally with `zensical build --clean --strict`, `check_site.py`, `check_render.py`, `check_prose.py`, `check_announce.py`, `look_pages.py`, `link_audit.py`, and `frontier.py check`.
