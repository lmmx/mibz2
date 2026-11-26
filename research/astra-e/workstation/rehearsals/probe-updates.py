"""Exercise fresh and fallback selection on real Cargo builds in disposable Git repositories."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[4]
WORKSTATION = ROOT / "research/astra-e/workstation"
RUST = r"""fn main() {
    assert_eq!(std::env::args().nth(1).as_deref(), Some("info"));
    println!(r#"{{"candidate":"{}","candidate_type":"fixture","arms":["lazy","opend","open","candidate"],"cpus":1}}"#, env!("MOONSHOT_CANDIDATE"));
}
"""


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()


def commit(repo, message):
    git(repo, "add", ".")
    git(
        repo,
        "-c",
        "user.name=Astra E fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "-qm",
        message,
    )
    return git(repo, "rev-parse", "HEAD")


def main(out):
    out.mkdir(parents=True)
    with TemporaryDirectory() as tmp:
        upstream, tree = (Path(tmp) / name for name in ("upstream", "measured"))
        harness = upstream / "bench/moonshot"
        (harness / "src").mkdir(parents=True)
        reporter = harness / "moonshot.py"
        for name in ("moonshot.py", "calibration.json"):
            shutil.copy(ROOT / "bench/moonshot" / name, harness / name)
        (harness / "src/main.rs").write_text(RUST)
        (harness / "Cargo.toml").write_text('[package]\nname="moonshot"\nversion="0.1.0"\nedition="2021"\n')
        subprocess.run(
            ["cargo", "generate-lockfile", "--manifest-path", str(harness / "Cargo.toml")], check=True
        )
        (upstream / ".just").mkdir()
        (upstream / ".just/fixture.just").write_text("")
        (upstream / "Justfile").write_text("")
        git(upstream, "init", "-qb", "master")
        old = commit(upstream, "working scorer fixture")
        git(upstream, "clone", "-q", str(upstream), str(tree))
        git(tree, "switch", "-q", "--detach", old)
        recipe = tree / "research/astra-e/workstation"
        recipe.mkdir(parents=True)
        for name in ("refresh.py", "test_protocol.py", "assess.py"):
            shutil.copy(WORKSTATION / name, recipe / name)
        (recipe / "scorer-pin.json").write_text(json.dumps({"last_working_path_revision": old}))
        commit(tree, "freeze fixture recipe")

        def trial(name, selected, failure=None):
            proof = out / name
            proof.mkdir()
            env = dict(
                os.environ,
                CARGO_BUILD_JOBS="1",
                CARGO_TARGET_DIR=str(Path(tmp) / name),
                PYTHONDONTWRITEBYTECODE="1",
                MOONSHOT_CANDIDATE=str(tree / "fixture.rs"),
            )
            with (proof / "console.log").open("w") as log:
                subprocess.run(
                    [sys.executable, str(recipe / "refresh.py"), str(proof)],
                    cwd=tree,
                    env=env,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    check=True,
                )
            result = json.loads((proof / "scorer-refresh.json").read_text())
            assert result["selected_build"] == selected
            assert len(result["attempts"]) == (2 if failure else 1)
            if failure:
                failed = result["attempts"][0]["failure"]
                assert failed["stage"] == failure and failed["exit_code"] > 0
                diagnostic = (proof / failed["log"]).read_text()
                assert (
                    "unrecognized arguments: --no-proxy" if failure == "interface" else "unknown_fixture_type"
                ) in diagnostic
            assert json.loads((proof / (selected + "-interface.json")).read_text())["rows"] == 256
            print(name, selected, failure or "compatible")

        reporter.write_text(reporter.read_text() + "\n# Compatible upstream revision.\n")
        commit(upstream, "compatible update")
        trial("compatible", "fresh")
        src = reporter.read_text()
        assert '"--no-proxy"' in src
        reporter.write_text(src.replace('"--no-proxy"', '"--retired-fixture-option"'))
        commit(upstream, "retired CLI flag fixture")
        trial("cli-break", "last-working", "interface")
        (harness / "src/main.rs").write_text(RUST + "\nconst INVALID: unknown_fixture_type = 1;\n")
        commit(upstream, "compiler diagnostic fixture")
        trial("build-break", "last-working", "build")


if __name__ == "__main__":
    main(Path(sys.argv[1]).resolve())
