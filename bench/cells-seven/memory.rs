// Linux native post-exec observations. Never use these instrumented rows as timings.
fn memory_sample() -> serde_json::Value {
    let status = std::fs::read_to_string("/proc/self/status").expect("Linux proc status");
    let get = |key: &str| -> u64 {
        status.lines().find_map(|line| line.strip_prefix(key))
            .expect("required proc field").split_whitespace().next().unwrap().parse().unwrap()
    };
    json!({"pid": std::process::id(), "rss_kib": get("VmRSS:"), "hwm_kib": get("VmHWM:")})
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let startup = memory_sample();
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("control") {
        let mib: usize = args.get(2).ok_or("control size missing")?.parse()?;
        if ![1, 64].contains(&mib) { return Err("control must be 1 or 64 MiB".into()); }
        let mut bytes = vec![0u8; mib << 20];
        for (i, byte) in bytes.iter_mut().enumerate() { *byte = (i as u8).wrapping_add(1); }
        std::hint::black_box(&mut bytes);
        eprintln!("{}", json!({"memory": {"startup": startup, "end": memory_sample()}, "control_mib": mib}));
        std::hint::black_box(bytes);
        return Ok(());
    }
    let result = reader_main();
    eprintln!("{}", json!({"memory": {"startup": startup, "end": memory_sample()}, "success": result.is_ok()}));
    result
}
