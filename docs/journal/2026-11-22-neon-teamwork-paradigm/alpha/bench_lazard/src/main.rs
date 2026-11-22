#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::Kernels;

const NONE: u32 = u32::MAX;

// =============================================================================
// Minimal self-contained SHA-256 for manifest.json verification (no extra deps)
// =============================================================================
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let bit_len = (data.len() as u64) * 8;
    let mut msg = Vec::with_capacity(((data.len() + 9 + 63) / 64) * 64);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (hv, v) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *hv = hv.wrapping_add(v);
        }
    }
    let mut s = String::with_capacity(64);
    for hv in h {
        s.push_str(&format!("{hv:08x}"));
    }
    s
}

// =============================================================================
// Build exact mibz2::runs::Runs from raw .L column via MTF + RLE2 + runs::front
// =============================================================================
fn build_runs_from_column(ll: &[u8], orig_ptr: u32, k: Kernels) -> Runs {
    let mut present = [false; 256];
    for &b in ll {
        present[b as usize] = true;
    }
    let mut seq_to_unseq = [0u8; 256];
    let mut mtf_list = [0u8; 256];
    let mut in_use = 0usize;
    for b in 0..256 {
        if present[b] {
            seq_to_unseq[in_use] = b as u8;
            mtf_list[in_use] = b as u8;
            in_use += 1;
        }
    }
    let eob = (in_use + 1) as u16;
    let mut syms: Vec<u16> = Vec::with_capacity(ll.len() / 2 + 16);
    let mut zero_run = 0usize;
    let flush_zeros = |zr: &mut usize, out: &mut Vec<u16>| {
        let mut z = *zr;
        while z > 0 {
            if (z & 1) != 0 {
                out.push(0); // RUNA
                z = (z - 1) >> 1;
            } else {
                out.push(1); // RUNB
                z = (z - 2) >> 1;
            }
        }
        *zr = 0;
    };
    for &b in ll {
        let pos = mtf_list[..in_use].iter().position(|&x| x == b).unwrap();
        if pos == 0 {
            zero_run += 1;
        } else {
            flush_zeros(&mut zero_run, &mut syms);
            // Move to front
            mtf_list.copy_within(0..pos, 1);
            mtf_list[0] = b;
            syms.push((pos + 1) as u16);
        }
    }
    flush_zeros(&mut zero_run, &mut syms);
    syms.push(eob);

    let sym_block = Symbols {
        crc: 0,
        orig_ptr,
        seq_to_unseq,
        syms,
        eob,
        max_len: 900_000,
    };
    let mut r = Runs::default();
    let n = runs::front(&sym_block, &mut r, k).expect("runs::front");
    assert_eq!(n, ll.len());
    r
}

/// Exact m_16 retained row count of Open<16> directly from I0 run records
fn compute_open_m16(records: &[u16], orig_ptr: u32) -> (usize, usize) {
    let mut m16 = 0usize;
    let mut r_max = 0usize;
    let mut row = 0u32;
    let mut i = 0usize;
    while i < records.len() {
        let byte = records[i] >> 8;
        let mut len = (records[i] & 0xff) as u32;
        i += 1;
        while i < records.len() && (records[i] >> 8) == byte {
            len += (records[i] & 0xff) as u32;
            i += 1;
        }
        r_max += 1;
        if len <= 16 {
            m16 += len as usize;
        } else if orig_ptr >= row && orig_ptr < row + len {
            // Marked row inside a thick strip becomes 1 retained row
            m16 += 1;
        }
        row += len;
    }
    (r_max, m16)
}

// =============================================================================
// Flat-Slab Safe-Rust Lazard-BWT (MAX-bucket elimination, zero HashMaps, zero per-block allocs)
// =============================================================================
#[derive(Clone, Copy, Default)]
struct RunNode {
    sym: u32,
    len: u32,
    prev: u32,
    next: u32,
    sym_prev: u32, // LIFO stack link for sym_stack[sym] (top = rightmost run of sym in L)
}

#[derive(Clone, Copy, Default)]
struct SymMeta {
    stack_top: u32, // rightmost RunNode index for this sym (NONE if empty)
    freq: u32,
    prev_sym: u32,  // ordered alphabet DLL
    next_sym: u32,
    stamp: u32,     // step stamp for O(1) k=1 dedup
    k1_sid: u32,    // sid created for (this_sym, x, k=1) at `stamp`
}

