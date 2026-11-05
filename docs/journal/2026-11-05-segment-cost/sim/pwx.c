// pw: a third, independent trace simulation of master's room walk and of the
// radix-partitioned traversal on real T-vectors (bzt's dumps). No timing; counts only.
//
// pw MODE ARGS...   (see main). Every walk is checked to emit exactly the plain walk's bytes.
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define START 0x80000000u
#define LINK 0x7fffffffu

typedef struct { uint32_t n, orig; uint32_t *t; } Blk;
static Blk load(const char *p) {
  FILE *f = fopen(p, "rb");
  if (!f) { perror(p); exit(1); }
  uint32_t h[2];
  if (fread(h, 4, 2, f) != 2) exit(1);
  Blk b = {h[0], h[1], malloc(4 * (size_t)h[0])};
  if (fread(b.t, 4, b.n, f) != b.n) exit(1);
  fclose(f);
  return b;
}

// ------------------------------------------------------------------ events
enum { C_TT, C_Q, C_LOG, C_META, C_FLAG, C_TILE, C_WMETA, C_SEGW, C_JMETA, C_SEGR, C_J2W, C_OUT, NC };
static const char *CN[NC] = {"tt", "queue", "log", "meta", "flag", "tile", "wmeta", "segw", "jmeta", "segr", "j2w", "out"};
typedef struct { uint64_t *e; size_t n, cap; } Ev;
// an event: class (3 bits) | is-step (1 bit) | byte address (48 bits) in the walk's own space
static inline void push(Ev *v, int cls, uint64_t addr, int step) {
  if (v->n == v->cap) { v->cap = v->cap ? 2 * v->cap : 1 << 20; v->e = realloc(v->e, 8 * v->cap); }
  v->e[v->n++] = ((uint64_t)cls << 60) | ((uint64_t)step << 59) | (addr & ((1ULL << 48) - 1));
}
// object bases (bytes) in a walk's space
#define TT_B (1ULL << 40)
#define Q_B (2ULL << 40)
#define LOG_B (3ULL << 40)
#define META_B (4ULL << 40)
#define TILE_B (5ULL << 40)
#define BUF_B (6ULL << 40)
#define BUFS_B (7ULL << 40)
#define STARTS_B (8ULL << 40)
#define NEXT_B (9ULL << 40)
#define POS_B (10ULL << 40)
#define FIN_B (11ULL << 40)
#define ARENA_B (12ULL << 40)
#define OUT2_B (13ULL << 40)
#define REC2_B (14ULL << 40)
#define OUT_B (15ULL << 40)
// T-vector layout: 4-byte entries (master), or compact30 (30 rows a 64-byte line)
static int compact30 = 0;
static inline uint64_t tt_addr(uint32_t row) {
  if (compact30) return TT_B + (uint64_t)(row / 30) * 64 + (row % 30) * 2;
  return TT_B + (uint64_t)row * 4;
}

static void random_rows(uint32_t n, uint32_t m, uint32_t *out) {
  uint64_t x = 0x2545f4914f6cdd1dULL ^ (uint64_t)n;
  for (uint32_t i = 0; i < m; i++) { x ^= x << 13; x ^= x >> 7; x ^= x << 17; out[i] = (uint32_t)(x % n); }
}
static int cmpu(const void *a, const void *b) { uint32_t x = *(const uint32_t *)a, y = *(const uint32_t *)b; return x < y ? -1 : x > y; }
static uint32_t make_starts(uint32_t n, uint32_t segs, uint32_t r0, uint32_t *s) {
  random_rows(n, segs - 1, s);
  s[segs - 1] = r0;
  qsort(s, segs, 4, cmpu);
  uint32_t m = 0;
  for (uint32_t i = 0; i < segs; i++) if (m == 0 || s[i] != s[m - 1]) s[m++] = s[i];
  return m;
}
static uint64_t splitmix(uint64_t x) {
  x += 0x9e3779b97f4a7c15ULL; x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9ULL; x = (x ^ (x >> 27)) * 0x94d049bb133111ebULL;
  return x ^ (x >> 31);
}

// segment outputs, for the byte-identity check: each segment's bytes and successor
typedef struct { uint8_t **b; uint32_t *len, *cap; uint32_t *next; uint32_t ns; } Segs;
static void segs_init(Segs *g, uint32_t ns) {
  g->ns = ns; g->b = calloc(ns, sizeof(uint8_t *)); g->len = calloc(ns, 4); g->cap = calloc(ns, 4);
  g->next = malloc(4 * ns);
  for (uint32_t i = 0; i < ns; i++) g->next[i] = UINT32_MAX;
}
static inline void seg_put(Segs *g, uint32_t s, uint8_t c) {
  if (g->len[s] == g->cap[s]) { g->cap[s] = g->cap[s] ? 2 * g->cap[s] : 16; g->b[s] = realloc(g->b[s], g->cap[s]); }
  g->b[s][g->len[s]++] = c;
}
static uint32_t seg_of(const uint32_t *starts, uint32_t ns, uint32_t row) {
  uint32_t lo = 0, hi = ns;
  while (lo < hi) { uint32_t m = (lo + hi) / 2; if (starts[m] < row) lo = m + 1; else hi = m; }
  return (lo < ns && starts[lo] == row) ? lo : UINT32_MAX;
}
static int check_join(const Blk *b, Segs *g, const uint32_t *starts, uint32_t ns, uint32_t r0) {
  // the plain walk
  uint32_t p = b->t[b->orig] & LINK;
  uint32_t seg = seg_of(starts, ns, r0), pos = 0;
  uint32_t k = 0, out = 0;
  (void)p;
  // walk from orig: bytes are t[row] low byte, row = t[row] >> 8, starting at r0
  uint32_t row = r0;
  while (out < b->n) {
    if (seg == UINT32_MAX) return 0;
    if (pos == g->len[seg]) { seg = g->next[seg]; pos = 0; k++; continue; }
    uint32_t q = b->t[row] & LINK;
    if ((uint8_t)q != g->b[seg][pos]) return 0;
    row = q >> 8; pos++; out++;
  }
  return 1;
}
static void segs_free(Segs *g) {
  for (uint32_t i = 0; i < g->ns; i++) free(g->b[i]);
  free(g->b); free(g->len); free(g->cap); free(g->next);
}

