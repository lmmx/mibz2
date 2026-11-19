//! Return directly to the interface between the two strip colours, plus the primary.
use super::{each_run, starts, Counts, PAGE};
use crate::bwt::{InverseBwt, Segmented};
use crate::{runs::Runs, Kernels};

#[derive(Clone, Copy)]
struct Run {
    start: u32,
    end: u32,
    image: u32,
    byte: u8,
    thin: bool,
}

#[derive(Clone, Copy)]
struct Port {
    start: u32,
    end: u32,
    rank: u32,
}

#[derive(Clone, Copy)]
struct Entry {
    end: u32,
    delta: u32,
    byte: u8,
    port: bool,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SwitchCounts {
    pub common: Counts,
    pub thin_area: usize,
    pub thin_traced: usize,
    pub thin_words: usize,
}

#[derive(Default)]
pub struct SwitchKernel {
    runs: Vec<Run>,
    image: Vec<u32>,
    ports: Vec<Port>,
    entries: Vec<Entry>,
    directory: Vec<u32>,
    port_directory: Vec<u32>,
    words: Vec<u8>,
    ends: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
}

fn add_port(ports: &mut Vec<Port>, start: u32, end: u32) {
    if start == end {
        return;
    }
    let mut rank = 0;
    if let Some(last) = ports.last_mut() {
        if last.end == start {
            last.end = end;
            return;
        }
        rank = last.rank + last.end - last.start;
    }
    ports.push(Port { start, end, rank });
}

impl SwitchKernel {
    fn partition(&mut self, records: &[u16], n: u32, p: u32, limit: u32) -> Counts {
        let mut counts = [0; 256];
        let mut freq = [0; 256];
        let mut stats = Counts {
            n: n as usize,
            ..Counts::default()
        };
        self.runs.clear();
        let mut last = None;
        each_run(records, p, limit, |c, x, w, thin| {
            let a = usize::from(c);
            self.runs.push(Run {
                start: x,
                end: x + w,
                image: counts[a],
                byte: c,
                thin,
            });
            stats.runs += usize::from(last != Some(c));
            last = Some(c);
            stats.thick += usize::from(!thin);
            counts[a] += w;
            freq[a] += 1;
        });
        assert_eq!(counts.iter().sum::<u32>(), n);
        let first = starts(counts);
        let mut slots = starts(freq);
        self.image.resize(self.runs.len(), 0);
        for (i, r) in self.runs.iter_mut().enumerate() {
            let a = usize::from(r.byte);
            self.image[slots[a] as usize] = i as u32;
            slots[a] += 1;
            r.image += first[a];
        }
        self.ports.clear();
        let (mut x, mut i, mut j) = (0, 0, 0);
        while x < n {
            let domain = self.runs[i];
            let image = self.runs[self.image[j] as usize];
            let image_end = image.image + (image.end - image.start);
            let end = domain.end.min(image_end);
            if domain.thin != image.thin {
                add_port(&mut self.ports, x, end);
            } else if (x..end).contains(&p) {
                add_port(&mut self.ports, p, p + 1);
            }
            i += usize::from(domain.end == end);
            j += usize::from(image_end == end);
            x = end;
        }
        // Split only where the port flag or the actual LF strip changes.
        self.entries.clear();
        j = 0;
        for r in &self.runs {
            x = r.start;
            while x < r.end {
                let port = self.ports.get(j);
                let on = port.is_some_and(|v| v.start <= x);
                let end = port.map_or(r.end, |v| r.end.min(if on { v.end } else { v.start }));
                self.entries.push(Entry {
                    end,
                    delta: r.image.wrapping_sub(r.start),
                    byte: r.byte,
                    port: on,
                });
                j += usize::from(port.is_some_and(|v| v.end == end));
                x = end;
            }
        }
        self.directory.clear();
        self.port_directory.clear();
        let (mut ti, mut pi) = (0, 0);
        for row in (0..n).step_by(PAGE) {
            while self.entries[ti].end <= row {
                ti += 1;
            }
            while pi < self.ports.len() && self.ports[pi].end <= row {
                pi += 1;
            }
            self.directory.push(ti as u32);
            self.port_directory.push(pi as u32);
        }
        stats
    }

    fn entry(&self, x: u32) -> Entry {
        let mut i = self.directory[x as usize / PAGE] as usize;
        while self.entries[i].end <= x {
            i += 1;
        }
        self.entries[i]
    }

    fn port(&self, x: u32) -> Port {
        let mut i = self.port_directory[x as usize / PAGE] as usize;
        while self.ports[i].end <= x {
            i += 1;
        }
        self.ports[i]
    }

