# The docs toolchain, pinned as in .github/workflows/site.yml (change both together).
docs_with := "--with zensical==0.0.71 --with mkdocstrings-python==2.0.9 --with ruff==0.17.1"

# The benchmark pages' data, from the bench-results branch (git fetch origin bench-results)
docs-bench:
   python3 docs/scripts/bench_history.py

# Build the site, serve it and rebuild on change
docs-serve: docs-bench
   uv run --no-project -p 3.13 {{docs_with}} zensical serve

# A clean, strict build into site/
docs-build: docs-bench
   uv run --no-project -p 3.13 {{docs_with}} zensical build --clean --strict

# The build, the site checks and the examples in the pages
docs-check: docs-build
   uv run --no-project -p 3.13 python docs/scripts/check_site.py
   uv run pytest docs/site
