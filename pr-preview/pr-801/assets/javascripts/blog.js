// The blog's interactive figures (docs/site/blog/).
//
// - .bf-chain: the pointer chain. "Follow pointer" moves from the current card to the card
//   it names, until every card has been visited; "Reset" goes back to card 0. Without
//   JavaScript the figure shows the whole path instead.
// - .bf-bitbuf: a bit buffer read one code at a time. Each step says in words what is at the
//   front and why it is that symbol; the button, labelled with what it will do, shifts that
//   code out. The code key lights the matching code. "Reset" starts again. Without
//   JavaScript the figure shows the whole decode instead.
// - .bf-anim-controls: replay and pause buttons on an animated figure, which plays once and
//   holds its last frame. Pause freezes it where it is; replay starts it from the beginning.
//   With reduced motion the figure starts as a still picture, and either button runs the
//   animation on request.
// - .bf-l3fill: workers' footprints filling a shared L3, set by a slider, with each count's
//   measurements said in words. Without JavaScript the figure shows a stepped summary.
//
// - .post-meta a and .post-card-date: the post's kind, read from its label (the text after the
//   last "·") and set as data-kind, which blog.css colours. The labels and their kinds are
//   KINDS below; docs/scripts/check_site.py keeps the posts to them.
//
// Runs on load and on each instant navigation, as bench.js does.
(() => {
  "use strict";

  const chain = (fig) => {
    if (fig.dataset.ready) return;
    fig.dataset.ready = "1";
    const next = fig.dataset.next.split(" ").map(Number);
    const cards = [...fig.querySelectorAll(".bf-cardx")];
    const q = (sel) => fig.querySelector(sel);
    const [head, buttons, at, count, trail, follow, reset] = [
      ".bf-walk-head", ".bf-buttons", ".bf-at", ".bf-count", ".bf-trail", ".bf-follow", ".bf-reset",
    ].map(q);
    let path = [0];
    const draw = () => {
      const cur = path[path.length - 1];
      cards.forEach((card, i) => {
        card.classList.toggle("is-current", i === cur);
        card.classList.toggle("is-visited", i !== cur && path.includes(i));
      });
      at.textContent = cur;
      count.textContent = `${path.length} of ${next.length} visited`;
      trail.textContent = path.join(" → ");
      follow.disabled = path.includes(next[cur]);
    };
    follow.addEventListener("click", () => {
      const to = next[path[path.length - 1]];
      if (!path.includes(to)) path.push(to);
      draw();
    });
    reset.addEventListener("click", () => {
      path = [0];
      draw();
      follow.focus();
    });
    head.hidden = false;
    buttons.hidden = false;
    draw();
  };

  const animated = (box) => {
    if (box.dataset.ready) return;
    box.dataset.ready = "1";
    const svg = box.closest("figure").querySelector(":scope > svg");
    const toggle = box.querySelector('[data-act="toggle"]');
    const replay = box.querySelector('[data-act="replay"]');
    let playing = !window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    // A reader who asks for the animation gets it, whatever their motion setting.
    const animations = () => {
      svg.classList.add("is-manual");
      return svg.getAnimations({ subtree: true });
    };
    const draw = () => {
      toggle.dataset.state = playing ? "playing" : "paused";
      const label = playing ? "Pause" : "Play";
      toggle.setAttribute("aria-label", label);
      toggle.title = label;
    };
    toggle.addEventListener("click", () => {
      playing = !playing;
      animations().forEach((a) => (playing ? a.play() : a.pause()));
      draw();
    });
    replay.addEventListener("click", () => {
      animations().forEach((a) => {
        a.currentTime = 0;
        a.play();
      });
      playing = true;
      draw();
    });
    // When the play ends, the button offers to play it again.
    svg.addEventListener("animationend", () => {
      if (playing && svg.getAnimations({ subtree: true }).every((a) => a.playState === "finished")) {
        playing = false;
        draw();
      }
    });
    box.hidden = false;
    draw();
  };

  const bitbuf = (fig) => {
    if (fig.dataset.ready) return;
    fig.dataset.ready = "1";
    const bits = fig.dataset.bits;
    const codes = Object.fromEntries(fig.dataset.codes.split(" ").map((p) => p.split(":")));
    const cells = [...fig.querySelectorAll(".bf-bit")];
    const keys = [...fig.querySelectorAll(".bf-key")];
    const q = (sel) => fig.querySelector(sel);
    const [step, state, buttons, live, shift, out, read, reset] = [
      ".bf-buf-step", ".bf-buf-state", ".bf-buttons", ".bf-buf-live", ".bf-buf-shift", ".bf-buf-out",
      ".bf-read", ".bf-reset",
    ].map(q);
    const codeAt = (r) => Object.keys(codes).find((c) => r.startsWith(c)) || "";
    // The whole decode, worked out once, so each step can say "step i of n"
    const plan = [];
    for (let r = bits; codeAt(r); r = r.slice(codeAt(r).length)) plan.push(codeAt(r));
    const nbits = (n) => `${n} bit${n === 1 ? "" : "s"}`;
    let i = 0;
    const draw = () => {
      const done = plan.slice(0, i).join("").length;
      const rem = bits.slice(done);
      const code = plan[i] || "";
      const prev = plan[i - 1] || "";
      cells.forEach((cell, k) => {
        cell.textContent = k < rem.length ? rem[k] : "\u00b7";
        cell.classList.toggle("is-next", k < code.length);
        cell.classList.toggle("is-empty", k >= rem.length);
      });
      keys.forEach((key) => key.classList.toggle("is-next", key.dataset.code === code));
      const shifted = prev
        ? `${codes[prev]}'s ${nbits(prev.length)} ${prev.length === 1 ? "was" : "were"} shifted out, and the rest of the buffer moved left to fill the gap. `
        : "";
      if (code) {
        const which = i === 0 ? "starts" : "now starts";
        step.textContent =
          `Step ${i + 1} of ${plan.length}. ${shifted}The buffer ${which} with ${code}. ` +
          `That is ${codes[code]}'s code, and no other code begins this way, so the next symbol is ` +
          `${codes[code]}. Its code is ${nbits(code.length)} long.`;
        read.textContent = `Read ${codes[code]}: shift out ${nbits(code.length)} \u2192`;
      } else {
        step.textContent =
          `Done. ${shifted}All ten bits are read, and they decoded to ${plan.map((c) => codes[c]).join(" ")}. ` +
          "The dotted cells are the room the shifts freed up; in mibz2 a refill would load more of the file into it.";
        read.textContent = "All bits read";
      }
      live.textContent = rem.length;
      shift.textContent = prev ? `buf <<= ${prev.length}; live -= ${prev.length};` : "nothing shifted yet";
      out.textContent = i ? plan.slice(0, i).map((c) => codes[c]).join(" ") : "none yet";
      read.disabled = !code;
    };
    read.addEventListener("click", () => {
      if (i < plan.length) i += 1;
      draw();
    });
    reset.addEventListener("click", () => {
      i = 0;
      draw();
      read.focus();
    });
    step.hidden = false;
    state.hidden = false;
    buttons.hidden = false;
    q(".bf-buf-static").hidden = true;
    draw();
  };

  // .bf-stepper: a figure that shows a process one step at a time (Back, Play, Next, a slider),
  // with each step said in words. data-model names the process: "lyndon" (cutting a Lyndon
  // word into its tree; data-stages lists the words to choose from), "induction"
  // (Rauzy–Veech induction on the block aab) or "cutloop" (cutting banana$'s loop open). Without JavaScript the figure shows its
  // static summary (.bf-step-static) instead.
  const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;");
  const T = (x, y, s, cls = "bf-t", size = 18, extra = "", anchor = "middle") =>
    `<text x="${x}" y="${y}" class="${cls}" text-anchor="${anchor}" style="font-size:${size}px${extra}">${esc(s)}</text>`;
  const code = (s) => `“${s}”`;

  const lyndonFrames = (w) => {
    const n = w.length;
    const off = w[0] === "#" ? 1 : 0; // positions count the letters after a leading #
    const pos = (p) => p - off;
    const order = [...Array(n).keys()].sort((a, b) => (w.slice(a) < w.slice(b) ? -1 : 1));
    const rank = [];
    order.forEach((p, r) => (rank[p] = r));
    const CW = Math.min(56, Math.floor(440 / n));
    const X0 = Math.round((540 - CW * n) / 2);
    const x = (p) => X0 + p * CW;
    const cx = (p) => X0 + (p + 0.5) * CW;
    // the whole traversal: look at a part, cut it, go left, go right, return
    const nodes = [];
    const walk = (a, b, d, parent) => {
      if (b - a < 2) return;
      let k = a + 1;
      for (let p = a + 2; p < b; p++) if (rank[p] < rank[k]) k = p;
      const node = { a, k, b, d, parent };
      nodes.push(node);
      walk(a, k, d + 1, node);
      walk(k, b, d + 1, node);
    };
    walk(0, n, 0, null);
    const depth = nodes.reduce((m, nd) => Math.max(m, nd.d), 0);
    const ROW = 34;
    const TOP = 150;
    const H = TOP + (depth + 2) * ROW + 10;
    const draw = (cut, active, look, done) => {
      const out = [];
      out.push(T(14, 52, "i", "bf-tm", 14, "", "start"));
      out.push(T(14, 118, "rank", "bf-tm", 14, "", "start"));
      for (let p = 0; p < n; p++) {
        if (pos(p) >= 0) out.push(T(cx(p), 52, pos(p), "bf-tm", 13));
        out.push(T(cx(p), 84, w[p], "bf-t-strong", 21));
        const cand = active && p > active.a && p < active.b;
        const low = look && p === active.k;
        out.push(T(cx(p), 118, rank[p], low ? "bf-t-strong" : cand ? "bf-t" : "bf-tm", low ? 17 : 15,
          low ? ";fill:var(--bf-amber)" : cand ? "" : ";opacity:0.45"));
        if (low) out.push(`<circle cx="${cx(p)}" cy="${112}" r="13" fill="none" stroke="var(--bf-amber)" stroke-width="2.5"/>`);
      }
      if (active) out.push(`<rect x="${x(active.a) + 1}" y="62" width="${(active.b - active.a) * CW - 2}" height="30" rx="6" fill="none" stroke="var(--bf-green)" stroke-width="3"/>`);
      // the tree so far: every part looked at, as a bar at its depth, cut where it has been cut
      const shown = nodes.filter((nd) => cut.has(nd) || nd === active);
      for (const nd of shown) {
        const y = TOP + nd.d * ROW;
        const cls = nd === active ? "bf-pill-hi" : done.has(nd) ? "bf-pill" : "bf-pill";
        const op = done.has(nd) && nd !== active ? ";opacity:0.6" : "";
        out.push(`<rect x="${x(nd.a) + 2}" y="${y}" width="${(nd.b - nd.a) * CW - 4}" height="${ROW - 8}" rx="6" class="${cls}" style="stroke:var(--bf-frame);stroke-width:1.5${op}"/>`);
        for (let p = nd.a; p < nd.b; p++) out.push(T(cx(p), y + 19, w[p], "bf-t", Math.min(17, CW - 8)));
        if (cut.has(nd)) out.push(`<path d="M${x(nd.k)} ${y - 3}V${y + ROW - 5}" fill="none" stroke="var(--bf-amber)" stroke-width="3.5" stroke-linecap="round"/>`);
      }
      return out.join("");
    };
    const frames = [];
    const cut = new Set();
    const done = new Set();
    const s = (nd) => code(w.slice(nd.a, nd.b));
    const at = (p) => (pos(p) < 0 ? "the #" : `position ${pos(p)}`);
    frames.push({
      svg: draw(cut, null, false, done),
      text: `The word ${code(w)}. The rank row numbers every suffix in dictionary order, 0 for the smallest. ` +
        "Next: look at the whole word and find where to cut it.",
    });
    nodes.forEach((nd, i) => {
      const suffix = w.slice(nd.k, nd.b);
      frames.push({
        svg: draw(cut, nd, true, done),
        text: `Look at ${s(nd)}. Its proper suffixes start after its first letter. The smallest is ${code(suffix)}, at ${at(nd.k)}: ` +
          `its rank, ${rank[nd.k]}, is the lowest inside the green box.`,
      });
      cut.add(nd);
      const left = { a: nd.a, b: nd.k }, right = { a: nd.k, b: nd.b };
      const nextNode = nodes[i + 1];
      let then;
      if (nextNode && nextNode.parent === nd) then = `Next, the ${nextNode.a === nd.a ? "left" : "right"} part, ${s(nextNode)}.`;
      else if (!nextNode) then = "Both parts are single letters, so this was the last cut.";
      else then = "Both parts are single letters.";
      frames.push({ svg: draw(cut, nd, false, done), text: `Cut before ${at(nd.k)}: ${s(left)} | ${s(right)}. ${then}` });
      // returning: when the next part to cut is not a child of this one, go back up
      if (nextNode && nextNode.parent !== nd) {
        const finished = [];
        let up = nd;
        while (up && up !== nextNode.parent) {
          done.add(up);
          finished.push(s(up));
          up = up.parent;
        }
        const also = finished.length > 1 ? `, so ${finished.slice(1).join(" and ")} ${finished.length > 2 ? "are" : "is"} too` : "";
        frames.push({
          svg: draw(cut, nextNode.parent, false, done),
          text: `${finished[0]} is finished${also}. Return up to ${s(nextNode.parent)}: its right part, ${s(nextNode)}, is still to be cut.`,
        });
      }
    });
    nodes.forEach((nd) => done.add(nd));
    frames.push({
      svg: draw(cut, null, false, new Set()),
      text: `Done: ${nodes.length} cut${nodes.length === 1 ? "" : "s"}, so ${nodes.length} join${nodes.length === 1 ? "" : "s"}. ` +
        "Read from the bottom up, each cut is one rule that joins two words, as the kernel's rules X → AB do.",
    });
    return { frames, height: H };
  };

  // Rauzy–Veech induction on the block aab: rows, LF's arrows, a moving token
  const inductionFrames = () => {
    const X = [120, 270, 420];
    const RY = 120;
    const byte = ["b", "a", "a"];
    // an arrow leaves a row from the left of its box and arrives at the right of the next,
    // so arrows in opposite directions never coincide; a "hot" arrow is the one just taken
    const arc = (from, to, hot, label) => {
      const stroke = hot ? "var(--bf-green)" : "var(--bf-muted)";
      const mk = hot ? "url(#bf-ind-hot)" : "url(#bf-ind-arrow)";
      const lab = (lx, ly) => (label ? T(lx, ly, label, "bf-t-strong", 20, ";fill:var(--bf-green-ink)") : "");
      if (from === to) {
        const x0 = X[from];
        return `<path d="M${x0 - 14} ${RY + 32}C${x0 - 70} ${RY + 150} ${x0 + 70} ${RY + 150} ${x0 + 14} ${RY + 36}" fill="none" stroke="${stroke}" stroke-width="3" marker-end="${mk}"/>` + lab(x0, RY + 150);
      }
      const a = X[from] - 14, b = X[to] + 14;
      const dip = (from < to ? 70 : 34) + Math.abs(b - a) * 0.3;
      const y0 = RY + 32;
      const mid = (a + b) / 2;
      return `<path d="M${a} ${y0}Q${mid} ${y0 + dip * 2} ${b} ${y0 + 4}" fill="none" stroke="${stroke}" stroke-width="${hot ? 4 : 2.5}" marker-end="${mk}"/>` + lab(mid, y0 + dip + 22);
    };
    const marker = (id, fill) => `<marker id="${id}" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0L10 5L0 10z" fill="${fill}"/></marker>`;
    const defs = `<defs>${marker("bf-ind-arrow", "var(--bf-muted)")}${marker("bf-ind-hot", "var(--bf-green)")}</defs>`;
    const scene = ({ rows = [0, 1, 2], gone = [], arcs = [], token = null, pieces = [], out = "", note = "" }) => {
      const o = [defs];
      o.push(T(20, 30, "rows of the block aab, last-column byte in each", "bf-tm", 15, "", "start"));
      for (const pc of pieces) {
        const xa = X[pc.rows[0]] - 52, xb = X[pc.rows[pc.rows.length - 1]] + 52;
        o.push(`<rect x="${xa}" y="${RY - 58}" width="${xb - xa}" height="24" rx="12" fill="var(--bf-${pc.hi ? "amber" : "steel"}-tint)" stroke="var(--bf-${pc.hi ? "amber" : "steel"})" stroke-width="1.5"/>`);
        o.push(T((xa + xb) / 2, RY - 41, pc.label, "bf-t-strong", 15));
      }
      for (const r of [0, 1, 2]) {
        const g = gone.includes(r);
        const here = token === r;
        o.push(`<rect x="${X[r] - 40}" y="${RY - 26}" width="80" height="56" rx="8" class="${g ? "bf-track" : here ? "bf-card-hi" : "bf-card"}" style="stroke:${here ? "var(--bf-green)" : "var(--bf-frame)"};stroke-width:${here ? 3.5 : 1.5}${g ? ";stroke-dasharray:5 5;opacity:0.55" : ""}"/>`);
        o.push(T(X[r], RY + 10, byte[r], "bf-t-strong", 26, g ? ";opacity:0.4" : ""));
        o.push(T(X[r], RY - 34 - (pieces.length ? 28 : 0), `row ${r}`, "bf-tm", 14, g ? ";opacity:0.5" : ""));
      }
      for (const a of arcs) o.push(arc(a[0], a[1], a[2] === "hot", a[3]));
      if (out) o.push(T(20, 330, out, "bf-t-strong", 20, "", "start"));
      if (note) o.push(T(20, 362, note, "bf-tm", 16, "", "start"));
      return o.join("");
    };
    const LF = [[0, 2], [1, 0], [2, 1]];
    const A = { rows: [1, 2], label: "A: rows 1–2, word a" };
    const B = { rows: [0], label: "B: row 0, word b" };
    const f = [];
    f.push({ svg: scene({ arcs: LF }), text: "The block aab has three rows. Each holds its last-column byte. LF sends row 0 to row 2, row 1 to row 0 and row 2 to row 1: the three arrows." });
    f.push({ svg: scene({ arcs: LF, token: 0, out: "read so far: b" }), text: "The walk: start at row 0 and read its byte, b." });
    f.push({ svg: scene({ arcs: [[0, 2, "hot"], [1, 0], [2, 1]], token: 2, out: "read so far: b a" }), text: "Follow row 0's arrow: the walk visits row 2 and reads a. One lookup per row visited." });
    f.push({ svg: scene({ arcs: [[0, 2], [1, 0], [2, 1, "hot"]], token: 1, out: "read so far: b a a" }), text: "Follow row 2's arrow to row 1 and read a." });
    f.push({ svg: scene({ arcs: [[0, 2], [1, 0, "hot"], [2, 1]], token: 0, out: "read so far: b a a  →  backwards: aab", note: "3 visits, 3 lookups" }), text: "Row 1's arrow returns to row 0, where the walk began. It read b a a: the block backwards, aab. Every row was visited once." });
    f.push({ svg: scene({ arcs: [[0, 2, "hot"], [1, 0], [2, 1]], pieces: [B, { ...A, hi: true }], token: null }), text: "The induction never walks. The runs are pieces: B is row 0 (byte b), A is rows 1–2 (bytes a a). At the right-hand end, A is the last piece, 2 rows long. The last row any arrow lands on is row 2, and B's arrow lands there. B is 1 row long, so B is the shorter." });
    f.push({ svg: scene({ arcs: [[0, 1, "hot", "ab"], [1, 0]], gone: [2], pieces: [{ rows: [0], label: "B: word ab", hi: true }, { rows: [1], label: "A: word a" }] }), text: "Only B's row lands on row 2, and from row 2 the path always goes on to row 1. So row 2 can be deleted. B's row now returns straight to row 1, skipping the visit to row 2, and its word records both bytes: ab. A shrinks to 1 row." });
    f.push({ svg: scene({ arcs: [[0, 1, "hot", "ab"], [1, 0]], gone: [2], token: null, pieces: [{ rows: [0], label: "B: word ab", hi: true }, { rows: [1], label: "A: word a" }] }), text: "Two rows are left. A is row 1 (word a), and B is row 0 (word ab). B's arrow lands on all of A, and the two have the same length, 1 row." });
    f.push({ svg: scene({ arcs: [[0, 0, "hot", "aab"]], gone: [1, 2], token: 0, pieces: [{ rows: [0], label: "one piece: word aab", hi: true }], note: "2 steps; no row was read one at a time" }), text: "Equal lengths: join them. A's row is deleted too, and B's row returns to itself in one jump, carrying the word a + ab = aab: the whole block. Each step deleted a whole piece's worth of rows, so a run of a thousand rows would go in one step." });
    return { frames: f, height: 380 };
  };

  // Cutting banana$'s loop open (the open-return post): the column annb$aa, its LF loop drawn
  // as a ring in the loop's order, the arrows leaving the kept rows 0, 4 and 5 cut one by
  // one, then each open path followed from its entrance to its exit, with its word.
  const cutloopFrames = () => {
    const L = "annb$aa";
    const LF = [1, 5, 6, 4, 0, 2, 3];
    const ring = [4, 0, 1, 5, 2, 6, 3]; // the loop, from the primary row
    const KEPT = [0, 4, 5];
    const CUTS = [4, 0, 5]; // the order the arrows are cut in
    const PATHS = [
      { e: 0, rows: [0], word: "", exit: 0 },
      { e: 1, rows: [1, 5], word: "n", exit: 5 },
      { e: 2, rows: [2, 6, 3, 4], word: "nab", exit: 4 },
    ];
    const HUE = ["green", "blue", "ultra"];
    const CX = 270, CY = 196, R = 112, NR = 25;
    const ang = {};
    ring.forEach((r, i) => (ang[r] = -Math.PI / 2 + (i * 2 * Math.PI) / 7));
    const at = (a, rad = R) => [CX + rad * Math.cos(a), CY + rad * Math.sin(a)];
    const f1 = (v) => v.toFixed(1);
    const D = (NR + 5) / R;
    const span = (from) => {
      const a0 = ang[from] + D;
      let a1 = ang[LF[from]] - D;
      if (a1 < a0) a1 += 2 * Math.PI;
      return [a0, a1];
    };
    const marker = (id, v) => `<marker id="${id}" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0L10 5L0 10z" fill="var(${v})"/></marker>`;
    const defs = `<defs>${marker("bf-cl-m", "--bf-muted")}${HUE.map((h) => marker(`bf-cl-${h}`, `--bf-${h}`)).join("")}</defs>`;
    const scene = ({ walk = false, kept = false, cut = [], shown = 0, cur = -1, done = false }) => {
      const o = [defs];
      // the arrows
      for (const r of ring) {
        const [a0, a1] = span(r);
        const [x0, y0] = at(a0), [x1, y1] = at(a1);
        const d = `M${f1(x0)} ${f1(y0)}A${R} ${R} 0 0 1 ${f1(x1)} ${f1(y1)}`;
        if (cut.includes(r)) {
          const [mx, my] = at((a0 + a1) / 2);
          o.push(`<path d="${d}" fill="none" stroke="var(--bf-muted)" stroke-width="1.5" stroke-dasharray="3 5" opacity="0.5"/>`);
          o.push(`<path d="M${f1(mx - 7)} ${f1(my - 7)}l14 14M${f1(mx + 7)} ${f1(my - 7)}l-14 14" stroke="var(--bf-amber)" stroke-width="3" stroke-linecap="round"/>`);
          continue;
        }
        const p = PATHS.findIndex((q) => q.rows.slice(0, -1).includes(r));
        const lit = walk || (p >= 0 && p < shown);
        const hue = walk ? "green" : p >= 0 ? HUE[p] : null;
        const stroke = lit ? `var(--bf-${hue})` : "var(--bf-muted)";
        const mk = lit ? `url(#bf-cl-${hue})` : "url(#bf-cl-m)";
        const w = lit && (walk || p === cur) ? 4 : lit ? 2.8 : 2;
        o.push(`<path d="${d}" fill="none" stroke="${stroke}" stroke-width="${w}" marker-end="${mk}"/>`);
      }
      // the rows
      for (const r of ring) {
        const [x, y] = at(ang[r]);
        const p = PATHS.findIndex((q) => q.rows.includes(r));
        const lit = p >= 0 && p < shown;
        const isKept = kept && KEPT.includes(r);
        const fill = lit ? `var(--bf-${HUE[p]}-tint)` : walk && r === 4 ? "var(--bf-green-tint)" : "var(--bf-surface)";
        const stroke = isKept ? "var(--bf-amber)" : lit ? `var(--bf-${HUE[p]})` : "var(--bf-frame)";
        o.push(`<circle cx="${f1(x)}" cy="${f1(y)}" r="${NR}" fill="${fill}" stroke="${stroke}" stroke-width="${isKept ? 4 : lit && p === cur ? 3 : 1.5}"/>`);
        o.push(T(f1(x), f1(y + 8), L[r], "bf-t-strong", 22));
        // labels outside the ring: the row, then what it has become
        const [lx, ly] = at(ang[r], R + NR + 30);
        const lines = [`row ${r}`];
        if (isKept) lines.push(`exit index ${KEPT.indexOf(r)}`);
        const e = cut.map((c) => LF[c]).includes(r) ? r : -1;
        if (e >= 0) lines.push(`entrance ${e}`);
        const top = ly - ((lines.length - 1) * 17) / 2 + 5;
        lines.forEach((s, k) => {
          const cls = k === 0 ? "bf-tm" : "bf-t-strong";
          const col = s.startsWith("exit") ? ";fill:var(--bf-amber)" : s.startsWith("entrance") ? ";fill:var(--bf-green-ink)" : "";
          o.push(T(f1(lx), f1(top + k * 17), s, cls, k === 0 ? 15 : 14, col));
        });
        // an entrance: a dot where the cut arrow used to arrive
        if (e >= 0) {
          const [dx, dy] = at(ang[r] - D + 0.02);
          o.push(`<circle cx="${f1(dx)}" cy="${f1(dy)}" r="6" fill="var(--bf-green)" stroke="var(--md-default-bg-color)" stroke-width="1.5"/>`);
        }
      }
      // the panel below the ring
      const PY = 400;
      if (walk) {
        o.push(T(20, PY, "read along the loop from row 4:  $  a  n  a  n  a  b", "bf-t-strong", 17, "", "start"));
        o.push(T(20, PY + 26, "backwards, that is the text banana$", "bf-t", 17, "", "start"));
      } else if (shown > 0 || done) {
        const cols = [16, 116, 300, 380, 526];
        ["entrance", "path (rows)", "exit row", "exit index", "word"].forEach((h, k) =>
          o.push(T(cols[k], PY, h, "bf-tm", 14, "", k === 4 ? "end" : "start")));
        PATHS.slice(0, shown).forEach((p, k) => {
          const y = PY + 28 + k * 27;
          const col = `;fill:var(--bf-${HUE[k] === "green" ? "green-ink" : HUE[k]})`;
          o.push(T(cols[0], y, `${p.e}`, "bf-t-strong", 17, col, "start"));
          o.push(T(cols[1], y, p.rows.join(" → "), "bf-t", 17, "", "start"));
          o.push(T(cols[2], y, `${p.exit}`, "bf-t", 17, "", "start"));
          o.push(T(cols[3], y, `${KEPT.indexOf(p.exit)}`, "bf-t-strong", 17, ";fill:var(--bf-amber)", "start"));
          o.push(T(cols[4], y, p.word || "(empty)", "bf-t-strong", 17, "", "end"));
        });
        if (done) o.push(T(16, PY + 28 + 3 * 27 + 4, "P = [0, 2, 1]: entrance index → exit index", "bf-t-strong", 17, "", "start"));
      } else if (kept) {
        o.push(T(20, PY, "kept rows 0, 4, 5 are the exits, numbered 0, 1, 2 in row order", "bf-t", 17, "", "start"));
        if (cut.length) o.push(T(20, PY + 24, `cut so far: the arrow${cut.length > 1 ? "s" : ""} leaving row${cut.length > 1 ? "s" : ""} ${cut.join(", ")}`, "bf-t", 17, "", "start"));
      } else {
        o.push(T(20, PY, "the column annb$aa: one arrow (LF) leaves each row", "bf-t", 17, "", "start"));
      }
      o.push(`<circle cx="388" cy="22" r="6" fill="var(--bf-green)"/>`);
      o.push(T(400, 26, "entrance", "bf-tm", 14, "", "start"));
      o.push(`<circle cx="476" cy="21" r="7" fill="none" stroke="var(--bf-amber)" stroke-width="3"/>`);
      o.push(T(489, 26, "kept", "bf-tm", 14, "", "start"));
      return o.join("");
    };
    const f = [];
    f.push({ svg: scene({}), text: "The seven rows of the column annb$aa, each with its last-column letter. Each arrow is LF: it goes from a row to the row of the rotation one letter earlier. Drawn in the order the arrows visit them, the rows form one loop. Next: walk it." });
    f.push({ svg: scene({ walk: true }), text: "Start at row 4, the primary row, and follow the arrows. The walk visits every row once and reads $ a n a n a b, which is banana$ backwards. Each step needs the row the last step found, so the steps can't overlap. Next: choose rows to keep." });
    f.push({ svg: scene({ kept: true }), text: "Keep rows 0, 4 and 5 (amber rings). Row 4 must be kept, because the text is read from it. Numbered in row order, the kept rows are exit indices 0, 1 and 2: these are where the open paths will end. Next: cut the arrow leaving each kept row." });
    CUTS.forEach((c, k) => {
      const last = k === CUTS.length - 1;
      f.push({
        svg: scene({ kept: true, cut: CUTS.slice(0, k + 1) }),
        text: `Cut the arrow leaving row ${c}. Row ${LF[c]}, where it arrived, now has no arrow coming in, so a path can start there: it is entrance ${LF[c]} (green dot).` +
          (LF[c] === 0 ? " Row 0 is now both a kept row and an entrance." : "") +
          (last ? " Three arrows cut, three entrances: rows 0, 1 and 2. Next: follow the arrows that are left, from each entrance." : ` Next: cut the arrow leaving row ${CUTS[k + 1]}.`),
      });
    });
    const say = [
      "Entrance 0 is row 0, which is itself kept. Its path takes no steps: it ends at once, at exit row 0, which is exit index 0, and reads nothing.",
      "From entrance 1 (row 1), the arrow goes to row 5 and reads n. Row 5 is kept, and its own arrow is cut, so the path stops: exit row 5, which is exit index 2, the third kept row.",
      "From entrance 2 (row 2), the arrows go to rows 6, 3 and 4, reading n, a, b. Row 4 is kept, so the path stops there: exit row 4, which is exit index 1. The word nab is ban, backwards.",
    ];
    PATHS.forEach((p, k) => f.push({ svg: scene({ kept: true, cut: CUTS, shown: k + 1, cur: k }), text: say[k] + (k < 2 ? ` Next: entrance ${k + 1}.` : " Next: what the three paths give.") }));
    f.push({ svg: scene({ kept: true, cut: CUTS, shown: 3, done: true }), text: "Every row lies on exactly one open path, and none joined a loop: each row of a loop already uses its one incoming arrow, and an entrance has lost its own. The paths give P = [0, 2, 1], from entrance index to exit index, and a word for each entrance. None of this needed to know where the cut arrows went: only the arrows that are left, and which rows are kept." });
    return { frames: f, height: 520 };
  };

  const stepper = (fig) => {
    if (fig.dataset.ready) return;
    fig.dataset.ready = "1";
    const q = (sel) => fig.querySelector(sel);
    const svg = q(".bf-step-svg");
    const text = q(".bf-step-text");
    const controls = q(".bf-step-controls");
    const stages = q(".bf-step-stages");
    const [prev, play, next, slider] = ["prev", "play", "next", "slider"].map((a) => q(`[data-act="${a}"]`));
    let model, i = 0, timer = null;
    const load = (word) => {
      const m = fig.dataset.model;
      model = m === "lyndon" ? lyndonFrames(word) : m === "cutloop" ? cutloopFrames() : inductionFrames();
      svg.setAttribute("viewBox", `0 0 540 ${model.height}`);
      slider.max = model.frames.length - 1;
      i = 0;
      show();
    };
    const show = () => {
      const fr = model.frames[i];
      svg.innerHTML = fr.svg;
      text.textContent = `Step ${i + 1} of ${model.frames.length}. ${fr.text}`;
      slider.value = i;
      prev.disabled = i === 0;
      next.disabled = i === model.frames.length - 1;
    };
    const stop = () => {
      if (timer) clearInterval(timer);
      timer = null;
      play.textContent = "Play";
    };
    prev.addEventListener("click", () => { stop(); if (i > 0) i--; show(); });
    next.addEventListener("click", () => { stop(); if (i < model.frames.length - 1) i++; show(); });
    slider.addEventListener("input", () => { stop(); i = Number(slider.value); show(); });
    play.addEventListener("click", () => {
      if (timer) return stop();
      if (i === model.frames.length - 1) i = 0;
      show();
      play.textContent = "Pause";
      timer = setInterval(() => {
        if (i >= model.frames.length - 1) return stop();
        i++;
        show();
      }, 2600);
    });
    if (stages) {
      stages.querySelectorAll("button").forEach((b) =>
        b.addEventListener("click", () => {
          stop();
          stages.querySelectorAll("button").forEach((o) => o.setAttribute("aria-pressed", String(o === b)));
          load(b.dataset.word);
        }));
      stages.hidden = false;
    }
    q(".bf-step-static").hidden = true;
    svg.removeAttribute("hidden"); // an SVG element has no .hidden property
    text.hidden = false;
    controls.hidden = false;
    load(stages ? stages.querySelector('[aria-pressed="true"]').dataset.word : "");
  };

  // .bf-l3fill: workers' footprints filling a shared L3, one block per worker, set by a slider.
  // data-l3 and data-mb are the cache's and a footprint's size in MB. Each count says in words
  // what was measured there (the i9 at 10 threads). Without JavaScript the figure shows its
  // static stepped summary (.bf-l3-static) instead.
  const L3_SAY = {
    1: "One footprint uses a tenth of the L3, and no other decoding worker competes with it for the cache (the scanner, the reader and the rest of the system still do). Not measured at 10 threads.",
    2: "Two footprints fill a fifth of the L3. Not measured at 10 threads; the model expects little contention this low.",
    3: "Three footprints fill under a third of the L3. Not measured at 10 threads; the model expects little contention this low.",
    4: "Four footprints fill 40% of the L3. Not measured at 10 threads; in the model, each worker added here still gains more than it costs the others.",
    5: "Five footprints fill half the L3. Not measured at 10 threads; in the model, each worker raises the others' misses a little, with no cliff.",
    6: "Six footprints fill 60%. Measured at 10 threads: 0.40 of ten workers' L3 misses per byte, and 0.922 of their wall time.",
    7: "Seven footprints fill 70%, the teams policy's budget. Measured at 10 threads: 0.52 of ten workers' L3 misses per byte, and the fastest count, 0.885 of the wall time.",
    8: "Eight footprints fill 80%, about where the fitted optimum lies. Measured at 10 threads: 0.65 of ten workers' L3 misses per byte, 0.902 of the wall time.",
    9: "Nine footprints fill 90%. Measured at 10 threads: 0.92 of ten workers' L3 misses per byte, 0.982 of the wall time.",
    10: "Ten footprints fill the whole L3. Every worker's data is evicted more often, and this is the slowest count measured at 10 threads.",
  };
  const l3fill = (fig) => {
    if (fig.dataset.ready) return;
    fig.dataset.ready = "1";
    const l3 = Number(fig.dataset.l3);
    const mb = Number(fig.dataset.mb);
    const q = (sel) => fig.querySelector(sel);
    const [svg, text, controls, input, out] = [
      ".bf-l3-svg", ".bf-l3-text", ".bf-l3-controls", "input", ".bf-l3-n",
    ].map(q);
    const BX = 10, BW = 500, BY = 30, BH = 44;
    const show = () => {
      const n = Number(input.value);
      out.textContent = String(n);
      const parts = [];
      parts.push(`<rect x="${BX}" y="${BY}" width="${BW}" height="${BH}" rx="5" class="bf-track"/>`);
      const w = (mb / l3) * BW;
      for (let k = 0; k < n; k++) {
        parts.push(`<rect x="${(BX + k * w).toFixed(1)}" y="${BY}" width="${(w - 2).toFixed(1)}" height="${BH}" rx="3" style="fill:var(--bf-green)"/>`);
      }
      for (const [f, lab] of [[0.7, "0.7"], [0.8, "0.8"]]) {
        const x = (BX + f * BW).toFixed(1);
        parts.push(`<path d="M${x} ${BY - 8}V${BY + BH + 8}" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:4 3"/>`);
        parts.push(T(x, BY - 12, lab, "bf-t-strong", 14, ";fill:var(--bf-amber)"));
      }
      const used = n * mb;
      parts.push(T(BX, BY + BH + 30, `${n} × ${mb} MB = ${used.toFixed(1)} MB of ${l3.toFixed(1)} MB`, "bf-t-strong", 16, "", "start"));
      parts.push(T(BX + BW, BY + BH + 30, `${Math.round((100 * used) / l3)}% of the L3`, "bf-t", 16, "", "end"));
      svg.innerHTML = parts.join("");
      text.textContent = L3_SAY[n] || "";
    };
    input.addEventListener("input", show);
    q(".bf-l3-static").hidden = true;
    svg.removeAttribute("hidden");
    text.hidden = false;
    controls.hidden = false;
    show();
  };

  // keep in step with KINDS in docs/scripts/check_site.py
  const KINDS = {
    "Research result": "result",
    "Research synthesis": "synthesis",
    "Research in progress": "progress",
    "Work in progress": "progress",
    "Investigation in progress": "progress",
    "Open question": "question",
    "Meta": "meta",
  };
  const kind = (el) => {
    const label = el.textContent.split("·").pop().trim();
    if (!KINDS[label]) return;
    el.dataset.kind = KINDS[label];
    // the post's hero, or its card on the index, takes the same colour for its bottom line
    const box = el.closest(".post-hero, .post-card");
    if (box) box.dataset.kind = KINDS[label];
  };

  const init = () => {
    document.querySelectorAll(".md-typeset .post-meta a, .post-card-date").forEach(kind);
    document.querySelectorAll(".bf-chain").forEach(chain);
    document.querySelectorAll(".bf-bitbuf").forEach(bitbuf);
    document.querySelectorAll(".bf-anim-controls").forEach(animated);
    document.querySelectorAll(".bf-stepper").forEach(stepper);
    document.querySelectorAll(".bf-l3fill").forEach(l3fill);
  };
  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(init);
  else if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
