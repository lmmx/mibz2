/* "Behind the scenes" (background/behind-the-scenes.md): the Observer's record, read at view
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
  function render(md, ctx, dir, account) {
    const body = md.replace(/^\s*#\s[^\n]*\n/, "");
    const holder = document.createElement("div");
    holder.innerHTML = window.DOMPurify.sanitize(window.marked.parse(body));
    /* A file's own headings sit below the page's sections (h2): the account's questions as h3,
       anything inside a fold as h4. */
    for (const h of holder.querySelectorAll("h1, h2, h3")) {
      const sub = document.createElement(account && h.tagName !== "H3" ? "h3" : "h4");
      sub.innerHTML = h.innerHTML;
      h.replaceWith(sub);
    }
    /* A table scrolls sideways on a narrow screen, as the site's own tables do */
    for (const t of holder.querySelectorAll("table")) {
      const wrap = el("div", { class: "md-typeset__scrollwrap" }, [el("div", { class: "md-typeset__table" })]);
      t.replaceWith(wrap);
      wrap.firstChild.appendChild(t);
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

  const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August",
    "September", "October", "November", "December"];
  function day(iso, time) {
    const d = new Date(iso);
    const t = time || `${d.toISOString().slice(11, 16)} UTC`;
    return `${d.getUTCDate()} ${MONTHS[d.getUTCMonth()]} ${d.getUTCFullYear()}, ${t}`;
  }

  /* The account's opening (everything before its first question) in its own box. */
  function lead(node) {
    const box = el("div", { class: "obs-lead" });
    while (node.firstChild && node.firstChild.tagName !== "H3") box.appendChild(node.firstChild);
    node.insertBefore(box, node.firstChild);
    return node;
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
      const d = el("details", {}, [el("summary", { text: "What it said then" }), l0]);
      d.addEventListener("toggle", () => {
        if (!d.open || l0.dataset.loaded) return;
        l0.dataset.loaded = "1";
        l0.textContent = "Loading…";
        const then = { repo: ctx.repo, ref: c.sha };
        getText(`${RAW}/${ctx.repo}/${c.sha}/current/account.md`)
          .then((md) => lead(render(md, then, "current/", true)))
          .catch(() => getText(`${RAW}/${ctx.repo}/${c.sha}/current/L0.md`).then((md) => render(md, then, "current/")))
          .then((node) => { l0.textContent = ""; l0.appendChild(node); })
          .catch((e) => { l0.textContent = `Not available (${e.message}).`; });
      });
      list.appendChild(el("li", { class: i === 0 ? "obs-move obs-move--current" : "obs-move" }, [
        el("p", { class: "obs-move__when", text: day(c.commit.committer.date) + (i === 0 ? " · the version shown above" : "") }),
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
    /* With the version's file list, ask only for files it has; without one (no API), try them all. */
    const wanted = ["current/account.md", "current/L0.md", "current/L1.md", "current/L2.md", "current/weak.md", "manifest.md", "changes.md", ...fronts]
      .filter((p) => (head ? files.includes(p) || p === "current/L0.md" : p !== "current/account.md"));
    return Promise.all(wanted.map((p) => getText(at(p)).catch(() => null))).then((texts) => {
      const got = Object.fromEntries(wanted.map((p, i) => [p, texts[i]]));
      if (!got["current/account.md"] && !got["current/L0.md"]) throw new Error("the record has no account");
      const top = document.createDocumentFragment();
      const out = document.createDocumentFragment();

      /* When the evidence was last checked, from the manifest; the commit's time otherwise. */
      const man = got["manifest.md"] || "";
      const read = man.match(/evidence inspected (\d{4}-\d\d-\d\d) ([0-9:\u2013-]+ UTC)/);
      const checked = read ? day(`${read[1]}T00:00:00Z`, read[2]) : head ? day(head.commit.committer.date) : null;
      const again = el("a", { href: "#", class: "obs-again", text: "read again" });
      again.addEventListener("click", (e) => { e.preventDefault(); start(root, true); });
      top.appendChild(el("p", { class: "obs-stamp" }, [
        document.createTextNode(checked ? `Last checked ${checked}` : "The latest account"),
        head ? document.createTextNode(" · ") : null,
        head ? el("a", { href: `https://github.com/${ctx.repo}/commit/${head.sha}`, rel: "noopener", text: "this version" }) : null,
        document.createTextNode(" · "),
        again,
      ].filter(Boolean)));

      /* The account, or, for a version written before there was one, its one line and five. */
      if (got["current/account.md"]) {
        top.appendChild(el("section", { class: "obs-account", id: "account" }, [lead(render(got["current/account.md"], ctx, "current/", true))]));
      } else {
        const older = el("div", { class: "obs-md" }, [render(got["current/L0.md"], ctx, "current/")]);
        lead(older);
        if (!head) {
          older.appendChild(el("p", { class: "obs-note" }, [
            document.createTextNode("GitHub turned this page away, so this is the short version. The full account is "),
            el("a", { href: `https://github.com/${ctx.repo}/tree/${ctx.ref}/current`, rel: "noopener", text: "on GitHub" }),
            document.createTextNode("."),
          ]));
        } else if (got["current/L1.md"]) older.appendChild(render(got["current/L1.md"], ctx, "current/"));
        top.appendChild(el("section", { class: "obs-account", id: "account" }, [older]));
      }

      /* Underneath: the record the account is written from, each part folded. */
      const parts = el("div", { class: "obs-fronts" });
      if (got["current/L2.md"]) parts.appendChild(disclose("the-plan", "The plan, as a tree", "each line of work and what it serves", render(got["current/L2.md"], ctx, "current/"), false));
      for (const p of fronts) {
        const md = got[p];
        if (!md) continue;
        const name = p.split("/").pop().replace(/\.md$/, "");
        parts.appendChild(disclose(`front-${name}`, frontTitle(md, name), "its reasons, status and sources", render(md, ctx, "current/L3/"), false));
      }
      if (got["current/weak.md"]) parts.appendChild(disclose("weak", "Where the reasoning is weak", "reasons that are stale, contradicted or unestablished", render(got["current/weak.md"], ctx, "current/"), false));
      if (got["manifest.md"]) parts.appendChild(disclose("evidence", "What this version rests on", "the evidence checked, and when", render(got["manifest.md"], ctx, ""), false));
      out.appendChild(section("how-we-know", "How we know", parts, "The account above is written from this record. Each part opens onto its evidence."));

      const history = el("div");
      if (commits && commits.length) history.appendChild(moves(ctx, commits));
      if (got["changes.md"]) history.appendChild(disclose("changes", "What changed, version by version", "in the Observer's words", render(got["changes.md"], ctx, ""), !commits));
      out.appendChild(section("earlier-versions", "Earlier versions", history, commits ? `${commits.length} version${commits.length === 1 ? "" : "s"}, newest first.` : null));

      const nowBox = document.getElementById("obs-now");
      if (nowBox) { nowBox.textContent = ""; nowBox.appendChild(top); }
      else out.insertBefore(top, out.firstChild);
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
          .catch(() => {
            /* Without the API there is no commit to pin every file to, and files read by the
               branch's name could come from two versions: show none rather than a mixture. */
            throw new Error("unpinned");
          }),
      )
      .then(({ ctx, files, head, commits }) => {
        if (head) root.dataset.sha = head.sha;
        return build(root, ctx, files, head, commits);
      })
      .catch((e) => {
        if (again) return;
        const p = root.querySelector(".obs-status") || root.appendChild(el("p", { class: "obs-status" }));
        p.textContent = "";
        p.append(
          e.message === "unpinned"
            ? "GitHub isn't answering this page just now (it allows 60 requests an hour from one address), so it can't be sure every part comes from one version of the account. Read it "
            : `The account could not be read here (${e.message}). Read it `,
          el("a", { href: `https://github.com/${repo}/tree/${branch}/current`, rel: "noopener", text: "on GitHub" }),
          ", or try again later.",
        );
      });
  }
  window.addEventListener("hashchange", onHash);
  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(init);
  else if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
