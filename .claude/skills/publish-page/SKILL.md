---
name: publish-page
description: Publish blog posts autonomously once they are news for an outside reader and pass the strict build, render check and reader gate; improve after merging. Other site pages retain the visual verification procedure.
---

# Publishing a page

**First, the post must be news and must pass the reader gate.** It carries a theoretical
discovery or an advance in the software, for a reader outside the project; process accounts,
digests and lone negative results go in the lab notebook (`docs/notebook/`). Run
`python3 docs/scripts/check_prose.py docs/site/blog/<post>.md`: its dek and reader gate must
pass (`write-blog-post`, "First: is it a post at all?").

**For a blog-only PR, publish on the strict build, `check_render.py`, `check_prose.py` and the
applicable mechanical merge checks.** Standing authorisation from Louis, 26 November 2026: any agent,
including the author, merges without waiting for human approval, editorial sign-off, preview,
`look`, or deployment completion. Do not leave a publishable PR for the maintainer.
The steps below are improvements to the live blog post, not prerequisites; corrections ship as
small follow-up PRs. Verify deployment afterwards when useful, and distinguish merged from live.
Protected files, production changes and other site pages retain their existing rules.

For other pages:

A page is ready when its rendered version at the PR's final head has been looked at. A green
strict build, the site checks and a read of the Markdown don't show how it renders.

0. For a blog post, run the `link-and-gloss` skill as an improvement pass: `python3 docs/scripts/link_audit.py
   docs/site/blog/<post>.md`, then resolve each line with a link, a gloss, a definition or a cut.

1. Build, and run the checks on the build:

   ```sh
   just docs-build
   uv run --no-project -p 3.13 python docs/scripts/check_render.py
   uv run --no-project -p 3.13 --with playwright==1.56.0 python docs/scripts/look_pages.py --changed origin/master
   ```

   `check_render.py` finds Markdown that rendered as something else: a list run into a
   paragraph, a wrapped line starting `#145` made into a heading, a nested list flattened.
   `look_pages.py` opens each changed page in Chromium, at desktop and phone width, light and
   dark. It fails on overflow, broken images, labels outside their figures, script errors
   and unrendered maths, and saves a screenshot of every section in `site-shots/`.

2. Read every screenshot, at both widths and in both themes. Look at each figure, table,
   equation and code block as a reader would.

3. Fix, rebuild and look again. CI runs both checks on the PR and keeps the screenshots as
   the `page-shots` artifact. Once the preview is deployed, run `look_pages.py` against it
   with `--base-url`.

4. For non-blog pages, before merging record in the PR which commit was looked at and
   what was found and fixed. For blog-only PRs, record these observations when made, without
   holding publication.

5. A new page goes to master through its PR, merged on green. Never push a new post to master
   directly; direct commits are only for amendments Louis asks for (the frontier's blog board, `frontier/README.md`; formerly coordinate-blog-post, "The
   PR opens first").

**No redirects, ever** (Louis, 25 November 2026). When a page's URL changes, move it: never add a
redirect, a moved-page stub or a meta refresh at the old path, whoever asks. Only Louis can ask
for one; see CLAUDE.md, "Site URLs: no redirects, ever".
