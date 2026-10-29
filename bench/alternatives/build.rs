//! Turns on examples/ab.rs's arms that need this package's dependencies, and, with
//! LIBSAIS_DIR (a directory holding liblibsais.a: bench/libsais.sh), the kernels binary's
//! libsais kernel
fn main() {
    println!("cargo::rustc-check-cfg=cfg(mibz2_alternatives)");
    println!("cargo::rustc-check-cfg=cfg(mibz2_libsais)");
    println!("cargo::rustc-cfg=mibz2_alternatives");
    println!("cargo::rerun-if-env-changed=LIBSAIS_DIR");
    if let Ok(dir) = std::env::var("LIBSAIS_DIR") {
        if std::path::Path::new(&dir).join("liblibsais.a").exists() {
            println!("cargo::rerun-if-changed={dir}/liblibsais.a");
            println!("cargo::rustc-link-search=native={dir}");
            println!("cargo::rustc-link-lib=static=libsais");
            println!("cargo::rustc-cfg=mibz2_libsais");
        }
    }
}