// ------------------------------------------------------------------ master's room walk
// T-vector accesses in the room walk's order: L lanes, `segs` segments from random_rows
// plus r0, sorted; a take reads its start entry; each offset steps every lane in order; a
// lane on a START is handed over after the offset, in lane order; then the tail.
static void gen_master(Blk *b, int L, uint32_t segs, Ev *v, int *ok) {
  uint32_t n = b->n, *t = b->t;
  uint32_t r0 = t[b->orig] >> 8;
  uint32_t *st = malloc(4 * segs);
  uint32_t ns = make_starts(n, segs, r0, st);
  for (uint32_t i = 0; i < ns; i++) { t[st[i]] |= START; push(v, C_FLAG, tt_addr(st[i]), 0); }
  Segs g; segs_init(&g, ns);
  // take order: sorted (master) or, with PW_MASTER_SHUFFLE set, a fixed random permutation
  uint32_t *ord = malloc(4 * ns);
  for (uint32_t i = 0; i < ns; i++) ord[i] = i;
  if (getenv("PW_MASTER_SHUFFLE")) for (uint32_t i = ns - 1; i > 0; i--) { uint32_t j = splitmix(i * 7919ULL) % (i + 1); uint32_t x = ord[i]; ord[i] = ord[j]; ord[j] = x; }
  uint32_t row[64], seg[64], queued = 0; int active = 0;
  while (active < L && queued < ns) {
    uint32_t sg = ord[queued]; uint32_t q = t[st[sg]] & LINK; push(v, C_TT, tt_addr(st[sg]), 1);
    seg_put(&g, sg, q); row[active] = q >> 8; seg[active] = sg; queued++; active++;
  }
  int live[64];
  for (int k = 0; k < active; k++) live[k] = 1;
  if (active == L) {
    for (;;) {
      uint64_t marked = 0;
      for (int k = 0; k < L; k++) {
        uint32_t q = t[row[k]]; push(v, C_TT, tt_addr(row[k]), 1);
        if (q & START) marked |= 1ULL << k;
        else { seg_put(&g, seg[k], q); row[k] = q >> 8; }
      }
      int stop = 0;
      while (marked) {
        int k = __builtin_ctzll(marked); marked &= marked - 1;
        g.next[seg[k]] = seg_of(st, ns, row[k]);
        if (queued == ns) { live[k] = 0; stop = 1; continue; }
        uint32_t sg = ord[queued]; uint32_t q = t[st[sg]] & LINK; push(v, C_TT, tt_addr(st[sg]), 1);
        seg_put(&g, sg, q); row[k] = q >> 8; seg[k] = sg; queued++;
      }
      if (stop) break;
    }
  }
  // tail: the live lanes round robin until each reads a START
  int any = 1;
  while (any) {
    any = 0;
    for (int k = 0; k < active; k++) {
      if (!live[k]) continue;
      uint32_t q = t[row[k]]; push(v, C_TT, tt_addr(row[k]), 1);
      if (q & START) { g.next[seg[k]] = seg_of(st, ns, row[k]); live[k] = 0; continue; }
      any = 1; seg_put(&g, seg[k], q); row[k] = q >> 8;
    }
  }
  for (uint32_t i = 0; i < ns; i++) { t[st[i]] &= LINK; push(v, C_FLAG, tt_addr(st[i]), 0); }
  *ok = check_join(b, &g, st, ns, r0);
  segs_free(&g); free(st); free(ord);
}

// ------------------------------------------------------------------ the partitioned walk
typedef struct {
  uint32_t segs; int lanes, P;
  int shuffle;      // takes in a random order instead of sorted (kills start-order locality)
  int esz;          // queue entry bytes (4, 8 or 16)
  int logb;         // bytes of log per output byte (0: bytes carried in the entry, A's acc)
  int build_flags;  // 1: START flags set by the build (no flag pass, no clear pass)
} PCfg;
typedef struct { uint64_t steps, visits, cold, seg_ends; double live_sum; } PStat;

