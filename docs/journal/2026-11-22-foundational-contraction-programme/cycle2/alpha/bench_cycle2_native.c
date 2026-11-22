/*
 * Cycle 2 Native C (-O3 -march=native) Benchmark & Exact Verifier on All 6 Wikidata Blocks:
 *   1. Escape B: W-IET Monotone-Polarity Interval-DAG (delta < 0 vs delta > 0, ~17 rounds,
 *                sumP ~ 2.6-3.6 r_0, Q ~ 0.96-1.10 m_16, ZERO qsort on D, sorted 2-way merge!)
 *                + Beta's Flat-Leaf (<= 16 B inline pool) + Memo-SLP Expansion!
 *   2. Compare Flat-Leaf (<= 16 B inline) Expansion vs Pure Memo-SLP Expansion directly
 *      against the i9-10900K bar (Open: 0.72-0.90 ns*n + 4.8-5.8 ns*m_16 = 8.2-10.1 ms / 6 blocks)!
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/* Compact Flat-Leaf (<= 16 B inline) + SLP Store!
 * If (word_id & 0x80000000u) == 0: it is a FLAT-LEAF index into flat_pool (16 bytes + 1 byte len)!
 * Wait: even simpler! Store ALL words in a unified struct or two pools:
 *   - inline_pool[id]: 16 bytes + len (id < 0x80000000u, or flag bit 31 = 1 means SLP node!)
 */
typedef struct {
    uint8_t bytes[16];
    uint8_t len;
} FlatWord;

typedef struct {
    uint32_t a;   /* child a (bit 31 of a in SlpNode? No, store type in separate field or bit 30!) */
    uint32_t b;   /* child b or power */
    uint32_t len; /* total byte length */
    uint8_t is_pow;
} SlpNode;

#define SLP_TAG 0x80000000u

typedef struct {
    FlatWord *flat;
    uint32_t flat_len, flat_cap;
    SlpNode *slp;
    uint32_t slp_len, slp_cap;
} HybridStore;

static void hs_init(HybridStore *h, uint32_t cap) {
    h->flat_cap = cap;
    h->flat = (FlatWord *)malloc((size_t)cap * sizeof(FlatWord));
    h->flat_len = 256;
    for (uint32_t c = 0; c < 256; c++) {
        h->flat[c].bytes[0] = (uint8_t)c;
        h->flat[c].len = 1;
    }
    h->slp_cap = cap;
    h->slp = (SlpNode *)malloc((size_t)cap * sizeof(SlpNode));
    h->slp_len = 0;
}

static void hs_free(HybridStore *h) {
    free(h->flat); free(h->slp);
}

static inline uint32_t hs_len(const HybridStore *h, uint32_t id) {
    return (id & SLP_TAG) ? h->slp[id & ~SLP_TAG].len : (uint32_t)h->flat[id].len;
}

static inline uint32_t hs_concat(HybridStore *h, uint32_t u, uint32_t v) {
    if (((u | v) & SLP_TAG) == 0) {
        uint8_t lu = h->flat[u].len;
        uint8_t lv = h->flat[v].len;
        if (lu + lv <= 16) {
            if (h->flat_len == h->flat_cap) {
                h->flat_cap *= 2;
                h->flat = (FlatWord *)realloc(h->flat, (size_t)h->flat_cap * sizeof(FlatWord));
            }
            uint32_t id = h->flat_len++;
            memcpy(h->flat[id].bytes, h->flat[u].bytes, lu);
            memcpy(h->flat[id].bytes + lu, h->flat[v].bytes, lv);
            h->flat[id].len = (uint8_t)(lu + lv);
            return id;
        }
    }
    if (h->slp_len == h->slp_cap) {
        h->slp_cap *= 2;
        h->slp = (SlpNode *)realloc(h->slp, (size_t)h->slp_cap * sizeof(SlpNode));
    }
    uint32_t sid = h->slp_len++;
    h->slp[sid].a = u;
    h->slp[sid].b = v;
    h->slp[sid].len = hs_len(h, u) + hs_len(h, v);
    h->slp[sid].is_pow = 0;
    return sid | SLP_TAG;
}

