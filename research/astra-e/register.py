"""Regenerate REGISTER.md from completed scorer bundles. Run from any directory."""

from collections import defaultdict
import hashlib
import json
from pathlib import Path
import re
import shlex
import statistics
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
REPO = "https://github.com/lmmx/mibz2"


def number(x):
    return "—" if x is None else f"{x:.3f}"


def interval(metric):
    return f"{number(metric['geomean'])} [{number(metric['lo'])}, {number(metric['hi'])}]"


def load(path):
    score = json.loads(path.read_text())
    directory = path.parents[1]
    label = directory.name
    manifest = (directory / "manifest.txt").read_text().splitlines()
    head, master = manifest[:2]
    revision = next(s.split()[-1] for s in manifest if s.startswith("scorer path revision "))
    opts = score["options"]
    argv = shlex.split(next(s for s in manifest if s.startswith("invocation: ")).removeprefix("invocation: "))
    candidate = argv[3]
    checksums = dict((name, sha) for sha, name in (s.split(maxsplit=1) for s in manifest if re.match(r"^[a-f0-9]{64}  ", s)))
    frozen = subprocess.check_output(["git", "show", f"{head}:{candidate}"], cwd=ROOT)
    preparation = []
    # Earlier milestones predate the common dispatcher. Their constructors
    # are generated from byte-checked repository sources by a sibling script.
    if subprocess.run(["git", "cat-file", "-e", f"{head}:research/astra-e/prepare.sh"],
                      cwd=ROOT, stderr=subprocess.DEVNULL).returncode == 0:
        preparation = [shlex.join(["bash", "research/astra-e/prepare.sh", candidate])]
    elif b"constructor.rs" in frozen or b"mod constructor;" in frozen:
        script = (Path(candidate).parent / "prepare.py").as_posix()
        if subprocess.run(["git", "cat-file", "-e", f"{head}:{script}"], cwd=ROOT,
                          stderr=subprocess.DEVNULL).returncode == 0:
            preparation = [shlex.join(["python3", script])]
    if hashlib.sha256(frozen).hexdigest() != checksums[candidate]:
        raise ValueError(f"{label}: candidate entry digest differs from recorded Git tree")
    note_file = directory / "assessment.json"
    note = json.loads(note_file.read_text()) if note_file.exists() else {}
    if candidate.startswith("research/astra-e/controls/"):
        kind = "historical control"
    elif "confirm" in label or "quiet" in label:
        kind = "confirmation"
    elif "pilot" in label:
        kind = "pilot"
    else:
        kind = "local score"
    kind = note.get("kind", kind)
    runs_file = path.with_name("runs.jsonl")
    runs = [json.loads(s) for s in runs_file.read_text().splitlines()] if runs_file.exists() else []
    info = score.get("info", {})
    printed = next((source[k] for source in (info, score) for k in ("scorer_version", "version") if source.get(k)), None)
    if printed is None:
        report = path.with_name("summary.txt").read_text()
        match = re.search(r"^.*?scorer[ _-]version\s*[:= ]\s*(\S+)", report, re.I | re.M)
        printed = match.group(1) if match else "no version field printed"
    compiler = next(s for s in manifest if s.startswith("rustc "))
    host = manifest[manifest.index(next(s for s in manifest if s.startswith("invocation: "))) - 1]
    digest = checksums.get(opts.get("corpus") or "corpora/wikidata.bz2", "unrecorded")
    return dict(label=label, score=score, opts=opts, path=candidate, head=head, master=master,
                revision=revision, argv=argv, gate=score["gate"], kind=kind, note=note,
                runs=runs, version=printed, compiler=compiler, host=host, digest=digest, preparation=preparation)


