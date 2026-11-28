claim: none
by: antigravity
outcome: done
evidence: frontier/tools/continuation.py; tests/test_continuation.py (38/38 PASS); frontier/CONTINUATIONS.md; ruff check & ruff format --check PASS; frontier.py check (0 errors); live continuation.py view PASS

**What was asked:** Fix all proven `continuation.py` defects from `#1386` (`prove-frontier-damaged` Defect #9) and `#1385` (`prove-frontier-cold` B1, B2, M3, F5) without touching `frontier/tools/frontier.py` (held by `#1341`).

**What changed:**
- **`#1386` Defect #9 (malformed `wait/*` ref isolation):** `GitRefStore._git` decodes UTF-8 safely; `GitRefStore._read(tolerate=True)` isolates missing/unreadable/malformed/non-UTF-8/empty `continuation.toml` refs into deterministic `[malformed]` rows in `view()` without hiding healthy continuations (while `get()`/`show` raise clean `ContinuationError`, exit 2); `waiter_live`, `acquired_live`, `timings`, and `view` guard non-int timestamps and priorities via `_safe_int`.
- **`#1385` B1 (`manual:<text>` resume):** Added `--manual` to `continuation.py resume` (explicitly allows resuming `manual:` pins, refuses non-`manual:` pins with exit 16, preserves `--independent` self-resume refusal and CAS); updated `live_probe` hint and `frontier/CONTINUATIONS.md`.
- **`#1385` B2 (`--hours` / `ttl` bounds):** Enforced finite `0 < --hours <= 8` (`0 < ttl <= MAX_ACQUIRE_TTL`) in both `main()` and `resume()`, exiting 2 cleanly on `0`, negative, `NaN`, `Inf`, or `> 8` hours.
- **`#1385` M3 & F5:** Included `--session <session>` in `_next_step()`'s printed `continuation.py done` command; formatted epoch timestamps as ISO-8601 alongside raw epochs in `show`; added `--next` to the `done --next-action` example in `frontier/CONTINUATIONS.md`.
- **Tests & bookkeeping:** Added `TestContinuationHardening` (4 test methods covering all 5 defect families on both `MemoryStore` and a private bare `GitRefStore`; **38/38** PASS) and noted completion of the `continuation.py` scope in `frontier-damage-hardening.toml` and `frontier-cold-loop-fixes.toml` (`why` only; frozen `stop_test` untouched).

**Authority note:** `frontier/tools/continuation.py` is under protected path `frontier/tools/`, but carries zero consequential/merge/gate authority (`TestAuthorityBounds` verified); ordinary author-owned integration applies (`CLAUDE.md` RULES O).
