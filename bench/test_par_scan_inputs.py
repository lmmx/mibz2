"""bench/par_scan_inputs.py: the expected sizes come from the manifest and the generator, and the
level-1 input must decode to the corpus's own bytes."""

import bz2
import hashlib
import importlib.util
import json
from pathlib import Path

import pytest

HERE = Path(__file__).parent
spec = importlib.util.spec_from_file_location("par_scan_inputs", HERE / "par_scan_inputs.py")
assert spec is not None and spec.loader is not None
inputs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inputs)

TEXT = b"".join(b"line %d of the corpus\n" % i for i in range(40_000))
RECORDS = bytes(range(256)) * 2_000


def setup(tmp_path, decoded_bytes=len(TEXT), gen_raw=len(RECORDS)):
    corpus = tmp_path / "wikidata.bz2"
    corpus.write_bytes(bz2.compress(TEXT, 9))
    manifest = tmp_path / "wikidata.json"
    manifest.write_text(
        json.dumps(
            {
                "corpus_sha256": hashlib.sha256(corpus.read_bytes()).hexdigest(),
                "decoded_bytes": decoded_bytes,
            }
        )
    )
    records = tmp_path / "gen" / "records.bz2"
    records.parent.mkdir()
    records.write_bytes(bz2.compress(RECORDS, 9))
    log = tmp_path / "gen.txt"
    log.write_text(
        f"{tmp_path}/gen/text.bz2: 5 -> 40 bytes\n"
        f"{records}: {gen_raw} -> {records.stat().st_size} bytes\n"
    )
    out = tmp_path / "inputs"
    out.mkdir()
    return [str(manifest), str(corpus), str(log), str(records), str(out)]


def test_anchored(tmp_path):
    args = setup(tmp_path)
    got = inputs.main(*args)["inputs"]
    digest = hashlib.sha256(TEXT).hexdigest()
    assert got["wikidata"]["expected_bytes"] == len(TEXT)
    assert got["wikidata-l1"]["expected_bytes"] == len(TEXT)
    assert got["wikidata"]["decoded_sha256"] == got["wikidata-l1"]["decoded_sha256"] == digest
    assert got["records"]["expected_bytes"] == len(RECORDS)
    out = Path(args[-1])
    assert bz2.decompress((out / "wikidata-l1.bz2").read_bytes()) == TEXT
    assert (out / "records.bz2").exists() and (out / "wikidata.bz2").exists()


def test_refused(tmp_path):
    # the manifest's size, not the run's: a corpus that decodes to another size is refused
    with pytest.raises(SystemExit, match="not the manifest's"):
        inputs.main(*setup(tmp_path / "a", decoded_bytes=len(TEXT) + 1))
    # the generator's stated size, likewise
    with pytest.raises(SystemExit, match="not the generator's"):
        inputs.main(*setup(tmp_path / "b", gen_raw=len(RECORDS) - 1))


def test_level1_must_match(tmp_path, monkeypatch):
    # a level-1 input that decodes to other bytes (one byte changed) is refused
    args = setup(tmp_path)
    real = bz2.compress
    monkeypatch.setattr(inputs.bz2, "compress", lambda d, lvl: real(d[:-1] + b"?", lvl))
    with pytest.raises(SystemExit, match="does not decode to the corpus's bytes"):
        inputs.main(*args)


@pytest.fixture(autouse=True)
def _mkdirs(tmp_path):
    for sub in ("a", "b"):
        (tmp_path / sub).mkdir()