// FIFO queues as chunk lists over one pool of CH-entry chunks (live footprint)
#define CH 64
typedef struct { uint32_t row, lane; uint8_t first; } QE;
static void gen_part(Blk *b, PCfg c, Ev *v, PStat *ps, int *ok) {
  uint32_t n = b->n, *t = b->t;
  uint32_t r0 = t[b->orig] >> 8;
  uint32_t *st = malloc(4 * c.segs);
  uint32_t ns = make_starts(n, c.segs, r0, st);
  for (uint32_t i = 0; i < ns; i++) { t[st[i]] |= START; if (!c.build_flags) push(v, C_FLAG, tt_addr(st[i]), 0); }
  uint32_t *order = malloc(4 * ns);
  for (uint32_t i = 0; i < ns; i++) order[i] = i;
  if (c.shuffle) for (uint32_t i = ns - 1; i > 0; i--) { uint32_t j = splitmix(i * 7919ULL) % (i + 1); uint32_t x = order[i]; order[i] = order[j]; order[j] = x; }
  Segs g; segs_init(&g, ns);
  int P = c.P;
  // queues: per region a growable ring of QE (simulation state) and an address stream
  QE **qd = calloc(P, sizeof(QE *)); size_t *qh = calloc(P, sizeof(size_t)), *qt = calloc(P, sizeof(size_t)), *qc = calloc(P, sizeof(size_t));
  // pooled chunk addresses
  size_t maxchunks = ns / CH + (size_t)c.lanes / CH + 4 * P + 16;
  uint32_t *freel = malloc(4 * maxchunks); size_t nfree = 0; uint32_t nextchunk = 0;
  uint32_t **chq = calloc(P, sizeof(uint32_t *)); size_t *chh = calloc(P, sizeof(size_t)), *cht = calloc(P, sizeof(size_t)), *chc = calloc(P, sizeof(size_t));
  uint64_t *qpush = calloc(P, 8), *qpop = calloc(P, 8);
  uint32_t *lseg = malloc(4 * (size_t)c.lanes);
  uint32_t *epoch = calloc(n / 16 + 2, 4); uint32_t vis = 0;
  uint64_t logpos = 0;
  long live = 0;
#define REGION(row) ((int)(((uint64_t)(row) * (uint64_t)P) / n))
#define ENQ(rg, R, K, F) do { int r_ = (rg); \
    if (qpush[r_] % CH == 0) { uint32_t id = nfree ? freel[--nfree] : nextchunk++; \
      if (cht[r_] - chh[r_] == chc[r_]) { size_t nc = chc[r_] ? 2 * chc[r_] : 8; uint32_t *nw = malloc(4 * nc); \
        for (size_t i_ = 0; i_ < cht[r_] - chh[r_]; i_++) nw[i_] = chq[r_][(chh[r_] + i_) % (chc[r_] ? chc[r_] : 1)]; \
        cht[r_] -= chh[r_]; chh[r_] = 0; free(chq[r_]); chq[r_] = nw; chc[r_] = nc; } \
      chq[r_][cht[r_]++ % chc[r_]] = id; } \
    uint32_t cur_ = chq[r_][(cht[r_] - 1) % chc[r_]]; \
    push(v, C_Q, Q_B + ((uint64_t)cur_ * CH + qpush[r_] % CH) * c.esz, 0); qpush[r_]++; \
    if (qt[r_] - qh[r_] == qc[r_]) { size_t nc = qc[r_] ? 2 * qc[r_] : 64; QE *nw = malloc(sizeof(QE) * nc); \
      for (size_t i_ = 0; i_ < qt[r_] - qh[r_]; i_++) nw[i_] = qd[r_][(qh[r_] + i_) % (qc[r_] ? qc[r_] : 1)]; \
      qt[r_] -= qh[r_]; qh[r_] = 0; free(qd[r_]); qd[r_] = nw; qc[r_] = nc; } \
    qd[r_][qt[r_]++ % qc[r_]] = (QE){(R), (K), (F)}; } while (0)
  uint32_t queued = 0; int nl = 0;
  while (nl < c.lanes && queued < ns) {
    uint32_t s = order[queued++]; lseg[nl] = s;
    push(v, C_META, META_B + (uint64_t)s * 4, 0);
    ENQ(REGION(st[s]), st[s], nl, 1); nl++; live++;
  }
  int r = 0;
  for (;;) {
    int found = -1;
    for (int d = 0; d < P; d++) { int x = (r + d) % P; if (qt[x] != qh[x]) { found = x; break; } }
    if (found < 0) break;
    r = found; ps->visits++; vis++; ps->live_sum += live;
    while (qt[r] != qh[r]) {
      QE e = qd[r][qh[r]++ % qc[r]];
      uint32_t cur = chq[r][chh[r] % chc[r]];
      push(v, C_Q, Q_B + ((uint64_t)cur * CH + qpop[r] % CH) * c.esz, 0); qpop[r]++;
      if (qpop[r] % CH == 0) { freel[nfree++] = chq[r][chh[r]++ % chc[r]]; }
      uint32_t q = t[e.row]; push(v, C_TT, tt_addr(e.row), 1); ps->steps++;
      uint32_t ln = e.row >> 4;
      if (epoch[ln] != vis) { epoch[ln] = vis; ps->cold++; }
      uint32_t k = e.lane;
      if ((q & START) && !e.first) {
        uint32_t s = lseg[k];
        g.next[s] = seg_of(st, ns, e.row);   // deferred: one 4-byte write of the row hit
        push(v, C_META, META_B + (1ULL << 32) + (uint64_t)s * 4, 0); ps->seg_ends++;
        if (queued < ns) {
          uint32_t s2 = order[queued++]; lseg[k] = s2;
          push(v, C_META, META_B + (uint64_t)s2 * 4, 0);
          ENQ(REGION(st[s2]), st[s2], k, 1);
        } else live--;
        continue;
      }
      q &= LINK;
      seg_put(&g, lseg[k], (uint8_t)q);
      if (c.logb) { push(v, C_LOG, LOG_B + logpos * c.logb, 0); logpos++; }
      uint32_t nr = q >> 8;
      ENQ(REGION(nr), nr, k, 0);
    }
    r = (r + 1) % P;
  }
  if (!c.build_flags) for (uint32_t i = 0; i < ns; i++) push(v, C_FLAG, tt_addr(st[i]), 0);
  for (uint32_t i = 0; i < ns; i++) t[st[i]] &= LINK;
  *ok = check_join(b, &g, st, ns, r0);
  for (int i = 0; i < P; i++) { free(qd[i]); free(chq[i]); }
  free(qd); free(qh); free(qt); free(qc); free(freel); free(chq); free(chh); free(cht); free(chc);
  free(qpush); free(qpop); free(lseg); free(epoch); free(order); segs_free(&g); free(st);
}

// ------------------------------------------------------------------ full models (segcost)
// one event per distinct 64-byte line of [a, a + len); `step` marks each line a step
static void lines(Ev *v, int cls, uint64_t a, uint64_t len, int step) {
  if (!len) return;
  for (uint64_t l = a >> 6; l <= (a + len - 1) >> 6; l++) push(v, cls, l << 6, step);
}
static uint8_t *plain_walk(const Blk *b) {
  uint8_t *o = malloc(b->n);
  uint32_t p = b->t[b->orig];
  for (uint32_t i = 0; i < b->n; i++) { p = b->t[p >> 8]; o[i] = (uint8_t)p; }
  return o;
}
typedef struct { uint64_t segs, tail_steps, offsets, bsearch, grows_needed; } FStat;

