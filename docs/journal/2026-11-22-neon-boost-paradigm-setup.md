# Setup journal: `research/neon-boost-paradigm`

*Date:* 2026-11-22 (UTC)  
*Purpose:* Isolated environment setup and connectivity test only; no mathematical investigation, research cycle, subagent launch, Boost invocation, or benchmark performed.

## Base state and branch

- **Repository:** `lmmx/mibz2` (`/home/antigravity/lmmx/mibz2`)
- **Base ref:** `origin/master`
- **Base SHA:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`docs(changelog): 0.5.11's crossover, the team threshold stays (#713)`)
- **Isolated branch:** `research/neon-boost-paradigm` (branched directly from `origin/master`, no production files modified, no merges)

## Charter documents read

All four charter documents were read from `8ff1187af6a920f7d57fd338680db3a5633a3ac5`:

- `docs/team/research-charter/MASTER-PROMPT.md`
- `docs/team/research-charter/CONTEXT.md`
- `docs/team/research-charter/METHOD.md`
- `docs/team/research-charter/FRONTIER.md`

## Remote branch and data accessibility

- Default clone fetch refspec is single-branch (`+refs/heads/master:refs/remotes/origin/master`); additional remote branches are accessible via `git ls-remote` and explicit `git fetch origin <branch>:refs/remotes/origin/<branch>`.
- Verified remote heads via `git ls-remote --heads origin`:
  - `master`: `8ff1187af6a920f7d57fd338680db3a5633a3ac5`
  - `data/admitted-bwt`: `41c03a5b22d630adb171231f5225da348b8055af`
  - `audit/neon-cycles-1-2`: `de8ebdbf0e660e22b65dbd69ded3d2c3fb0afde4`
  - `bench/open-admission`: `c4d9ca0a77dba1510e3994bdbcd44e147091315c`
  - `research/neon-bipolar-contraction`: `4598c5bf7d20b58d69324e5875bf410280bb1398`
  - `research/carbon-depth`: `e4cef6828dfe61202cf38b41c666daf9a4d45d1b`
  - `research/carbon-kitchen-sink`: `5b90e08deb5fa16c2ff612caa83bf627c2f5a50f`
  - `research/carbon-word-transport`: `34ddbdd69e783de6d72d230056da5532819e3662`
  - `research/carbon-wormhole`: `d6958f35c3b10b753ba221aaa9280416177d1d5b`
  - `research/open-phase-split`: `b12b5ff83b3f38c45d759d0e9e3b216b0285d2fd`
  - `research/run-network-census`: `bd781368e772fbd689b9ffa097904ec73a64a23c`
- Fetched `data/admitted-bwt` into `refs/remotes/origin/data/admitted-bwt` without disturbing the working tree, and verified SHA-256 and byte lengths of all six blocks (`block-0.L`, `block-2.L`, `block-85.L`, `block-143.L`, `block-207.L`, `block-210.L`) against `admitted-bwt/manifest.json`.

## Environment and toolchain

- **OS / kernel:** Linux `6.12.105+` (`x86_64`)
- **CPU / memory:** 2 vCPUs (`Intel Xeon Platinum 8481C`, L1d 48 KiB, L2 2 MiB, L3 105 MiB), 7.8 GiB RAM
- **Rust:** `rustc 1.99.1 (58cf8f9c3 2026-10-13)` / `cargo 1.99.1 (5f94df478 2026-08-27)` installed under `stable-x86_64-unknown-linux-gnu`
- **C/C++:** `gcc 12.2.0`, `clang 14.0.6`, `cmake`
- **Python / tools:** `Python 3.11.2`, `uv 0.13.7`, `gh 2.103.3`
- **Scorer:** `docs/team/acquisition/check_manifest.py` verified operational against `docs/team/acquisition/example-arc-walk.toml`

## Caveats / setup notes

1. **Rust toolchain selector (`rust-toolchain.toml` vs read-only `RUSTUP_HOME`):** `rust-toolchain.toml` pins `channel = "1.99.1"`, whereas `/usr/local/rustup` is mounted read-only and has only `stable-x86_64-unknown-linux-gnu` installed (which is itself `rustc 1.99.1`). Running bare `rustc`/`cargo` fails attempting to sync `1.99.1` into `/usr/local/rustup/tmp`. Invoking with `+stable` or `RUSTUP_TOOLCHAIN=stable` resolves cleanly to the exact `1.99.1` toolchain without modifying `rust-toolchain.toml`.
2. **Single-branch clone refspec:** Only `master` is fetched by default (`+refs/heads/master:refs/remotes/origin/master`). Other branches (`data/admitted-bwt`, prior `research/*` branches) are fetched explicitly on demand.

## Readiness status

**READY** for future mathematical investigation on `research/neon-boost-paradigm`. Awaiting next instruction.
