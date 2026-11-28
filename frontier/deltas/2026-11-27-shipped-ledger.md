claim: shipped-ledger
by: antigravity
outcome: done
evidence: frontier/ledger.tsv; frontier/tools/ledger.py; tests/test_ledger.py; frontier/README.md (section "The ledger")

**What was asked:** Backfill `frontier/ledger.tsv` from `v0.5.14` to `origin/master` with one row per merged PR that changes the software (`src/`, `python/`, `Cargo.*`, `pyproject.toml`, `build.rs` or a feature flag), build `frontier/tools/ledger.py` (listing new merged software PRs since the newest row plus `--missing` and `--check`), add `"The ledger"` to `frontier/README.md` linked from `"The blog board"`, and pass the frozen stop test.
**What changed:**
- Added `frontier/ledger.tsv` (`pr`, `merged`, `effect`, `state`, `measured`, `evidence`) covering all 10 merged software PRs in `v0.5.14..origin/master` (`246` first-parent commits checked against `Cargo.toml` and `src/`):
  - `default` (5): `#1040`, `#1033`, `#1034`, `#1080`, `#1129`.
  - `experimental` (5, all off by default behind Cargo features): `#895` (`ideal-*`), `#1010` (`mtf-fold`), `#1041` (`scan-oracle`), `#1053` (`a6b-endpoint-redesign`), `#1110` (`forward-anchor-lanes`).
  - Every `effect` is one plain user-facing sentence with no inline code, claim ids or `#N` references; every `measured` entry cites concrete figures, input and named machine (`i9`, `VM`, `CI`, `local`) or `not measured`.
- Added `frontier/tools/ledger.py` (stdlib only): default mode prints TSV stubs for merged software PRs after the newest ledger row; `--missing` fails non-zero whenever any root software PR since the last `v*` tag is missing (failing closed on shallow clones where the tag is not an ancestor); `--check` validates header, 6 columns, unique positive PRs, ISO dates, plain-language `effect` (no backticks/`#N`), allowed states, and named-machine measurements.
- Added `tests/test_ledger.py` verifying validation rules, synthetic git repo discovery, and the real `frontier/ledger.tsv` against `Cargo.toml` and `v0.5.14..origin/master`.
- Added `"The ledger"` to `frontier/README.md` and linked it from `"The blog board"` (explicitly authorised in `shipped-ledger.toml`'s `done_when`), and marked `shipped-ledger` `done`.
**Verification:** `ledger.py --missing` exits `0`, `ledger.py --check` exits `0`, `tests/test_ledger.py` and `tests/test_frontier.py` pass, `frontier.py check` reports `0 errors, 0 warnings`, and `ruff check`/`ruff format --check` pass.
**What's next:** Unblocks P2 `blog-cards-cite-ledger` and `claims-match-state`.