// master's room walk with every memory access it makes, at line granularity off the
// T-vector: the flag pass, the buffers' setup, the takes (header, start, entry, first
// byte), the staged steps (tile bytes), the flushes into each segment's own buffer (at a
// chunk's end and at a segment's end), the finishes (binary search, next), the tail, the
// flag clear, and the assembly (next, header, the buffer's lines, the output's lines)
#define TILE 256
static void gen_masterfull(Blk *b, int L, uint32_t segs, Ev *v, int *ok, FStat *fs) {
  uint32_t n = b->n, *t = b->t;
  uint32_t r0 = t[b->orig] >> 8;
  uint32_t *st = malloc(4 * segs);
  uint32_t ns = make_starts(n, segs, r0, st);
  for (uint32_t i = 0; i < ns; i++) { t[st[i]] |= START; push(v, C_FLAG, tt_addr(st[i]), 0); }
  uint64_t cap = 2ULL * n / ns + 64, stride = ((4 * cap + 63) / 64) * 64;
  // setup: each header cleared and reserved (24 B each), next resized (8 B each)
  lines(v, C_WMETA, BUFS_B, 24ULL * ns, 0);
  lines(v, C_WMETA, NEXT_B, 8ULL * ns, 0);
  uint8_t **buf = calloc(ns, sizeof(uint8_t *)); uint32_t *len = calloc(ns, 4), *nxt = malloc(4 * ns);
  for (uint32_t i = 0; i < ns; i++) { nxt[i] = UINT32_MAX; buf[i] = malloc(n + 1); }
  uint32_t row[64], seg[64], stt[64], queued = 0; int active = 0;
  uint8_t tile[64][TILE];
#define BUFA(s, off) (BUF_B + (uint64_t)(s) * stride + (off))
#define SEG_OF(rw, out) do { uint32_t lo_ = 0, hi_ = ns; while (lo_ < hi_) { uint32_t m_ = (lo_ + hi_) / 2; push(v, C_WMETA, STARTS_B + 4ULL * m_, 0); fs->bsearch++; if (st[m_] < (rw)) lo_ = m_ + 1; else hi_ = m_; } out = lo_; } while (0)
#define TAKE(k) do { uint32_t sg_ = queued++; push(v, C_WMETA, BUFS_B + 24ULL * sg_, 0); push(v, C_WMETA, STARTS_B + 4ULL * sg_, 0); \
    uint32_t q_ = t[st[sg_]] & LINK; push(v, C_TT, tt_addr(st[sg_]), 1); buf[sg_][0] = (uint8_t)q_; len[sg_] = 1; push(v, C_SEGW, BUFA(sg_, 0), 0); \
    row[k] = q_ >> 8; seg[k] = sg_; fs->segs++; } while (0)
#define FLUSH(k, from, to) do { uint32_t f_ = (from), e_ = (to); if (e_ > f_) { lines(v, C_TILE, TILE_B + (uint64_t)(k) * TILE + f_, e_ - f_, 0); \
    lines(v, C_SEGW, BUFA(seg[k], len[seg[k]]), e_ - f_, 0); memcpy(buf[seg[k]] + len[seg[k]], tile[k] + f_, e_ - f_); len[seg[k]] += e_ - f_; } } while (0)
#define FINISH(k) do { uint32_t so_; SEG_OF(row[k], so_); nxt[seg[k]] = so_; push(v, C_WMETA, NEXT_B + 8ULL * seg[k], 0); if (len[seg[k]] > cap) fs->grows_needed++; } while (0)
  while (active < L && queued < ns) { TAKE(active); active++; }
  int live[64];
  for (int k = 0; k < active; k++) live[k] = 1;
  int full_done = 0;
  if (active == L) {
    for (int k = 0; k < L; k++) stt[k] = 0;
    for (;;) {
      // a chunk of TILE offsets
      for (int k = 0; k < L; k++) stt[k] = 0;
      int j;
      for (j = 0; j < TILE; j++) {
        uint64_t marked = 0;
        fs->offsets++;
        for (int k = 0; k < L; k++) {
          uint32_t q = t[row[k]]; push(v, C_TT, tt_addr(row[k]), 1);
          if (q & START) marked |= 1ULL << k;
          else { tile[k][j] = (uint8_t)q; push(v, C_TILE, TILE_B + (uint64_t)k * TILE + j, 0); row[k] = q >> 8; }
        }
        while (marked) {
          int k = __builtin_ctzll(marked); marked &= marked - 1;
          FLUSH(k, stt[k], (uint32_t)j);
          FINISH(k);
          if (queued == ns) {
            // every other lane's bytes to j (or j + 1 if it stepped), then the tail
            for (int i = 0; i < L; i++) if (i != k) { int waiting = (marked >> i) & 1; FLUSH(i, stt[i], (uint32_t)j + (waiting ? 0 : 1)); stt[i] = TILE; }
            live[k] = 0; full_done = 1; break;
          }
          TAKE(k);
          stt[k] = j + 1;
        }
        if (full_done) break;
      }
      if (full_done) break;
      for (int k = 0; k < L; k++) { FLUSH(k, stt[k], TILE); }
    }
  }
  // the tail: the live lanes round robin, each writing straight into its buffer
  int any = 1;
  while (any) {
    any = 0;
    for (int k = 0; k < active; k++) {
      if (!live[k] || nxt[seg[k]] != UINT32_MAX) { live[k] = 0; continue; }
      uint32_t q = t[row[k]]; push(v, C_TT, tt_addr(row[k]), 1); fs->tail_steps++;
      if (q & START) { FINISH(k); live[k] = 0; continue; }
      any = 1; buf[seg[k]][len[seg[k]]] = (uint8_t)q; push(v, C_SEGW, BUFA(seg[k], len[seg[k]]), 0); len[seg[k]]++; row[k] = q >> 8;
    }
  }
  for (uint32_t i = 0; i < ns; i++) { t[st[i]] &= LINK; push(v, C_FLAG, tt_addr(st[i]), 0); }
  // the assembly
  uint8_t *out = malloc(n);
  uint32_t sg; SEG_OF(r0, sg);
  uint32_t o = 0;
  while (o < n) {
    push(v, C_JMETA, BUFS_B + 24ULL * sg, 1);
    uint32_t c = len[sg] < n - o ? len[sg] : n - o;
    lines(v, C_SEGR, BUFA(sg, 0), c, 0);
    lines(v, C_OUT, OUT_B + o, c, 1);
    memcpy(out + o, buf[sg], c); o += c;
    push(v, C_JMETA, NEXT_B + 8ULL * sg, 0);
    sg = nxt[sg];
    if (sg == UINT32_MAX && o < n) break;
  }
  uint8_t *pl = plain_walk(b);
  *ok = (o == n) && !memcmp(out, pl, n);
  free(pl); free(out);
  for (uint32_t i = 0; i < ns; i++) free(buf[i]);
  free(buf); free(len); free(nxt); free(st);
#undef BUFA
#undef SEG_OF
#undef TAKE
#undef FLUSH
#undef FINISH
}

