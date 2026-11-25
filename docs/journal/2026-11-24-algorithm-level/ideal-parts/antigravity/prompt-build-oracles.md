# Antigravity (Neon): build the four missing oracles for the ideal-parts matrix

Given to Louis to paste, 25 November 2026. This is a build task, separate from the mathematics pass (`prompt-1.md`). Either can run first; they don't depend on each other.

---

Work in the lmmx/mibz2 repository on a new branch `research/ideal-oracles`, from `origin/master`. Set `git config user.name "Louis Maddox"` and `git config user.email louismmx@gmail.com`. No AI attribution in any commit, PR title or PR body. Never put a PR or issue number straight after fix, fixes, close, closes, resolve or resolves: GitHub reads that as "close it". Write "(#835)" or "PR 835".

**The task.** The ideal-parts matrix (`docs/journal/2026-11-24-algorithm-level/ideal-parts/`) measures the decoder with each of its parts made *ideal*: its output supplied, already computed, recorded outside the clock. Read `README.md`, `parts.md` and `design.md` there first.
- **Two oracles exist:** S, the scanner (`scan-oracle`, `src/scan_oracle.rs` on branch `bench/four-arm`, PR 835), and E, emission (`supplied-codes`, PR 821).
- **Build the other four:** C, M, R and D, as defined in `parts.md`.

**The template is `src/scan_oracle.rs` on `bench/four-arm`.** Read it in full and copy its shape:
- a process-global mode, `Real`, `Record` or `Replay`, that the harness sets between decodes;
- a `Record` pass that runs the part as shipped and stores its output by block index, sharing repeated blocks;
- a `Replay` mode in which the part's own work is skipped. Its recorded output is handed over (one copy, charged to the handover) to the next stage, exactly where the real part's output would go;
- the store's bytes reported.

**The four:**
1. **C, construction** (target T2), feature `ideal-construction`. Replay supplies each block's built structures (Open's runs, retained map and induction) to emission. MTF/RLE2 still runs, its output discarded at the point C would consume it.
2. **M, MTF/RLE2** (target T3), feature `ideal-mtf`. Replay supplies each block's L, the BWT column, to construction. The worker's MTF and RLE2 inversion is skipped.
3. **R, RLE1, CRC and output** (target T6), feature `ideal-rle1`. Replay supplies each block's final bytes to the in-order reader. RLE1 expansion and the block CRC are skipped, and emission still runs.
4. **D, delivery**, feature `ideal-delivery`. This is a reader mode, not an oracle: it takes each block in order and discards it, without copying to the caller. It measures what the in-order reader and the copy cost.

**Hard rules:**
- **Off by default:** each oracle is a research feature, so the default build is byte-for-byte unchanged in behaviour. Put a retention note above each entry in `Cargo.toml`'s `[features]`, as the existing research features have: its purpose, and its removal condition ("the ideal-parts matrix is analysed").
- **Byte-identical output:** in `Replay`, the decoder's final output must equal the shipped decoder's, byte for byte. Test it at 1, 4 and 8 workers on the bench corpora: wikidata (`bench/wikidata.py` builds it from the `bench-data` branch) and the `ab gen corpora` set. Compare SHA-256s, and put the test in the PR.
- **The removed work is really removed:** in `Replay`, the skipped part's code must not run. Show it, by a counter under the research feature, or with the activity ledgers (`trace` feature), in which the part's busy time should be near zero.
- **Minimal hooks:** new code goes in new files (`src/ideal/…` or one file per oracle). Hooks in existing code are as small as `scan_oracle`'s were in `src/parallel.rs`: one branch on the mode at each hand-over point.
- **Don't touch:** `examples/ab.rs`, `bench/`, `.just/`, or `src/scan_oracle.rs`'s logic. The Pathfinder owns the recipe and the matrix binary, and integrates your oracles into them.
- **No allocator changes,** and no new `unsafe` unless the module already uses it and you justify it in the PR.
- **Checks:** `cargo test`, `cargo test --features <each new feature>`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --check`.

**Deliver:**
- one PR against master, titled "research(ideal-parts): oracles for construction, MTF/RLE2, RLE1/CRC/output and delivery";
- its body: each oracle's hand-over point (file:line), its store size on wikidata ×4, the byte-identity table, and the evidence that the removed work is gone;
- **`status.tsv` updated** in `docs/journal/2026-11-24-algorithm-level/ideal-parts/`: built and checked, for each oracle you deliver;
- a short `FROM: neon / TO: coordinator / TYPE: REVIEW` comment on #145 linking the PR at its head.

**If an oracle can't be made clean,** say so in the PR, with what blocks it, and deliver the others. Examples: a part whose output can't be handed over without re-running another part, or whose replay would itself dominate the cost. A clear "C can't be isolated because …" is a useful result.