#[derive(Clone, Copy, Default)]
struct Prod {
    y: u32,
    x: u32,
    k: u32,
    len: u32,
}

#[derive(Clone, Copy, Default)]
struct LazardStats {
    steps: usize,
    fx_runs: usize,
    fx_nonseq: usize,
    ux_runs: usize,
    ux_inplace: usize,
    spliced_runs: usize,
    tot_syms: usize,
    max_nodes: usize,
    emit_exp: usize,
    emit_cpy: usize,
    ws_phase1_bytes: usize,
    ws_phase2_bytes: usize,
}

#[derive(Default)]
struct LazardMaxKernel {
    nodes: Vec<RunNode>,
    syms: Vec<SymMeta>,
    prods: Vec<Prod>,
    // Reusable step scratch
    fx_rev: Vec<(u32, u32, u32)>,        // (sym, len, p_off_from_right_or_NONE)
    exit_rev: Vec<(u32, u32, u32, u32, u32)>, // (y, k, len, p_off_r_or_NONE, p_steps)
    exit_lr: Vec<(u32, u32, u32, u32)>,  // (sid, len, p_off_l_or_NONE, p_steps)
    extra_k: Vec<(u32, u32, u32)>,       // (ey, ek>=2, sid)
    ux_buf: Vec<u32>,
    // Reusable emission scratch
    first_pos: Vec<u32>,
    buf: Vec<u8>,
    stack: Vec<(u32, u32)>,
}

