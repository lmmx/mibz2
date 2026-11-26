// Stage harness for the target Linux x86-64 ABI. Read only the current thread's
// CPU time. Both CPU clock calls surround the wall clock, outside its interval.
#[cfg(not(all(target_os = "linux", target_pointer_width = "64")))]
compile_error!("The thread CPU clock in this stage harness requires 64-bit Linux.");
fn cpu_ns() -> u64 {
    use std::ffi::c_long;
    #[repr(C)]
    struct Timespec { secs: c_long, nanos: c_long }
    unsafe extern "C" { fn clock_gettime(id: i32, out: *mut Timespec) -> i32; }
    let mut stamp = Timespec { secs: 0, nanos: 0 };
    // SAFETY: stamp is a live, correctly aligned timespec. The Linux caller
    // receives two longs from the current thread clock and checks its return.
    assert_eq!(unsafe { clock_gettime(3, &mut stamp) }, 0);
    stamp.secs as u64 * 1_000_000_000 + stamp.nanos as u64
}