static inline uint32_t hs_power(HybridStore *h, uint32_t u, uint32_t k) {
    if (k == 1) return u;
    if ((u & SLP_TAG) == 0) {
        uint32_t lu = h->flat[u].len;
        if (lu * k <= 16) {
            if (h->flat_len == h->flat_cap) {
                h->flat_cap *= 2;
                h->flat = (FlatWord *)realloc(h->flat, (size_t)h->flat_cap * sizeof(FlatWord));
            }
            uint32_t id = h->flat_len++;
            for (uint32_t r = 0; r < k; r++)
                memcpy(h->flat[id].bytes + r * lu, h->flat[u].bytes, lu);
            h->flat[id].len = (uint8_t)(lu * k);
            return id;
        }
    }
    if (h->slp_len == h->slp_cap) {
        h->slp_cap *= 2;
        h->slp = (SlpNode *)realloc(h->slp, (size_t)h->slp_cap * sizeof(SlpNode));
    }
    uint32_t sid = h->slp_len++;
    h->slp[sid].a = u;
    h->slp[sid].b = k;
    h->slp[sid].len = hs_len(h, u) * k;
    h->slp[sid].is_pow = 1;
    return sid | SLP_TAG;
}

typedef struct {
    int32_t lo, hi, delta;
    uint32_t w;
    int32_t mark_off;
    uint8_t is_del;
} Piece;

typedef struct { Piece *data; int32_t len, cap; } PieceVec;
static inline void pv_init(PieceVec *v, int32_t cap) {
    v->len = 0; v->cap = cap > 16 ? cap : 16;
    v->data = (Piece *)malloc((size_t)v->cap * sizeof(Piece));
}
static inline void pv_push(PieceVec *v, Piece p) {
    if (v->len == v->cap) { v->cap *= 2; v->data = (Piece *)realloc(v->data, (size_t)v->cap * sizeof(Piece)); }
    v->data[v->len++] = p;
}
static inline void pv_free(PieceVec *v) { free(v->data); v->data = NULL; v->len = v->cap = 0; }

static inline int32_t combine_mark(int32_t m1, uint32_t len1, int32_t m2) {
    return (m1 >= 0) ? m1 : ((m2 >= 0) ? (int32_t)len1 + m2 : -1);
}

static inline int32_t find_piece(const Piece *arr, int32_t n, int32_t target) {
    int32_t l = 0, r = n;
    while (l + 1 < r) {
        int32_t m = l + ((r - l) >> 1);
        if (arr[m].lo <= target) l = m; else r = m;
    }
    return l;
}

typedef struct { int32_t a, b; } Range;
static inline int32_t find_range(const Range *arr, int32_t n, int32_t target) {
    if (n == 0) return 0;
    int32_t l = 0, r = n;
    while (l < r) {
        int32_t m = l + ((r - l) >> 1);
        if (arr[m].b <= target) l = m + 1; else r = m;
    }
    return l;
}

/* Fast radix sort on Piece by lo (3-pass 10-bit radix for lo < 2^30, or 2-pass 10-bit for n <= 1M!) */
static void radix_sort_pieces(Piece *a, Piece *tmp, int32_t n) {
    if (n <= 1) return;
    /* 2 passes of 10 bits covers [0, 2^20 = 1,048,576) — exact for all bzip2 blocks n <= 900,000! */
    int32_t cnt[1024];
    memset(cnt, 0, sizeof(cnt));
    for (int32_t i = 0; i < n; i++) cnt[a[i].lo & 1023]++;
    int32_t acc = 0;
    for (int i = 0; i < 1024; i++) { int32_t c = cnt[i]; cnt[i] = acc; acc += c; }
    for (int32_t i = 0; i < n; i++) tmp[cnt[a[i].lo & 1023]++] = a[i];

    memset(cnt, 0, sizeof(cnt));
    for (int32_t i = 0; i < n; i++) cnt[(tmp[i].lo >> 10) & 1023]++;
    acc = 0;
    for (int i = 0; i < 1024; i++) { int32_t c = cnt[i]; cnt[i] = acc; acc += c; }
    for (int32_t i = 0; i < n; i++) a[cnt[(tmp[i].lo >> 10) & 1023]++] = tmp[i];
}

/* Fast Flat-Leaf (<= 16 B) + Memo-SLP Expansion! */
typedef struct {
    uint32_t id;
    int32_t exit_pos;
} ExpFrame;

