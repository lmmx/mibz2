# Staged cooperative walk: source checkpoint

Inbox 6758227354. Source snapshot 2c70fd13ebc8565cbf1eab82536d72cfb2c84cfe; coop.rs and bwt.rs contents also match master ae44497e99765f3f1570c749ef5b381d7d8799a6.

Confirmed: coop::traverse is unstaged. Current room_segments already provides a per-thread tile/rows/marked-lane construction; the shared counter does not require a global chunk barrier. Finish/flush precede each ownership retirement and Relay's walkers decrement.

Corrections affecting price:
- An 8 KiB tile is per 32-lane walker, not per team. Three active walkers need 24 KiB; three such teams need 72 KiB, plus lane metadata.
- Plain K=3 uses up to 1536 starts, but the pipelined K=3 path uses 512*(K-1)=1024. Its leader joins late; distinct start density and tail fraction must be counted.
- Existing successful segment takes plus one exhausted claim per share invocation remain; no new per-byte atomic.
- #171's 4-thread runs used independent workers (old plan threshold >=6 cores); the archival result supports a transferable hypothesis, not a team result.
- The published bandwidth snippets expose DLmr/DLmw/Dw, not Ir. They do not independently reproduce 17.6/10.1. Share-nothing's older summary also says 17.8. Raw function/caller counts and matching row/output denominators are required before subtracting 7.5 from 38.0.
- Staging retains final segment-buffer writes and adds tile write/read traffic. It changes where latency is paid; it does not remove all store traffic.
- Fully costed proposal and smallest safe count experiment in progress. No build, compilation or new timing.
