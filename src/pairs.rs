//! Workers in SMT sibling pairs (construction 2, a design probe): two workers pinned to
//! one physical core's CPUs share a walk token, so at most one of them walks at a time,
//! and the core's two T-vectors, so at most one block is built and waiting beside the
//! one being walked. A worker that cannot walk does the front half (MTF, build) of its
//! next block instead; it sleeps only when it can do neither.

use super::bwt::TVector;
use std::sync::{Condvar, Mutex, MutexGuard};

/// Where each worker runs: the CPUs it is pinned to (empty: unpinned) and its pair (with
/// its place in it), if it has one
pub(crate) struct Placement {
    pub cpus: Vec<usize>,
    pub pair: Option<(usize, usize)>,
}

/// The CPUs this process may use, grouped by physical core (Linux: `Cpus_allowed_list`,
/// each CPU's sysfs package and core id); None where the OS doesn't say
pub(crate) fn allowed_cores() -> Option<Vec<Vec<usize>>> {
    #[cfg(target_os = "linux")]
    {
        use std::collections::BTreeMap;
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let allowed = status
            .lines()
            .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))?
            .trim()
            .to_string();
        let mut cores: BTreeMap<(String, String), Vec<usize>> = BTreeMap::new();
        for part in allowed.split(',') {
            let (a, b) = part.split_once('-').unwrap_or((part, part));
            let (a, b): (usize, usize) = (a.parse().ok()?, b.parse().ok()?);
            for c in a..=b {
                let t = format!("/sys/devices/system/cpu/cpu{c}/topology");
                let Ok(core) = std::fs::read_to_string(format!("{t}/core_id")) else {
                    continue; // (an allowed CPU that is offline)
                };
                let pkg = std::fs::read_to_string(format!("{t}/physical_package_id")).ok()?;
                cores
                    .entry((pkg.trim().to_string(), core.trim().to_string()))
                    .or_default()
                    .push(c);
            }
        }
        (!cores.is_empty()).then(|| cores.into_values().collect())
    }
    #[cfg(not(target_os = "linux"))]
    None
}

/// The placement of `threads` workers on `cores` (each core's allowed CPUs): only where
/// there are more workers than cores do two share one. Then `threads - cores` cores with
/// two CPUs or more take a pair each (pinned to the core, not to a CPU, so the scheduler
/// can still move either within it), the other workers one core each, and any beyond
/// two a core run unpinned. None (today's decoder) where no core need be shared.
pub(crate) fn place(threads: usize, cores: &[Vec<usize>]) -> Option<Vec<Placement>> {
    if threads <= cores.len() {
        return None;
    }
    let (two, one): (Vec<&Vec<usize>>, Vec<&Vec<usize>>) = cores.iter().partition(|c| c.len() >= 2);
    let pairs = (threads - cores.len()).min(two.len());
    if pairs == 0 {
        return None;
    }
    let mut v = Vec::with_capacity(threads);
    for (p, core) in two[..pairs].iter().enumerate() {
        for m in 0..2 {
            v.push(Placement {
                cpus: (*core).clone(),
                pair: Some((p, m)),
            });
        }
    }
    for core in two[pairs..].iter().chain(one.iter()) {
        if v.len() < threads {
            v.push(Placement {
                cpus: (*core).clone(),
                pair: None,
            });
        }
    }
    while v.len() < threads {
        v.push(Placement {
            cpus: Vec::new(),
            pair: None,
        });
    }
    Some(v)
}

/// Pin the calling thread to `cpus`; false where it can't be (not Linux, or refused)
pub(crate) fn pin(cpus: &[usize]) -> bool {
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
        }
        if cpus.is_empty() || cpus.iter().any(|&c| c >= 1024) {
            return false;
        }
        let mut mask = [0u64; 16];
        for &c in cpus {
            mask[c / 64] |= 1 << (c % 64);
        }
        // SAFETY: a cpu_set_t of 1024 bits, read by the kernel; pid 0 is the calling thread
        unsafe { sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr()) == 0 }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = cpus;
        false
    }
}

/// What a pair's two workers share
pub(crate) struct Pair {
    st: Mutex<PairState>,
    cv: Condvar,
}

struct PairState {
    /// the member walking, if one is (or the member it was handed to)
    walking: Option<usize>,
    /// each member's built blocks not yet walked
    built: [usize; 2],
    /// a member asleep, and what for (a walk, a table)
    asleep: [Option<Want>; 2],
    /// the core's T-vectors not in use: two in all, one walked while the other is built
    /// or waits
    tables: Vec<TVector>,
}

#[derive(Clone, Copy)]
pub(crate) struct Want {
    pub walk: bool,
    pub table: bool,
}

