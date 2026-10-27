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

  // The hero chart: every figure (each metric, corpus and thread count) of master's
  // recorded commits on one chart, as its change since the first commit of its class of
  // results, turned so that up is better (throughput's ratio, the others' inverse). A
  // class (a major bench version, a runner CPU and core count, a set of corpora) is never
  // joined to another: each starts at × 1 again after a hairline naming it. Wikidata's
  // lines are bold, with their 95% band; the other corpora's are faint. The legend hides
  // or shows a metric.
  const BETTER_UP = new Set(["mb_s"]);
  function hero(data, root) {
    // (drawn into #bench-hero, first on the page, when the page has one)
    const target = document.getElementById("bench-hero") || root;
    if (target !== root) { target.textContent = ""; target.classList.add("bh-root"); }
    const box = el("section", { class: "bh-hero" }, target);
    el("h2", { class: "bh-hero-title", id: "every-figure", text: "Every figure, every merge" }, box);
    const lines = [];
    for (const s of data.series) {
      if (state.threads !== "all" && String(s.threads) !== state.threads) continue;
      for (const l of s.lines) {
        METRICS.forEach((m, mi) => {
          const pts = l.points.filter((p) => p[m[0]]).sort((a, b) => a.c - b.c);
          if (!pts.length) return;
          const [b0, h0] = pts[0][m[0]];
          lines.push({
            s, m, mi, cls: l.class,
            pts: pts.map((p, i) => {
              const [v, h] = p[m[0]];
              const f = BETTER_UP.has(m[0]) ? v / b0 : b0 / v;
              return { c: p.c, f, half: f * Math.hypot(h / v, i ? h0 / b0 : 0), v, h, n: p.n, same: p.same_bytes };
            }),
          });
        });
      }
    }
    if (!lines.length) { el("p", { text: "No points yet." }, box); return; }
    const hidden = new Set(state.heroHidden || []);
    const cs = [...new Set(lines.flatMap((l) => l.pts.map((p) => p.c)))].sort((a, b) => a - b);
    const tag = (cm) => (cm.pr ? `#${cm.pr}` : cm.sha.slice(0, 7));
    const starts = new Map(); // class → its first commit
    for (const l of lines) if (!starts.has(l.cls) || l.pts[0].c < starts.get(l.cls)) starts.set(l.cls, l.pts[0].c);
    el("p", {
      class: "bh-hero-note",
      text: "Master's recorded commits, oldest to newest. Each line is one figure's change since the " +
        "first commit of its class of results: × 1.10 is 10% better, whether faster, less CPU or " +
        "less memory. Bold lines are the Wikidata dump blocks, with their 95% interval as a band; " +
        "faint ones are the other corpora. Solid lines are one thread, dashed ones four. Hover a " +
        "point for its figures; a commit's label links to its pull request.",
    }, box);
    const legend = el("div", { class: "bh-hero-legend", role: "group", "aria-label": "Metrics shown" }, box);
    METRICS.forEach((m, mi) => {
      const b = el("button", { type: "button", "aria-pressed": String(!hidden.has(m[0])), class: "bh-hero-key" }, legend);
      el("span", { class: "bh-swatch", style: `background:${slot(mi)}` }, b);
      b.append(`${m[1]} (${m[2]})`);
      b.addEventListener("click", () => {
        const h = new Set(state.heroHidden || []);
        if (h.has(m[0])) h.delete(m[0]); else h.add(m[0]);
        state.heroHidden = [...h];
        try { localStorage.setItem("mibz2-bench", JSON.stringify(state)); } catch (e) { /* (none) */ }
        render(root, data);
      });
    });
    const shown = lines.filter((l) => !hidden.has(l.m[0]));
    const W = 960, H = 360, L = 60, R = 150, T = 26, B = 40;
    const x = (c) => L + (cs.length === 1 ? (W - L - R) / 2 : (cs.indexOf(c) * (W - L - R)) / (cs.length - 1));
    const fs = shown.flatMap((l) => l.pts.flatMap((p) => (l.s.corpus === "wikidata" ? [p.f - p.half, p.f + p.half] : [p.f])));
    const ty = ticks(Math.min(1, ...fs) - 0.02, Math.max(1, ...fs) + 0.02, 6);
    const y0 = ty[0], y1 = ty[ty.length - 1];
    const y = (f) => T + (H - T - B) * (1 - (f - y0) / (y1 - y0 || 1));
    const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, role: "img", "aria-label": "Every figure's change since the first commit of its class, by commit" }, box);
    for (const v of ty) {
      svg("line", { x1: L, x2: W - R, y1: y(v), y2: y(v), class: Math.abs(v - 1) < 1e-9 ? "bh-grid bh-unity" : "bh-grid" }, s);
      svg("text", { x: L - 6, y: y(v) + 4, class: "bh-tick", "text-anchor": "end" }, s).textContent = `× ${v.toFixed(2)}`;
    }
    for (const [cls, c] of starts) {
      const bx = cs.length === 1 ? L : x(c) - (cs.indexOf(c) ? (W - L - R) / (cs.length - 1) / 2 : 0);
      if (cs.indexOf(c)) svg("line", { x1: bx, x2: bx, y1: T - 8, y2: H - B, class: "bh-hero-boundary" }, s);
      svg("text", { x: bx + 4, y: T - 12, class: "bh-tick" }, s).textContent = data.classes[cls];
    }
    for (const c of cs) {
      const cm = data.commits[c];
      const a = svg("a", { href: cm.pr ? `${data.repo}/pull/${cm.pr}` : `${data.repo}/commit/${cm.sha}`, target: "_top" }, s);
      svg("text", { x: x(c), y: H - B + 18, class: "bh-tick bh-hero-x", "text-anchor": "middle" }, a).textContent = tag(cm);
      svg("title", {}, a).textContent = `${cm.sha.slice(0, 7)} ${cm.subject}\n${cm.date.slice(0, 10)} · bench ${cm.bench}`;
    }
    // the faint corpora first, Wikidata's bold lines and bands over them
    const order = [...shown].sort((a, b) => (a.s.corpus === "wikidata") - (b.s.corpus === "wikidata"));
    const labels = [];
    for (const l of order) {
      const main = l.s.corpus === "wikidata";
      const col = slot(l.mi);
      const g = svg("g", { class: main ? "bh-hero-line bh-hero-main" : "bh-hero-line" }, s);
      if (main && l.pts.length > 1) {
        const up = l.pts.map((p) => `${x(p.c)},${y(p.f + p.half)}`);
        const dn = l.pts.map((p) => `${x(p.c)},${y(p.f - p.half)}`).reverse();
        svg("polygon", { points: [...up, ...dn].join(" "), fill: col, class: "bh-hero-band" }, g);
      }
      if (l.pts.length > 1) {
        svg("polyline", {
          points: l.pts.map((p) => `${x(p.c)},${y(p.f)}`).join(" "),
          stroke: col, fill: "none", "stroke-dasharray": l.s.threads === 1 ? null : "6 4",
        }, g);
      }
      for (const p of l.pts) {
        const cm = data.commits[p.c];
        if (main && p.half > 0) svg("line", { x1: x(p.c), x2: x(p.c), y1: y(p.f - p.half), y2: y(p.f + p.half), stroke: col, class: "bh-whisker" }, g);
        svg("circle", { cx: x(p.c), cy: y(p.f), r: main ? 4.5 : 2.5, fill: col }, g);
        const hit = svg("circle", { cx: x(p.c), cy: y(p.f), r: 10, class: "bh-hit" }, g);
        svg("title", {}, hit).textContent =
          `${l.m[1]} · ${l.s.corpus} · ${l.s.threads} thread${l.s.threads === 1 ? "" : "s"}\n` +
          `${tag(cm)} ${cm.sha.slice(0, 7)} ${cm.subject}\n` +
          `${l.m[3](p.v)} ± ${l.m[3](p.h)} ${l.m[2]} (95% interval, ${p.n} rounds)\n` +
          `× ${p.f.toFixed(3)} ± ${p.half.toFixed(3)} since ${tag(data.commits[l.pts[0].c])}` +
          (p.same ? "" : "\nSAME BYTES: NO");
      }
      if (main) {
        const last = l.pts[l.pts.length - 1];
        labels.push({ y: y(last.f), x: x(last.c) + 10, col, text: `${l.m[1]} ${l.s.threads}t × ${last.f.toFixed(2)}` });
      }
    }
    // the end labels, spread so they don't overlap
    labels.sort((a, b) => a.y - b.y);
    for (let i = 1; i < labels.length; i++) labels[i].y = Math.max(labels[i].y, labels[i - 1].y + 13);
    for (const lb of labels) svg("text", { x: lb.x, y: lb.y + 4, class: "bh-hero-label", fill: lb.col }, s).textContent = lb.text;
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
    hero(data, root);
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

  function init() {
    const root = document.getElementById("bench-history");
    if (!root) return;
    fetch(root.dataset.src)
      .then((r) => { if (!r.ok) throw new Error(r.status); return r.json(); })
      .then((d) => render(root, d))
      .catch((e) => { root.textContent = `The benchmark history could not be loaded (${e.message}).`; });
  }
  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(init);
  else if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
