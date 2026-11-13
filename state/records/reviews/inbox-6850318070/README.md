# Replaying this review

Data-only replay: no executable from the bundle is run, no input is decompressed.

Place the verified bundle as live.tar.gz and safely extract its 20261113T101816Z directory under live-raw/. Fetch the following exact UTF-8 files without adding a trailing newline:

- cb73bf909397e2fff8c90f0a00437061ad06478e: bench/cells-gap/live/driver.py as live-driver.py; prepare.py as live-prepare.py; src/trace.rs as live-trace.rs.
- d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2: src/cells.rs as live-source.rs.

Run replay.py from that directory. It imports only pure schedule/reconcile/analyse functions; never invokes driver.main, run, child processes or decoder binaries. It writes live-replay.json. The recorded replay.json is the result checked for this report.

routing-audit.json comes from the pinned coord.py helper over connector snapshots of #145/#160/#161/#162/#222. Five findings are explicitly historical/resolved; the old researcher-inbox LOG remains reported without reopening unrelated work. REVIEW route is #145. Pinned verify hardcodes FROM coordinator; exact Optimiser body/author/destination readback is recorded separately, with no claim that this helper passes.
