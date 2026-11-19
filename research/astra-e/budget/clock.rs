
#[derive(Default)]
pub struct Budget {
    pub start: u64,
    pub front: u64,
    pub inverse: u64,
    pub rle: u64,
    pub rows: usize,
    pub caps: usize,
    pub fused: bool,
}

#[inline(never)]
pub fn budget_clock() -> u64 {
    #[repr(C)]
    struct Timespec { seconds: i64, nanoseconds: i64 }
    extern "C" { fn clock_gettime(id: i32, time: *mut Timespec) -> i32; }
    let mut stamp = Timespec { seconds: 0, nanoseconds: 0 };
    // Linux x86_64 CLOCK_THREAD_CPUTIME_ID with a writable timespec.
    assert_eq!(unsafe { clock_gettime(3, &mut stamp) }, 0);
    stamp.seconds as u64 * 1_000_000_000 + stamp.nanoseconds as u64
}

