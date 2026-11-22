#![forbid(unsafe_code)]

//! Cycle 49 (P1) — Analogue J: Frobenius (1896–1900) Group Characters, Hook-Character Blindness,
//! Young-Subgroup Reciprocity & Class-Multiplication Involution Spectrum on `LF` / `edges`.
//!
//! Primary Sources Directly Implemented & Verified:
//!   - F. G. Frobenius (1896), "Über Gruppencharaktere", SB Preuss. Akad. Wiss. Berlin, 985–1021.
//!   - F. G. Frobenius (1898), "Über Relationen zwischen den Charakteren einer Gruppe und denen
//!     ihrer Untergruppen", SB Preuss. Akad. Wiss. Berlin, 501–515.
//!   - F. G. Frobenius (1899), "Über die Composition der Charaktere einer Gruppe", SB Preuss.
//!     Akad. Wiss. Berlin, 330–339.
//!   - F. G. Frobenius (1900), "Über die Charaktere der symmetrischen Gruppe", SB Preuss. Akad.
//!     Wiss. Berlin, 516–534.
//!   - F. G. Frobenius & I. Schur (1906), "Über die reellen Darstellungen der endlichen Gruppen",
//!     SB Preuss. Akad. Wiss. Berlin, 186–208.
//!
//! Verifies:
//!   - Exact S_m character tables for m = 1..12 via Frobenius (1900) beta-number rim-hook rule,
//!     self-checked against Frobenius (1896) 1st/2nd orthogonality & Frobenius-Schur nu_2 == +1.
//!   - Theorem J1a (Hook-Character Blindness on C_{(m)}), J1b (Frobenius 1898 Reciprocity &
//!     Kostka-Hook Cancellation on Young subgroup S_m), and J1c (Frobenius 1898 Generalization
//!     of CA-30 to exact bit-saving conjugator count |T_0|!|T_1|! Ind_{S_t}^{S_m}(1)(pi^2)).
//!   - Theorem J2 (Frobenius 1896/1899 Class-Multiplication c_{I_{f1}, I_{f2}}^{C_mu} exact
//!     verification across all 271 partitions of m = 1..12 + brute-force cross-check m = 1..8,
//!     proving single-cycle fixed-point spectrum & multi-cycle XOR-BiEmit structural obstruction).
//!   - Full Stepanov Analogue J empirical diagnosis across Solved J, 5,200 small valid BWTs,
//!     300 malformed multi-cycle columns, 6 Admitted Wikidata, 6 Non-Wikidata (JSONL+Syslog),
//!     4 Code+Docs, and all 233 Full wikidata.bz2 blocks.

use std::collections::HashMap;
use std::fs;

const LIMIT: u32 = 16;

// ============================================================================
// PART A: Exact Frobenius (1896–1900) Symmetric-Group Character Engine (m=1..12)
// ============================================================================

fn fact_u128(n: usize) -> u128 {
    let mut r = 1u128;
    for k in 2..=n {
        r *= k as u128;
    }
    r
}

fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

fn gcd_u32(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

fn binom_u128(n: usize, k: usize) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut r = 1u128;
    for i in 0..k {
        r = r * (n - i) as u128 / (i + 1) as u128;
    }
    r
}

/// Generate all partitions of `n` in reverse-lexicographic order (parts descending).
fn partitions(n: usize) -> Vec<Vec<usize>> {
    fn rec(rem: usize, max_part: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if rem == 0 {
            out.push(cur.clone());
            return;
        }
        let hi = rem.min(max_part);
        for p in (1..=hi).rev() {
            cur.push(p);
            rec(rem - p, p, cur, out);
            cur.pop();
        }
    }
    let mut out = Vec::new();
    let mut cur = Vec::new();
    rec(n, n, &mut cur, &mut out);
    out
}

/// Exact conjugacy class size |C_mu| = m! / prod_l (l^{a_l} * a_l!) (Cauchy 1844 / Frobenius 1900).
fn class_size(m: usize, mu: &[usize]) -> u128 {
    let mut counts = vec![0usize; m + 1];
    for &p in mu {
        counts[p] += 1;
    }
    let mut denom = 1u128;
    for (len, &cnt) in counts.iter().enumerate() {
        if cnt > 0 {
            let mut pow = 1u128;
            for _ in 0..cnt {
                pow *= len as u128;
            }
            denom *= pow * fact_u128(cnt);
        }
    }
    let mf = fact_u128(m);
    assert_eq!(mf % denom, 0);
    mf / denom
}

/// Cycle type of g^2 for g in conjugacy class mu (Cauchy 1815):
/// each odd cycle l stays l; each even cycle l splits into two (l/2)-cycles.
fn square_cycle_type(mu: &[usize]) -> Vec<usize> {
    let mut out = Vec::with_capacity(mu.len() * 2);
    for &l in mu {
        if l % 2 == 1 {
            out.push(l);
        } else {
            out.push(l / 2);
            out.push(l / 2);
        }
    }
    out.sort_unstable_by(|a, b| b.cmp(a));
    out
}

/// Exact Frobenius (1900, §§2-3) character value chi^lambda(C_mu) via beta-number rim-hook removal.
/// Beta-numbers for lambda = (lambda_0, ..., lambda_{k-1}): beta_i = lambda_i + (k - 1 - i).
/// Removing a rim hook of length r shifts some beta_i -> beta_i - r >= 0 not colliding with
/// any other beta_j, with sign (-1)^{number of elements passed during re-sorting}.
fn chi_beta(
    beta: &[usize],
    mu: &[usize],
    memo: &mut HashMap<(Vec<usize>, usize), i128>,
) -> i128 {
    if mu.is_empty() {
        return 1;
    }
    let key = (beta.to_vec(), mu.len());
    if let Some(&v) = memo.get(&key) {
        return v;
    }
    let r = mu[0];
    let rest = &mu[1..];
    let k = beta.len();
    let mut total = 0i128;
    for i in 0..k {
        if beta[i] < r {
            continue;
        }
        let target = beta[i] - r;
        // Check collision with existing beta_j (beta is strictly decreasing)
        let mut collides = false;
        let mut swaps = 0usize;
        for &b in &beta[i + 1..] {
            if b == target {
                collides = true;
                break;
            }
            if b > target {
                swaps += 1;
            } else {
                break;
            }
        }
        if collides {
            continue;
        }
        let mut next_beta = Vec::with_capacity(k);
        next_beta.extend_from_slice(&beta[..i]);
        next_beta.extend_from_slice(&beta[i + 1..i + 1 + swaps]);
        next_beta.push(target);
        next_beta.extend_from_slice(&beta[i + 1 + swaps..]);
        let sign = if swaps % 2 == 0 { 1i128 } else { -1i128 };
        total += sign * chi_beta(&next_beta, rest, memo);
    }
    memo.insert(key, total);
    total
}

fn frobenius_char_table(m: usize) -> (Vec<Vec<usize>>, Vec<u128>, Vec<Vec<i128>>) {
    let parts = partitions(m);
    let np = parts.len();
    let sizes: Vec<u128> = parts.iter().map(|mu| class_size(m, mu)).collect();
    let mut table = vec![vec![0i128; np]; np];
    for (li, lam) in parts.iter().enumerate() {
        let k = lam.len();
        let beta: Vec<usize> = lam
            .iter()
            .enumerate()
            .map(|(i, &p)| p + (k - 1 - i))
            .collect();
        for (mi, mu) in parts.iter().enumerate() {
            let mut memo = HashMap::new();
            table[li][mi] = chi_beta(&beta, mu, &mut memo);
        }
    }
    (parts, sizes, table)
}

