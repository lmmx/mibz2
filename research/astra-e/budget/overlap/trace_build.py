"""Build the source-frozen edge worker with thread-local phase marks in a private tree."""
import argparse
import io
import subprocess
import sys
import tarfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SOURCE = "badb2ae"
parser = argparse.ArgumentParser()
parser.add_argument("build", type=Path)
args = parser.parse_args()
dest = args.build.resolve()
dest.mkdir(exist_ok=True, parents=True)
paths = ["src", "docs/journal/2026-11-18-open-return/open_return.rs",
         "docs/journal/2026-11-18-open-return/endpoint.rs", "docs/journal/2026-11-18-open-return/segments.rs"]
paths += [f"research/astra-e/{name}" for name in ["edge-budget", "narrow-short", "narrow-endpoint",
                                                   "short-words", "lookahead"]]
data = subprocess.check_output(["git", "-C", ROOT, "archive", SOURCE, *paths])
with tarfile.open(fileobj=io.BytesIO(data)) as tar:
    tar.extractall(dest, filter="data")
subprocess.run([sys.executable, dest / "research/astra-e/edge-budget/prepare.py"], check=True)
library = dest / "src/lib.rs"
source = library.read_text()
needle = "if b.orig_ptr as usize >= n {"
assert source.count(needle) == 2
source = source.replace(needle, 'trace::mark(trace::Kind::MtfEnd, trace::NONE);\n    ' + needle)
library.write_text(source)
parallel = dest / "src/parallel.rs"
source = parallel.read_text()
needle = "drop(st);\n        mark!(Ready, place);"
assert source.count(needle) == 1
parallel.write_text(source.replace(needle, "mark!(Ready, place);\n        drop(st);"))
# Same ready publication. Mark under the existing lock so Taken cannot precede its timestamp.
main = dest / "main.rs"
main.write_text('#[path="research/astra-e/edge-budget/candidate.rs"] mod edge;\n' +
                (HERE / "trace.rs").read_text())
flags = ["rustc", "--edition=2021", "-Copt-level=3", "-Ccodegen-units=1", "--cfg", 'feature="trace"', '--cfg', 'feature="moonshot"']
subprocess.run([*flags, "--crate-type=rlib", "--crate-name=mibz2", library,
                "-o", dest / "libmibz2.rlib"], check=True)
subprocess.run([*flags, main, "--extern", f"mibz2={dest}/libmibz2.rlib", "-o", dest / "trace"], check=True)
