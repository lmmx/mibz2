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
enum { C_TT, C_Q, C_LOG, C_META, C_FLAG, NC };
static const char *CN[NC] = {"tt", "queue", "log", "meta", "flag"};
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
    if (!strcmp(kind, "master")) {
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
