#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/residual-24
out=${1:?fresh directory}
mkdir "$out"
out=$(cd "$out" && pwd)
python3 "$d/price.py" "$out"
cat >> "$out/new.rs" <<'RS'
#[test]
fn page_overflow_refuses_before_packing() {
    let mut k=Kernel::default();
    let n=8*(32768+1);
    k.bands=(0..n).map(|i|Band{end:i+1,code:(i&1)*64,delta:0}).collect();
    assert!(!k.build_packed::<8>(n as usize,19,&[0;256]));
    assert!(k.pack.is_empty() && k.high.is_empty());
    // A later small directory is freshly classified even after a refusal.
    k.bands.clear();k.bands.push(Band{end:1,code:0,delta:0});
    k.queues=vec![Vec::new();256];k.queues[0].push(0);
    let mut starts=[1;256];starts[0]=0;
    assert!(k.build_packed::<8>(1,1,&starts));
    assert_eq!((k.load24(0),k.label24::<8>(0,0,1)),(0,0));
}
RS
deps=${PAGE_DEPS:-${CARGO_TARGET_DIR:-target}/release/deps}
rustc --test --edition=2021 -C opt-level=3 -C codegen-units=1 "$out/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/test"
"$out/test" page_overflow
