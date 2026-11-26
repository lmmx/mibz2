# Antigravity (Neon): build the zero-copy oracles you designed (IDEAL-PARTS-2 §3)

Given to Louis to paste, 26 November 2026. This is a build task, characterisation only.

---

Work in the lmmx/mibz2 repository on a new branch `research/ideal-zero-copy`, **from `origin/bench/ideal-matrix-next`** (PR 909), not master. S's and E's oracles (`src/scan_oracle.rs`, `src/supplied_codes.rs`, `src/open/supplied.rs`) and the ledgers (`src/character.rs`) live only there. Open the PR **against `bench/ideal-matrix-next`**. *(Corrected 26 November: the first version said master, where those files don't exist.)* Use Louis's identity (`git config user.name "Louis Maddox"`, `git config user.email louismmx@gmail.com`), with no AI attribution anywhere. Never put a PR or issue number straight after fix, close or resolve.

**The task.** Implement the zero-copy oracle change from your `research/neon-movement/IDEAL-PARTS-2.md` §3:
- lock-free stores (`OnceLock`);
- `BlockBuf::Shared` for R;
- borrowed hand-overs for C and E;
- splitting `HandE_deliver` from `E_consume`, with an `E_full` mode that removes the word spelling too;
- your proposed accounting, so the floor's parts can be read directly.

**The aim:** measure the decoder's own residual, which the full matrix could not (`docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md` §4).

**Hard rules, as for PR 895:**
- every mode is off by default;
- byte-identical output in every mode at 1, 4 and 8 workers on wikidata, wikidata ×4 and the `ab gen corpora` set (SHA-256);
- show by counters or ledgers that the removed work is gone;
- new code in `src/ideal/`, with minimal hooks;
- don't touch `bench/`, `.just/` or `examples/ab.rs`: the Pathfinder owns the recipe;
- no allocator changes, and no new `unsafe` unless the module already uses it and you justify it;
- `cargo test`, `clippy --all-targets --all-features -D warnings` and `fmt --check`.

**Deliver:**
- one PR against `bench/ideal-matrix-next`, its body giving each hand-over's cost before and after on wikidata ×4;
- `ideal-parts/status.tsv` updated;
- a short `FROM: neon / TO: coordinator / TYPE: REVIEW` on #145 linking the PR at its head.
