//! `Kernels`: safe code gets only the features this CPU has (#340). That safe code cannot
//! write the fields is the `compile_fail` doc test on `Kernels` itself.

use mibz2::Kernels;

/// `detect` claims exactly what the CPU has, as its kernels need it
#[test]
fn detect_is_the_cpu() {
    let k = Kernels::detect();
    #[cfg(target_arch = "x86_64")]
    {
        assert_eq!(
            k.avx2(),
            std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("bmi1")
        );
        assert_eq!(
            k.clmul(),
            std::is_x86_feature_detected!("pclmulqdq")
                && std::is_x86_feature_detected!("sse4.1")
                && std::is_x86_feature_detected!("ssse3")
        );
    }
    #[cfg(not(target_arch = "x86_64"))]
    assert_eq!(k, Kernels::scalar());
}

/// `scalar` claims nothing, and `restrict` only ever takes features away
#[test]
fn restrict_only_removes() {
    let s = Kernels::scalar();
    assert!(!s.avx2() && !s.clmul());
    for k in [Kernels::scalar(), Kernels::detect()] {
        for avx2 in [false, true] {
            for clmul in [false, true] {
                let r = k.restrict(avx2, clmul);
                assert_eq!(r.avx2(), k.avx2() && avx2, "{k:?} {avx2} {clmul}");
                assert_eq!(r.clmul(), k.clmul() && clmul, "{k:?} {avx2} {clmul}");
            }
        }
        assert_eq!(k.restrict(true, true), k);
        assert_eq!(k.restrict(false, false), Kernels::scalar());
    }
}

/// `new_unchecked` gives back what it was told
#[test]
fn new_unchecked_round_trips() {
    // SAFETY: nothing decodes with these kernels; they are only read back
    let k = unsafe { Kernels::new_unchecked(true, false) };
    assert!(k.avx2() && !k.clmul());
    // SAFETY: the scalar kernels are sound on any CPU
    assert_eq!(
        unsafe { Kernels::new_unchecked(false, false) },
        Kernels::scalar()
    );
}
