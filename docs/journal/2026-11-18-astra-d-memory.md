# Astra D · Memory traffic per output byte

## Registered comparison

The unit is a byte after RLE1. Replay the 0.5.6 lazy worker at `1edb29fdc3d42dfddfc3d60e46456b611323760c`. Preserve buffer aliases across induction, writing and finishing. The frozen input is whole-stream ordinal 1 (large-block ordinal 0). The systematic sample is every 24th large Wikidata block in the pinned induction record set `2e4b34e086ac95e8218f1ee822a4652c8db5bb19`. Use the independently rewrapped compressed blocks and libbz2 as the byte oracle. Check each block's actual route.

Start with the compulsory payload bound, one compressed input read and one final output write. Charge ordinary cached stores for acquisition and eventual dirty eviction. Count line requests, modelled memory-controller transfers and address dependencies separately. Replay 64-byte lines through a 32 KiB L1D and a 256 KiB L2, then vary effective shared-cache capacity around 2 MiB. Test any substantial change against exact access counts first. Absolute optimality would require a stronger lower bound than compulsory I/O alone.

The i9 knee diagnostic, the ten-versus-seven comparison and the 0.5.6 release comparison have different versions and scopes. They provide the hardware evidence. The executable checks here use no elapsed times.
