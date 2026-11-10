# Three-table architecture: pricing checkpoint

Inbox6759345168; specs a2da49638ac54d7763de1c167bd054f67ffdc763. Requested source7bf5769be304d8908910de76a4fe5050ac8cf8c4; later correction96b7937be27ede9822dbcbd15e0ac712e5ddd41a inspected separately.

- Complete chunked fill is unbuilt. Existing from_t constructs T first and converts it; equal-slice starts are obtained by a full oracle traversal outside timing. This is not a complete segmented inverse benchmark.
- Correction uses Vec<[u64;2]> directory but bytes() charges12 bytes/word. At900k rows this omits56252 bytes for the main rounded directory (plus padding). A proposed start bitmap adds112504 bytes. Text exceeds1.3MB before allocation slack and scratch; corrected speed and original footprint cannot be combined.
- The12 proposed cells would be252 slots at21 rounds versus105 for the prior five-cell proposal, before sentinels. Neither D4's failed registration nor chunked's incomplete kernel is made eligible by adding a cell.
- Tunnel calculation independently gives maxima2189MB/s at4 for g=.7 and1471 at5 for g=1.25 with s4.2,a.33. The quoted range is a4–5-worker optimum, not ten; ten requires X(42MB), beyond the fitted36MB domain. Reducing accesses changes penalty amplitude, not automatically its footprint breakpoint.

Full price follows. No implementation or timing.
