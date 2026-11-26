# The implicit serial writer fails even with free index construction

With an immediate same-arm, same-block warm run before every sample, the carried-owner writer takes **9.149 [8.764, 9.551] times the complete frozen scalar-cap inverse** over the ten systematic original blocks. The individual ratios range from 6.76 to 11.12. In a separate rotating-block panel with one preliminary pass, the ratio is 10.194 [9.235, 11.253]. Every writer and scalar invocation returned the exact marked bytes. Both series have 24 paired rounds, each covering all ten blocks. CPU 1 and thread CPU clocks were used for this local stage. [Preregistered gate](PLAN.md) · [Immediately warmed pairs](warm.csv) · [Rotating-block pairs](times.csv) · [Intervals and index identities](summary.json)

The immediate-warm median times are 227.72 ms for the writer and 26.85 ms for scalar across ten blocks. The writer receives the complete immutable crossing index, compact bases, hit-slice ranges, carried mark and preallocated output, with export and index construction outside the clock. The serialized index occupies 2,380,740–6,386,392 bytes per block before output and C++ object overhead. The control receives the actual original capped Rust runs and mark, with its complete run entry and literal output timed. Every output is compared outside the clock. Both arms use the same code and block protocol in each series. [Emitter](writer.cpp) · [Exact packed exporter](export.py) · [Frozen Rust control and corpus protocol](../../../capped/PLAN.md)

The structural bound of 2.503 binary comparisons per visited row counts only the two searches. Every next owner, data-dependent offset and destination still depends on the preceding marked arc. The paid observation closes this particular serial writer at a favorable boundary. The order-free contraction, carried mark and every-row/word routing proofs remain independently checkable. Other residual schedules would require their own bound and paid evidence. [Geometry and owner census](../HINTS.md)

To reproduce, build the earlier actual-cap scalar calibration and create the ten original checked inputs according to the powers README, then set `p` to this directory. The fixture exporter regenerates every byte of all ten indexes; the manifest stores their SHA-256 checksums.

```bash
python3 -B "$p/export.py" "$s/checked" "$s/probe"
python3 "$p/build.py" "$s/capped" "$s/refs/scalar-build/release" "$s/probe"
taskset -c 1 nice -n 10 "$s/probe/probe" corpora/wikidata.bz2 "$s/probe"
taskset -c 1 nice -n 10 "$s/probe/probe" corpora/wikidata.bz2 "$s/probe" warm
python3 "$p/receipt.py" "$s/probe"
```
