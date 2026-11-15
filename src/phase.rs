//! Research only, behind the `phase-counters` feature (off by default): the phase attribution
//! of Cells' memory traffic (#145, LOG 6884716103, amended by LOGs 6884895257 and
//! 6884975709, all before any data). Without the feature this module is not compiled and
//! nothing in the decoder refers to it.
//!
//! **What it counts.** One counter pass a process, chosen by `MIBZ2_PHASE_PASS` when the
//! process first asks (`pass`): A1, A2, B1 or B2, the registration's events (`PASSES`), or
//! unset, which counts nothing. Each pass's events are Intel's Skylake-client encodings (the
//! i9-10900K's map, 06_A5), opened raw: the event and umask in `config`, an offcore-response
//! filter in `config1`. Every event counts user and kernel mode both (CML070), never the
//! hypervisor, and only the thread that opened it (pid 0, any CPU, not inherited).
//!
//! **The scopes, per worker thread.** Each thread that decodes a block in the parallel
//! decoder's `worker` (`whole_begin` / `whole_end`, around `decode_block`) opens, the first
//! time, three groups of the pass's events:
//! - **whole**, enabled for each block's `decode_block`;
//! - **phase**, enabled only inside `induce_cells` (the induction) and `write_cells` (the
//!   writer), one at a time (`Scope`), counting beside whole: so the pass's events are
//!   scheduled twice at most, which the PMU fits (at most 4 general counters and 2
//!   offcore-response filters a logical CPU with SMT on);
//! - **empty**, enabled and at once disabled after each block, once for each phase scope the
//!   block had: the counts an empty scope takes, its overhead.
//!
//! At each phase scope's edges the whole group is read too, so that its own counts inside
//! the induction and writer windows are known: rest = whole less those, and the registered
//! check, B_ind + B_wr + B_rest = whole within 1%, compares the phase group's counts with the
//! whole group's over the same windows: two counters of one event, independently scheduled.
//! Requests in flight cross a window's edges: the phases are approximate, and the report says
//! so. Each window's time enabled and time running are summed, and a window that ran less than
//! it was enabled is counted as multiplexed (`mux`), which the analysis treats as invalid.
//!
//! `report` gives the process's sums as one JSON object, for `examples/ab.rs`'s child record.

use std::cell::RefCell;
use std::sync::{Mutex, OnceLock};

/// The most events a pass counts
const MAX: usize = 2;

/// One event: perf's type, its config, its config1 (the offcore-response filter), its name
pub type Event = (u32, u64, u64, &'static str);

const RAW: u32 = 4;
const SOFTWARE: u32 = 1;

/// The registered passes (LOG 6884975709), and `SW`, software events in their place for the
/// tests where there is no PMU (never registered, never a recipe's)
pub const PASSES: [(&str, &[Event]); 5] = [
    (
        "A1",
        &[
            (
                RAW,
                0x01b7,
                0x3fc4000001,
                "OFFCORE_RESPONSE.DEMAND_DATA_RD.L3_MISS_LOCAL_DRAM.ANY_SNOOP (0xb7/0x01, offcore_rsp=0x3fc4000001)",
            ),
            (
                RAW,
                0x10b0,
                0,
                "OFFCORE_REQUESTS.L3_MISS_DEMAND_DATA_RD (0xb0/0x10)",
            ),
        ],
    ),
    (
        "A2",
        &[(
            RAW,
            0x01bb,
            0x3fc4000002,
            "OFFCORE_RESPONSE.DEMAND_RFO.L3_MISS_LOCAL_DRAM.ANY_SNOOP (0xbb/0x01, offcore_rsp=0x3fc4000002; CML058)",
        )],
    ),
    (
        "B1",
        &[
            (RAW, 0x3824, 0, "L2_RQSTS.PF_MISS (0x24/0x38)"),
            (RAW, 0x08b0, 0, "OFFCORE_REQUESTS.ALL_DATA_RD (0xb0/0x08)"),
        ],
    ),
    (
        "B2",
        &[(RAW, 0x01b0, 0, "OFFCORE_REQUESTS.DEMAND_DATA_RD (0xb0/0x01)")],
    ),
    (
        "SW",
        &[
            (SOFTWARE, 1, 0, "task-clock (software, tests only)"),
            (SOFTWARE, 2, 0, "page-faults (software, tests only)"),
        ],
    ),
];

/// The phases a scope can be
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// `induce_cells`
    Induction = 0,
    /// `write_cells`
    Writer = 1,
}