// the lean path (segcost/drv/src/lean.rs), access for access: implicit starts (row & (K-1)
// == 0), link-tested steps, takes in id order, pos at take, fin at finish, the tile dumped
// whole into the arena per chunk; the join as a second-level walk (L2 lanes, level-2
// starts at ids & (K2-1) == 0 and the head), each visit reading fin, pos and the
// segment's arena lines and writing its lane's buffer; level 3 copies the level-2
// buffers into the output
static void gen_lean(Blk *b, int L, int bits, int k2bits, int L2, Ev *v, int *ok, FStat *fs) {
  uint32_t n = b->n, *t = b->t;
  uint32_t r0 = t[b->orig] >> 8;
  uint32_t mask = (1u << bits) - 1;
  uint32_t ns = (n + mask) >> bits;
  int head = (r0 & mask) != 0;
  uint32_t total = ns + head;
  uint32_t *pos = calloc(total, 4); uint64_t *fin = calloc(total, 8);
  uint64_t arena_cap = ((uint64_t)n / L / TILE + 4) * L * TILE + 16;
  uint8_t *arena = malloc(arena_cap);
  uint32_t taken = 0;
#define NEXT_ID(out) do { if (taken == total) out = UINT32_MAX; else { out = (taken < (uint32_t)head) ? ns : taken - head; taken++; } } while (0)
#define START_ROW(id) ((id) == ns ? r0 : ((id) << bits))
  uint32_t rows[64], cur[64]; uint64_t live = 0;
  for (int k = 0; k < L; k++) {
    uint32_t id; NEXT_ID(id);
    rows[k] = 0; cur[k] = 0;
    if (id != UINT32_MAX) { cur[k] = id; rows[k] = START_ROW(id); pos[id] = k; push(v, C_WMETA, POS_B + 8ULL * id, 0); live |= 1ULL << k; fs->segs++; }
  }
  uint8_t tile[64][TILE];
  uint64_t g = 0, chunk = 0;
  for (;;) {
    int j;
    for (j = 0; j < TILE; j++) {
      fs->offsets++;
      for (int k = 0; k < L; k++) {
        uint32_t q = t[rows[k]]; push(v, C_TT, tt_addr(rows[k]), 1);
        tile[k][j] = (uint8_t)q; push(v, C_TILE, TILE_B + (uint64_t)k * TILE + j, 0);
        rows[k] = q >> 8;
      }
      uint64_t m = 0;
      for (int k = 0; k < L; k++) m |= (uint64_t)((rows[k] & mask) == 0) << k;
      m &= live;
      while (m) {
        int k = __builtin_ctzll(m); m &= m - 1;
        uint64_t gj = g + j;
        fin[cur[k]] = ((uint64_t)(rows[k] >> bits) << 32) | gj; push(v, C_WMETA, POS_B + 8ULL * cur[k] + 4, 0);
        uint32_t id; NEXT_ID(id);
        if (id != UINT32_MAX) { cur[k] = id; rows[k] = START_ROW(id); pos[id] = (uint32_t)((gj + 1) << 5) | k; push(v, C_WMETA, POS_B + 8ULL * id, 0); fs->segs++; }
        else live &= ~(1ULL << k);
      }
      if (!live) { j++; break; }
    }
    // the tile into the arena, whole
    if ((chunk + 1) * L * TILE + 16 > arena_cap) { fprintf(stderr, "arena\n"); exit(1); }
    for (int k = 0; k < L; k++) memcpy(arena + chunk * L * TILE + (uint64_t)k * TILE, tile[k], TILE);
    lines(v, C_TILE, TILE_B, (uint64_t)L * TILE, 0);
    lines(v, C_SEGW, ARENA_B + chunk * L * TILE, (uint64_t)L * TILE, 0);
    chunk++; g += TILE;
    if (!live) break;
  }
  uint32_t first = head ? ns : (r0 >> bits);
  // level 2
  uint32_t k2mask = (1u << k2bits) - 1;
  uint32_t n2 = (total + k2mask) >> k2bits;
  int head2 = (first & k2mask) != 0;
  uint32_t total2 = n2 + head2;
  uint32_t (*rec2)[4] = calloc(total2, 16);
  uint8_t **o2 = malloc(L2 * sizeof(uint8_t *)); uint64_t *dp = calloc(L2, 8);
  for (int k = 0; k < L2; k++) o2[k] = malloc(n + 64);
  uint32_t tk2 = 0;
#define NEXT2(out) do { if (tk2 == total2) out = UINT32_MAX; else { out = (tk2 < (uint32_t)head2) ? n2 : tk2 - head2; tk2++; } } while (0)
#define SEG2(id2) ((id2) == n2 ? first : ((id2) << k2bits))
  uint32_t sgl[64], cur2[64]; uint64_t stl[64]; uint64_t live2 = 0;
  for (int k = 0; k < L2; k++) {
    uint32_t id2; NEXT2(id2);
    if (id2 != UINT32_MAX) { cur2[k] = id2; sgl[k] = SEG2(id2); stl[k] = 0; live2 |= 1ULL << k; }
  }
  while (live2) {
    for (int k = 0; k < L2; k++) {
      if (!((live2 >> k) & 1)) continue;
      uint32_t sg = sgl[k];
      uint64_t fv = fin[sg]; uint32_t pv = pos[sg];
      push(v, C_JMETA, POS_B + 8ULL * sg, 1);  // the one 8-byte record (pos | succ, len)
      uint32_t lk = pv & 31; uint64_t g0 = pv >> 5, g1 = fv & 0xffffffffULL, end = g1 + 1;
      uint64_t gg = g0;
      while (gg < end) {
        uint64_t run = TILE - gg % TILE; if (run > end - gg) run = end - gg;
        uint64_t src = (gg / TILE) * L * TILE + (uint64_t)lk * TILE + gg % TILE;
        uint64_t rr = (run + 15) & ~15ULL;
        lines(v, C_SEGR, ARENA_B + src, rr, 0);
        lines(v, C_J2W, OUT2_B + (uint64_t)k * (n + 64ULL) + dp[k], rr, 0);
        memcpy(o2[k] + dp[k], arena + src, run);
        dp[k] += run; gg += run;
      }
      uint32_t nx = (uint32_t)(fv >> 32);
      if ((nx & k2mask) == 0 || nx == first) {
        uint32_t nx2 = (nx == first) ? (head2 ? n2 : (nx >> k2bits)) : (nx >> k2bits);
        rec2[cur2[k]][0] = k; rec2[cur2[k]][1] = (uint32_t)stl[k]; rec2[cur2[k]][2] = (uint32_t)(dp[k] - stl[k]); rec2[cur2[k]][3] = nx2;
        push(v, C_JMETA, REC2_B + 16ULL * cur2[k], 0);
        uint32_t id2; NEXT2(id2);
        if (id2 != UINT32_MAX) { cur2[k] = id2; sgl[k] = SEG2(id2); stl[k] = dp[k]; }
        else live2 &= ~(1ULL << k);
      } else sgl[k] = nx;
    }
  }
  // level 3
  uint8_t *out = malloc(n);
  uint32_t id2 = head2 ? n2 : (first >> k2bits);
  uint64_t o = 0;
  while (o < n) {
    push(v, C_JMETA, REC2_B + 16ULL * id2, 1);
    uint32_t lk = rec2[id2][0], off = rec2[id2][1], ln = rec2[id2][2];
    uint64_t c = ln < n - o ? ln : n - o;
    lines(v, C_SEGR, OUT2_B + (uint64_t)lk * (n + 64ULL) + off, c, 0);
    lines(v, C_OUT, OUT_B + o, c, 1);
    memcpy(out + o, o2[lk] + off, c); o += c;
    id2 = rec2[id2][3];
  }
  uint8_t *pl = plain_walk(b);
  *ok = !memcmp(out, pl, n);
  free(pl); free(out);
  for (int k = 0; k < L2; k++) free(o2[k]);
  free(o2); free(dp); free(rec2); free(pos); free(fin); free(arena);
#undef NEXT_ID
#undef START_ROW
#undef NEXT2
#undef SEG2
}

