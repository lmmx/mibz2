//! Write two independent walks into opposite ends of the final text.
pub(super) fn emit(
    forward: &[u32], backward: &[u32], dictionary: &[u8], offsets: &[u32],
    n: usize, mark: u32, bits: u32, out: &mut Vec<u8>,
) {
    out.resize(n, 0);
    let mask = (1u32 << bits) - 1;
    let (mut left, mut right) = (0usize, n);
    let (mut front, mut back) = (mark, mark);
    loop {
        // Both addresses are known before either load returns.
        let a = forward[front as usize];
        let b = backward[back as usize];
        front = a & mask;
        let ca = a >> bits;
        if ca < 256 {
            out[left] = ca as u8;
            left += 1;
        } else {
            let i = (ca - 256) as usize;
            let word = &dictionary[offsets[i] as usize..offsets[i + 1] as usize];
            out[left..left + word.len()].copy_from_slice(word);
            left += word.len();
        }
        if front == back {
            break;
        }
        back = b & mask;
        let cb = b >> bits;
        if cb < 256 {
            right -= 1;
            out[right] = cb as u8;
        } else {
            let i = (cb - 256) as usize;
            let word = &dictionary[offsets[i] as usize..offsets[i + 1] as usize];
            right -= word.len();
            out[right..right + word.len()].copy_from_slice(word);
        }
        if front == back {
            break;
        }
    }
    if left != right {
        assert!(left < right);
        out.copy_within(right.., left);
        out.truncate(left + n - right);
        super::repeat(n, out);
    }
}
