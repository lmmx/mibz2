pub fn corpora(scale: usize) -> Vec<(&'static str, Vec<u8>)> {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut text = Vec::new();
    for dir in ["src", "tests", "examples"] {
        let mut files: Vec<_> = std::fs::read_dir(format!("{root}/{dir}"))
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .collect();
        files.sort();
        for f in files.into_iter().filter(|f| f.is_file()) {
            text.extend(std::fs::read(f).unwrap());
        }
    }
    text.extend(std::fs::read(format!("{root}/README.md")).unwrap());
    let one = text.clone();
    let mut i = 0;
    while text.len() < 3_000_000 * scale {
        text.extend(format!("\n-- copy {i} --\n").bytes());
        text.extend(&one);
        i += 1;
    }
    let mut rng = Rng(0xfeed_beef_dead_cafe);
    let records: Vec<u8> = (0..600_000 * scale as u32)
        .flat_map(|i| (i.wrapping_mul(2_654_435_761) >> 12 | i << 20).to_le_bytes())
        .collect();
    let mut runs = Vec::new();
    while runs.len() < 2_500_000 * scale {
        let n = 1 + rng.below(600) as usize;
        runs.extend(std::iter::repeat(b"xyz"[rng.below(3) as usize]).take(n));
    }
    vec![
        ("text", text),
        ("random", rng.bytes(2_000_000 * scale)),
        ("one byte", vec![b'a'; 5_000_000 * scale]),
        ("pattern", b"0123456".repeat(700_000 * scale)),
        ("runs", runs),
        ("records", records),
    ]
}

