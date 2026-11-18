//! Names the candidate: MOONSHOT_CANDIDATE, a Rust source file (absolute, or relative to the
//! repository's root), compiled as the module `candidate`. Unset, the null candidate
//! (candidates/lazy_cells.rs: 0.5.6's own kernel, which must score 1.0).
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-env-changed=MOONSHOT_CANDIDATE");
    let here = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = here.join("../..");
    let path = match std::env::var("MOONSHOT_CANDIDATE") {
        Ok(p) if !p.is_empty() => {
            let p = PathBuf::from(p);
            if p.is_absolute() {
                p
            } else {
                root.join(p)
            }
        }
        _ => here.join("candidates/lazy_cells.rs"),
    };
    let path = path
        .canonicalize()
        .unwrap_or_else(|e| panic!("MOONSHOT_CANDIDATE {}: {e}", path.display()));
    let shown = path.display().to_string();
    // (the file and its siblings: a candidate may have submodules beside it)
    println!("cargo::rerun-if-changed={shown}");
    if let Some(dir) = path.parent() {
        println!("cargo::rerun-if-changed={}", dir.display());
    }
    println!("cargo::rustc-env=MOONSHOT_CANDIDATE_PATH={shown}");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("candidate.rs");
    std::fs::write(
        out,
        format!("#[path = {shown:?}]\n#[allow(clippy::all)]\npub mod candidate;\n"),
    )
    .unwrap();
}