impl LazardMaxKernel {
    fn invert_runs_counted(
        &mut self,
        records: &[u16],
        n: usize,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        count_stats: bool,
    ) -> LazardStats {
        let mut st = LazardStats::default();
        out.clear();
        if n == 0 {
            return st;
        }

        // Step 0: Sequential scan of R capped records -> maximal runs in flat slab `nodes[0..r)`
        self.nodes.clear();
        self.syms.clear();
        self.syms.resize(256, SymMeta {
            stack_top: NONE,
            freq: 0,
            prev_sym: NONE,
            next_sym: NONE,
            stamp: 0,
            k1_sid: NONE,
        });
        self.prods.clear();
        self.prods.resize(256, Prod { y: 0, x: 0, k: 0, len: 1 });

        let mut p_node: u32 = NONE;
        let mut p_off: u32 = 0;
        let mut row: u32 = 0;
        let mut i = 0usize;
        while i < records.len() {
            let rec = records[i];
            let c = (rec >> 8) as u32;
            let mut len = (rec & 0xff) as u32;
            i += 1;
            while i < records.len() && (records[i] >> 8) as u32 == c {
                len += (records[i] & 0xff) as u32;
                i += 1;
            }
            let nid = self.nodes.len() as u32;
            let prev = if nid == 0 { NONE } else { nid - 1 };
            let sym_prev = self.syms[c as usize].stack_top;
            self.syms[c as usize].stack_top = nid;
            self.syms[c as usize].freq += len;
            self.nodes.push(RunNode {
                sym: c,
                len,
                prev,
                next: NONE,
                sym_prev,
            });
            if nid > 0 {
                self.nodes[(nid - 1) as usize].next = nid;
            }
            if orig_ptr >= row && orig_ptr < row + len {
                p_node = nid;
                p_off = orig_ptr - row;
            }
            row += len;
        }
        debug_assert_eq!(row as usize, n);

        let mut head: u32 = 0;
        let mut tail: u32 = (self.nodes.len() - 1) as u32;

        // Build initial sorted symbol DLL over present bytes 0..256
        let mut sym_tail: u32 = NONE;
        let mut prev_s: u32 = NONE;
        for c in 0u32..256 {
            if self.syms[c as usize].freq > 0 {
                self.syms[c as usize].prev_sym = prev_s;
                if prev_s != NONE {
                    self.syms[prev_s as usize].next_sym = c;
                }
                prev_s = c;
                sym_tail = c;
            }
        }

        let mut p_byte_shift: usize = 0;
        let mut root_sym: u32 = NONE;
        let mut root_mult: u32 = 1;
        let mut step_id: u32 = 0;

        // Main MAX-Lazard elimination loop
        while sym_tail != NONE {
            let x = sym_tail;
            let fx = self.syms[x as usize].freq;
            let prev_x = self.syms[x as usize].prev_sym;
            sym_tail = prev_x;
            if prev_x != NONE {
                self.syms[prev_x as usize].next_sym = NONE;
            }
            if fx == 0 {
                continue;
            }
            step_id += 1;
            if count_stats {
                st.steps += 1;
            }

            // 1. Pop suffix F_x of length fx from tail of L
            self.fx_rev.clear();
            let mut rem = fx;
            let mut prev_pop_id: u32 = NONE;
            while rem > 0 {
                let nid = tail;
                if count_stats && prev_pop_id != NONE && nid.wrapping_add(1) != prev_pop_id {
                    st.fx_nonseq += 1;
                }
                prev_pop_id = nid;
                let node = self.nodes[nid as usize];
                if node.len <= rem {
                    let pro = if nid == p_node {
                        p_node = NONE;
                        node.len - 1 - p_off
                    } else {
                        NONE
                    };
                    self.fx_rev.push((node.sym, node.len, pro));
                    rem -= node.len;
                    tail = node.prev;
                    if tail != NONE {
                        self.nodes[tail as usize].next = NONE;
                    } else {
                        head = NONE;
                    }
                    // Pop from LIFO stack of node.sym (guaranteed top!)
                    debug_assert_eq!(self.syms[node.sym as usize].stack_top, nid);
                    self.syms[node.sym as usize].stack_top = node.sym_prev;
                } else {
                    let cut = node.len - rem;
                    let pro = if nid == p_node && p_off >= cut {
                        p_node = NONE;
                        node.len - 1 - p_off
                    } else {
                        NONE
                    };
                    self.fx_rev.push((node.sym, rem, pro));
                    self.nodes[nid as usize].len = cut;
                    rem = 0;
                }
            }
            if count_stats {
                st.fx_runs += self.fx_rev.len();
            }

            // Pure x-suffix check (fixed points at right end of F_x)
            let mut start_fx = 0usize;
            if !self.fx_rev.is_empty() && self.fx_rev[0].0 == x {
                let (_, len0, pro0) = self.fx_rev[0];
                if pro0 != NONE {
                    root_sym = x;
                    root_mult = if head == NONE && self.fx_rev.len() == 1 {
                        len0
                    } else {
                        1
                    };
                    break;
                }
                start_fx = 1;
            }
            if start_fx >= self.fx_rev.len() {
                continue;
            }

            // 2. Sequential right-to-left two-pointer queue scan inside F_x
            self.exit_rev.clear();
            let mut q_idx = 0usize;
            let mut q_off = 0u32;
            for idx in start_fx..self.fx_rev.len() {
                let (sym, length, pro) = self.fx_rev[idx];
                if sym != x {
                    self.exit_rev.push((sym, 1, length, pro, 0));
                } else {
                    let mut rl = length;
                    let mut cur_in = 0u32;
                    while rl > 0 {
                        let (ey, ek, elen, epro, epsteps) = self.exit_rev[q_idx];
                        let avail = elen - q_off;
                        let take = rl.min(avail);
                        let (new_pro, new_psteps) = if pro != NONE && pro >= cur_in && pro < cur_in + take {
                            (pro - cur_in, 0)
                        } else if epro != NONE && epro >= q_off && epro < q_off + take {
                            (epro - q_off, epsteps + 1)
                        } else {
                            (NONE, 0)
                        };
                        self.exit_rev.push((ey, ek + 1, take, new_pro, new_psteps));
                        cur_in += take;
                        rl -= take;
                        q_off += take;
                        if q_off == elen {
                            q_idx += 1;
                            q_off = 0;
                        }
                    }
                }
            }

            // 3. Allocate new symbols for suffix exit_rev[q_idx..] and build exit_lr (left-to-right!)
            let x_len = self.prods[x as usize].len;
            self.extra_k.clear();
            self.exit_lr.clear();
            let mut idx_e = self.exit_rev.len();
            while idx_e > q_idx {
                idx_e -= 1;
                let (ey, ek, elen, epro, epsteps) = self.exit_rev[idx_e];
                let start = if idx_e == q_idx { q_off } else { 0 };
                if elen <= start {
                    continue;
                }
                let sub_len = elen - start;
                let p_left = if epro != NONE && epro >= start && epro < elen {
                    sub_len - 1 - (epro - start)
                } else {
                    NONE
                };

                // O(1) symbol lookup/creation without HashMap!
                let sid = if ek == 1 {
                    if self.syms[ey as usize].stamp == step_id {
                        self.syms[ey as usize].k1_sid
                    } else {
                        let new_sid = self.syms.len() as u32;
                        let nx = self.syms[ey as usize].next_sym;
                        self.syms[ey as usize].stamp = step_id;
                        self.syms[ey as usize].k1_sid = new_sid;
                        self.syms[ey as usize].next_sym = new_sid;
                        if nx != NONE {
                            self.syms[nx as usize].prev_sym = new_sid;
                        } else {
                            sym_tail = new_sid;
                        }
                        self.syms.push(SymMeta {
                            stack_top: NONE,
                            freq: 0,
                            prev_sym: ey,
                            next_sym: nx,
                            stamp: 0,
                            k1_sid: NONE,
                        });
                        let y_len = self.prods[ey as usize].len;
                        self.prods.push(Prod {
                            y: ey,
                            x,
                            k: 1,
                            len: y_len + x_len,
                        });
                        new_sid
                    }
                } else {
                    // Rare ek >= 2: linear search tiny extra_k
                    let mut found = NONE;
                    for &(e_y, e_k, e_sid) in &self.extra_k {
                        if e_y == ey && e_k == ek {
                            found = e_sid;
                            break;
                        }
                    }
                    if found != NONE {
                        found
                    } else {
                        let new_sid = self.syms.len() as u32;
                        self.extra_k.push((ey, ek, new_sid));
                        // Insert in exact increasing-k position after ey:
                        // Walk forward from ey past any already-inserted symbols of (ey, x, k' < ek)
                        let mut anchor = ey;
                        loop {
                            let nx = self.syms[anchor as usize].next_sym;
                            if nx == NONE || (nx as usize) < 256 {
                                break;
                            }
                            let pr = self.prods[nx as usize];
                            if pr.y == ey && pr.x == x && pr.k < ek {
                                anchor = nx;
                            } else {
                                break;
                            }
                        }
                        let nx = self.syms[anchor as usize].next_sym;
                        self.syms[anchor as usize].next_sym = new_sid;
                        if nx != NONE {
                            self.syms[nx as usize].prev_sym = new_sid;
                        } else {
                            sym_tail = new_sid;
                        }
                        self.syms.push(SymMeta {
                            stack_top: NONE,
                            freq: 0,
                            prev_sym: anchor,
                            next_sym: nx,
                            stamp: 0,
                            k1_sid: NONE,
                        });
                        let y_len = self.prods[ey as usize].len;
                        self.prods.push(Prod {
                            y: ey,
                            x,
                            k: ek,
                            len: y_len + ek * x_len,
                        });
                        new_sid
                    }
                };
                self.syms[ey as usize].freq -= sub_len;
                self.syms[sid as usize].freq += sub_len;
                self.exit_lr.push((sid, sub_len, p_left, epsteps));
            }

            // 4. Collect ux_nodes from sym_stack[x] (right-to-left) and iterate in reverse (left-to-right!)
            self.ux_buf.clear();
            let mut cur_u = self.syms[x as usize].stack_top;
            while cur_u != NONE {
                self.ux_buf.push(cur_u);
                cur_u = self.nodes[cur_u as usize].sym_prev;
            }
            if count_stats {
                st.ux_runs += self.ux_buf.len();
            }

            let mut e_idx = 0usize;
            let mut e_off = 0u32;
            let mut u_i = self.ux_buf.len();
            while u_i > 0 {
                u_i -= 1;
                let uid = self.ux_buf[u_i];
                let unode = self.nodes[uid as usize];
                let mut rem = unode.len;
                let (sid0, elen0, epo0, epsteps0) = self.exit_lr[e_idx];

                // Fast path (95% of all ux_runs!): in-place symbol overwrite, zero prev/next splice!
                if elen0 - e_off >= rem {
                    if count_stats {
                        st.ux_inplace += 1;
                        st.spliced_runs += 1;
                    }
                    let sym_prev = self.syms[sid0 as usize].stack_top;
                    self.syms[sid0 as usize].stack_top = uid;
                    self.nodes[uid as usize].sym = sid0;
                    self.nodes[uid as usize].sym_prev = sym_prev;
                    if uid != p_node && epo0 != NONE && epo0 >= e_off && epo0 < e_off + rem {
                        p_node = uid;
                        p_off = epo0 - e_off;
                        p_byte_shift += ((epsteps0 + 1) as usize) * (x_len as usize);
                    }
                    e_off += rem;
                    if e_off == elen0 {
                        e_idx += 1;
                        e_off = 0;
                    }
                    continue;
                }

                // Multi-run splice (~4-5% of ux_runs): allocate replacement chain in flat slab
                let u_has_p = uid == p_node;
                let u_po = p_off;
                if u_has_p {
                    p_node = NONE;
                }
                let mut u_cur = 0u32;
                let mut chain_head = NONE;
                let mut chain_tail = NONE;
                while rem > 0 {
                    let (sid, elen, epo, epsteps) = self.exit_lr[e_idx];
                    let avail = elen - e_off;
                    let take = rem.min(avail);
                    let rnid = self.nodes.len() as u32;
                    let sym_prev = self.syms[sid as usize].stack_top;
                    self.syms[sid as usize].stack_top = rnid;
                    self.nodes.push(RunNode {
                        sym: sid,
                        len: take,
                        prev: chain_tail,
                        next: NONE,
                        sym_prev,
                    });
                    if chain_tail != NONE {
                        self.nodes[chain_tail as usize].next = rnid;
                    } else {
                        chain_head = rnid;
                    }
                    chain_tail = rnid;
                    if u_has_p && u_po >= u_cur && u_po < u_cur + take {
                        p_node = rnid;
                        p_off = u_po - u_cur;
                    } else if epo != NONE && epo >= e_off && epo < e_off + take {
                        p_node = rnid;
                        p_off = epo - e_off;
                        p_byte_shift += ((epsteps + 1) as usize) * (x_len as usize);
                    }
                    if count_stats {
                        st.spliced_runs += 1;
                    }
                    u_cur += take;
                    rem -= take;
                    e_off += take;
                    if e_off == elen {
                        e_idx += 1;
                        e_off = 0;
                    }
                }
                let pr = unode.prev;
                let nx = unode.next;
                self.nodes[chain_head as usize].prev = pr;
                self.nodes[chain_tail as usize].next = nx;
                if pr != NONE {
                    self.nodes[pr as usize].next = chain_head;
                } else {
                    head = chain_head;
                }
                if nx != NONE {
                    self.nodes[nx as usize].prev = chain_tail;
                } else {
                    tail = chain_tail;
                }
            }
        }

        debug_assert_ne!(root_sym, NONE);
        let tot_syms = self.syms.len();
        if count_stats {
            st.tot_syms = tot_syms;
            st.max_nodes = self.nodes.len();
            st.ws_phase1_bytes = self.nodes.len() * std::mem::size_of::<RunNode>()
                + tot_syms * std::mem::size_of::<SymMeta>()
                + tot_syms * std::mem::size_of::<Prod>();
        }

        // Phase 2: First-occurrence-memcpy SLP expansion directly into `self.buf`
        let l_root = self.prods[root_sym as usize].len as usize;
        self.buf.resize(l_root, 0);
        self.first_pos.clear();
        self.first_pos.resize(tot_syms, NONE);
        self.stack.clear();
        self.stack.push((root_sym, 1));
        let mut pos = 0usize;

        while let Some((s, mult)) = self.stack.pop() {
            if s < 256 {
                let b = s as u8;
                let m = mult as usize;
                self.buf[pos..pos + m].fill(b);
                pos += m;
            } else {
                let fp = self.first_pos[s as usize];
                if fp != NONE {
                    let src = fp as usize;
                    let slen = self.prods[s as usize].len as usize;
                    for _ in 0..mult {
                        self.buf.copy_within(src..src + slen, pos);
                        pos += slen;
                    }
                    if count_stats {
                        st.emit_cpy += 1;
                    }
                } else {
                    self.first_pos[s as usize] = pos as u32;
                    if count_stats {
                        st.emit_exp += 1;
                    }
                    if mult > 1 {
                        self.stack.push((s, mult - 1));
                    }
                    let pr = self.prods[s as usize];
                    self.stack.push((pr.x, pr.k));
                    self.stack.push((pr.y, 1));
                }
            }
        }
        debug_assert_eq!(pos, l_root);

        if count_stats {
            st.ws_phase2_bytes = tot_syms * (std::mem::size_of::<Prod>() + 4) + l_root;
        }

        // Final global ordering with exact rotation (-p_byte_shift mod l_root)
        let shift = (l_root - (p_byte_shift % l_root)) % l_root;
        out.reserve(n);
        for _ in 0..root_mult {
            out.extend_from_slice(&self.buf[shift..]);
            out.extend_from_slice(&self.buf[..shift]);
        }
        st
    }
}

