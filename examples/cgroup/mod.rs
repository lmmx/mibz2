//! `ab env`'s CPU-bandwidth census: the cgroup hierarchy this process is in, resolved from
//! `/proc/self/cgroup` and `/proc/self/mountinfo`, every level from the leaf up to the
//! visible boundary read, and coverage kept apart from the tightest limit
//! (docs/journal/2026-11-09-c29-follow-up-registration.md, after the Researcher's REVIEW
//! 6735626248).
//!
//! - **The hierarchy:** a v1 line whose controllers include `cpu` (`cpu` or a combined
//!   `cpu,cpuacct`), which binds the controller there; else the v2 line (`0::PATH`).
//! - **The mount:** a `cgroup2` mount (v2), or a `cgroup` mount whose super options carry
//!   every controller of that line (v1), whose root (mountinfo's field 4, unescaped) is the
//!   membership path or an ancestor of it; the shortest such root. A path or root with a
//!   `..` component, a level whose directory another mount covers, or no such mount: unknown.
//! - **Each level** from the leaf to the mount's root: v2's `cpu.max` (`Q P` or `max P`,
//!   integers, both positive), or its justified absence (the level's `cgroup.controllers`
//!   readable and without `cpu`: no bandwidth limit of its own); v1's `cpu.cfs_quota_us`
//!   (`-1`, unlimited) and `cpu.cfs_period_us`. Anything else (a read error, a missing file,
//!   a malformed one) is unknown, its reason kept.
//! - **The boundary** is the hierarchy's real root only when the mount's root is `/` and
//!   this process's cgroup namespace is the initial one (`cgroup:[4026531835]`, the kernel's
//!   `PROC_CGROUP_INIT_INO`): a visible `/` alone is not. There, in v2, `cpu.max` and
//!   `cgroup.type` are absent (both exist only on non-root cgroups): a justified absence.
//!   Elsewhere the ancestors above the boundary are hidden, and coverage is incomplete.
//! - **The state:** `unlimited` or `finite` only with complete coverage (every level
//!   justified, the boundary the real root), the tightest limit by exact integer comparison;
//!   else `unknown`, with the tightest observed limit kept as an upper bound only.
//!
//! Every read goes through [`Fs`], so the tests run it over fake `/proc` and `/sys` trees.

#[cfg(test)]
use std::collections::BTreeMap;

/// The kernel's `PROC_CGROUP_INIT_INO` (0xEFFFFFFB): the initial cgroup namespace
pub const INIT_CGROUP_NS: &str = "cgroup:[4026531835]";
pub const SCHEMA: &str = "ab/cgroup/1";

/// A file read: its text, absent (ENOENT), or another error
#[derive(Clone, Debug, PartialEq)]
pub enum Got {
    Text(String),
    Missing,
    Error(String),
}

impl Got {
    fn text(&self) -> Option<&str> {
        match self {
            Got::Text(s) => Some(s),
            _ => None,
        }
    }

    fn json(&self) -> J {
        match self {
            Got::Text(s) => J::obj([("text", J::s(s))]),
            Got::Missing => J::obj([("missing", J::Bool(true))]),
            Got::Error(e) => J::obj([("error", J::s(e))]),
        }
    }
}

/// Where the census reads: the real filesystem, or a test's fake tree
pub trait Fs {
    fn read(&self, path: &str) -> Got;
    fn link(&self, path: &str) -> Got;
}

pub struct Real;

