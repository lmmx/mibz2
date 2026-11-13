//! Diagnostic-only, safe, single-thread hierarchical accounting. No pointer access.
#![forbid(unsafe_code)]
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::time::Instant;

#[derive(Clone, Debug, Default)]
pub struct Span {
    pub calls: u64,
    pub inclusive_ns: u128,
    pub exclusive_ns: u128,
}
#[derive(Clone, Debug, Default)]
pub struct Block {
    pub bit: u64,
    pub primary: u32,
    pub stored_crc: u32,
    pub counters: BTreeMap<&'static str, u64>,
    pub outcome: String,
}
#[derive(Debug, Default)]
pub struct Report {
    pub phases: BTreeMap<&'static str, Span>,
    pub counters: BTreeMap<&'static str, u64>,
    pub blocks: Vec<Block>,
    pub clock_calls: u64,
    pub event_growths: u64,
    pub event_capacity: usize,
}
struct Frame {
    name: &'static str,
    start: Option<Instant>,
    children: u128,
}
#[derive(Default)]
struct State {
    enabled: bool,
    timed: bool,
    stack: Vec<Frame>,
    current: Option<Block>,
    bit: u64,
    report: Report,
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }

pub fn reset(timed: bool) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        assert!(s.stack.is_empty());
        *s = State {
            enabled: true,
            timed,
            ..State::default()
        };
    });
}
pub fn take() -> Report {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        assert!(s.stack.is_empty());
        assert!(s.current.is_none());
        s.enabled = false;
        s.report.event_capacity = s.report.blocks.capacity();
        std::mem::take(&mut s.report)
    })
}
pub struct Guard {
    depth: usize,
    active: Cell<bool>,
}
pub fn phase(name: &'static str) -> Guard {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let enabled = s.enabled;
        let depth = s.stack.len();
        if enabled {
            let start = if s.timed {
                s.report.clock_calls += 1;
                Some(Instant::now())
            } else {
                None
            };
            s.stack.push(Frame {
                name,
                start,
                children: 0,
            });
        }
        Guard {
            depth,
            active: Cell::new(enabled),
        }
    })
}
impl Guard {
    pub fn stop(&self) {
        if !self.active.replace(false) {
            return;
        }
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            assert_eq!(s.stack.len(), self.depth + 1, "non-LIFO phase");
            let f = s.stack.pop().unwrap();
            let elapsed = f.start.map_or(0, |start| {
                s.report.clock_calls += 1;
                start.elapsed().as_nanos()
            });
            assert!(elapsed >= f.children, "child spans exceed parent");
            let span = s.report.phases.entry(f.name).or_default();
            span.calls += 1;
            span.inclusive_ns += elapsed;
            span.exclusive_ns += elapsed - f.children;
            if let Some(parent) = s.stack.last_mut() {
                parent.children += elapsed;
            }
        });
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        self.stop();
    }
}
pub fn add(name: &'static str, n: usize) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if !s.enabled {
            return;
        }
        *s.report.counters.entry(name).or_default() += n as u64;
        if let Some(b) = &mut s.current {
            *b.counters.entry(name).or_default() += n as u64;
        }
    });
}
pub fn set(name: &'static str, n: usize) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if !s.enabled {
            return;
        }
        if let Some(b) = &mut s.current {
            b.counters.insert(name, n as u64);
        }
    });
}
pub fn capacity(name: &str, n: usize) {
    let key = match name {
        "cells_buf" => "cells_buf_capacity",
        "cells_pos" => "cells_pos_capacity",
        "inverted" => "inverted_capacity",
        "decoded" => "decoded_capacity",
        _ => panic!("unknown capacity"),
    };
    set(key, n);
}
pub fn growth(name: &str, old: usize, required: usize) {
    if required > old {
        let (growth, bytes) = match name {
            "inverted" => ("inverted_growths", "inverted_requested_bytes"),
            "cells_pos" => ("cells_pos_growths", "cells_pos_requested_bytes"),
            _ => panic!("unknown growth"),
        };
        add(growth, 1);
        add(bytes, required);
    }
}
pub fn scan_bit(bit: u64) {
    STATE.with(|s| s.borrow_mut().bit = bit);
}
pub fn outcome(outcome: &str) {
    STATE.with(|s| {
        if let Some(b) = &mut s.borrow_mut().current {
            b.outcome = outcome.into();
        }
    });
}
pub struct BlockGuard;
pub fn block(primary: u32, stored_crc: u32) -> BlockGuard {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if !s.enabled {
            return;
        }
        assert!(s.current.is_none());
        s.current = Some(Block {
            bit: s.bit,
            primary,
            stored_crc,
            ..Block::default()
        });
    });
    BlockGuard
}
impl Drop for BlockGuard {
    fn drop(&mut self) {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            if let Some(b) = s.current.take() {
                if s.report.blocks.len() == s.report.blocks.capacity() {
                    s.report.event_growths += 1;
                }
                s.report.blocks.push(b);
            }
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn untimed_stack_and_error_cleanup() {
        reset(false);
        {
            let _whole = phase("whole");
            let _child = phase("child");
            let _block = block(3, 7);
            add("abandoned", 5);
        }
        let r = take();
        assert_eq!(r.clock_calls, 0);
        assert_eq!(r.blocks[0].counters["abandoned"], 5);
        assert_eq!(r.phases["child"].inclusive_ns, 0);
        reset(false);
        let g = phase("twice");
        g.stop();
        g.stop();
        drop(g);
        assert_eq!(take().phases["twice"].calls, 1);
    }
}
