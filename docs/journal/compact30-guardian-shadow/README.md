# Shadow pilot replay

Read [decisions.md](decisions.md) for scope, outcomes and limitations. The scripts only parse archived evidence and static source/listings. They do not compile or execute constructors, run Miri, measure performance, or modify inputs.

With the pinned commits available in a repository:

```sh
python docs/journal/compact30-guardian-shadow/replay_p1.py --repo /path/to/mibz2 --out /tmp/guardian-p1
```

Compare the generated results and manifest with the committed JSON files. Python assertions must remain enabled. The manifest SHA-256 is the evidence digest in the cursor; it covers the exact pinned files, including whole compressed capture archives.

The supplemental Astra v3 replay was inspected and run from `0ac2e67e6826b9f2b1940cb876c3003e7a022144`, path `docs/journal/compact30-research/reviews/v3/replay.py`, using `--repo /path/to/mibz2`. Its result is saved as `astra-v3-replayed.json`; its manifest digest is embedded there. These are archive checks, not a fresh §6.2 review.
