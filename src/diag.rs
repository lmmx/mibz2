//! The service core's starvation diagnostic (feature `svc-diag`: research, opt-in, off by
//! default, compiled out of every timed build). An event log, not a timeline: it records
//! the state-changing transitions of `StreamDecoder::service`'s hub, its scanner's window
//! and its reader, so that bench/svc_diag.py can say what an idle team was actually waiting
//! on through the whole of its wait, rather than by the state it started in
//! (`TeamIdleStart`'s label). Asked for by the Researcher's REVIEW 6736071231
//! (state/reports/service-starvation-6735725456/report.md at d6f52bdd).
//!
//! **What an event holds:** a process-wide sequence number, taken inside the lock whose
//! state the event reports, so the merged order is consistent with each lock's own order;
//! a monotonic timestamp (indicative only: nothing here is timing evidence); the decoder's
//! id; its kind, its block's place and two arguments; and a snapshot of the state it was
//! taken under:
//! - **the hub's** (`HubSnap`), under the hub's lock: J jobs, P prepared, Q walked, A
//!   preparing, C finishing, U walking, the free front and back sets, the live blocks and
//!   whether the jobs are closed. The ownership identities
//!   `F = free_front + A + P + U` and `2T = free_back + U + Q + C` hold at every one.
//! - **the window's** (`WinSnap`), under the shared state's lock: issued, taken, the ready
//!   results' count and least and greatest places, the end and the window W.
//!
//! **Per-thread buffers:** each thread pushes to a buffer of its own (no lock shared while
//! decoding beyond the one the event is already under, and the sequence counter); a
//! thread's buffer goes to a shared list when the thread ends, and `take` collects a
//! decoder's events (the calling thread's too) once the decoder is dropped.
//!
//! Nothing here changes what the decoder does: the hub's counters A, C and U are kept only
//! for the log, and no event takes a lock of its own on the decoder's path.

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// No block: the place of an event that has none
pub const NONE: u64 = u64::MAX;

/// An event's kind. Those marked (hub) carry the hub's snapshot, taken under its lock;
/// (window) the shared state's; the rest none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// (hub, window) the decoder built: `place` its teams, `arg` its service workers; the
    /// snapshot's free fronts are F = T + L + K and free backs 2T, the window's slots W
    Config,
    /// the scanner: waiting for compressed input, a chunk got (`arg` its bytes), the end
    InputWait,
    InputGot,
    InputEnd,
    /// (window) the scanner: waiting for a window credit (issued − taken = W) at its first
    /// check, at each recheck after a wake-up still without one, and once it has one
    /// (`arg` 1 if it waited)
    SlotWait,
    SlotRecheck,
    SlotGot,
    /// (window) the scanner: a block read and given its place (`n` its symbols)
    Issued,
    /// the scanner: the block sent to the feeder's channel
    Enqueued,
    /// (window) the scanner's end: `arg` 1 if it ended in an error (its place then ready)
    ScanEnd,
    /// the feeder: a job received from the scanner's channel
    FeedRecv,
    /// (hub) the feeder: the job in the hub's queue; the jobs closed (the feeder's end)
    JobIn,
    Closed,
    /// (hub) a service worker's decision: a preparation (`arg` 0: P was empty, rule 1; 1:
    /// below the threshold, rule 3; `n` the block's symbols) or a finish (rule 2)
    SvcPrepare,
    SvcFinish,
    /// (hub) a service worker about to wait, at its first wait and each recheck after a
    /// wake-up (`arg`: why it can't prepare, `SvcWhy`), and its end
    SvcWait,
    SvcRecheck,
    SvcExit,
    /// (hub) a preparation done and queued (`arg` its rows, `n` the front's `ll` capacity)
    Prepared,
    /// (hub) a block leaving the decode from a service worker's preparing (an error, a
    /// panic or cancellation), its finishing (published or cancelled), or a team's walk
    /// (an error, a panic, or cancellation): its buffers back. `RetireUnowned` is a retire
    /// from no task: an accounting defect.
    PrepRetire,
    FinishRetire,
    WalkRetire,
    RetireUnowned,
    /// (hub) a team's first thread about to wait for a prepared block and a back set, at
    /// its first wait (`arg`: `TeamIdleStart`'s label, 1 if P was nonempty) and each
    /// recheck after a wake-up; a block taken; its end
    TeamWait,
    TeamRecheck,
    TeamGot,
    TeamExit,
    /// (hub) a team's block walked and queued for finishing (its front set back)
    Walked,
    /// (window) a block's result in `ready` (`arg` 1: an error; 2: dropped, cancelled)
    Publish,
    /// (window) the reader about to wait for its next block in order (the earliest missing
    /// place, `taken`), each recheck, a block taken, and its end (`arg` 1: an error or
    /// cancellation)
    ReadWait,
    ReadRecheck,
    Taken,
    ReadEnd,
    /// (window) the decode cancelled
    Cancel,
}

