//! Turns on examples/ab.rs's arms that need this package's dependencies
fn main() {
    println!("cargo::rustc-check-cfg=cfg(mibz2_alternatives)");
    println!("cargo::rustc-cfg=mibz2_alternatives");
}