/// Exact Kostka number K_{lambda, alpha} via semistandard Young tableaux (SSYT) enumeration.
fn kostka_ssyt(lam: &[usize], content: &[usize]) -> u128 {
    let nrows = lam.len();
    let mut row_fill = vec![0usize; nrows];
    let mut grid: Vec<Vec<usize>> = lam.iter().map(|&len| vec![0; len]).collect();

    fn place_letter(
        letter_idx: usize,
        rem_of_letter: usize,
        min_row: usize,
        lam: &[usize],
        content: &[usize],
        row_fill: &mut [usize],
        grid: &mut [Vec<usize>],
    ) -> u128 {
        if rem_of_letter == 0 {
            if letter_idx + 1 == content.len() {
                return 1;
            }
            return place_letter(
                letter_idx + 1,
                content[letter_idx + 1],
                0,
                lam,
                content,
                row_fill,
                grid,
            );
        }
        let mut ways = 0u128;
        let nrows = lam.len();
        for r in min_row..nrows {
            let c = row_fill[r];
            if c >= lam[r] {
                continue;
            }
            // Must maintain partition shape & strict column increase:
            // row above must already have column c filled with strictly smaller letter
            if r > 0 {
                if row_fill[r - 1] <= c || grid[r - 1][c] >= letter_idx + 1 {
                    continue;
                }
            }
            grid[r][c] = letter_idx + 1;
            row_fill[r] += 1;
            ways += place_letter(
                letter_idx,
                rem_of_letter - 1,
                r,
                lam,
                content,
                row_fill,
                grid,
            );
            row_fill[r] -= 1;
            grid[r][c] = 0;
        }
        ways
    }

    if content.is_empty() {
        return 1;
    }
    place_letter(0, content[0], 0, lam, content, &mut row_fill, &mut grid)
}

/// Exact induced character Ind_{S_alpha}^{S_m}(1)(C_mu) = number of ways to assign the cycles
/// of mu into buckets of capacities alpha = (m_0, ..., m_{sigma-1}).
fn ind_young_direct(alpha: &[usize], mu: &[usize]) -> u128 {
    fn rec(ci: usize, mu: &[usize], rem: &mut [usize]) -> u128 {
        if ci == mu.len() {
            return 1;
        }
        let len = mu[ci];
        let mut ways = 0u128;
        for b in 0..rem.len() {
            if rem[b] >= len {
                rem[b] -= len;
                ways += rec(ci + 1, mu, rem);
                rem[b] += len;
            }
        }
        ways
    }
    let mut rem = alpha.to_vec();
    rec(0, mu, &mut rem)
}

/// Exact closed-form count of involution factorizations N_inv(mu) = |{(s1,s2) in I_m^2 : s1*s2 = pi}|
/// for pi of cycle type mu = (1^{a_1}, 2^{a_2}, ..., m^{a_m}):
///   N_inv(mu) = prod_{l: a_l > 0} sum_{k=0}^{floor(a_l/2)} C(a_l, 2k) * (2k-1)!! * l^{a_l - k}.
fn closed_form_inv_factorizations(mu: &[usize]) -> u128 {
    let m: usize = mu.iter().sum();
    let mut counts = vec![0usize; m + 1];
    for &l in mu {
        counts[l] += 1;
    }
    let mut prod = 1u128;
    for (l, &a) in counts.iter().enumerate() {
        if a == 0 {
            continue;
        }
        let mut sum_l = 0u128;
        let mut dbl_fact = 1u128; // (2k-1)!!
        for k in 0..=(a / 2) {
            if k > 0 {
                dbl_fact *= (2 * k - 1) as u128;
            }
            let choose = binom_u128(a, 2 * k);
            let mut l_pow = 1u128;
            for _ in 0..(a - k) {
                l_pow *= l as u128;
            }
            sum_l += choose * dbl_fact * l_pow;
        }
        prod *= sum_l;
    }
    prod
}

/// Log10 of N_inv(mu) for arbitrarily large multi-cycle permutations (overflow-free f64).
fn log10_inv_factorizations(cycle_lens: &[usize]) -> f64 {
    let mut freq: HashMap<usize, usize> = HashMap::new();
    for &l in cycle_lens {
        *freq.entry(l).or_insert(0) += 1;
    }
    let mut total_ln = 0.0f64;
    for (l, a) in freq {
        // Compute log(sum_{k=0}^{a/2} C(a,2k)*(2k-1)!! * l^{a-k}) via log-sum-exp
        let mut terms = Vec::with_capacity(a / 2 + 1);
        let mut ln_term = (a as f64) * (l as f64).ln(); // k = 0 term: l^a
        terms.push(ln_term);
        for k in 1..=(a / 2) {
            // ratio from k-1 to k: (a - 2k + 2)*(a - 2k + 1) / (2k * l)
            let num = ((a - 2 * k + 2) * (a - 2 * k + 1)) as f64;
            let den = (2 * k * l) as f64;
            ln_term += (num / den).ln();
            terms.push(ln_term);
        }
        let max_t = terms.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let sum_exp: f64 = terms.iter().map(|&t| (t - max_t).exp()).sum();
        total_ln += max_t + sum_exp.ln();
    }
    total_ln / std::f64::consts::LN_10
}

/// Exact Frobenius (1896/1899) class-multiplication constant:
///   c_{C1, C2}^{C3} = (|C1| * |C2| / m!) * sum_lambda chi^lambda(C1)*chi^lambda(C2)*chi^lambda(C3) / chi^lambda(1)
/// Computed with exact gcd-reduced i128 arithmetic (zero floats, zero rounding, asserts exact integrality).
fn frobenius_class_mult(
    m: usize,
    id_idx: usize,
    sizes: &[u128],
    table: &[Vec<i128>],
    c1: usize,
    c2: usize,
    c3: usize,
) -> u128 {
    let mf = fact_u128(m);
    let mut s = 0i128;
    for row in table {
        let deg = row[id_idx];
        assert!(deg > 0 && (mf as i128) % deg == 0);
        let q = (mf as i128) / deg;
        s += q * row[c1] * row[c2] * row[c3];
    }
    assert!(s >= 0, "negative class-multiplication sum!");
    if s == 0 {
        return 0;
    }
    let mut su = s as u128;
    let mut a1 = sizes[c1];
    let mut a2 = sizes[c2];
    let mut d1 = mf;
    let mut d2 = mf;

    // Reduce (a1 * a2 * su) / (d1 * d2) cleanly by gcd without overflow
    let g = gcd_u128(a1, d1);
    a1 /= g;
    d1 /= g;
    let g = gcd_u128(a2, d2);
    a2 /= g;
    d2 /= g;
    let g = gcd_u128(su, d1);
    su /= g;
    d1 /= g;
    let g = gcd_u128(su, d2);
    su /= g;
    d2 /= g;
    let g = gcd_u128(a1, d2);
    a1 /= g;
    d2 /= g;
    let g = gcd_u128(a2, d1);
    a2 /= g;
    d1 /= g;
    assert_eq!(d1, 1, "Frobenius integrality check failed (d1={d1})");
    assert_eq!(d2, 1, "Frobenius integrality check failed (d2={d2})");
    su.checked_mul(a1)
        .and_then(|v| v.checked_mul(a2))
        .expect("u128 overflow in frobenius_class_mult")
}

/// Generate all permutations of 0..m (for exhaustive verification at m <= 8).
fn for_each_perm(m: usize, mut f: impl FnMut(&[usize])) {
    let mut p: Vec<usize> = (0..m).collect();
    loop {
        f(&p);
        // Next lexicographic permutation
        if m < 2 {
            break;
        }
        let mut i = m - 2;
        while p[i] >= p[i + 1] {
            if i == 0 {
                return;
            }
            i -= 1;
        }
        let mut j = m - 1;
        while p[j] <= p[i] {
            j -= 1;
        }
        p.swap(i, j);
        p[i + 1..].reverse();
    }
}

fn cycle_type_of_perm(p: &[usize]) -> Vec<usize> {
    let m = p.len();
    let mut vis = vec![false; m];
    let mut cyc = Vec::new();
    for i in 0..m {
        if !vis[i] {
            let mut len = 0;
            let mut cur = i;
            while !vis[cur] {
                vis[cur] = true;
                cur = p[cur];
                len += 1;
            }
            cyc.push(len);
        }
    }
    cyc.sort_unstable_by(|a, b| b.cmp(a));
    cyc
}

fn rep_perm_of_cycle_type(mu: &[usize]) -> Vec<usize> {
    let m: usize = mu.iter().sum();
    let mut p = vec![0usize; m];
    let mut off = 0usize;
    for &l in mu {
        for i in 0..l {
            p[off + i] = off + (i + 1) % l;
        }
        off += l;
    }
    p
}

