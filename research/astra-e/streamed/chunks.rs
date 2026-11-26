use mibz2::{rle, ErrorKind, Kernels};

const CHUNK: usize = 16 * 1024;

// An interior cut is invalid inside the first four equal bytes of a run, or
// immediately before that run's count byte. Together these two tests rule out
// both cases, independent of the actual parse of the preceding bytes.
fn safe_prefix(input: &[u8]) -> usize {
    debug_assert!(input.len() >= 8);
    let last = input.len() - 1;
    for c in ((last - 3)..=last).rev() {
        if input[c] != input[c - 1]
            && (input[c - 1] != input[c - 2]
                || input[c - 1] != input[c - 3]
                || input[c - 1] != input[c - 4])
        {
            return c;
        }
    }
    // The previous flush supplied a certified start. A full 5-byte token is
    // locally decidable wherever at least five bytes remain. Leave 0..4 bytes.
    let mut i = 0;
    while i + 4 < input.len() {
        let byte = input[i];
        if input[i + 1] == byte && input[i + 2] == byte && input[i + 3] == byte {
            i += 5;
        } else {
            i += 1;
        }
    }
    i
}

fn flush(chunk: &mut Vec<u8>, out: &mut Vec<u8>, k: Kernels) -> Result<(), ErrorKind> {
    let prefix = safe_prefix(chunk);
    rle::unrle(&chunk[..prefix], out, k)?;
    let tail = chunk.len() - prefix;
    chunk.copy_within(prefix.., 0);
    chunk.truncate(tail);
    Ok(())
}

// Words may be longer than the staging buffer, even the length of a block.
// Always append at most CHUNK bytes before draining a full chunk.
fn word(chunk: &mut Vec<u8>, bytes: &[u8], out: &mut Vec<u8>, k: Kernels) -> Result<(), ErrorKind> {
    if bytes.len() <= CHUNK {
        chunk.extend_from_slice(bytes);
        if chunk.len() >= CHUNK {
            flush(chunk, out, k)?;
        }
    } else {
        for part in bytes.chunks(CHUNK) {
            chunk.extend_from_slice(part);
            if chunk.len() >= CHUNK {
                flush(chunk, out, k)?;
            }
        }
    }
    Ok(())
}
