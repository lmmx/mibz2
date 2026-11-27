//! One scanner comparison sample on a fixed worker plan. The parent hashes check samples.
#[cfg(feature = "scan-oracle")]
mod sample {
    use mibz2::parallel::SLOTS_PER_THREAD;
    use mibz2::{scan_oracle, BzReader, Kernels};
    use std::io::{self, Read, Write};
    use std::time::Instant;

    fn decode(data: &'static [u8], workers: usize, mut dest: impl Write) -> io::Result<u64> {
        let mut reader = BzReader::parallel_workers(
            data,
            workers,
            workers * SLOTS_PER_THREAD,
            Kernels::detect(),
            true,
        );
        let mut buf = vec![0; 4 << 20];
        let mut total = 0;
        loop {
            let count = reader.read(&mut buf)?;
            if count == 0 {
                return Ok(total);
            }
            dest.write_all(&buf[..count])?;
            total += count as u64;
        }
    }

    pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<_> = std::env::args().collect();
        if args.len() != 6 {
            return Err("expected FILE WORKERS SCANNER_CPU real|C check|time".into());
        }
        let data: &'static [u8] = std::fs::read(&args[1])?.leak();
        let workers: usize = args[2].parse()?;
        let pin: usize = args[3].parse()?;
        let check = match args[5].as_str() {
            "check" => true,
            "time" => false,
            _ => return Err("unknown sample mode".into()),
        };
        scan_oracle::set_scanner_cpu(Some(pin));
        #[cfg(feature = "ideal-construction")]
        let constructed = {
            use mibz2::ideal::{construction as c, Mode};
            if args[4] != "C" {
                return Err("expected construction replay".into());
            }
            c::set_mode(Mode::Record);
            decode(data, workers, io::sink())?;
            let (_, _, _, conflicts, _) = c::store_size();
            let count = c::take_constructed();
            if count == 0 || conflicts != 0 {
                return Err("invalid construction store".into());
            }
            c::set_mode(Mode::Replay);
            count
        };
        #[cfg(not(feature = "ideal-construction"))]
        if args[4] != "real" {
            return Err("expected real construction".into());
        }

        scan_oracle::take_scanned();
        let start = Instant::now();
        let bytes = if check {
            decode(data, workers, io::stdout().lock())?
        } else {
            decode(data, workers, io::sink())?
        };
        let wall = start.elapsed().as_secs_f64();
        let blocks = scan_oracle::take_scanned();
        if scan_oracle::scanner_pin_ok() != Some(true)
            || blocks == 0
            || scan_oracle::take_replayed() != 0
        {
            return Err("real scanner or scanner pin missing".into());
        }
        #[cfg(feature = "ideal-construction")]
        let replayed = {
            use mibz2::ideal::construction as c;
            let built = c::take_replayed();
            if built == 0
                || built + c::take_refused_replayed() != constructed
                || c::take_constructed() != 0
                || c::take_lock_acquires() != 0
            {
                return Err("incomplete construction-only replay".into());
            }
            built
        };
        #[cfg(not(feature = "ideal-construction"))]
        let replayed = 0;
        let status = std::fs::read_to_string("/proc/self/status")?;
        let rss: u64 = status
            .lines()
            .find_map(|s| s.strip_prefix("VmHWM:"))
            .and_then(|s| s.split_whitespace().next())
            .ok_or("missing peak RSS")?
            .parse()?;
        #[cfg(target_arch = "x86_64")]
        let bmi2 = std::is_x86_feature_detected!("bmi2");
        #[cfg(not(target_arch = "x86_64"))]
        let bmi2 = false;
        eprintln!("{{\"wall_s\":{wall},\"peak_kib\":{rss},\"out\":{bytes},\"scanned\":{blocks},\"replayed_c\":{replayed},\"scanner_cpu\":{pin},\"bmi2\":{bmi2}}}");
        Ok(())
    }
}

fn main() {
    #[cfg(feature = "scan-oracle")]
    if let Err(e) = sample::run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
    #[cfg(not(feature = "scan-oracle"))]
    {
        eprintln!("build with --features scan-oracle");
        std::process::exit(1);
    }
}
