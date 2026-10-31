# P74-C full-decoder trial: the i9-10900K run card (harness v1.2)

**Superseded pins:** v1.1 was head `3870c12` (pin `e3f0940`). Use v1.2 below (amendment 2, `docs/journal/2026-10-31-p74c-decoder-v1.2-validation/results.md`).

**What it runs:** the frozen trial `2026-10-31-p74c-decoder-protocol.md`, with amendment 1
(harness v1.2: amendments 1 and 2). It is primary at w = 4 on 4 physical cores, with 20 paired rounds of B,
A and A′. There are no code, threshold or corpus changes.

| Item | Value |
|---|---|
| Branch | `research/p74c-decoder` |
| Head (verify before running) | `33e0618b1e70190f4e8390eb69ebe2778142fc77` |
| Pin (`bench/p74c-decoder-pin`) | `b406a3bcfcd61d8189e56143c9e48b565ad06688` (the runner checks that HEAD differs from it only in the pin file) |
| Recipe | `just bench-p74c-decoder i9-10900K` (`.just/bench.just:222`, which runs `bench/p74c_decoder_run.sh`) |
| Toolchain | rustc 1.90.0, installed by the runner if it is missing; `RUSTFLAGS` is cleared |
| Results | `bench/results/p74c-decoder/i9-10900K/` and its `.tar.gz` (git-ignored), or under `P74C_RESULTS_DIR` |
| Duration | about 30–60 minutes (≈ 8,200 child decodes) |
| Validated by | v1.1: CI run 41198956443; v1.2: the lifecycle tests, CI integrity run 41242541582 (3 of 3 jobs) and a local dry run; see `2026-10-31-p74c-decoder-v1.2-validation/results.md` |

**Steps:**

```bash
cd ~/dev/mibz2
git fetch origin
git switch research/p74c-decoder
git pull --ff-only
git rev-parse HEAD   # must print 33e0618b1e70190f4e8390eb69ebe2778142fc77
just bench-p74c-decoder i9-10900K
```

- Make no edits, pulls or branch changes between checking HEAD and the run.
- Close the browser and keep the machine quiet.
- The runner refuses a dirty tree, a HEAD that isn't the pin, an existing label, or any
  input whose hash differs from `frozen.json`.
- Keep the archive (`bench/results/p74c-decoder/i9-10900K.tar.gz`) and send it for
  analysis.

**Read the result alongside:**
- `2026-10-31-p74c-decoder-rss-note.md`: B's extra peak RSS on the small controls is a
  32-byte `ll` reservation mismatch, recorded before the result;
- `2026-10-31-p74c-decoder-ci-41198956443/results.md`: the exploratory figures and the
  model's +1.5% to +3% projection at w = 4.