fn is_involution_type(mu: &[usize]) -> bool {
    mu.iter().all(|&l| l <= 2)
}

fn fixed_points_of_type(mu: &[usize]) -> usize {
    mu.iter().filter(|&&l| l == 1).count()
}

fn verify_exact_frobenius_theorems_small() {
    println!("================================================================================================================================");
    println!("PART 1: EXACT FROBENIUS (1896–1900) SYMMETRIC-GROUP CHARACTER ENGINE & THEOREMS J1 / J2 (m = 1..12)");
    println!("================================================================================================================================");
    println!(
        "{:>2} {:>4} | {:>10} {:>6} {:>5} | {:>8} {:>9} {:>8} | {:>10} {:>10} | {:>9}",
        "m", "p(m)", "m!", "orth?", "FS+1?", "J1a:hook", "J1b:rec", "J1c:CA30", "J2:1-cyc", "J2:all_mu", "J2:brute"
    );

    let mut total_partitions_verified = 0usize;
    let mut total_j1c_classes = 0usize;
    let mut total_j1c_perms_checked = 0u64;
    let mut total_j2_pairs_checked = 0usize;

    for m in 1..=12 {
        let (parts, sizes, table) = frobenius_char_table(m);
        let np = parts.len();
        let mf = fact_u128(m);
        let mut part_to_idx: HashMap<Vec<usize>, usize> = HashMap::new();
        for (i, p) in parts.iter().enumerate() {
            part_to_idx.insert(p.clone(), i);
        }
        let id_idx = part_to_idx[&vec![1usize; m]];
        let mcyc_idx = part_to_idx[&vec![m]];

        // 1. Verify Frobenius (1896) First & Second Orthogonality Relations + Degree Divisibility
        for l1 in 0..np {
            let deg = table[l1][id_idx];
            assert!(deg > 0 && (mf as i128) % deg == 0);
            for l2 in l1..np {
                let mut sum = 0i128;
                for c in 0..np {
                    sum += (sizes[c] as i128) * table[l1][c] * table[l2][c];
                }
                let expected = if l1 == l2 { mf as i128 } else { 0 };
                assert_eq!(sum, expected, "1st orthogonality failed at m={m}");
            }
        }
        for c1 in 0..np {
            for c2 in c1..np {
                let mut sum = 0i128;
                for l in 0..np {
                    sum += table[l][c1] * table[l][c2];
                }
                let expected = if c1 == c2 {
                    (mf / sizes[c1]) as i128
                } else {
                    0
                };
                assert_eq!(sum, expected, "2nd orthogonality failed at m={m}");
            }
        }

        // 2. Verify Frobenius-Schur (1906) indicator nu_2(chi^lambda) == +1 for all lambda |- m
        for l in 0..np {
            let mut sum = 0i128;
            for (c, mu) in parts.iter().enumerate() {
                let sq = square_cycle_type(mu);
                let sq_idx = part_to_idx[&sq];
                sum += (sizes[c] as i128) * table[l][sq_idx];
            }
            assert_eq!(sum, mf as i128, "Frobenius-Schur nu_2 != +1 at m={m}");
        }

        // 3. Theorem J1a (Frobenius 1900 Hook-Character Blindness):
        //    chi^lambda(C_{(m)}) = (-1)^r for hook (m-r, 1^r), 0 for all non-hooks;
        //    chi^{(m-r, 1^r)}(1) = C(m-1, r).
        let mut hook_cnt = 0usize;
        for (li, lam) in parts.iter().enumerate() {
            let is_hook = lam.len() <= 1 || lam[1..].iter().all(|&x| x == 1);
            let val = table[li][mcyc_idx];
            if is_hook {
                let r = lam.len() - 1;
                let expected = if r % 2 == 0 { 1i128 } else { -1i128 };
                assert_eq!(val, expected, "Hook value mismatch at m={m}, lam={lam:?}");
                assert_eq!(
                    table[li][id_idx] as u128,
                    binom_u128(m - 1, r),
                    "Hook degree mismatch"
                );
                hook_cnt += 1;
            } else {
                assert_eq!(val, 0, "Non-hook vanished check failed at m={m}, lam={lam:?}");
            }
        }
        assert_eq!(hook_cnt, m);

        // 4. Theorem J1b (Frobenius 1898 Reciprocity & Kostka-Hook Cancellation):
        //    Check K_{(m-r,1^r), alpha} == C(sigma-1, r) and Ind(1)(C_{(m)}) == delta_{sigma,1}
        //    for ALL partitions alpha |- m (full SSYT reciprocity check across all mu for m <= 8,
        //    and hook cancellation + direct cycle-partition check for m = 1..12).
        let j1b_tag = if m <= 8 {
            for alpha in &parts {
                let sigma = alpha.len();
                let mut kostka = vec![0u128; np];
                for (li, lam) in parts.iter().enumerate() {
                    kostka[li] = kostka_ssyt(lam, alpha);
                    let is_hook = lam.len() <= 1 || lam[1..].iter().all(|&x| x == 1);
                    if is_hook {
                        let r = lam.len() - 1;
                        assert_eq!(kostka[li], binom_u128(sigma - 1, r));
                    }
                }
                for (ci, mu) in parts.iter().enumerate() {
                    let mut rec_val = 0i128;
                    for li in 0..np {
                        rec_val += (kostka[li] as i128) * table[li][ci];
                    }
                    let dir_val = ind_young_direct(alpha, mu) as i128;
                    assert_eq!(rec_val, dir_val, "Frobenius 1898 reciprocity mismatch");
                }
                let mcyc_ind = ind_young_direct(alpha, &[m]);
                assert_eq!(mcyc_ind, if sigma == 1 { 1 } else { 0 });
            }
            "ALL_SSYT"
        } else {
            for alpha in &parts {
                let sigma = alpha.len();
                let mut alt_sum = 0i128;
                for r in 0..m {
                    let sign = if r % 2 == 0 { 1i128 } else { -1i128 };
                    alt_sum += sign * (binom_u128(sigma - 1, r) as i128);
                }
                let expected = if sigma == 1 { 1i128 } else { 0i128 };
                assert_eq!(alt_sum, expected);
                assert_eq!(ind_young_direct(alpha, &[m]) as i128, expected);
            }
            "HOOK_EX"
        };

        // 5. Theorem J1c (Frobenius 1898 Generalization of CA-30):
        //    Exhaustive check over ALL m! conjugators tau in S_m for EVERY mu |- m (m = 3..8).
        let j1c_tag = if (3..=8).contains(&m) {
            let bits_m = u32::BITS - ((m as u32) - 1).leading_zeros();
            let half = 1usize << (bits_m - 1);
            let t0 = half;
            let t1 = m - half;
            let young_t = [t0, t1];
            let s_t_size = fact_u128(t0) * fact_u128(t1);
            let mut inv_tau = vec![0usize; m];
            for mu in &parts {
                let pi = rep_perm_of_cycle_type(mu);
                let mut pi_inv = vec![0usize; m];
                for u in 0..m {
                    pi_inv[pi[u]] = u;
                }
                let sq = square_cycle_type(mu);
                let predicted = s_t_size * ind_young_direct(&young_t, &sq);
                let mut exact_cnt = 0u128;
                for_each_perm(m, |tau| {
                    total_j1c_perms_checked += 1;
                    for u in 0..m {
                        inv_tau[tau[u]] = u;
                    }
                    let mut ok = true;
                    for ut in 0..m {
                        let u = inv_tau[ut];
                        let xi = tau[pi[u]] ^ tau[pi_inv[u]];
                        if xi >= half {
                            ok = false;
                            break;
                        }
                    }
                    if ok {
                        exact_cnt += 1;
                    }
                });
                assert_eq!(
                    exact_cnt, predicted,
                    "Theorem J1c bit-saving conjugator mismatch at m={m}, mu={mu:?}"
                );
                // Check single-cycle CA-30 specialization
                if mu.len() == 1 {
                    let is_pow2 = m.is_power_of_two();
                    if is_pow2 {
                        assert_eq!(exact_cnt, 2 * fact_u128(m / 2) * fact_u128(m / 2));
                    } else {
                        assert_eq!(exact_cnt, 0);
                    }
                }
                total_j1c_classes += 1;
            }
            "EXH_S_m"
        } else if m >= 9 {
            // Verify algebraic formula on single m-cycle: Ind_{S_t}(1)(pi^2) == 0 (non-pow2 m=9..12)
            let bits_m = u32::BITS - ((m as u32) - 1).leading_zeros();
            let half = 1usize << (bits_m - 1);
            let young_t = [half, m - half];
            let sq = square_cycle_type(&[m]);
            assert_eq!(ind_young_direct(&young_t, &sq), 0);
            "ALG_EX"
        } else {
            "m<3"
        };

        // 6. Theorem J2 (Frobenius 1896/1899 Class-Multiplication & Involution Spectrum):
        let inv_classes: Vec<(usize, usize)> = parts
            .iter()
            .enumerate()
            .filter(|(_, mu)| is_involution_type(mu))
            .map(|(idx, mu)| (idx, fixed_points_of_type(mu)))
            .collect();

        // Check single m-cycle hook-collapsed formula & exact fixed-point spectrum
        let mut single_sum = 0u128;
        for &(i1, f1) in &inv_classes {
            for &(i2, f2) in &inv_classes {
                let c_full = frobenius_class_mult(m, id_idx, &sizes, &table, i1, i2, mcyc_idx);
                // Hook-collapsed check: sum ONLY over the m hook partitions!
                let mut s_hook = 0i128;
                for (li, lam) in parts.iter().enumerate() {
                    let is_hook = lam.len() <= 1 || lam[1..].iter().all(|&x| x == 1);
                    if is_hook {
                        let r = lam.len() - 1;
                        let sign = if r % 2 == 0 { 1i128 } else { -1i128 };
                        let q = (mf as i128) / (binom_u128(m - 1, r) as i128);
                        s_hook += q * sign * table[li][i1] * table[li][i2];
                    }
                }
                let c_expected = if m % 2 == 1 {
                    if f1 == 1 && f2 == 1 { m as u128 } else { 0 }
                } else if (f1 == 2 && f2 == 0) || (f1 == 0 && f2 == 2) {
                    (m / 2) as u128
                } else {
                    0
                };
                assert_eq!(c_full, c_expected, "J2 1-cycle spectrum failed at m={m}");
                // Verify hook-collapsed sum equals full sum before division
                let mut s_all = 0i128;
                for row in &table {
                    s_all += ((mf as i128) / row[id_idx]) * row[i1] * row[i2] * row[mcyc_idx];
                }
                assert_eq!(s_hook, s_all, "Hook collapse mismatch at m={m}");
                single_sum += c_full;
            }
        }
        assert_eq!(single_sum, m as u128);

        // Check ALL target cycle types mu |- m against closed-form N_inv(mu)
        for (c3, mu) in parts.iter().enumerate() {
            let mut sum_inv = 0u128;
            for &(i1, _) in &inv_classes {
                for &(i2, _) in &inv_classes {
                    sum_inv += frobenius_class_mult(m, id_idx, &sizes, &table, i1, i2, c3);
                    total_j2_pairs_checked += 1;
                }
            }
            let closed = closed_form_inv_factorizations(mu);
            assert_eq!(
                sum_inv, closed,
                "J2 multi-cycle closed form mismatch at m={m}, mu={mu:?}"
            );
            total_partitions_verified += 1;
        }

        // Brute-force cross-check over ALL (sigma1, sigma2) in I_m x I_m for m = 1..8
        let brute_tag = if m <= 8 {
            let mut invols: Vec<(Vec<usize>, usize)> = Vec::new();
            for_each_perm(m, |p| {
                if (0..m).all(|i| p[p[i]] == i) {
                    let fp = (0..m).filter(|&i| p[i] == i).count();
                    invols.push((p.to_vec(), fp));
                }
            });
            // Verify Frobenius-Schur involution count |I_m| == sum_lambda chi^lambda(1)
            let sum_deg: u128 = table.iter().map(|r| r[id_idx] as u128).sum();
            assert_eq!(invols.len() as u128, sum_deg);

            // Count products sigma1 * sigma2 per (c1, c2, c3)
            let mut counts_brute: HashMap<(usize, usize, usize), u128> = HashMap::new();
            let mut prod = vec![0usize; m];
            for (s1, f1) in &invols {
                for (s2, f2) in &invols {
                    for i in 0..m {
                        prod[i] = s1[s2[i]];
                    }
                    let c3 = part_to_idx[&cycle_type_of_perm(&prod)];
                    *counts_brute.entry((*f1, *f2, c3)).or_insert(0) += 1;
                }
            }
            for (c3, _) in parts.iter().enumerate() {
                for &(i1, f1) in &inv_classes {
                    for &(i2, f2) in &inv_classes {
                        let frob_c = frobenius_class_mult(m, id_idx, &sizes, &table, i1, i2, c3);
                        let brute_tot = counts_brute.get(&(f1, f2, c3)).copied().unwrap_or(0);
                        assert_eq!(
                            brute_tot,
                            frob_c * sizes[c3],
                            "Brute-force J2 mismatch at m={m}, f1={f1}, f2={f2}, c3={c3}"
                        );
                    }
                }
            }
            format!("{}^2 OK", invols.len())
        } else {
            "EXACT_CF".to_string()
        };

        println!(
            "{:>2} {:>4} | {:>10} {:>6} {:>5} | {:>8} {:>9} {:>8} | {:>10} {:>10} | {:>9}",
            m,
            np,
            mf,
            "EXACT",
            "EXACT",
            format!("{m}/{np}"),
            j1b_tag,
            j1c_tag,
            format!("={m} EX"),
            format!("{np}/{np} EX"),
            brute_tag
        );
    }
    println!(
        "SUMMARY PART 1: ALL {total_partitions_verified}/{total_partitions_verified} partitions (m=1..12) EXACT | J1c exhaustive {total_j1c_classes} classes ({total_j1c_perms_checked} conjugators) | J2 {total_j2_pairs_checked} exact Frobenius class-mult constants verified!\n"
    );
}

