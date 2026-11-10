"""bench/par_scan_finish.sh: a failed or late bundle leaves no VALID or DRY outcome and nothing
`just submit` would take; a successful one leaves the outcome as it was."""

import subprocess
import tarfile
from pathlib import Path

HERE = Path(__file__).parent


def attempt(tmp_path):
    d = tmp_path / "par-scan" / "20261110T000000Z"
    d.mkdir(parents=True)
    (d / "console.log").write_text("a run\n")
    return d


def finish(d, outcome, secs, *bundler):
    return subprocess.run(
        ["bash", str(HERE / "par_scan_finish.sh"), str(d), outcome, str(secs), *bundler],
        capture_output=True,
        text=True,
        timeout=120,
    )


def bundles(d):
    """What `just submit` would see: the recipe directory's *.tar.gz."""
    return sorted(p.name for p in d.parent.glob("*.tar.gz"))


def test_success(tmp_path):
    d = attempt(tmp_path)
    r = finish(d, "VALID: wikidata: MIXED;", 60, "bash", str(HERE / "bundle.sh"), str(d))
    assert r.returncode == 0, r.stdout + r.stderr
    assert (d / "outcome.txt").read_text().strip() == "VALID: wikidata: MIXED;"
    assert bundles(d) == [d.name + ".tar.gz"]
    with tarfile.open(d.parent / (d.name + ".tar.gz")) as t:
        member = t.extractfile(f"{d.name}/outcome.txt")
        assert member is not None and member.read().decode().startswith("VALID")
    assert not (d / "FINALISATION-FAILED.txt").exists()


def test_bundling_timeout(tmp_path):
    # the bundle step passes its deadline: no VALID outcome survives, nothing to submit
    for outcome in ("VALID: wikidata: FASTER;", "DRY: the recipe ran end to end"):
        d = attempt(tmp_path / outcome[:3])
        r = finish(d, outcome, 1, "sleep", "600")
        assert r.returncode == 2, r.stdout + r.stderr
        got = (d / "outcome.txt").read_text()
        assert got.startswith("INVALID: finalisation failed: the bundle passed its deadline")
        assert outcome in got and (d / "FINALISATION-FAILED.txt").exists()
        assert bundles(d) == []


def test_bundle_made_then_late(tmp_path):
    # a tarball written before the step failed is set aside, never left to be submitted
    d = attempt(tmp_path)
    script = f'tar -czf "{d}.tar.gz" -C "{d.parent}" "{d.name}"; sleep 600'
    r = finish(d, "VALID: x;", 1, "bash", "-c", script)
    assert r.returncode == 2
    assert bundles(d) == [] and (d.parent / (d.name + ".tar.gz.failed")).exists()
    assert (d / "outcome.txt").read_text().startswith("INVALID: finalisation failed")
    # and the submission refuses the set-aside file before reading anything else
    s = subprocess.run(
        ["bash", str(HERE / "submit.sh"), str(d.parent / (d.name + ".tar.gz.failed"))],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert s.returncode == 1 and "is not a .tar.gz bundle" in s.stderr


def test_bundling_failure(tmp_path):
    d = attempt(tmp_path)
    r = finish(d, "VALID: x;", 60, "sh", "-c", "exit 7")
    assert r.returncode == 2
    assert "the bundle failed (exit 7)" in (d / "outcome.txt").read_text()
    assert bundles(d) == []
