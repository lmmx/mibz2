// RLE1 word actions for an incoming parser state.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct State {
    last: u8,
    count: u8,
}
impl State {
    #[inline(always)]
    fn byte(&mut self, b: u8, out: &mut Vec<u8>) {
        if self.count == 4 {
            out.resize(out.len() + usize::from(b), self.last);
            *self = Self::default();
        } else {
            out.push(b);
            self.count = if self.last == b { self.count + 1 } else { 1 };
            self.last = b;
        }
    }
    fn feed(&mut self, data: &[u8], out: &mut Vec<u8>) {
        for &b in data {
            self.byte(b, out);
        }
    }
}

// After the first input, all five counts with known last byte w[0] cover
// every incoming state. A change when 4 is absent behaves as a fresh start.
fn first_cut(w: &[u8]) -> usize {
    let mut mask = 31u8;
    for i in 1..w.len() {
        if mask.is_power_of_two() || (mask & 16 == 0 && w[i] != w[i - 1]) {
            return i;
        }
        mask = if w[i] == w[i - 1] {
            (mask & 15) << 1 | mask >> 4
        } else {
            u8::from(mask & 15 != 0) * 2 | mask >> 4
        };
    }
    w.len()
}

const OWNED: u32 = 1 << 31;
const RAW_ONLY: u32 = u32::MAX;

#[derive(Clone, Copy)]
struct Action {
    source: u32, // literal spelling offset, tagged decoded offset, or RAW_ONLY
    size: u32,   // number of decoded bytes from an empty incoming state
    cut: u32,    // first reusable encoded suffix
    output_cut: u32,
    first: u8,
    after: State,
}

#[derive(Default)]
pub(super) struct Book {
    actions: Vec<Action>,
    expanded: Vec<u8>,
}

impl Book {
    fn compile(&mut self, spelling: &[u8], offsets: &[u32], budget: usize) {
        self.actions.clear();
        self.expanded.clear();
        for pair in offsets.windows(2) {
            let raw = &spelling[pair[0] as usize..pair[1] as usize];
            let cut = first_cut(raw);
            let mut a = Action {
                source: pair[0],
                size: raw.len() as u32,
                cut: cut as u32,
                output_cut: cut as u32,
                first: raw[0],
                after: State::default(),
            };
            if !raw
                .windows(4)
                .any(|q| q[0] == q[1] && q[0] == q[2] && q[0] == q[3])
            {
                let last = *raw.last().unwrap();
                a.after = State {
                    last,
                    count: raw.iter().rev().take_while(|&&b| b == last).count() as u8,
                };
            } else {
                // Size before allocating: an unvisited word may contain long
                // expansions, and construction is bounded even in that case.
                let mut st = State::default();
                let mut size = 0u32;
                let mut counted = false;
                for (i, &b) in raw.iter().enumerate() {
                    if i == cut {
                        a.output_cut = size;
                    }
                    if st.count == 4 {
                        counted = true;
                        size += u32::from(b);
                        st = State::default();
                    } else {
                        size += 1;
                        st.count = if st.last == b { st.count + 1 } else { 1 };
                        st.last = b;
                    }
                }
                a.after = st;
                a.size = size;
                if cut == raw.len() {
                    a.output_cut = size;
                }
                if counted {
                    if size as usize <= budget - self.expanded.len() {
                        a.source = self.expanded.len() as u32 | OWNED;
                        State::default().feed(raw, &mut self.expanded);
                    } else {
                        a.source = RAW_ONLY;
                    }
                }
            }
            self.actions.push(a);
        }
    }

    #[inline]
    fn write(
        &self,
        i: usize,
        spelling: &[u8],
        offsets: &[u32],
        remaining: usize,
        state: &mut State,
        out: &mut Vec<u8>,
    ) -> usize {
        let w = &spelling[offsets[i] as usize..offsets[i + 1] as usize];
        let a = self.actions[i];
        if w.len() > remaining || a.source == RAW_ONLY {
            let take = w.len().min(remaining);
            state.feed(&w[..take], out);
            return take;
        }
        let decoded = if a.source & OWNED == 0 {
            w
        } else {
            let begin = (a.source & !OWNED) as usize;
            &self.expanded[begin..begin + a.size as usize]
        };
        if state.count == 0 || (state.count < 4 && state.last != a.first) {
            out.extend_from_slice(decoded);
            *state = a.after;
        } else if (a.cut as usize) < w.len() {
            state.feed(&w[..a.cut as usize], out);
            out.extend_from_slice(&decoded[a.output_cut as usize..]);
            *state = a.after;
        } else {
            state.feed(w, out);
        }
        w.len()
    }
}
