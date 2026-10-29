// "Throughput–memory frontier" (docs/site/benchmarks/frontier.md): its charts, in plain
// SVG, from data-src (assets/data/frontier.json). Only the file's measured points are
// drawn: no models, no fits. Each chart names its machine, workload, level and run.
//
// - Throughput against measured peak RSS: series i9-decoder-coop-20261029 only, one mark
//   per arm and thread count, the nondominated ones ringed and joined by their step.
// - Throughput against live table memory (computed, blocks live × each series'
//   table_mb_per_block: 3.6 MB full blocks, 1.946 MB the native corpus's): the full
//   decoder's two runs (i9-decoder-lbzip2-9-20261028's worker counts, and the coop run's
//   workers and teams), with the knee of the workstation journal §9 marked.
// - Beside it, on its own small axes: i9-inversion-pr0-20261029, the research harness's
//   inversion, which is a different level and never shares axes with the decoder.
//
// Measured RSS and computed table bytes stay on separate charts. Lines join one run's
// worker counts only, never points of two series. The page's tables hold every value,
// and stand in if this script does not run. Runs on load and on each instant navigation,
// as bench.js and couriers.js do.
(() => {
  "use strict";
  const SVG = "http://www.w3.org/2000/svg";
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

  const DECODER_RUN = "i9-decoder-lbzip2-9-20261028"; // series 1
  const INVERSION = "i9-inversion-pr0-20261029"; // series 2
  const COOP = "i9-decoder-coop-20261029"; // series 3
  const KNEE_MB = 14.4; // workstation journal §9: 0.7 of the 20 MB L3
  const JOURNAL9 = "https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-28-workstation-scaling.md";

  // Series 3's arms. Hue for the three team-planning arms (bench.css's first three slots,
  // which validate on every pair in both schemes); the workers, the baseline, in ink.
  // Each arm also has its own shape, so identity is never hue alone.
  const ARMS = {
    "mibz2": { label: "workers (arm mibz2)", shape: "circle", fill: "var(--bh-ink-2)" },
    "mibz2-coop2": { label: "teams of 2 (arm mibz2-coop2)", shape: "square", fill: "var(--bh-s1)" },
    "mibz2-coop3": { label: "teams of 3 (arm mibz2-coop3)", shape: "triangle", fill: "var(--bh-s2)" },
    "mibz2-auto": { label: "the policy, v0.2.0's default (arm mibz2-auto)", shape: "diamond", fill: "var(--bh-s3)" },
  };

  const fmt = (v, d = 0) => (v === null || v === undefined ? "–" : Number(v).toFixed(d));
  const runId = (s) => { const m = /\(run ([^;)\s]+)/.exec(s.source || ""); return m ? m[1] : s.id; };
  const k = (q) => (q.threads_used && q.blocks_live ? Math.round(q.threads_used / q.blocks_live) : 1);
  const planKey = (q) => `${q.blocks_live}x${k(q)}`;
  const planText = (q) => {
    const b = q.blocks_live, kk = k(q);
    if (kk === 1) return `${b} × 1 (${b} workers)`;
    return `${b} × ${kk} (${b} team${b === 1 ? "" : "s"} of ${kk})`;
  };
  const planShort = (q) => `${q.blocks_live}×${k(q)}`;

  const niceTicks = (hi, n = 5) => {
    const step0 = hi / n || 1;
    const mag = 10 ** Math.floor(Math.log10(step0));
    const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((v) => v >= step0);
    const out = [];
    for (let v = 0; v < hi + step * 1e-9; v += step) out.push(+v.toFixed(10));
    if (out[out.length - 1] < hi) out.push(+(out[out.length - 1] + step).toFixed(10));
    return out;
  };

  // A marker: its shape centred on (cx, cy), about 9px across
  function marker(g, shape, cx, cy, attrs) {
    const r = 4.5;
    if (shape === "square") return svg("rect", { x: cx - 4, y: cy - 4, width: 8, height: 8, rx: 1, ...attrs }, g);
    if (shape === "triangle") return svg("path", { d: `M${cx},${cy - 5.4}L${cx + 5},${cy + 3.6}L${cx - 5},${cy + 3.6}Z`, ...attrs }, g);
    if (shape === "diamond") return svg("path", { d: `M${cx},${cy - 5.6}L${cx + 5.2},${cy}L${cx},${cy + 5.6}L${cx - 5.2},${cy}Z`, ...attrs }, g);
    return svg("circle", { cx, cy, r, ...attrs }, g);
  }
  function legendItem(ul, layer, text) {
    const li = el("li", {}, ul);
    const s = svg("svg", { width: 16, height: 14, viewBox: "0 0 16 14", "aria-hidden": "true", class: "fr-key" }, li);
    if (layer.line) svg("line", { x1: 0, x2: 16, y1: 7, y2: 7, class: `fr-line ${layer.lineClass || ""}`, stroke: layer.lineStroke || layer.fill }, s);
    if (layer.ring) svg("circle", { cx: 8, cy: 7, r: 6, class: "fr-nd-ring" }, s);
    else if (layer.step) svg("path", { d: "M1,11H8V3H15", class: "fr-step" }, s);
    else marker(s, layer.shape, 8, 7, layer.hollow ? { class: "fr-mark fr-hollow", style: `stroke:${layer.fill}` } : { class: "fr-mark", fill: layer.fill });
    li.append(text || layer.label);
  }

  // One scatter chart. layers: [{label, shape, fill, hollow, line, points: [{x, y, rows,
  // title, group, plan, nd, aria}]}]; refs: vertical rules; step: points to join by a
  // step; labels: sparse direct labels.
  function scatter(fig, o) {
    const box = el("div", { class: "fr-chart" }, fig);
    const W = Math.max(300, Math.round(box.clientWidth) || 640);
    const narrow = W < 520;
    const H = o.height || (narrow ? 300 : 340);
    const M = { l: 46, r: narrow ? 14 : 22, t: 22, b: 40 };
    const pts = o.layers.flatMap((L) => L.points.map((p) => ({ ...p, layer: L })));
    const xt = niceTicks(o.xMax || Math.max(...pts.map((p) => p.x)) * 1.04, narrow ? 4 : 7);
    const yt = niceTicks(o.yMax || Math.max(...pts.map((p) => p.y)) * 1.06, 5);
    const x1 = xt[xt.length - 1], y1 = yt[yt.length - 1];
    const x = (v) => M.l + ((W - M.l - M.r) * v) / x1;
    const y = (v) => M.t + (H - M.t - M.b) * (1 - v / y1);
    const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, role: "group", "aria-label": o.aria }, box);
    // grid, axes
    for (const v of yt) {
      svg("line", { x1: M.l, x2: W - M.r, y1: y(v), y2: y(v), class: v === 0 ? "fr-base" : "bh-grid" }, s);
      svg("text", { x: M.l - 6, y: y(v) + 4, "text-anchor": "end", class: "bh-tick" }, s).textContent = String(v);
    }
    svg("line", { x1: x(0), x2: x(0), y1: M.t, y2: H - M.b, class: "fr-base" }, s);
    for (const v of xt) svg("text", { x: x(v), y: H - M.b + 15, "text-anchor": "middle", class: "bh-tick" }, s).textContent = String(v);
    svg("text", { x: (M.l + W - M.r) / 2, y: H - 6, "text-anchor": "middle", class: "bh-tick fr-axis-title" }, s).textContent = o.xLabel;
    svg("text", { x: M.l, y: 11, "text-anchor": "start", class: "bh-tick fr-axis-title" }, s).textContent = o.yLabel;
    // reference rules
    for (const r of o.refs || []) {
      svg("line", { x1: x(r.x), x2: x(r.x), y1: M.t, y2: H - M.b, class: "fr-ref" }, s);
      r.lines.forEach((t, i) => {
        svg("text", { x: x(r.x) + 5, y: M.t + 10 + i * 13, class: "bh-tick fr-ref-label" }, s).textContent = t;
      });
    }
    // the nondominated set's step: from each to the next, across then up
    if (o.step && o.step.length > 1) {
      const sp = o.step.slice().sort((a, b) => a.x - b.x);
      let d = `M${x(sp[0].x)},${y(sp[0].y)}`;
      for (const p of sp.slice(1)) d += `H${x(p.x)}V${y(p.y)}`;
      svg("path", { d, class: "fr-step" }, s);
    }
    // lines: one run's worker counts, each its own
    for (const L of o.layers) {
      if (!L.line || L.points.length < 2) continue;
      const lp = L.points.slice().sort((a, b) => a.x - b.x);
      svg("polyline", { points: lp.map((p) => `${x(p.x)},${y(p.y)}`).join(" "), class: `fr-line ${L.lineClass || ""}`, stroke: L.lineStroke || L.fill }, s);
    }
    // rings under the marks: the nondominated points
    for (const p of pts) if (p.nd) svg("circle", { cx: x(p.x), cy: y(p.y), r: 9, class: "fr-nd-ring" }, s);
    // marks
    const marks = svg("g", {}, s);
    const hover = svg("g", { class: "fr-hover", visibility: "hidden" }, s);
    for (const p of pts) {
      p.px = x(p.x); p.py = y(p.y);
      const g = svg("g", { tabindex: 0, class: "fr-pt", role: "img", "aria-label": p.aria }, marks);
      const L = p.layer;
      marker(g, L.shape, p.px, p.py, L.hollow ? { class: "fr-mark fr-hollow", style: `stroke:${L.fill}` } : { class: "fr-mark", fill: L.fill });
      p.g = g;
    }
    // direct labels, sparse
    for (const lb of o.labels || []) {
      const anchor = lb.side === "left" ? "end" : lb.side === "above" ? "middle" : "start";
      const dx = lb.side === "left" ? -10 : lb.side === "above" ? 0 : 10;
      const dy = lb.side === "above" ? -12 : 4;
      svg("text", { x: x(lb.x) + dx, y: y(lb.y) + dy, "text-anchor": anchor, class: "bh-tick fr-direct" }, s).textContent = lb.text;
    }
    // the tooltip: the nearest mark to the pointer, with every mark drawn on it
    const tip = el("div", { class: "fr-tip", hidden: "" }, box);
    // the mark, every mark drawn on it, and every other run of its plan in its series
    const near = (p0) => pts.filter((q) => Math.hypot(q.px - p0.px, q.py - p0.py) < 6 || (q.plan && q.plan === p0.plan && q.series === p0.series));
    const show = (p0) => {
      const group = near(p0).sort((a, b) => (a === p0 ? -1 : b === p0 ? 1 : 0));
      tip.textContent = "";
      const samePlan = group.length > 1 && group.every((q) => q.series === p0.series && q.plan && q.plan === p0.plan);
      if (samePlan) el("div", { class: "fr-tip-note", text: `One plan, ${group.length} measurements of it: their spread is noise around one configuration.` }, tip);
      else if (group.length > 1) el("div", { class: "fr-tip-note", text: `${group.length} marks drawn here:` }, tip);
      for (const q of group) {
        const blk = el("div", { class: "fr-tip-pt" }, tip);
        const head = el("div", { class: "fr-tip-head" }, blk);
        const ks = svg("svg", { width: 12, height: 12, viewBox: "0 0 16 14", "aria-hidden": "true" }, head);
        marker(ks, q.layer.shape, 8, 7, q.layer.hollow ? { class: "fr-mark fr-hollow", style: `stroke:${q.layer.fill}` } : { class: "fr-mark", fill: q.layer.fill });
        el("strong", { text: q.value }, head);
        head.append(` ${q.title}`);
        for (const r of q.rows) el("div", { class: "fr-tip-row", text: r }, blk);
      }
      hover.textContent = "";
      for (const q of group) svg("circle", { cx: q.px, cy: q.py, r: 8, class: "fr-hover-ring" }, hover);
      hover.setAttribute("visibility", "visible");
      tip.hidden = false;
      const r = s.getBoundingClientRect(), hb = box.getBoundingClientRect(), sc = r.width / W;
      const px = r.left - hb.left + p0.px * sc, py = r.top - hb.top + p0.py * sc;
      const tw = tip.offsetWidth, th = tip.offsetHeight;
      if (narrow) {
        tip.style.left = `${Math.max(0, Math.min(hb.width - tw, px - tw / 2))}px`;
        tip.style.top = `${r.bottom - hb.top + 2}px`;
      } else {
        let left = px + 16;
        if (left + tw > hb.width) left = px - tw - 16;
        tip.style.left = `${Math.max(0, left)}px`;
        tip.style.top = `${Math.max(0, Math.min(r.height - th, py - th / 2))}px`;
      }
    };
    const hide = () => { tip.hidden = true; hover.setAttribute("visibility", "hidden"); };
    const at = (ev) => {
      const r = s.getBoundingClientRect();
      const mx = ((ev.clientX - r.left) * W) / r.width, my = ((ev.clientY - r.top) * H) / r.height;
      let best = null, bd = 26;
      for (const p of pts) {
        const d = Math.hypot(p.px - mx, p.py - my);
        if (d < bd) { bd = d; best = p; }
      }
      return best;
    };
    s.addEventListener("pointermove", (ev) => { const p = at(ev); if (p) show(p); else hide(); });
    s.addEventListener("pointerdown", (ev) => { const p = at(ev); if (p) show(p); else hide(); });
    s.addEventListener("pointerleave", (ev) => { if (ev.pointerType === "mouse") hide(); });
    for (const p of pts) {
      p.g.addEventListener("focus", () => show(p));
      p.g.addEventListener("blur", hide);
      p.g.addEventListener("keydown", (ev) => { if (ev.key === "Escape") { hide(); p.g.blur(); } });
    }
    return box;
  }

  function caption(fig, title, series, extra) {
    el("figcaption", { text: title }, fig);
    const sub = el("p", { class: "fr-sub" }, fig);
    for (const s of series) {
      const line = el("span", { class: "fr-sub-line" }, sub);
      line.textContent = `${s.machine} · ${s.workload} · ${s.level} · ${s.placement} · run ${runId(s)}`;
    }
    if (extra) el("p", { class: "fr-sub", text: extra }, fig);
  }
  function source(fig, series, note) {
    if (note) el("p", { class: "fr-note", text: note }, fig);
    const p = el("p", { class: "fr-source" }, fig);
    p.append("Measured. Source: ");
    series.forEach((s, i) => {
      if (i) p.append("; ");
      el("code", { text: s.id }, p);
      p.append(`, ${s.source}`);
    });
    p.append(".");
  }

  const coopRows = (q) => [
    `${q.config}: ${q.threads_asked} threads asked, plan ${planText(q)}${q.threads_used !== q.threads_asked ? `, ${q.threads_used} threads used` : ""}`,
    `against p4 (paired): ${fmt(q.ratio_p4, 3)} (95%: ${fmt(q.ratio_p4_low, 3)}–${fmt(q.ratio_p4_high, 3)})`,
    `peak RSS ${fmt(q.peak_rss_mb, 1)} MB (measured) · CPU ${fmt(q.cpu_s, 3)} s`,
    `live tables ${fmt(q.table_mb, 1)} MB (computed)${q.nondominated ? " · nondominated" : ""}`,
  ];
  const coopAria = (q, axis) => `${ARMS[q.arm] ? ARMS[q.arm].label : q.arm}, ${q.config}: plan ${planText(q)}, ${q.mb_s} MB/s, ${axis}, against p4 ${fmt(q.ratio_p4, 3)} (${fmt(q.ratio_p4_low, 3)} to ${fmt(q.ratio_p4_high, 3)}), CPU ${fmt(q.cpu_s, 3)} s${q.nondominated ? ", nondominated" : ""}`;

  // Chart 1: throughput against measured peak RSS, the coop run only
  function rssChart(root, s3) {
    const fig = el("figure", { class: "fr-panel" }, root);
    caption(fig, "Throughput against measured peak RSS", [s3]);
    const lg = el("ul", { class: "fr-legend" }, fig);
    const layers = [];
    for (const [arm, a] of Object.entries(ARMS)) {
      const ps = s3.points.filter((q) => q.arm === arm && q.peak_rss_mb !== null);
      if (!ps.length) continue;
      const L = { ...a, points: ps.map((q) => ({
        x: q.peak_rss_mb, y: q.mb_s, nd: q.nondominated, series: s3.id, plan: planKey(q),
        value: `${q.mb_s} MB/s`, title: `· ${a.label}`, rows: coopRows(q),
        aria: coopAria(q, `peak RSS ${fmt(q.peak_rss_mb, 1)} MB measured`),
      })) };
      layers.push(L);
      legendItem(lg, L);
    }
    legendItem(lg, { ring: true }, "nondominated in this run (by median), joined by its step");
    const nd = s3.points.filter((q) => q.nondominated);
    const labels = [];
    const side = { "1x3": "right", "2x3": "left", "3x3": "left", "4x2": "right" };
    for (const q of nd) labels.push({ x: q.peak_rss_mb, y: q.mb_s, text: planShort(q), side: side[planKey(q)] || "right" });
    for (const q of s3.points.filter((p) => p.arm === "mibz2")) labels.push({ x: q.peak_rss_mb, y: q.mb_s, text: `p${q.threads_asked}`, side: q.threads_asked === 8 ? "left" : "right" });
    scatter(fig, {
      layers, labels, step: nd.map((q) => ({ x: q.peak_rss_mb, y: q.mb_s })),
      xLabel: "peak RSS, MB (measured, decimal)", yLabel: "MB/s decoded (higher is faster)",
      aria: `Throughput against measured peak RSS, ${s3.machine}, ${s3.level}, run ${runId(s3)}`,
    });
    source(fig, [s3], "Each mark is one arm at one thread count, its median over 20 interleaved rounds; labels give the plan (live blocks × threads a block) of the nondominated marks, and the worker counts. Where one plan ran at several thread counts (the policy at 8, 10 and 20 threads is 4 teams of 2 on this machine, beside teams of 2 at 8; 2 teams of 3 ran at both 6 and 8), its marks sit almost on one another and their spread is noise around one configuration. No worker counts below 4 were measured, so this is not the whole frontier: the low-memory end is open.");
    // every plotted point, as a table
    const det = el("details", { class: "fr-table" }, fig);
    el("summary", { text: "The plotted points as a table" }, det);
    const wrap = el("div", { class: "fr-scroll" }, det);
    const tbl = el("table", {}, wrap);
    const hr = el("tr", {}, el("thead", {}, tbl));
    for (const h of ["config", "arm", "threads asked", "plan", "MB/s", "against p4 (95%)", "peak RSS MB", "CPU-s", "live tables MB", "nondominated"]) el("th", { text: h }, hr);
    const tb = el("tbody", {}, tbl);
    for (const q of s3.points) {
      const tr = el("tr", {}, tb);
      for (const v of [q.config, q.arm, q.threads_asked, planShort(q), q.mb_s, `${fmt(q.ratio_p4, 3)} (${fmt(q.ratio_p4_low, 3)}–${fmt(q.ratio_p4_high, 3)})`, fmt(q.peak_rss_mb, 1), fmt(q.cpu_s, 3), fmt(q.table_mb, 1), q.nondominated ? "yes" : "no"]) el("td", { text: String(v) }, tr);
    }
  }

  // Chart 2: throughput against computed live table memory, the decoder's two runs. The
  // runs are different workloads (series 1 recompressed the corpus with lbzip2 -9), so
  // each has its own panel, one machine and one workload a chart, on the same scales.
  function tableChart(root, s1, s3) {
    const fig = el("figure", { class: "fr-panel" }, root);
    el("figcaption", { text: "Throughput against live table memory (computed), the full decoder" }, fig);
    el("p", { class: "fr-sub", text: "Two runs, two workloads: a panel each, on the same scales. Live tables are blocks live × each workload's mean table, computed (3.6 MB full blocks in series 1; 1.95 MB the native blocks in series 3); the dashed rule is the knee the workstation journal §9 measured at full blocks." }, fig);
    const grid = el("div", { class: "fr-facets" }, fig);
    const all = [s1, s3].filter(Boolean).flatMap((s) => s.points);
    const xMax = Math.max(...all.map((q) => q.table_mb)) * 1.04;
    const yMax = Math.max(...all.map((q) => q.mb_s)) * 1.06;
    const knee = [{ x: KNEE_MB, lines: ["knee at full blocks, about 14.4 MB:", "0.7 of the 20 MB L3", "(workstation journal §9)"] }];
    const xLabel = "live tables, MB (computed: blocks live × the workload's mean table)";
    const yLabel = "MB/s decoded (higher is faster)";
    const facet = (s, title) => {
      const f = el("div", { class: "fr-facet" }, grid);
      el("div", { class: "fr-facet-title", text: title }, f);
      el("p", { class: "fr-sub", text: `${s.machine} · ${s.workload} · ${s.level} · ${s.placement} · run ${runId(s)}` }, f);
      return f;
    };
    const workers = { label: "workers", shape: "circle", fill: "var(--bh-ink-2)", line: true, lineStroke: "var(--bh-ink-2)", lineClass: "fr-line-faint" };
    if (s1) {
      const f = facet(s1, "Series 1: workers");
      const L = { ...workers, points: s1.points.map((q) => ({
        x: q.table_mb, y: q.mb_s, series: s1.id, plan: `${q.blocks_live}x1`,
        value: `${q.mb_s} MB/s`, title: "· workers, series 1",
        rows: [`${q.config}: ${q.blocks_live} workers (${q.blocks_live} × 1)`, `live tables ${fmt(q.table_mb, 1)} MB (computed)`, `CPU ${q.cpu_s === null ? "not recorded" : `${fmt(q.cpu_s, 3)} s`} · peak RSS not recorded`],
        aria: `Series 1, ${q.config}: ${q.blocks_live} workers, ${q.mb_s} MB/s, live tables ${fmt(q.table_mb, 1)} MB computed`,
      })) };
      scatter(f, {
        layers: [L], refs: knee, xMax, yMax, xLabel, yLabel,
        labels: s1.points.map((q) => ({ x: q.table_mb, y: q.mb_s, text: q.config, side: "above" })),
        aria: `Series 1, throughput against computed live tables: ${s1.machine}, ${s1.level}, run ${runId(s1)}`,
      });
    }
    if (s3) {
      const f = facet(s3, "Series 3: workers and teams");
      const lg = el("ul", { class: "fr-legend" }, f);
      const layers = [];
      for (const [arm, a] of Object.entries(ARMS)) {
        const ps = s3.points.filter((q) => q.arm === arm);
        if (!ps.length) continue;
        const L = { ...a, line: arm === "mibz2", lineStroke: a.fill, lineClass: "fr-line-faint", points: ps.map((q) => ({
          x: q.table_mb, y: q.mb_s, series: s3.id, plan: planKey(q),
          value: `${q.mb_s} MB/s`, title: `· ${a.label}`, rows: coopRows(q),
          aria: coopAria(q, `live tables ${fmt(q.table_mb, 1)} MB computed`),
        })) };
        layers.push(L);
        legendItem(lg, L);
      }
      scatter(f, {
        layers, refs: knee, xMax, yMax, xLabel, yLabel,
        labels: s3.points.filter((q) => q.arm === "mibz2").map((q) => ({ x: q.table_mb, y: q.mb_s, text: `p${q.threads_asked}`, side: "above" })),
        aria: `Series 3, throughput against computed live tables: ${s3.machine}, ${s3.level}, run ${runId(s3)}`,
      });
    }
    source(fig, [s1, s3].filter(Boolean), "Live table memory is computed, not measured: live blocks × the workload's mean table (rows × 4 bytes): 3.6 MB for series 1's full 900,000-row blocks, 1.95 MB for series 3's native blocks (corrected on 2026-10-29 from 3.6 MB). A team of k threads shares one block's table. The native blocks knee at 4 workers, about 7.8 MB of tables, left of the rule: the knee's cause is under test. Measured memory is on the chart above. Each line joins one run's worker counts only. Marks of one plan at several thread counts are noise around one configuration. The rule is the knee the workstation journal §9 measured (at two block sizes), not a fit to these points.");
    const a = el("p", { class: "fr-source" }, fig);
    el("a", { href: JOURNAL9, text: "The workstation journal, §9" }, a);
    a.append(", where the knee was measured.");
  }

  // Beside it: the research harness's inversion, its own level, on its own small axes
  function inversionPanel(root, s2) {
    const fig = el("figure", { class: "fr-panel fr-small" }, root);
    caption(fig, "Inversion only, research harness, pinned; not the decoder", [s2], "Its own axes: these MB/s are the inversion's (MTF, build and walk), a different level from the decoder's above, and are not comparable with them.");
    const shapeOf = (q) => (k1(q) === 1 ? "circle" : k1(q) === 3 ? "triangle" : "square");
    const k1 = (q) => Math.max(1, Math.round(q.cores / q.blocks_live));
    const kinds = [
      ["circle", "production workers"],
      ["triangle", "cooperative, teams of 3"],
      ["square", "cooperative, teams of 2"],
    ];
    const lg = el("ul", { class: "fr-legend" }, fig);
    const layers = [];
    for (const [shape, label] of kinds) {
      const ps = s2.points.filter((q) => shapeOf(q) === shape);
      if (!ps.length) continue;
      const L = { label, shape, fill: "var(--bh-ink-2)", points: ps.map((q) => ({
        x: q.table_mb, y: q.mb_s, series: s2.id, plan: null,
        value: `${q.mb_s} MB/s`, title: `· ${q.config}, ${label}`,
        rows: [`${q.blocks_live} live tables on ${q.cores} cores, pinned`, `live tables ${fmt(q.table_mb, 1)} MB (computed)`, "inversion only (research harness), not the decoder"],
        aria: `Inversion only, ${q.config}: ${label}, ${q.mb_s} MB/s, live tables ${fmt(q.table_mb, 1)} MB computed`,
      })) };
      layers.push(L);
      legendItem(lg, L);
    }
    const p4 = s2.points.find((q) => q.config === "p4");
    scatter(fig, {
      layers, height: 220, xMax: 20,
      labels: p4 ? [{ x: p4.table_mb, y: p4.mb_s, text: "p4", side: "right" }] : [],
      xLabel: "live tables, MB (computed)", yLabel: "MB/s, inversion only",
      aria: `Inversion only, research harness, pinned, not the decoder: throughput against computed live tables, run ${runId(s2)}`,
    });
    source(fig, [s2], "Configurations of one plan (f, x and y are variants of the cooperative walk at 3×3 and 4×2) share a column. Its points are in the page's series 2 table.");
  }

  function draw(root, data) {
    root.textContent = "";
    root.classList.add("bh-root", "fr-root");
    const by = Object.fromEntries((data.series || []).map((s) => [s.id, s]));
    const s1 = by[DECODER_RUN], s2 = by[INVERSION], s3 = by[COOP];
    if (s3) rssChart(root, s3);
    if (s1 || s3) tableChart(root, s1, s3);
    if (s2) inversionPanel(root, s2);
    if (!s1 && !s2 && !s3) el("p", { text: "No measured series yet; the tables below give every value." }, root);
  }

  // Redrawn on resize, each chart sized to its box
  let drawn = null, resizeT, lastW = 0;
  window.addEventListener("resize", () => {
    clearTimeout(resizeT);
    resizeT = setTimeout(() => {
      if (!drawn || !drawn[0].isConnected) return;
      const w = drawn[0].clientWidth;
      if (w !== lastW) { lastW = w; draw(...drawn); }
    }, 150);
  });

  function init() {
    const root = document.getElementById("frontier-charts");
    if (!root) return;
    fetch(root.dataset.src)
      .then((r) => { if (!r.ok) throw new Error(r.status); return r.json(); })
      .then((d) => { draw(root, d); drawn = [root, d]; lastW = root.clientWidth; })
      .catch((e) => {
        const p = el("p", { class: "fr-note", text: `The charts could not be drawn (${e.message}); the tables below give every value.` });
        root.textContent = "";
        root.appendChild(p);
      });
  }
  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(init);
  else if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
