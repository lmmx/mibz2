// bzt: decode chosen blocks of a .bz2 file to their T-vector, exactly as mibz2's
// build_two_pass makes it (t[s] = (i << 8) | ll[s], s = ftab[ll[i]]++), and check each
// block by walking it, undoing RLE1 and comparing the block CRC stored in the file.
// Output per block: OUT/tt-b<k>.bin = u32 n, u32 orig_ptr, n x u32 entries (Route A's format).
// usage: bzt FILE.bz2 OUTDIR BLOCK...      (blocks counted from 0 over the whole file)
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static uint8_t *buf;
static size_t blen, bpos; // bit position
static inline uint32_t bit1(void) {
  size_t by = bpos >> 3;
  if (by >= blen) { fprintf(stderr, "eof\n"); exit(2); }
  uint32_t v = (buf[by] >> (7 - (bpos & 7))) & 1;
  bpos++;
  return v;
}
static uint32_t bits(int k) {
  uint32_t v = 0;
  for (int i = 0; i < k; i++) v = (v << 1) | bit1();
  return v;
}
static uint32_t crctab[256];
static void crcinit(void) {
  for (uint32_t i = 0; i < 256; i++) {
    uint32_t c = i << 24;
    for (int j = 0; j < 8; j++) c = (c & 0x80000000u) ? (c << 1) ^ 0x04c11db7u : c << 1;
    crctab[i] = c;
  }
}

#define MAXN 1000000
static uint8_t ll[MAXN];
static uint32_t tt[MAXN];

