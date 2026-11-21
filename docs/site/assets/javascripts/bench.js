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
  let state = { metric: "mb_s", threads: "all", cpu: null, tCorpus: "wikidata", tThreads: "4", tFig: "wall", tView: "ci", tScale: "rel", wsRun: null, wsCorpus: "wikidata", wsLinT: null };
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

  // OKLab and OKLCH for the releases' ramp: hex in, hex out, mixing as CSS color-mix(in
  // oklch) does (the shorter way round the hue), out-of-gamut channels clipped as Chromium
  // renders them
  const Ok = (() => {
    const lin = (c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
    const gam = (c) => (c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055);
    const lab = (hex) => {
      const [r, g, b] = [1, 3, 5].map((k) => lin(parseInt(hex.trim().slice(k, k + 2), 16) / 255));
      const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
      const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
      const q = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
      return [0.2104542553 * l + 0.793617785 * m - 0.0040720468 * q, 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * q, 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * q];
    };
    const lch = (hex) => { const [L, a, b] = lab(hex); return [L, Math.hypot(a, b), ((Math.atan2(b, a) * 180) / Math.PI + 360) % 360]; };
    const hex = ([L, C, h]) => {
      const a = C * Math.cos((h * Math.PI) / 180), b = C * Math.sin((h * Math.PI) / 180);
      const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3, m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3, q = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
      const rgb = [4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * q, -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * q, -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * q];
      return "#" + rgb.map((c) => Math.round(255 * gam(Math.min(1, Math.max(0, c)))).toString(16).padStart(2, "0")).join("");
    };
    const mix = (p, q, t) => {
      let dh = q[2] - p[2];
      if (dh > 180) dh -= 360;
      if (dh < -180) dh += 360;
      return [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t, (p[2] + dh * t + 360) % 360];
    };
    const dE = (p, q) => { const a = lab(p), b = lab(q); return Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]); };
    // the ramp through the stops, walked by distance: at(t) is the colour a fraction t of the
    // way along it by OKLab arc length, so equal steps in t are equal perceived steps
    const ramp = (stops) => {
      const S = stops.map(lch), N = 400, pts = [], cum = [0];
      for (let k = 0; k <= N; k++) {
        const u = (k * (S.length - 1)) / N, a = Math.min(S.length - 2, Math.floor(u));
        pts.push(hex(mix(S[a], S[a + 1], u - a)));
        if (k) cum.push(cum[k - 1] + dE(pts[k - 1], pts[k]));
      }
      return (t) => {
        const g = Math.max(0, Math.min(1, t)) * cum[N];
        let k = 1;
        while (k < N && cum[k] < g) k++;
        const f = (g - cum[k - 1]) / (cum[k] - cum[k - 1] || 1);
        return hex(mix(lch(pts[k - 1]), lch(pts[k]), f));
      };
    };
    return { lch, hex, ramp };
  })();

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
  // Absolute (the Scale chips): each figure in its own units, CI's chain carried from master's
  // records (bench-history.json's series, ciAnchor). [the records' field, the units in a
  // readout, on a narrow axis and on the axis, the figure's level as a phrase, its format and
  // its intervals' half-widths', and which way is better (in either view)]
  const ABS = {
    wall: { key: "mb_s", unit: "MB/s", short: "MB/s", axis: "MB/s of output", what: "MB/s", fmt: (v) => fmtMB(v), fmtH: (v) => fmtMB(v), better: "Up is faster." },
    cpu: { key: "cpu_s", unit: "CPU seconds", short: "CPU s", axis: "CPU seconds", what: "CPU seconds", fmt: (v) => v.toPrecision(3), fmtH: (v) => String(+v.toPrecision(2)), better: "Down is less CPU time." },
    rss: { key: "rss_mb", unit: "MB", short: "MB", axis: "MB of peak RSS", what: "peak RSS", fmt: (v) => v.toFixed(1), fmtH: (v) => String(+v.toPrecision(2)), better: "Down is less memory." },
  };
  // an axis's tick labels: as many decimals as its step needs, so no two ticks read alike
  const fmtStep = (t) => {
    const st = t.length > 1 ? Math.abs(t[1] - t[0]) : 1;
    let d = 0;
    while (d < 6 && Math.abs(st * 10 ** d - Math.round(st * 10 ** d)) > 1e-6) d++;
    return (v) => v.toFixed(d);
  };
  const pct = (L, sign) => 100 * (Math.exp(sign * L) - 1);
  const fmtPct = (v) => {
    const a = Math.abs(v).toFixed(Math.abs(v) < 10 ? 1 : 0);
    return `${+a === 0 ? "" : v >= 0 ? "+" : "−"}${a}%`; // no sign on a change that rounds to zero
  };
  // What changed in a release (assets/release-notes.json, docs/scripts/release_notes.py): a
  // panel under the hero with the notes of the release in focus, the newest until a release or
  // a release-scaling run is pointed at, tapped or chosen. It keeps the last one shown. Its
  // lede, its "why" sentences (the first of each bold summary in CHANGELOG.md), its changes
  // with their pull requests, and the posts tagged with it (front matter `releases`); links
  // to its entry on this page and to its CHANGELOG section.
  const GH = "https://github.com/lmmx/mibz2";
  let rnShow = () => {};
  // plain text with its code spans and its #N references (to GitHub, which redirects an
  // issue number to its pull request)
  const rich = (text, parent) => {
    for (const part of text.split(/(`[^`]+`|#\d+\b)/)) {
      if (!part) continue;
      if (part.startsWith("`")) el("code", { text: part.slice(1, -1) }, parent);
      else if (/^#\d+$/.test(part)) el("a", { href: `${GH}/issues/${part.slice(1)}`, text: part }, parent);
      else parent.append(part);
    }
    return parent;
  };
  const longDate = (d) => {
    const [y, m, day] = d.split("-").map(Number);
    return `${day} ${["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"][m - 1]} ${y}`;
  };
  function notesPanel(host, data) {
    rnShow = () => {};
    const N = data.notes && data.notes.releases;
    if (!N || !N.length) return;
    const by = new Map(N.map((r) => [r.version, r]));
    const panel = el("aside", { class: "bh-rn", "aria-live": "polite", "aria-label": "What changed in the release in focus" }, host);
    let cur = null;
    const more = (n, what, r) => {
      const li = el("li", { class: "bh-rn-more" });
      li.append(`and ${n} more ${what} in `);
      el("a", { href: `#${r.anchor}`, text: "its notes" }, li);
      return li;
    };
    rnShow = (v) => {
      const r = by.get(v);
      if (!r || r.version === cur) return;
      cur = r.version;
      panel.textContent = "";
      const head = el("p", { class: "bh-rn-head" }, panel);
      el("span", { class: "bh-rn-kicker", text: "What changed in " }, head);
      el("strong", { text: r.version }, head);
      el("span", { class: "bh-rn-date", text: longDate(r.date) }, head);
      const links = el("span", { class: "bh-rn-links" }, head);
      el("a", { href: `#${r.anchor}`, text: "Notes" }, links);
      el("a", { href: r.changelog, text: "CHANGELOG" }, links);
      if (r.lede) rich(r.lede, el("p", { class: "bh-rn-lede" }, panel));
      const cap = r.lede ? 3 : 4;
      if (r.why.length) {
        const ul = el("ul", { class: "bh-rn-why" }, panel);
        for (const w of r.why.slice(0, cap)) rich(w, el("li", {}, ul));
        if (r.why.length > cap) ul.append(more(r.why.length - cap, r.why.length - cap === 1 ? "note" : "notes", r));
      }
      if (r.changes.length) {
        const ul = el("ul", { class: "bh-rn-prs", "aria-label": "Its changes" }, panel);
        for (const c of r.changes.slice(0, 4)) {
          const li = el("li", {}, ul);
          for (const pr of c.prs) el("a", { class: "bh-rn-pr", href: pr.url, text: `#${pr.n}` }, li);
          if (c.scope) el("span", { class: "bh-rn-scope", text: c.scope }, li);
          rich(c.title, el("span", { class: "bh-rn-title" }, li));
        }
        if (r.changes.length > 4) ul.append(more(r.changes.length - 4, "changes", r));
      }
      if (r.posts.length) {
        const read = el("p", { class: "bh-rn-read" }, panel);
        el("span", { class: "bh-rn-kicker", text: "Read" }, read);
        for (const post of r.posts) el("a", { class: "bh-rn-post", href: `${data.notesRoot || "../../"}${post.path}`, text: post.title }, read);
      }
      if (!r.lede && !r.why.length && !r.changes.length && !r.posts.length) {
        el("p", { class: "bh-rn-lede", text: "Its CHANGELOG section lists no change to the decoder." }, panel);
      }
    };
    rnShow(N[0].version);
  }

  function trajectory(data, root) {
    const host = document.getElementById("bench-hero") || el("div", {}, root);
    host.textContent = "";
    host.classList.add("bh-root");
    const box = el("section", { class: "bh-hero" }, host);
    notesPanel(host, data);
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
    if (state.tView === "ws") { releaseScaling(ws, box, seg, (data.scaling && data.scaling.lineage) || []); return; }
    if (start === null || start === undefined || !data.links || !data.links.length) {
      el("p", { class: "bh-hero-note", text: "No merged pull request's benchmark is linked yet (bench/links.py)." }, box);
      return;
    }
    const keys = Object.keys(ch.series);
    const corpora = [...new Set(keys.map((k) => k.split("|")[0]))].sort((a, b) => (a !== "wikidata") - (b !== "wikidata") || a.localeCompare(b));
    if (!corpora.includes(state.tCorpus)) state.tCorpus = corpora.includes("wikidata") ? "wikidata" : corpora[0];
    if (!TRAJ.some((f) => f[0] === state.tFig)) state.tFig = "wall";
    const ciThreads = [...new Set(keys.filter((k) => k.startsWith(`${state.tCorpus}|`)).map((k) => k.split("|")[1]))];
    // The release-scaling machine's releases (modelled, throughput only), on this corpus
    const lg = state.tFig === "wall" ? ((data.scaling && data.scaling.lineage) || []).find((g) => g.corpus === state.tCorpus) : null;
    const lgThreads = lg ? lg.threads.map(String).filter((t) => releaseSeries(data, lg, start, t)) : [];
    const threads = [...new Set([...ciThreads, ...lgThreads])].sort((a, b) => a - b);
    const opts = threads.map((t) => [t, `${t} thread${t === "1" ? "" : "s"}`]);
    if (lgThreads.length > 1) opts.push(["all", `All (${shortCpu(lg.cpu)})`]);
    if (!opts.some((o) => o[0] === state.tThreads)) state.tThreads = threads.includes("4") ? "4" : threads[threads.length - 1];
    seg("Corpus", corpora.map((c) => [c, c]), state.tCorpus, (v) => (state.tCorpus = v));
    seg("Threads", opts, state.tThreads, (v) => (state.tThreads = v));
    seg("Figure", TRAJ.map((f) => [f[0], f[1]]), state.tFig, (v) => (state.tFig = v));
    // Relative (the default: change since the start) or absolute (the figure in its own units:
    // MB/s, CPU seconds or MB of peak RSS), for every figure
    seg("Scale", [["rel", "Relative"], ["abs", "Absolute"]], state.tScale, (v) => (state.tScale = v));
    const abs = state.tScale === "abs";
    const fig = TRAJ.find((f) => f[0] === state.tFig);
    const sign = fig[2];
    const ax = ABS[fig[0]];
    const T = state.tThreads;
    const tWord = `${T} thread${T === "1" ? "" : "s"}`;
    if (T === "all") { releaseOverlay(data, lg, start, lgThreads, box, abs); return; }
    const key = `${state.tCorpus}|${T}`;
    const ciOn = ciThreads.includes(T);
    const chain = ciOn ? ch.series[key][fig[0]] : [];
    const own = ciOn ? (data.changes[key] || {})[fig[0]] || [] : [];
    const rs = lgThreads.includes(T) ? releaseSeries(data, lg, start, T) : null;
    const idx = [];
    for (let i = start; i < data.links.length; i++) idx.push(i);
    // Absolute: CI's chain in the figure's units, on one runner model's scale through one master
    // record (ciAnchor), drawn only over the anchor's segment: a gap leaves the rest unanchored
    const anc = abs && ciOn ? ciAnchor(data, state.tCorpus, T, chain, ax.key) : null;
    const pts = idx.filter((i) => chain[i] && (!abs || (anc && chain[i][2] === chain[anc.i][2])));
    const unanchored = anc ? new Set(idx.filter((i) => chain[i] && chain[i][2] !== chain[anc.i][2]).map((i) => chain[i][2])).size : 0;
    const lead = el("p", { class: "bh-hero-focus" }, box);
    const body = el("div", { class: "bh-hero-chart" }, box);
    const key2 = el("p", { class: "bh-hero-commit bh-traj-key" }, box);
    const readout = el("p", { class: "bh-hero-commit", "aria-live": "polite" }, box);
    if (!pts.length && !rs) { el("p", { text: abs && ciOn ? `No master record at a chained merge anchors this chain in ${ax.unit}.` : "No link measured this figure." }, body); return; }
    const tag = (l) => (l.pr ? `#${l.pr}` : l.sha.slice(0, 7));
    const decoderLinks = pts.filter((i) => data.links[i].decoder);
    const last = pts.length ? pts[pts.length - 1] : null;
    const cpuR = rs ? shortCpu(lg.cpu) : null;
    const rLast = rs ? rs.pts[rs.pts.length - 1] : null;
    const relHead = rs ? `${fmtPct(rLast.c[0])} (95%: ${fmtPct(rLast.c[1])} to ${fmtPct(rLast.c[2])}) from ${rs.base} to ${rLast.version}` : "";
    // the top panel's figures: the change since the start (relative), or the figure in its own
    // units (absolute); its axis's labels, once its ticks are known
    const fmtV = abs ? ax.fmt : fmtPct;
    let fmtTop = fmtPct;
    // the chain at merge i, and its 95% interval [lo, hi]. Absolute: the anchor's record carried
    // by the chain's ratios; the interval is the links' between merge i and the anchor (the
    // chain's variances differenced, as they add in quadrature) with the record's own, so it
    // narrows to the record's at the anchor
    const cv = (i) => (abs ? anc.v * Math.exp(sign * (chain[i][0] - chain[anc.i][0])) : pct(chain[i][0], sign));
    const cb = (i) => {
      if (abs) {
        const w = Math.sqrt(Math.abs(chain[i][1] ** 2 - chain[anc.i][1] ** 2) + anc.r ** 2), m = cv(i);
        return [m * Math.exp(-w), m * Math.exp(w)];
      }
      const a = pct(chain[i][0] - chain[i][1], sign), b = pct(chain[i][0] + chain[i][1], sign);
      return [Math.min(a, b), Math.max(a, b)];
    };
    // a release's figures: its change since the oldest placed release, or its modelled MB/s
    const rv = (p) => (abs ? p.mb : p.c);
    // Absolute with CI's chain and the releases: two machines, two scales, so the releases take
    // a panel of their own, never CI's axis
    const sep = !!(abs && pts.length && rs);
    const ancName = anc ? `${anc.sha}${anc.pr ? ` (#${anc.pr})` : ""}` : "";
    if (pts.length) {
      const [L, S, sg] = chain[last];
      const segStart = pts.find((i) => chain[i][2] === sg);
      const cpus = [...new Set(decoderLinks.map((i) => data.links[i].cpu))];
      if (abs) {
        const [lo, hi] = cb(last);
        const strong = el("strong", { text: `${fig[1]}, ${state.tCorpus} at ${tWord}: ${fmtV(cv(last))} ${ax.unit} on the ${anc.cpu}'s scale` }, lead);
        strong.title = `the chain in ${ax.unit}, through one master record on one runner model`;
        lead.append(` (95%: ${fmtV(lo)} to ${fmtV(hi)}) at ${tag(data.links[last])}. Anchored at ${ancName}, master's newest record on the ${anc.cpu} (${fmtV(anc.v)} ± ${ax.fmtH(anc.h)} ${ax.unit}, ${anc.n} rounds), the runner model with the most records of ${state.tCorpus} at ${tWord} (${anc.count} of ${anc.total}). The chain's ratios, measured on ${cpus.join(", ")}, carry it to every other merge from ${tag(data.links[segStart])}'s base on: an ${anc.cpu}'s ${ax.what}, not any one run's. ${ax.better}`);
        if (unanchored) lead.append(` ${unanchored} earlier segment${unanchored === 1 ? "" : "s"}, before a gap, ${unanchored === 1 ? "has" : "have"} no anchor and ${unanchored === 1 ? "is" : "are"} not drawn.`);
        if (rs) lead.append(` Below it, in a panel of its own: the ${cpuR}'s releases, modelled, in MB/s on the ${cpuR}: another machine, on another scale.`);
      } else {
        const lv = pct(L, sign), lo = pct(L - S, sign), hi = pct(L + S, sign);
        const strong = el("strong", { text: `${fig[1]}, ${state.tCorpus} at ${tWord}: ${fmtPct(lv)}` }, lead);
        strong.title = "chained change since the segment's start";
        lead.append(` (95%: ${fmtPct(Math.min(lo, hi))} to ${fmtPct(Math.max(lo, hi))}) from ${tag(data.links[segStart])}'s base to ${tag(data.links[last])}, over ${decoderLinks.filter((i) => chain[i][2] === sg).length} decoder merges, each measured on its own runner (${cpus.join(", ")}). ${ax.better}`);
        if (rs) lead.append(` Dotted: the ${cpuR}'s releases, modelled, ${relHead}; another machine, with its own zero at ${rs.base}.`);
      }
    } else if (abs) {
      el("strong", { text: `${fig[1]}, ${state.tCorpus} at ${tWord}, ${cpuR} releases only: ${fmtMB(rLast.mb[0])} MB/s at ${rLast.version}` }, lead);
      lead.append(` (95%: ${fmtMB(rLast.mb[1])} to ${fmtMB(rLast.mb[2])}), modelled from ${lg.runs.length} release-against-release runs on one ${cpuR}: MB/s on that machine. CI's runners have 4 CPUs, so no merge is measured at ${tWord}: each point is a release at the last merge its tag contains, and the dotted line between releases is not measured. Up is faster.`);
    } else {
      el("strong", { text: `${fig[1]}, ${state.tCorpus} at ${tWord}, ${cpuR} releases only: ${fmtPct(rLast.c[0])}` }, lead);
      lead.append(` (95%: ${fmtPct(rLast.c[1])} to ${fmtPct(rLast.c[2])}) from ${rs.base} to ${rLast.version}, modelled from ${lg.runs.length} release-against-release runs on one ${cpuR}. CI's runners have 4 CPUs, so no merge is measured at ${tWord}: each point is a release at the last merge its tag contains, and the dotted line between releases is not measured. Up is faster.`);
    }

    const W = Math.max(300, Math.round(body.clientWidth) || 700);
    const narrow = W < 560;
    const H1 = narrow ? 230 : 280, H2 = narrow ? 130 : 150, gapY = 38;
    // (absolute, with both: the releases' panel between the chain's and the merges' own)
    const H3 = sep ? (narrow ? 150 : 170) : 0, gap3 = sep ? 50 : 0;
    const H = H1 + gap3 + H3 + gapY + H2;
    const label = abs
      ? (pts.length
        ? `${fig[1]} of ${state.tCorpus} at ${tWord} in ${ax.unit} on the ${anc.cpu}'s scale, anchored at ${anc.sha}: the chain over ${pts.length} merges, and each merge's own change${rs ? `; below, in a separate panel, the ${cpuR}'s releases, modelled, in MB/s on the ${cpuR}` : ""}`
        : `${fig[1]} of ${state.tCorpus} at ${tWord}: the ${cpuR}'s releases, modelled, in MB/s on the ${cpuR}, and each release's change against the one before it`)
      : (pts.length ? `${fig[1]} of ${state.tCorpus} at ${tWord}: the change chained over ${pts.length} merges, and each merge's own change${rs ? `, with the ${cpuR}'s releases, modelled` : ""}` : `${fig[1]} of ${state.tCorpus} at ${tWord}: the ${cpuR}'s releases, modelled, relative to ${rs.base}, and each release's change against the one before it`);
    const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, role: "img", "aria-label": label }, body);
    // y scales: the chain and the releases (top, one scale: change since each one's start; or,
    // absolute, MB/s), the releases' own panel when absolute with both (middle, MB/s on their
    // machine) and each link's own change, or each release's against the one before (bottom),
    // in percent
    const band = pts.flatMap(cb);
    const ext = rs && !sep ? rs.pts.flatMap((p) => [rv(p)[1], rv(p)[2]]) : [];
    band.push(...ext);
    // (the outer ticks at or past the releases' outermost marks, so none leaves its panel; CI's
    // own scales as they were)
    const cover = (t, lo, hi) => {
      const st = t.length > 1 ? t[1] - t[0] : 1;
      while (t[t.length - 1] < hi) t.push(+(t[t.length - 1] + st).toFixed(10));
      while (t[0] > lo) t.unshift(+(t[0] - st).toFixed(10));
      return t;
    };
    let t1;
    if (abs) {
      t1 = cover(ticks(Math.min(...band), Math.max(...band), narrow ? 4 : 5), Math.min(...band), Math.max(...band));
      // (throughput's axis as it was; CPU seconds and MB to their step, which can be fine)
      fmtTop = fig[0] === "wall" ? fmtMB : fmtStep(t1);
    } else {
      const rvx = rs ? ext : [0];
      t1 = cover(ticks(Math.min(0, ...band), Math.max(0, ...band), narrow ? 4 : 5), Math.min(0, ...rvx), Math.max(0, ...rvx));
    }
    const v3 = sep ? rs.pts.flatMap((p) => [p.mb[1], p.mb[2]]) : [];
    const t3 = sep ? cover(ticks(Math.min(...v3), Math.max(...v3), 3), Math.min(...v3), Math.max(...v3)) : [];
    const ownV = ciOn
      ? idx.filter((i) => own[i]).map((i) => [pct(own[i][0] - own[i][1], sign), pct(own[i][0] + own[i][1], sign)]).flat()
      : rs.pts.filter((p) => p.step).flatMap((p) => [p.step.c[1], p.step.c[2]]);
    const t2 = ticks(Math.min(0, ...ownV), Math.max(0, ...ownV), 3);
    // (absolute too: there a panel sits above, so no whisker may leave this one)
    if (!ciOn || abs) cover(t2, Math.min(0, ...ownV), Math.max(0, ...ownV));
    const meas = svg("text", { class: "bh-tick", x: 0, y: -50 }, s);
    const textW = (t) => { meas.textContent = t; return meas.getComputedTextLength(); };
    const tw = Math.max(24, ...t1.map((v) => textW(fmtTop(v))), ...t2.map((v) => textW(fmtPct(v))), ...t3.map((v) => textW(fmtMB(v))));
    meas.remove();
    // (with releases on the top panel, a row above the plot names each one at its merge;
    // absolute, a row above that names the axis)
    const M = { l: Math.ceil(tw) + 12, r: 12, t: (rs && !sep ? 30 : 14) + (abs ? 16 : 0), b: 26 };
    const inner = W - M.l - M.r;
    const xs = new Map(idx.map((i, k) => [i, M.l + (idx.length === 1 ? inner / 2 : 8 + (k * (inner - 16)) / (idx.length - 1))]));
    const y1 = (v) => M.t + (H1 - M.t - 8) * (1 - (v - t1[0]) / (t1[t1.length - 1] - t1[0] || 1));
    const top3 = H1 + gap3;
    const y3 = (v) => top3 + (H3 - 8) * (1 - (v - t3[0]) / (t3[t3.length - 1] - t3[0] || 1));
    const top2 = H1 + gap3 + H3 + gapY;
    const y2 = (v) => top2 + (H2 - M.b) * (1 - (v - t2[0]) / (t2[t2.length - 1] - t2[0] || 1));
    for (const v of t1) {
      svg("line", { x1: M.l, x2: W - M.r, y1: y1(v), y2: y1(v), class: v === 0 && !abs ? "bh-zero" : "bh-grid" }, s);
      svg("text", { x: M.l - 6, y: y1(v) + 4, class: "bh-tick", "text-anchor": "end" }, s).textContent = fmtTop(v);
    }
    for (const v of t3) {
      svg("line", { x1: M.l, x2: W - M.r, y1: y3(v), y2: y3(v), class: "bh-grid" }, s);
      svg("text", { x: M.l - 6, y: y3(v) + 4, class: "bh-tick", "text-anchor": "end" }, s).textContent = fmtMB(v);
    }
    for (const v of t2) {
      svg("line", { x1: M.l, x2: W - M.r, y1: y2(v), y2: y2(v), class: v === 0 ? "bh-zero" : "bh-grid" }, s);
      svg("text", { x: M.l - 6, y: y2(v) + 4, class: "bh-tick", "text-anchor": "end" }, s).textContent = fmtPct(v);
    }
    if (abs) {
      svg("text", { x: M.l, y: 12, class: "bh-unit" }, s).textContent = pts.length
        ? (narrow ? `${ax.short}, ${anc.cpu}'s scale, at ${anc.sha}` : `${ax.axis}, on the ${anc.cpu}'s scale, anchored at ${ancName}`)
        : (narrow ? `MB/s on the ${cpuR}, modelled` : `MB/s of output on the ${cpuR}, modelled`);
    }
    if (sep) {
      svg("text", { x: M.l, y: H1 + 14, class: "bh-unit" }, s).textContent = narrow
        ? `${cpuR} releases: MB/s on the ${cpuR}`
        : `${cpuR} releases, modelled: MB/s on the ${cpuR}, another machine and scale`;
    }
    svg("text", { x: M.l, y: top2 - 14, class: "bh-unit" }, s).textContent = ciOn
      ? (narrow ? "each merge's own change (95%)" : "each merge's own change, with its 95% interval (hollow: the decoder unchanged, an A/A pair)")
      : (narrow ? "each release against the one before (modelled, 95%)" : `each release against the one before it, modelled on the ${cpuR}, with its 95% interval`);
    // segments of the chain: a band and a line each, never bridged across a gap
    const segs = [];
    for (const i of pts) {
      const g = chain[i][2];
      if (!segs.length || segs[segs.length - 1].g !== g) segs.push({ g, pts: [] });
      segs[segs.length - 1].pts.push(i);
    }
    const gapLabels = [];
    for (const sgm of segs) {
      const p = sgm.pts;
      const upper = p.map((i) => `${xs.get(i)},${y1(cb(i)[1])}`);
      const lower = p.map((i) => `${xs.get(i)},${y1(cb(i)[0])}`).reverse();
      svg("polygon", { points: [...upper, ...lower].join(" "), class: "bh-band" }, s);
      svg("polyline", { points: p.map((i) => `${xs.get(i)},${y1(cv(i))}`).join(" "), class: "bh-chain" }, s);
      if (sgm !== segs[0]) {
        const xg = xs.get(p[0]) - 6;
        svg("line", { x1: xg, x2: xg, y1: M.t, y2: H - M.b, class: "bh-gap" }, s);
        // the label on the side of the line with room for it, so it never leaves the plot
        const right = xg > M.l + inner / 2;
        const gt = svg("text", { x: right ? xg - 3 : xg + 3, y: M.t + 10, class: "bh-tick", "text-anchor": right ? "end" : "start" }, s);
        gt.textContent = narrow ? "gap" : "gap: a decoder change with no link";
        // (a label that would run into an earlier one's says only "gap", or nothing)
        const span = (e) => { const b = e.getBBox(); return [b.x, b.x + b.width]; };
        const hits = (e) => gapLabels.some((q) => { const [a, b] = span(e), [c, d] = span(q); return a < d + 4 && c < b + 4; });
        if (hits(gt)) gt.textContent = "gap";
        if (hits(gt)) gt.remove();
        else gapLabels.push(gt);
        svg("title", {}, gt).textContent = "a decoder change with no link: the chain starts again here";
      }
    }
    // absolute: the anchor, ringed
    if (anc && pts.length) {
      const ring = svg("circle", { cx: xs.get(anc.i), cy: y1(cv(anc.i)), r: 7, class: "bh-anchor" }, s);
      svg("title", {}, ring).textContent = `the anchor: ${ancName}, master's record on the ${anc.cpu}, ${fmtV(anc.v)} ± ${ax.fmtH(anc.h)} ${ax.unit}`;
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
    const on = new Set(pts);
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
      if (abs && on.has(i)) {
        const [lo, hi] = cb(i);
        readout.append(` · chained ${fmtV(cv(i))} ${ax.unit} on the ${anc.cpu}'s scale (95%: ${fmtV(lo)} to ${fmtV(hi)})`);
        if (i === anc.i) readout.append(` · the anchor: master's record here on the ${anc.cpu}, ${anc.n} rounds`);
      } else if (!abs && chain[i]) readout.append(` · chained ${fmtPct(pct(chain[i][0], sign))}`);
      if (l.base_ok === false || l.head_ok === false) readout.append(" · its run's base or head differs from the merge's parent or the merge on the benchmarked paths");
      readout.append(" · ");
      el("a", { href: l.run, text: "run" }, readout);
      readout.append(" · ");
      el("a", { href: `${data.repo}/pull/${l.pr}`, text: "pull request" }, readout);
    };
    if (pts.length) {
      for (const i of idx) {
        const l = data.links[i];
        const g = svg("g", { tabindex: 0, class: "bh-pt", role: "button", "aria-label": abs && on.has(i) ? `${tag(l)}: ${fmtV(cv(i))} ${ax.unit} on the ${anc.cpu}'s scale` : `${tag(l)}` }, s);
        if (on.has(i)) svg("circle", { cx: xs.get(i), cy: y1(cv(i)), r: l.decoder ? 3.5 : 2.5, class: l.decoder ? "bh-dot-chain" : "bh-dot-aa" }, g);
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
    }
    if (rs) {
      const tip = el("div", { class: "bh-tip", hidden: "" }, box);
      drawReleases(s, rs, { x: (p) => xs.get(p.link), y: sep ? y3 : y1, v: rv, y2: ciOn ? null : y2, top: sep ? top3 : M.t, left: M.l, right: W - M.r, W, narrow, tip, box, readout: pts.length ? null : readout, tag, data });
    }
    // the key: what is measured, and what is modelled
    if (pts.length) {
      const k1 = el("span", { class: "bh-ws-key" }, key2);
      el("span", { class: "bh-swatch is-line", style: "border-color:var(--bh-s1)" }, k1);
      k1.append(abs
        ? ` CI runners, merge by merge (measured, chained; ${ax.unit} on the ${anc.cpu}'s scale, anchored at ${anc.sha}; 95% band)  `
        : ` CI runners, merge by merge (measured, chained, 95% band)  `);
    }
    if (rs) {
      const k2 = el("span", { class: "bh-ws-key" }, key2);
      el("span", { class: "bh-swatch is-dotted" }, k2);
      k2.append(abs
        ? ` ${cpuR}, releases (modelled, MB/s on the ${cpuR}${sep ? ", in their own panel" : ""}; dotted between releases, not measured)`
        : ` ${cpuR}, releases (modelled; dotted between releases, not measured)`);
    }
    let note;
    if (abs) {
      note = pts.length
        ? `Top: the chain in ${ax.unit}, on one runner model's scale. CI's merges ran on several runner CPUs and the chain multiplies paired ratios, so it has no absolute scale of its own: it borrows the ${anc.cpu}'s, through master's newest record on that runner at a chained merge (${ancName}, ringed). Each merge's figure is that record carried by the chain's ratios; the band is the links' 95% interval between the merge and the anchor with the record's own, so it is narrowest at the anchor. ${runnerNote(fig[0], anc, ax)}${rs ? ` Middle: the ${cpuR}'s releases (wikidata, Release scaling), each at the last merge its tag contains, their throughput modelled in MB/s on the ${cpuR}, with 95% whiskers: another machine on another scale, so a panel of its own; its levels are not comparable with the ${anc.cpu}'s. Point at or tap a diamond for its release.` : ""} Bottom: each merge's own change, its head against its base on one runner, in percent in either view. Point at, tap or Tab to a merge for its figures. What the chain assumes is under the chart.`
        : `Only the ${cpuR} has figures at ${tWord}. Top: each release's throughput in MB/s on that one machine, modelled from release-against-release runs (the Release scaling view), with 95% whiskers, at the last merge its tag contains; the dotted line only joins releases. Below: each release against the one before it, in percent. Point at, tap or Tab to a release for its figures.`;
    } else {
      note = pts.length
        ? `Top: the change chained since the start, with its 95% band. Below: each merge's own change, its head against its base on one runner. Point at, tap or Tab to a merge for its figures.${rs ? ` Diamonds: the ${cpuR}'s releases (wikidata, Release scaling), each at the last merge its tag contains, its throughput modelled from release-against-release runs relative to ${rs.base}, with 95% whiskers. The two series share the axis and its meaning (change since its own start, up faster), not a zero or a machine; point at or tap a diamond for its release.` : ""} What the chain assumes is under the chart.`
        : `Only the ${cpuR} has figures at ${tWord}. Top: each release's throughput relative to ${rs.base}, modelled from release-against-release runs on that one machine (the Release scaling view), with 95% whiskers, at the last merge its tag contains; the dotted line only joins releases. Below: each release against the one before it. Point at, tap or Tab to a release for its figures.`;
    }
    el("p", { class: "bh-hero-note", text: note }, box);
  }

  // MB/s on an axis or in a readout
  const fmtMB = (v) => v.toFixed(Math.abs(v) < 100 ? 1 : 0);

  // What the anchor's runner model means for an absolute figure (the note under the chart):
  // throughput and CPU time scale with the runner's speed, so the level is that runner's;
  // peak RSS hardly depends on the runner, which the records show (their spread across runner
  // models: ciAnchor's spread)
  function runnerNote(f, anc, ax) {
    const sp = anc.spread;
    const range = sp && sp.models > 1 ? `master's newest records on the ${sp.models} runner models at this corpus and thread count, at different merges, range from ${ax.fmt(sp.lo)} to ${ax.fmt(sp.hi)} ${ax.unit}` : "";
    const across = `It reads as what an ${anc.cpu} would have measured, assuming each relative change holds across runners`;
    if (f === "wall") return `${across}; other runners run up to 2× faster or slower.`;
    if (f === "cpu") return `${across}. CPU time scales with the runner's speed${range ? `: ${range}` : ""}.`;
    const apart = range ? (100 * (sp.hi / sp.lo - 1)).toFixed(1) : null;
    if (range && apart <= 10) {
      return `${across}. Peak RSS, unlike time, hardly depends on the runner: ${range}, within ${apart}% of each other. So this level is close to any runner's, not only the ${anc.cpu}'s.`;
    }
    return `${across}${range ? `. Peak RSS depends on the runner less than time does, but here ${range}, ${apart}% apart` : ""}.`;
  }

  // CI's chain in a figure's own units (field: mb_s, cpu_s or rss_mb, each a [mean, half-width]
  // in master's records). Its merges ran on several runner CPUs and the chain multiplies paired
  // ratios (head over base, of wall, CPU and peak RSS alike: bench/links.py), so it has no
  // absolute scale of its own: it borrows one runner model's. That is the model with the most
  // master records of the corpus at T threads (bench-history.json's series; ties to the newest
  // record), through its newest record at a merge on the chain. The chain's ratios carry that
  // record to every other merge of its segment. Also each model's newest record's spread (lo,
  // hi, models), for the note. Null when no record of any model sits at a chained merge.
  function ciAnchor(data, corpus, T, chain, field = "mb_s") {
    const sr = (data.series || []).find((q) => q.corpus === corpus && String(q.threads) === T);
    if (!sr) return null;
    const link = new Map(data.links.map((l, i) => [l.sha, i]));
    const cpuOf = (c) => ((data.classes_meta || [])[c] || {}).cpu || data.classes[c];
    const recs = sr.lines.flatMap((l) => l.points.filter((p) => p[field]).map((p) => ({ p, cpu: cpuOf(l.class), i: link.get(data.commits[p.c].sha) })));
    const count = new Map(), newest = new Map(), latest = new Map();
    for (const r of recs) {
      count.set(r.cpu, (count.get(r.cpu) || 0) + 1);
      newest.set(r.cpu, Math.max(newest.has(r.cpu) ? newest.get(r.cpu) : -1, r.p.c));
      if (!latest.has(r.cpu) || r.p.c > latest.get(r.cpu).p.c) latest.set(r.cpu, r);
    }
    const lv = [...latest.values()].map((r) => r.p[field][0]);
    const spread = { lo: Math.min(...lv), hi: Math.max(...lv), models: lv.length };
    const cpus = [...count.keys()].sort((a, b) => count.get(b) - count.get(a) || newest.get(b) - newest.get(a));
    for (const cpu of cpus) {
      const at = recs.filter((r) => r.cpu === cpu && r.i !== undefined && chain[r.i]).sort((a, b) => b.p.c - a.p.c)[0];
      if (!at) continue;
      const [v, h] = at.p[field], cm = data.commits[at.p.c];
      return { cpu, i: at.i, v, h, r: Math.log1p(h / v), n: at.p.n, sha: cm.sha.slice(0, 7), pr: cm.pr, count: count.get(cpu), total: recs.length, spread };
    }
    return null;
  }

  // A machine's name without its maker's marks: "Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz" is
  // the i9-10900K
  const shortCpu = (c) => String(c).replace(/^.*?\b(?:Core\(TM\)|Ryzen)\s+/, "").replace(/\s+CPU\b.*$/, "").replace(/\s+@.*$/, "");

  // The release-scaling lineage's releases on the merge axis at T threads: each release that
  // has a tag on master after the chain's start (bench_history.py, release_positions) at the
  // last merge its tag contains, with its modelled throughput (lbzip2's measured MB/s in the
  // newest run over the release's pooled wall ratio, as the All releases view has it), its
  // change since the lineage's oldest placed release, and against the release before it.
  // Null when fewer than two releases can be placed.
  function releaseSeries(data, g, start, T) {
    const P = g.per_thread[T];
    if (!P || !P.anchor_mbps) return null;
    const pos = new Map((data.releases || []).filter((r) => r.link >= start && r.link < data.links.length).map((r) => [r.version, r]));
    const R = g.releases, k = g.t975;
    const ok = (i) => P.x[i] !== null && P.x[i] !== undefined;
    const placed = R.map((_, i) => i).filter((i) => ok(i) && pos.has(R[i]));
    if (placed.length < 2) return null;
    const b = placed[0];
    const mb = (i, z = 0) => P.anchor_mbps / Math.exp(P.x[i] - z * Math.sqrt(P.v[i]));
    // throughput of release i against release a: the ratio of their pooled figures
    const vs = (a, i) => {
      const d = P.x[a] - P.x[i], h = a === i ? 0 : k * Math.sqrt(P.v[a] + P.v[i]);
      return [100 * (Math.exp(d) - 1), 100 * (Math.exp(d - h) - 1), 100 * (Math.exp(d + h) - 1)];
    };
    const runs = (data.scaling && data.scaling.runs) || [];
    return {
      g, T, base: R[b], cpu: shortCpu(g.cpu),
      pts: placed.map((i) => {
        const rel = pos.get(R[i]);
        const prev = i > 0 && ok(i - 1) ? i - 1 : null;
        let direct = null;
        if (prev !== null) {
          // the pair measured in one run, if one was: its wall change, as throughput
          const r = g.runs.find((r) => r.new === R[i] && r.old === R[prev]);
          const wr = r && runs.find((w) => w.id === r.id);
          const c = wr && ((wr.corpora[g.corpus] || {}).change || {})[R[prev]];
          const e = c && c[T];
          // (the older arm's wall against the newer's: the newer's throughput against the older's)
          if (e) direct = { run: r, c: [e[0], e[1], e[2]] };
        }
        return {
          i, version: R[i], rel, link: rel.link, c: vs(b, i), mb: [mb(i), mb(i, -k), mb(i, k)], n: P.n[i],
          step: prev !== null ? { from: R[prev], c: vs(prev, i) } : null, direct,
          runs: g.runs.filter((r) => r.new === R[i] || r.old === R[i]),
        };
      }),
    };
  }

  // A release's figures, as the rows of its card
  const releaseRows = (rs, p, tag, data) => {
    const t = rs.T;
    const rows = [
      [`tag v${p.version} at ${p.rel.sha.slice(0, 7)} (${p.rel.date.slice(0, 10)}), placed at ${tag(data.links[p.link])}, the last merge it contains${p.rel.unlinked ? `; ${p.rel.unlinked} decoder change(s) with no link between them` : ""}`],
      [`${p.mb[0].toFixed(0)} MB/s modelled (95%: ${p.mb[1].toFixed(0)} to ${p.mb[2].toFixed(0)}) at ${t} thread${t === "1" ? "" : "s"}`, true],
      [p.version === rs.base ? `the zero: the oldest release placed` : `${fmtPct(p.c[0])} since ${rs.base} (95%: ${fmtPct(p.c[1])} to ${fmtPct(p.c[2])})`],
    ];
    if (p.step) rows.push([`against ${p.step.from}: ${fmtPct(p.step.c[0])} (95%: ${fmtPct(p.step.c[1])} to ${fmtPct(p.step.c[2])}), modelled` + (p.direct ? `; ${fmtPct(p.direct.c[0])} (${fmtPct(p.direct.c[1])} to ${fmtPct(p.direct.c[2])}) measured directly in their run` : `; no run measured the pair directly`)]);
    rows.push([`from ${p.n} run${p.n === 1 ? "" : "s"} on the ${rs.cpu}: ${p.runs.map((r) => `${r.new} against ${r.old} (${r.date})`).join("; ")}`]);
    return rows;
  };

  // The releases' marks on a merge-axis plot: whiskers, diamonds, the dotted line between
  // them (not measured), the row of release names above the plot, and each one's card.
  // o: { x(p), y, v(p) (the figures y plots: [mid, lo, hi], p.c by default), y2 (the steps'
  // scale, or null), top, left, right, W, narrow, tip, box, readout (or null), tag, data }
  function drawReleases(s, rs, o) {
    const v = o.v || ((p) => p.c);
    const gr = svg("g", { class: "bh-i9" }, s);
    svg("polyline", { points: rs.pts.map((p) => `${o.x(p)},${o.y(v(p)[0])}`).join(" "), class: "bh-i9-line" }, gr);
    const dia = (cx, cy, r, parent, cls = "bh-i9-dot") => svg("path", { d: `M${cx},${cy - r}L${cx + r},${cy}L${cx},${cy + r}L${cx - r},${cy}Z`, class: cls }, parent);
    // the names above the plot, as many as fit, the newest always
    {
      const minGap = o.narrow ? 38 : 40, put = [];
      [...rs.pts].reverse().forEach((p, k) => {
        const xx = o.x(p);
        if (k && put.some((q) => Math.abs(q - xx) < minGap)) return;
        put.push(xx);
        const anchor = xx + 16 > o.right ? "end" : xx - 16 < o.left ? "start" : "middle";
        svg("text", { x: xx, y: o.top - 14, class: "bh-tick bh-i9-name", "text-anchor": anchor }, gr).textContent = p.version;
      });
      for (const p of rs.pts) svg("line", { x1: o.x(p), x2: o.x(p), y1: o.top - 10, y2: o.top - 4, class: "bh-i9-rug" }, gr);
    }
    const tip = o.tip;
    const card = (p, px, py) => {
      rnShow(p.version);
      const rows = releaseRows(rs, p, o.tag, o.data);
      if (o.readout) {
        o.readout.textContent = "";
        el("strong", { text: `${p.version} ` }, o.readout);
        o.readout.append(rows.map((r) => r[0]).join(" · "));
      }
      if (!tip) return;
      tip.textContent = "";
      el("div", { class: "bh-tip-title", text: `${p.version} · ${rs.cpu} · ${rs.T} thread${rs.T === "1" ? "" : "s"}` }, tip);
      for (const [text, on] of rows) {
        const d = el("div", { class: `bh-tip-row${on ? " is-on" : ""}` }, tip);
        if (on) el("span", { class: "bh-swatch", style: "background:var(--bh-i9)" }, d);
        el("span", { text }, d);
      }
      tip.hidden = false;
      const r = s.getBoundingClientRect(), hb = o.box.getBoundingClientRect(), z = r.width / o.W;
      const ax = r.left - hb.left + px * z, ay = r.top - hb.top + py * z;
      const left = ax + 14 + tip.offsetWidth <= hb.width ? ax + 14 : ax - 14 - tip.offsetWidth;
      tip.style.left = `${Math.max(0, Math.min(hb.width - tip.offsetWidth, left))}px`;
      tip.style.top = `${ay + 12}px`;
    };
    const hide = () => { if (tip) tip.hidden = true; };
    for (const p of rs.pts) {
      const xx = o.x(p), yy = o.y(v(p)[0]);
      const a = svg("g", { class: "bh-pt bh-i9-pt", tabindex: 0, role: "button", "aria-label": `${p.version} on the ${rs.cpu} at ${rs.T} threads: ${p.mb[0].toFixed(0)} MB/s modelled, ${fmtPct(p.c[0])} since ${rs.base}` }, gr);
      if (v(p)[2] - v(p)[1] > 0) svg("line", { x1: xx, x2: xx, y1: o.y(v(p)[1]), y2: o.y(v(p)[2]), class: "bh-whisker bh-i9-whisker" }, a);
      dia(xx, yy, 5, a);
      if (o.y2 && p.step) {
        svg("line", { x1: xx, x2: xx, y1: o.y2(p.step.c[1]), y2: o.y2(p.step.c[2]), class: "bh-whisker bh-i9-whisker" }, a);
        dia(xx, o.y2(p.step.c[0]), 4.5, a);
      }
      svg("circle", { cx: xx, cy: yy, r: 12, class: "bh-hit" }, a);
      if (o.y2 && p.step) svg("circle", { cx: xx, cy: o.y2(p.step.c[0]), r: 12, class: "bh-hit" }, a);
      const show = () => card(p, xx, yy);
      a.addEventListener("pointerenter", show);
      a.addEventListener("focus", show);
      a.addEventListener("click", (ev) => { ev.stopPropagation(); show(); });
      // (a touch's pointerleave follows its tap at once: the card stays until a tap elsewhere)
      a.addEventListener("pointerleave", (ev) => { if (ev.pointerType !== "touch") hide(); });
      a.addEventListener("blur", hide);
    }
    s.addEventListener("click", (ev) => { if (!ev.target.closest(".bh-i9-pt")) hide(); });
    if (o.readout) card(rs.pts[rs.pts.length - 1], 0, 0), hide();
  }

  // Every thread count of the release-scaling machine's releases on the merge axis, in one
  // chart: a series per thread count, each the change since the oldest release (the axis the
  // single-count views use). Thread count is ordinal, so its colour is its place along the
  // releases' ramp (bench.css --bh-rel-0..4, walked by OKLab arc length: Ok.ramp), low to
  // high, every series at full strength. Pointing at a series (its line, a point, its end
  // label or its key entry) raises it and fades the rest; a click or tap keeps it. Absolute
  // (abs): each release's modelled MB/s on that machine instead, so the series sit at their
  // own levels.
  const ovlFocus = { pinned: null, hover: null };
  function releaseOverlay(data, g, start, ts, box, abs = false) {
    const all = ts.map((t) => releaseSeries(data, g, start, t)).filter(Boolean);
    const cpu = shortCpu(g.cpu), base = all[0].base, newest = all[0].pts[all[0].pts.length - 1].version;
    const tw = (t) => `${t} thread${t === "1" ? "" : "s"}`;
    // a release's figures on the axis: [mid, lo, hi], change since the base or MB/s
    const val = (p) => (abs ? p.mb : p.c);
    const fmtV = abs ? fmtMB : fmtPct;
    const lead = el("p", { class: "bh-hero-focus" }, box);
    if (abs) {
      el("strong", { text: `Throughput, ${g.corpus}, ${cpu} releases at every thread count: MB/s of output on the ${cpu}, modelled` }, lead);
      lead.append(`. At ${newest}: ` + all.map((r) => `${tw(r.T)} ${fmtMB(r.pts[r.pts.length - 1].mb[0])}`).join(", ") + ` MB/s. One machine, one scale. CI's chain is not drawn here. Up is faster.`);
    } else {
      el("strong", { text: `Throughput, ${g.corpus}, ${cpu} releases at every thread count: change since ${base}, modelled` }, lead);
      lead.append(`. ${newest} against ${base}: ` + all.map((r) => `${tw(r.T)} ${fmtPct(r.pts[r.pts.length - 1].c[0])}`).join(", ") + `. CI's chain is not drawn here. Up is faster.`);
    }
    const body = el("div", { class: "bh-hero-chart" }, box);
    const legend = el("p", { class: "bh-hero-commit" }, box);
    const readout = el("p", { class: "bh-hero-commit", "aria-live": "polite" }, box);
    const tip = el("div", { class: "bh-tip", hidden: "" }, box);
    const tag = (l) => (l.pr ? `#${l.pr}` : l.sha.slice(0, 7));
    const idx = [];
    for (let i = start; i < data.links.length; i++) idx.push(i);
    const W = Math.max(300, Math.round(body.clientWidth) || 700);
    const narrow = W < 560;
    const H = narrow ? 300 : 360;
    const vals = all.flatMap((r) => r.pts.flatMap((p) => [val(p)[1], val(p)[2]]));
    // (relative: from zero; absolute: the levels' own range, from the lowest)
    const z = abs ? [] : [0];
    const yt = ticks(Math.min(...z, ...vals), Math.max(...z, ...vals), narrow ? 4 : 5);
    while (yt[yt.length - 1] < Math.max(...vals)) yt.push(+(yt[yt.length - 1] + (yt[1] - yt[0])).toFixed(10));
    while (yt[0] > Math.min(...vals)) yt.unshift(+(yt[0] - (yt[1] - yt[0])).toFixed(10));
    const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, role: "img", "aria-label": abs ? `The ${cpu}'s releases at ${ts.join(", ")} threads: throughput in MB/s of output on the ${cpu}, modelled, by merge` : `The ${cpu}'s releases at ${ts.join(", ")} threads: throughput change since ${base}, modelled, by merge` }, body);
    const meas = svg("text", { class: "bh-tick", x: 0, y: -50 }, s);
    const textW = (t) => { meas.textContent = t; return meas.getComputedTextLength(); };
    const M = { l: Math.ceil(Math.max(24, abs ? Math.max(...yt.map((v) => textW(fmtMB(v)))) : textW("+100%"))) + 12, r: 36, t: abs ? 46 : 30, b: 26 };
    meas.remove();
    const inner = W - M.l - M.r;
    const xs = new Map(idx.map((i, k) => [i, M.l + (idx.length === 1 ? inner / 2 : 8 + (k * (inner - 16)) / (idx.length - 1))]));
    const x = (p) => xs.get(p.link);
    const y = (v) => M.t + (H - M.t - M.b) * (1 - (v - yt[0]) / (yt[yt.length - 1] - yt[0] || 1));
    for (const v of yt) {
      svg("line", { x1: M.l, x2: W - M.r, y1: y(v), y2: y(v), class: v === 0 && !abs ? "bh-zero" : "bh-grid" }, s);
      svg("text", { x: M.l - 6, y: y(v) + 4, class: "bh-tick", "text-anchor": "end" }, s).textContent = fmtV(v);
    }
    if (abs) svg("text", { x: M.l, y: 12, class: "bh-unit" }, s).textContent = `MB/s of output on the ${cpu}, modelled`;
    // x labels: as many merges as fit, the last always shown
    const minGap = narrow ? 40 : 46, put = [];
    idx.forEach((i, k) => {
      const xx = xs.get(i), last = k === idx.length - 1;
      if (!last && put.length && xx - put[put.length - 1].x < minGap) return;
      if (last) while (put.length && xx - put[put.length - 1].x < minGap) put.pop().el.remove();
      const t = svg("text", { x: xx, y: H - 8, class: "bh-tick", "text-anchor": k === 0 ? "start" : last ? "end" : "middle" }, s);
      t.textContent = tag(data.links[i]);
      put.push({ x: xx, el: t });
    });
    // the releases' names above the plot, as many as fit, the newest always
    const names = [];
    [...all[0].pts].reverse().forEach((p, k) => {
      const xx = x(p);
      svg("line", { x1: xx, x2: xx, y1: M.t - 10, y2: M.t - 4, class: "bh-ovl-rug" }, s);
      if (k && names.some((q) => Math.abs(q - xx) < (narrow ? 38 : 40))) return;
      names.push(xx);
      svg("text", { x: xx, y: M.t - 14, class: "bh-tick", "text-anchor": xx + 16 > W - M.r ? "end" : xx - 16 < M.l ? "start" : "middle" }, s).textContent = p.version;
    });
    // the colours: the releases' ramp, low threads to high, at equal perceived steps
    const cs = getComputedStyle(box), cv = (k) => cs.getPropertyValue(k).trim();
    const full = Ok.ramp([0, 1, 2, 3, 4].map((k) => cv(`--bh-rel-${k}`)));
    const n = all.length;
    const col = all.map((_, j) => (n < 2 ? full(1) : full(j / (n - 1))));
    const groups = all.map((rs, j) => svg("g", { class: "bh-rel bh-ovl", "data-t": rs.T, style: `--rc:${col[j]};--rf:${col[j]}` }, s));
    const card = (rs, j, p) => {
      rnShow(p.version);
      const rows = releaseRows(rs, p, tag, data);
      readout.textContent = "";
      el("strong", { text: `${tw(rs.T)}, ${p.version} ` }, readout);
      readout.append(rows.map((r) => r[0]).join(" · "));
      tip.textContent = "";
      el("div", { class: "bh-tip-title", text: `${tw(rs.T)} · ${p.version} · ${cpu}` }, tip);
      for (const [text, on] of rows) {
        const d = el("div", { class: `bh-tip-row${on ? " is-on" : ""}` }, tip);
        if (on) el("span", { class: "bh-swatch", style: `background:${col[j]}` }, d);
        el("span", { text }, d);
      }
      tip.hidden = false;
      const r = s.getBoundingClientRect(), hb = box.getBoundingClientRect(), z = r.width / W;
      const ax = r.left - hb.left + x(p) * z, ay = r.top - hb.top + y(val(p)[0]) * z;
      const left = ax + 14 + tip.offsetWidth <= hb.width ? ax + 14 : ax - 14 - tip.offsetWidth;
      tip.style.left = `${Math.max(0, Math.min(hb.width - tip.offsetWidth, left))}px`;
      tip.style.top = `${ay + 12}px`;
    };
    const hideTip = () => { tip.hidden = true; };
    const paint = () => {
      const key = ovlFocus.pinned !== null && ts.includes(ovlFocus.pinned) ? ovlFocus.pinned : ovlFocus.hover;
      groups.forEach((gr, j) => {
        gr.classList.toggle("is-dim", key !== null && all[j].T !== key);
        gr.classList.toggle("is-on", all[j].T === key);
      });
      if (key !== null && all.some((r) => r.T === key)) raise.setAttribute("href", `#bh-ovl-${key}`);
      else raise.removeAttribute("href");
      for (const b of legend.querySelectorAll(".bh-ws-item")) {
        b.classList.toggle("is-focus", b.dataset.key === key);
        b.setAttribute("aria-pressed", String(b.dataset.key === ovlFocus.pinned));
      }
    };
    const enter = (j) => { ovlFocus.hover = all[j].T; paint(); };
    const leave = (j) => { if (ovlFocus.hover === all[j].T) { ovlFocus.hover = null; paint(); } };
    const pin = (j) => { ovlFocus.pinned = ovlFocus.pinned === all[j].T ? null : all[j].T; ovlFocus.hover = all[j].T; paint(); };
    all.forEach((rs, j) => {
      const gr = groups[j];
      gr.id = `bh-ovl-${rs.T}`;
      const line = rs.pts.map((p) => `${x(p)},${y(val(p)[0])}`).join(" ");
      svg("polyline", { points: line, class: "bh-line", fill: "none" }, gr);
      const lh = svg("polyline", { points: line, class: "bh-line-hit", fill: "none" }, gr);
      const nearest = (ev) => {
        const r = s.getBoundingClientRect(), px = ((ev.clientX - r.left) * W) / r.width;
        return rs.pts.reduce((b, p) => (Math.abs(x(p) - px) < Math.abs(x(b) - px) ? p : b), rs.pts[0]);
      };
      lh.addEventListener("pointerenter", () => enter(j));
      lh.addEventListener("pointermove", (ev) => card(rs, j, nearest(ev)));
      lh.addEventListener("pointerleave", (ev) => { if (ev.pointerType !== "touch") { leave(j); hideTip(); } });
      lh.addEventListener("click", (ev) => { pin(j); card(rs, j, nearest(ev)); });
      for (const p of rs.pts) {
        const a = svg("g", { class: "bh-pt", tabindex: 0, role: "button", "aria-label": abs ? `${tw(rs.T)}, ${p.version}: ${fmtMB(p.mb[0])} MB/s on the ${cpu}, modelled` : `${tw(rs.T)}, ${p.version}: ${fmtPct(p.c[0])} since ${base}, modelled` }, gr);
        const [m, lo, hi] = val(p);
        if (hi - lo > 0) svg("line", { x1: x(p), x2: x(p), y1: y(lo), y2: y(hi), class: "bh-whisker" }, a);
        svg("circle", { cx: x(p), cy: y(m), r: 3, class: "bh-dot" }, a);
        svg("circle", { cx: x(p), cy: y(m), r: 9, class: "bh-hit" }, a);
        const show = () => { enter(j); card(rs, j, p); };
        a.addEventListener("pointerenter", show);
        a.addEventListener("focus", show);
        // (a touch's pointerleave follows its tap at once: the tap pins, and the card stays)
        a.addEventListener("pointerleave", (ev) => { if (ev.pointerType !== "touch") { leave(j); hideTip(); } });
        a.addEventListener("blur", () => { leave(j); hideTip(); });
        a.addEventListener("click", (ev) => { ev.stopPropagation(); pin(j); card(rs, j, p); });
        a.addEventListener("keydown", (ev) => { if (ev.key === "Enter" || ev.key === " ") { ev.preventDefault(); pin(j); } });
      }
    });
    s.addEventListener("click", (ev) => { if (!ev.target.closest(".bh-pt, .bh-line-hit")) hideTip(); });
    // the series in focus, drawn again on top of the others
    const raise = svg("use", { class: "bh-raise", "aria-hidden": "true" }, s);
    // direct labels at the right-hand end, nudged apart so neighbours never collide
    const ends = all.map((rs, j) => {
      const p = rs.pts[rs.pts.length - 1];
      return { j, x0: x(p), y0: y(val(p)[0]), y: y(val(p)[0]) };
    }).sort((a, b) => a.y0 - b.y0);
    const gap = 11;
    for (let q = 1; q < ends.length; q++) ends[q].y = Math.max(ends[q].y, ends[q - 1].y + gap);
    const over = ends.length ? ends[ends.length - 1].y - (H - M.b) : 0;
    if (over > 0) for (const e of ends) e.y -= over;
    for (let q = ends.length - 2; q >= 0; q--) ends[q].y = Math.min(ends[q].y, ends[q + 1].y - gap);
    for (const e of ends) {
      const lg = svg("g", { class: "bh-end" }, groups[e.j]);
      svg("line", { x1: e.x0 + 4, x2: e.x0 + 10, y1: e.y0, y2: e.y, class: "bh-leader" }, lg);
      svg("text", { x: e.x0 + 12, y: e.y + 4, class: "bh-tick bh-end-label" }, lg).textContent = `${all[e.j].T}t`;
      lg.addEventListener("pointerenter", () => enter(e.j));
      lg.addEventListener("pointerleave", () => leave(e.j));
      lg.addEventListener("click", () => pin(e.j));
    }
    // the key, in thread order: pointing or focusing raises a series, a click or tap keeps it
    all.forEach((rs, j) => {
      const b = el("button", { type: "button", class: "bh-ws-key bh-ws-item", "aria-pressed": "false", "data-key": rs.T, style: `--rc:${col[j]};--rf:${col[j]}` }, legend);
      el("span", { class: "bh-swatch" }, b);
      el("span", { text: tw(rs.T) }, b);
      b.addEventListener("click", () => pin(j));
      b.addEventListener("pointerenter", () => enter(j));
      b.addEventListener("focus", () => enter(j));
      b.addEventListener("pointerleave", () => leave(j));
      b.addEventListener("blur", () => leave(j));
    });
    const kd = el("span", { class: "bh-ws-key bh-ovl-key" }, legend);
    el("span", { class: "bh-swatch is-dotted is-ink" }, kd);
    kd.append(` ${cpu}, releases (modelled; not measured between releases)`);
    paint();
    readout.textContent = "Point at, tap or Tab to a release for its figures; point at a thread count (its line, end label or key entry) to raise it, and click or tap to keep it.";
    el("p", { class: "bh-hero-note", text: `Each series: one thread count, the ${cpu}'s releases at the last merge each tag contains, from ${tag(data.links[idx[0]])} to ${tag(data.links[idx[idx.length - 1]])}, its throughput modelled from release-against-release runs (the Release scaling view) ${abs ? `in MB/s of output on the ${cpu}, lbzip2's MB/s measured in the newest run, scaled by each release's pooled wall ratio to lbzip2; every series on that one machine's scale` : `relative to ${base}`}; the series in focus shows its 95% whiskers, ${abs ? "the same intervals in MB/s" : "its change's intervals"}. The dotted lines only join releases: modelled, and not measured between releases.` }, box);
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
  function releaseScaling(ws, box, seg, lin) {
    // A newly recorded run (ws[0], the newest) is shown first, whatever run a reader last
    // picked: a remembered choice only holds until a newer release's run arrives.
    if (state.wsNewest !== ws[0].id) {
      state.wsNewest = ws[0].id;
      state.wsRun = ws[0].id;
    }
    // "all": every release against every other, inferred from the runs (lineage)
    if (!(state.wsRun === "all" && lin.length) && !ws.some((r) => r.id === state.wsRun)) state.wsRun = ws[0].id;
    // Each run is named by its releases, so runs on one day and machine can be told apart
    const name = (r) => {
      const older = r.arms.filter((a) => a !== r.ref && /^\d+\.\d+\.\d+$/.test(a));
      return `${r.ref}${older.length ? " against " + older.join(", ") : ""} · ${r.date} · ${r.cpu}`;
    };
    const runOpts = ws.map((r) => [r.id, name(r)]);
    if (lin.length) runOpts.unshift(["all", `All releases (${lin[0].releases[0]} to ${lin[0].releases[lin[0].releases.length - 1]}, inferred)`]);
    if (runOpts.length > 1) seg("Run", runOpts, state.wsRun, (v) => (state.wsRun = v));
    if (state.wsRun === "all") { releaseLineage(ws, lin, box, seg); return; }
    const run = ws.find((r) => r.id === state.wsRun);
    rnShow(run.ref);
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

  // The hero's All releases view: every release against every other, inferred from the
  // pairwise runs (bench_release_scaling.py, lineage()). Per thread count, each release's log
  // wall time relative to lbzip2 (in every run, the fixed anchor) is pooled over the runs it
  // appears in; two releases compare as the difference of their figures. The curves put each
  // release on the newest run's measured lbzip2 throughput: the shape is measured, each
  // release's level relative to lbzip2 is modelled. The matrix is release against release at
  // one thread count.
  // the All releases view's isolated release: pinned by a click or tap, kept across redraws
  const linFocus = { pinned: null, hover: null };
  const RECENT = 5; // the newest releases drawn at full strength; the older ones muted
  function releaseLineage(ws, lin, box, seg) {
    const corpora = [...new Set(lin.map((g) => g.corpus))].sort((a, b) => (a !== "wikidata") - (b !== "wikidata") || a.localeCompare(b));
    if (!corpora.includes(state.wsCorpus)) state.wsCorpus = corpora[0];
    if (corpora.length > 1) seg("Corpus", corpora.map((c) => [c, c]), state.wsCorpus, (v) => (state.wsCorpus = v));
    const g = lin.find((l) => l.corpus === state.wsCorpus);
    const R = g.releases, ts = g.threads.map(String), n = R.length;
    linFocus.hover = null;
    if (!ts.includes(state.wsLinT)) state.wsLinT = ts[ts.length - 1];
    seg("Matrix at", ts.map((t) => [t, `${t} thread${t === "1" ? "" : "s"}`]), state.wsLinT, (v) => (state.wsLinT = v));
    const P = g.per_thread, k = g.t975;
    const ok = (t, i) => P[t].x[i] !== null && P[t].x[i] !== undefined;
    // modelled MB/s: lbzip2's measured MB/s in the newest run over the release's fitted wall ratio
    const mb = (t, i, z = 0) => P[t].anchor_mbps / Math.exp(P[t].x[i] - z * Math.sqrt(P[t].v[i]));
    const T = state.wsLinT, oi = 0, ni = n - 1;
    const lead = el("p", { class: "bh-hero-focus" }, box);
    const body = el("div", { class: "bh-hero-chart" }, box);
    const legend = el("p", { class: "bh-hero-commit" }, box);
    const note = el("p", { class: "bh-hero-commit" }, box);
    // the headline: the newest release against the oldest, both ways, at the chosen threads
    const p = P[T];
    const dx = p.x[oi] - p.x[ni], hw = k * Math.sqrt(p.v[oi] + p.v[ni]);
    const ch = p.chain[ni] !== null ? p.chain[ni] - p.chain[oi] : null;
    el("strong", { text: `${R[ni]} against ${R[oi]}, ${state.wsCorpus} on ${g.cpu} at ${T} thread${T === "1" ? "" : "s"}: ${fmtPct(100 * (Math.exp(dx) - 1))} throughput` }, lead);
    lead.append(` (95%: ${fmtPct(100 * (Math.exp(dx - hw) - 1))} to ${fmtPct(100 * (Math.exp(dx + hw) - 1))}), through ${g.anchor} as the anchor` +
      (ch !== null ? `; ${fmtPct(100 * (Math.exp(ch) - 1))} through the chain of the ${g.runs.length} runs' direct pairs.` : "."));
    // the curves
    const W = Math.max(300, Math.round(body.clientWidth) || 700);
    const narrow = W < 560;
    const H = narrow ? 260 : 320, M = { l: 52, r: 46, t: 16, b: 36 };
    const vals = [];
    for (const t of ts) {
      if (P[t].anchor_mbps) vals.push(P[t].anchor_mbps);
      R.forEach((_, i) => { if (ok(t, i) && P[t].anchor_mbps) vals.push(mb(t, i, k)); });
    }
    const yt = ticks(0, Math.max(...vals), narrow ? 4 : 5);
    while (yt[yt.length - 1] < Math.max(...vals)) yt.push(+(yt[yt.length - 1] + (yt[1] - yt[0])).toFixed(10));
    const y = (v) => M.t + (H - M.t - M.b) * (1 - v / yt[yt.length - 1]);
    const x = (i) => M.l + (ts.length === 1 ? (W - M.l - M.r) / 2 : (i * (W - M.l - M.r)) / (ts.length - 1));
    const s = svg("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, role: "img", "aria-label": `Modelled throughput of ${state.wsCorpus} on ${g.cpu} against threads, for releases ${R.join(", ")}, and ${g.anchor} as measured` }, body);
    for (const v of yt) {
      svg("line", { x1: M.l, x2: W - M.r, y1: y(v), y2: y(v), class: "bh-grid" }, s);
      svg("text", { x: M.l - 6, y: y(v) + 4, class: "bh-tick", "text-anchor": "end" }, s).textContent = v.toLocaleString();
    }
    svg("text", { x: M.l, y: M.t - 4, class: "bh-unit" }, s).textContent = `MB/s of output, on ${g.anchor}'s scale in the newest run`;
    ts.forEach((t, i) => {
      svg("text", { x: x(i), y: H - M.b + 16, class: "bh-tick", "text-anchor": "middle" }, s).textContent = t;
    });
    svg("text", { x: (M.l + W - M.r) / 2, y: H - 4, class: "bh-tick", "text-anchor": "middle" }, s).textContent = "threads";
    // the matrix's thread count
    svg("line", { x1: x(ts.indexOf(T)), x2: x(ts.indexOf(T)), y1: M.t, y2: H - M.b, class: "bh-grid bh-guide" }, s);
    // lbzip2, measured: dashed and muted
    const lp = ts.map((t, i) => [x(i), P[t].anchor_mbps]).filter(([, v]) => v);
    svg("polyline", { points: lp.map(([px, v]) => `${px},${y(v)}`).join(" "), class: "bh-line bh-anchor", stroke: "var(--bh-old)", fill: "none" }, s);
    // Focus and context. The newest RECENT releases take the full ramp (bench.css: the five
    // stops --bh-rel-0..4, their own per scheme), spread over its whole length by perceived
    // distance, so neighbours sit as far apart as the ramp allows; the older ones a muted
    // band, still in order (lightness), thin and low in chroma. Pointing at, focusing or
    // pinning any release shows it at full strength: an older one takes the full ramp's
    // colour at its own place among all the releases.
    const cs = getComputedStyle(box), cv = (k) => cs.getPropertyValue(k).trim();
    const full = Ok.ramp([0, 1, 2, 3, 4].map((k) => cv(`--bh-rel-${k}`)));
    const recent = Math.min(RECENT, n), old = n - recent;
    const muteL = [+cv("--bh-rel-mute-l0"), +cv("--bh-rel-mute-l1")], muteC = +cv("--bh-rel-mute-c");
    const strong = R.map((_, i) => (n < 2 ? full(1) : full(i / (n - 1))));
    const base = R.map((_, i) => {
      if (i >= old) return recent < 2 ? full(1) : full((i - old) / (recent - 1));
      const f = old < 2 ? 1 : i / (old - 1);
      return Ok.hex([muteL[0] + (muteL[1] - muteL[0]) * f, muteC, Ok.lch(strong[i])[2]]);
    });
    const hue = (i) => `--rc:${base[i]};--rf:${i >= old ? base[i] : strong[i]}`;
    // each release's marks in one group: drawn oldest first, so the newest is on top
    const tip = el("div", { class: "bh-tip", hidden: "" }, box);
    const groups = R.map((r, i) => svg("g", { class: `bh-rel ${i >= old ? "is-recent" : "is-muted"}${i === ni ? " is-newest" : ""}`, "data-i": i, style: hue(i) }, s));
    const runsOf = (i) => g.runs.filter((r) => r.new === R[i] || r.old === R[i]);
    const showTip = (i, t, px, py) => {
      tip.textContent = "";
      el("div", { class: "bh-tip-title", text: `${R[i]} · ${t} thread${t === "1" ? "" : "s"}` }, tip);
      const row = (text, on) => {
        const d = el("div", { class: `bh-tip-row${on ? " is-on" : ""}` }, tip);
        if (on) el("span", { class: "bh-swatch", style: `background:${i >= old ? base[i] : strong[i]}` }, d);
        el("span", { text }, d);
      };
      row(`${mb(t, i).toFixed(0)} MB/s modelled (95%: ${mb(t, i, -k).toFixed(0)} to ${mb(t, i, k).toFixed(0)})`, true);
      row(`wall ${Math.exp(P[t].x[i]).toFixed(3)}× ${g.anchor}'s`);
      row(`from ${P[t].n[i]} run${P[t].n[i] === 1 ? "" : "s"}: ${runsOf(i).map((r) => `${r.new} against ${r.old} (${r.date})`).join("; ")}`);
      const nr = g.runs.find((r) => r.id === g.newest_run);
      row(`on ${g.anchor}'s ${P[t].anchor_mbps.toFixed(0)} MB/s in the newest run${nr ? ` (${nr.new} against ${nr.old}, ${nr.date})` : ""}`);
      tip.hidden = false;
      const r = s.getBoundingClientRect(), hb = box.getBoundingClientRect(), z = r.width / W;
      const ax = r.left - hb.left + px * z, ay = r.top - hb.top + py * z;
      const left = ax + 14 + tip.offsetWidth <= hb.width ? ax + 14 : ax - 14 - tip.offsetWidth;
      tip.style.left = `${Math.max(0, Math.min(hb.width - tip.offsetWidth, left))}px`;
      tip.style.top = `${ay + 12}px`;
    };
    const hideTip = () => { tip.hidden = true; };
    // isolation: pointing at or focusing a release fades the rest; a click or tap on its
    // legend entry or a point keeps it, until clicked again or another is
    const paint = () => {
      const key = linFocus.pinned !== null && R.includes(linFocus.pinned) ? linFocus.pinned : linFocus.hover;
      groups.forEach((gr, i) => {
        gr.classList.toggle("is-dim", key !== null && R[i] !== key);
        gr.classList.toggle("is-on", R[i] === key);
      });
      if (key !== null && R.includes(key)) raise.setAttribute("href", `#bh-rel-${R.indexOf(key)}`);
      else raise.removeAttribute("href");
      // the release in focus, else the newest, in the notes under the chart
      rnShow(key !== null && R.includes(key) ? key : R[R.length - 1]);
      for (const b of legend.querySelectorAll(".bh-ws-item")) {
        b.classList.toggle("is-focus", b.dataset.key === key);
        b.setAttribute("aria-pressed", String(b.dataset.key === linFocus.pinned));
      }
    };
    const enter = (i) => { linFocus.hover = R[i]; paint(); };
    const leave = (i) => { if (linFocus.hover === R[i]) { linFocus.hover = null; paint(); } };
    const pin = (i) => { linFocus.pinned = linFocus.pinned === R[i] ? null : R[i]; linFocus.hover = R[i]; paint(); };
    R.forEach((_, i) => {
      const gr = groups[i];
      gr.id = `bh-rel-${i}`;
      const pts = ts.map((t, j) => [j, t]).filter(([, t]) => ok(t, i) && P[t].anchor_mbps);
      const line = pts.map(([j, t]) => `${x(j)},${y(mb(t, i))}`).join(" ");
      svg("polyline", { points: line, class: "bh-line", fill: "none" }, gr);
      // a wide, invisible stroke along the line: pointing near it shows its nearest point
      const lh = svg("polyline", { points: line, class: "bh-line-hit", fill: "none" }, gr);
      const nearest = (ev) => {
        const r = s.getBoundingClientRect(), px = ((ev.clientX - r.left) * W) / r.width;
        return pts.reduce((b, p) => (Math.abs(x(p[0]) - px) < Math.abs(x(b[0]) - px) ? p : b), pts[0]);
      };
      lh.addEventListener("pointerenter", () => enter(i));
      const near = (ev) => { const [j, t] = nearest(ev); showTip(i, t, x(j), y(mb(t, i))); };
      lh.addEventListener("pointermove", near);
      lh.addEventListener("pointerleave", (ev) => { if (ev.pointerType !== "touch") { leave(i); hideTip(); } });
      lh.addEventListener("click", (ev) => { pin(i); near(ev); });
      for (const [j, t] of pts) {
        const a = svg("g", { class: "bh-pt", tabindex: 0, role: "button", "aria-label": `${R[i]} at ${t} thread${t === "1" ? "" : "s"}: ${mb(t, i).toFixed(0)} MB/s modelled` }, gr);
        svg("line", { x1: x(j), x2: x(j), y1: y(mb(t, i, k)), y2: y(mb(t, i, -k)), class: "bh-whisker" }, a);
        svg("circle", { cx: x(j), cy: y(mb(t, i)), r: 3, class: "bh-dot" }, a);
        svg("circle", { cx: x(j), cy: y(mb(t, i)), r: 10, class: "bh-hit" }, a);
        const show = () => { enter(i); showTip(i, t, x(j), y(mb(t, i))); };
        a.addEventListener("pointerenter", show);
        a.addEventListener("focus", show);
        // (a touch's pointerleave follows its tap at once: the tap pins, and the card stays)
        a.addEventListener("pointerleave", (ev) => { if (ev.pointerType !== "touch") { leave(i); hideTip(); } });
        a.addEventListener("blur", () => { leave(i); hideTip(); });
        a.addEventListener("click", () => { pin(i); showTip(i, t, x(j), y(mb(t, i))); });
        a.addEventListener("keydown", (ev) => { if (ev.key === "Enter" || ev.key === " ") { ev.preventDefault(); pin(i); } });
      }
    });
    // a tap or click away from every point puts the card away
    s.addEventListener("click", (ev) => { if (!ev.target.closest(".bh-pt, .bh-line-hit")) hideTip(); });
    // the release in focus, drawn again on top of the others (a copy: its marks keep their
    // place, so focus is never lost)
    const raise = svg("use", { class: "bh-raise", "aria-hidden": "true" }, s);
    // direct labels at the right-hand end, nudged apart so neighbours never collide; a leader
    // in the release's colour ties each label to its line
    const ends = R.map((r, i) => {
      const pts = ts.map((t, j) => [j, t]).filter(([, t]) => ok(t, i) && P[t].anchor_mbps);
      if (!pts.length) return null;
      const [j, t] = pts[pts.length - 1];
      return { r, i, x0: x(j), y0: y(mb(t, i)), y: y(mb(t, i)) };
    }).filter(Boolean).sort((a, b) => a.y0 - b.y0);
    const gap = 11;
    for (let q = 1; q < ends.length; q++) ends[q].y = Math.max(ends[q].y, ends[q - 1].y + gap);
    const over = ends.length ? ends[ends.length - 1].y - (H - M.b) : 0;
    if (over > 0) for (const e of ends) e.y -= over;
    for (let q = ends.length - 2; q >= 0; q--) ends[q].y = Math.min(ends[q].y, ends[q + 1].y - gap);
    for (const e of ends) {
      const lg = svg("g", { class: "bh-end" }, groups[e.i]);
      svg("line", { x1: e.x0 + 4, x2: e.x0 + 12, y1: e.y0, y2: e.y, class: "bh-leader" }, lg);
      svg("text", { x: e.x0 + 14, y: e.y + 4, class: `bh-tick bh-end-label${e.i === ni ? " is-newest" : ""}` }, lg).textContent = e.r;
      lg.addEventListener("pointerenter", () => enter(e.i));
      lg.addEventListener("pointerleave", () => leave(e.i));
      lg.addEventListener("click", () => pin(e.i));
    }
    // the key in version order, oldest first, as the ramp runs: each entry a button, pointing
    // or focusing isolates its release, a click or tap keeps it (pressed)
    R.forEach((r, i) => {
      const b = el("button", { type: "button", class: `bh-ws-key bh-ws-item${i >= old ? "" : " is-muted"}${i === ni ? " is-newest" : ""}`, "aria-pressed": "false", "data-key": r, style: hue(i) }, legend);
      el("span", { class: "bh-swatch" }, b);
      el("span", { text: r }, b);
      b.addEventListener("click", () => pin(i));
      b.addEventListener("pointerenter", () => enter(i));
      b.addEventListener("focus", () => enter(i));
      b.addEventListener("pointerleave", () => leave(i));
      b.addEventListener("blur", () => leave(i));
    });
    paint();
    const item = el("span", { class: "bh-ws-key" }, legend);
    el("span", { class: "bh-swatch is-dashed", style: "background:var(--bh-old)" }, item);
    item.append(` ${g.anchor} (measured)`);
    // the drift diagnostic, at the chosen threads and over every thread count
    const worst = (cl) => cl.reduce((b, c) => (!b || Math.abs(c.d) > Math.abs(b.d) ? c : b), null);
    const wT = worst(p.closure);
    const wAll = worst(ts.flatMap((t) => P[t].closure.map((c) => ({ ...c, t }))));
    const drift = (c) => `${c.release} ${fmtPct(100 * (Math.exp(c.d) - 1))} (${(Math.abs(c.d) / c.se).toFixed(1)} standard errors)`;
    note.textContent =
      `Modelled from ${g.runs.length} paired runs on one machine (${g.runs[0].date} to ${g.runs[g.runs.length - 1].date}), not one run of every release. ` +
      `Measured: ${g.anchor}'s MB/s in the newest run (dashed), and each release's wall time against ${g.anchor} in the runs it was in. ` +
      `Modelled: each release's curve, its pooled ratio to ${g.anchor} applied to that MB/s; whiskers are 95% intervals of the ratio alone. ` +
      (wT ? `Drift, a release against itself in its two runs (the later less the earlier): largest at ${T} thread${T === "1" ? "" : "s"} ${drift(wT)}` +
        (wAll && wAll.t !== T ? `; largest at any thread count ${drift(wAll)} at ${wAll.t} thread${wAll.t === "1" ? "" : "s"}.` : ", the largest at any thread count.") : "") +
      " Point at, tap or Tab to a point for its figures; point at a release (its line, end label or key entry) to pick it out, and click or tap to keep it.";
    kneeTable(g, box);
    // the matrix: row against column, newest first, throughput change, at the chosen threads
    el("p", { class: "bh-hero-commit", text: `Throughput of the row's release against the column's at ${T} thread${T === "1" ? "" : "s"}, with its 95% interval. Coloured where the interval excludes zero: blue faster, orange slower; a ring marks a pair measured directly in one run.` }, box);
    const order = R.map((_, i) => n - 1 - i);
    const tbl = el("table", { class: "bh-matrix" }, el("div", { class: "bh-scroll" }, box));
    const hr = el("tr", {}, el("thead", {}, tbl));
    el("th", { text: "row ÷ column", scope: "col" }, hr);
    for (const j of order) el("th", { text: R[j], scope: "col" }, hr);
    const tb = el("tbody", {}, tbl);
    const direct = new Set(g.runs.map((r) => `${r.new}|${r.old}`));
    const maxL = Math.max(1e-9, ...R.flatMap((_, i) => R.map((_, j) => (ok(T, i) && ok(T, j) ? Math.abs(p.x[i] - p.x[j]) : 0))));
    for (const i of order) {
      const tr = el("tr", {}, tb);
      el("th", { text: R[i], scope: "row" }, tr);
      for (const j of order) {
        const td = el("td", {}, tr);
        if (i === j) { td.textContent = "·"; td.className = "is-self"; continue; }
        if (!ok(T, i) || !ok(T, j)) { td.textContent = "–"; continue; }
        const d = p.x[j] - p.x[i], h = k * Math.sqrt(p.v[i] + p.v[j]);
        const sig = d - h > 0 || d + h < 0;
        const share = Math.round(12 + 48 * Math.min(1, Math.abs(d) / maxL));
        if (sig) td.style.background = `color-mix(in srgb, var(${d > 0 ? "--bh-s1" : "--bh-s2"}) ${share}%, transparent)`;
        if (direct.has(`${R[i]}|${R[j]}`) || direct.has(`${R[j]}|${R[i]}`)) td.classList.add("is-direct");
        el("span", { text: fmtPct(100 * (Math.exp(d) - 1)) }, td);
        el("small", { text: `${fmtPct(100 * (Math.exp(d - h) - 1))} to ${fmtPct(100 * (Math.exp(d + h) - 1))}` }, td);
        td.title = `${R[i]} against ${R[j]} at ${T} threads: wall ${Math.exp(-d).toFixed(3)}× (modelled through ${g.anchor})`;
      }
    }
  }

  // The knee: per release, newest first, the thread count of its highest modelled throughput,
  // and that peak against the release's own 8- and 10-thread figures. A strip shades each
  // thread count by its share of the release's peak (from 80% up), the knee ringed. Modelled,
  // as the curves are: lbzip2's measured MB/s in the newest run over each pooled wall ratio.
  function kneeTable(g, box) {
    const R = g.releases, ts = g.threads.map(String), P = g.per_thread, k = g.t975;
    const ok = (t, i) => P[t] && P[t].anchor_mbps && P[t].x[i] !== null && P[t].x[i] !== undefined;
    const mb = (t, i) => P[t].anchor_mbps / Math.exp(P[t].x[i]);
    el("p", { class: "bh-hero-commit", text: `The knee, modelled: each release's thread count of highest throughput on the ${shortCpu(g.cpu)}, newest first, and its peak against its own 8- and 10-thread figures (95%). The strip shades each thread count by its share of that release's peak, from 80% (blank) to 100%; the ring marks the knee. The intervals are the releases' ratios to lbzip2 alone, not lbzip2's own MB/s at each thread count.` }, box);
    const tbl = el("table", { class: "bh-matrix bh-knee" }, el("div", { class: "bh-scroll" }, box));
    const hr = el("tr", {}, el("thead", {}, tbl));
    el("th", { text: "release", scope: "col" }, hr);
    for (const t of ts) el("th", { text: t, scope: "col", class: "bh-knee-t" }, hr);
    for (const h of ["knee", "peak MB/s", "peak ÷ 8", "peak ÷ 10"]) el("th", { text: h, scope: "col" }, hr);
    const tb = el("tbody", {}, tbl);
    const vs = (a, b, i) => {
      if (a === b) return "the knee";
      if (!ok(b, i)) return "–";
      const d = Math.log(mb(a, i) / mb(b, i)), h = k * Math.sqrt(P[a].v[i] + P[b].v[i]);
      return [fmtPct(100 * (Math.exp(d) - 1)), `${fmtPct(100 * (Math.exp(d - h) - 1))} to ${fmtPct(100 * (Math.exp(d + h) - 1))}`];
    };
    for (let i = R.length - 1; i >= 0; i--) {
      const have = ts.filter((t) => ok(t, i));
      if (!have.length) continue;
      const knee = have.reduce((b, t) => (mb(t, i) > mb(b, i) ? t : b), have[0]);
      const peak = mb(knee, i);
      const tr = el("tr", {}, tb);
      el("th", { text: R[i], scope: "row" }, tr);
      for (const t of ts) {
        const td = el("td", { class: `bh-knee-cell${t === knee ? " is-knee" : ""}` }, tr);
        if (!ok(t, i)) { td.textContent = "–"; continue; }
        const f = mb(t, i) / peak;
        const share = Math.round(70 * Math.max(0, Math.min(1, (f - 0.8) / 0.2)));
        if (share) td.style.background = `color-mix(in srgb, var(--bh-s1) ${share}%, transparent)`;
        td.textContent = (100 * f).toFixed(0);
        td.title = `${R[i]} at ${t} threads: ${mb(t, i).toFixed(0)} MB/s modelled, ${(100 * f).toFixed(1)}% of its peak (at ${knee})`;
      }
      el("td", { text: knee, class: "bh-knee-at" }, tr);
      el("td", { text: peak.toFixed(0) }, tr);
      for (const b of ["8", "10"]) {
        const v = vs(knee, b, i), td = el("td", {}, tr);
        if (typeof v === "string") { td.textContent = v; td.className = "is-self"; continue; }
        el("span", { text: v[0] }, td);
        el("small", { text: v[1] }, td);
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

  // The release ramp's colours are worked out when drawn (from the scheme's stops): a change
  // of scheme redraws
  let schemeWatch = false;
  const watchScheme = () => {
    if (schemeWatch || typeof MutationObserver === "undefined") return;
    schemeWatch = true;
    const mo = new MutationObserver(() => { if (drawn.hero && drawn.hero[1].isConnected) trajectory(...drawn.hero); });
    for (const e of [document.documentElement, document.body]) mo.observe(e, { attributes: true, attributeFilter: ["data-md-color-scheme"] });
  };

  function init() {
    watchScheme();
    const root = document.getElementById("bench-history");
    if (root) {
      fetch(root.dataset.src)
        .then((r) => { if (!r.ok) throw new Error(r.status); return r.json(); })
        .then((d) => {
          // the release-scaling runs, if any are recorded (none is no error: the view is offered only with one)
          const hero = document.getElementById("bench-hero");
          const src = hero && hero.dataset.ws;
          const ws = src ? fetch(src).then((r) => (r.ok ? r.json() : null)).catch(() => null) : Promise.resolve(null);
          // each release's notes and posts (none is no error: the panel is left out)
          const nsrc = hero && hero.dataset.notes;
          const notes = nsrc ? fetch(nsrc).then((r) => (r.ok ? r.json() : null)).catch(() => null) : Promise.resolve(null);
          if (nsrc) d.notesRoot = new URL("../", new URL(nsrc, location.href)).href;
          return Promise.all([ws, notes]).then(([w, n]) => { d.scaling = w; d.notes = n; return d; });
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
