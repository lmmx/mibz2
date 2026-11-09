//! Shared by examples/sweep.rs and examples/run_fed_stage.rs (by `#[path]`: not an example).

/// The process's own hardware counters, by perf_event_open(2), as one group (scheduled onto
/// the PMU together or not at all): instructions, cycles, last-level cache misses and
/// back-end stall cycles (that one optional: not every PMU counts it), user space only,
/// unscaled, with the group's times enabled and running
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod imp {
    use std::fs::File;
    use std::io::Read;
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::sync::atomic::{AtomicIsize, Ordering};

    pub const EVENTS: usize = 4;
    // PERF_COUNT_HW_INSTRUCTIONS, _CPU_CYCLES, _CACHE_MISSES, _STALLED_CYCLES_BACKEND
    const CONFIG: [(u64, &str); EVENTS] = [
        (1, "instructions"),
        (0, "cycles"),
        (3, "LLC misses"),
        (8, "back-end stall cycles"),
    ];
    const SYS_PERF_EVENT_OPEN: usize = 298;
    const SYS_IOCTL: usize = 16;
    const IOC_ENABLE: usize = 0x2400;
    const IOC_DISABLE: usize = 0x2401;
    const IOC_RESET: usize = 0x2403;

    // The errno of the first counter that would not open
    static WHY: AtomicIsize = AtomicIsize::new(0);

    /// perf_event_attr as of its first version (PERF_ATTR_SIZE_VER0, 64 bytes)
    #[repr(C)]
    struct Attr {
        kind: u32,
        size: u32,
        config: u64,
        sample_period: u64,
        sample_type: u64,
        read_format: u64,
        flags: u64,
        wakeup_events: u32,
        bp_type: u32,
        config1: u64,
    }

    /// # Safety
    /// A raw system call: the arguments must be valid for call `n`
    unsafe fn syscall(n: usize, a: [usize; 5]) -> isize {
        let ret: isize;
        // SAFETY: the caller's; the syscall instruction clobbers rcx and r11 only
        unsafe {
            core::arch::asm!("syscall", inlateout("rax") n as isize => ret,
                in("rdi") a[0], in("rsi") a[1], in("rdx") a[2], in("r10") a[3], in("r8") a[4],
                lateout("rcx") _, lateout("r11") _, options(nostack));
        }
        ret
    }

    /// One event of the group led by `leader` (or the leader itself, disabled until the
    /// group is enabled); the leader reads the group's values with its times enabled and
    /// running, so every ratio comes from the same intervals
    fn open(config: u64, leader: Option<&File>) -> Result<File, isize> {
        let attr = Attr {
            kind: 0, // PERF_TYPE_HARDWARE
            size: 64,
            config,
            sample_period: 0,
            sample_type: 0,
            read_format: 1 | 2 | 8, // TOTAL_TIME_ENABLED | TOTAL_TIME_RUNNING | GROUP
            // disabled (the leader only), exclude_kernel, exclude_hv
            flags: u64::from(leader.is_none()) | (1 << 5) | (1 << 6),
            wakeup_events: 0,
            bp_type: 0,
            config1: 0,
        };
        let group = leader.map_or(usize::MAX, |f| f.as_raw_fd() as usize);
        let args = [&attr as *const Attr as usize, 0, usize::MAX, group, 0];
        // SAFETY: attr outlives the call; pid 0 (this process), cpu -1 (any)
        let fd = unsafe { syscall(SYS_PERF_EVENT_OPEN, args) };
        if fd < 0 {
            return Err(-fd);
        }
        // SAFETY: a new file descriptor this process owns
        Ok(unsafe { File::from_raw_fd(fd as i32) })
    }

    fn ioctl(f: &File, req: usize) {
        // SAFETY: a perf event's descriptor; the argument is PERF_IOC_FLAG_GROUP
        unsafe { syscall(SYS_IOCTL, [f.as_raw_fd() as usize, req, 1, 0, 0]) };
    }

    pub struct Counters {
        // the leader (instructions) first, then the events that joined its group
        files: Vec<(usize, File)>,
    }

    impl Counters {
        /// The largest of instructions + cycles + LLC misses + back-end stalls, the same
        /// without the stalls, and instructions + cycles alone that opens and is scheduled
        /// onto the PMU in a short trial (a group can open and still never be scheduled,
        /// its events together needing more counters than the PMU has); else none
        pub fn open() -> Option<Counters> {
            for events in [&[0usize, 1, 2, 3][..], &[0, 1, 2], &[0, 1]] {
                let Some(mut c) = Self::open_group(events) else {
                    continue;
                };
                c.start();
                let mut x = 0u64;
                for i in 0..1_000_000u64 {
                    x = std::hint::black_box(x.wrapping_mul(31).wrapping_add(i));
                }
                let (_, enabled, running) = c.stop();
                if enabled > 0 && running > 0 {
                    return Some(c);
                }
                WHY.store(-1, Ordering::Relaxed);
            }
            None
        }
        fn open_group(events: &[usize]) -> Option<Counters> {
            let leader = match open(CONFIG[events[0]].0, None) {
                Ok(f) => f,
                Err(e) => {
                    WHY.store(e, Ordering::Relaxed);
                    return None;
                }
            };
            let mut files = Vec::new();
            for &i in &events[1..] {
                match open(CONFIG[i].0, Some(&leader)) {
                    Ok(f) => files.push((i, f)),
                    Err(e) => {
                        WHY.store(e, Ordering::Relaxed);
                        return None;
                    }
                }
            }
            files.insert(0, (events[0], leader));
            Some(Counters { files })
        }
        pub fn has(&self, i: usize) -> bool {
            self.files.iter().any(|(j, _)| *j == i)
        }
        pub fn names(&self) -> Vec<&'static str> {
            self.files.iter().map(|(i, _)| CONFIG[*i].1).collect()
        }
        pub fn start(&mut self) {
            ioctl(&self.files[0].1, IOC_RESET);
            ioctl(&self.files[0].1, IOC_ENABLE);
        }
        /// The group's counts since `start`, unscaled, and its times enabled and running
        /// (running below enabled: the group was multiplexed out for part of it)
        pub fn stop(&mut self) -> ([u64; EVENTS], u64, u64) {
            ioctl(&self.files[0].1, IOC_DISABLE);
            let mut v = [0u64; EVENTS];
            let mut buf = vec![0u8; 8 * (3 + self.files.len())];
            if self.files[0].1.read_exact(&mut buf).is_err() {
                return (v, 0, 0);
            }
            let w = |k: usize| u64::from_ne_bytes(buf[8 * k..8 * k + 8].try_into().unwrap());
            for (j, (i, _)) in self.files.iter().enumerate() {
                v[*i] = w(3 + j);
            }
            (v, w(1), w(2))
        }
    }

    pub fn why() -> String {
        match WHY.load(Ordering::Relaxed) {
            1 | 13 => "not permitted: perf_event_paranoid".into(),
            2 | 19 | 95 => "no PMU exposed to this machine".into(),
            -1 => "the counters opened but were never scheduled onto the PMU".into(),
            e => format!("perf_event_open: errno {e}"),
        }
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
mod imp {
    pub const EVENTS: usize = 4;
    pub struct Counters;
    impl Counters {
        pub fn open() -> Option<Counters> {
            None
        }
        pub fn names(&self) -> Vec<&'static str> {
            Vec::new()
        }
        pub fn has(&self, _: usize) -> bool {
            false
        }
        pub fn start(&mut self) {}
        pub fn stop(&mut self) -> ([u64; EVENTS], u64, u64) {
            ([0; EVENTS], 0, 0)
        }
    }
    pub fn why() -> String {
        "counters are read on Linux x86_64 only".into()
    }
}

pub use imp::*;
