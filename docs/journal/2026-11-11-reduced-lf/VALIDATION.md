# Validation record — 2026-11-11

- Python 3.12.14, standard library only; assertions enabled.
- Initial full run: exit 0, empty stderr. Source and schema-1 results preserved
  in commit `51a15284150c5c9960eb234fae5c0d79d33ee043`.
- Instrumentation completion: exit 0, empty stderr. Added cycle coverage,
  inverse-word copying, comparison with fresh updates and total-transition
  counters, plus a termination assertion. Fixed bounds and verdict unchanged.
- Final complete reproduction: exit 0, empty stderr;
  `cmp results.json reproduced.json` passed. Every original counter and all
  minimal witnesses were also compared with the first commit and are identical.
- `python3 -O experiment.py` was checked to fail explicitly with
  `Assertions must be enabled`, preventing a silently unchecked run.
- `ruff format` and `ruff check`: pass. The first lint reported missing
  docstrings, which were supplied; no assertion failure occurred in any run.
- `git diff --check`: pass.
- Extra inspection of an already-exhausted input: `cbaa`, black runs `{0,2}`,
  keeps the first-domain and last-image run black. Asserted returns are
  `{0: (3,(0,)), 2: (0,(2,)), 3: (2,(2,1))}`; interval construction agrees.

Reproduce with the commands in README.md; SHA256SUMS records artifact digests.
The first lint launch encountered read-only default uv cache/tool directories;
using `UV_CACHE_DIR` and `UV_TOOL_DIR` under `/tmp` resolved that tool setup.

Source/coordination limitations: HAL denied thesis access. Author preprints
were read through the web tool. The repository's direct-HTTP `coord.py audit`
failed with `Tunnel connection failed: 403 Forbidden`; it did **not** pass.
Current channel/inbox material was read through the GitHub connector instead.
The pre-data comment was posted and read back through that connector;
`coord.py route` selected #145 and its local routing check passed.

No timing, real blocks, production tests, workstation execution, independent
mathematical review or B5 certification was part of this research experiment.
