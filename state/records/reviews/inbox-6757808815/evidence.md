# Design E evidence checkpoint
Inbox6757808815; source84bf177bd0af9fa416964eca2c0f56ad278b119e.
Run42745744064 artifact13731585881 ZIP SHA2563734059c48e66288bb0498a8f55cacbe27cd68c075691e6968358011cafda3ba verified. Five head stage records replayed using paired log ratios and t4. Registered FALSIFIED preserved. Result document is at later b759304ee171ee316675f1a8cf8f3b455a977fb5, not the measured pin.
Counts are the registered three-block inclusive callgrind summaries, not raw listing replay.
E has two failures: per-row decoding and insufficient footprint at full block. At900k rows2.039B/row is1.8351MB before bitmap/scratch, not1.2MB. Removing all approximately22 bookkeeping Ir still leaves26.2Ir/row, above the15 target.
Ranked new construction under analysis: D at k2 with8-byte piece records, not k4 padded12-byte records. At r2/n=.088,8r2/n+.3125=.1.0165B/row including an explicit64-bit segment bitmap and32-bit rank beside64-bit boundary word. About.915MB before exceptions/scratch. Full per-piece loop can spend up to30Ir for two emitted bytes; stop/tail losses must be charged.
The1.44 CPU growth ratio across worker counts is not by itself a permissible kernel slowdown; retain it as descriptive, not a replacement for whole-decoder comparison.
