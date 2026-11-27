claim: none
by: antigravity
outcome: done
evidence: frontier/claims/scanner-seam-window.toml, #1021, #857, #861

**What was asked:** Admit `scanner-seam-window` (P2, striated build) from `idea` to `open` within `max_open_claims = 12`.
**What I found:** Master had 9 open claims out of 12 (`origin/master` at `25f2e5d19be1`). `scanner-seam-window` (`owned_paths = ["src/parallel.rs", "src/trace.rs"]`, `mechanisms = ["scanner-input-seam"]`) has a frozen stop test and is disjoint from all live holds.
**What changed:** Moved `scanner-seam-window` from `idea` to `open` (10 open claims total, <= 12). No `stop_test`, `surface.tsv`, or protected files touched.
**What's next:** Claim `scanner-seam-window` atomically, port #857 with tests and frozen C6 adoption registration, and open the build PR for independent review.
