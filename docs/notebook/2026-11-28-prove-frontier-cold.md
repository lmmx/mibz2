# Proving the frontier cold: a stranger's end-to-end walkthrough on a private bare remote

*28 November 2026 — `prove-frontier-cold` (`probe`, smooth zone). Executed strictly against an isolated private bare clone of `master` (`77de5b69`, `/tmp/prove-frontier-cold/remote.git`) with two fresh participant clones (`stranger-a`, `stranger-b`), following only `frontier/README.md`, `CLAUDE.md`, `AGENTS.md`, `frontier/CONTINUATIONS.md`, `frontier/deltas/README.md`, and `frontier/flags/README.md`. Nothing was pushed to `lmmx/mibz2` during testing.*

---

## Summary tally

- **16 findings:** **5 `breaks`**, **6 `misleads`**, **5 `friction`**.
- **What passed cleanly:** atomic `claim` CAS + double-claim refusal; `owned_paths` exact/parent-dir/component-boundary overlap (`src/` conflicts with `src/parallel.rs`, `src/parallel` does not false-positive); `mechanisms` disjointness; `needs_louis` & non-open status refusals; `limits.toml` `max_live_holds` enforcement; stale-local-checkout safety (`claim` and `verify` both read authoritative `origin/master` for `needs_louis` and open flags, including path-scoped `flags: src/parallel.rs`); fail-closed non-GitHub behavior in `brief`, `take-review`, and `continuation.py` (`pr:` -> `[unknown]`); subdirectory execution (`cwd = frontier/`); `ledger.py` (`default`, `--missing`, `--check`); `frontier.py news`, `status`, `arrive`/`population`/`depart`; `continuation.py` CAS, duplicate-declare refusal, missing-disposition refusal, wrong-session refusal, and `--independent` self-resume refusal (`EXIT=16`).
- **Follow-on claim:** `frontier/claims/frontier-cold-loop-fixes.toml` (`priority = 2`, `status = "idea"`, sequenced behind `#1341` and `brief-state-first` on `frontier/tools/frontier.py`).

---

## `breaks` (5)

### B1. `continuation.py`: `manual:<text>` pins are permanently unresumable
- **Where:** `frontier/tools/continuation.py:548–559, 940` vs `frontier/CONTINUATIONS.md:22`.
- **Evidence:** `CONTINUATIONS.md:22` promises `` `manual:<text>` | never on its own: resume by hand ``. After declaring/chaining `prove-frontier-cold--manual-step` on `manual:wait for human`:
  ```text
  $ python3 frontier/tools/continuation.py resume prove-frontier-cold--manual-step --session s-man-1 --by stranger-a
  EXIT=15
  not-ready: waiting: manual: wait for human; resume by hand (polls 1)
  ```
  `continuation.py resume --help` accepts only `--session`, `--by`, `--hours`, `key` — there is no `--manual` / `--force` flag. Any `manual:` continuation is stuck in `waiting` forever and can never be resumed or marked `done`.
- **Proposed fix:** Add `--manual` to `continuation.py resume` permitting explicit human/peer acquisition of `manual:` pins (while still rejecting automatic readiness in `view`/`watch`), and document `--manual` in `frontier/CONTINUATIONS.md:22`.

### B2. `frontier.py` & `continuation.py`: unbounded, zero, and negative `--hours` break lease TTL bounds
- **Where:** `frontier/tools/frontier.py:3080, 3099, 3153`; `frontier/tools/continuation.py:562, 1031, 1086`.
- **Evidence:**
  ```text
  $ frontier.py claim prove-frontier-damaged --by stranger-b --hours 0      -> EXIT=0 (expires == claimed_at)
  $ frontier.py claim prove-frontier-damaged --by stranger-b --hours -5     -> EXIT=0 (expires 5h in past: 2026-11-28T11:02:44+00:00)
  $ frontier.py claim prove-frontier-damaged --by stranger-b --hours 9999   -> EXIT=0 (locks claim until 2028-01-19T07:02:48+00:00, 417 days!)
  $ frontier.py renew prove-frontier-cold --session S --hours 9999          -> EXIT=0 (renewed until 2028-01-19T07:04:09+00:00)
  $ frontier.py renew prove-frontier-cold --session S --hours -5            -> EXIT=0 (retroactively expires own lease!)
  $ frontier.py claim prove-frontier-damaged --by stranger-b --hours abc    -> EXIT=1 (raw Traceback ValueError)
  $ continuation.py resume prove-frontier-cold--indep-gate --session s --by stranger-b --hours -5   -> EXIT=0
  $ continuation.py resume prove-frontier-cold--indep-gate --session s --by stranger-b --hours 9999 -> EXIT=0 (acquired_until = +417d)
  ```