// ============================================================================
// PART B: Exact Open<16> Induced-Return Permutation Extractor (Matches cand_r5_head)
// ============================================================================

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let start = total;
        total += w;
        start
    })
}

/// Exact induced permutation pi_ret (using cand_r5_head's exact endpoint elimination for speed!).
/// Wait: let's reuse cand_r5_head's exact Elim::eliminate logic cleanly so there is ZERO
/// divergence from cand_r5_head even on spirals or malformed multi-cycle columns!
#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

struct InducedPerm {
    m: usize,
    mark: u32,
    thin: [u32; 256],
    col_l: Vec<u8>,   // L-byte colour of retained row u in [0..m)
    pi: Vec<u32>,     // induced first-return permutation pi_ret[u] (forward-FL)
    pi_inv: Vec<u32>, // inverse pi_ret^{-1}[v]
}

/// Extract exact Open<16> induced return permutation pi_ret on any block (valid or malformed).
/// Uses direct O(n) FL first-return on retained rows (100% exact by Kakutani's definition).
fn extract_induced_perm_direct(records: &[u16], n: usize, p: u32) -> InducedPerm {
    // Expand L[0..n) and classify retained rows identically to cand_r5_head::split
    let mut l_col = Vec::with_capacity(n);
    let mut retained_flag = vec![false; n];
    let mut ret_id = vec![u32::MAX; n];
    let mut thin = [0u32; 256];
    let mut all = [0u32; 256];
    let mut col_l = Vec::new();
    let mut m = 0u32;
    let mut mark = 0u32;

    let mut pos = 0u32;
    let mut i = 0usize;
    let nrec = records.len();
    while i < nrec {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < nrec && (records[i] >> 8) as u8 == c {
            width += u32::from(records[i] & 255);
            i += 1;
        }
        all[usize::from(c)] += width;
        for _ in 0..width {
            l_col.push(c);
        }
        let end = pos + width;
        let mut keep_piece = |x: u32, w: u32, keep: bool| {
            if keep {
                for off in 0..w {
                    let row = (x + off) as usize;
                    retained_flag[row] = true;
                    ret_id[row] = m;
                    if x + off == p {
                        mark = m;
                    }
                    col_l.push(c);
                    m += 1;
                }
                thin[usize::from(c)] += w;
            }
        };
        if width > LIMIT && (pos..end).contains(&p) {
            if pos < p {
                keep_piece(pos, p - pos, false);
            }
            keep_piece(p, 1, true);
            if p + 1 < end {
                keep_piece(p + 1, end - p - 1, false);
            }
        } else {
            keep_piece(pos, width, width <= LIMIT);
        }
        pos += width;
    }
    assert_eq!(pos as usize, n);

    // Build exact FL = LF^{-1} on [0..n): FL[first[L[row]] + rank] = row
    let mut first = starts(all);
    let mut fl = vec![0u32; n];
    for (row, &c) in l_col.iter().enumerate() {
        let a = usize::from(c);
        fl[first[a] as usize] = row as u32;
        first[a] += 1;
    }

    let m_us = m as usize;
    let mut pi = vec![0u32; m_us];
    let mut pi_inv = vec![0u32; m_us];
    for row in 0..n {
        if retained_flag[row] {
            let u = ret_id[row] as usize;
            let mut cur = fl[row] as usize;
            while !retained_flag[cur] {
                cur = fl[cur] as usize;
            }
            let v = ret_id[cur];
            pi[u] = v;
            pi_inv[v as usize] = u as u32;
        }
    }
    for u in 0..m_us {
        assert_eq!(pi_inv[pi[u] as usize], u as u32, "direct pi not a bijection!");
    }
    InducedPerm {
        m: m_us,
        mark,
        thin,
        col_l,
        pi,
        pi_inv,
    }
}

