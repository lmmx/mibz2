// Only the separately compiled diagnostic executable enables the stock timeline.
fn write_trace(path:&std::path::Path, workers:usize) {
    use std::io::Write;
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);
    while mibz2::trace::finished()<workers+1 && std::time::Instant::now()<deadline {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(mibz2::trace::finished()>=workers+1);
    let traces=mibz2::trace::take();
    mibz2::trace::set_level(0);
    let mut out=std::io::BufWriter::new(std::fs::File::create(path).unwrap());
    writeln!(out,"thread,kind,ns,place,cpu_ns,run_ns,wait_ns,len").unwrap();
    for t in traces {
        for m in t.marks {
            writeln!(out,"{},{:?},{},{},{},{},{},{}",t.name,m.kind,m.ns,m.place,m.cpu_ns,m.run_ns,m.wait_ns,m.len).unwrap();
        }
    }
}
