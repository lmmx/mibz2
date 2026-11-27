// The blog's copy-announcement control (claim blog-announcement-copy).
//
// A small loudhailer button on each post card of the blog index, and one on each post's own
// page, copies the post's social announcement as one line of plain text:
//
//   <kind emoji> mibz2 blog ⠶ <title> <canonical URL>
//
// The emoji says what kind of post it is, read from the same label that colours the post's card
// (social_cards.py KINDS): 💡 a result, 🧩 a synthesis, 🔬 research in progress, ❓ an open
// question, 🧭 meta. An unknown label keeps the loudhailer, 📣.
//
// Nothing is written per post. On the index the title is the card's .post-card-title and the
// URL is the card's link resolved against the page's canonical URL, so a newly indexed post
// gets its control with the index entry. On a post's page the title is its og:title (the
// front matter's title) and the URL its canonical link. "Copied" shows only once the
// clipboard write has succeeded; if the browser refuses it, a small panel shows the line
// selected, to copy by hand. docs/scripts/check_announce.py tests all of this in Chromium.
//
// The button is a sibling of the card's link, never inside it: it is appended to the card's
// grid container and placed in the card's own grid row, top right (blog.css), so the cards'
// markup and sibling order stay as check_site.py and blog.css expect.
//
// Runs on load and on each instant navigation, as blog.js does.
(() => {
  "use strict";

  const PREFIX = " mibz2 blog ⠶ ";
  // The post's kind, by the label on its card or meta line, as its emoji (social_cards.py KINDS).
  const EMOJI = {
    "Research result": "\u{1F4A1}",
    "Research synthesis": "\u{1F9E9}",
    "Research in progress": "\u{1F52C}",
    "Work in progress": "\u{1F52C}",
    "Investigation in progress": "\u{1F52C}",
    "Open question": "\u{2753}",
    "Meta": "\u{1F9ED}",
  };
  const FALLBACK = "\u{1F4E3}";
  const emojiOf = (el) => (el && EMOJI[el.textContent.split("·").pop().trim()]) || FALLBACK;
  const LABEL = "Copy announcement";
  const POST = /\/blog\/(\d{4}-\d{2}-\d{2}-[^/]+)\/$/;
  const ICON =
    '<svg class="post-announce-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">' +
    '<path d="M3.5 10.2v3.6c0 .6.4 1 1 1h2.6l8.4 4.6V4.6L7.1 9.2H4.5c-.6 0-1 .4-1 1z"/>' +
    '<path d="M7.6 14.8l1.5 4.6h2.1l-1.3-4"/>' +
    '<path d="M18.6 9.2a4 4 0 0 1 0 5.6M20.6 7a7 7 0 0 1 0 10"/></svg>' +
    '<svg class="post-announce-done" viewBox="0 0 24 24" aria-hidden="true" focusable="false">' +
    '<path d="M5 12.5l4.5 4.5L19 7.5"/></svg>';

  const canonical = () => {
    const link = document.querySelector('link[rel="canonical"]');
    return link ? link.href : location.href;
  };

  const status = () => {
    let s = document.querySelector(".post-announce-status");
    if (!s) {
      s = document.createElement("div");
      s.className = "post-announce-status";
      s.setAttribute("role", "status");
      s.setAttribute("aria-live", "polite");
      document.body.appendChild(s);
    }
    return s;
  };

  const writeClipboard = async (text) => {
    try {
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(text);
        return true;
      }
    } catch (_) {
      /* fall through to the older route */
    }
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.setAttribute("readonly", "");
    ta.style.cssText = "position:fixed;top:0;left:0;width:1px;height:1px;opacity:0;";
    document.body.appendChild(ta);
    const active = document.activeElement;
    ta.select();
    let ok = false;
    try {
      ok = document.execCommand("copy") === true;
    } catch (_) {
      ok = false;
    }
    ta.remove();
    if (active && active.focus) active.focus();
    return ok;
  };

  let panel = null;
  const closeFallback = () => {
    if (!panel) return;
    const back = panel._from;
    panel.remove();
    panel = null;
    if (back && back.isConnected) back.focus();
  };
  const showFallback = (text, from) => {
    closeFallback();
    panel = document.createElement("div");
    panel.className = "post-announce-fallback";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-label", "Announcement text");
    panel._from = from;
    const note = document.createElement("p");
    note.textContent = "Copying was blocked. The announcement is selected: copy it from here.";
    const field = document.createElement("textarea");
    field.readOnly = true;
    field.rows = 3;
    field.value = text;
    field.setAttribute("aria-label", "Announcement");
    const close = document.createElement("button");
    close.type = "button";
    close.textContent = "Close";
    close.addEventListener("click", closeFallback);
    panel.append(note, field, close);
    panel.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        e.preventDefault();
        closeFallback();
      }
    });
    document.body.appendChild(panel);
    field.focus();
    field.select();
    field.setSelectionRange(0, text.length);
  };

  const button = (slug, text) => {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "post-announce";
    b.dataset.post = slug;
    b.setAttribute("aria-label", LABEL);
    b.title = LABEL;
    b.innerHTML = ICON;
    let timer = 0;
    b.addEventListener("click", async (e) => {
      e.preventDefault();
      e.stopPropagation();
      delete b.dataset.state;
      if (await writeClipboard(text)) {
        closeFallback();
        b.dataset.state = "copied";
        b.title = "Copied";
        const s = status();
        s.textContent = "";
        s.textContent = "Copied announcement";
        clearTimeout(timer);
        timer = setTimeout(() => {
          delete b.dataset.state;
          b.title = LABEL;
        }, 1800);
      } else {
        showFallback(text, b);
      }
    });
    return b;
  };

  const line = (emoji, title, url) =>
    emoji + PREFIX + title.replace(/\s+/g, " ").trim() + " " + url;

  // A card's link as its canonical URL. The theme may already have made the link absolute
  // against wherever the page is served (a local preview, say), so take its path relative to
  // this page's directory and resolve that against the page's canonical URL.
  const canonicalOf = (a, base) => {
    const here = new URL(".", location.href);
    const link = new URL(a.getAttribute("href"), location.href);
    if (link.origin === here.origin && link.pathname.startsWith(here.pathname)) {
      return new URL(link.pathname.slice(here.pathname.length), base).href;
    }
    return link.href;
  };

  const index = () => {
    const base = canonical();
    document.querySelectorAll(".post-list").forEach((list) => {
      if (list.dataset.announce) return;
      list.dataset.announce = "1";
      const grids = [list, ...list.querySelectorAll(":scope > .post-chain")];
      grids.forEach((grid) => {
        let row = 0;
        const add = [];
        [...grid.children].forEach((child) => {
          if (child.classList.contains("post-announce")) return;
          row += 1;
          child.style.gridRow = String(row);
          child.style.gridColumn = "1";
          if (!child.matches("a.post-card")) return;
          const title = child.querySelector(".post-card-title");
          const href = child.getAttribute("href");
          if (!title || !href) return;
          const url = canonicalOf(child, base);
          const m = POST.exec(new URL(url).pathname);
          if (!m) return;
          const b = button(m[1], line(emojiOf(child.querySelector(".post-card-date")), title.textContent, url));
          b.style.gridRow = String(row);
          b.style.gridColumn = "1";
          add.push(b);
        });
        add.forEach((b) => grid.appendChild(b));
      });
    });
  };

  const post = () => {
    const url = canonical();
    const m = POST.exec(new URL(url).pathname);
    if (!m) return;
    const article = document.querySelector("article");
    if (!article || article.querySelector(".post-announce")) return;
    const h1 = article.querySelector("h1");
    if (!h1) return;
    const og = document.querySelector('meta[property="og:title"]');
    let title = og && og.content;
    if (!title) {
      const c = h1.cloneNode(true);
      c.querySelectorAll(".headerlink").forEach((a) => a.remove());
      title = c.textContent.replace(/[­​-‍⁠]/g, "");
    }
    const labels = article.querySelectorAll(".post-meta a");
    const b = button(m[1], line(emojiOf(labels[labels.length - 1]), title, url));
    const meta = article.querySelector(".post-meta");
    if (meta) {
      b.classList.add("post-announce--inline");
      meta.appendChild(b);
    } else {
      b.classList.add("post-announce--heading");
      h1.insertAdjacentElement("afterend", b);
    }
  };

  const init = () => {
    if (document.querySelector(".post-list")) index();
    else post();
  };

  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(init);
  else if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