static void expand_hybrid_slp(
    const HybridStore *h, uint32_t root_id, uint8_t *out,
    int32_t *first_pos, ExpFrame *stack,
    int64_t *flat_emits, int64_t *memo_copies
) {
    int32_t sp = 0;
    int32_t cur = 0;
    int64_t fe = 0, mc = 0;
    stack[sp++] = (ExpFrame){ root_id, -1 };

    while (sp > 0) {
        ExpFrame f = stack[--sp];
        uint32_t id = f.id;
        if (f.exit_pos >= 0) {
            first_pos[id & ~SLP_TAG] = f.exit_pos;
            continue;
        }
        if ((id & SLP_TAG) == 0) {
            /* FLAT-LEAF (<= 16 B): single unaligned 16-byte copy, ZERO first_pos lookup! */
            const FlatWord *fw = &h->flat[id];
            memcpy(out + cur, fw->bytes, 16);
            cur += fw->len;
            fe++;
            continue;
        }
        uint32_t sid = id & ~SLP_TAG;
        int32_t fp = first_pos[sid];
        if (fp >= 0) {
            uint32_t ln = h->slp[sid].len;
            memcpy(out + cur, out + fp, ln);
            cur += (int32_t)ln;
            mc++;
            continue;
        }
        SlpNode nd = h->slp[sid];
        if (!nd.is_pow) {
            stack[sp++] = (ExpFrame){ id, cur };
            stack[sp++] = (ExpFrame){ nd.b, -1 };
            stack[sp++] = (ExpFrame){ nd.a, -1 };
        } else {
            /* Power node */
            uint32_t base = nd.a;
            uint32_t rep = nd.b;
            uint32_t blen = hs_len(h, base);
            int32_t sh = cur;
            int32_t save_sp = sp;
            stack[sp++] = (ExpFrame){ base, -1 };
            while (sp > save_sp) {
                ExpFrame g = stack[--sp];
                uint32_t gid = g.id;
                if (g.exit_pos >= 0) { first_pos[gid & ~SLP_TAG] = g.exit_pos; continue; }
                if ((gid & SLP_TAG) == 0) {
                    const FlatWord *fw = &h->flat[gid];
                    memcpy(out + cur, fw->bytes, 16);
                    cur += fw->len; fe++;
                } else {
                    uint32_t gsid = gid & ~SLP_TAG;
                    int32_t gfp = first_pos[gsid];
                    if (gfp >= 0) {
                        uint32_t gln = h->slp[gsid].len;
                        memcpy(out + cur, out + gfp, gln);
                        cur += (int32_t)gln; mc++;
                    } else {
                        SlpNode gnd = h->slp[gsid];
                        stack[sp++] = (ExpFrame){ gid, cur };
                        stack[sp++] = (ExpFrame){ gnd.b, -1 };
                        stack[sp++] = (ExpFrame){ gnd.a, -1 };
                    }
                }
            }
            uint32_t written = blen;
            uint32_t target = blen * rep;
            while (written < target) {
                uint32_t chunk = (written <= target - written) ? written : (target - written);
                memcpy(out + sh + written, out + sh, chunk);
                written += chunk; mc++;
            }
            cur = sh + (int32_t)target;
            first_pos[sid] = sh;
        }
    }
    *flat_emits = fe;
    *memo_copies = mc;
}

static double now_ms(void) {
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1e3 + (double)ts.tv_nsec * 1e-6;
}
static int cmp_dbl(const void *a, const void *b) {
    double da = *(const double *)a, db = *(const double *)b;
    return (da > db) - (da < db);
}

static void ref_ibwt(const uint8_t *L, int32_t n, int32_t p, uint8_t *out) {
    int32_t C[256] = {0};
    for (int32_t i = 0; i < n; i++) C[L[i]]++;
    int32_t acc = 0;
    for (int c = 0; c < 256; c++) { int32_t cnt = C[c]; C[c] = acc; acc += cnt; }
    int32_t *LF = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    int32_t cur[256]; memcpy(cur, C, sizeof(cur));
    for (int32_t i = 0; i < n; i++) LF[i] = cur[L[i]]++;
    int32_t x = p;
    for (int32_t k = n - 1; k >= 0; k--) { out[k] = L[x]; x = LF[x]; }
    free(LF);
}

typedef struct { uint8_t c; int32_t len; } RunRec;

