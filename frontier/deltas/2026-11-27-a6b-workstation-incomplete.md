claim: a6b-endpoint-run
by: codex
outcome: partial
evidence: b87790e5f6aec4871f36e56d110151e3064834b1; run-20261127T072936Z.tar.gz SHA-256 fc6f8ad2320e5a40c866d75ecdc4203253e1b55746c1bcf1ebb5ea3e88080f43; #1053; #1085

**Hardware receipt:** Recomputed the archive SHA-256 and matched its evidence-side manifest. Intel i9-10900K reported 10 cores/20 CPUs and Rust/Cargo 1.99.1. The recipe was 02e492a7 and commits.txt declared faa9df70 for control and feature-enabled candidate. The queue invocation reached the i9.
**Outcome:** INCOMPLETE (exit 1), before compilation. The runner fetched faa9df70 into its shallow checkout, but the scratch clone could not check it out. Six archive members contain the console, outcome, toolchain, topology and intended commits. There are no binary hashes, runs.jsonl, summary or timing measurements. No C6 hardware criterion can be scored. Preserve the receipt and existing source/registration.
**Follow-on:** Reopen the existing recipe claim for the source checkout and shallow-checkout regression test. Its narrowed file ownership is disjoint from the shared queue. Use a newly reviewed green recipe coordinate for the first measured attempt; keep the original evidence and signal history.
