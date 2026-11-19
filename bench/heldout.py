"""The held-out corpora of the open port's C6 registration
(docs/journal/2026-11-19-open-port-registration.md; #145 CHECKPOINT 7004759994, the
Coordinator's correction of 19 November): block distributions the admission was never fitted
on and nothing was timed on before the registration was frozen.

    heldout.py build OUTDIR WIKIDATA_HELDOUT_BZ2

From the frozen commit's own tree (`git show HEAD:PATH`, sorted paths, so the bytes are fixed by
the commit), each written at bzip2 levels 1 and 9 (100k and 900k blocks) with Python's libbzip2:
- `ho-prose`: the journal's prose, every `docs/**/*.md` and `docs/**/*.txt`;
- `ho-code`: other code than the `text` guard's Rust: every `*.py`, `*.just`, `*.yml` and
  `docs/**/*.asm`;
- `ho-structured`: structured data that is not Wikidata: every `docs/**/*.json`, `*.jsonl` and
  `*.csv`, and `bench/**/*.json`;
- `ho-wikidata`: the held-out Wikidata windows (bench/wikidata-heldout.json, Wikimedia's own
  level-9 blocks, copied unchanged as `-l9`), and the same bytes re-encoded at level 1.

Writes OUTDIR/heldout.json: each corpus's sources' count, its decoded and compressed bytes and
its SHA-256. Standard library only.
"""
import bz2
import fnmatch
import hashlib
import json
import subprocess
import sys
from pathlib import Path

SETS = {
    "ho-prose": ["docs/*.md", "docs/*.txt"],
    "ho-code": ["*.py", "*.just", "*.yml", "docs/*.asm"],
    "ho-structured": ["docs/*.json", "docs/*.jsonl", "docs/*.csv", "bench/*.json"],
}


def tracked():
    out = subprocess.run(["git", "ls-tree", "-r", "--name-only", "HEAD"], check=True, capture_output=True, text=True)
    return sorted(out.stdout.split("\n")[:-1])


def matches(path, patterns):
    # fnmatch's * crosses '/', so "docs/*.md" is every Markdown file under docs/
    return any(fnmatch.fnmatch(path, p) for p in patterns)


def show(path):
    return subprocess.run(["git", "show", f"HEAD:{path}"], check=True, capture_output=True).stdout


def main(argv):
    if len(argv) != 4 or argv[1] != "build":
        sys.exit("usage: heldout.py build OUTDIR WIKIDATA_HELDOUT_BZ2")
    out, wiki = Path(argv[2]), Path(argv[3])
    out.mkdir(parents=True, exist_ok=True)
    paths = tracked()
    manifest = {}

    def write(name, data, level, sources, raw=None):
        z = raw if raw is not None else bz2.compress(data, compresslevel=level)
        f = out / f"{name}-l{level}.bz2"
        f.write_bytes(z)
        manifest[f.name] = {"sources": sources, "decoded_bytes": len(data), "bytes": len(z),
                            "sha256": hashlib.sha256(z).hexdigest()}
        print(f"{f.name}: {sources} sources, {len(data)} bytes -> {len(z)}", file=sys.stderr)

    for name, patterns in SETS.items():
        chosen = [p for p in paths if matches(p, patterns)]
        data = b"".join(show(p) for p in chosen)
        if not data:
            sys.exit(f"{name}: no sources")
        for level in (1, 9):
            write(name, data, level, len(chosen))
    z9 = wiki.read_bytes()
    data = bz2.decompress(z9)
    write("ho-wikidata", data, 9, 12, raw=z9)
    write("ho-wikidata", data, 1, 12)
    (out / "heldout.json").write_text(json.dumps(manifest, indent=1) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