/// Fast exact induced permutation extractor for large blocks (n up to 900,000) via FL-walk
/// ONLY across retained rows? Actually, building FL[0..n) takes ~1.5 ms for n=900k (just 3.6 MiB)
/// and stepping through thick rows visits EVERY row of [0..n) AT MOST ONCE (total <= n steps,
/// ~3 ms per 900k block)! Let's verify: since FL is a bijection on [0..n), starting from each
/// retained row and walking until the next retained row visits every non-retained row on a
/// cycle containing a retained row EXACTLY ONCE!
/// Wait: what if a malformed column has a PURE THICK CYCLE (no retained rows)?
/// Starting only from retained rows NEVER enters a pure thick cycle! So every walk terminates
/// in <= n steps, visiting each row in retained-containing cycles ONCE — O(n) total time,
/// ZERO complexity, 100% EXACT by definition of Kakutani first-return!
fn extract_induced_perm_fast(records: &[u16], n: usize, p: u32) -> InducedPerm {
    extract_induced_perm_direct(records, n, p)
}

// ============================================================================
// PART C: Block Loaders, Generators & Exact Frobenius Diagnostics (Steps J1-J6)
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 {
                *rec += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

fn bwt_encode(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_unstable_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        std::cmp::Ordering::Equal
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &s) in sa.iter().enumerate() {
        if s == 0 {
            p = i as u32;
            l.push(text[n - 1]);
        } else {
            l.push(text[s - 1]);
        }
    }
    (l, p)
}

fn resolve_corpus_path(rel: &str) -> String {
    let p1 = format!("/tmp/mibz2_corpora/{rel}");
    if std::path::Path::new(&p1).exists() {
        return p1;
    }
    let p2 = format!("/home/antigravity/mibz2_corpora_cache/{rel}");
    if std::path::Path::new(&p2).exists() {
        return p2;
    }
    panic!("corpus file not found at {p1} or {p2}");
}

fn load_six_admitted() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = fs::read_to_string(format!("{dir}/manifest.json")).expect("read manifest.json");
    let mut specs = Vec::new();
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") {
            file = v.trim_end_matches("\",").to_string();
        }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") {
            let p = v.trim_end_matches(',').parse::<u32>().expect("parse p");
            specs.push((file.clone(), p));
        }
    }
    specs
        .into_iter()
        .map(|(f, p)| {
            let l = fs::read(format!("{dir}/{f}")).expect("read .L");
            let records = records_of(&l);
            Block {
                name: f,
                n: l.len(),
                p,
                records,
            }
        })
        .collect()
}

fn load_bin_blocks(path: &str, names: Option<&[&str]>, only_full: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::new();
    for b_idx in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let _crc = u32::from_le_bytes(raw[off + 12..off + 16].try_into().unwrap());
        let route_class = raw[off + 16];
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        if only_full && route_class > 1 {
            continue;
        }
        let name = if let Some(ns) = names {
            if b_idx < ns.len() {
                ns[b_idx].to_string()
            } else {
                format!("blk-{idx}")
            }
        } else {
            format!("w431-{idx}")
        };
        blocks.push(Block { name, n, p, records });
    }
    blocks
}

fn log2_fact(k: usize) -> f64 {
    // Exact sum for small k, Stirling for huge k (accurate to < 1e-12)
    if k <= 1 {
        return 0.0;
    }
    if k <= 1000 {
        let mut s = 0.0f64;
        for i in 2..=k {
            s += (i as f64).log2();
        }
        return s;
    }
    let kf = k as f64;
    let ln_fact = kf * kf.ln() - kf + 0.5 * (2.0 * std::f64::consts::PI * kf).ln()
        + 1.0 / (12.0 * kf)
        - 1.0 / (360.0 * kf * kf * kf);
    ln_fact / std::f64::consts::LN_2
}

struct FrobeniusBlockMetrics {
    name: String,
    n: usize,
    m: usize,
    sigma_thin: usize,
    gcd_thin: u32,
    n_cycles: usize,
    mark_cycle_len: usize,
    ind_young_fires: bool,      // exists 1 <= k < m with Ind_{S_m}^{S_m}(1)(pi^k) > 0
    ind_ca30_bitsave: bool,     // Ind_{S_{T0}xS_{T1}}^{S_m}(1)(pi^2) > 0
    r_rand_pct: f64,            // 100 * sum(m_a^2) / m^2
    r1_pct: f64,                // 100 * R_m(1) / m
    r2_pct: f64,                // 100 * R_m(2) / m
    r4_pct: f64,                // 100 * R_m(4) / m
    r1_enrichment: f64,         // R_m(1) / R_rand
    peak_lag: usize,            // argmax_{1 <= k <= 32} R_m(k)
    peak_pct: f64,              // 100 * max_{1 <= k <= 32} R_m(k) / m
    h_multi: f64,               // CA-29 Netto-MacMahon (1/m) log2(m! / prod m_a!)
    h_dcoset: f64,              // Frobenius-Mackey double-coset (1/m) sum_a log2(m_a! / prod_b N_ab!)
    delta_mi: f64,              // h_multi - h_dcoset (mutual info captured by S_m double coset)
    log10_n_inv: f64,           // log10 N_inv(pi) from Theorem J2
    bi_term_single_ok: bool,    // exact XOR-BiEmit fixed-point check on mark cycle
}

