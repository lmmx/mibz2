---
description: "What mibz2 decodes, what it checks, and how it decodes in parallel."
---

# Concepts

mibz2 decodes bzip2 as libbzip2 1.0.8 defines it, byte for byte, with every check
libbzip2 makes, and decodes several blocks at once.

- [What is checked, and when](checks.md): block CRCs, stream CRCs, what an error reports,
  and the four inputs mibz2 refuses where libbzip2 does not.
- [How it works](how-it-works.md): the one sequential phase (the scanner), the workers,
  and the inverse BWT walked as 12 interleaved chains.
- [Prior work](prior-work.md): libbzip2; Kärkkäinen, Kempa and Puglisi (DCC 2012);
  lbzip2 and pbzip2.