/// This process's pass: its name and events, from `MIBZ2_PHASE_PASS` (unset or empty: none)
pub fn pass() -> Option<(&'static str, &'static [Event])> {
    static PASS: OnceLock<Option<(&'static str, &'static [Event])>> = OnceLock::new();
    *PASS.get_or_init(|| {
        let name = std::env::var("MIBZ2_PHASE_PASS").ok()?;
        if name.is_empty() {
            return None;
        }
        let p = PASSES.iter().find(|(n, _)| *n == name);
        assert!(
            p.is_some(),
            "MIBZ2_PHASE_PASS is one of A1, A2, B1, B2 (or SW)"
        );
        p.copied()
    })
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const SYS_PERF_EVENT_OPEN: Option<i64> = Some(298);
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const SYS_PERF_EVENT_OPEN: Option<i64> = Some(241);
#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
)))]
const SYS_PERF_EVENT_OPEN: Option<i64> = None;

extern "C" {
    fn syscall(num: std::ffi::c_long, ...) -> std::ffi::c_long;
    fn ioctl(fd: std::ffi::c_int, request: std::ffi::c_ulong, ...) -> std::ffi::c_int;
    fn read(fd: std::ffi::c_int, buf: *mut std::ffi::c_void, n: usize) -> isize;
    fn close(fd: std::ffi::c_int) -> std::ffi::c_int;
}

const IOC_ENABLE: u64 = 0x2400;
const IOC_DISABLE: u64 = 0x2401;
/// PERF_IOC_FLAG_GROUP: the request applies to the whole group
const FLAG_GROUP: u64 = 1;
/// read_format: total time enabled, total time running, the group's values in one read
const READ_FORMAT: u64 = 1 | 2 | 8;

fn errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

/// One event of a group opened: the leader where `group` is -1 (disabled until enabled),
/// else a member following it. User and kernel mode both, not the hypervisor, not inherited
fn open(ev: &Event, group: i32) -> Result<i32, String> {
    let Some(nr) = SYS_PERF_EVENT_OPEN else {
        return Err("perf_event_open is not available on this target".into());
    };
    // struct perf_event_attr, its first 64 bytes (PERF_ATTR_SIZE_VER0, which holds config1),
    // the rest zero: type and size, config, sample period and type (none), read_format,
    // the flags (bit 0 disabled, for the leader; bit 6 exclude_hv), wakeup and bp_type
    // (none), config1
    let mut attr = [0u64; 8];
    attr[0] = ev.0 as u64 | (64 << 32);
    attr[1] = ev.1;
    attr[4] = READ_FORMAT;
    attr[5] = if group < 0 { 1 } else { 0 } | 1 << 6;
    attr[7] = ev.2;
    // SAFETY: the attribute is a valid perf_event_attr of the size it declares; pid 0 (this
    // thread), any CPU, the group's leader or -1, close-on-exec (PERF_FLAG_FD_CLOEXEC)
    let fd = unsafe {
        syscall(
            nr as std::ffi::c_long,
            attr.as_ptr(),
            0i64,
            -1i64,
            group as i64,
            8u64,
        )
    };
    if fd < 0 {
        Err(format!("perf_event_open {}: errno {}", ev.3, errno()))
    } else {
        Ok(fd as i32)
    }
}

/// A group's reading: time enabled, time running, the values (ns, ns, counts)
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct Snap {
    enabled: u64,
    running: u64,
    v: [u64; MAX],
}