struct BlockSpec {
    name: &'static str,
    n: usize,
    p: u32,
    sha256: &'static str,
}

const BLOCKS: &[BlockSpec] = &[
    BlockSpec {
        name: "block-0.L",
        n: 899_931,
        p: 264_045,
        sha256: "db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67",
    },
    BlockSpec {
        name: "block-2.L",
        n: 900_000,
        p: 320_807,
        sha256: "8b47073232d1d9cff7a0688399cfd405211045e15228706f213780e083fc4174",
    },
    BlockSpec {
        name: "block-85.L",
        n: 899_935,
        p: 325_088,
        sha256: "5def63ec66c914e1986a97f8206a8debd09f6e936f0bc8d8f4cab8544ff64fe8",
    },
    BlockSpec {
        name: "block-143.L",
        n: 899_869,
        p: 802_868,
        sha256: "eb443421d4d7b459a1db02a00c068f4c2f1e3b7e70f85674228ea2cba30ceba2",
    },
    BlockSpec {
        name: "block-210.L",
        n: 899_959,
        p: 791_429,
        sha256: "dd4d0c7d03dd6bc7ec8752ad03b2fd5607222c54bebab8d81ae98a2c277f3dcd",
    },
    BlockSpec {
        name: "block-207.L",
        n: 899_998,
        p: 205_180,
        sha256: "7343ef019b44f0e838e68abc09e36d9806a0f45249cf2286702a2e482e555099",
    },
];

