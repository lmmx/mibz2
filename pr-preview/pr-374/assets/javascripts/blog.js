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
  // word into its tree; data-stages lists the words to choose from) or "induction"
  // (Rauzy–Veech induction on the block aab). Without JavaScript the figure shows its
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
      model = fig.dataset.model === "lyndon" ? lyndonFrames(word) : inductionFrames();
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

  const init = () => {
    document.querySelectorAll(".bf-chain").forEach(chain);
    document.querySelectorAll(".bf-bitbuf").forEach(bitbuf);
    document.querySelectorAll(".bf-anim-controls").forEach(animated);
    document.querySelectorAll(".bf-stepper").forEach(stepper);
  };
  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(init);
  else if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