/// One group of the pass's events, the leader first
struct Group {
    fds: Vec<i32>,
}

impl Group {
    fn open(events: &[Event]) -> Result<Group, String> {
        let mut g = Group { fds: Vec::new() };
        for ev in events {
            let leader = g.fds.first().copied().unwrap_or(-1);
            g.fds.push(open(ev, leader)?);
        }
        Ok(g)
    }
    fn ctl(&self, req: u64) -> Result<(), String> {
        // SAFETY: an open perf event descriptor, the group flag its argument
        let r = unsafe { ioctl(self.fds[0], req as std::ffi::c_ulong, FLAG_GROUP) };
        if r != 0 {
            return Err(format!("ioctl {req:#x}: errno {}", errno()));
        }
        Ok(())
    }
    fn read(&self) -> Result<Snap, String> {
        let mut buf = [0u64; 3 + MAX];
        let want = 8 * (3 + self.fds.len());
        // SAFETY: the buffer holds `want` bytes: nr, time enabled, time running, a value an
        // event (PERF_FORMAT_GROUP with the two times)
        let n = unsafe { read(self.fds[0], buf.as_mut_ptr().cast(), want) };
        if n != want as isize || buf[0] as usize != self.fds.len() {
            return Err(format!("read: {n} bytes, errno {}", errno()));
        }
        let mut s = Snap {
            enabled: buf[1],
            running: buf[2],
            v: [0; MAX],
        };
        s.v[..self.fds.len()].copy_from_slice(&buf[3..3 + self.fds.len()]);
        Ok(s)
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        for &fd in self.fds.iter().rev() {
            // SAFETY: a descriptor this group opened and owns, closed once
            unsafe {
                close(fd);
            }
        }
    }
}

/// A sum of windows: values, times enabled and running, windows, and those multiplexed
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Win {
    pub v: [u64; MAX],
    pub enabled: u64,
    pub running: u64,
    pub windows: u64,
    pub mux: u64,
}

impl Win {
    fn add(&mut self, a: Snap, b: Snap) {
        let (e, r) = (
            b.enabled.wrapping_sub(a.enabled),
            b.running.wrapping_sub(a.running),
        );
        for i in 0..MAX {
            self.v[i] += b.v[i].wrapping_sub(a.v[i]);
        }
        self.enabled += e;
        self.running += r;
        self.windows += 1;
        self.mux += u64::from(r != e);
    }
    fn merge(&mut self, o: &Win) {
        for i in 0..MAX {
            self.v[i] += o.v[i];
        }
        self.enabled += o.enabled;
        self.running += o.running;
        self.windows += o.windows;
        self.mux += o.mux;
    }
    fn json(&self, n: usize) -> String {
        let v: Vec<String> = self.v[..n].iter().map(u64::to_string).collect();
        format!(
            "{{\"values\":[{}],\"enabled\":{},\"running\":{},\"windows\":{},\"mux\":{}}}",
            v.join(","),
            self.enabled,
            self.running,
            self.windows,
            self.mux
        )
    }
}

/// A process's (or a thread's) sums
#[derive(Clone, Default, Debug)]
pub struct Sums {
    pub threads: u64,
    pub blocks: u64,
    /// the whole group over each block's `decode_block`
    pub whole: Win,
    /// the whole group inside the induction's and the writer's windows
    pub whole_in: [Win; 2],
    /// the phase group in the induction's and the writer's windows
    pub phase: [Win; 2],
    /// the empty scopes: enabled and at once disabled
    pub empty: Win,
    /// the empty scopes' own wall time (ns): an enable, a disable and two reads each
    pub empty_ns: u64,
    pub errors: Vec<String>,
}

impl Sums {
    fn merge(&mut self, o: &Sums) {
        self.threads += o.threads;
        self.blocks += o.blocks;
        self.whole.merge(&o.whole);
        for p in 0..2 {
            self.whole_in[p].merge(&o.whole_in[p]);
            self.phase[p].merge(&o.phase[p]);
        }
        self.empty.merge(&o.empty);
        self.empty_ns += o.empty_ns;
        self.errors.extend(o.errors.iter().cloned());
    }
}

