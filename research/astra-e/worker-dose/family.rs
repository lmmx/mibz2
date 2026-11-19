use crate::{bwt::{Cells, InverseBwt}, runs::Runs, Kernels};

#[path = "../edge-budget/candidate.rs"]
mod edge;

enum Kernel {
    Eager(Cells),
    Edge(edge::Candidate),
}

pub struct Dose<const OBSERVE: bool>(Kernel);

impl<const OBSERVE: bool> Default for Dose<OBSERVE> {
    fn default() -> Self {
        // launch() names each persistent worker mibz2-0 through mibz2-(W-1).
        // A kernel constructed outside a pool uses the measured edge implementation.
        let thread = std::thread::current();
        let index = thread.name().and_then(|s| s.strip_prefix("mibz2-"))
            .and_then(|s| s.parse::<usize>().ok());
        let eager = index.is_some_and(|i| i % 4 == 0);
        if OBSERVE {
            eprintln!("dose-init {:?} {}", index, if eager { "eager" } else { "edge" });
        }
        if eager {
            Self(Kernel::Eager(Cells::default()))
        } else {
            Self(Kernel::Edge(edge::Candidate::default()))
        }
    }
}

macro_rules! dispatch {
    ($kernel:expr, $method:ident($($arg:expr),*)) => {
        match $kernel {
            Kernel::Eager(inner) => inner.$method($($arg),*),
            Kernel::Edge(inner) => inner.$method($($arg),*),
        }
    };
}

impl<const OBSERVE: bool> InverseBwt for Dose<OBSERVE> {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        dispatch!(&mut self.0, invert(ll, counts, p, out))
    }

    fn wants_checkpoints(&self) -> bool {
        dispatch!(&self.0, wants_checkpoints())
    }

    fn invert_checkpointed(&mut self, ll: &[u8], counts: &[u32; 256], cp: &[[u32; 256]], p: u32, out: &mut Vec<u8>) {
        dispatch!(&mut self.0, invert_checkpointed(ll, counts, cp, p, out))
    }

    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        dispatch!(&mut self.0, invert_in_place(ll, counts, p))
    }

    fn reads_runs(&self) -> bool {
        dispatch!(&self.0, reads_runs())
    }

    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if OBSERVE {
            eprintln!("dose-block {} {}", std::thread::current().name().unwrap_or(""), n);
        }
        dispatch!(&mut self.0, invert_runs(r, n, p, out, k))
    }
}