fn got(r: std::io::Result<String>) -> Got {
    match r {
        Ok(s) => Got::Text(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Got::Missing,
        Err(e) => Got::Error(match e.raw_os_error() {
            Some(n) => format!("errno {n}: {e}"),
            None => e.to_string(),
        }),
    }
}

impl Fs for Real {
    fn read(&self, path: &str) -> Got {
        got(std::fs::read_to_string(path))
    }
    fn link(&self, path: &str) -> Got {
        got(std::fs::read_link(path).map(|p| p.to_string_lossy().into_owned()))
    }
}

/// A minimal JSON value, written in key order as given
#[derive(Clone, Debug, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

impl J {
    pub fn s(s: &str) -> J {
        J::Str(s.to_string())
    }
    pub fn n(x: impl ToString) -> J {
        J::Num(x.to_string())
    }
    pub fn obj<const N: usize>(kv: [(&str, J); N]) -> J {
        J::Obj(kv.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
    pub fn write(&self, out: &mut String) {
        match self {
            J::Null => out.push_str("null"),
            J::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            J::Num(n) => out.push_str(n),
            J::Str(s) => {
                out.push('"');
                for c in s.chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\t' => out.push_str("\\t"),
                        '\r' => out.push_str("\\r"),
                        c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                        c => out.push(c),
                    }
                }
                out.push('"');
            }
            J::Arr(v) => {
                out.push('[');
                for (i, x) in v.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    x.write(out);
                }
                out.push(']');
            }
            J::Obj(kv) => {
                out.push('{');
                for (i, (k, v)) in kv.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    J::Str(k.clone()).write(out);
                    out.push(':');
                    v.write(out);
                }
                out.push('}');
            }
        }
    }
}

impl std::fmt::Display for J {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        self.write(&mut s);
        f.write_str(&s)
    }
}

/// One mountinfo line, its root and mount point unescaped
#[derive(Clone, Debug, PartialEq)]
pub struct Mount {
    pub id: u64,
    pub parent: u64,
    pub root: String,
    pub point: String,
    pub fstype: String,
    pub source: String,
    pub super_opts: String,
    pub raw: String,
}

/// mountinfo's octal escapes (`\040` space, `\011` tab, `\012` newline, `\134` backslash)
pub fn unescape(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\'
            && i + 3 < b.len()
            && b[i + 1..i + 4].iter().all(|c| (b'0'..=b'7').contains(c))
        {
            let v = (b[i + 1] - b'0') as u32 * 64
                + (b[i + 2] - b'0') as u32 * 8
                + (b[i + 3] - b'0') as u32;
            if v < 256 {
                out.push(v as u8);
                i += 4;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// mountinfo's lines (proc_pid_mountinfo(5)): the mounts, and the lines that don't parse
pub fn parse_mountinfo(text: &str) -> (Vec<Mount>, Vec<String>) {
    let mut mounts = Vec::new();
    let mut bad = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let f: Vec<&str> = line.split(' ').collect();
        let sep = f.iter().position(|x| *x == "-");
        let parsed = (|| {
            let sep = sep?;
            if sep < 6 || f.len() < sep + 4 {
                return None;
            }
            Some(Mount {
                id: f[0].parse().ok()?,
                parent: f[1].parse().ok()?,
                root: unescape(f[3]),
                point: unescape(f[4]),
                fstype: f[sep + 1].to_string(),
                source: unescape(f[sep + 2]),
                super_opts: f[sep + 3].to_string(),
                raw: line.to_string(),
            })
        })();
        match parsed {
            Some(m) => mounts.push(m),
            None => bad.push(line.to_string()),
        }
    }
    (mounts, bad)
}

/// One /proc/self/cgroup line: hierarchy ID, controllers, path
#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub id: String,
    pub controllers: Vec<String>,
    pub path: String,
    pub raw: String,
}

/// /proc/self/cgroup (cgroups(7)): the lines, and those that don't parse
pub fn parse_membership(text: &str) -> (Vec<Member>, Vec<String>) {
    let mut out = Vec::new();
    let mut bad = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let mut f = line.splitn(3, ':');
        match (f.next(), f.next(), f.next()) {
            (Some(id), Some(ctl), Some(path)) if path.starts_with('/') && !id.is_empty() => out
                .push(Member {
                    id: id.to_string(),
                    controllers: ctl
                        .split(',')
                        .filter(|c| !c.is_empty())
                        .map(String::from)
                        .collect(),
                    path: path.to_string(),
                    raw: line.to_string(),
                }),
            _ => bad.push(line.to_string()),
        }
    }
    (out, bad)
}

fn comps(p: &str) -> Vec<&str> {
    p.split('/')
        .filter(|c| !c.is_empty() && *c != ".")
        .collect()
}

fn cg_path(c: &[&str]) -> String {
    format!("/{}", c.join("/"))
}

