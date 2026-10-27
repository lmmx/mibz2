# 2026-10-27: The documentation site and the benchmark tracker

## Current State

- The Benchmark workflow (`.github/workflows/bench.yml`, lmmx/mibz2#11) runs on pull requests and master pushes that touch `src/**`, `Cargo.toml`, `Cargo.lock`, `examples/ab.rs`, `examples/windows.rs`, `tests/common/**`, `bench/**` or the workflow, and from the Actions tab
- A pull request's run builds its head and its merge base with master on one runner and times them as two arms of one `ab run` (`mibz2` and `mibz2@base/ab`, the base's own harness binary), 10 rounds at 1 thread and at the runner's cores, the arms' order rotated and reversed round to round (examples/ab.rs `run`)
- A pull request's figures go to one comment, found by its `<!-- mibz2-bench -->` marker and edited in place (bench/comment.py); a pull request's runs are never recorded
- A master push's run times mibz2, pbz2 0.2.0 and lbzip2 and appends the commit's result to the orphan `bench-results` branch: `runs/<commit>.json`, `raw/<commit>.jsonl` and a line of `records.jsonl` (bench/report.py `record`) — nothing is committed to master
- The workload is the crate's six corpora at `ab gen` scale 4 (8–20 MB of output each) and 12 MB of the Wikidata dump's own blocks: 12 byte-range windows of 1 MiB of `wikidata-20261019-all.json.bz2` at (2i + 1)/24 of the file, 431 whole blocks, 11,854,933 B → 209,700,196 B (17.69×) (bench/wikidata.json)
- bench/wikidata.json pins each window's offset, length and SHA-256 and the corpus's SHA-256 `82f65ace…`; bench/wikidata.py reads missing windows by HTTP byte range from dumps.wikimedia.org and builds the corpus with examples/windows.rs, which checks the blocks against libbzip2 and lbzip2
- Master's run copies the windows to the orphan `bench-data` branch when they came from Wikimedia — dumps.wikimedia.org listed dated dumps from 20260914 to 20261023 on 2026-10-27, about six weeks
- bench/report.py gives, per corpus and thread count, the median wall, CPU and peak RSS, MB/s, cycles a byte at the runner's reported clock, the paired ratios of wall, CPU and peak RSS against the reference arm (geometric mean, 95% t-interval over the rounds) and a same-bytes check over every run's output hash
- bench/report.py splits mibz2's CPU on the Wikidata blocks at N threads into the scanner thread (`mibz2-scan`), the workers (`mibz2-0`…) and the rest, from ab's per-thread CPU (`thread_cpu`)
- bench/report.py models a whole dump's decode (103,488,697,311 B of bz2), labelled as modelled: the scanner's time per byte of bz2, the workers' per byte of output (the dump's output taken as its bz2 bytes times the sample's 17.69×), the wall at N threads as the larger of the two (the workers' over N) times the sample's wall over that bound, and the same gap as a fixed cost beside it; its interval adds the windows' ratio spread (±14.0%) in quadrature to the part scaled by output
- The bench version is semver, `BENCH = "1.0.0"` in bench/report.py, stamped on every result as `bench`
- The first pull request run (lmmx/mibz2#11, head c6ef0b4, an AMD EPYC 9V45 runner with 4 cores) took 3m05s in all; mibz2 decoded the Wikidata blocks at 252 MB/s on 1 thread and 478 MB/s on 4, and every arm gave the same bytes on every corpus
- examples/ab.rs `run` takes `--settle S`, the length of its load sample before each run (default 2 s, as before); the workflow passes 0.25
- The documentation site is Zensical (`zensical.toml`, `docs/site/`), adapted from lmmx/pq-glue's: home, installation, quickstart, concepts (checks, how it works, prior work), benchmarks (methodology, history, competitors), the Python API by mkdocstrings (`::: mibz2`), the Rust API as a summary and a link to docs.rs, background (history, journals) and maintainers (development, documentation, releasing)
- `.github/workflows/site.yml` builds the site on master pushes, pull requests and each completed Benchmark run on master (`workflow_run`), deploys master to the `gh-pages` branch and a pull request to `gh-pages/pr-preview/pr-<number>/` with rossjrw/pr-preview-action's comment, once Pages serves `gh-pages`
- docs/scripts/bench_history.py reads `bench-results` (`--store`, or `origin/bench-results`) and writes `docs/site/assets/bench-history.json` and `docs/generated/competitors.md` before each build; a series holds only commits of one bench major version, runner CPU model, core count and corpus set (bench_history.py `build`)
- The History page draws one chart per corpus and thread count in plain SVG (docs/site/assets/javascripts/bench.js): throughput, CPU, wall or peak RSS, each point a commit with its 95% interval and a link to the commit, one line per class, and a table of every point
- The Against other decoders page includes `docs/generated/competitors.md`, bench/report.py's master tables for the latest record, with the runner, rustc, pbz2 and lbzip2 versions
- docs/scripts/check_site.py (pq-glue's, for `mibz2`) checks the nav, internal links and anchors, llms.txt descriptions and API coverage; docs/conftest.py runs the pages' Python examples (`pytest docs/site`)

## Missing

- No record on `bench-results` until the Benchmark workflow runs on a push to master; the History and Against other decoders pages say so until then
- No end-to-end figures: the benchmark times the decoder alone (wikidata-pq's tracker holds the pipeline's)
- No aarch64 runner in the benchmark: every figure is an x86-64 runner's

## Divergence

- README.md's benchmark disclosure (`cargo bench`, a shared 4-vCPU VM) predates the CI benchmark and does not mention it