fn median(vals: &mut [f64]) -> f64 {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = vals.len();
    if m % 2 == 1 {
        vals[m / 2]
    } else {
        0.5 * (vals[m / 2 - 1] + vals[m / 2])
    }
}

fn main() {
    let k = Kernels::detect();
    let dir = Path::new("/tmp/mibz2-blocks");

    println!("=== 1. SHA-256 VERIFICATION, BYTE-FOR-BYTE PARITY, & EXACT Q vs m_16 ===");
    let mut loaded: Vec<(String, Runs, usize, u32, usize, usize)> = Vec::new();
    let mut open_k = OpenCells::default();
    let mut laz_k = LazardMaxKernel::default();
    let mut out_open = Vec::new();
    let mut out_laz = Vec::new();

    for spec in BLOCKS {
        let data = fs::read(dir.join(spec.name)).expect("read block");
        assert_eq!(data.len(), spec.n, "len mismatch {}", spec.name);
        let got_sha = sha256_hex(&data);
        assert_eq!(got_sha, spec.sha256, "SHA-256 mismatch {}", spec.name);

        let r = build_runs_from_column(&data, spec.p, k);
        let (r_max, m16) = compute_open_m16(r.records(), spec.p);

        assert!(open_k.invert_runs(&r, spec.n, spec.p, &mut out_open, k));
        let st = laz_k.invert_runs_counted(r.records(), spec.n, spec.p, &mut out_laz, true);
        assert_eq!(out_laz, out_open, "BYTE-FOR-BYTE MISMATCH on {}!", spec.name);

        // Irregular accesses Q:
        //   Phase 1: fx_nonseq (non-contiguous tail hops) + ux_runs (arena writes) + fx_runs (sym_stack pop lookup)
        //   Phase 2: emit_exp (prod read) + emit_cpy (buf copy read)
        let q_p1 = st.fx_nonseq + st.ux_runs + st.fx_runs;
        let q_p2 = st.emit_exp + st.emit_cpy;
        let q_tot = q_p1 + q_p2;
        println!(
            "{:11} | r={:6} m_16={:6} | Lazard: steps={:5} fx_nonseq={:5} ux={:6} ({:.1}% inpl) tot_syms={:6} | Q_p1={:6} Q_p2={:6} Q_tot={:6} ({:.2}x m_16) | WS: P1={:.2}MB P2={:.2}MB | PARITY OK",
            spec.name,
            r_max,
            m16,
            st.steps,
            st.fx_nonseq,
            st.ux_runs,
            100.0 * st.ux_inplace as f64 / st.ux_runs as f64,
            st.tot_syms,
            q_p1,
            q_p2,
            q_tot,
            q_tot as f64 / m16 as f64,
            st.ws_phase1_bytes as f64 / 1e6,
            st.ws_phase2_bytes as f64 / 1e6,
        );
        loaded.push((spec.name.to_string(), r, spec.n, spec.p, r_max, m16));
    }

    // =========================================================================
    // 2. SINGLE-THREAD (1T) A/A CONTROL & A/B BENCHMARK (interleaved rounds)
    // =========================================================================
    let warmup = 3;
    let rounds = 15;
    println!("\n=== 2. SINGLE-THREAD (1T) A/A CONTROL (OpenCells vs OpenCells) & A/B (OpenCells vs Lazard-MAX) ===");
    println!("{:11} | {:>10} {:>10} {:>8} | {:>10} {:>10} {:>8}", "Block", "AA_Open1", "AA_Open2", "AA_rat", "AB_Open", "AB_Lazard", "Laz/Open");

    let mut open_a = OpenCells::default();
    let mut open_b = OpenCells::default();
    let mut laz_b = LazardMaxKernel::default();
    let mut buf_a = Vec::with_capacity(900_000);
    let mut buf_b = Vec::with_capacity(900_000);

    for (name, r, n, p, _, _) in &loaded {
        for _ in 0..warmup {
            open_a.invert_runs(r, *n, *p, &mut buf_a, k);
            open_b.invert_runs(r, *n, *p, &mut buf_b, k);
            laz_b.invert_runs_counted(r.records(), *n, *p, &mut buf_b, false);
        }

        // A/A control: OpenCells vs OpenCells
        let mut aa1 = Vec::with_capacity(rounds);
        let mut aa2 = Vec::with_capacity(rounds);
        for rd in 0..rounds {
            if rd % 2 == 0 {
                let t0 = Instant::now();
                open_a.invert_runs(r, *n, *p, &mut buf_a, k);
                aa1.push(t0.elapsed().as_secs_f64() * 1e3);
                let t1 = Instant::now();
                open_b.invert_runs(r, *n, *p, &mut buf_b, k);
                aa2.push(t1.elapsed().as_secs_f64() * 1e3);
            } else {
                let t1 = Instant::now();
                open_b.invert_runs(r, *n, *p, &mut buf_b, k);
                aa2.push(t1.elapsed().as_secs_f64() * 1e3);
                let t0 = Instant::now();
                open_a.invert_runs(r, *n, *p, &mut buf_a, k);
                aa1.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            assert_eq!(buf_a.len(), *n);
            assert_eq!(buf_b.len(), *n);
        }

        // A/B: OpenCells vs Lazard-MAX
        let mut ab_o = Vec::with_capacity(rounds);
        let mut ab_l = Vec::with_capacity(rounds);
        for rd in 0..rounds {
            if rd % 2 == 0 {
                let t0 = Instant::now();
                open_a.invert_runs(r, *n, *p, &mut buf_a, k);
                ab_o.push(t0.elapsed().as_secs_f64() * 1e3);
                let t1 = Instant::now();
                laz_b.invert_runs_counted(r.records(), *n, *p, &mut buf_b, false);
                ab_l.push(t1.elapsed().as_secs_f64() * 1e3);
            } else {
                let t1 = Instant::now();
                laz_b.invert_runs_counted(r.records(), *n, *p, &mut buf_b, false);
                ab_l.push(t1.elapsed().as_secs_f64() * 1e3);
                let t0 = Instant::now();
                open_a.invert_runs(r, *n, *p, &mut buf_a, k);
                ab_o.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            assert_eq!(buf_a.len(), *n);
            assert_eq!(buf_b.len(), *n);
        }

        let m_aa1 = median(&mut aa1);
        let m_aa2 = median(&mut aa2);
        let m_abo = median(&mut ab_o);
        let m_abl = median(&mut ab_l);
        println!(
            "{:11} | {:8.3}ms {:8.3}ms {:7.3}x | {:8.3}ms {:8.3}ms {:7.3}x",
            name,
            m_aa1,
            m_aa2,
            m_aa2 / m_aa1,
            m_abo,
            m_abl,
            m_abl / m_abo,
        );
    }

    // =========================================================================
    // 3. 4-WORKER CONCURRENT (4W) A/A CONTROL & A/B BENCHMARK
    //    Each of 4 workers decodes the block simultaneously (block-parallel regime)
    // =========================================================================
    println!("\n=== 3. FOUR CONCURRENT WORKERS (4W) A/A & A/B (wall ms per 4-block batch, median of 11 rounds) ===");
    println!("{:11} | {:>10} {:>10} {:>8} | {:>10} {:>10} {:>8}", "Block", "AA_Open1", "AA_Open2", "AA_rat", "AB_Open", "AB_Lazard", "Laz/Open");

    let rounds_4w = 11;
    let run_4w_open = |r: &Arc<Runs>, n: usize, p: u32| -> f64 {
        let bar = Arc::new(Barrier::new(4));
        let mut handles = Vec::with_capacity(4);
        for _ in 0..4 {
            let r_c = Arc::clone(r);
            let b_c = Arc::clone(&bar);
            handles.push(thread::spawn(move || {
                let mut ok = OpenCells::default();
                let mut out = Vec::with_capacity(n);
                // Warm kernel buffers once before timed barrier
                ok.invert_runs(&r_c, n, p, &mut out, k);
                b_c.wait();
                let t0 = Instant::now();
                ok.invert_runs(&r_c, n, p, &mut out, k);
                let dt = t0.elapsed().as_secs_f64() * 1e3;
                assert_eq!(out.len(), n);
                dt
            }));
        }
        handles.into_iter().map(|h| h.join().unwrap()).fold(0.0f64, f64::max)
    };

    let run_4w_lazard = |r: &Arc<Runs>, n: usize, p: u32| -> f64 {
        let bar = Arc::new(Barrier::new(4));
        let mut handles = Vec::with_capacity(4);
        for _ in 0..4 {
            let r_c = Arc::clone(r);
            let b_c = Arc::clone(&bar);
            handles.push(thread::spawn(move || {
                let mut lk = LazardMaxKernel::default();
                let mut out = Vec::with_capacity(n);
                lk.invert_runs_counted(r_c.records(), n, p, &mut out, false);
                b_c.wait();
                let t0 = Instant::now();
                lk.invert_runs_counted(r_c.records(), n, p, &mut out, false);
                let dt = t0.elapsed().as_secs_f64() * 1e3;
                assert_eq!(out.len(), n);
                dt
            }));
        }
        handles.into_iter().map(|h| h.join().unwrap()).fold(0.0f64, f64::max)
    };

    for (name, r, n, p, _, _) in loaded {
        let r_arc = Arc::new(r);
        let mut aa1 = Vec::with_capacity(rounds_4w);
        let mut aa2 = Vec::with_capacity(rounds_4w);
        let mut ab_o = Vec::with_capacity(rounds_4w);
        let mut ab_l = Vec::with_capacity(rounds_4w);
        for rd in 0..rounds_4w {
            if rd % 2 == 0 {
                aa1.push(run_4w_open(&r_arc, n, p));
                aa2.push(run_4w_open(&r_arc, n, p));
                ab_o.push(run_4w_open(&r_arc, n, p));
                ab_l.push(run_4w_lazard(&r_arc, n, p));
            } else {
                aa2.push(run_4w_open(&r_arc, n, p));
                aa1.push(run_4w_open(&r_arc, n, p));
                ab_l.push(run_4w_lazard(&r_arc, n, p));
                ab_o.push(run_4w_open(&r_arc, n, p));
            }
        }
        let m_aa1 = median(&mut aa1);
        let m_aa2 = median(&mut aa2);
        let m_abo = median(&mut ab_o);
        let m_abl = median(&mut ab_l);
        println!(
            "{:11} | {:8.3}ms {:8.3}ms {:7.3}x | {:8.3}ms {:8.3}ms {:7.3}x",
            name,
            m_aa1,
            m_aa2,
            m_aa2 / m_aa1,
            m_abo,
            m_abl,
            m_abl / m_abo,
        );
    }
}