int main(void) {
    const char *files[6] = {"block-0.L","block-2.L","block-85.L","block-143.L","block-210.L","block-207.L"};
    const int32_t ns[6] = {899931,900000,899935,899869,899959,899998};
    const int32_t ps[6] = {264045,320807,325088,802868,791429,205180};

    printf("=== NATIVE C BENCHMARK: W-IET MONOTONE INTERVAL-DAG + FLAT-LEAF (<=16B) + RADIX SORT ===\n");
    printf("block       |  r0   |  m16   | rnd | Q_rand (x m16) | flat_em  memo_cp | T_acq  T_con  T_exp  T_tot (ms) | exact\n");
    printf("------------+-------+--------+-----+----------------+------------------+----------------------------------+------\n");

    double sa=0, sc=0, se=0, st=0;
    int64_t sq=0, sfe=0, smc=0;

    for (int bi = 0; bi < 6; bi++) {
        char path[256]; snprintf(path, sizeof(path), "/tmp/mibz2-blocks/%s", files[bi]);
        FILE *fp = fopen(path, "rb"); int32_t n = ns[bi], p = ps[bi];
        uint8_t *L = (uint8_t *)malloc((size_t)n);
        fread(L, 1, (size_t)n, fp); fclose(fp);

        RunRec *runs = (RunRec *)malloc((size_t)n * sizeof(RunRec));
        int32_t r0 = 0, counts[256] = {0}, m16 = 0, p_thin = 0;
        for (int32_t i = 0; i < n; ) {
            uint8_t c = L[i]; int32_t j = i + 1;
            while (j < n && L[j] == c) j++;
            int32_t ln = j - i;
            runs[r0++] = (RunRec){c, ln};
            counts[c] += ln;
            if (ln <= 16) { m16 += ln; if (i <= p && p < j) p_thin = 1; }
            i = j;
        }
        if (!p_thin) m16++;

        uint8_t *ref_out = (uint8_t *)malloc((size_t)n);
        ref_ibwt(L, n, p, ref_out);
        free(L);

        uint8_t *cyc_buf = (uint8_t *)malloc((size_t)n + 32);
        uint8_t *out_buf = (uint8_t *)malloc((size_t)n + 32);

        double ta[5], tc[5], te[5], tt[5];
        int64_t last_q = 0, last_fe = 0, last_mc = 0;
        int last_rnd = 0, exact_ok = 1;

        for (int trial = 0; trial < 5; trial++) {
            double t0 = now_ms();

            /* Stage 1: Acquire initial pieces from I_0 (NO bucket splitting needed for delta-DAG!) */
            int32_t C[256], acc = 0;
            for (int c = 0; c < 256; c++) { C[c] = acc; acc += counts[c]; }
            int32_t cur_t[256]; memcpy(cur_t, C, sizeof(cur_t));

            PieceVec pieces, next_p, stack;
            pv_init(&pieces, r0 + 16);
            pv_init(&next_p, r0 + 16);
            pv_init(&stack, r0 + 16);
            Piece *radix_tmp = (Piece *)malloc((size_t)(r0 * 2 + 1024) * sizeof(Piece));
            Range *merged_d = (Range *)malloc((size_t)(r0 * 2 + 1024) * sizeof(Range));

            int32_t pos = 0;
            for (int32_t k = 0; k < r0; k++) {
                uint8_t c = runs[k].c; int32_t ln = runs[k].len;
                int32_t delta = cur_t[c] - pos; cur_t[c] += ln;
                int32_t hi = pos + ln;
                if (pos <= p && p < hi) {
                    if (pos < p) pv_push(&pieces, (Piece){pos, p, delta, c, -1, 0});
                    pv_push(&pieces, (Piece){p, p + 1, delta, c, 0, 0});
                    if (p + 1 < hi) pv_push(&pieces, (Piece){p + 1, hi, delta, c, -1, 0});
                } else {
                    pv_push(&pieces, (Piece){pos, hi, delta, c, -1, 0});
                }
                pos = hi;
            }
            HybridStore hs;
            hs_init(&hs, (uint32_t)(r0 * 4 + 1024));

            double t1 = now_ms();

            /* Stage 2: W-IET Monotone-Polarity Interval-DAG (delta < 0 vs delta > 0) */
            int has_pcyc = 0; uint32_t pcyc_w = 0; int32_t pcyc_m = -1;
            int rounds = 0; int64_t q_rand = 0;

            while (!has_pcyc) {
                int32_t np = pieces.len;
                int64_t w_down = 0, w_up = 0;
                int did_sp = 0;
                next_p.len = 0;
                for (int32_t i = 0; i < np; i++) {
                    Piece pc = pieces.data[i];
                    if (pc.delta == 0) {
                        if (pc.mark_off >= 0) { has_pcyc = 1; pcyc_w = pc.w; pcyc_m = pc.mark_off; }
                        continue;
                    }
                    int32_t L = pc.hi - pc.lo;
                    if (pc.delta > 0 && pc.delta < L) {
                        did_sp = 1;
                        int32_t d = pc.delta, q = L / d, rem = L % d;
                        if (rem > 0) pv_push(&next_p, (Piece){pc.lo, pc.lo + rem, (q + 1) * d, hs_power(&hs, pc.w, (uint32_t)(q + 1)), -1, 0});
                        if (d - rem > 0) pv_push(&next_p, (Piece){pc.lo + rem, pc.lo + d, q * d, hs_power(&hs, pc.w, (uint32_t)q), -1, 0});
                    } else if (pc.delta < 0 && (-pc.delta) < L) {
                        did_sp = 1;
                        int32_t s = -pc.delta, q = L / s, rem = L % s;
                        if (s - rem > 0) pv_push(&next_p, (Piece){pc.hi - s, pc.hi - rem, -q * s, hs_power(&hs, pc.w, (uint32_t)q), -1, 0});
                        if (rem > 0) pv_push(&next_p, (Piece){pc.hi - rem, pc.hi, -(q + 1) * s, hs_power(&hs, pc.w, (uint32_t)(q + 1)), -1, 0});
                    } else {
                        if (pc.delta < 0) w_down += L; else w_up += L;
                        pv_push(&next_p, pc);
                    }
                }
                if (has_pcyc) break;
                if (did_sp) {
                    PieceVec tmp = pieces; pieces = next_p; next_p = tmp;
                    radix_sort_pieces(pieces.data, radix_tmp, pieces.len);
                    continue;
                }
                if (w_down + w_up <= 1) break;
                rounds++;
                int del_down = (w_down >= w_up);

                /* Tag deleted pieces & build merged_d in ONE sequential scan (pieces is ALREADY sorted by lo!) */
                int32_t nm = 0;
                Piece *arr = next_p.data;
                int32_t ncur = next_p.len;
                for (int32_t i = 0; i < ncur; i++) {
                    int is_d = del_down ? (arr[i].delta < 0) : (arr[i].delta > 0);
                    arr[i].is_del = (uint8_t)is_d;
                    if (is_d) {
                        if (nm > 0 && merged_d[nm - 1].b == arr[i].lo) merged_d[nm - 1].b = arr[i].hi;
                        else merged_d[nm++] = (Range){arr[i].lo, arr[i].hi};
                    }
                }

                /* Partition retained pieces into direct-keep vs stack (hitting D) */
                pieces.len = 0;
                stack.len = 0;
                for (int32_t i = 0; i < ncur; i++) {
                    if (arr[i].is_del) continue;
                    Piece pc = arr[i];
                    int32_t t_lo = pc.lo + pc.delta, t_hi = pc.hi + pc.delta;
                    int32_t im = find_range(merged_d, nm, t_lo);
                    int32_t cur_pos = t_lo;
                    while (cur_pos < t_hi) {
                        if (im >= nm || merged_d[im].a >= t_hi) {
                            pv_push(&pieces, (Piece){cur_pos - pc.delta, t_hi - pc.delta, pc.delta, pc.w, pc.mark_off, 0});
                            cur_pos = t_hi;
                        } else {
                            int32_t da = merged_d[im].a, db = merged_d[im].b;
                            if (cur_pos < da) {
                                pv_push(&pieces, (Piece){cur_pos - pc.delta, da - pc.delta, pc.delta, pc.w, pc.mark_off, 0});
                                cur_pos = da;
                            }
                            int32_t ov = (t_hi < db) ? t_hi : db;
                            pv_push(&stack, (Piece){cur_pos - pc.delta, ov - pc.delta, pc.delta, pc.w, pc.mark_off, 0});
                            cur_pos = ov;
                            if (cur_pos == db) im++;
                        }
                    }
                }

                /* Resolve acyclic DAG hops! */
                while (stack.len > 0) {
                    Piece cur = stack.data[--stack.len];
                    int32_t t_lo = cur.lo + cur.delta, t_hi = cur.hi + cur.delta;
                    int32_t idx = find_piece(arr, ncur, t_lo);
                    int32_t cur_pos = t_lo;
                    uint32_t w_len = hs_len(&hs, cur.w);
                    while (cur_pos < t_hi) {
                        q_rand++;
                        const Piece *q = &arr[idx];
                        int32_t ov = (t_hi < q->hi) ? t_hi : q->hi;
                        int32_t slo = cur_pos - cur.delta, shi = ov - cur.delta;
                        int32_t nd = cur.delta + q->delta;
                        uint32_t nw = hs_concat(&hs, cur.w, q->w);
                        int32_t nm_off = combine_mark(cur.mark_off, w_len, q->mark_off);
                        int32_t nt_lo = slo + nd, nt_hi = shi + nd;
                        int32_t im = find_range(merged_d, nm, nt_lo);
                        int32_t npos = nt_lo;
                        while (npos < nt_hi) {
                            if (im >= nm || merged_d[im].a >= nt_hi) {
                                pv_push(&pieces, (Piece){npos - nd, nt_hi - nd, nd, nw, nm_off, 0});
                                npos = nt_hi;
                            } else {
                                int32_t da = merged_d[im].a, db = merged_d[im].b;
                                if (npos < da) {
                                    pv_push(&pieces, (Piece){npos - nd, da - nd, nd, nw, nm_off, 0});
                                    npos = da;
                                }
                                int32_t nov = (nt_hi < db) ? nt_hi : db;
                                pv_push(&stack, (Piece){npos - nd, nov - nd, nd, nw, nm_off, 0});
                                npos = nov;
                                if (npos == db) im++;
                            }
                        }
                        cur_pos = ov;
                        idx++;
                    }
                }

                radix_sort_pieces(pieces.data, radix_tmp, pieces.len);
            }

            double t2 = now_ms();

            /* Stage 3: Flat-Leaf (<= 16 B) + Memo-SLP Expansion + Reverse/Rotate */
            int32_t *first_pos = (int32_t *)malloc((size_t)(hs.slp_len + 1) * sizeof(int32_t));
            memset(first_pos, -1, (size_t)(hs.slp_len + 1) * sizeof(int32_t));
            ExpFrame *exp_stack = (ExpFrame *)malloc((size_t)(hs.slp_len * 2 + 1024) * sizeof(ExpFrame));
            int64_t fe = 0, mc = 0;
            expand_hybrid_slp(&hs, pcyc_w, cyc_buf, first_pos, exp_stack, &fe, &mc);

            int32_t q_len = (int32_t)hs_len(&hs, pcyc_w);
            int32_t dst = n - 1;
            for (int32_t k = pcyc_m; k < q_len && dst >= 0; k++) out_buf[dst--] = cyc_buf[k];
            for (int32_t k = 0; k < pcyc_m && dst >= 0; k++) out_buf[dst--] = cyc_buf[k];

            double t3 = now_ms();

            if (trial == 0) {
                if (memcmp(out_buf, ref_out, (size_t)n) != 0) exact_ok = 0;
                last_q = q_rand; last_fe = fe; last_mc = mc; last_rnd = rounds;
            }
            ta[trial] = t1 - t0; tc[trial] = t2 - t1; te[trial] = t3 - t2; tt[trial] = t3 - t0;

            free(first_pos); free(exp_stack);
            free(radix_tmp); free(merged_d);
            pv_free(&pieces); pv_free(&next_p); pv_free(&stack);
            hs_free(&hs);
        }

        qsort(ta, 5, sizeof(double), cmp_dbl);
        qsort(tc, 5, sizeof(double), cmp_dbl);
        qsort(te, 5, sizeof(double), cmp_dbl);
        qsort(tt, 5, sizeof(double), cmp_dbl);
        sa += ta[2]; sc += tc[2]; se += te[2]; st += tt[2];
        sq += last_q; sfe += last_fe; smc += last_mc;

        printf("%-11s | %5d | %6d | %3d | %6ld (%5.3fx) | %7ld  %7ld | %5.2f  %5.2f  %5.2f  %5.2f ms | %s\n",
               files[bi], r0, m16, last_rnd, (long)last_q, (double)last_q / m16,
               (long)last_fe, (long)last_mc, ta[2], tc[2], te[2], tt[2], exact_ok ? "PASS" : "FAIL");

        free(runs); free(ref_out); free(cyc_buf); free(out_buf);
    }
    printf("------------+-------+--------+-----+----------------+------------------+----------------------------------+------\n");
    printf("SUM 6 BLKS  |       | 895391 |     | %6ld (%5.3fx) | %7ld  %7ld | %5.2f  %5.2f  %5.2f  %5.2f ms |\n",
           (long)sq, (double)sq / 895391.0, (long)sfe, (long)smc, sa, sc, se, st);
    return 0;
}