// lean v2: the same walk, but each segment's bytes and its record in one 32-byte slot at
// id * 32 (succ | len << 20, then 28 bytes inline; a longer segment keeps 24 inline and
// an offset into an overflow area, appended in finish order), flushed from the tile at
// the chunk's end and at the segment's end. The join reads one slot (one line) a visit.
#define SLOT_B (16ULL << 40)
#define OVF_B (17ULL << 40)
static void gen_lean2(Blk *b, int L, int bits, int k2bits, int L2, Ev *v, int *ok, FStat *fs) {
  uint32_t n = b->n, *t = b->t;
  uint32_t r0 = t[b->orig] >> 8;
  uint32_t mask = (1u << bits) - 1;
  uint32_t ns = (n + mask) >> bits;
  int head = (r0 & mask) != 0;
  uint32_t total = ns + head;
  // per segment: its bytes (simulation state), succ
  uint8_t **sb = calloc(total, sizeof(uint8_t *)); uint32_t *sl = calloc(total, 4), *sc = calloc(total, 4), *succ = calloc(total, 4);
  uint64_t *ovf = calloc(total, 8); uint64_t ovfpos = 0;
  uint32_t taken = 0;
#define NEXT_ID(out) do { if (taken == total) out = UINT32_MAX; else { out = (taken < (uint32_t)head) ? ns : taken - head; taken++; } } while (0)
#define START_ROW(id) ((id) == ns ? r0 : ((id) << bits))
#define PUTB(id, c) do { if (sl[id] == sc[id]) { sc[id] = sc[id] ? 2 * sc[id] : 32; sb[id] = realloc(sb[id], sc[id]); } sb[id][sl[id]++] = (c); } while (0)
  // the address of byte i of segment id: inline below 28 (24 if it overflows: decided at
  // its finish, so a flush before then writes the inline area, as a real slot would)
#define SLOTW(id, from, to) do { uint32_t f_ = (from), e_ = (to); \
    if (f_ < 28) { uint32_t ee_ = e_ < 28 ? e_ : 28; lines(v, C_SEGW, SLOT_B + 32ULL * (id) + 4 + f_, ee_ - f_, 0); f_ = ee_; } \
    if (e_ > f_) { if (!ovf[id]) { ovf[id] = ovfpos + 1; ovfpos += 4096; } lines(v, C_SEGW, OVF_B + ovf[id] - 1 + (f_ - 28), e_ - f_, 0); } } while (0)
  uint32_t rows[64], cur[64], st[64]; uint64_t live = 0;
  for (int k = 0; k < L; k++) {
    uint32_t id; NEXT_ID(id);
    rows[k] = 0; cur[k] = 0;
    if (id != UINT32_MAX) { cur[k] = id; rows[k] = START_ROW(id); live |= 1ULL << k; fs->segs++; }
  }
  uint8_t tile[64][TILE];
  for (;;) {
    for (int k = 0; k < L; k++) st[k] = 0;
    int j;
    for (j = 0; j < TILE; j++) {
      fs->offsets++;
      for (int k = 0; k < L; k++) {
        uint32_t q = t[rows[k]]; push(v, C_TT, tt_addr(rows[k]), 1);
        tile[k][j] = (uint8_t)q; push(v, C_TILE, TILE_B + (uint64_t)k * TILE + j, 0);
        if ((live >> k) & 1) PUTB(cur[k], (uint8_t)q);
        rows[k] = q >> 8;
      }
      uint64_t m = 0;
      for (int k = 0; k < L; k++) m |= (uint64_t)((rows[k] & mask) == 0) << k;
      m &= live;
      while (m) {
        int k = __builtin_ctzll(m); m &= m - 1;
        uint32_t s0 = cur[k];
        // the segment's bytes from st[k] to j into its slot, then its record word
        uint32_t len = sl[s0], had = len - (j + 1 - st[k]);
        lines(v, C_TILE, TILE_B + (uint64_t)k * TILE + st[k], j + 1 - st[k], 0);
        SLOTW(s0, had, len);
        succ[s0] = rows[k] >> bits; push(v, C_WMETA, SLOT_B + 32ULL * s0, 0);
        uint32_t id; NEXT_ID(id);
        if (id != UINT32_MAX) { cur[k] = id; rows[k] = START_ROW(id); st[k] = j + 1; fs->segs++; }
        else { live &= ~(1ULL << k); }
      }
      if (!live) { j++; break; }
    }
    // the chunk's end: each live lane's partial segment into its slot
    for (int k = 0; k < L; k++) if ((live >> k) & 1) {
      uint32_t s0 = cur[k], len = sl[s0], part = (uint32_t)j - st[k];
      if (part) { lines(v, C_TILE, TILE_B + (uint64_t)k * TILE + st[k], part, 0); SLOTW(s0, len - part, len); }
    }
    if (!live) break;
  }
  uint32_t first = head ? ns : (r0 >> bits);
  uint32_t k2mask = (1u << k2bits) - 1;
  uint32_t n2 = (total + k2mask) >> k2bits;
  int head2 = (first & k2mask) != 0;
  uint32_t total2 = n2 + head2;
  uint32_t (*rec2)[4] = calloc(total2, 16);
  uint8_t **o2 = malloc(L2 * sizeof(uint8_t *)); uint64_t *dp = calloc(L2, 8);
  for (int k = 0; k < L2; k++) o2[k] = malloc(n + 64);
  uint32_t tk2 = 0;
