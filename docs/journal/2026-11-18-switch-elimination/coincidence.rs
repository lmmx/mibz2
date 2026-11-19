//! First return to unmatched tiles. Bands are stored in image order, with
//! their IDs in a second list for domain order. A round uses two linear scans
//! and follows disjoint chains of whole bands once.
use crate::{Kernels, bwt::InverseBwt, runs::Runs};
use std::time::Instant;
const NONE: u32 = u32::MAX;
const LITERAL: u32 = 1 << 31;

#[derive(Default, Clone, Copy)]
struct Band {
    w: u32,
    word: u32,
}

#[derive(Default)]
struct Exchange {
    image: Vec<Band>,
    domain: Vec<u32>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Round {
    pub rows: usize,
    pub bands: usize,
    pub common: usize,
    pub removed: usize,
    pub next_rows: usize,
    pub next_bands: usize,
    pub chains: usize,
    pub joined: usize,
    pub children: usize,
}

#[derive(Default, Debug)]
pub struct WriteStats {
    pub roots: usize,
    pub opened: usize,
    pub literal_bytes: usize,
    pub copies: usize,
    pub copied: usize,
}

struct Word {
    offset: u32,
    count: u32,
    len: u32,
    first: u32,
}

#[derive(Default)]
struct Words {
    records: Vec<Word>,
    children: Vec<u32>,
    literal: Vec<u8>,
    path: Vec<u32>,
    stack: Vec<u32>,
    stats: WriteStats,
}

impl Words {
    fn clear(&mut self) {
        self.records.clear();
        self.children.clear();
        self.literal.clear();
        self.path.clear();
        self.stack.clear();
        self.stats = WriteStats::default();
    }

    fn finish_path(&mut self) -> u32 {
        if self.path.len() == 1 {
            return self.path[0];
        }
        let id = 256 + self.records.len() as u32;
        let count = self.path.len() as u32;
        let (offset, len) = if self.path.iter().all(|&x| x < 256) {
            let off = self.literal.len() as u32;
            self.literal
                .extend(self.path.iter().rev().map(|&x| x as u8));
            (LITERAL | off, count)
        } else {
            let off = self.children.len() as u32;
            let mut len = 0;
            for &x in self.path.iter().rev() {
                self.children.push(x);
                len += if x < 256 {
                    1
                } else {
                    self.records[(x - 256) as usize].len
                };
            }
            (off, len)
        };
        self.records.push(Word {
            offset,
            count,
            len,
            first: NONE,
        });
        id
    }

    fn write(&mut self, root: u32, n: usize, out: &mut Vec<u8>) {
        self.stack.clear();
        self.stack.push(root);
        while let Some(id) = self.stack.pop() {
            let room = n - out.len();
            if room == 0 {
                return;
            }
            if id < 256 {
                out.push(id as u8);
                self.stats.literal_bytes += 1;
                continue;
            }
            let w = &mut self.records[(id - 256) as usize];
            let take = room.min(w.len as usize);
            if w.offset & LITERAL != 0 {
                let at = (w.offset & !LITERAL) as usize;
                out.extend_from_slice(&self.literal[at..at + take]);
                self.stats.literal_bytes += take;
            } else if w.first != NONE {
                let at = w.first as usize;
                out.extend_from_within(at..at + take);
                self.stats.copies += 1;
                self.stats.copied += take;
            } else {
                w.first = out.len() as u32;
                let at = w.offset as usize;
                self.stack
                    .extend(self.children[at..at + w.count as usize].iter().rev());
                self.stats.opened += 1;
            }
        }
    }
}

#[derive(Default)]
struct Work {
    successor: Vec<u32>,
    has_predecessor: Vec<bool>,
    parts: Vec<u32>,
}

impl Exchange {
    fn clear(&mut self) {
        self.image.clear();
        self.domain.clear();
    }

    fn round(
        &self,
        m: u32,
        p: u32,
        next: &mut Self,
        work: &mut Work,
        words: &mut Words,
    ) -> (Round, u32) {
        let b = self.image.len();
        let mut s = Round {
            rows: m as usize,
            bands: b,
            ..Round::default()
        };
        next.clear();
        work.successor.clear();
        work.successor.resize(b, NONE);
        work.has_predecessor.clear();
        work.has_predecessor.resize(b, false);
        work.parts.clear();
        let mut domain_index = 0;
        let mut spent = 0;
        let (mut old_row, mut retained, mut mark) = (0, 0, NONE);
        for (i, band) in self.image.iter().enumerate() {
            work.parts.push(next.image.len() as u32);
            let mut left = band.w;
            while left > 0 {
                let j = self.domain[domain_index] as usize;
                let width = self.image[j].w;
                let take = left.min(width - spent);
                let exact = left == band.w && spent == 0 && left == width;
                let marked = old_row <= p && p < old_row + take;
                if exact && !marked {
                    work.successor[i] = j as u32;
                    work.has_predecessor[j] = true;
                    s.removed += 1;
                } else {
                    if marked {
                        mark = retained + p - old_row;
                    }
                    next.image.push(Band {
                        w: take,
                        word: NONE,
                    });
                    retained += take;
                }
                left -= take;
                spent += take;
                old_row += take;
                if spent == width {
                    domain_index += 1;
                    spent = 0;
                    if left == 0 {
                        s.common += 1;
                    }
                }
            }
        }
        work.parts.push(next.image.len() as u32);
        assert_ne!(mark, NONE);
        for &first in &self.domain {
            if work.has_predecessor[first as usize] {
                continue;
            }
            words.path.clear();
            let mut edge = first as usize;
            loop {
                words.path.push(self.image[edge].word);
                let successor = work.successor[edge];
                if successor == NONE {
                    break;
                }
                edge = successor as usize;
            }
            s.chains += 1;
            if words.path.len() > 1 {
                s.joined += 1;
                s.children += words.path.len();
            }
            let word = words.finish_path();
            for k in work.parts[edge]..work.parts[edge + 1] {
                next.image[k as usize].word = word;
                next.domain.push(k);
            }
        }
        debug_assert_eq!(next.domain.len(), next.image.len());
        s.next_rows = retained as usize;
        s.next_bands = next.image.len();
        debug_assert_eq!(s.next_bands, 2 * b - s.common - s.removed);
        (s, mark)
    }

