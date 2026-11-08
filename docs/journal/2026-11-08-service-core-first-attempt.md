# The service core: the first attempt was INVALID, and the rerun

8 November 2026. This concerns the run registered in `2026-11-07-service-core-registration.md`,
frozen at 9ef3c17d (#251).

## The first attempt

- **Bundle:** `service-core/20261108T091321Z.tar.gz` on `evidence`, at 70d0af48.
  - Its SHA-256 is 718e552c71d909d8eb798550b8da20874d90e86dfbe6646a8a4c142321b0213c, which
    matches the notice on #145.
  - It was run at 223d93bb with `just submit`, from the workstation queue.
- **INVALID on one check alone: the noise control N1.**
  - N1 compares team3-twin@10 against team3@10. It came out at 1.018 [1.000, 1.035], and its
    upper bound is above the registered 1 ± 0.03.
  - Every other validity check passed: 20 rounds in every cell, the clean tree, the frozen
    inputs, the commit, the ten-core topology, the same bytes from every run, every cell's
    plan, and all six traces complete.
- **The cause is not a recipe defect.** The twin, the same decoder under another name, ran
  1.8% faster than team3 in this run. A recipe fix can't address that, and loosening N1 would
  change the A/A method, which is reserved to Louis (RULES C6). So nothing is changed.

**An INVALID run judges nothing.** Its comparisons are left unread here, as the registration
requires. They are in the bundle's `service-core.md`.

## The rerun, registered before it runs

- **The same recipe, unchanged.** The measured inputs are still exactly those at 9ef3c17d.
  This note is outside them, so the rerun's commit differs from 223d93bb in this file alone.
- **The rerun is the decision run.** Its outcome stands whatever it is, VALID or INVALID.
  There will be no third attempt to reach a valid one.
- **If the rerun is also INVALID on N1,** N1's method goes to Louis (C6), and the architecture
  question waits for his call.
