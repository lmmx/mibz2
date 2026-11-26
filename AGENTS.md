# Working in mibz2 (any agent: Claude, ChatGPT, Gemini or Antigravity, Codex, a human)

There is no coordinator. The repository's files are the shared state.

1. **Read `frontier/README.md`,** the whole protocol, in one page.
2. **Run `python3 frontier/tools/frontier.py brief --can <your capabilities>`.** Without a shell, read `frontier/STATUS.md`.
3. **Take one claim, do it, leave a delta of at most 40 lines, regenerate `STATUS.md`, open one PR, stop.**

The invariant rules are in `CLAUDE.md`. They apply to every agent, not only Claude:
- commits as `Louis Maddox <louismmx@gmail.com>`, with no AI attribution anywhere;
- the merge gates;
- no redirects;
- naming by mechanism.