- **Proposed fix:** Validate `math.isfinite(hours) and 0 < hours <= MAX_HOURS` (e.g., `24` for `frontier.py`, `8` for `continuation.py`), exiting `2` with a clean error message on invalid or out-of-range values.

### B3. `frontier.py mark`: `--pr <N>` is silently dropped on `--status review`, and `check` accepts `status = "review"` with no `pr`
- **Where:** `frontier/tools/frontier.py:3173–3187` vs `frontier/README.md:48, 177`.
- **Evidence:** `frontier/README.md:48` says: `mark status = "review" and pr = <N> on the head`.
  ```text
  $ frontier.py mark prove-frontier-cold --status review --pr 999 -> EXIT=0
  prove-frontier-cold: status review (commit this in your PR)
  # git diff shows ONLY `status = "review"` — `pr = 999` was silently dropped!
  $ frontier.py check -> EXIT=0 (0 errors)
  ```
- **Proposed fix:** Teach `mark` to accept `--pr N` (updating or inserting `pr = N` in the TOML), and decide explicitly whether `check` should require `pr > 0` on `status = "review"` or whether `README.md:48` should state that `brief` discovers PRs directly without `pr = <N>`.

### B4. `frontier.py check`: flag-resolution check enforces the inverted invariant
- **Where:** `frontier/tools/frontier.py:2307–2312` vs `frontier/README.md:63`, `frontier/flags/README.md:18`.
- **Evidence:** Both docs state: *"A different agent from the flagged work's author resolves it. An implementer never dismisses an objection to its own work."* Yet `frontier.py:2309` checks `if not rb or rb == h.get("by"):`:
  ```text
  # Flag filed by stranger-b (by: stranger-b) against stranger-a's work:
  # When stranger-b sets status: resolved / resolved_by: stranger-b:
  $ frontier.py check -> EXIT=1
  error: flags/2026-11-28-cold-flag-test.md: resolved flag needs resolved_by: by a different agent from by:
  ```
  Because `by:` is the **flag filer** rather than the flagged work's author, `check` forbids the flag filer from closing its own flag once verified fixed, while permitting the flagged work's author (`stranger-a`) to set `resolved_by: stranger-a` and dismiss the flag on its own work.