    fn emit(&self, p: u32, n: usize, words: &mut Words, out: &mut Vec<u8>) {
        let mut from = vec![0; self.image.len()];
        let mut to = Vec::with_capacity(self.image.len());
        let mut row = 0;
        for &id in &self.domain {
            from[id as usize] = row;
            row += self.image[id as usize].w;
        }
        row = 0;
        for b in &self.image {
            to.push(row);
            row += b.w;
        }
        out.clear();
        out.reserve(n);
        let mut x = p;
        while out.len() < n {
            let i = to.partition_point(|&base| base <= x) - 1;
            words.stats.roots += 1;
            words.write(self.image[i].word, n, out);
            x = from[i] + x - to[i];
            if x == p {
                while out.len() < n {
                    let take = out.len().min(n - out.len());
                    out.extend_from_within(..take);
                    words.stats.copied += take;
                }
                break;
            }
        }
    }
}

#[derive(Default)]
pub struct Coincidence {
    runs: Vec<Band>,
    active: Exchange,
    scratch: Exchange,
    work: Work,
    words: Words,
    pub rounds: Vec<Round>,
    pub budget_stop: bool,
    pub build_seconds: f64,
    pub output_seconds: f64,
}

impl Coincidence {
    pub fn stats(&self) -> &WriteStats {
        &self.words.stats
    }

    fn add(&mut self, c: u32, w: u32) {
        if let Some(last) = self.runs.last_mut() {
            if last.word == c {
                last.w += w;
                return;
            }
        }
        self.runs.push(Band { word: c, w });
    }

    fn run(&mut self, n: usize, p: u32, out: &mut Vec<u8>, begin: Instant) {
        self.active.clear();
        self.words.clear();
        self.rounds.clear();
        let mut count = [0u32; 256];
        for b in &self.runs {
            count[b.word as usize] += 1;
        }
        let mut sum = 0;
        let mut slot = count.map(|width| {
            let at = sum;
            sum += width;
            at
        });
        self.active.image.resize(self.runs.len(), Band::default());
        for &b in &self.runs {
            let j = &mut slot[b.word as usize];
            self.active.image[*j as usize] = b;
            self.active.domain.push(*j);
            *j += 1;
        }
        let (mut m, mut mark) = (n as u32, p);
        let mut total = 0;
        self.budget_stop = false;
        while m > 64 && self.active.image.len() > 1 {
            let b = self.active.image.len();
            // Charge an input band once per round. At most 2b output pieces.
            if total + b > 2 * n {
                self.budget_stop = true;
                break;
            }
            total += b;
            let (s, next_mark) =
                self.active
                    .round(m, mark, &mut self.scratch, &mut self.work, &mut self.words);
            self.rounds.push(s);
            m = s.next_rows as u32;
            mark = next_mark;
            std::mem::swap(&mut self.active, &mut self.scratch);
        }
        let built = Instant::now();
        self.active.emit(mark, n, &mut self.words, out);
        self.build_seconds = (built - begin).as_secs_f64();
        self.output_seconds = built.elapsed().as_secs_f64();
    }
}

impl InverseBwt for Coincidence {
    fn reads_runs(&self) -> bool {
        true
    }

    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, _: Kernels) -> bool {
        let begin = Instant::now();
        self.runs.clear();
        let mut sum = 0;
        for &rec in r.records() {
            let w = u32::from(rec as u8);
            if w == 0 {
                return false;
            }
            self.add((rec >> 8) as u32, w);
            sum += w as usize;
        }
        if sum != n || n == 0 {
            return false;
        }
        self.run(n, p, out, begin);
        true
    }

    fn invert(&mut self, l: &[u8], _: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        let begin = Instant::now();
        self.runs.clear();
        for &c in l {
            self.add(c as u32, 1);
        }
        if l.is_empty() {
            out.clear();
            return;
        }
        self.run(l.len(), p, out, begin);
    }
}

#[cfg(test)]
#[path = "coincidence_audit.rs"]
mod audit;
