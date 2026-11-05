---
name: publish-page
description: Use before opening or merging a PR that adds or changes a page of the mibz2 site, above all a blog post. The rendered page must be looked at, not just built.
---

# Publishing a page

A page is ready when its rendered version at the PR's final head has been looked at. A green
strict build, the site checks and a read of the Markdown don't show how it renders.

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

4. Before merging, record in the PR which commit was looked at, and what was found and
   fixed.
