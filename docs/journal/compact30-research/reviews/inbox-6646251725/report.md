# Post-P87 ranking: progress

Inbox 6646251725; baseline e26e3d9cbac6bcd2106b7070c869dd91fcd7217d.

Raw Callgrind caller/callee arcs checked in all 14 archived profiles. Each `measured` wrapper adds 16 Ir. Candidate block-1 decode_block = 20,184,340, build = 5,429,062, room_segments = 9,859,825, rest = 4,895,453. Build and room_segments match the paired pre-P87 arm on every full block. The 400/400/432/542-Ir full-block reductions are fresh-Scratch counts, not recurring steady-state savings.

Conditional compact-table parity becomes 10,861,581 walk Ir (12.0688122 per row), holding the old compact build/rest fixed. Remaining work: publish ranking, explicit B handover/replay obligations and falsifiers, update canonical ledger, post one REVIEW, complete cursor.