fn join(point: &str, rel: &[&str]) -> String {
    if rel.is_empty() {
        point.to_string()
    } else {
        format!("{}/{}", point.trim_end_matches('/'), rel.join("/"))
    }
}

/// `a` is `b` or one of its ancestors, component-wise
fn ancestor_or_self(a: &str, b: &str) -> bool {
    let (a, b) = (comps(a), comps(b));
    a.len() <= b.len() && a[..] == b[..a.len()]
}

/// The mount a path resolves through: the longest mount point containing it, and of mounts
/// stacked on that point, the top one (the one no other names as its parent)
pub fn visible_mount(mounts: &[Mount], dir: &str) -> Option<usize> {
    let best = mounts
        .iter()
        .filter(|m| ancestor_or_self(&m.point, dir))
        .map(|m| comps(&m.point).len())
        .max()?;
    let at: Vec<usize> = (0..mounts.len())
        .filter(|&i| {
            ancestor_or_self(&mounts[i].point, dir) && comps(&mounts[i].point).len() == best
        })
        .collect();
    at.iter()
        .rev()
        .copied()
        .find(|&i| {
            !at.iter()
                .any(|&j| j != i && mounts[j].parent == mounts[i].id)
        })
        .or(at.last().copied())
}

/// Which cgroup version carries the CPU controller
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Version {
    V1,
    V2,
}

/// One visited level's reading
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// a finite limit: `quota` µs each `period` µs
    Limit { quota: u64, period: u64 },
    /// an explicit unlimited reading (`max`, v1's -1)
    Unlimited,
    /// no bandwidth limit of its own, and why that's known
    NoController(String),
    /// the hierarchy's real root (v2: no `cpu.max` there by definition), and why
    Root(String),
    /// not known, and why
    Unknown(String),
}

