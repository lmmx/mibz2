// The blog's interactive figures (docs/site/blog/).
//
// - .bf-chain: the pointer chain. "Follow pointer" moves from the current card to the card
//   it names, until every card has been visited; "Reset" goes back to card 0. Without
//   JavaScript the figure shows the whole path instead.
// - .bf-bitbuf: a bit buffer read one code at a time. "Read the next code" consumes the code
//   at the front (its bits highlighted), shifts the rest up and logs the symbol; "Reset"
//   starts again. Without JavaScript the figure shows the whole decode instead.
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
    const q = (sel) => fig.querySelector(sel);
    const [state, buttons, next, live, shift, out, read, reset] = [
      ".bf-buf-state", ".bf-buttons", ".bf-buf-next", ".bf-buf-live", ".bf-buf-shift", ".bf-buf-out",
      ".bf-read", ".bf-reset",
    ].map(q);
    let rem = bits;
    let decoded = [];
    let last = "";
    const front = () => Object.keys(codes).find((c) => rem.startsWith(c)) || "";
    const draw = () => {
      const code = front();
      cells.forEach((cell, i) => {
        cell.textContent = i < rem.length ? rem[i] : "·";
        cell.classList.toggle("is-next", i < code.length);
        cell.classList.toggle("is-empty", i >= rem.length);
      });
      next.textContent = code ? `${codes[code]} = ${code}` : "none: all ten bits read";
      live.textContent = rem.length;
      shift.textContent = last ? `buf <<= ${last.length}; live -= ${last.length};` : "nothing consumed yet";
      out.textContent = decoded.length ? decoded.join(" ") : "none yet";
      read.disabled = !code;
    };
    read.addEventListener("click", () => {
      const code = front();
      if (!code) return;
      decoded.push(codes[code]);
      rem = rem.slice(code.length);
      last = code;
      draw();
    });
    reset.addEventListener("click", () => {
      rem = bits;
      decoded = [];
      last = "";
      draw();
      read.focus();
    });
    state.hidden = false;
    buttons.hidden = false;
    q(".bf-buf-static").hidden = true;
    draw();
  };

  const init = () => {
    document.querySelectorAll(".bf-chain").forEach(chain);
    document.querySelectorAll(".bf-bitbuf").forEach(bitbuf);
    document.querySelectorAll(".bf-anim-controls").forEach(animated);
  };
  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(init);
  else if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