def point_leaders(entries):
    groups = defaultdict(list)
    for e in entries:
        if e["kind"] == "historical control" or e["gate"]["verdict"] != "PASS":
            continue
        opts = e["opts"]
        for w, cell in e["score"].get("timing", {}).items():
            signature = (e["revision"], e["digest"], e["compiler"], e["host"], e["kind"],
                         tuple(opts["workers"] or ()), opts["rounds"], opts["settle"], opts["repeat"],
                         opts.get("with_opend"), opts.get("quick"), int(w))
            groups[signature].append((e["label"], cell["candidate"]))
    lines = ["| Scorer / design / W | Lowest wall estimate vs lazy [95% CI] | Lowest median RSS ratio vs lazy |",
             "|---|---|---|"]
    for signature, options in sorted(groups.items()):
        rev, _, _, _, kind, grid, rounds, settle, repeat, opend, quick, w = signature
        fastest, wall = min(options, key=lambda t: t[1]["geomean"])
        smallest, memory = min(options, key=lambda t: t[1]["peak_rss_ratio"])
        design = f"`{rev}` {kind}, grid {','.join(map(str,grid))}, {rounds}r ×{repeat}, settle {settle:g}s"
        design += f", {'quick' if quick else 'full'}, {'3' if opend else '2'} arms, **W{w}**"
        lines.append(f"| {design} | {fastest} · {interval(wall)} | {smallest} · {number(memory['peak_rss_ratio'])} |")
    return lines if groups else ["No completed timed experimental candidate."]


def describe(e):
    label, s, opts = e["label"], e["score"], e["opts"]
    prefix = "results/" + label
    gate = e["gate"]
    lines = [f"## {label}", "", f"{e['kind'].capitalize()}. **{gate['verdict']}** on {gate['cases']} scorer cases "
             f"({'quick' if opts.get('quick') else 'full'}). "
             f"[Per-case results]({prefix}/score/check.jsonl) · [manifest]({prefix}/manifest.txt) · [score]({prefix}/score/summary.txt)", "",
             f"The build tree, including sibling modules, is [`{e['head']}`]({REPO}/tree/{e['head']}). "
             f"[Candidate entry]({REPO}/blob/{e['head']}/{e['path']}). "
             f"Fetched master `{e['master']}`, scorer path `{e['revision']}` ({e['version']}). "
             f"Corpus SHA-256 `{e['digest']}`.", ""]
    if s.get("timing"):
        lines.extend(["| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |",
                      "|---:|---:|---|---|---:|---|---:|"])
        for w, cell in sorted(s["timing"].items(), key=lambda t:int(t[0])):
            a = cell["candidate"]
            other = cell.get("opend")
            peaks = {}
            for arm in ("candidate", "lazy", "opend"):
                vals = [r["peak_rss_kb"] for r in e["runs"] if r["arm"] == arm and r["workers"] == int(w)]
                peaks[arm] = f"{statistics.median(vals):,.0f}" if vals else "—"
            direct = a["geomean"] / other["geomean"] if other else None
            lines.append(f"| {w} | {cell['rounds']} | {interval(a)} | {interval(other) if other else '—'} | "
                         f"{number(direct)} | {' / '.join(peaks.values())} | {number(a['peak_rss_ratio'])} |")
        lines.extend(["", f"¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks]({prefix}/score/runs.jsonl)", ""])
    proxy = s.get("proxy", {})
    if proxy.get("metrics"):
        lines.extend(["| W model | Instructions/B | L2 / L3 / DRAM B/B | Dependent DRAM / past-L2 links/B | DRAM MLP | Calibration |", "|---:|---:|---|---|---:|---|"])
        for w, m in sorted(proxy["metrics"]["candidate"].items(), key=lambda t:int(t[0])):
            traffic = ' / '.join(number(m[k]) for k in ('l2_bytes_per_byte','l3_bytes_per_byte','dram_bytes_per_byte'))
            links = ' / '.join(f"{m[k]:.5f}" for k in ('dep_dram_links_per_byte','dep_past_l2_links_per_byte'))
            lines.append(f"| {w} | {number(m['instructions_per_byte'])} | {traffic} | {links} | {number(m['dram_mlp'])} | "
                         f"{'outside' if proxy['out_of_calibration'][w] else 'within envelope'} |")
        lines.append("")
    else:
        lines.extend(["This score has no completed instruction, cache-transfer, dependency or overlap model.", ""])
    if e["note"].get("assessment"):
        lines.extend([e["note"]["assessment"], ""])
    verification = e["note"].get("verification", "Independent candidate CI and target evidence have not been recorded for this run in this register.")
    if e["kind"] == "historical control":
        verification = "The i9 measured the historical Open arm separately. [Matched i9 results](prior.md)"
    lines.extend([verification, "", "Reproduce in a fresh checkout of the build tree above.", "", "```sh",
                  f"git checkout {e['master']} -- bench/moonshot .just Justfile", *e["preparation"], "bash bench/moonshot/corpus.sh",
                  shlex.join(["python3", "bench/moonshot/moonshot.py", "score", e["path"], *e["argv"][4:]]), "```", ""])
    return lines