impl Kind {
    pub fn name(self) -> String {
        format!("{self:?}")
    }
}

/// Why a service worker waits (`SvcWait`'s `arg`): no block to finish, and no job (0),
/// no front set for a job (1: never, while the ownership identities hold), or the prepared
/// queue at its threshold (2)
pub mod svc_why {
    pub const NO_JOB: u64 = 0;
    pub const NO_FRONT: u64 = 1;
    pub const THRESHOLD: u64 = 2;
}

/// The hub's state at an event, read under its lock
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HubSnap {
    /// jobs queued (J), prepared blocks queued (P), walked blocks queued (Q)
    pub j: u32,
    pub p: u32,
    pub q: u32,
    /// preparations under way (A), finishes under way (C), teams walking (U)
    pub a: u32,
    pub c: u32,
    pub u: u32,
    pub free_front: u32,
    pub free_back: u32,
    pub live: u32,
    pub closed: bool,
}

/// The window's state at an event, read under the shared state's lock
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WinSnap {
    pub issued: u64,
    pub taken: u64,
    /// results ready and not taken, and their least and greatest places (`NONE` if none)
    pub ready: u32,
    pub ready_min: u64,
    pub ready_max: u64,
    /// the number of places once the scanner has ended (`NONE` before)
    pub end: u64,
    pub slots: u64,
    pub cancelled: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub seq: u64,
    pub ns: u64,
    pub dec: u64,
    pub kind: Kind,
    pub place: u64,
    pub arg: u64,
    pub n: u64,
    pub hub: Option<HubSnap>,
    pub win: Option<WinSnap>,
}

/// One thread's events, in its order
#[derive(Clone, Debug)]
pub struct ThreadLog {
    pub name: String,
    pub events: Vec<Event>,
}

/// The hub's own counters for the log (`HubState::d`): its decoder's id, and the tasks
/// under way that the hub's queues don't show
#[derive(Debug, Default)]
pub struct HubCounts {
    pub dec: u64,
    pub preparing: u32,
    pub finishing: u32,
    pub walking: u32,
}

/// The task this thread has under way, for a retire to be accounted to (`take_task`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Task {
    Idle,
    Prepare(u64),
    Finish(u64),
    Walk(u64),
}

static NEXT_DEC: AtomicU64 = AtomicU64::new(1);
static SEQ: AtomicU64 = AtomicU64::new(0);
static EPOCH: OnceLock<Instant> = OnceLock::new();
static DONE: Mutex<Vec<ThreadLog>> = Mutex::new(Vec::new());

/// A new decoder's id (never 0: 0 is a decoder that logs nothing)
pub fn new_decoder() -> u64 {
    NEXT_DEC.fetch_add(1, Ordering::Relaxed)
}

struct Buf(ThreadLog);

impl Drop for Buf {
    fn drop(&mut self) {
        let log = std::mem::replace(
            &mut self.0,
            ThreadLog {
                name: String::new(),
                events: Vec::new(),
            },
        );
        if !log.events.is_empty() {
            DONE.lock().unwrap_or_else(|e| e.into_inner()).push(log);
        }
    }
}

thread_local! {
    static BUF: RefCell<Option<Buf>> = const { RefCell::new(None) };
    static TASK: Cell<Task> = const { Cell::new(Task::Idle) };
}

