"""Build stock, low-single, and composed-pair research frontends from one checked source."""
import hashlib
import subprocess
import sys
from pathlib import Path

here = Path(__file__).resolve().parent
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True,exist_ok=True)
ref = "d0a42980e896211906234c7679b38760ae44a46e"
stock = subprocess.check_output(["git","show",f"{ref}:src/runs.rs"]).decode()
assert hashlib.sha256(stock.encode()).hexdigest() == "bb8d4fa30854986e5d7d5304a40ee8809588b1bbb259edfb9fe3e11cd9cfdca9"
front = stock[stock.index("/// `src/mtf.rs`'s first level"):stock.index("/// The block's LF table from its records")]
front = front.replace("unsafe fn front_avx2(", "pub(super) unsafe fn front_avx2(")
header = """use mibz2::block::Symbols;
use super::Runs;
const CAP:u32=127;
"""
start = front.index("            let s = *spw.add(tz)")
stop = front.index("            if w != 0 {",start)
single = front[start:stop]
single = single.replace("            if s < 33 {","""            if s <= 16 {
                let mask = steps.add(s - 1);
                f0 = _mm_shuffle_epi8(f0, _mm_load_si128((*mask).a.as_ptr().cast()));
            } else if s < 33 {""")
move = single[single.index("            if s <= 16 {"):single.index("            let c = ")]
pair = """            let s = *spw.add(tz) as usize;
            let c = if s <= 16 && w != 0 && *spw.add(w.trailing_zeros() as usize) <= 16 {
                let second_tz = w.trailing_zeros() as usize;
                // Both literal bits are set, so the two symbols are at least 2.
                let second_rank = *spw.add(second_tz) as usize - 1;
                let idx = 16 * (s - 1) + second_rank;
                let mask = PAIRS.0.as_ptr().add(idx);
                f0 = _mm_shuffle_epi8(f0, _mm_load_si128(mask.cast()));
                let both = _mm_cvtsi128_si32(f0) as u32;
                emit!(((both >> 8) & 255) as u64, _bzhi_u64(after, second_tz as u32) >> tz, second_tz - tz);
                w &= w - 1;
                tz = second_tz;
                (both & 255) as u64
            } else {
""" + move + """                _mm_cvtsi128_si32(f0) as u8 as u64
            };
"""
for name, update, table in [("stock",front[start:stop],""),("single",single,""),("pair",pair,(here/"masks.rs").read_text())]:
    (out/(name+".rs")).write_text(header+table+front[:start]+update+front[stop:])
wide = pair.replace("s <= 16 && w != 0 && *spw.add(w.trailing_zeros() as usize) <= 16",
    "w != 0 && ((s - 1) | (*spw.add(w.trailing_zeros() as usize) as usize - 1)) < 32")
startwide = wide.index("                let idx =")
endwide = wide.index("                let both =",startwide)
wide = wide[:startwide]+"""                let mask=WIDE.0.as_ptr().add(32 * (s-1) + second_rank) as *const __m128i;
                let g0=_mm_or_si128(_mm_shuffle_epi8(f0,_mm_load_si128(mask)),_mm_shuffle_epi8(f1,_mm_load_si128(mask.add(1))));
                f1=_mm_or_si128(_mm_shuffle_epi8(f0,_mm_load_si128(mask.add(2))),_mm_shuffle_epi8(f1,_mm_load_si128(mask.add(3))));
                f0=g0;
"""+wide[endwide:]
original_move = front[front.index("            if s < 33 {",start):front.index("            let c = ",start)]
wide = wide.replace(move, original_move)
(out/"wide.rs").write_text(header+(here/"wide.rs").read_text()+front[:start]+wide+front[stop:])
a=wide.index("                let mask=WIDE.")
b=wide.index("                let both =",a)
compact=wide[:a]+"""                let mask=COMPACT.0.as_ptr().add(32*(s-1)+second_rank) as *const __m128i;
                let low=_mm_load_si128(mask);
                let high=_mm_load_si128(mask.add(1));
                let flip=_mm_set1_epi8(-128);
                let g0=_mm_or_si128(_mm_shuffle_epi8(f0,low),_mm_shuffle_epi8(f1,_mm_xor_si128(low,flip)));
                f1=_mm_or_si128(_mm_shuffle_epi8(f0,high),_mm_shuffle_epi8(f1,_mm_xor_si128(high,flip)));
                f0=g0;
"""+wide[b:]
(out/"compact.rs").write_text(header+(here/"compact.rs").read_text()+front[:start]+compact+front[stop:])
