// "Why 32 couriers beat one" (docs/site/concepts/couriers.md): two pieces, in plain SVG.
//
// - #courier-schematic: a schematic of one dispatcher and N couriers, redrawn as the
//   controls change. Its timings are made up for the picture (a step's paperwork takes 1
//   or 2 ticks, an address 31 ticks to come back): it shows the shape of the idea, and it
//   measures nothing. It prints no figure.
// - #lane-sweeps: the measured walk timings by lane count, from data-src
//   (assets/data/lane-sweeps.json): one chart a panel, each on its own axes, each naming
//   its source. A new sweep is a new panel in that file.
//
// Runs on load and on each instant navigation, as bench.js does.
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
  const reduced = () => window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  // ---- The schematic ----------------------------------------------------------------

  const WAIT = 31; // ticks for an address to come back (made up: the picture's scale)
  const TICKS = 160; // the window drawn
  const PAPER = { more: 2, less: 1 }; // ticks of the dispatcher's paperwork a step

  // One dispatcher serves the couriers whose address has come back, in turn (first come,
  // first served); each step takes it `paper` ticks, then that courier waits WAIT ticks
  // for its next address. Returns each courier's intervals, and the dispatcher's.
  function simulate(n, paper) {
    const lanes = Array.from({ length: n }, () => []);
    const desk = [];
    let queue = Array.from({ length: n }, (_, i) => ({ lane: i, ready: 0 }));
    let t = 0;
    while (t < TICKS) {
      queue.sort((a, b) => a.ready - b.ready || a.lane - b.lane);
      const next = queue.shift();
      const start = Math.max(t, next.ready);
      if (start >= TICKS) break;
      if (start > next.ready) lanes[next.lane].push(["queue", next.ready, start]);
      lanes[next.lane].push(["paper", start, start + paper]);
      lanes[next.lane].push(["wait", start + paper, start + paper + WAIT]);
      desk.push([start, start + paper]);
      queue.push({ lane: next.lane, ready: start + paper + WAIT });
      t = start + paper;
    }
    // a courier still queued at the window's end
    for (const q of queue) if (q.ready < TICKS) lanes[q.lane].push(["queue", q.ready, TICKS]);
    const busy = desk.reduce((s, [a, b]) => s + Math.min(b, TICKS) - a, 0) / TICKS;
    // (every courier but the first queues at the start, as the dispatcher sends them off
    // one by one: only a wait for the dispatcher after an address came back counts)
    const queued = lanes.some((l) => l.some(([k, a]) => k === "queue" && a > 0));
    return { lanes, desk, busy, queued };
  }

  function schematic(host) {
    host.textContent = "";
    host.classList.add("cr-root");
    let n = 4, paper = "more", raf = null;
    const controls = el("div", { class: "cr-controls" }, host);
    const lab = el("label", { class: "cr-slider" }, controls);
    el("span", { text: "Couriers: " }, lab);
    const out = el("output", { text: String(n) }, lab);
    const range = el("input", { type: "range", min: 1, max: 64, step: 1, value: n, "aria-label": "Number of couriers" }, lab);
    const seg = el("div", { class: "cr-seg", role: "radiogroup", "aria-label": "Paperwork a step" }, controls);
    el("span", { text: "Paperwork a step: " }, seg);
    const btns = {};
    for (const [k, text] of [["more", "more"], ["less", "less"]]) {
      btns[k] = el("button", { type: "button", role: "radio", "aria-checked": String(k === paper), text }, seg);
      btns[k].addEventListener("click", () => { paper = k; draw(); });
    }
    const replay = el("button", { type: "button", class: "cr-replay", text: "Replay" }, controls);
    const body = el("div", { class: "cr-chart" }, host);
    const legend = el("ul", { class: "cr-legend" }, host);
    for (const [cls, text] of [
      ["cr-paper", "the dispatcher doing this courier's paperwork"],
      ["cr-wait", "waiting for the next address"],
      ["cr-queue", "address back, waiting for the dispatcher"],
    ]) {
      const li = el("li", {}, legend);
      el("span", { class: `cr-swatch ${cls}` }, li);
      li.append(text);
    }
    const status = el("p", { class: "cr-status", "aria-live": "polite" }, host);

    function draw() {
      if (raf) cancelAnimationFrame(raf);
      out.textContent = String(n);
      for (const [k, b] of Object.entries(btns)) b.setAttribute("aria-checked", String(k === paper));
      const sim = simulate(n, PAPER[paper]);
      body.textContent = "";
      const W = Math.max(300, Math.round(body.clientWidth) || 640);
      const L = 78, R = 8, T = 6;
      const rowH = Math.max(2, Math.min(14, Math.floor(220 / n)));
      const gap = rowH >= 6 ? 2 : rowH >= 3 ? 1 : 0;
      const lanesH = n * rowH;
      const deskY = T + lanesH + 12, deskH = 16;
      const H = deskY + deskH + 26;
      const x = (t) => L + ((W - L - R) * t) / TICKS;
      const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, role: "img", "aria-label": `Schematic: ${n} courier${n === 1 ? "" : "s"} and one dispatcher, ${paper} paperwork a step` }, body);
      const clipId = `cr-clip-${Math.random().toString(36).slice(2)}`;
      const clip = svg("clipPath", { id: clipId }, svg("defs", {}, s));
      const reveal = svg("rect", { x: 0, y: 0, width: W, height: H }, clip);
      const g = svg("g", { "clip-path": `url(#${clipId})` }, s);
      sim.lanes.forEach((ivs, i) => {
        const y = T + i * rowH;
        for (const [k, a, b] of ivs) {
          const b2 = Math.min(b, TICKS);
          if (b2 <= a) continue;
          svg("rect", { x: x(a), y, width: Math.max(0.6, x(b2) - x(a) - (k === "paper" ? 0 : 0.5)), height: rowH - gap, class: `cr-${k}` }, g);
        }
      });
      const lbl = (text, y) => { svg("text", { x: L - 8, y, "text-anchor": "end", class: "cr-tick" }, s).textContent = text; };
      if (n <= 8) sim.lanes.forEach((_, i) => lbl(`courier ${i + 1}`, T + i * rowH + rowH / 2 + 4));
      else { lbl("courier 1", T + rowH / 2 + 4); lbl(`courier ${n}`, T + lanesH - rowH / 2 + 4); }
      svg("rect", { x: L, y: deskY, width: W - L - R, height: deskH, class: "cr-idle" }, s);
      for (const [a, b] of sim.desk) svg("rect", { x: x(a), y: deskY, width: Math.max(0.6, x(Math.min(b, TICKS)) - x(a)), height: deskH, class: "cr-paper" }, g);
      lbl("dispatcher", deskY + deskH / 2 + 4);
      if (W >= 480) svg("text", { x: W - R, y: H - 6, "text-anchor": "end", class: "cr-tick" }, s).textContent = "time →";
      svg("text", { x: L, y: H - 6, class: "cr-tick" }, s).textContent = "schematic: made-up timings, not to scale";
      status.textContent =
        sim.busy < 0.5 ? "The dispatcher is idle most of the time: every courier is waiting for an address."
        : !sim.queued && sim.busy < 0.97 ? "The dispatcher is busy part of the time, and no courier waits for it."
        : !sim.queued ? "Just enough couriers: the dispatcher is never idle, and no courier waits for it."
        : sim.busy < 0.97 ? "The dispatcher is busy most of the time, and couriers begin to wait for it."
        : "The dispatcher is never idle: couriers queue for it, so more couriers deliver no more. Less paperwork a step is what helps now.";
      if (reduced()) return;
      const t0 = performance.now(), dur = 2600;
      const step = (now) => {
        const f = Math.min(1, (now - t0) / dur);
        reveal.setAttribute("width", String(L + (W - L) * f));
        if (f < 1) raf = requestAnimationFrame(step);
      };
      reveal.setAttribute("width", String(L));
      raf = requestAnimationFrame(step);
    }
    range.addEventListener("input", () => { n = +range.value; draw(); });
    replay.addEventListener("click", draw);
    draw();
    return draw;
  }

  // ---- The measured lane sweeps -------------------------------------------------------

  const SLOTS = 4; // bench.css's --bh-s1..s4
  const slot = (i) => `var(--bh-s${(i % SLOTS) + 1})`;
  const ticks = (lo, hi, n = 4) => {
    const step0 = (hi - lo) / n || 1;
    const mag = 10 ** Math.floor(Math.log10(step0));
    const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((v) => v >= step0);
    const out = [];
    for (let v = Math.floor(lo / step) * step; v <= hi + step * 1e-9; v += step) out.push(+v.toFixed(10));
    while (out[out.length - 1] < hi) out.push(+(out[out.length - 1] + step).toFixed(10));
    return out;
  };

  function panel(p, root) {
    const fig = el("figure", { class: "cr-panel" }, root);
    el("figcaption", { text: p.title }, fig);
    if (p.what) el("p", { class: "cr-note", text: p.what }, fig);
    const series = p.series.filter((s) => s.points && s.points.length);
    const pts = series.flatMap((s) => s.points);
    if (!pts.length) { el("p", { text: "No measured points yet." }, fig); return; }
    const box = el("div", { class: "cr-chart" }, fig);
    const W = Math.max(300, Math.round(box.clientWidth) || 640), H = 240;
    const M = { l: 48, r: series.length > 1 ? 92 : 16, t: 26, b: 38 };
    const xs = [...new Set(pts.map((q) => q.x))].sort((a, b) => a - b);
    const x0 = xs[0], x1 = xs[xs.length - 1], pad = 14;
    const hi = Math.max(...pts.map((q) => (q.hi !== undefined ? q.hi : q.y)));
    const ty = ticks(0, hi * 1.08);
    const y1 = ty[ty.length - 1];
    const x = (v) => (x1 === x0 ? (M.l + W - M.r) / 2 : M.l + pad + ((W - M.l - M.r - 2 * pad) * (v - x0)) / (x1 - x0));
    const y = (v) => M.t + (H - M.t - M.b) * (1 - v / (y1 || 1));
    const unit = (p.y && p.y.unit) || "";
    const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, role: "img", "aria-label": `${p.title}: ${p.y.label} by ${p.x.label}, measured (${p.source.label})` }, box);
    for (const v of ty) {
      svg("line", { x1: M.l, x2: W - M.r, y1: y(v), y2: y(v), class: "bh-grid" }, s);
      svg("text", { x: M.l - 6, y: y(v) + 4, "text-anchor": "end", class: "bh-tick" }, s).textContent = String(v);
    }
    for (const v of xs) svg("text", { x: x(v), y: H - M.b + 16, "text-anchor": "middle", class: "bh-tick" }, s).textContent = String(v);
    svg("text", { x: (M.l + W - M.r) / 2, y: H - 4, "text-anchor": "middle", class: "bh-tick" }, s).textContent = p.x.label;
    svg("text", { x: 4, y: 11, class: "bh-tick" }, s).textContent = `${unit} (lower is faster)`;
    series.forEach((ser, i) => {
      const sp = ser.points.slice().sort((a, b) => a.x - b.x);
      if (sp.length > 1) svg("polyline", { points: sp.map((q) => `${x(q.x)},${y(q.y)}`).join(" "), class: "bh-line", stroke: slot(i) }, s);
      for (const q of sp) {
        const g = svg("g", { tabindex: 0, class: "cr-pt" }, s);
        if (q.lo !== undefined && q.hi !== undefined) svg("line", { x1: x(q.x), x2: x(q.x), y1: y(q.lo), y2: y(q.hi), class: "bh-whisker", stroke: slot(i) }, g);
        svg("circle", { cx: x(q.x), cy: y(q.y), r: 4.5, class: "bh-dot", fill: slot(i) }, g);
        svg("circle", { cx: x(q.x), cy: y(q.y), r: 12, class: "bh-hit" }, g);
        svg("title", {}, g).textContent = `${ser.label}: ${q.x} ${p.x.label}, ${q.y} ${unit}` +
          (q.lo !== undefined ? ` (${q.lo}–${q.hi})` : "") + (q.note ? `\n${q.note}` : "") + `\nmeasured: ${p.source.label}`;
      }
      if (series.length > 1) {
        const last = sp[sp.length - 1];
        svg("text", { x: x(last.x) + 8, y: y(last.y) + 4, class: "bh-tick cr-direct" }, s).textContent = ser.label;
      }
    });
    if (series.length > 1) {
      const lg = el("ul", { class: "bh-legend" }, fig);
      series.forEach((ser, i) => {
        const li = el("li", {}, lg);
        el("span", { class: "bh-swatch", style: `background:${slot(i)}` }, li);
        li.append(ser.label);
      });
    }
    const src = el("p", { class: "cr-source" }, fig);
    src.append("Measured. ");
    if (p.measured) src.append(`${p.measured} `);
    src.append("Source: ");
    el("a", { href: p.source.url, text: p.source.label }, src);
    src.append(".");
    // the same points as a table
    const det = el("details", { class: "cr-table" }, fig);
    el("summary", { text: "The points as a table" }, det);
    const tbl = el("table", {}, det);
    const hr = el("tr", {}, el("thead", {}, tbl));
    el("th", { text: p.x.label }, hr);
    for (const ser of series) el("th", { text: `${ser.label} (${unit})` }, hr);
    const tb = el("tbody", {}, tbl);
    for (const v of xs) {
      const tr = el("tr", {}, tb);
      el("td", { text: String(v) }, tr);
      for (const ser of series) {
        const q = ser.points.find((r) => r.x === v);
        el("td", { text: q ? String(q.y) : "–" }, tr);
      }
    }
  }

  function sweeps(root, data) {
    root.textContent = "";
    root.classList.add("bh-root", "cr-root");
    for (const p of data.panels || []) panel(p, root);
    if (!(data.panels || []).length) el("p", { text: "No measured sweeps yet." }, root);
  }

  // Redrawn on resize, each chart sized to its box
  const drawn = { schematic: null, sweeps: null };
  let resizeT;
  window.addEventListener("resize", () => {
    clearTimeout(resizeT);
    resizeT = setTimeout(() => {
      if (drawn.schematic && drawn.schematic[0].isConnected) drawn.schematic[1]();
      if (drawn.sweeps && drawn.sweeps[0].isConnected) sweeps(...drawn.sweeps);
    }, 150);
  });

  function init() {
    const sc = document.getElementById("courier-schematic");
    if (sc) drawn.schematic = [sc, schematic(sc)];
    const sw = document.getElementById("lane-sweeps");
    if (sw) {
      fetch(sw.dataset.src)
        .then((r) => { if (!r.ok) throw new Error(r.status); return r.json(); })
        .then((d) => { sweeps(sw, d); drawn.sweeps = [sw, d]; })
        .catch((e) => { sw.textContent = `The measured sweeps could not be loaded (${e.message}).`; });
    }
  }
  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(init);
  else if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
