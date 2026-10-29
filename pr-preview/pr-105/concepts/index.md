# Concepts

mibz2 decodes bzip2 as libbzip2 1.0.8 defines it, byte for byte, with every check libbzip2 makes, and decodes several blocks at once.

- [Why 32 couriers beat one](<https://lmmx.github.io/mibz2/pr-preview/pr-105/concepts/couriers/index.md>): memory-level parallelism, the decoder's premise. One courier waits at every stop to learn the next address; 32 couriers' waits overlap. The lanes, the prefetch, the per-step cost that binds next, and the measurements.
- [What is checked, and when](<https://lmmx.github.io/mibz2/pr-preview/pr-105/concepts/checks/index.md>): block CRCs, stream CRCs, what an error reports, and the four inputs mibz2 refuses where libbzip2 does not.
- [How it works](<https://lmmx.github.io/mibz2/pr-preview/pr-105/concepts/how-it-works/index.md>): the one sequential phase (the scanner), the workers, and the inverse BWT walked as 32 interleaved chains.
- [Prior work](<https://lmmx.github.io/mibz2/pr-preview/pr-105/concepts/prior-work/index.md>): libbzip2; Kärkkäinen, Kempa and Puglisi (DCC 2012); lbzip2 and pbzip2.