- **Proposed fix:** Add an optional `flagged_author:` header (or resolve against the flagged claim's author/holder rather than `by:`) so `check` enforces the actual documented invariant, and align `frontier/flags/README.md:18` and `frontier.py:2309–2312`.

### B5. `frontier.py`: hand-rolled `argv` parsing crashes with raw Python tracebacks on missing/bad arguments
- **Where:** `frontier/tools/frontier.py:3079–3187`.
- **Evidence:** 8/8 tested missing/malformed invocations crash with raw tracebacks:
  ```text
  $ frontier.py claim                                           -> EXIT=1  IndexError: list index out of range (argv[2])
  $ frontier.py verify / renew / release / mark                 -> EXIT=1  IndexError: list index out of range (argv[2])
  $ frontier.py claim prove-frontier-cold --by                  -> EXIT=1  IndexError: list index out of range (opt lambda)
  $ frontier.py brief --can                                     -> EXIT=1  IndexError: list index out of range (opt lambda)
  $ frontier.py take-review prove-frontier-cold --pr notanint   -> EXIT=1  ValueError: invalid literal for int()
  ```
- **Proposed fix:** Replace the `opt` lambda and `argv[2]` indexing in `frontier.py:main()` with stdlib `argparse` subparsers (matching `continuation.py` and `ledger.py`).

---

## `misleads` (6)

### M1. `frontier.py`: unknown flags and typos (`--session` on `claim`, `--agent`, `--hour`, `--bogus`) are silently ignored
- **Where:** `frontier/tools/frontier.py:3079, 3098–3100`.
- **Evidence:**
  ```text
  $ frontier.py claim prove-frontier-cold --by antigravity --session cold-ag-01 -> EXIT=0
  acquired prove-frontier-cold ...; your session is a494b370f849   # silently ignored --session!
  $ frontier.py claim prove-frontier-cold --agent stranger-a --hour 2 --bogus xyz -> EXIT=0
  # Acquired an 8-hour hold as holder = "unknown" with zero error or warning!
  ```
- **Proposed fix:** Eliminated automatically by moving `frontier.py:main()` to `argparse` (B5).

### M2. `frontier.py claim` & `arrive`: required `--by` / `--as` silently default to `"unknown"`
- **Where:** `frontier/tools/frontier.py:3099, 3112` vs `frontier.py --help:8, 18`, `frontier/README.md:23–24`.
- **Evidence:** Help and README list `--by AGENT` and `--as FAMILY` as required (unbracketed).
  ```text
  $ frontier.py claim prove-frontier-cold -> EXIT=0   # writes holder = "unknown" to origin/hold/prove-frontier-cold!
  $ frontier.py arrive                    -> EXIT=0   # publishes present as unknown/<id> (idle)
  ```
- **Proposed fix:** Make `--by` required on `claim` / `take-review` and `--as` required on `arrive`.

### M3. `continuation.py resume`: printed follow-up `done` command omits required `--session`
- **Where:** `frontier/tools/continuation.py` (`_next_step`, printed on `resume` line 565).
- **Evidence:**
  ```text
  $ continuation.py resume prove-frontier-cold--wait-hold --session s-cold-1 --by stranger-a -> EXIT=0
  ... then record `continuation.py done prove-frontier-cold--wait-hold --disposition ...`.
  # Copy-pasting that exact command fails:
  $ continuation.py done prove-frontier-cold--wait-hold --disposition "..." -> EXIT=2
  continuation.py done: error: the following arguments are required: --session
  ```
- **Proposed fix:** Pass `session` into `_next_step` and print `` `continuation.py done <key> --session <session> --disposition ...` ``.

### M4. `frontier.py verify` & `renew`: released holds are misreported as `"has lapsed"`
- **Where:** `frontier/tools/frontier.py` (`mine()`, called from `verify:3134` and `renew:3152`).
- **Evidence:** Immediately after `release prove-frontier-damaged --session 81390b172de9`:
  ```text
  $ frontier.py verify prove-frontier-damaged --session 81390b172de9 -> EXIT=1
  prove-frontier-damaged's hold by 81390b172de9 has lapsed: re-claim it before any consequential action
  ```
- **Proposed fix:** Check `lease.get("released")` in `mine()` and report `"was already released"` distinctly from TTL expiry (`"has lapsed"`).

### M5. `frontier.py --help` & `frontier/README.md:41` omit `review` from `mark --status`
- **Where:** `frontier/tools/frontier.py:17` and `frontier/README.md:41` vs `frontier/tools/frontier.py:3183` and `frontier/README.md:48`.
- **Evidence:** `--help` prints `mark ID --status open|idea|done|dropped|blocked`; `README.md:41` says `done|blocked|open|dropped`; `mark --status bogus` prints `usage: mark ID --status blocked|done|dropped|idea|open|review` (and `mark` also silently edits `frontier/blog/*.toml` even though blog cards advance via `stage`, not `status`!).
- **Proposed fix:** Align `--help` and `README.md:41` with `STATES` (and refuse `mark --status` on blog cards unless explicitly intended, or support `--stage`).

### M6. `CLAUDE.md:25, 77` & `frontier/README.md:56–57`: merge gate omits `coord.py`'s path, omits `GITHUB_TOKEN`, and names MCP-only `expectedHeadSha`
- **Where:** `CLAUDE.md:25, 77, 79`; `frontier/README.md:56–57`.
- **Evidence:** Docs say `coord.py head-sha --pr <n> --sha <sha>` without path (a stranger gets `command not found`; actual path is `python3 .github/coordination/coord.py head-sha`, and unauthenticated calls hit GitHub's 60/hr IP 403 unless `GITHUB_TOKEN="$(gh auth token)"` is exported) and `pass that SHA as expectedHeadSha` (MCP parameter; `gh pr merge` uses `--match-head-commit <sha>`).
- **Proposed fix:** Document the exact shell command (`GITHUB_TOKEN="$(gh auth token)" python3 .github/coordination/coord.py head-sha --pr <N> --sha <SHA>`) and name both `expectedHeadSha` (MCP) and `--match-head-commit` (`gh`).

---

## `friction` (5)

### F1. `frontier.py --help` exits `2`
- **Where:** `frontier/tools/frontier.py:3188–3190`.
- **Evidence:** `python3 frontier/tools/frontier.py --help` -> `EXIT=2` (unknown-command fallback), unlike `continuation.py --help` (`EXIT=0`) and `ledger.py --help` (`EXIT=0`).
- **Proposed fix:** Handle `-h` / `--help` via `argparse` (`EXIT=0`).

### F2. `brief` reprints 57 lines (~16 KB) of `frontier/README.md` before state, and `--can shell,rust,repo-write` hides `explore`/`synthesise`
- **Where:** `frontier/README.md:7–64` (already tracked by `brief-state-first` and `rule-budget`); `frontier/README.md:21, 29`.
- **Evidence:** Bare `brief` (`CLAUDE.md:9`) shows `## Or explore or synthesise: synthesise, explore`; `brief --can shell,rust,repo-write` drops that section silently because `explore`/`synthesise` list `needs = ["long-context"]` and `README.md:21` does not tell a stranger when to include `long-context`.
- **Proposed fix:** Keep `brief-state-first` moving state above prose, and note in `README.md:22` that `long-context` gates `explore`/`synthesise`.

### F3. `frontier.py release` on an already-released hold pushes a duplicate `released = true` commit
- **Where:** `frontier/tools/frontier.py:3160–3171`.
- **Evidence:** Calling `release prove-frontier-damaged --session 81390b172de9` twice exited `0` twice (`released prove-frontier-damaged`), writing a second identical `released = true` commit to `hold/prove-frontier-damaged`.
- **Proposed fix:** If `lease.get("released")` is already `True`, print `prove-frontier-damaged already released` and return `0` without pushing.

### F4. Missing `--session` on `verify`/`renew`/`release` prints `is not held by session None`
- **Where:** `frontier/tools/frontier.py:3134, 3152, 3162–3163`.
- **Evidence:** `frontier.py release prove-frontier-cold` -> `EXIT=1: prove-frontier-cold is not held by session None`.
- **Proposed fix:** Resolved by `argparse` requiring `--session` (B5).

### F5. `continuation.py done --next-action` example drops `next` to `""`; `show` prints raw Unix epochs; `CLAUDE.md` is 245 lines vs `README.md:285`'s `"near 120 lines"`
- **Where:** `frontier/CONTINUATIONS.md:46–48`, `frontier/tools/continuation.py:show()`, `frontier/README.md:285` vs `CLAUDE.md` (245 lines).
- **Evidence:** Running the exact `done --next-action ... --next-dep ...` command from `CONTINUATIONS.md:46–48` writes `next = ""` on the chained continuation; `show` prints `declared_at = 1795882049` (raw epoch) rather than ISO-8601.
- **Proposed fix:** Include `--next "..."` in `CONTINUATIONS.md:48` (or warn on empty `--next`), print ISO timestamps in `show` comments alongside raw fields, and reconcile `README.md:285` under `rule-budget`.

---

## Threads not tested within the 40-minute cap

- Live GitHub PR API paths for `take-review` / `yield-review` / `finish-review` / `cancel-review` (tested fail-closed on the private bare remote; live PR API transitions need a GitHub test repo or mocked `gh api`).
- `continuation.py watch` concurrent waiter contention (`EXIT=11` / `EXIT=14`) and half-TTL renewal over > 7.5 min.
- `frontier/tools/frontier.py check` against `blog/*.toml` `news` / `changes` / `evidence` error paths and `docs/site/scripts/check_prose.py` reader gate.
