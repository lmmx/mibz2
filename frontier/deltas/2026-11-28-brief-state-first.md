claim: brief-state-first
by: claude-code
outcome: done
evidence: frontier/tools/frontier.py (brief_rule, brief_cursor), tests/test_frontier.py::TestBriefRule; bytes measured on bd75fc8

**What was asked:** make `frontier.py brief` print the state and reprint the README's brief section only when this checkout has not seen it, without anyone missing a rule change.

**What I found:** on master bd75fc8, `brief --can shell` printed 24,355 bytes before this change. After it, a checkout's first brief prints the same 24,355 bytes, and every later brief prints 8,474 (−65%). The rule is replaced by a two-line header naming its 12-hex SHA-256 digest and `--rules`. The state section, from `## Needs Louis` on, is byte-identical in all three outputs (sha256 dfe7de0d… for each).

**What changed:**
- `brief` hashes the stripped `<!-- brief -->` section and compares the hash with a local cursor, `$(git rev-parse --absolute-git-dir)/frontier-brief-rule`. Each worktree has its own cursor, it is never committed and it never dirties the tree.
- The full rule is printed when there is no cursor (a fresh clone), when the cursor is unreadable or unrecognised, on `--rules`, and whenever any byte differs. A changed rule opens with "The loop changed since your last checkpoint (rule old -> new): read it again."
- The cursor is written only when the rule is printed. A deleted cursor therefore means the rule is printed again, never that a change is missed.
- No rule text was deleted or reworded; that is the job of `rule-budget`. The usage line in the tool's docstring now names `--rules`.
- Tests (`TestBriefRule`, 2 cases): first brief prints the rule; an unchanged rule gives the digest line; `--rules` forces it; a one-byte change reprints it with the note, once; cursors are per checkout; and a lost or garbage cursor reprints. The full frontier suite passes: 126 tests.

**What's next:** README step 1 still says to read the brief; a session whose output is truncated before the rule ends has still advanced its cursor. If that bites, run `brief --rules`. The cursor could also be written only after a successful exit, but that would not stop truncation by a pager. `rule-budget` (Needs Louis) is the claim that shrinks the rule itself.

**Caveats:** the saving holds only per checkout. Cloud sessions start from a fresh clone, so their first brief still costs the full 24 KB; the saving is on the second and later checkpoints within a session. `renew` and `claim` do not print the brief, so the saving applies only to `brief` calls.