static TOTALS: Mutex<Option<Sums>> = Mutex::new(None);

/// A thread's groups, and where it is
struct Counting {
    whole: Group,
    phase: Group,
    empty: Group,
    /// the whole group at the block's start, and at the open phase scope's start
    at_block: Snap,
    at_scope: Option<(Phase, Snap, Snap)>,
    /// phase scopes this block, each matched by an empty scope after it
    scopes: u64,
    sums: Sums,
}

enum State {
    Idle,
    Counting(Box<Counting>),
    /// opening failed, or a control or read did: counted no further (the error is kept)
    Failed,
}

thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State::Idle) };
}

fn fail(e: String) {
    let mut t = TOTALS.lock().unwrap_or_else(|p| p.into_inner());
    t.get_or_insert_with(Sums::default).errors.push(e);
    STATE.with(|s| *s.borrow_mut() = State::Failed);
}

/// The start of a block's `decode_block` on a worker: the thread's groups opened the first
/// time, the whole group read and enabled
pub fn whole_begin() {
    let Some((_, events)) = pass() else { return };
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if matches!(*s, State::Idle) {
            let opened = (|| {
                Ok::<_, String>(Counting {
                    whole: Group::open(events)?,
                    phase: Group::open(events)?,
                    empty: Group::open(events)?,
                    at_block: Snap::default(),
                    at_scope: None,
                    scopes: 0,
                    sums: Sums {
                        threads: 1,
                        ..Default::default()
                    },
                })
            })();
            match opened {
                Ok(c) => *s = State::Counting(Box::new(c)),
                Err(e) => {
                    drop(s);
                    return fail(e);
                }
            }
        }
        let State::Counting(c) = &mut *s else { return };
        let r = c.whole.read().and_then(|a| {
            c.at_block = a;
            c.scopes = 0;
            c.whole.ctl(IOC_ENABLE)
        });
        if let Err(e) = r {
            drop(s);
            fail(e);
        }
    })
}

/// The end of the block's `decode_block`: the whole group disabled and read, then an empty
/// scope for each phase scope the block had, and the thread's sums so far added to the
/// process's
pub fn whole_end() {
    if pass().is_none() {
        return;
    }
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let State::Counting(c) = &mut *s else { return };
        let r = (|| {
            c.whole.ctl(IOC_DISABLE)?;
            let b = c.whole.read()?;
            c.sums.whole.add(c.at_block, b);
            c.sums.blocks += 1;
            let t0 = std::time::Instant::now();
            for _ in 0..c.scopes {
                let a = c.empty.read()?;
                c.empty.ctl(IOC_ENABLE)?;
                c.empty.ctl(IOC_DISABLE)?;
                let b = c.empty.read()?;
                c.sums.empty.add(a, b);
            }
            c.sums.empty_ns += t0.elapsed().as_nanos() as u64;
            let mut t = TOTALS.lock().unwrap_or_else(|p| p.into_inner());
            t.get_or_insert_with(Sums::default).merge(&c.sums);
            // (the thread is counted once)
            c.sums = Sums::default();
            Ok::<_, String>(())
        })();
        if let Err(e) = r {
            drop(s);
            fail(e);
        }
    })
}

/// A phase scope: the phase group enabled from here until it is dropped, the whole group
/// read at both edges. Nothing where the thread is not counting (no pass, the calling
/// thread's decode, or a failed thread)
pub struct Scope(bool);

impl Scope {
    #[inline]
    pub fn new(p: Phase) -> Scope {
        if pass().is_none() {
            return Scope(false);
        }
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            let State::Counting(c) = &mut *s else {
                return Scope(false);
            };
            let r = (|| {
                let ph = c.phase.read()?;
                let w = c.whole.read()?;
                c.at_scope = Some((p, w, ph));
                c.phase.ctl(IOC_ENABLE)
            })();
            match r {
                Ok(()) => Scope(true),
                Err(e) => {
                    drop(s);
                    fail(e);
                    Scope(false)
                }
            }
        })
    }
}