#define NEXT2(out) do { if (tk2 == total2) out = UINT32_MAX; else { out = (tk2 < (uint32_t)head2) ? n2 : tk2 - head2; tk2++; } } while (0)
#define SEG2(id2) ((id2) == n2 ? first : ((id2) << k2bits))
  uint32_t sgl[64], cur2[64]; uint64_t stl[64]; uint64_t live2 = 0;
  for (int k = 0; k < L2; k++) {
    uint32_t id2; NEXT2(id2);
    if (id2 != UINT32_MAX) { cur2[k] = id2; sgl[k] = SEG2(id2); stl[k] = 0; live2 |= 1ULL << k; }
  }
  while (live2) {
    for (int k = 0; k < L2; k++) {
      if (!((live2 >> k) & 1)) continue;
      uint32_t sg = sgl[k], len = sl[sg];
      push(v, C_JMETA, SLOT_B + 32ULL * sg, 1);  // the slot: record and inline bytes, one line
      if (len > 28) lines(v, C_SEGR, OVF_B + ovf[sg] - 1, len - 24, 0);
      lines(v, C_J2W, OUT2_B + (uint64_t)k * (n + 64ULL) + dp[k], (len + 15) & ~15u, 0);
      memcpy(o2[k] + dp[k], sb[sg], len); dp[k] += len;
      uint32_t nx = succ[sg];
      if ((nx & k2mask) == 0 || nx == first) {
        uint32_t nx2 = (nx == first) ? (head2 ? n2 : (nx >> k2bits)) : (nx >> k2bits);
        rec2[cur2[k]][0] = k; rec2[cur2[k]][1] = (uint32_t)stl[k]; rec2[cur2[k]][2] = (uint32_t)(dp[k] - stl[k]); rec2[cur2[k]][3] = nx2;
        push(v, C_JMETA, REC2_B + 16ULL * cur2[k], 0);
        uint32_t id2; NEXT2(id2);
        if (id2 != UINT32_MAX) { cur2[k] = id2; sgl[k] = SEG2(id2); stl[k] = dp[k]; }
        else live2 &= ~(1ULL << k);
      } else sgl[k] = nx;
    }
  }
  uint8_t *out = malloc(n);
  uint32_t id2 = head2 ? n2 : (first >> k2bits);
  uint64_t o = 0;
  while (o < n) {
    push(v, C_JMETA, REC2_B + 16ULL * id2, 1);
    uint32_t lk = rec2[id2][0], off = rec2[id2][1], ln = rec2[id2][2];
    uint64_t c = ln < n - o ? ln : n - o;
    lines(v, C_SEGR, OUT2_B + (uint64_t)lk * (n + 64ULL) + off, c, 0);
    lines(v, C_OUT, OUT_B + o, c, 1);
    memcpy(out + o, o2[lk] + off, c); o += c;
    id2 = rec2[id2][3];
  }
  uint8_t *pl = plain_walk(b);
  *ok = !memcmp(out, pl, n);
  free(pl); free(out);
  for (int k = 0; k < L2; k++) free(o2[k]);
  for (uint32_t i = 0; i < total; i++) free(sb[i]);
  free(o2); free(dp); free(rec2); free(sb); free(sl); free(sc); free(succ); free(ovf);
#undef NEXT_ID
#undef START_ROW
#undef PUTB
#undef SLOTW
#undef NEXT2
#undef SEG2
}

// ------------------------------------------------------------------ the cache model
typedef struct { uint64_t sets; int ways; uint64_t *tag, *stamp, clk; } Cache;
static Cache mk(uint64_t bytes, int ways) {
  Cache c; c.ways = ways; c.sets = bytes / 64 / ways; c.clk = 0;
  c.tag = malloc(8 * c.sets * ways); c.stamp = calloc(c.sets * ways, 8);
  for (uint64_t i = 0; i < c.sets * ways; i++) c.tag[i] = UINT64_MAX;
  return c;
}
static void freec(Cache *c) { free(c->tag); free(c->stamp); }
static inline int acc(Cache *c, uint64_t line) {
  uint64_t s = line % c->sets; uint64_t *tg = c->tag + s * c->ways, *sp = c->stamp + s * c->ways;
  c->clk++;
  int lru = 0;
  for (int w = 0; w < c->ways; w++) {
    if (tg[w] == line) { sp[w] = c->clk; return 1; }
    if (sp[w] < sp[lru]) lru = w;
  }
  tg[lru] = line; sp[lru] = c->clk;
  return 0;
}
// virtual 4 KiB page -> pseudo-random physical frame (per walk instance)
static inline uint64_t phys(uint64_t addr, int inst) {
  uint64_t page = addr >> 12;
  uint64_t frame = splitmix(page * 1315423911ULL + (uint64_t)inst * 0x9e37ULL) & ((1ULL << 34) - 1);
  return (frame << 6) | ((addr >> 6) & 63);
}
typedef struct { uint64_t acc[NC], l1[NC], l2[NC]; } Stat;
// several walks on one core (SMT siblings), interleaved a step at a time
static void simulate(Ev *w, int nw, uint64_t l1b, int l1w, uint64_t l2b, int l2w, Stat *out) {
  Cache L1 = mk(l1b, l1w), L2 = mk(l2b, l2w);
  size_t pos[4] = {0};
  memset(out, 0, sizeof(Stat) * nw);
  int alive = nw;
  while (alive) {
    alive = 0;
    for (int i = 0; i < nw; i++) {
      // events of walk i up to and including its next step
      while (pos[i] < w[i].n) {
        uint64_t e = w[i].e[pos[i]++];
        int cls = e >> 60, step = (e >> 59) & 1;
        uint64_t a = e & ((1ULL << 48) - 1);
        uint64_t pl = phys(a, i);
        out[i].acc[cls]++;
        if (!acc(&L1, pl)) { out[i].l1[cls]++; if (!acc(&L2, pl)) out[i].l2[cls]++; }
        if (step) break;
      }
      if (pos[i] < w[i].n) alive = 1;
    }
  }
  freec(&L1); freec(&L2);
}

