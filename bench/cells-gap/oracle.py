"""Independently check every census block's bytes through Python/libbzip2."""

import bz2
import hashlib
import json
import sys
from pathlib import Path


def verify(census, corpus):
    records = [json.loads(line) for line in Path(census).read_text().splitlines()]
    assert records[0]["input_sha256"] == hashlib.sha256(Path(corpus).read_bytes()).hexdigest()
    digest = hashlib.sha256()
    count = total = 0
    with bz2.open(corpus, "rb") as reader:
        for r in records:
            if r["kind"] != "block":
                continue
            b = r["data"]
            data = reader.read(b["bytes"])
            assert len(data) == b["bytes"]
            assert hashlib.sha256(data).hexdigest() == b["sha256"], b["block"]
            digest.update(data)
            count += 1
            total += len(data)
        assert not reader.read(1), "unconsumed oracle bytes"
    return {"oracle": "Python/libbzip2", "blocks": count, "bytes": total,
            "decoded_sha256": digest.hexdigest(), "parity": True}


if __name__ == "__main__":
    print(json.dumps(verify(*sys.argv[1:])))
