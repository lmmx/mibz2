// The benchmark history (docs/site/benchmarks/history.md): a small chart per corpus and
// thread count of master's mibz2 figures over its commits, from
// assets/bench-history.json (docs/scripts/bench_history.py), in plain SVG. Each class of
// results (the bench major that last set these figures' meaning, a runner CPU and core
// count, a set of corpora) is a line of its own, never joined to another, and so is each
// content of a corpus (its sha256). Runs on any page with a #bench-history
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
  let state = { metric: "mb_s", threads: "all", cpu: null, tCorpus: "wikidata", tThreads: "4", tFig: "wall", tView: "ci", wsRun: null, wsCorpus: "wikidata" };
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
  // a class's colour: its place among the classes on screen (render's `shown`), so one
  // runner's classes take the first slots; past the slots, muted
  let colourOf = new Map();
  const slot = (cls) => {
    const k = colourOf.has(cls) ? colourOf.get(cls) : cls;
    return k < SLOTS ? `var(--bh-s${k + 1})` : "var(--bh-old)";
  };

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
    const W = 640, H = 222, L = 56, R = 14, T = 24, B = 34;
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
    svg("text", { x: 4, y: 11, class: "bh-unit" }, s).textContent = unit;
    const every = Math.max(1, Math.ceil(cs.length / 8));
    cs.forEach((c, i) => {
      if (i % every && i !== cs.length - 1) return;
      const anchor = cs.length > 1 && i === 0 ? "start" : cs.length > 1 && i === cs.length - 1 ? "end" : "middle";
      svg("text", { x: x(c), y: H - B + 16, class: "bh-tick", "text-anchor": anchor }, s).textContent = data.commits[c].sha.slice(0, 7);
    });
    for (const line of series.lines) {
      const lp = line.points.filter((p) => p[key]);
      // joined only between neighbouring commits: a commit of another class (or one with no
      // point) between two of this line's breaks it, so no segment spans commits it didn't
      // measure
      const runs = [];
      for (const p of lp) {
        const last = runs[runs.length - 1];
        if (last && xi.get(p.c) === xi.get(last[last.length - 1].c) + 1) last.push(p);
        else runs.push([p]);
      }
      for (const run of runs.filter((r) => r.length > 1)) {
        svg("polyline", {
          points: run.map((p) => `${x(p.c)},${y(p[key][0])}`).join(" "),
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
          (line.content ? `corpus ${series.corpus} sha256 ${line.content}\n` : "") +
          `${metric[1]}: ${fmt(m)} ± ${fmt(h)} ${unit} (95% interval, ${p.n} rounds)` + (p.same_bytes ? "" : "\nSAME BYTES: NO");
      }
    }
  }

  // The trajectory (#bench-hero, first on the page): each merged pull request's own paired
  // benchmark (its head against its base, on one runner, in alternating order: bench/links.py)
  // chained into one cumulative change, from the first link whose arms ran in balanced order
  // (bench 3.0.0). A link that left the decoder alone (the harness, docs or CI) is an A/A
  // pair: drawn, adding nothing. A decoder change with no link (a gap) starts a new segment,
  // never bridged. Below, each link's own change and its 95% interval. The figures are
  // ratios, so links measured on different runners chain only on the assumption that a
  // relative change holds across runners: the page says so.
  const TRAJ = [
    // [link figure, label, sign: +1 when a lower ratio is better and shown as an increase]
    ["wall", "Throughput", -1],
    ["cpu", "CPU time", 1],
    ["rss", "Peak RSS", 1],
  ];
  const pct = (L, sign) => 100 * (Math.exp(sign * L) - 1);
  const fmtPct = (v) => `${v >= 0 ? "+" : "−"}${Math.abs(v).toFixed(Math.abs(v) < 10 ? 1 : 0)}%`;
  function trajectory(data, root) {
    const host = document.getElementById("bench-hero") || el("div", {}, root);
    host.textContent = "";
    host.classList.add("bh-root");
    const box = el("section", { class: "bh-hero" }, host);
    const ch = data.chain || {};
    const start = ch.start;
    const head = el("div", { class: "bh-hero-head" }, box);
    const ws = (data.scaling && data.scaling.runs) || [];
    if (!ws.length) state.tView = "ci";
    el("h2", { class: "bh-hero-title", id: "trajectory", text: state.tView === "ws" ? "Release scaling" : "Merge by merge, chained" }, head);
    const seg = (label, options, value, set) => {
      const g = el("div", { class: "bh-seg", role: "radiogroup", "aria-label": label }, head);
      for (const [v, t] of options) {
        const b = el("button", { type: "button", role: "radio", "aria-checked": String(v === value), text: t, "data-hero-id": `${label}-${v}` }, g);
        b.addEventListener("click", () => {
          set(v);
          try { localStorage.setItem("mibz2-bench", JSON.stringify(state)); } catch (e) { /* (none) */ }
          trajectory(data, root);
          host.querySelector(`[data-hero-id="${label}-${v}"]`).focus();
        });
      }
    };
    // A release-scaling run (bench/release-scaling/, docs/scripts/bench_release_scaling.py): offered only
    // when one is recorded, so the page needs none
    if (ws.length) seg("View", [["ci", "CI, merge by merge"], ["ws", "Release scaling"]], state.tView, (v) => (state.tView = v));
    if (state.tView === "ws") { releaseScaling(ws, box, seg); return; }
    if (start === null || start === undefined || !data.links || !data.links.length) {
      el("p", { class: "bh-hero-note", text: "No merged pull request's benchmark is linked yet (bench/links.py)." }, box);
      return;
    }
    const keys = Object.keys(ch.series);
    const corpora = [...new Set(keys.map((k) => k.split("|")[0]))].sort((a, b) => (a !== "wikidata") - (b !== "wikidata") || a.localeCompare(b));
    if (!corpora.includes(state.tCorpus)) state.tCorpus = corpora.includes("wikidata") ? "wikidata" : corpora[0];
    const threads = [...new Set(keys.filter((k) => k.startsWith(`${state.tCorpus}|`)).map((k) => k.split("|")[1]))].sort((a, b) => a - b);
    if (!threads.includes(state.tThreads)) state.tThreads = threads.includes("4") ? "4" : threads[threads.length - 1];
    if (!TRAJ.some((f) => f[0] === state.tFig)) state.tFig = "wall";
    seg("Corpus", corpora.map((c) => [c, c]), state.tCorpus, (v) => (state.tCorpus = v));
    seg("Threads", threads.map((t) => [t, `${t} thread${t === "1" ? "" : "s"}`]), state.tThreads, (v) => (state.tThreads = v));
    seg("Figure", TRAJ.map((f) => [f[0], f[1]]), state.tFig, (v) => (state.tFig = v));
    const fig = TRAJ.find((f) => f[0] === state.tFig);
    const sign = fig[2];
    const key = `${state.tCorpus}|${state.tThreads}`;
    const chain = ch.series[key][fig[0]];
    const own = (data.changes[key] || {})[fig[0]] || [];
    const idx = [];
    for (let i = start; i < data.links.length; i++) idx.push(i);
    const pts = idx.filter((i) => chain[i]);
    const lead = el("p", { class: "bh-hero-focus" }, box);
    const body = el("div", { class: "bh-hero-chart" }, box);
    const readout = el("p", { class: "bh-hero-commit", "aria-live": "polite" }, box);
    if (!pts.length) { el("p", { text: "No link measured this figure." }, body); return; }
    const tag = (l) => (l.pr ? `#${l.pr}` : l.sha.slice(0, 7));
    const decoderLinks = pts.filter((i) => data.links[i].decoder);
    const last = pts[pts.length - 1];
    const [L, S, sg] = chain[last];
    const segStart = pts.find((i) => chain[i][2] === sg);
    const cpus = [...new Set(decoderLinks.map((i) => data.links[i].cpu))];
    const better = sign < 0 ? "faster" : "less";
    const lv = pct(L, sign), lo = pct(L - S, sign), hi = pct(L + S, sign);
    const strong = el("strong", { text: `${fig[1]}, ${state.tCorpus} at ${state.tThreads} thread${state.tThreads === "1" ? "" : "s"}: ${fmtPct(lv)}` }, lead);
    strong.title = "chained change since the segment's start";
    lead.append(` (95%: ${fmtPct(Math.min(lo, hi))} to ${fmtPct(Math.max(lo, hi))}) from ${tag(data.links[segStart])}'s base to ${tag(data.links[last])}, over ${decoderLinks.filter((i) => chain[i][2] === sg).length} decoder merges, each measured on its own runner (${cpus.join(", ")}). ${sign < 0 ? "Up is faster." : `Down is ${better}.`}`);

    const W = Math.max(300, Math.round(body.clientWidth) || 700);
    const narrow = W < 560;
    const H1 = narrow ? 230 : 280, H2 = narrow ? 130 : 150, gapY = 38;
    const H = H1 + gapY + H2;
    const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, role: "img", "aria-label": `${fig[1]} of ${state.tCorpus} at ${state.tThreads} threads: the change chained over ${pts.length} merges, and each merge's own change` }, body);
    // y scales: the chain (top) and each link's own change (bottom), in percent
    const band = pts.map((i) => [pct(chain[i][0] - chain[i][1], sign), pct(chain[i][0] + chain[i][1], sign)]).flat();
    const t1 = ticks(Math.min(0, ...band), Math.max(0, ...band), narrow ? 4 : 5);
    const ownV = idx.filter((i) => own[i]).map((i) => [pct(own[i][0] - own[i][1], sign), pct(own[i][0] + own[i][1], sign)]).flat();
    const t2 = ticks(Math.min(0, ...ownV), Math.max(0, ...ownV), 3);
    const meas = svg("text", { class: "bh-tick", x: 0, y: -50 }, s);
    const textW = (t) => { meas.textContent = t; return meas.getComputedTextLength(); };
    const tw = Math.max(24, ...[...t1, ...t2].map((v) => textW(fmtPct(v))));
    meas.remove();
    const M = { l: Math.ceil(tw) + 12, r: 12, t: 14, b: 26 };
    const inner = W - M.l - M.r;
    const xs = new Map(idx.map((i, k) => [i, M.l + (idx.length === 1 ? inner / 2 : 8 + (k * (inner - 16)) / (idx.length - 1))]));
    const y1 = (v) => M.t + (H1 - M.t - 8) * (1 - (v - t1[0]) / (t1[t1.length - 1] - t1[0] || 1));
    const top2 = H1 + gapY;
    const y2 = (v) => top2 + (H2 - M.b) * (1 - (v - t2[0]) / (t2[t2.length - 1] - t2[0] || 1));
    for (const v of t1) {
      svg("line", { x1: M.l, x2: W - M.r, y1: y1(v), y2: y1(v), class: v === 0 ? "bh-zero" : "bh-grid" }, s);
      svg("text", { x: M.l - 6, y: y1(v) + 4, class: "bh-tick", "text-anchor": "end" }, s).textContent = fmtPct(v);
    }
    for (const v of t2) {
      svg("line", { x1: M.l, x2: W - M.r, y1: y2(v), y2: y2(v), class: v === 0 ? "bh-zero" : "bh-grid" }, s);
      svg("text", { x: M.l - 6, y: y2(v) + 4, class: "bh-tick", "text-anchor": "end" }, s).textContent = fmtPct(v);
    }
    svg("text", { x: M.l, y: top2 - 14, class: "bh-unit" }, s).textContent = narrow ? "each merge's own change (95%)" : "each merge's own change, with its 95% interval (hollow: the decoder unchanged, an A/A pair)";
    // segments of the chain: a band and a line each, never bridged across a gap
    const segs = [];
    for (const i of pts) {
      const g = chain[i][2];
      if (!segs.length || segs[segs.length - 1].g !== g) segs.push({ g, pts: [] });
      segs[segs.length - 1].pts.push(i);
    }
    for (const sgm of segs) {
      const p = sgm.pts;
      const upper = p.map((i) => `${xs.get(i)},${y1(Math.max(pct(chain[i][0] - chain[i][1], sign), pct(chain[i][0] + chain[i][1], sign)))}`);
      const lower = p.map((i) => `${xs.get(i)},${y1(Math.min(pct(chain[i][0] - chain[i][1], sign), pct(chain[i][0] + chain[i][1], sign)))}`).reverse();
      svg("polygon", { points: [...upper, ...lower].join(" "), class: "bh-band" }, s);
      svg("polyline", { points: p.map((i) => `${xs.get(i)},${y1(pct(chain[i][0], sign))}`).join(" "), class: "bh-chain" }, s);
      if (sgm !== segs[0]) {
        const xg = xs.get(p[0]) - 6;
        svg("line", { x1: xg, x2: xg, y1: M.t, y2: H - M.b, class: "bh-gap" }, s);
        // the label on the side of the line with room for it, so it never leaves the plot
        const right = xg > M.l + inner / 2;
        const gt = svg("text", { x: right ? xg - 3 : xg + 3, y: M.t + 10, class: "bh-tick", "text-anchor": right ? "end" : "start" }, s);
        gt.textContent = narrow ? "gap" : "gap: a decoder change with no link";
        svg("title", {}, gt).textContent = "a decoder change with no link: the chain starts again here";
      }
    }
    // x labels: as many as fit with a gap between them, the last always shown
    const minGap = narrow ? 40 : 46;
    const drawn = [];
    idx.forEach((i, k) => {
      const xx = xs.get(i);
      const last = k === idx.length - 1;
      if (!last && drawn.length && xx - drawn[drawn.length - 1].x < minGap) return;
      if (last) while (drawn.length && xx - drawn[drawn.length - 1].x < minGap) drawn.pop().el.remove();
      const anchor = idx.length > 1 && k === 0 ? "start" : last && idx.length > 1 ? "end" : "middle";
      const t = svg("text", { x: xx, y: H - 8, class: "bh-tick", "text-anchor": anchor }, s);
      t.textContent = tag(data.links[i]);
      drawn.push({ x: xx, el: t });
    });
    const show = (i) => {
      const l = data.links[i];
      readout.textContent = "";
      el("strong", { text: `${tag(l)} ` }, readout);
      readout.append(`${l.subject.replace(/\s*\(#\d+\)\s*$/, "")} · ${l.date.slice(0, 10)} · ${l.cpu}, bench ${l.bench}`);
      const o = own[i];
      if (o) {
        const a = pct(o[0] - o[1], sign), b = pct(o[0] + o[1], sign);
        readout.append(` · own change ${fmtPct(pct(o[0], sign))} (95%: ${fmtPct(Math.min(a, b))} to ${fmtPct(Math.max(a, b))}, ${o[2]} rounds)`);
      } else readout.append(" · not measured");
      if (!l.decoder) readout.append(" · the decoder unchanged: an A/A pair, adding nothing");
      if (chain[i]) readout.append(` · chained ${fmtPct(pct(chain[i][0], sign))}`);
      if (l.base_ok === false || l.head_ok === false) readout.append(" · its run's base or head differs from the merge's parent or the merge on the benchmarked paths");
      readout.append(" · ");
      el("a", { href: l.run, text: "run" }, readout);
      readout.append(" · ");
      el("a", { href: `${data.repo}/pull/${l.pr}`, text: "pull request" }, readout);
    };
    for (const i of idx) {
      const l = data.links[i];
      const g = svg("g", { tabindex: 0, class: "bh-pt", role: "button", "aria-label": `${tag(l)}` }, s);
      if (chain[i]) svg("circle", { cx: xs.get(i), cy: y1(pct(chain[i][0], sign)), r: l.decoder ? 3.5 : 2.5, class: l.decoder ? "bh-dot-chain" : "bh-dot-aa" }, g);
      const o = own[i];
      if (o) {
        const a = pct(o[0] - o[1], sign), b = pct(o[0] + o[1], sign);
        const excl = Math.min(a, b) > 0 || Math.max(a, b) < 0;
        svg("line", { x1: xs.get(i), x2: xs.get(i), y1: y2(Math.min(a, b)), y2: y2(Math.max(a, b)), class: "bh-whisker", stroke: "var(--bh-ink-2)" }, g);
        svg("circle", { cx: xs.get(i), cy: y2(pct(o[0], sign)), r: 3.5, class: l.decoder ? (excl ? "bh-dot-own bh-dot-excl" : "bh-dot-own") : "bh-dot-aa" }, g);
      }
      svg("rect", { x: xs.get(i) - Math.max(4, inner / idx.length / 2), y: M.t, width: Math.max(8, inner / idx.length), height: H - M.t - M.b, class: "bh-hit" }, g);
      svg("title", {}, g).textContent = `${tag(l)} ${l.subject}`;
      for (const ev of ["pointerenter", "focus", "click"]) g.addEventListener(ev, () => show(i));
    }
    show(last);
    el("p", { class: "bh-hero-note", text: "Top: the change chained since the start, with its 95% band. Below: each merge's own change, its head against its base on one runner. Point at, tap or Tab to a merge for its figures. What the chain assumes is under the chart." }, box);
  }

  function table(data, metric, root, shown = () => true) {
    const [key, label, unit, fmt] = metric;
    const d = el("details", { class: "bh-table" }, root);
    el("summary", { text: `Every point as a table (${label}, ${unit})` }, d);
    const t = el("table", {}, el("div", { class: "bh-scroll" }, d));
    const hr = el("tr", {}, el("thead", {}, t));
    for (const h of ["corpus", "threads", "commit", "date", "class", `${label} (${unit})`, "± 95%", "rounds"]) el("th", { text: h }, hr);
    const tb = el("tbody", {}, t);
    for (const s of data.series) {
      if (state.threads !== "all" && String(s.threads) !== state.threads) continue;
      for (const l of s.lines) for (const p of l.points) {
        if (!p[key] || !shown(l.class)) continue;
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

  // Every link as a table: each merged pull request's own paired change, before the chain's
  // start too (bench 1-2: the head ran first in every round, so not chained)
  function links(data, root) {
    if (!data.links || !data.links.length) return;
    const d = el("details", { class: "bh-table" }, root);
    el("summary", { text: `Every merge's own benchmark (${data.links.length} linked pull requests)` }, d);
    const key = `${state.tCorpus}|${state.tThreads}`;
    const fig = TRAJ.find((f) => f[0] === state.tFig) || TRAJ[0];
    const own = (data.changes[key] || {})[fig[0]] || [];
    const chain = ((data.chain.series || {})[key] || {})[fig[0]] || [];
    const t = el("table", {}, el("div", { class: "bh-scroll" }, d));
    const hr = el("tr", {}, el("thead", {}, t));
    for (const h of ["merge", "pull request", "runner", "bench", "decoder", `${fig[1]}, ${key.replace("|", " at ")} threads (95%)`, "chained"]) el("th", { text: h }, hr);
    const tb = el("tbody", {}, t);
    data.links.forEach((l, i) => {
      const tr = el("tr", {}, tb);
      el("a", { href: `${data.repo}/commit/${l.sha}`, text: l.sha.slice(0, 7) }, el("td", {}, tr)).title = l.subject;
      el("a", { href: `${data.repo}/pull/${l.pr}`, text: `#${l.pr}` }, el("td", {}, tr));
      el("td", { text: l.cpu }, tr);
      el("td", { text: `${l.bench}${l.balanced ? "" : " (head first)"}` }, tr);
      el("td", { text: l.decoder ? "changed" : "unchanged (A/A)" }, tr);
      const o = own[i];
      const a = o && pct(o[0] - o[1], fig[2]), b = o && pct(o[0] + o[1], fig[2]);
      el("td", { text: o ? `${fmtPct(pct(o[0], fig[2]))} (${fmtPct(Math.min(a, b))}, ${fmtPct(Math.max(a, b))})` : "–" }, tr);
      el("td", { text: chain[i] ? fmtPct(pct(chain[i][0], fig[2])) : "–" }, tr);
    });
    if (data.gaps && data.gaps.length) {
      el("p", { text: `Decoder changes with no link (the chain breaks at each): ${data.gaps.map((g) => (g.pr ? `#${g.pr}` : g.sha.slice(0, 7))).join(", ")}.` }, d);
    }
  }

  // The hero's Release scaling view: one recorded run, each arm's throughput (the median of its
  // rounds) against threads, and its paired wall change against the reference arm. A run is
  // one machine's: runs are never combined, and they inform rather than decide.
  function releaseScaling(ws, box, seg) {
    // A newly recorded run (ws[0], the newest) is shown first, whatever run a reader last
    // picked: a remembered choice only holds until a newer release's run arrives.
    if (state.wsNewest !== ws[0].id) {
      state.wsNewest = ws[0].id;
      state.wsRun = ws[0].id;
    }
    if (!ws.some((r) => r.id === state.wsRun)) state.wsRun = ws[0].id;
    const run = ws.find((r) => r.id === state.wsRun);
    // Each run is named by its releases, so runs on one day and machine can be told apart
    const name = (r) => {
      const older = r.arms.filter((a) => a !== r.ref && /^\d+\.\d+\.\d+$/.test(a));
      return `${r.ref}${older.length ? " against " + older.join(", ") : ""} · ${r.date} · ${r.cpu}`;
    };
    if (ws.length > 1) seg("Run", ws.map((r) => [r.id, name(r)]), state.wsRun, (v) => (state.wsRun = v));
    const corpora = Object.keys(run.corpora).sort((a, b) => (a !== "wikidata") - (b !== "wikidata") || a.localeCompare(b));
    if (!corpora.includes(state.wsCorpus)) state.wsCorpus = corpora[0];
    if (corpora.length > 1) seg("Corpus", corpora.map((c) => [c, c]), state.wsCorpus, (v) => (state.wsCorpus = v));
    const c = run.corpora[state.wsCorpus];
    const ts = c.threads.map(String);
    const top = ts[ts.length - 1];
    const lead = el("p", { class: "bh-hero-focus" }, box);
    const body = el("div", { class: "bh-hero-chart" }, box);
    const legend = el("p", { class: "bh-hero-commit" }, box);
    const note = el("p", { class: "bh-hero-commit" }, box);
    const mb = (a, t) => (c.mbps[a] || {})[t];
    el("strong", { text: `${state.wsCorpus} on ${run.cpu} at ${top} threads: ${run.ref} ${(mb(run.ref, top) || 0).toFixed(0)} MB/s` }, lead);
    const others = run.arms.filter((a) => a !== run.ref && (c.change[a] || {})[top]);
    if (others.length) {
      lead.append(`. Wall against ${run.ref}: ` + others.map((a) => {
        const [e, lo, hi] = c.change[a][top];
        return `${a} ${fmtPct(e)} (95%: ${fmtPct(lo)} to ${fmtPct(hi)})`;
      }).join("; ") + ". Above 0 is slower than " + run.ref + ".");
    }
    const W = Math.max(300, Math.round(body.clientWidth) || 700);
    const narrow = W < 560;
    const H = narrow ? 240 : 300, M = { l: 52, r: 14, t: 16, b: 36 };
    const vals = run.arms.flatMap((a) => ts.map((t) => mb(a, t)).filter((v) => v));
    const yt = ticks(0, Math.max(...vals), narrow ? 4 : 5);
    // the top tick at or above the highest point, so no line leaves the plot
    while (yt[yt.length - 1] < Math.max(...vals)) yt.push(+(yt[yt.length - 1] + (yt[1] - yt[0])).toFixed(10));
    const y = (v) => M.t + (H - M.t - M.b) * (1 - v / yt[yt.length - 1]);
    const x = (i) => M.l + (ts.length === 1 ? (W - M.l - M.r) / 2 : (i * (W - M.l - M.r)) / (ts.length - 1));
    const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, role: "img", "aria-label": `Throughput of ${state.wsCorpus} on ${run.cpu} against threads, for ${run.arms.join(", ")}` }, body);
    for (const v of yt) {
      svg("line", { x1: M.l, x2: W - M.r, y1: y(v), y2: y(v), class: "bh-grid" }, s);
      svg("text", { x: M.l - 6, y: y(v) + 4, class: "bh-tick", "text-anchor": "end" }, s).textContent = v.toLocaleString();
    }
    svg("text", { x: M.l, y: M.t - 4, class: "bh-unit" }, s).textContent = "MB/s of output (median of the rounds)";
    ts.forEach((t, i) => {
      svg("text", { x: x(i), y: H - M.b + 16, class: "bh-tick", "text-anchor": "middle" }, s).textContent = t;
    });
    svg("text", { x: (M.l + W - M.r) / 2, y: H - 4, class: "bh-tick", "text-anchor": "middle" }, s).textContent = "threads";
    run.arms.forEach((a, k) => {
      const col = k < SLOTS ? `var(--bh-s${k + 1})` : "var(--bh-old)";
      const pts = ts.map((t, i) => [x(i), mb(a, t)]).filter(([, v]) => v);
      svg("polyline", { points: pts.map(([px, v]) => `${px},${y(v)}`).join(" "), class: "bh-line", stroke: col, fill: "none" }, s);
      for (const [px, v] of pts) {
        const d = svg("circle", { cx: px, cy: y(v), r: 4, class: "bh-dot", fill: col }, s);
        svg("title", {}, d).textContent = `${a}: ${v.toFixed(0)} MB/s`;
      }
      const item = el("span", { class: "bh-ws-key" }, legend);
      el("span", { class: "bh-swatch", style: `background:${col}` }, item);
      item.append(` ${a}${a === run.ref ? " (reference)" : ""}  `);
    });
    const cores = run.cores ? `${run.cores} cores, ${run.cpus} CPUs` : `${run.cpus} CPUs`;
    note.textContent = `${run.date} · ${cores} · ${run.rounds} paired rounds · ${c.same_bytes ? "every arm gave the same bytes" : "the arms' outputs differ"}. A local run informs; it doesn't decide (the tables below).` + (run.note ? ` ${run.note}` : "");
  }

  function render(root, data) {
    root.textContent = "";
    root.classList.add("bh-root");
    if (!data.commits.length) {
      el("p", { text: "No master commit is recorded yet: the first record appears once the Benchmark workflow has run on a push to master." }, root);
      return;
    }
    trajectory(data, root);
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
    const meta = data.classes_meta || [];
    const recs = {};
    for (const s of data.series) for (const l of s.lines) for (const p of l.points) {
      const cpu = (meta[l.class] || {}).cpu;
      if (cpu) (recs[cpu] = recs[cpu] || new Set()).add(p.c);
    }
    const cpus = Object.keys(recs).sort((a, b) => recs[b].size - recs[a].size);
    if (state.cpu !== "all" && !cpus.includes(state.cpu)) state.cpu = data.default_cpu && cpus.includes(data.default_cpu) ? data.default_cpu : cpus[0] || "all";
    pick("Runner", [...cpus.map((c) => [c, `${c} (${recs[c].size} records)`]), ["all", "All runners (mixed: not comparable)"]], state.cpu, (v) => (state.cpu = v));
    const shown = (cls) => state.cpu === "all" || (meta[cls] || {}).cpu === state.cpu;
    colourOf = new Map(data.classes.map((_, i) => i).filter(shown).map((cls, k) => [cls, k]));
    const legend = el("ul", { class: "bh-legend" }, root);
    data.classes.forEach((c, i) => {
      if (!shown(i)) return;
      const li = el("li", {}, legend);
      el("span", { class: "bh-swatch", style: `background:${slot(i)}` }, li);
      li.append(c);
    });
    const metric = METRICS.find((m) => m[0] === state.metric) || METRICS[0];
    const grid = el("div", { class: "bh-grid-charts" }, root);
    for (const s of data.series) {
      if (state.threads !== "all" && String(s.threads) !== state.threads) continue;
      chart(data, { ...s, lines: s.lines.filter((l) => shown(l.class)) }, metric, grid);
    }
    table(data, metric, root, shown);
    links(data, root);
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
  // the second (bench.css: --bh-s2..s8). A single-threaded decoder is drawn hollow, its
  // whisker dashed, and listed in a legend group of its own, so it may share a
  // block-parallel decoder's hue: bz2ngrs takes lbzip2's (beside libbzip2's and
  // libbz2-rs's: no normal-vision pair below ΔE 22, colour-blind 6.5-6.9, the floor band
  // that the legend's groups and the tables carry). Any other decoder takes the muted slot.
  const CMP_ORDER = ["pbz2", "lbzip2", "indexed_bzip2", "fbz", "lbzip2-rs", "libbzip2", "libbz2-rs"];
  const CMP_SHARED = { bz2ngrs: "lbzip2" };
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
      if (CMP_SHARED[a]) a = CMP_SHARED[a];
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
      if (drawn.hero && drawn.hero[1].isConnected) trajectory(...drawn.hero);
      if (drawn.cmp && drawn.cmp[1].isConnected) alternativesHero(...drawn.cmp);
    }, 150);
  });

  function init() {
    const root = document.getElementById("bench-history");
    if (root) {
      fetch(root.dataset.src)
        .then((r) => { if (!r.ok) throw new Error(r.status); return r.json(); })
        .then((d) => {
          // the release-scaling runs, if any are recorded (none is no error: the view is offered only with one)
          const hero = document.getElementById("bench-hero");
          const src = hero && hero.dataset.ws;
          const ws = src ? fetch(src).then((r) => (r.ok ? r.json() : null)).catch(() => null) : Promise.resolve(null);
          return ws.then((w) => { d.scaling = w; return d; });
        })
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
