// The benchmark history (docs/site/benchmarks/history.md): a small chart per corpus and
// thread count of master's mibz2 figures over its commits, from
// assets/bench-history.json (docs/scripts/bench_history.py), in plain SVG. Each class of
// results (a major bench version, a runner CPU and core count, a set of corpora) is a
// line of its own, never joined to another. Runs on any page with a #bench-history
// element, on load and on each instant navigation.
(() => {
  "use strict";
  const SVG = "http://www.w3.org/2000/svg";
  const METRICS = [
    ["mb_s", "Throughput", "MB/s of output", (v) => v.toFixed(v < 100 ? 1 : 0)],
    ["cpu_s", "CPU time", "s, user + sys", (v) => v.toFixed(3)],
    ["wall_s", "Wall time", "s", (v) => v.toFixed(3)],
    ["rss_mb", "Peak RSS", "MB", (v) => v.toFixed(1)],
  ];
  const SLOTS = 4; // categorical slots (bench.css: --bh-s1..s4), then muted
  let state = { metric: "mb_s", threads: "all" };
  try {
    state = { ...state, ...JSON.parse(localStorage.getItem("mibz2-bench") || "{}") };
  } catch (e) { /* (no storage: the defaults) */ }

  const el = (tag, attrs = {}, parent = null, ns = null) => {
    const e = ns ? document.createElementNS(ns, tag) : document.createElement(tag);
    for (const [k, v] of Object.entries(attrs)) {
      if (v === null || v === undefined) continue;
      if (k === "text") e.textContent = v;
      else e.setAttribute(k, v);
    }
    if (parent) parent.appendChild(e);
    return e;
  };
  const svg = (tag, attrs, parent) => el(tag, attrs, parent, SVG);
  const slot = (cls) => (cls < SLOTS ? `var(--bh-s${cls + 1})` : "var(--bh-old)");

  // Nice ticks over [lo, hi]
  const ticks = (lo, hi, n = 4) => {
    if (hi <= lo) { hi = lo + (lo || 1) * 0.1; lo = lo - (lo || 1) * 0.1; }
    const step0 = (hi - lo) / n;
    const mag = 10 ** Math.floor(Math.log10(step0));
    const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => s >= step0);
    const out = [];
    for (let v = Math.floor(lo / step) * step; v <= hi + step * 1e-9; v += step) out.push(+v.toFixed(10));
    return out;
  };

  function chart(data, series, metric, root) {
    const [key, , unit, fmt] = metric;
    const fig = el("figure", { class: "bh-chart" }, root);
    el("figcaption", { text: `${series.corpus} · ${series.threads} thread${series.threads === 1 ? "" : "s"}` }, fig);
    const pts = series.lines.flatMap((l) => l.points.filter((p) => p[key]).map((p) => ({ ...p, cls: l.class })));
    if (!pts.length) { el("p", { text: "No points." }, fig); return; }
    const W = 640, H = 210, L = 56, R = 14, T = 12, B = 34;
    const cs = [...new Set(pts.map((p) => p.c))].sort((a, b) => a - b);
    const xi = new Map(cs.map((c, i) => [c, i]));
    const x = (c) => L + (cs.length === 1 ? (W - L - R) / 2 : (xi.get(c) * (W - L - R)) / (cs.length - 1));
    const lo = Math.min(...pts.map((p) => p[key][0] - p[key][1]));
    const hi = Math.max(...pts.map((p) => p[key][0] + p[key][1]));
    const ty = ticks(Math.max(0, lo - (hi - lo) * 0.1), hi + (hi - lo) * 0.1);
    const y0 = ty[0], y1 = ty[ty.length - 1];
    const y = (v) => T + (H - T - B) * (1 - (v - y0) / (y1 - y0 || 1));
    const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, role: "img", "aria-label": `${fig.firstChild.textContent}: ${metric[1]} (${unit}) by commit` }, fig);
    for (const v of ty) {
      svg("line", { x1: L, x2: W - R, y1: y(v), y2: y(v), class: "bh-grid" }, s);
      svg("text", { x: L - 6, y: y(v) + 4, class: "bh-tick", "text-anchor": "end" }, s).textContent = fmt(v);
    }
    svg("text", { x: 4, y: T + 2, class: "bh-unit" }, s).textContent = unit;
    const every = Math.max(1, Math.ceil(cs.length / 8));
    cs.forEach((c, i) => {
      if (i % every && i !== cs.length - 1) return;
      svg("text", { x: x(c), y: H - B + 16, class: "bh-tick", "text-anchor": "middle" }, s).textContent = data.commits[c].sha.slice(0, 7);
    });
    for (const line of series.lines) {
      const lp = line.points.filter((p) => p[key]);
      if (lp.length > 1) {
        svg("polyline", {
          points: lp.map((p) => `${x(p.c)},${y(p[key][0])}`).join(" "),
          class: "bh-line", stroke: slot(line.class),
        }, s);
      }
      for (const p of lp) {
        const cm = data.commits[p.c];
        const a = svg("a", { href: `${data.repo}/commit/${cm.sha}`, target: "_top" }, s);
        const [m, h] = p[key];
        if (h > 0) svg("line", { x1: x(p.c), x2: x(p.c), y1: y(m - h), y2: y(m + h), class: "bh-whisker", stroke: slot(line.class) }, a);
        svg("circle", { cx: x(p.c), cy: y(m), r: 4.5, class: "bh-dot", fill: slot(line.class) }, a);
        svg("circle", { cx: x(p.c), cy: y(m), r: 12, class: "bh-hit" }, a);
        svg("title", {}, a).textContent =
          `${cm.sha.slice(0, 7)} ${cm.subject}\n${cm.date.slice(0, 10)} · ${data.classes[line.class]}\n` +
          `${metric[1]}: ${fmt(m)} ± ${fmt(h)} ${unit} (95% interval, ${p.n} rounds)` + (p.same_bytes ? "" : "\nSAME BYTES: NO");
      }
    }
  }

  // The hero chart (#bench-hero, first on the page), after wikidata-pq's: every figure
  // of one corpus (each metric at 1 and 4 threads) over master's recorded commits, each
  // line on its own scale (its own range, padded), so every figure's shape shows at once;
  // the left axis, in that figure's own units, is the highlighted line's, with its 95%
  // band. Pointing at (or focusing) a legend entry or a line highlights it and fades the
  // rest; a click or tap keeps it highlighted, and another releases it. A class of results
  // (a major bench version, a runner CPU and core count, a set of corpora) is never joined
  // to another: a dashed rule names each new one.
  const hero = { pinned: null, hover: null, commit: null };
  const HERO_DEFAULT = "mb_s-4";
  function hero_(data, root) {
    const host = document.getElementById("bench-hero") || el("div", {}, root);
    host.textContent = "";
    host.classList.add("bh-root");
    const box = el("section", { class: "bh-hero" }, host);
    const corpora = [...new Set(data.series.map((s) => s.corpus))];
    if (!corpora.includes(state.heroCorpus)) state.heroCorpus = corpora.includes("wikidata") ? "wikidata" : corpora[0];
    const head = el("div", { class: "bh-hero-head" }, box);
    el("h2", { class: "bh-hero-title", id: "every-figure", text: "Every figure, every merge" }, head);
    const seg = el("div", { class: "bh-seg", role: "radiogroup", "aria-label": "Corpus" }, head);
    for (const c of corpora) {
      const b = el("button", { type: "button", role: "radio", "aria-checked": String(c === state.heroCorpus), text: c, "data-hero-id": `corpus-${c}` }, seg);
      b.addEventListener("click", () => {
        state.heroCorpus = c;
        try { localStorage.setItem("mibz2-bench", JSON.stringify(state)); } catch (e) { /* (none) */ }
        hero_(data, root);
        host.querySelector(`[data-hero-id="corpus-${c}"]`).focus();
      });
    }
    // the figures: each metric at each thread count, its points by commit
    const figs = [];
    METRICS.forEach((m, mi) => {
      for (const s of data.series.filter((q) => q.corpus === state.heroCorpus).sort((a, b) => a.threads - b.threads)) {
        const vals = new Map();
        for (const l of s.lines) for (const p of l.points) if (p[m[0]]) vals.set(p.c, { v: p[m[0]][0], h: p[m[0]][1], n: p.n, cls: l.class, same: p.same_bytes });
        if (vals.size) figs.push({ key: `${m[0]}-${s.threads}`, m, mi, threads: s.threads, label: `${m[1]}, ${s.threads} thread${s.threads === 1 ? "" : "s"}`, vals });
      }
    });
    const byKey = new Map(figs.map((f) => [f.key, f]));
    const cs = [...new Set(figs.flatMap((f) => [...f.vals.keys()]))].sort((a, b) => a - b);
    const focusP = el("p", { class: "bh-hero-focus" }, box);
    const body = el("div", { class: "bh-hero-chart" }, box);
    const link = el("p", { class: "bh-hero-commit" }, box);
    const legend = el("div", { class: "bh-hero-legend" }, box);
    el("p", { class: "bh-hero-note", text: "Each line is scaled to its own range over the commits recorded, so every figure's shape shows at once; the axis on the left, in the figure's own units, is the highlighted line's, with its 95% interval as a band. Point at a legend entry or a line to highlight it; click or tap to keep it highlighted, and again to release it (on the keyboard: Tab to an entry, Enter to keep; on the chart, ←/→ a commit, ↑/↓ a line). Solid lines are one thread, dashed ones four. A dashed rule marks a new class of results (a bench version, a runner kind, a set of corpora), whose lines are never joined to the last's. Each commit's label links to its pull request." }, box);
    const tip = el("div", { class: "bh-tip", hidden: "" }, box);
    if (!cs.length) { el("p", { text: "No points yet." }, body); return; }
    if (!byKey.has(hero.pinned)) hero.pinned = null;
    if (!byKey.has(hero.hover)) hero.hover = null;
    const current = () => hero.pinned || hero.hover || (byKey.has(HERO_DEFAULT) ? HERO_DEFAULT : figs[0].key);
    const tag = (cm) => (cm.pr ? `#${cm.pr}` : cm.sha.slice(0, 7));

    const W = Math.max(300, Math.round(body.clientWidth) || 700), H = W < 560 ? 320 : 400;
    const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, role: "img", "aria-label": `${figs.length} figures of the ${state.heroCorpus} corpus over ${cs.length} master commits, each on its own scale` }, body);
    for (const f of figs) {
      const vs = [...f.vals.values()].flatMap((o) => [o.v - o.h, o.v + o.h]);
      const lo = Math.min(...vs), hi = Math.max(...vs);
      const pad = (hi - lo) * 0.12 || Math.abs(hi) * 0.05 || 1;
      f.ticks = ticks(Math.max(0, lo - pad), hi + pad, W < 560 ? 4 : 5);
      f.lo = f.ticks[0]; f.hi = f.ticks[f.ticks.length - 1];
    }
    const meas = svg("text", { class: "bh-tick", x: 0, y: -50 }, s);
    const textW = (t) => { meas.textContent = t; return meas.getComputedTextLength(); };
    let tw = 24;
    for (const f of figs) for (const t of f.ticks) tw = Math.max(tw, textW(f.m[3](t)));
    const M = { l: Math.ceil(tw) + 14, r: 14, t: 26, b: 30 };
    const inner = W - M.l - M.r, ih = H - M.t - M.b;
    const xi = new Map(cs.map((c, i) => [c, i]));
    const padX = cs.length > 1 ? Math.min(24, inner / (cs.length - 1) / 2) : 0;
    const x = (c) => (cs.length === 1 ? M.l + inner / 2 : M.l + padX + (xi.get(c) * (inner - 2 * padX)) / (cs.length - 1));
    const y = (f, v) => M.t + ((f.hi - v) * ih) / (f.hi - f.lo || 1);

    const gridG = svg("g", {}, s);
    const deco = svg("g", {}, s);
    svg("line", { x1: M.l, x2: W - M.r, y1: H - M.b, y2: H - M.b, class: "bh-grid" }, deco);
    // each class's first commit: a dashed rule before it (none before the first), and its name
    const first = new Map();
    for (const f of figs) for (const [c, o] of f.vals) if (!first.has(o.cls) || c < first.get(o.cls)) first.set(o.cls, c);
    for (const [cls, c] of [...first].sort((a, b) => a[1] - b[1])) {
      const i = xi.get(c);
      const xx = i ? (x(cs[i - 1]) + x(c)) / 2 : M.l;
      if (i) svg("line", { x1: xx, x2: xx, y1: M.t - 8, y2: H - M.b, class: "bh-hero-rule" }, deco);
      svg("text", { x: xx + 3, y: M.t - 12, class: "bh-tick" }, deco).textContent = data.classes[cls];
    }
    const placed = [];
    for (const c of cs.slice().reverse()) {
      const cm = data.commits[c], text = tag(cm), w = textW(text);
      const lx = Math.max(M.l + w / 2, Math.min(W - w / 2, x(c)));
      if (!placed.every((q) => Math.abs(q - lx) >= w + 12)) continue;
      placed.push(lx);
      const a = svg("a", { href: cm.pr ? `${data.repo}/pull/${cm.pr}` : `${data.repo}/commit/${cm.sha}`, class: "bh-hero-xlink", "aria-label": `${text}: ${cm.subject}` }, deco);
      svg("text", { x: lx, y: H - M.b + 17, "text-anchor": "middle", class: "bh-tick" }, a).textContent = text;
    }
    meas.remove();

    const bandG = svg("g", {}, s);
    const lineG = svg("g", {}, s);
    const paths = new Map();
    for (const f of figs) {
      // one path per class, never joined across a class's boundary
      let d = "", prev = null;
      for (const c of cs) {
        if (!f.vals.has(c)) { prev = null; continue; }
        const o = f.vals.get(c);
        d += `${prev !== null && prev === o.cls ? "L" : "M"}${x(c).toFixed(1)} ${y(f, o.v).toFixed(1)}`;
        prev = o.cls;
      }
      const p = svg("path", { d: d + (f.vals.size === 1 ? "h0" : ""), class: `bh-hero-line${f.threads === 1 ? "" : " bh-dashed"}`, stroke: slot(f.mi) }, lineG);
      paths.set(f.key, p);
    }
    const axisG = svg("g", {}, s);
    const cross = svg("line", { y1: M.t, y2: H - M.b, class: "bh-cross", visibility: "hidden" }, s);
    const dots = svg("g", {}, s);
    const hit = svg("rect", { x: M.l - 8, y: M.t - 6, width: inner + 16, height: ih + 6, class: "bh-hit bh-hero-hit", tabindex: 0, "data-hero-id": "plot", "aria-label": "The chart: ←/→ moves between commits, ↑/↓ between lines, Enter keeps the highlighted line, Escape releases it" }, s);

    const paint = () => {
      const key = current(), f = byKey.get(key), active = !!(hero.pinned || hero.hover);
      for (const [k, p] of paths) {
        p.classList.toggle("is-on", k === key);
        p.classList.toggle("is-dim", active && k !== key);
      }
      lineG.appendChild(paths.get(key));
      gridG.textContent = ""; axisG.textContent = ""; dots.textContent = ""; bandG.textContent = "";
      const col = slot(f.mi);
      svg("line", { x1: M.l - 0.5, x2: M.l - 0.5, y1: M.t, y2: H - M.b, class: "bh-hero-axis", stroke: col }, axisG);
      for (const t of f.ticks) {
        svg("line", { x1: M.l, x2: W - M.r, y1: y(f, t), y2: y(f, t), class: "bh-grid" }, gridG);
        svg("line", { x1: M.l - 5, x2: M.l - 0.5, y1: y(f, t), y2: y(f, t), class: "bh-hero-axis", stroke: col }, axisG);
        svg("text", { x: M.l - 8, y: y(f, t) + 4, "text-anchor": "end", class: "bh-tick" }, axisG).textContent = f.m[3](t);
      }
      // the highlighted line's 95% band and whiskers, one band per class
      let run = [];
      const flush = () => {
        if (run.length > 1) {
          const up = run.map((c) => `${x(c)},${y(f, f.vals.get(c).v + f.vals.get(c).h)}`);
          const dn = run.map((c) => `${x(c)},${y(f, f.vals.get(c).v - f.vals.get(c).h)}`).reverse();
          svg("polygon", { points: [...up, ...dn].join(" "), class: "bh-hero-band", fill: col }, bandG);
        }
        run = [];
      };
      for (const c of cs) {
        if (!f.vals.has(c)) { flush(); continue; }
        if (run.length && f.vals.get(run[run.length - 1]).cls !== f.vals.get(c).cls) flush();
        run.push(c);
        const o = f.vals.get(c);
        svg("line", { x1: x(c), x2: x(c), y1: y(f, o.v - o.h), y2: y(f, o.v + o.h), class: "bh-whisker", stroke: col }, dots);
        svg("circle", { cx: x(c), cy: y(f, o.v), r: hero.commit === c ? 5 : 3.5, class: "bh-marker", fill: col }, dots);
      }
      flush();
      const lastC = cs.filter((c) => f.vals.has(c)).pop(), lv = f.vals.get(lastC);
      focusP.textContent = "Axis: ";
      el("strong", { text: f.label }, focusP);
      focusP.append(` (${f.m[2]}) · ${f.m[3](lv.v)} ± ${f.m[3](lv.h)} at ${tag(data.commits[lastC])}${hero.pinned ? " · kept: click it again to release it" : ""}`);
      for (const b of legend.querySelectorAll(".bh-hero-item")) {
        b.classList.toggle("is-focus", b.dataset.key === key);
        b.setAttribute("aria-pressed", String(b.dataset.key === hero.pinned));
      }
    };
    const commitLinks = (c) => {
      const cm = data.commits[c];
      link.textContent = "";
      el("strong", { text: tag(cm) }, link);
      link.append(` · ${cm.date.slice(0, 10)} · ${cm.subject} · `);
      el("a", { href: `${data.repo}/commit/${cm.sha}`, text: `commit ${cm.sha.slice(0, 7)}` }, link);
      if (cm.pr) { link.append(" · "); el("a", { href: `${data.repo}/pull/${cm.pr}`, text: `pull request #${cm.pr}` }, link); }
      if (cm.run) { link.append(" · "); el("a", { href: cm.run, text: "bench run" }, link); }
    };
    const hideTip = () => { tip.hidden = true; cross.setAttribute("visibility", "hidden"); hero.commit = null; };
    const show = (c, py) => {
      hero.commit = c;
      commitLinks(c);
      cross.setAttribute("x1", x(c)); cross.setAttribute("x2", x(c)); cross.setAttribute("visibility", "visible");
      paint();
      const key = current(), cm = data.commits[c];
      tip.textContent = "";
      el("div", { class: "bh-tip-title", text: `${tag(cm)} · ${cm.sha.slice(0, 7)} · bench ${cm.bench}` }, tip);
      el("div", { class: "bh-tip-subject", text: cm.subject.length > 90 ? `${cm.subject.slice(0, 88)}…` : cm.subject }, tip);
      for (const f of figs) {
        if (!f.vals.has(c)) continue;
        const o = f.vals.get(c);
        const row = el("div", { class: `bh-tip-row${f.key === key ? " is-on" : ""}` }, tip);
        el("span", { class: "bh-swatch", style: `background:${slot(f.mi)}` }, row);
        el("span", { text: `${f.label}: ${f.m[3](o.v)} ± ${f.m[3](o.h)} ${f.m[2]}${o.same ? "" : " (SAME BYTES: NO)"}` }, row);
      }
      tip.hidden = false;
      const r = s.getBoundingClientRect(), hb = box.getBoundingClientRect(), k = r.width / W;
      const px = r.left - hb.left + x(c) * k, tw2 = tip.offsetWidth, th = tip.offsetHeight;
      let left = px + 14;
      if (left + tw2 > hb.width) left = px - tw2 - 14;
      tip.style.left = `${Math.max(0, Math.min(hb.width - tw2, left))}px`;
      const top = r.top - hb.top + M.t * k, bottom = r.top - hb.top + (H - M.b) * k;
      if (W < 560) tip.style.top = `${r.bottom - hb.top + 4}px`;
      else tip.style.top = `${py < M.t + ih / 2 ? Math.max(top, bottom - th) : top}px`;
    };
    const nearest = (c, py) => {
      let best = null, bd = Infinity;
      for (const f of figs) {
        if (!f.vals.has(c)) continue;
        const d = Math.abs(y(f, f.vals.get(c).v) - py);
        if (d < bd) { bd = d; best = f.key; }
      }
      return best;
    };
    const at = (ev) => {
      const r = s.getBoundingClientRect();
      const px = ((ev.clientX - r.left) * W) / r.width, py = ((ev.clientY - r.top) * H) / r.height;
      const c = cs.reduce((a, q) => (Math.abs(x(q) - px) < Math.abs(x(a) - px) ? q : a), cs[0]);
      return { c, py, key: nearest(c, py) };
    };
    const move = (ev) => { const { c, py, key } = at(ev); if (!hero.pinned && key) hero.hover = key; show(c, py); };
    hit.addEventListener("pointermove", move);
    hit.addEventListener("pointerdown", move);
    hit.addEventListener("click", (ev) => {
      const { c, py, key } = at(ev);
      if (hero.pinned) { hero.pinned = null; hero.hover = key; } else hero.pinned = key;
      show(c, py);
    });
    hit.addEventListener("pointerleave", () => { hideTip(); hero.hover = null; paint(); });
    let ki = cs.length - 1;
    const keyShow = () => { const f = byKey.get(current()); show(cs[ki], f.vals.has(cs[ki]) ? y(f, f.vals.get(cs[ki]).v) : M.t); };
    hit.addEventListener("focus", keyShow);
    hit.addEventListener("blur", () => { hideTip(); hero.hover = null; paint(); });
    hit.addEventListener("keydown", (ev) => {
      if (ev.key === "ArrowLeft") ki = Math.max(0, ki - 1);
      else if (ev.key === "ArrowRight") ki = Math.min(cs.length - 1, ki + 1);
      else if (ev.key === "ArrowUp" || ev.key === "ArrowDown") {
        const c = cs[ki];
        const order = figs.filter((f) => f.vals.has(c)).sort((a, b) => y(a, a.vals.get(c).v) - y(b, b.vals.get(c).v)).map((f) => f.key);
        const i = order.indexOf(current());
        const next = order[i < 0 ? 0 : Math.max(0, Math.min(order.length - 1, i + (ev.key === "ArrowUp" ? -1 : 1)))];
        if (hero.pinned) hero.pinned = next; else hero.hover = next;
      } else if (ev.key === "Enter" || ev.key === " ") {
        if (hero.pinned) { hero.hover = hero.pinned; hero.pinned = null; } else hero.pinned = current();
      } else if (ev.key === "Escape") { hero.pinned = null; hero.hover = null; }
      else return;
      ev.preventDefault();
      keyShow();
    });

    // The legend under the chart, by metric: pointing at or focusing an entry highlights
    // its line and fades the rest; a click, tap or Enter keeps it, and another releases it
    METRICS.forEach((m, mi) => {
      const items = figs.filter((f) => f.mi === mi);
      if (!items.length) return;
      const g = el("div", { class: "bh-hero-lgroup", role: "group", "aria-label": m[1] }, legend);
      el("div", { class: "bh-hero-ghead", text: `${m[1]} (${m[2]})` }, g);
      for (const f of items) {
        const b = el("button", { type: "button", class: "bh-hero-item", "aria-pressed": "false", "data-key": f.key, "data-hero-id": `fig-${f.key}` }, g);
        svg("line", { x1: 1, x2: 21, y1: 5, y2: 5, stroke: slot(mi), "stroke-width": 2.5, "stroke-dasharray": f.threads === 1 ? null : "5 3" }, svg("svg", { width: 22, height: 10, "aria-hidden": "true" }, b));
        const lv = f.vals.get(cs.filter((c) => f.vals.has(c)).pop());
        el("span", { text: `${f.threads} thread${f.threads === 1 ? "" : "s"}` }, b);
        el("span", { class: "bh-num", text: `${f.m[3](lv.v)} ± ${f.m[3](lv.h)}` }, b);
        b.addEventListener("click", () => { hero.pinned = hero.pinned === f.key ? null : f.key; hero.hover = f.key; paint(); });
        const enter = () => { hero.hover = f.key; paint(); };
        const leave = () => { if (hero.hover === f.key) { hero.hover = null; paint(); } };
        b.addEventListener("pointerenter", enter);
        b.addEventListener("focus", enter);
        b.addEventListener("pointerleave", leave);
        b.addEventListener("blur", leave);
      }
    });
    commitLinks(cs[cs.length - 1]);
    paint();
  }

  function table(data, metric, root) {
    const [key, label, unit, fmt] = metric;
    const d = el("details", { class: "bh-table" }, root);
    el("summary", { text: `Every point as a table (${label}, ${unit})` }, d);
    const t = el("table", {}, d);
    const hr = el("tr", {}, el("thead", {}, t));
    for (const h of ["corpus", "threads", "commit", "date", "class", `${label} (${unit})`, "± 95%", "rounds"]) el("th", { text: h }, hr);
    const tb = el("tbody", {}, t);
    for (const s of data.series) {
      if (state.threads !== "all" && String(s.threads) !== state.threads) continue;
      for (const l of s.lines) for (const p of l.points) {
        if (!p[key]) continue;
        const cm = data.commits[p.c];
        const tr = el("tr", {}, tb);
        el("td", { text: s.corpus }, tr);
        el("td", { text: s.threads }, tr);
        const a = el("a", { href: `${data.repo}/commit/${cm.sha}`, text: cm.sha.slice(0, 7) }, el("td", {}, tr));
        a.title = cm.subject;
        el("td", { text: cm.date.slice(0, 10) }, tr);
        el("td", { text: data.classes[l.class] }, tr);
        el("td", { text: fmt(p[key][0]) }, tr);
        el("td", { text: fmt(p[key][1]) }, tr);
        el("td", { text: p.n }, tr);
      }
    }
  }

  function render(root, data) {
    root.textContent = "";
    root.classList.add("bh-root");
    if (!data.commits.length) {
      el("p", { text: "No master commit is recorded yet: the first record appears once the Benchmark workflow has run on a push to master." }, root);
      return;
    }
    hero_(data, root);
    const controls = el("div", { class: "bh-controls" }, root);
    const pick = (label, options, value, set) => {
      const lab = el("label", {}, controls);
      lab.append(`${label} `);
      const sel = el("select", {}, lab);
      for (const [v, t] of options) el("option", { value: v, text: t, ...(v === value ? { selected: "" } : {}) }, sel);
      sel.addEventListener("change", () => {
        set(sel.value);
        try { localStorage.setItem("mibz2-bench", JSON.stringify(state)); } catch (e) { /* (none) */ }
        render(root, data);
      });
    };
    pick("Figure", METRICS.map((m) => [m[0], `${m[1]} (${m[2]})`]), state.metric, (v) => (state.metric = v));
    const threads = [...new Set(data.series.map((s) => String(s.threads)))];
    pick("Threads", [["all", "all"], ...threads.map((t) => [t, t])], state.threads, (v) => (state.threads = v));
    const legend = el("ul", { class: "bh-legend" }, root);
    data.classes.forEach((c, i) => {
      const li = el("li", {}, legend);
      el("span", { class: "bh-swatch", style: `background:${slot(i)}` }, li);
      li.append(c);
    });
    const metric = METRICS.find((m) => m[0] === state.metric) || METRICS[0];
    const grid = el("div", { class: "bh-grid-charts" }, root);
    for (const s of data.series) {
      if (state.threads !== "all" && String(s.threads) !== state.threads) continue;
      chart(data, s, metric, grid);
    }
    table(data, metric, root);
    el("p", { class: "bh-generated", text: `${data.commits.length} commits recorded; data generated ${data.generated}.` }, root);
  }

  // The Alternatives page's chart (#alternatives-hero): master's latest record,
  // each other decoder's time (or CPU, or peak RSS) over mibz2's on each corpus, the
  // geometric mean of the rounds' paired ratios with its 95% interval, on a log scale
  // about 1 (mibz2). Pointing at or focusing a legend entry highlights that decoder and
  // fades the rest; a click, tap or Enter keeps it. A row's tooltip gives every decoder's
  // own figures there. The legend is in two groups, block-parallel and single-threaded
  // (the record's `parallel`): a single-threaded decoder runs at 1 thread only, so at N
  // threads its point (hollow, its whisker dashed) is its 1-thread run against mibz2's
  // at N, and says so.
  const CMP_METRICS = [
    ["wall", "Wall time", "wall_ratio"],
    ["cpu", "CPU time", "cpu_ratio"],
    ["rss", "Peak RSS", "rss_ratio"],
  ];
  const cmp = { pinned: null, hover: null };
  // Each decoder's colour slot, fixed (a decoder keeps its colour whichever ran), from
  // the second (bench.css: --bh-s2..s8)
  const CMP_ORDER = ["pbz2", "lbzip2", "indexed_bzip2", "fbz", "lbzip2-rs", "libbzip2", "libbz2-rs"];
  const CMP_SLOTS = 8;
  function alternativesHero(d, host) {
    host.textContent = "";
    host.classList.add("bh-root");
    const box = el("section", { class: "bh-hero" }, host);
    if (!d) { el("p", { text: "No master commit is recorded yet." }, box); return; }
    const others = d.arms.filter((a) => a !== d.reference);
    const single = (a) => d.parallel?.[a] === false;
    const threadsAll = [...new Set(Object.values(d.figures).flatMap((f) => Object.keys(f)))].sort((a, b) => a - b);
    if (!threadsAll.includes(String(state.cmpThreads))) state.cmpThreads = threadsAll[threadsAll.length - 1];
    if (!CMP_METRICS.some((m) => m[0] === state.cmpMetric)) state.cmpMetric = "wall";
    const metric = CMP_METRICS.find((m) => m[0] === state.cmpMetric);
    const save = () => { try { localStorage.setItem("mibz2-bench", JSON.stringify(state)); } catch (e) { /* (none) */ } };
    const head = el("div", { class: "bh-hero-head" }, box);
    el("h2", { class: "bh-hero-title", id: "every-alternative", text: "mibz2 and its alternatives, now" }, head);
    const bar = el("div", { class: "bh-hero-bar" }, head);
    const seg = (label, opts, value, set, id) => {
      const g = el("div", { class: "bh-seg", role: "radiogroup", "aria-label": label }, bar);
      for (const [v, t] of opts) {
        const b = el("button", { type: "button", role: "radio", "aria-checked": String(String(v) === String(value)), text: t, "data-hero-id": `${id}-${v}` }, g);
        b.addEventListener("click", () => { set(v); save(); alternativesHero(d, host); host.querySelector(`[data-hero-id="${id}-${v}"]`).focus(); });
      }
    };
    seg("Figure", CMP_METRICS.map((m) => [m[0], m[1]]), state.cmpMetric, (v) => (state.cmpMetric = v), "cmp-metric");
    seg("Threads", threadsAll.map((t) => [t, `${t} thread${t === "1" ? "" : "s"}`]), state.cmpThreads, (v) => (state.cmpThreads = v), "cmp-threads");
    const tag = d.pr ? `#${d.pr}` : d.head.slice(0, 7);
    const sub = el("p", { class: "bh-hero-focus" }, box);
    sub.append(`At master's latest record, `);
    el("a", { href: d.pr ? `${d.repo}/pull/${d.pr}` : `${d.repo}/commit/${d.head}`, text: `${tag} (${d.head.slice(0, 7)})` }, sub);
    sub.append(`, ${d.date.slice(0, 10)}: ${d.rounds} rounds on one ${d.cpu} runner, ${d.cores} cores · `);
    if (d.run) el("a", { href: d.run, text: "bench run" }, sub);
    const body = el("div", { class: "bh-hero-chart" }, box);
    const legend = el("div", { class: "bh-hero-legend bh-cmp-legend" }, box);
    const many = state.cmpThreads !== "1";
    el("p", { class: "bh-hero-note", text: `Each point is an alternative's ${metric[1].toLowerCase()} over mibz2's on that corpus at ${state.cmpThreads} thread${many ? "s" : ""}: the geometric mean of the rounds' paired ratios, with its 95% interval as a whisker, on a log scale.${others.some(single) ? ` A single-threaded decoder (hollow, its whisker dashed) runs at 1 thread only${many ? `: here its point is its 1-thread run against mibz2's at ${state.cmpThreads} threads` : ""}.` : ""} Right of the line (above 1×), the alternative takes more ${state.cmpMetric === "rss" ? "memory" : "time"} than mibz2, so mibz2 is ${state.cmpMetric === "rss" ? "leaner" : "faster"}; left of it, the alternative is. Point at a legend entry to highlight that alternative; click or tap to keep it. Point at a row for every decoder's own figures there. The tables below give the same run in full.` }, box);
    const tip = el("div", { class: "bh-tip", hidden: "" }, box);

    const order = Object.keys(d.figures).sort((a, b) => (a === "wikidata" ? -1 : b === "wikidata" ? 1 : a.localeCompare(b)));
    const rows = order.filter((c) => d.figures[c][state.cmpThreads]);
    const pts = [];
    for (const c of rows) for (const a of others) {
      const r = d.figures[c][state.cmpThreads].arms[a]?.[metric[2]];
      if (r) pts.push({ c, a, r });
    }
    if (!pts.length) { el("p", { text: "No figures for this choice." }, body); return; }
    const lo = Math.min(1, ...pts.map((p) => p.r[1])), hi = Math.max(1, ...pts.map((p) => p.r[2]));
    const L0 = Math.log(lo) - 0.12, L1 = Math.log(hi) + 0.12;
    const W = Math.max(300, Math.round(body.clientWidth) || 700);
    const narrow = W < 560;
    const step = narrow ? 11 : 12; // (between one decoder's point and the next in a row)
    const rowH = Math.max(narrow ? 34 : 38, others.length * step + 14);
    const M = { l: narrow ? 92 : 190, r: 18, t: 26, b: 34 };
    const H = M.t + M.b + rows.length * rowH;
    const x = (v) => M.l + ((Math.log(v) - L0) * (W - M.l - M.r)) / (L1 - L0);
    const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, role: "img", "aria-label": `Each alternative's ${metric[1]} over mibz2's, per corpus, at ${state.cmpThreads} threads` }, body);
    const bandsG = svg("g", {}, s);
    // ticks: 1-2-5 steps across the range, and the 1× line
    const cand = narrow ? [0.1, 0.25, 0.5, 1, 2, 5, 10, 20] : [0.1, 0.2, 0.25, 0.5, 0.67, 0.75, 1, 1.25, 1.5, 2, 3, 4, 5, 7, 10, 20, 50];
    // (1× always; then each candidate at least 34px from every tick kept, so a wide range
    // keeps its labels apart)
    const kept = [1];
    for (const t of cand.filter((t) => t !== 1 && Math.log(t) >= L0 && Math.log(t) <= L1)) {
      if (kept.every((k) => Math.abs(x(t) - x(k)) >= 34)) kept.push(t);
    }
    for (const t of kept.filter((t) => Math.log(t) >= L0 && Math.log(t) <= L1)) {
      svg("line", { x1: x(t), x2: x(t), y1: M.t, y2: H - M.b, class: t === 1 ? "bh-cmp-one" : "bh-grid" }, s);
      svg("text", { x: x(t), y: H - M.b + 16, "text-anchor": "middle", class: "bh-tick" }, s).textContent = `${t}×`;
    }
    svg("text", { x: x(1), y: M.t - 10, "text-anchor": "middle", class: "bh-tick bh-cmp-ref" }, s).textContent = "mibz2";
    if (Math.log(lo) < 0) svg("text", { x: M.l + 2, y: H - 4, class: "bh-tick" }, s).textContent = `← the alternative ${state.cmpMetric === "rss" ? "leaner" : "faster"}`;
    svg("text", { x: W - M.r, y: H - 4, "text-anchor": "end", class: "bh-tick" }, s).textContent = `mibz2 ${state.cmpMetric === "rss" ? "leaner" : "faster"} →`;
    const colour = (a) => {
      const k = CMP_ORDER.includes(a) ? 1 + CMP_ORDER.indexOf(a) : 1 + CMP_ORDER.length + others.filter((o) => !CMP_ORDER.includes(o)).indexOf(a);
      return k < CMP_SLOTS ? `var(--bh-s${k + 1})` : "var(--bh-old)";
    };
    const runAt = (a) => (single(a) && many ? " (1 thread)" : "");
    const groups = new Map(others.map((a) => [a, svg("g", { class: "bh-cmp-arm" }, s)]));
    rows.forEach((c, i) => {
      const y0 = M.t + i * rowH, yc = y0 + rowH / 2;
      if (i % 2) svg("rect", { x: 0, y: y0, width: W, height: rowH, class: "bh-cmp-band" }, bandsG);
      const f = d.figures[c][state.cmpThreads], me = f.arms[d.reference];
      const lab = svg("text", { x: 4, y: yc - (narrow ? 2 : 3), class: "bh-cmp-corpus" }, s);
      lab.textContent = c;
      svg("text", { x: 4, y: yc + (narrow ? 11 : 12), class: "bh-tick" }, s).textContent =
        narrow ? `${me.mb_s.toFixed(0)} MB/s` : `mibz2 ${me.mb_s.toFixed(me.mb_s < 100 ? 1 : 0)} MB/s · ${me.rss_mb.toFixed(0)} MB`;
      others.forEach((a, j) => {
        const r = f.arms[a]?.[metric[2]];
        if (!r) return;
        const yy = yc + (j - (others.length - 1) / 2) * step;
        const g = groups.get(a);
        svg("line", { x1: x(r[1]), x2: x(r[2]), y1: yy, y2: yy, stroke: colour(a), class: "bh-cmp-ci", "stroke-dasharray": single(a) ? "3 3" : null }, g);
        svg("circle", single(a)
          ? { cx: x(r[0]), cy: yy, r: 4, stroke: colour(a), class: "bh-cmp-hollow" }
          : { cx: x(r[0]), cy: yy, r: 5, fill: colour(a), class: "bh-marker" }, g);
        svg("text", { x: x(r[0]) + (r[0] >= 1 ? 9 : -9), y: yy + 4, "text-anchor": r[0] >= 1 ? "start" : "end", class: "bh-tick bh-cmp-val" }, g).textContent = `${r[0].toFixed(2)}×`;
      });
      const hit = svg("rect", { x: 0, y: y0, width: W, height: rowH, class: "bh-hit", tabindex: 0, "aria-label": `${c}: ${others.map((a) => `${a}${runAt(a)} ${f.arms[a]?.[metric[2]]?.[0]?.toFixed(2)}×`).join(", ")}` }, s);
      const show = () => {
        tip.textContent = "";
        el("div", { class: "bh-tip-title", text: `${c} · ${state.cmpThreads} thread${state.cmpThreads === "1" ? "" : "s"}${f.same_bytes ? " · same bytes" : " · BYTES DIFFER"}` }, tip);
        const corp = d.corpora[c];
        if (corp) el("div", { class: "bh-tip-subject", text: `${(corp.bytes / 1e6).toFixed(corp.bytes < 1e6 ? 3 : 1)} MB → ${(corp.out / 1e6).toFixed(1)} MB` }, tip);
        for (const a of d.arms) {
          const o = f.arms[a];
          if (!o) continue;
          const row = el("div", { class: `bh-tip-row${a === d.reference ? " is-on" : ""}` }, tip);
          el("span", { class: `bh-swatch${single(a) ? " is-hollow" : ""}`, style: `${single(a) ? "border-color" : "background"}:${a === d.reference ? "var(--bh-ink-2)" : colour(a)}` }, row);
          const rr = o[metric[2]];
          const who = single(a) ? ` (single-threaded${many ? ": its 1-thread run" : ""})` : "";
          el("span", { text: `${a}${who}${o.same_bytes === false ? " · BYTES DIFFER" : ""}: ${o.mb_s.toFixed(o.mb_s < 100 ? 1 : 0)} MB/s · wall ${o.wall.toFixed(3)} s · CPU ${o.cpu.toFixed(3)} s · ${o.rss_mb.toFixed(0)} MB${rr ? ` · ${rr[0].toFixed(2)}× (${rr[1].toFixed(2)}–${rr[2].toFixed(2)})` : ""}` }, row);
        }
        tip.hidden = false;
        const r = s.getBoundingClientRect(), hb = box.getBoundingClientRect(), k = r.width / W;
        tip.style.left = `${Math.max(0, Math.min(hb.width - tip.offsetWidth, r.left - hb.left + M.l * k))}px`;
        tip.style.top = `${r.top - hb.top + (y0 + rowH) * k + 4}px`;
      };
      hit.addEventListener("pointerenter", show);
      hit.addEventListener("focus", show);
      hit.addEventListener("pointerleave", () => { tip.hidden = true; });
      hit.addEventListener("blur", () => { tip.hidden = true; });
    });
    const paint = () => {
      const key = cmp.pinned || cmp.hover;
      for (const [a, g] of groups) {
        g.classList.toggle("is-dim", !!key && a !== key);
        g.classList.toggle("is-on", a === key);
      }
      for (const b of legend.querySelectorAll(".bh-hero-item")) {
        b.classList.toggle("is-focus", b.dataset.key === key);
        b.setAttribute("aria-pressed", String(b.dataset.key === cmp.pinned));
      }
    };
    const lgroups = [
      [`block-parallel (${state.cmpThreads} thread${many ? "s" : ""})`, others.filter((a) => !single(a))],
      ["single-threaded (1 thread)", others.filter(single)],
    ].filter(([, as]) => as.length);
    for (const [title, as] of lgroups) {
      const lg = el("div", { class: "bh-hero-lgroup", role: "group", "aria-label": title }, legend);
      if (lgroups.length > 1 || as.some(single)) el("div", { class: "bh-hero-ghead", text: title }, lg);
      for (const a of as) {
        const b = el("button", { type: "button", class: "bh-hero-item", "aria-pressed": "false", "data-key": a }, lg);
        el("span", { class: `bh-swatch${single(a) ? " is-hollow" : ""}`, style: `${single(a) ? "border-color" : "background"}:${colour(a)}` }, b);
        const tc = Object.entries(d.toolchain || {});
        const ver = tc.find(([k]) => k === a) || tc.find(([k]) => k.startsWith(`${a} `));
        el("span", { text: ver ? `${a} ${String(ver[1]).replace(/^lbzip2 version /, "").replace(/ \(.*\)$/, "")}` : a }, b);
        const wk = d.figures.wikidata?.[state.cmpThreads]?.arms[a]?.[metric[2]];
        el("span", { class: "bh-num", text: wk ? `wikidata ${wk[0].toFixed(2)}×` : "" }, b);
        b.addEventListener("click", () => { cmp.pinned = cmp.pinned === a ? null : a; cmp.hover = a; paint(); });
        const enter = () => { cmp.hover = a; paint(); };
        const leave = () => { if (cmp.hover === a) { cmp.hover = null; paint(); } };
        b.addEventListener("pointerenter", enter);
        b.addEventListener("focus", enter);
        b.addEventListener("pointerleave", leave);
        b.addEventListener("blur", leave);
      }
    }
    paint();
  }

  // Redrawn on resize (each chart is sized to its box), whichever the page has
  const drawn = { hero: null, cmp: null };
  let resizeT;
  window.addEventListener("resize", () => {
    clearTimeout(resizeT);
    resizeT = setTimeout(() => {
      if (drawn.hero && drawn.hero[1].isConnected) hero_(...drawn.hero);
      if (drawn.cmp && drawn.cmp[1].isConnected) alternativesHero(...drawn.cmp);
    }, 150);
  });

  function init() {
    const root = document.getElementById("bench-history");
    if (root) {
      fetch(root.dataset.src)
        .then((r) => { if (!r.ok) throw new Error(r.status); return r.json(); })
        .then((d) => { render(root, d); drawn.hero = [d, root]; })
        .catch((e) => { root.textContent = `The benchmark history could not be loaded (${e.message}).`; });
    }
    const ch = document.getElementById("alternatives-hero");
    if (ch) {
      fetch(ch.dataset.src)
        .then((r) => { if (!r.ok) throw new Error(r.status); return r.json(); })
        .then((d) => { alternativesHero(d, ch); drawn.cmp = [d, ch]; })
        .catch((e) => { ch.textContent = `The comparison could not be loaded (${e.message}).`; });
    }
  }
  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(init);
  else if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