impl Kind {
    fn known(&self) -> bool {
        !matches!(self, Kind::Unknown(_))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Level {
    pub path: String,
    pub dir: String,
    pub kind: Kind,
    pub files: Vec<(String, Got)>,
}

/// A limit, and the level it's at
#[derive(Clone, Debug, PartialEq)]
pub struct Limit {
    pub quota: u64,
    pub period: u64,
    pub path: String,
}

/// The tightest of the limits, by exact comparison (q1 / p1 < q2 / p2 iff q1 p2 < q2 p1)
pub fn tightest<'a>(it: impl Iterator<Item = &'a Level>) -> Option<Limit> {
    let mut best: Option<Limit> = None;
    for l in it {
        if let Kind::Limit { quota, period } = l.kind {
            let tighter = match &best {
                None => true,
                Some(b) => {
                    (quota as u128) * (b.period as u128) < (b.quota as u128) * (period as u128)
                }
            };
            if tighter {
                best = Some(Limit {
                    quota,
                    period,
                    path: l.path.clone(),
                });
            }
        }
    }
    best
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Unlimited,
    Finite(Limit),
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Census {
    pub membership: Got,
    pub mountinfo: Got,
    pub ns_cgroup: Got,
    pub ns_mnt: Got,
    /// mountinfo's cgroup and cgroup2 lines, and every line a visited level resolves through
    pub mount_lines: Vec<String>,
    pub bad_lines: Vec<String>,
    pub version: Option<Version>,
    pub hybrid: bool,
    pub member: Option<Member>,
    pub mount: Option<Mount>,
    pub levels: Vec<Level>,
    pub boundary_real_root: bool,
    pub boundary_why: String,
    /// why coverage is incomplete (none: complete)
    pub gaps: Vec<String>,
    pub state: State,
    pub observed: Option<Limit>,
    pub cpuset: Option<(String, Got)>,
}

/// v2's `cpu.max`: `Q P` or `max P`, integers, both positive
pub fn parse_cpu_max(s: &str) -> Result<Kind, String> {
    let f: Vec<&str> = s.split_whitespace().collect();
    let pos = |x: &str| x.parse::<u64>().ok().filter(|&v| v > 0);
    match f[..] {
        ["max", p] if pos(p).is_some() => Ok(Kind::Unlimited),
        [q, p] => match (pos(q), pos(p)) {
            (Some(quota), Some(period)) => Ok(Kind::Limit { quota, period }),
            _ => Err(format!("cpu.max {s:?}: not two positive integers")),
        },
        _ => Err(format!("cpu.max {s:?}: not `Q P` or `max P`")),
    }
}

/// v1's `cpu.cfs_quota_us` (-1: unlimited) and `cpu.cfs_period_us`
pub fn parse_cfs(quota: &str, period: &str) -> Result<Kind, String> {
    let p = period.trim().parse::<u64>().ok().filter(|&v| v > 0);
    match (quota.trim().parse::<i64>(), p) {
        (Ok(-1), Some(_)) => Ok(Kind::Unlimited),
        (Ok(q), Some(period)) if q > 0 => Ok(Kind::Limit {
            quota: q as u64,
            period,
        }),
        _ => Err(format!(
            "cpu.cfs_quota_us {quota:?}, cpu.cfs_period_us {period:?}: not -1 or a positive quota, with a positive period"
        )),
    }
}

fn read_level(fs: &dyn Fs, v: Version, dir: &str, real_root: bool) -> (Kind, Vec<(String, Got)>) {
    let mut files = Vec::new();
    let mut rd = |name: &str| {
        let g = fs.read(&format!("{dir}/{name}"));
        files.push((name.to_string(), g.clone()));
        g
    };
    let kind = match v {
        Version::V2 => {
            let max = rd("cpu.max");
            let kind = if real_root {
                let ty = rd("cgroup.type");
                match (&max, &ty) {
                    (Got::Missing, Got::Missing) => Kind::Root(
                        "the hierarchy's root: cpu.max and cgroup.type exist only on non-root cgroups, and neither is here".into(),
                    ),
                    (Got::Text(_), _) => Kind::Unknown("a cpu.max at the claimed root: not the hierarchy's root".into()),
                    (_, Got::Text(_)) => Kind::Unknown("a cgroup.type at the claimed root: not the hierarchy's root".into()),
                    (Got::Error(e), _) | (_, Got::Error(e)) => Kind::Unknown(format!("at the claimed root: {e}")),
                }
            } else {
                match &max {
                    Got::Text(s) => parse_cpu_max(s).unwrap_or_else(Kind::Unknown),
                    Got::Error(e) => Kind::Unknown(format!("cpu.max: {e}")),
                    Got::Missing => match rd("cgroup.controllers") {
                        Got::Text(c) if !c.split_whitespace().any(|x| x == "cpu") => Kind::NoController(
                            "no cpu.max, and cgroup.controllers lacks cpu: no bandwidth limit at this level".into(),
                        ),
                        Got::Text(_) => Kind::Unknown("no cpu.max, though cgroup.controllers lists cpu".into()),
                        Got::Missing => Kind::Unknown("neither cpu.max nor cgroup.controllers".into()),
                        Got::Error(e) => Kind::Unknown(format!("no cpu.max; cgroup.controllers: {e}")),
                    },
                }
            };
            rd("cpu.max.burst");
            rd("cpu.weight");
            rd("cpu.stat");
            kind
        }
        Version::V1 => {
            let q = rd("cpu.cfs_quota_us");
            let p = rd("cpu.cfs_period_us");
            let kind = match (&q, &p) {
                (Got::Text(q), Got::Text(p)) => parse_cfs(q, p).unwrap_or_else(Kind::Unknown),
                (Got::Missing, _) | (_, Got::Missing) => {
                    Kind::Unknown("cpu.cfs_quota_us or cpu.cfs_period_us missing".into())
                }
                (Got::Error(e), _) | (_, Got::Error(e)) => Kind::Unknown(format!("cfs: {e}")),
            };
            rd("cpu.cfs_burst_us");
            rd("cpu.shares");
            rd("cpu.stat");
            kind
        }
    };
    (kind, files)
}

/// A hierarchy's mount for a membership line: the mount whose root contains its path, the
/// shortest root; Err, why there is none
fn select_mount<'a>(
    mounts: &'a [Mount],
    v: Version,
    m: &Member,
) -> Result<(usize, &'a Mount), String> {
    if comps(&m.path).contains(&"..") {
        return Err(format!(
            "the membership path {:?} leaves this cgroup namespace's root",
            m.path
        ));
    }
    let fits = |x: &Mount| match v {
        Version::V2 => x.fstype == "cgroup2",
        Version::V1 => {
            x.fstype == "cgroup"
                && m.controllers
                    .iter()
                    .all(|c| x.super_opts.split(',').any(|o| o == c))
        }
    };
    let cands: Vec<(usize, &Mount)> = mounts.iter().enumerate().filter(|(_, x)| fits(x)).collect();
    if cands.is_empty() {
        return Err(format!("no {v:?} mount carries the line {:?}", m.raw));
    }
    cands
        .into_iter()
        .filter(|(_, x)| !comps(&x.root).contains(&"..") && ancestor_or_self(&x.root, &m.path))
        .min_by_key(|(_, x)| comps(&x.root).len())
        .ok_or_else(|| {
            format!(
                "no {v:?} mount's root contains the membership path {:?}",
                m.path
            )
        })
}

/// The census: membership, mounts, every level, the boundary and the coverage
pub fn collect(fs: &dyn Fs) -> Census {
    let membership = fs.read("/proc/self/cgroup");
    let mountinfo = fs.read("/proc/self/mountinfo");
    let ns_cgroup = fs.link("/proc/self/ns/cgroup");
    let ns_mnt = fs.link("/proc/self/ns/mnt");
    let mut c = Census {
        membership: membership.clone(),
        mountinfo: mountinfo.clone(),
        ns_cgroup: ns_cgroup.clone(),
        ns_mnt,
        mount_lines: Vec::new(),
        bad_lines: Vec::new(),
        version: None,
        hybrid: false,
        member: None,
        mount: None,
        levels: Vec::new(),
        boundary_real_root: false,
        boundary_why: String::new(),
        gaps: Vec::new(),
        state: State::Unknown,
        observed: None,
        cpuset: None,
    };
    let (members, bad) = parse_membership(membership.text().unwrap_or(""));
    c.bad_lines
        .extend(bad.iter().map(|l| format!("/proc/self/cgroup: {l:?}")));
    let (mounts, bad) = parse_mountinfo(mountinfo.text().unwrap_or(""));
    c.bad_lines
        .extend(bad.iter().map(|l| format!("/proc/self/mountinfo: {l:?}")));
    c.mount_lines = mounts
        .iter()
        .filter(|m| m.fstype == "cgroup" || m.fstype == "cgroup2")
        .map(|m| m.raw.clone())
        .collect();
    if membership.text().is_none() {
        c.gaps.push(format!("/proc/self/cgroup: {:?}", membership));
    }
    if mountinfo.text().is_none() {
        c.gaps
            .push(format!("/proc/self/mountinfo: {:?}", mountinfo));
    }
    let v1 = members
        .iter()
        .find(|m| m.id != "0" && m.controllers.iter().any(|x| x == "cpu"));
    let v2 = members
        .iter()
        .find(|m| m.id == "0" && m.controllers.is_empty());
    c.hybrid = v1.is_some() && v2.is_some();
    let (v, member) = match (v1, v2) {
        (Some(m), _) => (Version::V1, m.clone()),
        (None, Some(m)) => (Version::V2, m.clone()),
        (None, None) => {
            c.gaps.push(
                "no cgroup line carries the cpu controller (no v1 cpu line, no v2 line)".into(),
            );
            return finish(c, &mounts, fs, &members);
        }
    };
    c.version = Some(v);
    c.member = Some(member.clone());
    let (mi, mount) = match select_mount(&mounts, v, &member) {
        Ok(x) => x,
        Err(e) => {
            c.gaps.push(e);
            return finish(c, &mounts, fs, &members);
        }
    };
    c.mount = Some(mount.clone());
    let root = comps(&mount.root);
    let path = comps(&member.path);
    c.boundary_real_root = root.is_empty() && ns_cgroup.text() == Some(INIT_CGROUP_NS);
    c.boundary_why = if c.boundary_real_root {
        format!("the mount's root is / and the cgroup namespace is the initial one ({INIT_CGROUP_NS}): the hierarchy's real root")
    } else if !root.is_empty() {
        format!(
            "the mount's root {:?} is below the hierarchy's root: its ancestors are not visible here",
            mount.root
        )
    } else {
        format!(
            "the cgroup namespace {:?} is not the initial one ({INIT_CGROUP_NS}): ancestors above its root are hidden",
            ns_cgroup
        )
    };
    if !c.boundary_real_root {
        c.gaps.push(c.boundary_why.clone());
    }
    for i in (root.len()..=path.len()).rev() {
        let dir = join(&mount.point, &path[root.len()..i]);
        let at_root = i == root.len() && c.boundary_real_root;
        let vis = visible_mount(&mounts, &dir);
        let (kind, files) = if vis != Some(mi) {
            (
                Kind::Unknown(format!(
                    "{dir} resolves through another mount ({:?}), not the selected hierarchy's",
                    vis.map(|j| &mounts[j].raw)
                )),
                Vec::new(),
            )
        } else {
            read_level(fs, v, &dir, at_root)
        };
        if let Kind::Unknown(why) = &kind {
            c.gaps.push(format!("{}: {why}", cg_path(&path[..i])));
        }
        c.levels.push(Level {
            path: cg_path(&path[..i]),
            dir,
            kind,
            files,
        });
    }
    finish(c, &mounts, fs, &members)
}

fn finish(mut c: Census, mounts: &[Mount], fs: &dyn Fs, members: &[Member]) -> Census {
    c.observed = tightest(c.levels.iter());
    let complete = c.gaps.is_empty()
        && !c.levels.is_empty()
        && c.boundary_real_root
        && c.levels.iter().all(|l| l.kind.known());
    c.state = match (complete, &c.observed) {
        (false, _) => State::Unknown,
        (true, None) => State::Unlimited,
        (true, Some(l)) => State::Finite(l.clone()),
    };
    // the effective cpuset at the leaf, where it can be read (descriptive only)
    let cs = members
        .iter()
        .find(|m| m.id != "0" && m.controllers.iter().any(|x| x == "cpuset"))
        .map(|m| (Version::V1, m))
        .or_else(|| {
            members
                .iter()
                .find(|m| m.id == "0" && m.controllers.is_empty())
                .map(|m| (Version::V2, m))
        });
    if let Some((v, m)) = cs {
        if let Ok((_, mount)) = select_mount(mounts, v, m) {
            let rel = &comps(&m.path)[comps(&mount.root).len()..];
            let file = if v == Version::V1 {
                "cpuset.effective_cpus"
            } else {
                "cpuset.cpus.effective"
            };
            let p = format!("{}/{file}", join(&mount.point, rel));
            let g = fs.read(&p);
            c.cpuset = Some((p, g));
        }
    }
    c
}

fn limit_json(l: &Option<Limit>) -> J {
    match l {
        None => J::Null,
        Some(l) => J::obj([
            ("quota", J::n(l.quota)),
            ("period", J::n(l.period)),
            ("path", J::s(&l.path)),
        ]),
    }
}

impl Census {
    /// The legacy reading (`ab env`'s `cpu_quota` and `quota_state`): Some(None)
    /// unlimited, Some(Some(CPUs)) finite, None unknown
    pub fn legacy(&self) -> Option<Option<f64>> {
        match &self.state {
            State::Unlimited => Some(None),
            State::Finite(l) => Some(Some(l.quota as f64 / l.period as f64)),
            State::Unknown => None,
        }
    }

