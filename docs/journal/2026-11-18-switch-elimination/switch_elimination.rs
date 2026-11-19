//! Research kernel for Investigation 3. The marked orbit is a straight-line
//! grammar built by eliminating ordered switches and recoding their bands.
use crate::{Kernels, bwt::InverseBwt, runs::Runs};
use std::time::Instant;
#[path = "coincidence.rs"]
pub mod coincidence;
#[path = "forest.rs"]
pub mod forest;
#[path = "network.rs"]
mod network;
use network::Network;

const NONE: u32 = u32::MAX;

#[derive(Default, Debug, Clone, Copy)]
pub struct Pass {
    pub before_v: usize,
    pub before_e: usize,
    pub before_m: usize,
    pub after_v: usize,
    pub after_e: usize,
    pub after_m: usize,
    pub loop_vertices: usize,
    pub splices: usize,
    pub budget: bool,
}

#[derive(Default, Debug)]
pub struct OutputStats {
    pub roots: usize,
    pub opened: usize,
    pub literals: usize,
    pub hits: usize,
    pub copied: usize,
}

struct Rule {
    left: u32,
    right: u32,
    len: u32,
    first: u32,
}

#[derive(Default)]
struct Grammar {
    rules: Vec<Rule>,
    stack: Vec<u32>,
    stats: OutputStats,
}

impl Grammar {
    fn len(&self, id: u32) -> u32 {
        if id < 256 {
            1
        } else {
            self.rules[(id - 256) as usize].len
        }
    }

    fn join(&mut self, left: u32, right: u32) -> u32 {
        let id = self.rules.len() as u32 + 256;
        self.rules.push(Rule {
            left,
            right,
            len: self.len(left) + self.len(right),
            first: NONE,
        });
        id
    }

    // Once a whole nonterminal has appeared in the output, its next occurrence
    // can be copied. Only the very last expansion can be interrupted.
    fn write(&mut self, root: u32, limit: usize, out: &mut Vec<u8>) {
        self.stack.clear();
        self.stack.push(root);
        while let Some(id) = self.stack.pop() {
            if out.len() == limit {
                break;
            }
            if id < 256 {
                out.push(id as u8);
                self.stats.literals += 1;
            } else {
                let r = &mut self.rules[(id - 256) as usize];
                if r.first == NONE {
                    r.first = out.len() as u32;
                    self.stack.extend([r.right, r.left]);
                    self.stats.opened += 1;
                } else {
                    let take = (r.len as usize).min(limit - out.len());
                    let start = r.first as usize;
                    out.extend_from_within(start..start + take);
                    self.stats.hits += 1;
                    self.stats.copied += take;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Switches {
    runs: Vec<(u8, u32)>,
    graph: Network,
    scratch: Network,
    words: Grammar,
    pub passes: Vec<Pass>,
    pub build_seconds: f64,
    pub output_seconds: f64,
}

impl Switches {
    pub fn output_stats(&self) -> &OutputStats {
        &self.words.stats
    }

    fn add_run(&mut self, byte: u8, width: u32) {
        if let Some((last, n)) = self.runs.last_mut() {
            if *last == byte {
                *n += width;
                return;
            }
        }
        self.runs.push((byte, width));
    }

    fn reconstruct(&mut self, n: usize, p: u32, out: &mut Vec<u8>, begin: Instant) {
        self.passes.clear();
        self.words.rules.clear();
        self.words.stats = OutputStats::default();
        let mut mark = self.graph.from_runs(&self.runs, p);
        let mut charged_recodings = 0;
        loop {
            let allowance = 2 * n - charged_recodings;
            let mut s = self.graph.sweep(mark.0, &mut self.words, allowance);
            let upper = 2 * s.after_e - s.after_v;
            let terminal = s.after_e == s.after_v || s.after_m <= 64;
            let stop = terminal || s.budget || upper + self.words.rules.len() > allowance;
            s.budget |= !terminal && stop;
            self.passes.push(s);
            if stop {
                break;
            }
            mark = self.graph.recode(&mut self.scratch, mark);
            charged_recodings += self.scratch.initial_band_count();
            std::mem::swap(&mut self.graph, &mut self.scratch);
        }
        let built = Instant::now();
        self.graph.emit(mark, n, &mut self.words, out);
        self.build_seconds = (built - begin).as_secs_f64();
        self.output_seconds = built.elapsed().as_secs_f64();
    }
}

impl InverseBwt for Switches {
    fn reads_runs(&self) -> bool {
        true
    }

    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, _: Kernels) -> bool {
        let start = Instant::now();
        self.runs.clear();
        let mut total = 0;
        for &rec in r.records() {
            let width = u32::from(rec as u8);
            if width == 0 {
                return false;
            }
            self.add_run((rec >> 8) as u8, width);
            total += width as usize;
        }
        if total != n || n == 0 {
            return false;
        }
        self.reconstruct(n, p, out, start);
        true
    }

    fn invert(&mut self, l: &[u8], _: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        let start = Instant::now();
        self.runs.clear();
        for &c in l {
            self.add_run(c, 1);
        }
        if l.is_empty() {
            out.clear();
            return;
        }
        self.reconstruct(l.len(), p, out, start);
    }
}