/// An event on this thread's buffer (nothing for decoder 0). Called inside the lock whose
/// state the snapshot holds, so that the sequence number orders it with that lock's other
/// events.
#[inline]
pub fn record(
    dec: u64,
    kind: Kind,
    place: u64,
    arg: u64,
    n: u64,
    hub: Option<HubSnap>,
    win: Option<WinSnap>,
) {
    if dec == 0 {
        return;
    }
    let seq = SEQ.fetch_add(1, Ordering::SeqCst);
    let ns = EPOCH.get_or_init(Instant::now).elapsed().as_nanos() as u64;
    let ev = Event {
        seq,
        ns,
        dec,
        kind,
        place,
        arg,
        n,
        hub,
        win,
    };
    // (a thread ending while its buffer is dropped records nothing more)
    let _ = BUF.try_with(|b| {
        b.borrow_mut()
            .get_or_insert_with(|| {
                Buf(ThreadLog {
                    name: std::thread::current().name().unwrap_or("?").to_string(),
                    events: Vec::with_capacity(1024),
                })
            })
            .0
            .events
            .push(ev)
    });
}

/// This thread's task from here (a service worker's preparing or finishing, a team's walk)
pub fn set_task(t: Task) {
    let _ = TASK.try_with(|c| c.set(t));
}

/// This thread's task, which ends here (`Idle` from here)
pub fn take_task() -> Task {
    TASK.try_with(|c| c.replace(Task::Idle))
        .unwrap_or(Task::Idle)
}

/// Decoder `dec`'s events, by thread: call once the decoder is dropped (its threads
/// joined). The calling thread's buffer (the reader's, usually) is collected too.
pub fn take(dec: u64) -> Vec<ThreadLog> {
    let _ = BUF.try_with(|b| drop(b.borrow_mut().take()));
    let mut done = DONE.lock().unwrap_or_else(|e| e.into_inner());
    let mut out = Vec::new();
    for log in done.iter_mut() {
        let (mine, rest): (Vec<Event>, Vec<Event>) =
            log.events.drain(..).partition(|e| e.dec == dec);
        log.events = rest;
        if !mine.is_empty() {
            out.push(ThreadLog {
                name: log.name.clone(),
                events: mine,
            });
        }
    }
    done.retain(|l| !l.events.is_empty());
    out
}

/// Every thread's events in one order, by sequence number: (thread index, event)
pub fn merged(logs: &[ThreadLog]) -> Vec<(usize, Event)> {
    let mut all: Vec<(usize, Event)> = logs
        .iter()
        .enumerate()
        .flat_map(|(i, l)| l.events.iter().map(move |e| (i, *e)))
        .collect();
    all.sort_by_key(|(_, e)| e.seq);
    all
}

fn opt(v: u64) -> String {
    if v == NONE {
        "-".into()
    } else {
        v.to_string()
    }
}

/// The columns of `tsv`
pub const COLUMNS: &str = "seq\tns\tthread\tkind\tplace\targ\tn\t\
j\tp\tq\ta\tc\tu\tfree_front\tfree_back\tlive\tclosed\t\
issued\ttaken\tready\tready_min\tready_max\tend\tslots\tcancelled";

/// The merged log as tab-separated values, a header first (`COLUMNS`); an absent
/// snapshot's fields and a `NONE` are `-`. bench/svc_diag.py reads it.
pub fn tsv(logs: &[ThreadLog]) -> String {
    let mut s = String::from(COLUMNS);
    s.push('\n');
    for (t, e) in merged(logs) {
        let mut row = vec![
            e.seq.to_string(),
            e.ns.to_string(),
            logs[t].name.clone(),
            e.kind.name(),
            opt(e.place),
            e.arg.to_string(),
            e.n.to_string(),
        ];
        match e.hub {
            Some(h) => row.extend(
                [
                    h.j,
                    h.p,
                    h.q,
                    h.a,
                    h.c,
                    h.u,
                    h.free_front,
                    h.free_back,
                    h.live,
                ]
                .iter()
                .map(|v| v.to_string())
                .chain([u8::from(h.closed).to_string()]),
            ),
            None => row.extend(std::iter::repeat("-".to_string()).take(10)),
        }
        match e.win {
            Some(w) => row.extend([
                w.issued.to_string(),
                w.taken.to_string(),
                w.ready.to_string(),
                opt(w.ready_min),
                opt(w.ready_max),
                opt(w.end),
                w.slots.to_string(),
                u8::from(w.cancelled).to_string(),
            ]),
            None => row.extend(std::iter::repeat("-".to_string()).take(8)),
        }
        s.push_str(&row.join("\t"));
        s.push('\n');
    }
    s
}

/// What `check` found in a complete log
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub teams: u64,
    pub services: u64,
    pub lookahead: u64,
    pub slots: u64,
    pub blocks: u64,
    pub events: usize,
    pub hub_events: usize,
}