    fn build(&mut self, p: u32, stats: &mut SwitchCounts) -> (u32, u32) {
        let last = *self.ports.last().unwrap();
        let m = last.rank + last.end - last.start;
        let pp = self.port(p);
        let mark = pp.rank + p - pp.start;
        // Each port band has at least one row. Use the narrow representation
        // whenever the actual codes fit, also for large interfaces.
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        self.edges.resize(m as usize, 0);
        self.extra.clear();
        self.words.clear();
        self.ends.clear();
        self.ends.push(0);
        let mut run = 0;
        for pi in 0..self.ports.len() {
            let port = self.ports[pi];
            let mut a = port.start;
            while a < port.end {
                while self.runs[run].end <= a {
                    run += 1;
                }
                let first_run = self.runs[run];
                let mut width = port.end.min(first_run.end) - a;
                let start = self.words.len();
                self.words.push(first_run.byte);
                let mut x = first_run.image + (a - first_run.start);
                loop {
                    let t = self.entry(x);
                    if t.port {
                        break;
                    }
                    width = width.min(t.end - x);
                    self.words.push(t.byte);
                    x = x.wrapping_add(t.delta);
                }
                let exit = self.port(x);
                width = width.min(exit.end - x);
                let height = self.words.len() - start;
                let common = &mut stats.common;
                common.bands += 1;
                if first_run.thin {
                    stats.thin_traced += height;
                    stats.thin_area += width as usize * height;
                    stats.thin_words += 1;
                } else {
                    common.words += 1;
                    common.copies += width as usize;
                    common.traced += height;
                    common.thick_area += width as usize * height;
                    common.unit_work += usize::from(width == 1) * height;
                    common.longest = common.longest.max(height);
                }
                let code = if height == 1 {
                    u32::from(self.words.pop().unwrap())
                } else {
                    self.words[start..].reverse();
                    let id = 255 + self.ends.len() as u32;
                    self.ends.push(self.words.len() as u32);
                    id
                };
                let src = port.rank + (a - port.start);
                let dst = (exit.rank + x - exit.start) as usize;
                let span = width as usize;
                if code > (u32::MAX >> bits) && self.extra.is_empty() {
                    self.extra.extend(self.edges.iter().map(|e| e >> bits));
                }
                if !self.extra.is_empty() {
                    self.extra[dst..dst + span].fill(code);
                }
                let mut value = (code << bits) | src;
                for e in &mut self.edges[dst..dst + span] {
                    *e = value;
                    value = value.wrapping_add(1);
                }
                a += width;
            }
        }
        stats.common.retained = m as usize;
        stats.common.table_bytes = 4 * (self.edges.len() + self.extra.len());
        (mark, bits)
    }

    pub fn run(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        out: &mut Vec<u8>,
    ) -> SwitchCounts {
        let common = self.partition(records, n as u32, p, limit);
        let mut stats = SwitchCounts {
            common,
            ..SwitchCounts::default()
        };
        let (mark, bits) = self.build(p, &mut stats);
        super::emit(
            &self.edges,
            &self.extra,
            &self.words,
            &self.ends,
            n,
            mark,
            bits,
            out,
        );
        stats
    }

    pub fn price(
        &mut self,
        runs: &Runs,
        n: usize,
        p: u32,
        limit: u32,
        out: &mut Vec<u8>,
    ) -> (f64, f64) {
        let start = std::time::Instant::now();
        let common = self.partition(runs.records(), n as u32, p, limit);
        let mut stats = SwitchCounts {
            common,
            ..SwitchCounts::default()
        };
        let (mark, bits) = self.build(p, &mut stats);
        let built = start.elapsed().as_secs_f64();
        let start = std::time::Instant::now();
        super::emit(
            &self.edges,
            &self.extra,
            &self.words,
            &self.ends,
            n,
            mark,
            bits,
            out,
        );
        (built, start.elapsed().as_secs_f64())
    }

    pub fn column(&mut self, l: &[u8], p: u32, limit: u32, out: &mut Vec<u8>) -> SwitchCounts {
        self.run(&super::records_of(l), l.len(), p, limit, out)
    }
}

#[derive(Default)]
pub struct Switch<const LIMIT: u32> {
    kernel: SwitchKernel,
    other: Segmented,
}
impl<const LIMIT: u32> InverseBwt for Switch<LIMIT> {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, _: Kernels) -> bool {
        self.kernel.run(r.records(), n, p, LIMIT, out);
        true
    }
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.other.invert(l, c, p, out);
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.other.invert_in_place(l, c, p)
    }
}