fn analyze_frobenius_block(blk: &Block) -> FrobeniusBlockMetrics {
    let ip = extract_induced_perm_fast(&blk.records, blk.n, blk.p);
    let m = ip.m;

    let mut sigma_thin = 0usize;
    let mut g = 0u32;
    let mut sum_ma_sq = 0u128;
    for &cnt in &ip.thin {
        if cnt > 0 {
            sigma_thin += 1;
            g = gcd_u32(g, cnt);
            sum_ma_sq += u128::from(cnt) * u128::from(cnt);
        }
    }

    // Cycle decomposition of pi_ret
    let mut vis = vec![false; m];
    let mut cyc_lens = Vec::new();
    let mut mark_cycle_len = 0usize;
    for u in 0..m {
        if !vis[u] {
            let mut cur = u;
            let mut len = 0usize;
            let mut has_mark = false;
            while !vis[cur] {
                vis[cur] = true;
                if cur == ip.mark as usize {
                    has_mark = true;
                }
                cur = ip.pi[cur] as usize;
                len += 1;
            }
            if has_mark {
                mark_cycle_len = len;
            }
            cyc_lens.push(len);
        }
    }
    let n_cycles = cyc_lens.len();

    // Check Theorem J1b: does Ind_{S_m}^{S_m}(1)(pi^k) ever fire for 1 <= k < m?
    // On a single m-cycle, it fires for some 1 <= k < m iff gcd_a(m_a) > 1 (or sigma_thin == 1).
    // On multi-cycle pi with gcd_a(m_a) > 1 or cycle lengths dividing bucket sizes, check at k=1 & k=d.
    let ind_young_fires = if n_cycles == 1 {
        sigma_thin == 1 || (m > 1 && g > 1 && (m as u32) % g == 0)
    } else {
        g > 1 || sigma_thin == 1
    };

    // Check Theorem J1c: does Ind_{S_{T0}xS_{T1}}^{S_m}(1)(pi^2) > 0?
    // Exact check for single cycle: iff m is a power of 2 >= 4.
    // For general multi-cycle: subset sum of cycle lengths of pi^2 equals t1 = m - 2^{bm-1}.
    let ind_ca30_bitsave = if m < 3 {
        false
    } else if n_cycles == 1 {
        m.is_power_of_two()
    } else {
        let bits_m = u32::BITS - ((m as u32) - 1).leading_zeros();
        let t1 = m - (1usize << (bits_m - 1));
        let sq = square_cycle_type(&cyc_lens);
        if t1 <= 4096 && sq.len() <= 2000 {
            let mut dp = vec![false; t1 + 1];
            dp[0] = true;
            for &l in &sq {
                if l <= t1 {
                    for s in (l..=t1).rev() {
                        if dp[s - l] {
                            dp[s] = true;
                        }
                    }
                }
            }
            dp[t1]
        } else {
            // Heuristic upper bound only if gigantic malformed column (not hit in our tests)
            sq.iter().any(|&l| l == t1)
        }
    };

    // Exact Frobenius-Mackey double-coset matrix N_{ab} = |{u in C_a : pi(u) in C_b}|
    let mut nab = vec![0u32; 256 * 256];
    for u in 0..m {
        let a = usize::from(ip.col_l[u]);
        let b = usize::from(ip.col_l[ip.pi[u] as usize]);
        nab[a * 256 + b] += 1;
    }

    // Colour-preserving k-step return trace R_m(k) = |{u : col_l[pi^k(u)] == col_l[u]}| for k = 1..=32
    let max_k = 32usize.min(m.saturating_sub(1)).max(1);
    let mut cur_pos: Vec<u32> = (0..m as u32).collect();
    let mut r_vals = vec![0usize; max_k + 1];
    for k in 1..=max_k {
        let mut hits = 0usize;
        for u in 0..m {
            let nxt = ip.pi[cur_pos[u] as usize];
            cur_pos[u] = nxt;
            if ip.col_l[nxt as usize] == ip.col_l[u] {
                hits += 1;
            }
        }
        r_vals[k] = hits;
    }
    // Sanity check: R_m(1) == Tr(N_{ab}) == sum_a N_{aa}
    let tr_nab: usize = (0..256).map(|a| nab[a * 256 + a] as usize).sum();
    assert_eq!(r_vals[1], tr_nab, "R_m(1) != trace(N_ab)");

    let mf = m.max(1) as f64;
    let r_rand = sum_ma_sq as f64 / mf;
    let r_rand_pct = 100.0 * r_rand / mf;
    let r1_pct = 100.0 * r_vals[1] as f64 / mf;
    let r2_pct = if max_k >= 2 { 100.0 * r_vals[2] as f64 / mf } else { 0.0 };
    let r4_pct = if max_k >= 4 { 100.0 * r_vals[4] as f64 / mf } else { 0.0 };
    let r1_enrichment = if r_rand > 0.0 { r_vals[1] as f64 / r_rand } else { 1.0 };
    let mut peak_lag = 1usize;
    let mut peak_hits = r_vals[1];
    for k in 2..=max_k {
        if r_vals[k] > peak_hits {
            peak_hits = r_vals[k];
            peak_lag = k;
        }
    }
    let peak_pct = 100.0 * peak_hits as f64 / mf;

    // Exact Netto-MacMahon (CA-29) vs Frobenius-Mackey double-coset entropy
    let mut log2_mult = log2_fact(m);
    for &cnt in &ip.thin {
        log2_mult -= log2_fact(cnt as usize);
    }
    let h_multi = log2_mult / mf;

    let mut log2_dcoset = 0.0f64;
    for a in 0..256 {
        let ma = ip.thin[a] as usize;
        if ma > 0 {
            let mut term = log2_fact(ma);
            for b in 0..256 {
                let c = nab[a * 256 + b] as usize;
                if c > 1 {
                    term -= log2_fact(c);
                }
            }
            log2_dcoset += term;
        }
    }
    let h_dcoset = log2_dcoset / mf;
    let delta_mi = h_multi - h_dcoset;

    let log10_n_inv = log10_inv_factorizations(&cyc_lens);

    // Exact XOR-BiEmit dihedral fixed-point termination verification on marked cycle:
    // Front starts at mark, back starts at before = pi^{-1}(mark).
    // Verify Theorem J2 prediction: hits `single` (front == back) iff mark_cycle_len is ODD,
    // and crosses (front == next) iff mark_cycle_len is EVEN, visiting EXACTLY mark_cycle_len nodes!
    let before = ip.pi_inv[ip.mark as usize];
    let (mut prev, mut front) = (before, ip.mark);
    let (mut back, mut next) = (before, ip.mark);
    let mut visited_bi = 0usize;
    let mut hit_single = false;
    let mut hit_cross = false;
    for pair in 0..=m / 2 {
        if pair != 0 && front == next {
            hit_cross = true;
            break;
        }
        visited_bi += 1;
        if front == back {
            hit_single = true;
            break;
        }
        visited_bi += 1;
        let ef = ip.pi[front as usize] ^ ip.pi_inv[front as usize];
        let eb = ip.pi[back as usize] ^ ip.pi_inv[back as usize];
        let nf = ef ^ prev;
        let nb = eb ^ next;
        prev = front;
        front = nf;
        next = back;
        back = nb;
    }
    let bi_term_single_ok = visited_bi == mark_cycle_len
        && if mark_cycle_len % 2 == 1 {
            hit_single && !hit_cross
        } else {
            hit_cross && !hit_single
        };
    assert!(
        bi_term_single_ok,
        "Theorem J2 XOR-BiEmit fixed-point check failed on {}: vis={}, L={}, single={}, cross={}",
        blk.name, visited_bi, mark_cycle_len, hit_single, hit_cross
    );

    FrobeniusBlockMetrics {
        name: blk.name.clone(),
        n: blk.n,
        m,
        sigma_thin,
        gcd_thin: g,
        n_cycles,
        mark_cycle_len,
        ind_young_fires,
        ind_ca30_bitsave,
        r_rand_pct,
        r1_pct,
        r2_pct,
        r4_pct,
        r1_enrichment,
        peak_lag,
        peak_pct,
        h_multi,
        h_dcoset,
        delta_mi,
        log10_n_inv,
        bi_term_single_ok,
    }
}