def main():
    scores = [load(p) for p in sorted((HERE / "results").glob("*/score/summary.json"))]
    lines = ["# Astra E best-so-far register", "", "The reference is 0.5.6 LazyCells in the same shipped worker loop and 4W slots. "
             "The full Wikidata input decodes to 209,700,196 bytes per copy. Each gate result and timing scope below belongs to one immutable build. "
             "The local Xeon measurements do not establish the i9 target. [Frozen target rule](workstation/README.md) · [Historical i9 frontier](prior.md) · [Astra E draft PR and CI](https://github.com/lmmx/mibz2/pull/607/checks)", "",
             "## Local leaders by tested design", "", "These are the lowest recorded point estimates within each group of experiments with matching scorer revision, corpus hash, compiler, host, design and worker count. "
             "Each experiment pairs its candidate with its own lazy control. Between-experiment order remains uncertain when timing variation is large. A shared quiet comparison is needed to establish which candidate wins head to head. "
             "Pilot runs may overlap other work. Confirmation runs have their own rows. The historical control stays separate. RSS leaders remain visible when timing and memory favour different candidates.", "", *point_leaders(scores), ""]
    for manifest in sorted((HERE / "proxy").glob("*/manifest.json")):
        meta = json.loads(manifest.read_text())
        files = sorted(manifest.parent.glob("*.json"))
        samples = [(p, json.loads(p.read_text())) for p in files if p != manifest]
        samples = [(p, r) for p, r in samples if "metrics" in r and "ordinals" in r]
        if not samples:
            continue
        lines.extend([f"## Partial proxy · {manifest.parent.name}", "",
                      f"Build `{meta['head']}` used scorer `{meta['source_path_revision']}`. "
                      f"Rows identify the actual stream-block indices. This single-worker screen uses the listed LLC share and latency settings. "
                      f"It omits the scorer's full reference-arm calibration checks. [Manifest]({manifest.relative_to(HERE)})", "",
                      "| Stream blocks | W model | Arm | Instructions/B | L2 / L3 / DRAM B/B | Dependent DRAM / past-L2 links/B | DRAM MLP | Raw |",
                      "|---|---:|---|---:|---|---|---:|---|"])
        for path, row in samples:
            m = row["metrics"]
            traffic = " / ".join(number(m[k]) for k in ("l2_bytes_per_byte", "l3_bytes_per_byte", "dram_bytes_per_byte"))
            links = " / ".join(f"{m[k]:.5f}" for k in ("dep_dram_links_per_byte", "dep_past_l2_links_per_byte"))
            lines.append(f"| {','.join(map(str,row['ordinals']))} | {row['workers']} | {row['arm']} | "
                         f"{number(m['instructions_per_byte'])} | {traffic} | {links} | {number(m['dram_mlp'])} | [receipt]({path.relative_to(HERE)}) |")
        lines.append("")
    lines.extend(["Reproduction commands below replay the historical scorer on the recorded master commit. For a new scoring round, use `bash research/astra-e/score.sh <new-label> <candidate> ...` to refresh master first. "
                  "Per-run `assessment.json` may record `kind`, `assessment` and `verification` (including independent run links). Regenerate with `python3 research/astra-e/register.py`.", ""])
    for e in scores:
        lines.extend(describe(e))
    (HERE / "REGISTER.md").write_text("\n".join(lines))


if __name__ == "__main__":
    main()
