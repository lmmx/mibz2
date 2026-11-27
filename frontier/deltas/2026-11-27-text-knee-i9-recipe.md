claim: text-knee-i9
by: antigravity
outcome: partial
evidence: docs/journal/2026-11-27-text-knee-i9/registration.md

## Frozen registration and shallow-safe i9 recipe (`just bench-text-knee`)

- **Claim:** `text-knee-i9` (admitted in `#1135` at `2fed5f18`; `stop_test` preserved verbatim under C2).
- **Files:** `docs/journal/2026-11-27-text-knee-i9/registration.md`, `Justfile` (`import ".just/text-knee.just"`), `.just/text-knee.just`, `bench/text-knee.sh`, `bench/text_knee.py`, `bench/test_text_knee.py`.
- **Pinned text panel (`2fed5f184bf66ddaa50c72779a126dafe7811eef`):**
  - `docs_md` (`docs/**/*.md`, 656 files, `1×` raw → `16×` multistream, 183.7 MB raw / 208 full blocks, wall `.bz2` SHA-256 `857095a4...`, wall raw `ef330c7a...`).
  - `repo_json` (`**/*.json`, 181 files, `5×` raw → `20×` multistream, 189.7 MB raw / 220 full blocks, wall `.bz2` SHA-256 `83fd2eb4...`, wall raw `2778e69b...`).
  - `src_rs` (`src/**/*.rs`, 52 files, `6×` raw → `20×` multistream, 183.1 MB raw / 180 full blocks, wall `.bz2` SHA-256 `6e7e6bb4...`, wall raw `a0486cee...`).
  - Control: `corpora/wikidata.bz2` (`82f65ace...`) base + `4×` multistream (`0b820d14...`, 838.8 MB raw / 932 full blocks) via `origin/bench-data` if absent locally.
- **Shallow-checkout safe (`#1133` §2 invariant):** builds `ab`, `station_balance`, and `windows` in place into private `CARGO_TARGET_DIR` (`rustc 1.99.1`, baseline x86-64 codegen with `RUSTFLAGS` rejected), and fetches `PANEL_COMMIT` / `bench-data` into isolated temporary repos (`git init` + `git fetch --depth=1 --no-tags`).
- **Protocol & scoring:** untimed `ab child libbzip2 1` vs `mibz2 8` + Python `bz2.open` parity on all 4 corpora; 3 interleaved rounds of `station_balance` (5 passes, core 0 CPU 0); 9 interleaved shuffled rounds (`SEED=1135`) across `k = 4..10` × 4 corpora via `ab run --arms mibz2 --prespin-ms 100 --settle 0.2 --max-busy 1.5 --sysfs-state`; exact frozen `CONFIRMED`/`REFUTED`/`INCONCLUSIVE`/`VOID` scoring; one tarball via `bench/bundle.sh`.
- **Verification:** `python3 -W error -m unittest discover -s bench -p 'test_text_knee.py' -v` **11/11 PASS** (zero warnings, incl. real detached `git clone --depth=1 --single-branch --no-local` shallow runner test and pinned panel SHA-256s); `ruff check`, `ruff format --check`, `bash -n bench/text-knee.sh` clean; live Rust 1.99.1 smoke run verified build, `ab env --facts` quota check, `libbzip2 == mibz2` parity on all 4 corpora, `station_balance` on all 4 base streams, `ab child mibz2 4`, and non-i9 `VOID` bundle packing.
- **Next:** independent C6 `take-review`, merge, and workstation queueing under the standing i9 authority.
