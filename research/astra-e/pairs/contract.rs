// Two ASCII literals in one edge. Keep all original dictionary codes below first_pair.
const ALPHABET: u32 = 128;
const PAIR_CODES: u32 = ALPHABET * ALPHABET;

fn contract<const PROBE: bool>(
    edges: &mut [u32],
    mark: u32,
    bits: u32,
    first_pair: u32,
) -> (usize, usize) {
    let mask = (1 << bits) - 1;
    let mut lookups = 0;
    let mut rewritten = 0;
    for i in 0..edges.len() {
        let edge = edges[i];
        let first = edge >> bits;
        let next = edge & mask;
        if first >= ALPHABET || next == mark {
            continue;
        }
        let successor = edges[next as usize];
        if PROBE {
            lookups += 1;
        }
        let second = successor >> bits;
        if second < ALPHABET {
            let code = first_pair + first * ALPHABET + second;
            edges[i] = (code << bits) | (successor & mask);
            if PROBE {
                rewritten += 1;
            }
        }
    }
    (lookups, rewritten)
}

