//! Measure the registered Open worker topology, then hash another decode at the same k.
use mibz2::{parallel::SLOTS_PER_THREAD, BzReader, Kernels};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::time::Instant;

fn decode(data: &'static [u8], workers: usize, mut dest: impl Write) -> io::Result<u64> {
    let mut reader = BzReader::parallel_workers(
        data,
        workers,
        SLOTS_PER_THREAD * workers,
        Kernels::detect(),
        true,
    );
    let mut buf = vec![0; 4 << 20];
    let mut bytes = 0;
    loop {
        let count = reader.read(&mut buf)?;
        if count == 0 {
            return Ok(bytes);
        }
        dest.write_all(&buf[..count])?;
        bytes += count as u64;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("expected a compressed corpus path and worker count".into());
    }
    // The feeder thread owns its input for this single-sample process.
    let data: &'static [u8] = std::fs::read(&args[1])?.leak();
    let workers = args[2].parse()?;
    if !matches!(workers, 1 | 8 | 10) {
        return Err("expected 1, 8 or 10 workers".into());
    }
    let start = Instant::now();
    let bytes = decode(data, workers, io::sink())?;
    let wall = start.elapsed().as_secs_f64();
    let status = std::fs::read_to_string("/proc/self/status")?;
    let rss: u64 = status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|v| v.split_whitespace().next())
        .ok_or("missing peak RSS")?
        .parse()?;
    let mut hash = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let checked = decode(
        data,
        workers,
        hash.stdin.take().ok_or("missing hash input")?,
    );
    let out = hash.wait_with_output()?;
    if checked? != bytes || !out.status.success() {
        return Err("hash pass failed".into());
    }
    let digest = std::str::from_utf8(&out.stdout)?
        .split_whitespace()
        .next()
        .ok_or("empty digest")?;
    println!(
        "{{\"k\":{workers},\"wall_s\":{wall},\"bytes\":{bytes},\"maxrss_kib\":{rss},\"sha256\":\"{digest}\"}}"
    );
    Ok(())
}
