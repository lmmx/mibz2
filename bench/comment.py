"""The benchmark's comment on a pull request: one comment, found by its marker and edited
in place on every run.

    comment.py pending PR HEAD   # the run has started: the last figures kept, marked stale
    comment.py final PR FILE     # the run's figures (report.py's Markdown)
    comment.py failed PR HEAD    # the run ended without figures

Needs GH_TOKEN, GITHUB_REPOSITORY and RUN_URL.
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

MARK = "<!-- mibz2-bench -->"
STATUS = re.compile(r"^<!-- status -->.*$\n?", re.M)


def gh(*args: str, body: str | None = None) -> str:
    if body is not None:
        args = (*args, "--input", "-")
    p = subprocess.run(
        ["gh", "api", *args],
        input=json.dumps({"body": body}) if body is not None else None,
        capture_output=True,
        text=True,
    )
    if p.returncode:
        sys.exit(f"gh api {' '.join(args)}: {p.stderr.strip()}")
    return p.stdout


def find(repo: str, pr: str) -> dict | None:
    pages = json.loads(gh(f"repos/{repo}/issues/{pr}/comments", "--paginate", "--slurp"))
    return next((c for page in pages for c in page if c["body"].startswith(MARK)), None)


def write(repo: str, pr: str, comment: dict | None, text: str) -> None:
    if comment:
        gh("-X", "PATCH", f"repos/{repo}/issues/comments/{comment['id']}", body=text)
    else:
        gh("-X", "POST", f"repos/{repo}/issues/{pr}/comments", body=text)


def with_status(body: str, line: str) -> str:
    """The comment with its status line (after the marker) replaced."""
    rest = STATUS.sub("", body.removeprefix(MARK).lstrip("\n"))
    return f"{MARK}\n<!-- status --> {line}\n\n{rest.lstrip()}"


def main() -> None:
    mode, pr = sys.argv[1], sys.argv[2]
    repo, run = os.environ["GITHUB_REPOSITORY"], os.environ["RUN_URL"]
    comment = find(repo, pr)
    if mode == "final":
        write(repo, pr, comment, Path(sys.argv[3]).read_text())
        return
    head = sys.argv[3][:7]
    if mode == "pending":
        line = f"⏳ **Running** for `{head}` ([run]({run})); the figures below are the last run's, if any."
        body = comment["body"] if comment else f"{MARK}\n## Benchmark: mibz2 decoder\n"
    else:
        line = f"❌ **The run for `{head}` ended without figures** ([run]({run})); the figures below are an earlier run's, if any."
        body = comment["body"] if comment else f"{MARK}\n## Benchmark: mibz2 decoder\n"
    write(repo, pr, comment, with_status(body, line))


if __name__ == "__main__":
    main()