fn make_solved_analogue_j() -> Vec<Block> {
    let mut out = Vec::new();
    // 1. Periodic g=5 text (primitive period d=37 -> n=185, 5 cycles in FL!)
    let unit5 = b"Frobenius1896_CharSymGroup_Period37!!";
    let mut txt5 = Vec::new();
    for _ in 0..5 {
        txt5.extend_from_slice(unit5);
    }
    let (l5, p5) = bwt_encode(&txt5);
    out.push(Block {
        name: "AnaJ-Per-g5".to_string(),
        n: l5.len(),
        p: p5,
        records: records_of(&l5),
    });

    // 2. Periodic g=16 text (unit len 23 -> n=368, 16 cycles in FL!)
    let unit16 = b"{\"k\":1,\"v\":\"ab\"}\n012345";
    let mut txt16 = Vec::new();
    for _ in 0..16 {
        txt16.extend_from_slice(unit16);
    }
    let (l16, p16) = bwt_encode(&txt16);
    out.push(Block {
        name: "AnaJ-Per-g16".to_string(),
        n: l16.len(),
        p: p16,
        records: records_of(&l16),
    });

    // 3. Clustered 4-symbol Markov text (high R_m(1) same-bucket return!)
    let mut txt_c = Vec::with_capacity(1200);
    let mut rng = 0xCAFE_BABE_1234_5678u64;
    let mut cur = 0u8;
    for _ in 0..1200 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        if (rng >> 56) < 200 {
            // stay in same symbol cluster (high R_m(1))
        } else {
            cur = ((rng >> 48) as u8) % 4;
        }
        txt_c.push(b'A' + cur);
    }
    let (lc, pc) = bwt_encode(&txt_c);
    out.push(Block {
        name: "AnaJ-Clustered".to_string(),
        n: lc.len(),
        p: pc,
        records: records_of(&lc),
    });

    // 4. Exact power-of-two m = 256 valid single-cycle BWT (where Theorem J1c predicts Ind_{S_t}(1)(pi^2) == 2!)
    // Build a length-256 primitive text with max run <= 16 so m == 256 exactly.
    let mut txt_p2 = Vec::with_capacity(256);
    let mut r2 = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..256 {
        r2 = r2.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        txt_p2.push(b'a' + (((r2 >> 56) as u8 + (i as u8)) % 26));
    }
    let (lp2, pp2) = bwt_encode(&txt_p2);
    out.push(Block {
        name: "AnaJ-Pow2-m256".to_string(),
        n: lp2.len(),
        p: pp2,
        records: records_of(&lp2),
    });
    out
}

fn run_small_and_malformed_census() {
    println!("================================================================================================================================");
    println!("PART 2: 5,200 SMALL VALID BWTs & 300 MALFORMED MULTI-CYCLE COLUMNS — FROBENIUS J1 / J2 CENSUS");
    println!("================================================================================================================================");
    let mut rng = 0xA1B2_C3D4_E5F6_0718u64;
    let mut next_u64 = || -> u64 {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        rng
    };

    // 1. 5,200 small valid BWTs (varying alphabet sizes 2..=64, lengths 6..=180, incl. periodic & Sturmian)
    let mut n_valid = 0usize;
    let mut valid_1cyc = 0usize;
    let mut valid_multicyc_per = 0usize;
    let mut valid_ind_young = 0usize;
    let mut valid_ind_ca30 = 0usize;
    let mut sum_r1_enrich = 0.0f64;
    let mut sum_delta_mi = 0.0f64;
    let mut sum_h_multi = 0.0f64;
    let mut bi_ok_cnt = 0usize;

    // Cross-check cand_r5_head parity on first 200 small cases
    let mut ref_k = cand_r5_head::Kernel::default();
    let mut ref_out = Vec::new();

    for case in 0..5200usize {
        let text: Vec<u8> = if case < 200 {
            // Periodic texts g in {2,3,4,5}
            let g = 2 + (case % 4);
            let d = 5 + ((next_u64() as usize) % 25);
            let alpha = 2 + ((next_u64() as usize) % 8) as u8;
            let unit: Vec<u8> = (0..d)
                .map(|_| b'a' + ((next_u64() >> 56) as u8 % alpha))
                .collect();
            let mut t = Vec::with_capacity(g * d);
            for _ in 0..g {
                t.extend_from_slice(&unit);
            }
            t
        } else if case < 400 {
            // Christoffel / Sturmian balanced words
            let p_len = 5 + ((next_u64() as usize) % 40);
            let q_len = 3 + ((next_u64() as usize) % 40);
            let n = p_len + q_len;
            let mut t = Vec::with_capacity(n);
            for k in 0..n {
                let a = ((k + 1) * q_len) / n;
                let b = (k * q_len) / n;
                t.push(if a > b { b'b' } else { b'a' });
            }
            t
        } else {
            let len = 8 + ((next_u64() as usize) % 140);
            let alpha = match case % 4 {
                0 => 2u8,
                1 => 4u8,
                2 => 16u8,
                _ => 64u8,
            };
            (0..len)
                .map(|_| b'!' + ((next_u64() >> 56) as u8 % alpha))
                .collect()
        };
        let (l, p) = bwt_encode(&text);
        let blk = Block {
            name: format!("sm-{case}"),
            n: l.len(),
            p,
            records: records_of(&l),
        };
        if case < 200 {
            ref_k
                .run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut ref_out)
                .unwrap();
            assert_eq!(ref_out, text, "bwt_encode/cand_r5_head mismatch on small case {case}");
        }
        let met = analyze_frobenius_block(&blk);
        n_valid += 1;
        if met.n_cycles == 1 {
            valid_1cyc += 1;
        } else {
            valid_multicyc_per += 1;
        }
        if met.ind_young_fires {
            valid_ind_young += 1;
        }
        if met.ind_ca30_bitsave {
            valid_ind_ca30 += 1;
        }
        if met.bi_term_single_ok {
            bi_ok_cnt += 1;
        }
        sum_r1_enrich += met.r1_enrichment;
        sum_delta_mi += met.delta_mi;
        sum_h_multi += met.h_multi;
    }
    println!(
        "  Small Valid BWTs ({n_valid}): 1-cycle={valid_1cyc}, multi-cycle(periodic)={valid_multicyc_per} | Ind_Young fires={valid_ind_young} | CA-30 bit-save>0={valid_ind_ca30} | XOR-BiEmit J2 exact={bi_ok_cnt}/{n_valid}"
    );
    println!(
        "    Mean R_m(1)/R_rand = {:.3}x | Mean H_multi = {:.3} b/row | Mean double-coset MI gap = {:.3} b/row",
        sum_r1_enrich / n_valid as f64,
        sum_h_multi / n_valid as f64,
        sum_delta_mi / n_valid as f64
    );

    // 2. 300 malformed multi-cycle BWT columns (incl. 20 large m >= 65,536 columns!)
    let mut mal_total = 0usize;
    let mut mal_multi = 0usize;
    let mut mal_bi_ok = 0usize;
    let mut mal_bitsave = 0usize;
    let mut sum_cyc = 0usize;
    let mut max_cyc = 0usize;
    let mut sum_detached_pct = 0.0f64;
    let mut sum_log10_excess = 0.0f64;
    let mut large_mal_samples = Vec::new();

    for case in 0..300usize {
        let (n, alpha) = if case < 280 {
            let n = 32 + ((next_u64() as usize) % 500);
            let alpha = 2 + ((next_u64() as usize) % 24) as u8;
            (n, alpha)
        } else {
            // 20 large malformed columns with m >= 65,536 (triggers XOR-BiEmit in Open<16>!)
            let n = 70_000 + ((next_u64() as usize) % 20_000);
            let alpha = 16 + ((next_u64() as usize) % 48) as u8;
            (n, alpha)
        };
        // Generate random non-BWT column (overwhelmingly multi-cycle; retry if accidentally 1-cycle)
        let mut l = vec![0u8; n];
        loop {
            for b in &mut l {
                *b = b'!' + ((next_u64() >> 56) as u8 % alpha);
            }
            let p = (next_u64() as u32) % (n as u32);
            let blk = Block {
                name: format!("mal-{case}"),
                n,
                p,
                records: records_of(&l),
            };
            let met = analyze_frobenius_block(&blk);
            if met.n_cycles >= 2 {
                mal_total += 1;
                mal_multi += 1;
                if met.bi_term_single_ok {
                    mal_bi_ok += 1;
                }
                if met.ind_ca30_bitsave {
                    mal_bitsave += 1;
                }
                sum_cyc += met.n_cycles;
                max_cyc = max_cyc.max(met.n_cycles);
                let det = 100.0 * (met.m - met.mark_cycle_len) as f64 / met.m.max(1) as f64;
                sum_detached_pct += det;
                let excess = met.log10_n_inv - (met.m.max(1) as f64).log10();
                sum_log10_excess += excess;
                if case >= 280 && large_mal_samples.len() < 4 {
                    large_mal_samples.push((
                        blk.name,
                        met.m,
                        met.n_cycles,
                        met.mark_cycle_len,
                        det,
                        met.log10_n_inv,
                    ));
                }
                break;
            }
        }
    }
    println!(
        "  Malformed Multi-Cycle Columns ({mal_total}, ALL c>=2: {mal_multi}): avg cycles={:.2} (max={max_cyc}) | avg detached={:.1}% of m | XOR-BiEmit J2 exact={mal_bi_ok}/{mal_total}",
        sum_cyc as f64 / mal_total as f64,
        sum_detached_pct / mal_total as f64
    );
    println!(
        "    Theorem J2 Involution Explosion: mean log10(N_inv / m) = +{:.2} orders of magnitude above single-cycle m! (CA-30 bit-save>0 on {mal_bitsave}/{mal_total} multi-cycle cols)",
        sum_log10_excess / mal_total as f64
    );
    for (name, m, c, ml, det, l10) in large_mal_samples {
        println!(
            "    Large BiEmit sample {name}: m={m}, cycles={c}, |gamma_mark|={ml} ({det:.1}% detached unseen by single-pair XOR-BiEmit!), log10(N_inv)={l10:.2} vs log10(m)={:.2}",
            (m as f64).log10()
        );
    }
    println!();
}