    pub fn json(&self) -> J {
        let levels = self
            .levels
            .iter()
            .map(|l| {
                let (kind, quota, period, why) = match &l.kind {
                    Kind::Limit { quota, period } => ("limit", J::n(quota), J::n(period), J::Null),
                    Kind::Unlimited => ("unlimited", J::Null, J::Null, J::Null),
                    Kind::NoController(w) => ("no_controller", J::Null, J::Null, J::s(w)),
                    Kind::Root(w) => ("root", J::Null, J::Null, J::s(w)),
                    Kind::Unknown(w) => ("unknown", J::Null, J::Null, J::s(w)),
                };
                J::obj([
                    ("path", J::s(&l.path)),
                    ("dir", J::s(&l.dir)),
                    ("kind", J::s(kind)),
                    ("quota", quota),
                    ("period", period),
                    ("why", why),
                    (
                        "files",
                        J::Obj(l.files.iter().map(|(k, g)| (k.clone(), g.json())).collect()),
                    ),
                ])
            })
            .collect();
        let mount = match &self.mount {
            None => J::Null,
            Some(m) => J::obj([
                ("id", J::n(m.id)),
                ("parent", J::n(m.parent)),
                ("root", J::s(&m.root)),
                ("point", J::s(&m.point)),
                ("fstype", J::s(&m.fstype)),
                ("source", J::s(&m.source)),
                ("super", J::s(&m.super_opts)),
            ]),
        };
        let state = match &self.state {
            State::Unlimited => "unlimited",
            State::Finite(_) => "finite",
            State::Unknown => "unknown",
        };
        let tight = match &self.state {
            State::Finite(l) => Some(l.clone()),
            _ => None,
        };
        J::obj([
            ("schema", J::s(SCHEMA)),
            ("membership", self.membership.json()),
            (
                "mountinfo_error",
                match &self.mountinfo {
                    Got::Text(_) => J::Null,
                    g => g.json(),
                },
            ),
            (
                "mount_lines",
                J::Arr(self.mount_lines.iter().map(|s| J::s(s)).collect()),
            ),
            (
                "malformed",
                J::Arr(self.bad_lines.iter().map(|s| J::s(s)).collect()),
            ),
            (
                "ns",
                J::obj([
                    ("cgroup", self.ns_cgroup.json()),
                    ("mnt", self.ns_mnt.json()),
                ]),
            ),
            (
                "hierarchy",
                match self.version {
                    None => J::Null,
                    Some(Version::V1) => J::s("v1"),
                    Some(Version::V2) => J::s("v2"),
                },
            ),
            ("hybrid", J::Bool(self.hybrid)),
            (
                "membership_line",
                self.member.as_ref().map_or(J::Null, |m| J::s(&m.raw)),
            ),
            ("mount", mount),
            ("levels", J::Arr(levels)),
            (
                "boundary",
                J::obj([
                    (
                        "path",
                        self.levels.last().map_or(J::Null, |l| J::s(&l.path)),
                    ),
                    ("real_root", J::Bool(self.boundary_real_root)),
                    ("why", J::s(&self.boundary_why)),
                ]),
            ),
            (
                "coverage",
                J::s(if matches!(self.state, State::Unknown) {
                    "incomplete"
                } else {
                    "complete"
                }),
            ),
            ("gaps", J::Arr(self.gaps.iter().map(|s| J::s(s)).collect())),
            ("state", J::s(state)),
            ("tightest", limit_json(&tight)),
            ("observed_tightest", limit_json(&self.observed)),
            (
                "cpuset_effective",
                match &self.cpuset {
                    None => J::Null,
                    Some((p, g)) => J::obj([("file", J::s(p)), ("read", g.json())]),
                },
            ),
        ])
    }
}

/// A fake tree for the tests: absolute path to contents; anything else is missing
#[cfg(test)]
#[derive(Default)]
pub struct Fake {
    pub files: BTreeMap<String, Got>,
    pub links: BTreeMap<String, Got>,
}

#[cfg(test)]
impl Fake {
    pub fn file(mut self, p: &str, s: &str) -> Self {
        self.files.insert(p.into(), Got::Text(s.into()));
        self
    }
    pub fn err(mut self, p: &str, e: &str) -> Self {
        self.files.insert(p.into(), Got::Error(e.into()));
        self
    }
    pub fn link(mut self, p: &str, s: &str) -> Self {
        self.links.insert(p.into(), Got::Text(s.into()));
        self
    }
}

#[cfg(test)]
impl Fs for Fake {
    fn read(&self, p: &str) -> Got {
        self.files.get(p).cloned().unwrap_or(Got::Missing)
    }
    fn link(&self, p: &str) -> Got {
        self.links.get(p).cloned().unwrap_or(Got::Missing)
    }
}