int main(int argc, char **argv) {
  if (argc < 4) { fprintf(stderr, "usage: bzt FILE OUTDIR BLOCK...\n"); return 1; }
  crcinit();
  FILE *f = fopen(argv[1], "rb");
  if (!f) { perror("open"); return 1; }
  fseek(f, 0, SEEK_END);
  blen = ftell(f);
  fseek(f, 0, SEEK_SET);
  buf = malloc(blen);
  if (fread(buf, 1, blen, f) != blen) return 1;
  fclose(f);
  int nwant = argc - 3;
  long *want = calloc(nwant, sizeof(long));
  long maxwant = 0;
  for (int i = 0; i < nwant; i++) { want[i] = atol(argv[3 + i]); if (want[i] > maxwant) maxwant = want[i]; }
  long blk = 0;
  bpos = 0;
  for (;;) {
    // a stream header at a byte boundary
    if ((bpos >> 3) + 4 > blen) break;
    if (bits(8) != 'B' || bits(8) != 'Z' || bits(8) != 'h') { fprintf(stderr, "bad stream at %zu\n", bpos); return 1; }
    uint32_t level = bits(8) - '0';
    (void)level;
    for (;;) {
      uint64_t magic = ((uint64_t)bits(24) << 24) | bits(24);
      if (magic == 0x177245385090ULL) { bits(32); bpos = (bpos + 7) & ~(size_t)7; break; }
      if (magic != 0x314159265359ULL) { fprintf(stderr, "bad block magic\n"); return 1; }
      uint32_t bcrc = bits(32);
      if (bits(1)) { fprintf(stderr, "randomised\n"); return 1; }
      uint32_t orig = bits(24);
      uint32_t used16 = bits(16);
      uint8_t seq[256];
      int nin = 0;
      for (int i = 0; i < 16; i++)
        if (used16 & (0x8000 >> i)) {
          uint32_t u = bits(16);
          for (int j = 0; j < 16; j++) if (u & (0x8000 >> j)) seq[nin++] = (uint8_t)(i * 16 + j);
        }
      int alpha = nin + 2;
      int ngroups = bits(3);
      int nsel = bits(15);
      static uint8_t selmtf[32768], sel[32768];
      for (int i = 0; i < nsel; i++) { int j = 0; while (bit1()) j++; selmtf[i] = j; }
      uint8_t pos[6] = {0, 1, 2, 3, 4, 5};
      for (int i = 0; i < nsel; i++) {
        int v = selmtf[i]; uint8_t tmp = pos[v];
        while (v > 0) { pos[v] = pos[v - 1]; v--; }
        pos[0] = tmp; sel[i] = tmp;
      }
      static uint8_t len[6][258];
      for (int g = 0; g < ngroups; g++) {
        int cur = bits(5);
        for (int s = 0; s < alpha; s++) {
          while (bit1()) { if (bit1()) cur--; else cur++; }
          len[g][s] = cur;
        }
      }
      // canonical codes: limit/base/perm per group, as libbzip2's hbCreateDecodeTables
      static int32_t limit[6][24], base[6][24], perm[6][258], minl[6];
      for (int g = 0; g < ngroups; g++) {
        int mn = 32, mx = 0;
        for (int s = 0; s < alpha; s++) { if (len[g][s] > mx) mx = len[g][s]; if (len[g][s] < mn) mn = len[g][s]; }
        int pp = 0;
        for (int l = mn; l <= mx; l++) for (int s = 0; s < alpha; s++) if (len[g][s] == l) perm[g][pp++] = s;
        int32_t cnt[24] = {0};
        for (int s = 0; s < alpha; s++) cnt[len[g][s]]++;
        int code = 0, idx = 0;
        for (int l = 0; l < 24; l++) { limit[g][l] = -1; base[g][l] = 0; }
        for (int l = mn; l <= mx; l++) {
          base[g][l] = idx - code; // perm index = code + base
          code += cnt[l]; idx += cnt[l];
          limit[g][l] = code - 1;
          code <<= 1;
        }
        minl[g] = mn;
      }
      uint8_t yy[256];
      for (int i = 0; i < 256; i++) yy[i] = i;
      uint32_t counts[256] = {0};
      int n = 0, gi = -1, gleft = 0, g = 0;
      int runpos = 0, runacc = 0;
      for (;;) {
        if (gleft == 0) { gi++; if (gi >= nsel) { fprintf(stderr, "selectors\n"); return 1; } g = sel[gi]; gleft = 50; }
        gleft--;
        int l = minl[g];
        int32_t code = bits(l);
        while (l < 24 && (limit[g][l] < 0 || code > limit[g][l])) { code = (code << 1) | bit1(); l++; }
        if (l >= 24) { fprintf(stderr, "bad code\n"); return 1; }
        int sym = perm[g][code + base[g][l]];
        if (sym <= 1) { runacc += (sym + 1) << runpos; runpos++; continue; }
        if (runpos) {
          uint8_t c = seq[yy[0]];
          if (n + runacc > MAXN) { fprintf(stderr, "overflow\n"); return 1; }
          for (int k = 0; k < runacc; k++) ll[n++] = c;
          counts[c] += runacc;
          runpos = 0; runacc = 0;
        }
        if (sym == alpha - 1) break;
        int v = sym - 1;
        uint8_t tmp = yy[v];
        memmove(yy + 1, yy, v);
        yy[0] = tmp;
        uint8_t c = seq[tmp];
        if (n >= MAXN) { fprintf(stderr, "overflow\n"); return 1; }
        ll[n++] = c;
        counts[c]++;
      }
      int keep = 0;
      for (int i = 0; i < nwant; i++) if (want[i] == blk) keep = 1;
      if (keep) {
        // mibz2's two-pass build: t[s] low byte = ll[s]; t[ftab[ll[i]]++] |= i << 8
        uint32_t ftab[256], cum = 0;
        for (int c = 0; c < 256; c++) { ftab[c] = cum; cum += counts[c]; }
        for (int i = 0; i < n; i++) tt[i] = ll[i];
        for (int i = 0; i < n; i++) { uint32_t s = ftab[ll[i]]++; tt[s] = (tt[s] & 0xff) | ((uint32_t)i << 8); }
        // walk (bwt::walk: p = t[orig]; then n times p = t[p >> 8], byte p as u8), RLE1, CRC
        uint32_t p = tt[orig];
        uint32_t crc = 0xffffffffu;
        int last = -1, run = 0;
        long outn = 0;
        for (int k = 0; k < n; k++) {
          p = tt[p >> 8];
          int c = p & 0xff;
          if (run == 4) { for (int r = 0; r < c; r++) { crc = (crc << 8) ^ crctab[(crc >> 24) ^ last]; outn++; } run = 0; last = -1; continue; }
          crc = (crc << 8) ^ crctab[(crc >> 24) ^ c];
          outn++;
          if (c == last) run++; else { run = 1; last = c; }
        }
        crc = ~crc;
        printf("block %ld: n=%d orig_ptr=%u out=%ld crc=%08x stored=%08x %s\n", blk, n, orig, outn, crc, bcrc,
               crc == bcrc ? "OK" : "MISMATCH");
        if (crc != bcrc) return 3;
        char path[4096];
        snprintf(path, sizeof path, "%s/tt-b%ld.bin", argv[2], blk);
        if (argv[2][0] == 0x2d) { blk++; if (blk > maxwant) { printf("done at block %ld\n", blk); return 0; } continue; }
        FILE *o = fopen(path, "wb");
        uint32_t hdr[2] = {(uint32_t)n, orig};
        fwrite(hdr, 4, 2, o);
        fwrite(tt, 4, n, o);
        fclose(o);
      }
      blk++;
      if (blk > maxwant) { printf("done at block %ld\n", blk); return 0; }
    }
  }
  printf("blocks=%ld\n", blk);
  return 0;
}