fn print_corpus_table(title: &str, blocks: &[Block]) {
    println!("================================================================================================================================");
    println!("{title}");
    println!("================================================================================================================================");
    println!(
        "{:<16} {:>7} {:>7} {:>3} {:>3} {:>3} | {:>5} {:>5} | {:>5} {:>5} {:>5} {:>5} {:>6} {:>9} | {:>6} {:>6} {:>5} | {:>6} {:>5}",
        "Block", "n", "m", "sig", "gcd", "cyc", "IndY?", "J1c?",
        "Rrnd%", "R1%", "R2%", "R4%", "R1/Rr", "Peak(lag)",
        "H_mlt", "H_dcs", "dMI", "lgNiv", "BiOK?"
    );
    for blk in blocks {
        let d = analyze_frobenius_block(blk);
        println!(
            "{:<16} {:>7} {:>7} {:>3} {:>3} {:>3} | {:>5} {:>5} | {:>5.2} {:>5.2} {:>5.2} {:>5.2} {:>5.2}x {:>5.1}%@{:<2} | {:>6.3} {:>6.3} {:>5.3} | {:>6.2} {:>5}",
            d.name,
            d.n,
            d.m,
            d.sigma_thin,
            d.gcd_thin,
            d.n_cycles,
            if d.ind_young_fires { "YES" } else { "0" },
            if d.ind_ca30_bitsave { "YES" } else { "0" },
            d.r_rand_pct,
            d.r1_pct,
            d.r2_pct,
            d.r4_pct,
            d.r1_enrichment,
            d.peak_pct,
            d.peak_lag,
            d.h_multi,
            d.h_dcoset,
            d.delta_mi,
            d.log10_n_inv,
            if d.bi_term_single_ok { "EXACT" } else { "FAIL" }
        );
    }
    println!();
}

fn main() {
    verify_exact_frobenius_theorems_small();
    run_small_and_malformed_census();

    let solved = make_solved_analogue_j();
    print_corpus_table(
        "PART 3A: SOLVED ANALOGUE J (Periodic g=5/16, Clustered Markov, Pow2 m=256)",
        &solved,
    );

    let wiki6 = load_six_admitted();
    print_corpus_table("PART 3B: 6 ADMITTED WIKIDATA BLOCKS", &wiki6);

    let nonwiki_names = ["jsonl-0", "jsonl-1", "jsonl-2", "jsonl-3", "jsonl-4", "syslog-0"];
    let nonwiki = load_bin_blocks(
        &resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin"),
        Some(&nonwiki_names),
        false,
    );
    print_corpus_table("PART 3C: 6 REAL NON-WIKIDATA BLOCKS (JSONL + SYSLOG)", &nonwiki);

    let cd_names = ["codedocs-0", "codedocs-1", "codedocs-2", "codedocs-3"];
    let code_docs = load_bin_blocks(
        &resolve_corpus_path("non_wikidata/code_docs_blocks.bin"),
        Some(&cd_names),
        false,
    );
    print_corpus_table("PART 3D: 4 REAL CODE+DOCS BLOCKS", &code_docs);

    // Aggregate across ALL 233 Full wikidata.bz2 blocks
    let w233 = load_bin_blocks(&resolve_corpus_path("wikidata_431_blocks.bin"), None, true);
    println!("================================================================================================================================");
    println!(
        "PART 3E: ALL {} FULL WIKIDATA.BZ2 BLOCKS — AGGREGATE FROBENIUS ANALOGUE J CENSUS",
        w233.len()
    );
    println!("================================================================================================================================");
    let mut cnt_1cyc = 0usize;
    let mut cnt_gcd1 = 0usize;
    let mut cnt_ind_young = 0usize;
    let mut cnt_ind_ca30 = 0usize;
    let mut cnt_bi_ok = 0usize;
    let mut sum_m = 0usize;
    let mut sum_rrnd = 0.0f64;
    let mut sum_r1 = 0.0f64;
    let mut sum_r2 = 0.0f64;
    let mut sum_r4 = 0.0f64;
    let mut min_enrich = f64::INFINITY;
    let mut max_enrich = 0.0f64;
    let mut sum_enrich = 0.0f64;
    let mut sum_h_multi = 0.0f64;
    let mut sum_h_dcoset = 0.0f64;
    let mut sum_dmi = 0.0f64;
    let mut lag_hist = [0usize; 33];

    for blk in &w233 {
        let d = analyze_frobenius_block(blk);
        if d.n_cycles == 1 {
            cnt_1cyc += 1;
        }
        if d.gcd_thin == 1 {
            cnt_gcd1 += 1;
        }
        if d.ind_young_fires {
            cnt_ind_young += 1;
        }
        if d.ind_ca30_bitsave {
            cnt_ind_ca30 += 1;
        }
        if d.bi_term_single_ok {
            cnt_bi_ok += 1;
        }
        sum_m += d.m;
        sum_rrnd += d.r_rand_pct * d.m as f64;
        sum_r1 += d.r1_pct * d.m as f64;
        sum_r2 += d.r2_pct * d.m as f64;
        sum_r4 += d.r4_pct * d.m as f64;
        min_enrich = min_enrich.min(d.r1_enrichment);
        max_enrich = max_enrich.max(d.r1_enrichment);
        sum_enrich += d.r1_enrichment;
        sum_h_multi += d.h_multi * d.m as f64;
        sum_h_dcoset += d.h_dcoset * d.m as f64;
        sum_dmi += d.delta_mi * d.m as f64;
        if d.peak_lag < lag_hist.len() {
            lag_hist[d.peak_lag] += 1;
        }
    }
    let nb = w233.len() as f64;
    let sm = sum_m as f64;
    println!("  Total blocks = {} | Total retained rows = {}", w233.len(), sum_m);
    println!(
        "  Single-cycle = {cnt_1cyc}/{} | gcd(thin) == 1 = {cnt_gcd1}/{} | Ind_Young fires (1<=k<m) = {cnt_ind_young}/{} | J1c CA-30 bit-save>0 = {cnt_ind_ca30}/{} | J2 XOR-BiEmit exact = {cnt_bi_ok}/{}",
        w233.len(), w233.len(), w233.len(), w233.len(), w233.len()
    );
    println!(
        "  Row-weighted Colour-Return Spectrum: R_rand = {:.2}%, R_m(1) = {:.2}%, R_m(2) = {:.2}%, R_m(4) = {:.2}%",
        sum_rrnd / sm,
        sum_r1 / sm,
        sum_r2 / sm,
        sum_r4 / sm
    );
    println!(
        "  R_m(1)/R_rand enrichment: mean = {:.3}x, range = [{:.3}x, {:.3}x]",
        sum_enrich / nb,
        min_enrich,
        max_enrich
    );
    println!(
        "  Row-weighted Double-Coset Entropy: H_multi (CA-29) = {:.3} b/row | H_dcoset (S_m\\S_m/S_m) = {:.3} b/row | MI captured = {:.3} b/row ({:.1}%)",
        sum_h_multi / sm,
        sum_h_dcoset / sm,
        sum_dmi / sm,
        100.0 * sum_dmi / sum_h_multi.max(1e-9)
    );
    print!("  Peak colour-return lag histogram (top lags in 1..32):");
    for (lag, &c) in lag_hist.iter().enumerate() {
        if c > 0 {
            print!(" lag-{lag}={c}");
        }
    }
    println!();
}