static void report(const char *label, Stat *s, uint32_t n, PStat *ps) {
  uint64_t l2all = 0, l1all = 0;
  for (int c = 0; c < NC; c++) { l2all += s->l2[c]; l1all += s->l1[c]; }
  printf("%-44s L1 %.3f  L2 all %.3f  tt %.3f | q %.3f log %.3f meta %.3f flag %.3f | ttloads %.3f",
         label, (double)l1all / n, (double)l2all / n, (double)s->l2[C_TT] / n, (double)s->l2[C_Q] / n,
         (double)s->l2[C_LOG] / n, (double)s->l2[C_META] / n, (double)s->l2[C_FLAG] / n, (double)s->acc[C_TT] / n);
  if (ps) printf(" | cold %.3f spv %.1f live %.0f", (double)ps->cold / n, (double)ps->steps / ps->visits, ps->live_sum / ps->visits);
  printf("\n");
}

static void reportf(const char *label, Stat *s, uint32_t n, FStat *fs) {
  uint64_t l2all = 0, l1all = 0, acc = 0;
  for (int c = 0; c < NC; c++) { l2all += s->l2[c]; l1all += s->l1[c]; acc += s->acc[c]; }
  printf("%-40s segs %6lu | L2/B all %.4f", label, (unsigned long)fs->segs, (double)l2all / n);
  for (int c = 0; c < NC; c++) if (s->acc[c]) printf(" %s %.4f", CN[c], (double)s->l2[c] / n);
  printf(" | L1/B %.4f | ttloads/B %.4f | L2 miss/seg %.3f | offsets %lu bsearch/seg %.1f grows %lu\n", (double)l1all / n,
         (double)s->acc[C_TT] / n, fs->segs ? (double)l2all / fs->segs : 0, (unsigned long)fs->offsets,
         fs->segs ? (double)fs->bsearch / fs->segs : 0, (unsigned long)fs->grows_needed);
}

int main(int argc, char **argv) {
  if (argc < 3) { fprintf(stderr, "pw CFG FILE... (see source)\n"); return 1; }
  // argv[1]: "L2KB,ways,L1KB,l1ways"; then a list of jobs on stdin
  uint64_t l2kb, l1kb; int l2w, l1w;
  sscanf(argv[1], "%lu,%d,%lu,%d", &l2kb, &l2w, &l1kb, &l1w);
  int nb = argc - 2;
  Blk bl[4];
  for (int i = 0; i < nb && i < 4; i++) bl[i] = load(argv[2 + i]);
  char line[512];
  // job: "master L SEGS" | "part SEGS LANES P SHUFFLE ESZ LOGB BUILDFLAGS" ; prefix "c30 " for compact30
  // all blocks given are run on one core together (SMT) when nb > 1
  while (fgets(line, sizeof line, stdin)) {
    if (line[0] == '#' || line[0] == '\n') continue;
    char kind[16]; compact30 = 0;
    char *p = line;
    if (!strncmp(p, "c30 ", 4)) { compact30 = 1; p += 4; }
    sscanf(p, "%15s", kind);
    Ev ev[4] = {{0}}; PStat ps[4] = {{0}}; Stat st[4]; int ok[4];
    char label[128];
    if (!strcmp(kind, "mfull") || !strcmp(kind, "lean") || !strcmp(kind, "lean2")) {
      FStat fs[4] = {{0}};
      if (!strcmp(kind, "mfull")) {
        int L; uint32_t S; sscanf(p, "%*s %d %u", &L, &S);
        for (int i = 0; i < nb; i++) gen_masterfull(&bl[i], L, S, &ev[i], &ok[i], &fs[i]);
        snprintf(label, sizeof label, "%smfull L%d S%u", compact30 ? "c30 " : "", L, S);
      } else {
        int L, bits, k2, L2; sscanf(p, "%*s %d %d %d %d", &L, &bits, &k2, &L2);
        int v2 = !strcmp(kind, "lean2");
        for (int i = 0; i < nb; i++) { if (v2) gen_lean2(&bl[i], L, bits, k2, L2, &ev[i], &ok[i], &fs[i]); else gen_lean(&bl[i], L, bits, k2, L2, &ev[i], &ok[i], &fs[i]); }
        snprintf(label, sizeof label, "%s%s L%d K%d K2_%d L2_%d", compact30 ? "c30 " : "", kind, L, 1 << bits, 1 << k2, L2);
      }
      simulate(ev, nb, l1kb * 1024, l1w, l2kb * 1024, l2w, st);
      for (int i = 0; i < nb; i++) { char lb[160]; snprintf(lb, sizeof lb, "%s [b%d]%s", label, i, ok[i] ? "" : " BAD"); reportf(lb, &st[i], bl[i].n, &fs[i]); }
    } else if (!strcmp(kind, "master")) {
      int L; uint32_t S; sscanf(p, "%*s %d %u", &L, &S);
      for (int i = 0; i < nb; i++) gen_master(&bl[i], L, S, &ev[i], &ok[i]);
      snprintf(label, sizeof label, "%smaster L%d S%u", compact30 ? "c30 " : "", L, S);
      simulate(ev, nb, l1kb * 1024, l1w, l2kb * 1024, l2w, st);
      for (int i = 0; i < nb; i++) { char lb[160]; snprintf(lb, sizeof lb, "%s [b%d]%s", label, i, ok[i] ? "" : " BAD"); report(lb, &st[i], bl[i].n, NULL); }
    } else {
      PCfg c; sscanf(p, "%*s %u %d %d %d %d %d %d", &c.segs, &c.lanes, &c.P, &c.shuffle, &c.esz, &c.logb, &c.build_flags);
      for (int i = 0; i < nb; i++) gen_part(&bl[i], c, &ev[i], &ps[i], &ok[i]);
      snprintf(label, sizeof label, "%spart S%u L%d P%d%s e%d g%d%s", compact30 ? "c30 " : "", c.segs, c.lanes, c.P,
               c.shuffle ? " shuf" : "", c.esz, c.logb, c.build_flags ? " bf" : "");
      simulate(ev, nb, l1kb * 1024, l1w, l2kb * 1024, l2w, st);
      for (int i = 0; i < nb; i++) { char lb[160]; snprintf(lb, sizeof lb, "%s [b%d]%s", label, i, ok[i] ? "" : " BAD"); report(lb, &st[i], bl[i].n, &ps[i]); }
    }
    fflush(stdout);
    for (int i = 0; i < nb; i++) free(ev[i].e);
  }
  return 0;
}