impl Drop for Scope {
    #[inline]
    fn drop(&mut self) {
        if !self.0 {
            return;
        }
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            let State::Counting(c) = &mut *s else { return };
            let r = (|| {
                c.phase.ctl(IOC_DISABLE)?;
                let w = c.whole.read()?;
                let ph = c.phase.read()?;
                let (p, w0, ph0) = c.at_scope.take().ok_or("a scope ended that never began")?;
                c.sums.whole_in[p as usize].add(w0, w);
                c.sums.phase[p as usize].add(ph0, ph);
                c.scopes += 1;
                Ok::<_, String>(())
            })();
            if let Err(e) = r {
                drop(s);
                fail(e.to_string());
            }
        })
    }
}

/// The process's sums so far
pub fn totals() -> Sums {
    TOTALS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
        .unwrap_or_default()
}

/// The process's sums as one JSON object: the pass and its events, the modes counted, the
/// whole, whole-in-window, phase and empty windows, the empty scopes' wall time, the errors
pub fn report() -> String {
    let Some((name, events)) = pass() else {
        return "null".into();
    };
    let s = totals();
    let n = events.len();
    let names: Vec<String> = events.iter().map(|e| format!("\"{}\"", e.3)).collect();
    let errors: Vec<String> = s
        .errors
        .iter()
        .map(|e| format!("\"{}\"", e.replace('"', "'")))
        .collect();
    format!(
        "{{\"pass\":\"{name}\",\"events\":[{}],\"modes\":\"user+kernel\",\"threads\":{},\"blocks\":{},\
         \"whole\":{},\"whole_in_induction\":{},\"whole_in_writer\":{},\"induction\":{},\"writer\":{},\
         \"empty\":{},\"empty_ns\":{},\"errors\":[{}]}}",
        names.join(","),
        s.threads,
        s.blocks,
        s.whole.json(n),
        s.whole_in[0].json(n),
        s.whole_in[1].json(n),
        s.phase[0].json(n),
        s.phase[1].json(n),
        s.empty.json(n),
        s.empty_ns,
        errors.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win_sums_deltas_and_flags_multiplexing() {
        let mut w = Win::default();
        let a = Snap {
            enabled: 10,
            running: 10,
            v: [5, 7],
        };
        let b = Snap {
            enabled: 30,
            running: 30,
            v: [9, 8],
        };
        w.add(a, b);
        assert_eq!(
            (w.v, w.enabled, w.running, w.windows, w.mux),
            ([4, 1], 20, 20, 1, 0)
        );
        let c = Snap {
            enabled: 50,
            running: 45,
            v: [10, 8],
        };
        w.add(b, c);
        assert_eq!((w.v, w.windows, w.mux), ([5, 1], 2, 1));
        assert!(w.json(1).starts_with("{\"values\":[5],"));
    }

    #[test]
    fn the_registered_passes() {
        let p = |n| PASSES.iter().find(|(m, _)| *m == n).unwrap().1;
        assert_eq!(
            p("A1").iter().map(|e| (e.0, e.1, e.2)).collect::<Vec<_>>(),
            [(4, 0x01b7, 0x3fc4000001), (4, 0x10b0, 0)]
        );
        assert_eq!(p("A2")[0].1..p("A2")[0].2, 0x01bb..0x3fc4000002);
        assert_eq!(
            p("B1").iter().map(|e| e.1).collect::<Vec<_>>(),
            [0x3824, 0x08b0]
        );
        assert_eq!(p("B2")[0].1, 0x01b0);
        // (at most two events a pass: whole and phase together fit 4 counters and the 2
        // offcore-response filters)
        for (_, evs) in PASSES {
            assert!(evs.len() <= MAX);
            assert!(evs.iter().filter(|e| e.2 != 0).count() <= 1);
        }
    }
}
