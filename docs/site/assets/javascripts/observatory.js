/* "Behind the scenes" (background/behind-the-scenes.md): the Oracle's record, read at view
   time from the orphan branch observatory/intention. One API call finds the branch's newest
   commit, one lists its files; every level is then fetched from raw.githubusercontent.com
   at that commit, so the levels on screen are one settled version. If the API refuses
   (60 requests an hour per address), the files are read at the branch name instead, and
   the history is left out. Markdown is rendered with marked and sanitised with DOMPurify,
   both pinned and loaded only on this page. */
(function () {
  const LIBS = [
    "https://cdn.jsdelivr.net/npm/marked@12.0.2/marked.min.js",
    "https://cdn.jsdelivr.net/npm/dompurify@3.1.6/dist/purify.min.js",
  ];
  const API = "https://api.github.com";
  const RAW = "https://raw.githubusercontent.com";
  let libs = null;

  function load(src) {
    return new Promise((resolve, reject) => {
      const s = document.createElement("script");
      s.src = src;
      s.onload = resolve;
      s.onerror = () => reject(new Error(`could not load ${src}`));
      document.head.appendChild(s);
    });
  }
  function loadLibs() {
    if (window.marked && window.DOMPurify) return Promise.resolve();
    if (!libs) libs = LIBS.reduce((p, src) => p.then(() => load(src)), Promise.resolve());
    return libs;
  }

  function el(tag, attrs, children) {
    const e = document.createElement(tag);
    for (const [k, v] of Object.entries(attrs || {})) {
      if (k === "text") e.textContent = v;
      else if (k === "html") e.innerHTML = v;
      else e.setAttribute(k, v);
    }
    for (const c of children || []) if (c) e.appendChild(c);
    return e;
  }

  function getJSON(url) {
    return fetch(url).then((r) => {
      if (!r.ok) throw new Error(`${r.status} from the GitHub API`);
      return r.json();
    });
  }
  function getText(url) {
    return fetch(url).then((r) => {
      if (!r.ok) throw new Error(`${r.status} for ${url.split("/").slice(-2).join("/")}`);
      return r.text();
    });
  }

  /* Each level file opens with its own "# L…" heading, which the page's section heading
     replaces. Relative links are made absolute at the version shown: L3/x.md becomes the
     in-page front, anything else the file on GitHub at that commit. */
  function render(md, ctx, dir) {
    const body = md.replace(/^\s*#\s[^\n]*\n/, "");
    const holder = document.createElement("div");
    holder.innerHTML = window.DOMPurify.sanitize(window.marked.parse(body));
    /* A file's own headings sit below the page's sections (h2) and a front's fold. */
    for (const h of holder.querySelectorAll("h1, h2, h3")) {
      const h4 = document.createElement("h4");
      h4.innerHTML = h.innerHTML;
      h.replaceWith(h4);
    }
    for (const a of holder.querySelectorAll("a[href]")) {
      const href = a.getAttribute("href");
      if (/^[a-z]+:|^#/.test(href)) {
        if (/^https?:/.test(href)) a.setAttribute("rel", "noopener");
        continue;
      }
      const path = new URL(href, `https://x/${dir}`).pathname.slice(1);
      const front = path.match(/^current\/L3\/([^/]+)\.md$/);
      if (front) a.setAttribute("href", `#front-${front[1]}`);
      else a.setAttribute("href", `https://github.com/${ctx.repo}/blob/${ctx.ref}/${path}`);
    }
    holder.className = "obs-md";
    return holder;
  }

  function section(id, title, node, note) {
    return el("section", { class: "obs-level", id }, [
      el("h2", { text: title }),
      note ? el("p", { class: "obs-note", text: note }) : null,
      node,
    ]);
  }

  function disclose(id, title, sub, node, open) {
    const d = el("details", { class: "obs-front", id }, [
      el("summary", {}, [el("span", { class: "obs-front__title", text: title }), sub ? el("span", { class: "obs-front__sub", text: sub }) : null]),
      node,
    ]);
    if (open) d.open = true;
    return d;
  }

  function when(iso) {
    const d = new Date(iso);
    return `${d.toISOString().slice(0, 10)} ${d.toISOString().slice(11, 16)} UTC`;
  }

  function frontTitle(md, file) {
    const m = md.match(/^\s*#\s+(.*)$/m);
    return m ? m[1].trim() : file.replace(/\.md$/, "");
  }

  /* Every move: one entry per commit, newest first, each with its diff from the version
     before it and, on opening, that version's L0. */
  function moves(ctx, commits) {
    const list = el("ol", { class: "obs-moves" });
    commits.forEach((c, i) => {
      const prev = commits[i + 1];
      const [subject] = c.commit.message.split("\n");
      const links = el("p", { class: "obs-move__links" }, [
        el("a", { href: c.html_url, rel: "noopener", text: c.sha.slice(0, 8) }),
        prev ? document.createTextNode(" · ") : null,
        prev ? el("a", { href: `https://github.com/${ctx.repo}/compare/${prev.sha}...${c.sha}`, rel: "noopener", text: "what changed" }) : null,
        document.createTextNode(" · "),
        el("a", { href: `https://github.com/${ctx.repo}/tree/${c.sha}`, rel: "noopener", text: "files at this version" }),
      ]);
      const l0 = el("div", { class: "obs-move__l0" });
      const d = el("details", {}, [el("summary", { text: "L0 at this version" }), l0]);
      d.addEventListener("toggle", () => {
        if (!d.open || l0.dataset.loaded) return;
        l0.dataset.loaded = "1";
        l0.textContent = "Loading…";
        getText(`${RAW}/${ctx.repo}/${c.sha}/current/L0.md`)
          .then((md) => { l0.textContent = ""; l0.appendChild(render(md, { repo: ctx.repo, ref: c.sha }, "current/")); })
          .catch((e) => { l0.textContent = `Not available (${e.message}).`; });
      });
      list.appendChild(el("li", { class: i === 0 ? "obs-move obs-move--current" : "obs-move" }, [
        el("p", { class: "obs-move__when", text: when(c.commit.committer.date) + (i === 0 ? " · shown above" : "") }),
        el("p", { class: "obs-move__subject", text: subject }),
        links,
        d,
      ]));
    });
    return list;
  }

  function build(root, ctx, files, head, commits) {
    const at = (p) => `${RAW}/${ctx.repo}/${ctx.ref}/${p}`;
    const fronts = files.filter((p) => /^current\/L3\/[^/]+\.md$/.test(p)).sort();
    const wanted = ["current/L0.md", "current/L1.md", "current/L2.md", "current/weak.md", "manifest.md", "changes.md", ...fronts];
    return Promise.all(wanted.map((p) => getText(at(p)).catch(() => null))).then((texts) => {
      const got = Object.fromEntries(wanted.map((p, i) => [p, texts[i]]));
      if (!got["current/L0.md"]) throw new Error("current/L0.md is missing from the record");
      const out = document.createDocumentFragment();

      const man = got["manifest.md"] || "";
      const through = (man.match(/through \[(\d+)\]/) || [])[1];
      const read = (man.match(/evidence inspected ([^,;*]+)/) || [])[1];
      const stamp = head
        ? [document.createTextNode("Account "),
           el("a", { href: `https://github.com/${ctx.repo}/commit/${head.sha}`, rel: "noopener", text: head.sha.slice(0, 8) }),
           document.createTextNode(`, settled ${when(head.commit.committer.date)}`),
           through ? document.createTextNode(`; covers the channel through ${through}`) : null,
           read ? document.createTextNode(`; evidence read ${read.trim()}`) : null]
        : [document.createTextNode("The branch's latest files. GitHub's API was unavailable, so the account's commit and its history are not shown")];
      const again = el("a", { href: "#", class: "obs-again", text: "read again" });
      again.addEventListener("click", (e) => { e.preventDefault(); start(root, true); });
      out.appendChild(el("p", { class: "obs-stamp" }, [...stamp.filter(Boolean), document.createTextNode(" · "), again]));
      const jumps = [["now", "Now"], ["in-five-lines", "In five lines"], ["the-tree", "The tree"], ["each-front", "Each front"],
        ["where-the-why-is-weak", "Weak links"], ["this-version", "This version"], ["every-move", "Every move"]];
      const nav = el("nav", { class: "obs-jumps", "aria-label": "Levels of the record" });
      jumps.forEach(([id, label], i) => {
        if (i) nav.appendChild(document.createTextNode(" · "));
        nav.appendChild(el("a", { href: `#${id}`, text: label }));
      });
      out.appendChild(nav);

      out.appendChild(section("now", "Now", render(got["current/L0.md"], ctx, "current/")));
      if (got["current/L1.md"]) out.appendChild(section("in-five-lines", "In five lines", render(got["current/L1.md"], ctx, "current/")));
      if (got["current/L2.md"]) out.appendChild(section("the-tree", "The tree", render(got["current/L2.md"], ctx, "current/")));

      if (fronts.length) {
        const wrap = el("div", { class: "obs-fronts" });
        for (const p of fronts) {
          const md = got[p];
          if (!md) continue;
          const name = p.split("/").pop().replace(/\.md$/, "");
          wrap.appendChild(disclose(`front-${name}`, frontTitle(md, name), null, render(md, ctx, "current/L3/"), false));
        }
        out.appendChild(section("each-front", "Each front", wrap, "Each opens onto its reasons, its status and its sources."));
      }
      if (got["current/weak.md"]) {
        out.appendChild(section("where-the-why-is-weak", "Where the “why” is weak", disclose("weak-detail", "Weak links in the chain of reasons", null, render(got["current/weak.md"], ctx, "current/"), false)));
      }
      if (got["manifest.md"]) {
        out.appendChild(section("this-version", "This version", disclose("manifest-detail", "Manifest", "what was covered, and the evidence checked", render(got["manifest.md"], ctx, ""), false)));
      }
      const movesBody = el("div");
      if (commits && commits.length) movesBody.appendChild(moves(ctx, commits));
      if (got["changes.md"]) movesBody.appendChild(disclose("changes-detail", "The Oracle's own change log", "changes.md", render(got["changes.md"], ctx, ""), !commits));
      out.appendChild(section("every-move", "Every move", movesBody, commits ? `${commits.length} settled version${commits.length === 1 ? "" : "s"}, newest first.` : null));

      root.textContent = "";
      root.appendChild(out);
      if (location.hash) {
        const t = document.getElementById(location.hash.slice(1));
        if (t) { if (t.tagName === "DETAILS") t.open = true; t.scrollIntoView(); }
      }
    });
  }

  function onHash() {
    const t = document.getElementById(location.hash.slice(1));
    if (t && t.tagName === "DETAILS") t.open = true;
  }

  /* Every five minutes while the page is visible, one API call asks for the branch's head;
     the account is read again only if it has moved. */
  let timer = null;
  function watch(root) {
    clearInterval(timer);
    timer = setInterval(() => {
      if (!document.body.contains(root)) { clearInterval(timer); return; }
      if (document.hidden || !root.dataset.sha) return;
      getJSON(`${API}/repos/${root.dataset.repo}/commits?sha=${encodeURIComponent(root.dataset.branch)}&per_page=1`)
        .then((c) => { if (c[0] && c[0].sha !== root.dataset.sha) start(root, true); })
        .catch(() => {});
    }, 300000);
  }

  function init() {
    const root = document.getElementById("obs-root");
    if (!root || root.dataset.started) return;
    root.dataset.started = "1";
    start(root, false);
    watch(root);
  }

  function start(root, again) {
    const repo = root.dataset.repo;
    const branch = root.dataset.branch;
    const enc = encodeURIComponent(branch);
    loadLibs()
      .then(() =>
        getJSON(`${API}/repos/${repo}/commits?sha=${enc}&per_page=100`)
          .then((commits) =>
            getJSON(`${API}/repos/${repo}/git/trees/${commits[0].sha}?recursive=1`).then((tree) => ({
              ctx: { repo, ref: commits[0].sha },
              files: tree.tree.filter((t) => t.type === "blob").map((t) => t.path),
              head: commits[0],
              commits,
            })),
          )
          .catch(() =>
            /* No API: the branch's files by name, the fronts found from L2's links. */
            getText(`${RAW}/${repo}/${branch}/current/L2.md`)
              .catch(() => "")
              .then((l2) => ({
                ctx: { repo, ref: branch },
                files: [...new Set([...l2.matchAll(/\]\((L3\/[^)\s]+\.md)\)/g)].map((m) => `current/${m[1]}`))],
                head: null,
                commits: null,
              })),
          ),
      )
      .then(({ ctx, files, head, commits }) => {
        if (head) root.dataset.sha = head.sha;
        return build(root, ctx, files, head, commits);
      })
      .catch((e) => {
        if (again) return;
        const p = root.querySelector(".obs-status") || root.appendChild(el("p", { class: "obs-status" }));
        p.insertBefore(document.createTextNode(`The account could not be read here (${e.message}). `), p.firstChild);
      });
  }
  window.addEventListener("hashchange", onHash);
  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(init);
  else if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
