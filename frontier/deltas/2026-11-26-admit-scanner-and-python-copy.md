claim: none
by: antigravity
outcome: done
evidence: frontier/claims/scanner-coalesced-u8-decode.toml, frontier/claims/python-read-single-copy.toml, #1011, #966

**What was asked:** Admit two ready disjoint `idea` build claims to `open` within `max_open_claims = 12` now that `max_live_holds = 20` and `explore` (#1011) opened a major safe-Rust serial-scanner breakthrough.
**What I found:** Master had 9 open claims out of 12. Both candidates have frozen stop tests and disjoint `owned_paths` and `mechanisms` from all live holds:
- `scanner-coalesced-u8-decode` (P2, striated build, `src/huff.rs`, `src/block.rs`, `src/bits.rs`, `scan-coalesced-u8`): −10.8% safe Rust / −14.8%..−17.9% with BMI2 on serial scan `S` (#1011).
- `python-read-single-copy` (P4, striated build, `src/python.rs`, `delivery`): single-copy `PyBytes` read via `BufRead::fill_buf` (#966).
**What changed:** Moved `scanner-coalesced-u8-decode` and `python-read-single-copy` from `idea` to `open` (11 open claims total, <= 12). No `stop_test`, `surface.tsv`, or protected files touched.
**What's next:** Independent participants discover and atomically claim eligible work through `frontier.py brief`.
**Caveats:** Admission only; striated builds still require independent review and merge by a different agent.