impl Pair {
    pub fn new() -> Pair {
        Pair {
            st: Mutex::new(PairState {
                walking: None,
                built: [0; 2],
                asleep: [None; 2],
                tables: vec![TVector::new(), TVector::new()],
            }),
            cv: Condvar::new(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, PairState> {
        // (nothing panics while it is held: a poisoned lock is still consistent)
        self.st.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The token, for `me`'s next walk (it holds a built block): true if free, or handed
    /// to `me` by its sibling's `end_walk`
    pub fn try_walk(&self, me: usize) -> bool {
        let mut st = self.lock();
        let ok = match st.walking {
            None => {
                st.walking = Some(me);
                true
            }
            Some(w) => w == me,
        };
        if ok {
            st.built[me] -= 1;
        }
        ok
    }

    /// A walk's end: its table back to the core, and the token to the sibling if it is
    /// asleep waiting to walk, else free; the sibling woken either way (a table is free)
    pub fn end_walk(&self, me: usize, table: TVector) {
        let mut st = self.lock();
        debug_assert_eq!(st.walking, Some(me));
        st.tables.push(table);
        let s = me ^ 1;
        st.walking = match st.asleep[s] {
            Some(w) if w.walk && st.built[s] > 0 => Some(s),
            _ => None,
        };
        drop(st);
        self.cv.notify_all();
    }

    /// One of the core's tables for a build, if one is free
    pub fn take_table(&self) -> Option<TVector> {
        self.lock().tables.pop()
    }

    /// A build done: `me` holds one more built block
    pub fn built(&self, me: usize) {
        self.lock().built[me] += 1;
    }

    /// A table back unused, or from a block dropped (an error, a cancel), its built count
    /// with it
    pub fn give_back(&self, me: usize, table: TVector, was_built: bool) {
        let mut st = self.lock();
        st.tables.push(table);
        if was_built {
            st.built[me] -= 1;
        }
        drop(st);
        self.cv.notify_all();
    }

    /// Sleep until `me` can walk (the token free or handed to it) or build (a table free),
    /// as `want` says. Called only when its sibling is walking or about to (`me` holds
    /// a built block with the token taken, or an unbuilt one with both tables taken), so
    /// the sibling's `end_walk` wakes it.
    pub fn sleep(&self, me: usize, want: Want) {
        let mut st = self.lock();
        loop {
            let walk = want.walk && st.walking.map_or(true, |w| w == me);
            let table = want.table && !st.tables.is_empty();
            if walk || table {
                st.asleep[me] = None;
                return;
            }
            st.asleep[me] = Some(want);
            st = self.cv.wait(st).unwrap_or_else(|e| e.into_inner());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement() {
        let ci = vec![vec![0, 1], vec![2, 3]];
        // 4 workers on CI's 2 cores x 2 siblings: two pairs, each pinned to its core
        let p = place(4, &ci).unwrap();
        assert_eq!(p.len(), 4);
        assert_eq!(p[0].cpus, vec![0, 1]);
        assert_eq!(p[1].pair, Some((0, 1)));
        assert_eq!(p[3].cpus, vec![2, 3]);
        assert_eq!(p[3].pair, Some((1, 1)));
        // no core shared: today's decoder (1t, 2t, the bench's 2t on CPUs 0,2)
        assert!(place(2, &ci).is_none());
        assert!(place(1, &ci).is_none());
        assert!(place(2, &[vec![0], vec![2]]).is_none());
        // 3 workers: one pair, one worker alone on the other core
        let p = place(3, &ci).unwrap();
        assert_eq!(
            (p[0].pair, p[2].pair, p[2].cpus.clone()),
            (Some((0, 0)), None, vec![2, 3])
        );
        // a 4-core, 8-thread laptop at 6: two pairs and two single workers
        let lap: Vec<Vec<usize>> = (0..4).map(|c| vec![c, c + 4]).collect();
        let p = place(6, &lap).unwrap();
        assert_eq!(p.iter().filter(|x| x.pair.is_some()).count(), 4);
        assert_eq!(p[4].cpus, vec![2, 6]);
        // no SMT (one CPU a core): never paired
        assert!(place(8, &[vec![0], vec![1], vec![2], vec![3]]).is_none());
    }

    #[test]
    fn token_hands_over_to_a_sleeper() {
        let pair = std::sync::Arc::new(Pair::new());
        let t0 = pair.take_table().unwrap();
        let t1 = pair.take_table().unwrap();
        assert!(pair.take_table().is_none());
        pair.built(0);
        pair.built(1);
        assert!(pair.try_walk(0));
        assert!(!pair.try_walk(1));
        let p = pair.clone();
        let h = std::thread::spawn(move || {
            p.sleep(
                1,
                Want {
                    walk: true,
                    table: false,
                },
            );
            assert!(p.try_walk(1)); // handed over: 0 can't take it back first
            p.end_walk(1, t1);
        });
        while pair.lock().asleep[1].is_none() {
            std::thread::yield_now();
        }
        pair.end_walk(0, t0);
        assert_ne!(pair.lock().walking, Some(0));
        h.join().unwrap();
        assert_eq!(pair.lock().tables.len(), 2);
    }
}