/// A clean decode's log checked complete and consistent, or the first thing wrong:
/// - one `Config`; sequence numbers unique, and increasing in each thread's own order;
/// - the ownership identities `F = free_front + A + P + U` and `2T = free_back + U + Q + C`
///   at every hub snapshot, and no `RetireUnowned`;
/// - every place from 0 to the end through every stage exactly once, in the pipeline's
///   causal order (issued, received by the feeder, queued, its preparation started and
///   done, taken by a team, walked, its finish started, published, taken by the reader),
///   with its two events that nothing orders against the rest after their own
///   predecessor: the scanner's `Enqueued` (after the send, so the feeder can be first)
///   after `Issued`, and the service worker's `FinishRetire` (after the publish, so the
///   reader can be first) after `Publish`;
/// - the scanner, the feeder, each service worker, each team and the reader ended once,
///   the hub empty and the window drained at the end.
pub fn check(logs: &[ThreadLog]) -> Result<Summary, String> {
    use Kind::*;
    let all = merged(logs);
    for w in all.windows(2) {
        if w[0].1.seq == w[1].1.seq {
            return Err(format!("sequence {} twice", w[0].1.seq));
        }
    }
    for l in logs {
        if l.events.windows(2).any(|w| w[0].seq >= w[1].seq) {
            return Err(format!("{}: its events out of order", l.name));
        }
    }
    let configs: Vec<&Event> = all
        .iter()
        .map(|(_, e)| e)
        .filter(|e| e.kind == Config)
        .collect();
    let [cfg] = configs[..] else {
        return Err(format!("{} Config events", configs.len()));
    };
    let (teams, services) = (cfg.place, cfg.arg);
    let h0 = cfg.hub.ok_or("Config without the hub's snapshot")?;
    let w0 = cfg.win.ok_or("Config without the window's snapshot")?;
    let fronts = u64::from(h0.free_front);
    if u64::from(h0.free_back) != 2 * teams || fronts < teams + services {
        return Err(format!("Config: {h0:?}"));
    }
    let mut s = Summary {
        teams,
        services,
        lookahead: fronts - teams - services,
        slots: w0.slots,
        events: all.len(),
        ..Summary::default()
    };
    let order = [
        Issued, FeedRecv, JobIn, SvcPrepare, Prepared, TeamGot, Walked, SvcFinish, Publish, Taken,
    ];
    // (each after its predecessor only)
    let loose = [(Enqueued, Issued), (FinishRetire, Publish)];
    let mut seen: std::collections::BTreeMap<u64, Vec<Kind>> = Default::default();
    let mut last = None;
    for (_, e) in &all {
        if let Some(h) = e.hub {
            s.hub_events += 1;
            let (a, p, u, q, c) = (h.a, h.p, h.u, h.q, h.c);
            if u64::from(h.free_front + a + p + u) != fronts {
                return Err(format!("{:?} at {}: fronts {h:?}", e.kind, e.seq));
            }
            if u64::from(h.free_back + u + q + c) != 2 * teams {
                return Err(format!("{:?} at {}: backs {h:?}", e.kind, e.seq));
            }
            if u64::from(a) > services || u64::from(c) > services || u64::from(u) > teams {
                return Err(format!("{:?} at {}: tasks {h:?}", e.kind, e.seq));
            }
            last = Some(h);
        }
        if e.kind == RetireUnowned {
            return Err(format!("a retire from no task at {}", e.seq));
        }
        if order.contains(&e.kind) || loose.iter().any(|&(k, _)| k == e.kind) {
            seen.entry(e.place).or_default().push(e.kind);
        }
    }
    let end = all
        .iter()
        .rev()
        .find_map(|(_, e)| e.win.map(|w| w.end))
        .ok_or("no window snapshot")?;
    if end == NONE {
        return Err("the scanner never ended".into());
    }
    s.blocks = end;
    if seen.keys().copied().ne(0..end) {
        return Err(format!(
            "places seen {:?}, end {end}",
            seen.keys().collect::<Vec<_>>()
        ));
    }
    for (place, kinds) in &seen {
        let chain: Vec<Kind> = kinds
            .iter()
            .copied()
            .filter(|k| order.contains(k))
            .collect();
        let at = |k: Kind| kinds.iter().position(|&x| x == k);
        let once = |k: Kind| kinds.iter().filter(|&&x| x == k).count() == 1;
        if chain[..] != order[..]
            || loose
                .iter()
                .any(|&(k, after)| !once(k) || at(k) < at(after))
        {
            return Err(format!("block {place}: {kinds:?}"));
        }
    }
    let count = |k: Kind| all.iter().filter(|(_, e)| e.kind == k).count() as u64;
    for (k, want) in [
        (ScanEnd, 1),
        (Closed, 1),
        (SvcExit, services),
        (TeamExit, teams),
        (ReadEnd, 1),
    ] {
        if count(k) != want {
            return Err(format!("{} {k:?} events, not {want}", count(k)));
        }
    }
    let h = last.ok_or("no hub snapshot")?;
    if (h.j, h.p, h.q, h.a, h.c, h.u, h.live) != (0, 0, 0, 0, 0, 0, 0) || !h.closed {
        return Err(format!("the hub at its end: {h:?}"));
    }
    let w = all
        .iter()
        .rev()
        .find_map(|(_, e)| e.win)
        .ok_or("no window snapshot")?;
    if w.issued != end || w.taken != end || w.ready != 0 {
        return Err(format!("the window at its end: {w:?}"));
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hub(dec: u64, kind: Kind, place: u64) {
        record(dec, kind, place, 0, 0, Some(HubSnap::default()), None);
    }

    #[test]
    fn decoder_zero_records_nothing() {
        record(0, Kind::JobIn, 1, 0, 0, None, None);
        assert!(take(0).is_empty());
    }

    #[test]
    fn per_thread_buffers_merge_in_sequence_order_and_none_is_lost() {
        let dec = new_decoder();
        let other = new_decoder();
        // a lock's order is the sequence's: each thread records under the one lock
        // (spawned and joined, not scoped: a scoped thread's end is signalled before its
        // thread-locals are dropped, a joined thread's after)
        let lock = std::sync::Arc::new(Mutex::new(Vec::new()));
        let threads: Vec<_> = (0..4u64)
            .map(|t| {
                let lock = lock.clone();
                std::thread::Builder::new()
                    .name(format!("diag-test-{t}"))
                    .spawn(move || {
                        for i in 0..1000u64 {
                            let mut order = lock.lock().unwrap();
                            hub(dec, Kind::JobIn, t * 1000 + i);
                            order.push(t * 1000 + i);
                            drop(order);
                            // another decoder's events, interleaved, not to be taken
                            hub(other, Kind::Prepared, i);
                        }
                    })
                    .unwrap()
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        // and the calling thread's own, still in its buffer
        hub(dec, Kind::Closed, NONE);
        let logs = take(dec);
        assert_eq!(logs.len(), 5);
        assert!(logs.iter().all(|l| l.events.iter().all(|e| e.dec == dec)));
        for l in &logs {
            assert!(
                l.events.windows(2).all(|w| w[0].seq < w[1].seq),
                "{}",
                l.name
            );
        }
        let m = merged(&logs);
        assert_eq!(m.len(), 4001);
        let places: Vec<u64> = m
            .iter()
            .filter(|(_, e)| e.kind == Kind::JobIn)
            .map(|(_, e)| e.place)
            .collect();
        assert_eq!(places, *lock.lock().unwrap(), "the lock's order");
        assert_eq!(m.last().unwrap().1.kind, Kind::Closed);
        // the other decoder's are still there, every one, and then gone
        let rest = take(other);
        assert_eq!(rest.iter().map(|l| l.events.len()).sum::<usize>(), 4000);
        assert!(take(other).is_empty() && take(dec).is_empty());
    }

    #[test]
    fn tasks_are_per_thread_and_taken_once() {
        set_task(Task::Finish(3));
        std::thread::spawn(|| assert_eq!(take_task(), Task::Idle))
            .join()
            .unwrap();
        assert_eq!(take_task(), Task::Finish(3));
        assert_eq!(take_task(), Task::Idle);
    }

    #[test]
    fn tsv_has_a_row_an_event_and_a_field_a_column() {
        let dec = new_decoder();
        record(dec, Kind::InputWait, NONE, 0, 0, None, None);
        hub(dec, Kind::JobIn, 0);
        record(dec, Kind::Taken, 0, 0, 0, None, Some(WinSnap::default()));
        let t = tsv(&take(dec));
        let rows: Vec<&str> = t.lines().collect();
        assert_eq!(rows.len(), 4);
        let width = COLUMNS.split('\t').count();
        assert!(rows.iter().all(|r| r.split('\t').count() == width));
        assert!(rows[1].contains("\tInputWait\t-\t"));
    }

    /// A hand-built event: kind, place, arg and the snapshots
    type Row = (Kind, u64, u64, Option<HubSnap>, Option<WinSnap>);

    #[test]
    fn check_rejects_a_dropped_or_duplicated_event() {
        // a hand-built clean log of one block, then each stage dropped or doubled
        use Kind::*;
        let full = HubSnap {
            free_front: 5,
            free_back: 2,
            ..HubSnap::default()
        };
        let win = |issued, taken, end| WinSnap {
            issued,
            taken,
            end,
            slots: 4,
            ready_min: NONE,
            ready_max: NONE,
            ..WinSnap::default()
        };
        let mut evs = vec![(Config, 1u64, 2u64, Some(full), Some(win(0, 0, NONE)))];
        let h = |j, p, q, a, c, u, ff, fb, closed| HubSnap {
            j,
            p,
            q,
            a,
            c,
            u,
            free_front: ff,
            free_back: fb,
            live: a + p + u + q + c,
            closed,
        };
        for (k, hs, ws) in [
            (Issued, None, Some(win(1, 0, NONE))),
            (Enqueued, None, None),
            (FeedRecv, None, None),
            (JobIn, Some(h(1, 0, 0, 0, 0, 0, 5, 2, false)), None),
            (SvcPrepare, Some(h(0, 0, 0, 1, 0, 0, 4, 2, false)), None),
            (Prepared, Some(h(0, 1, 0, 0, 0, 0, 4, 2, false)), None),
            (TeamGot, Some(h(0, 0, 0, 0, 0, 1, 4, 1, false)), None),
            (Walked, Some(h(0, 0, 1, 0, 0, 0, 5, 1, false)), None),
            (SvcFinish, Some(h(0, 0, 0, 0, 1, 0, 5, 1, false)), None),
            (Publish, None, Some(win(1, 0, NONE))),
            (FinishRetire, Some(h(0, 0, 0, 0, 0, 0, 5, 2, false)), None),
            (ScanEnd, None, Some(win(1, 0, 1))),
            (Closed, Some(h(0, 0, 0, 0, 0, 0, 5, 2, true)), None),
            (Taken, None, Some(win(1, 1, 1))),
            (SvcExit, Some(h(0, 0, 0, 0, 0, 0, 5, 2, true)), None),
            (SvcExit, Some(h(0, 0, 0, 0, 0, 0, 5, 2, true)), None),
            (TeamExit, Some(h(0, 0, 0, 0, 0, 0, 5, 2, true)), None),
            (ReadEnd, None, Some(win(1, 1, 1))),
        ] {
            let place = if matches!(k, ScanEnd | Closed | SvcExit | TeamExit | ReadEnd) {
                NONE
            } else {
                0
            };
            evs.push((k, place, 0, hs, ws));
        }
        let log = |evs: &[Row]| {
            vec![ThreadLog {
                name: "t".into(),
                events: evs
                    .iter()
                    .enumerate()
                    .map(|(i, &(kind, place, arg, hub, win))| Event {
                        seq: i as u64,
                        ns: i as u64,
                        dec: 1,
                        kind,
                        place,
                        arg,
                        n: 0,
                        hub,
                        win,
                    })
                    .collect(),
            }]
        };
        let s = check(&log(&evs)).unwrap();
        assert_eq!((s.teams, s.services, s.lookahead, s.blocks), (1, 2, 2, 1));
        for i in 1..evs.len() {
            let mut dropped = evs.clone();
            dropped.remove(i);
            assert!(check(&log(&dropped)).is_err(), "{:?} dropped", evs[i].0);
            let mut twice = evs.clone();
            twice.insert(i, evs[i]);
            assert!(check(&log(&twice)).is_err(), "{:?} twice", evs[i].0);
        }
        // an ownership identity broken (a front lost)
        let mut lost = evs.clone();
        lost[5].3.as_mut().unwrap().free_front -= 1;
        assert!(check(&log(&lost)).unwrap_err().contains("fronts"));
        // two stages swapped
        let mut swapped = evs.clone();
        swapped.swap(6, 7);
        assert!(check(&log(&swapped)).is_err());
    }
}
