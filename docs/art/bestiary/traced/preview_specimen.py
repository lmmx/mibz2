#!/usr/bin/env python3
"""Animate a batch-traced master from its spec's assembly_tree and write a preview page.

    preview_specimen.py specs/NAME.json OUT.html

The master already wraps each articulated part in translate(pivot) ... translate(-pivot), so each
animation is a plain rotation (or scale) in degrees about the part's own origin.
"""
import json, re, sys
from pathlib import Path

T = Path(__file__).resolve().parent
spec = json.load(open(sys.argv[1])); slug = spec["specimen"]
svg = (T / f"{slug}-master.svg").read_text()

def walk(ns):
    for n in ns:
        yield n; yield from walk(n.get("children", []))

for n in walk(spec.get("assembly_tree", [])):
    a = n.get("anim")
    if not a: continue
    vals = ";".join(v.split()[0] if a["type"] == "rotate" else v for v in a["values"].split(";"))
    tag = (f'<animateTransform attributeName="transform" type="{a["type"]}" values="{vals}" keyTimes="{a["keyTimes"]}" '
           f'calcMode="spline" keySplines="{a["keySplines"]}" dur="{a["dur"]}" repeatCount="indefinite"/>')
    svg, c = re.subn(rf'(<g id="{re.escape(n["gid"])}"[^>]*>)', lambda m: m.group(1) + tag, svg, count=1)
    if not c: print("no group", n["gid"], file=sys.stderr)
svg = re.sub(r'\s(width|height)="[\d.]+"', "", svg, count=2)
fig = lambda cls, cap: f'<figure class="{cls}">{svg}<figcaption>{cap}</figcaption></figure>'
page = f"""<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{spec['title'][:40]}</title>
<style>:root{{--bg:#faf6ee;--fg:#2a1f16;--mut:#7a6a58;--card:#fffdf8;--line:#e4d9c6}}
@media (prefers-color-scheme:dark){{:root:not([data-theme="light"]){{--bg:#17130f;--fg:#efe6d8;--mut:#a89884;--card:#211b15;--line:#3a3027}}}}
:root[data-theme="dark"]{{--bg:#17130f;--fg:#efe6d8;--mut:#a89884;--card:#211b15;--line:#3a3027}}
body{{margin:0;background:var(--bg);color:var(--fg);font:16px/1.5 Georgia,serif;padding:24px 16px}} main{{max-width:980px;margin:0 auto}}
h1{{font-size:1.4rem;margin:0 0 4px}} p{{color:var(--mut);margin:0 0 18px}}
.hero{{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:16px;margin:0 0 16px}} .hero svg{{width:100%;max-width:620px;display:block;margin:0 auto}}
.row{{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:14px}} figure{{margin:0;background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px}}
figure svg{{width:100%;display:block}} figcaption{{color:var(--mut);font-size:.8rem;text-align:center}}
.claude svg{{--wash:#e0703f;--wash2:#a8431c}} .lapis svg{{--wash:#1f5fff;--wash2:#0b2fa8}} .moon svg{{--wash:#00c8d7;--wash2:#ff4518}}</style></head>
<body><main><h1>{spec['title']}</h1><p>{spec.get('source_style','')}, {spec.get('date_range','')}.</p><div class="hero">{svg}</div>
<div class="row">{fig('claude','Claude orange')}{fig('lapis','roundel lapis')}{fig('moon','cyan and orange')}</div></main></body></html>"""
Path(sys.argv[2]).write_text(page); print(sys.argv[2], len(page))
