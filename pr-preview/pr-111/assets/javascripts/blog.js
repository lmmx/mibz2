// The blog's interactive figures (docs/site/blog/).
//
// - .bf-chain: the pointer chain. "Follow pointer" moves from the current card to the card
//   it names, until every card has been visited; "Reset" goes back to card 0. Without
//   JavaScript the figure shows the whole path instead.
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

  const init = () => document.querySelectorAll(".bf-chain").forEach(chain);
  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(init);
  else if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
