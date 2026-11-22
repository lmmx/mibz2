/*
 * Cycle 6 — Task 1.3:
 * Packed 8-Byte u64 Native C Benchmark (`gcc -O3 -march=native`) of Thin-Only Static M1
 * (cap <= 8 B -> ZERO SLP, inline u64 word!) comparing w_min = 1 vs w_min = 2 across
 * ALL 6 authenticated Wikidata blocks!
 *
 * Packed 8-Byte Layout per strip (ONE single 8-B u64 load per R1 hop!):
 *   bits  0..31 : delta (int32_t)
 *   bits 32..55 : next_r1 (24-bit unsigned; 0xFFFFFF = none; supports up to 16.7M strips!)
 *   bits 56..63 : byte c (uint8_t)
 *   -> Total packed R1 array = 8 B * q0 = 400 KiB .. 1.00 MiB (vs 20 B * q0 in Cycle 5!)
 *   -> Output per core head: ONE u64 word (bytes packed in low bits) + u8 len (<= 8 B, ZERO SLP!)
 *
 * Strictly checksum-verified (serial vs K=8 MLP match identically, volatile sink -> zero DCE!).
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

typedef struct { uint8_t c; int32_t len; } RunRec;
typedef struct { int32_t lo, hi, delta; uint8_t c; } StripMeta;

#define NO_NEXT 0xFFFFFFu

static inline uint64_t pack_r1(int32_t delta, uint32_t nxt, uint8_t c) {
    return ((uint64_t)(uint32_t)delta) | ((uint64_t)(nxt & 0xFFFFFFu) << 32) | ((uint64_t)c << 56);
}
static inline int32_t  unpack_delta(uint64_t v) { return (int32_t)(uint32_t)v; }
static inline uint32_t unpack_next(uint64_t v)  { return (uint32_t)((v >> 32) & 0xFFFFFFu); }
static inline uint8_t  unpack_byte(uint64_t v)  { return (uint8_t)(v >> 56); }

static double now_ms(void) {
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1e3 + (double)ts.tv_nsec * 1e-6;
}
static int cmp_dbl(const void *a, const void *b) {
    double da = *(const double *)a, db = *(const double *)b;
    return (da > db) - (da < db);
}

static void bench_wmin(
    const char *files[6], const int32_t ns[6], const int32_t ps[6],
    RunRec *all_runs[6], const int32_t r0s[6], const int32_t counts_all[6][256], const int32_t m16s[6],
    int w_min, int chain_cap
) {
    printf("\n=== PACKED 8-B u64 THIN-ONLY STATIC M1 (w_min=%d, cap=%d B, ZERO SLP!) ===\n", w_min, chain_cap);
    printf("block       |   r0  |  m16   | ops_R1 | sav_rows (rho)  | T_buildG  T_peel_1  T_peel_8 | ns/op(8) | PeakM(8B)\n");
    printf("------------+-------+--------+--------+-----------------+------------------------------+----------+----------\n");

    double sb=0, sp1=0, sp8=0;
    int64_t sops=0, ssav=0, sm16=0;

    for (int bi = 0; bi < 6; bi++) {
        int32_t r0 = r0s[bi], p = ps[bi], m16 = m16s[bi];
        const RunRec *runs = all_runs[bi];
        const int32_t *counts = counts_all[bi];

        int32_t q_cap = r0 + 4;
        StripMeta *st = (StripMeta *)malloc((size_t)q_cap * sizeof(StripMeta));
        int32_t *j_order = (int32_t *)malloc((size_t)q_cap * sizeof(int32_t));
        uint64_t *packed = (uint64_t *)malloc((size_t)q_cap * sizeof(uint64_t));
        uint8_t *has_pred = (uint8_t *)malloc((size_t)q_cap);
        int32_t *heads = (int32_t *)malloc((size_t)q_cap * sizeof(int32_t));
        int32_t *out_delta = (int32_t *)malloc((size_t)q_cap * sizeof(int32_t));
        uint64_t *out_word = (uint64_t *)malloc((size_t)q_cap * sizeof(uint64_t));
        uint8_t *out_wlen = (uint8_t *)malloc((size_t)q_cap);

        double tb[7], tp1[7], tp8[7];
        int32_t last_ops = 0, last_sav = 0;

        for (int tr = 0; tr < 7; tr++) {
            double t0 = now_ms();

            int32_t C[256], run_cnt[256] = {0}, acc = 0;
            for (int c = 0; c < 256; c++) { C[c] = acc; acc += counts[c]; }
            int32_t cur_t[256]; memcpy(cur_t, C, sizeof(cur_t));

            int32_t q0 = 0, pos = 0;
            for (int32_t k = 0; k < r0; k++) {
                uint8_t c = runs[k].c; int32_t ln = runs[k].len;
                int32_t d = cur_t[c] - pos; cur_t[c] += ln;
                int32_t hi = pos + ln;
                if (pos <= p && p < hi) {
                    if (pos < p) { st[q0++] = (StripMeta){pos, p, d, c}; run_cnt[c]++; }
                    st[q0++] = (StripMeta){p, p + 1, d, c}; run_cnt[c]++;
                    if (p + 1 < hi) { st[q0++] = (StripMeta){p + 1, hi, d, c}; run_cnt[c]++; }
                } else {
                    st[q0++] = (StripMeta){pos, hi, d, c}; run_cnt[c]++;
                }
                pos = hi;
            }
            int32_t j_off[256], jacc = 0;
            for (int c = 0; c < 256; c++) { j_off[c] = jacc; jacc += run_cnt[c]; }
            for (int32_t u = 0; u < q0; u++) {
                j_order[j_off[st[u].c]++] = u;
                packed[u] = pack_r1(st[u].delta, NO_NEXT, st[u].c);
                has_pred[u] = 0;
            }

            /* Linear two-pointer merge filling ONLY thin exact matches (w_min <= width <= 16) into packed[u]! */
            int32_t ii = 0, jj = 0;
            while (ii < q0 && jj < q0) {
                int32_t v = ii, u = j_order[jj];
                int32_t v_lo = st[v].lo, v_hi = st[v].hi;
                int32_t u_tlo = st[u].lo + st[u].delta, u_thi = st[u].hi + st[u].delta;
                if (u_tlo == v_lo && u_thi == v_hi) {
                    int32_t wid = v_hi - v_lo;
                    if (u != v && wid >= w_min && wid <= 16) {
                        packed[u] = pack_r1(st[u].delta, (uint32_t)v, st[u].c);
                        has_pred[v] = 1;
                    }
                    ii++; jj++;
                } else if (u_thi <= v_hi) {
                    jj++;
                } else {
                    ii++;
                }
            }
            /* Collect chain heads that ACTUALLY HAVE a successor (skip singleton non-chained strips completely!) */
            int32_t n_heads = 0;
            for (int32_t u = 0; u < q0; u++) {
                if (!has_pred[u] && unpack_next(packed[u]) != NO_NEXT) {
                    heads[n_heads++] = u;
                }
            }
            double t1 = now_ms();

            /* === STAGE B1: Serial packed 8-B chain peel (capped at chain_cap <= 8 B!) === */
            int32_t ops_ser = 0, sav_ser = 0;
            for (int32_t i = 0; i < n_heads; i++) {
                int32_t cur = heads[i];
                int32_t wid = st[cur].hi - st[cur].lo;
                while (cur != (int32_t)NO_NEXT) {
                    uint64_t pv = packed[cur];
                    int32_t head = cur;
                    int32_t d_acc = unpack_delta(pv);
                    uint64_t w_acc = (uint64_t)unpack_byte(pv);
                    int len = 1;
                    uint32_t nxt = unpack_next(pv);
                    while (nxt != NO_NEXT && len < chain_cap) {
                        uint64_t nv = packed[nxt];
                        ops_ser++;
                        sav_ser += wid;
                        d_acc += unpack_delta(nv);
                        w_acc |= ((uint64_t)unpack_byte(nv) << (len * 8));
                        len++;
                        nxt = unpack_next(nv);
                    }
                    out_delta[head] = d_acc;
                    out_word[head] = w_acc;
                    out_wlen[head] = (uint8_t)len;
                    cur = (int32_t)nxt;
                }
            }
            double t2 = now_ms();
            uint64_t chk1 = 0;
            for (int32_t i = 0; i < n_heads; i++) {
                int32_t h = heads[i];
                chk1 ^= ((uint64_t)(uint32_t)out_delta[h] * 1315423911ULL) ^ out_word[h] ^ out_wlen[h];
            }

            /* === STAGE B2: K=8 Interleaved MLP packed 8-B chain peel (ONE u64 load per hop!) === */
            int32_t cur_l[8], head_l[8], d_l[8], len_l[8];
            uint64_t w_l[8];
            int32_t h_idx = 0, active = 0;
            for (int ln = 0; ln < 8; ln++) cur_l[ln] = -1;
            for (int ln = 0; ln < 8 && h_idx < n_heads; ln++) {
                int32_t h = heads[h_idx++];
                uint64_t pv = packed[h];
                head_l[ln] = h;
                d_l[ln] = unpack_delta(pv);
                w_l[ln] = (uint64_t)unpack_byte(pv);
                len_l[ln] = 1;
                cur_l[ln] = (int32_t)unpack_next(pv);
                active++;
            }
            while (active > 0) {
                for (int ln = 0; ln < 8; ln++) {
                    int32_t c = cur_l[ln];
                    if (c < 0) continue;
                    /* ONE SINGLE 8-BYTE u64 LOAD!! */
                    uint64_t nv = packed[c];
                    d_l[ln] += unpack_delta(nv);
                    w_l[ln] |= ((uint64_t)unpack_byte(nv) << (len_l[ln] * 8));
                    len_l[ln]++;
                    uint32_t nx = unpack_next(nv);
                    if (nx == NO_NEXT || len_l[ln] == chain_cap) {
                        int32_t h = head_l[ln];
                        out_delta[h] = d_l[ln];
                        out_word[h] = w_l[ln];
                        out_wlen[h] = (uint8_t)len_l[ln];
                        if (nx != NO_NEXT) {
                            /* Continue next cap-chunk starting at nx! */
                            uint64_t pv2 = packed[nx];
                            head_l[ln] = (int32_t)nx;
                            d_l[ln] = unpack_delta(pv2);
                            w_l[ln] = (uint64_t)unpack_byte(pv2);
                            len_l[ln] = 1;
                            uint32_t nx2 = unpack_next(pv2);
                            if (nx2 == NO_NEXT) {
                                out_delta[nx] = d_l[ln]; out_word[nx] = w_l[ln]; out_wlen[nx] = 1;
                                goto refill_lane;
                            }
                            cur_l[ln] = (int32_t)nx2;
                        } else {
                        refill_lane:
                            if (h_idx < n_heads) {
                                int32_t nh = heads[h_idx++];
                                uint64_t pv = packed[nh];
                                head_l[ln] = nh;
                                d_l[ln] = unpack_delta(pv);
                                w_l[ln] = (uint64_t)unpack_byte(pv);
                                len_l[ln] = 1;
                                cur_l[ln] = (int32_t)unpack_next(pv);
                            } else {
                                cur_l[ln] = -1;
                                active--;
                            }
                        }
                    } else {
                        cur_l[ln] = (int32_t)nx;
                    }
                }
            }
            double t3 = now_ms();
            uint64_t chk2 = 0;
            for (int32_t i = 0; i < n_heads; i++) {
                int32_t h = heads[i];
                chk2 ^= ((uint64_t)(uint32_t)out_delta[h] * 1315423911ULL) ^ out_word[h] ^ out_wlen[h];
            }
            if (chk1 != chk2) { fprintf(stderr, "CHECKSUM MISMATCH!\n"); exit(1); }
            volatile uint64_t sink = chk1 ^ chk2; (void)sink;

            tb[tr] = t1 - t0; tp1[tr] = t2 - t1; tp8[tr] = t3 - t2;
            last_ops = ops_ser; last_sav = sav_ser;
        }
        qsort(tb, 7, sizeof(double), cmp_dbl);
        qsort(tp1, 7, sizeof(double), cmp_dbl);
        qsort(tp8, 7, sizeof(double), cmp_dbl);
        sb += tb[3]; sp1 += tp1[3]; sp8 += tp8[3];
        sops += last_ops; ssav += last_sav; sm16 += m16;
        double ns_op = (last_ops > 0) ? (tp8[3] * 1e6 / (double)last_ops) : 0.0;
        double pk8_kib = (double)r0 * 8.0 / 1024.0;
        printf("%-11s | %5d | %6d | %6d | %7d (%5.2f) | %6.2f    %6.2f    %6.2f | %6.2f ns | %5.0f KiB\n",
               files[bi], r0, m16, last_ops, last_sav, (double)last_sav / (last_ops ? last_ops : 1),
               tb[3], tp1[3], tp8[3], ns_op, pk8_kib);

        free(st); free(j_order); free(packed); free(has_pred);
        free(heads); free(out_delta); free(out_word); free(out_wlen);
    }
    printf("------------+-------+--------+--------+-----------------+------------------------------+----------+----------\n");
    printf("SUM 6 BLKS  |509269 | 895391 | %6ld | %7ld (%5.2f) | %6.2f    %6.2f    %6.2f | %6.2f ns |\n",
           (long)sops, (long)ssav, (double)ssav / (sops ? sops : 1), sb, sp1, sp8, sp8 * 1e6 / (double)sops);
}

int main(void) {
    const char *files[6] = {"block-0.L","block-2.L","block-85.L","block-143.L","block-210.L","block-207.L"};
    const int32_t ns[6] = {899931,900000,899935,899869,899959,899998};
    const int32_t ps[6] = {264045,320807,325088,802868,791429,205180};

    RunRec *all_runs[6]; int32_t r0s[6], counts_all[6][256], m16s[6];
    for (int bi = 0; bi < 6; bi++) {
        char path[256]; snprintf(path, sizeof(path), "/tmp/mibz2-blocks/%s", files[bi]);
        FILE *fp = fopen(path, "rb"); int32_t n = ns[bi], p = ps[bi];
        uint8_t *L = (uint8_t *)malloc((size_t)n); fread(L, 1, (size_t)n, fp); fclose(fp);
        all_runs[bi] = (RunRec *)malloc((size_t)n * sizeof(RunRec));
        memset(counts_all[bi], 0, sizeof(counts_all[bi]));
        int32_t r0 = 0, m16 = 0, pt = 0;
        for (int32_t i = 0; i < n; ) {
            uint8_t c = L[i]; int32_t j = i + 1; while (j < n && L[j] == c) j++;
            int32_t ln = j - i;
            all_runs[bi][r0++] = (RunRec){c, ln};
            counts_all[bi][c] += ln;
            if (ln <= 16) { m16 += ln; if (i <= p && p < j) pt = 1; }
            i = j;
        }
        if (!pt) m16++;
        r0s[bi] = r0; m16s[bi] = m16;
        free(L);
    }

    bench_wmin(files, ns, ps, all_runs, r0s, counts_all, m16s, 1, 8);
    bench_wmin(files, ns, ps, all_runs, r0s, counts_all, m16s, 2, 8);

    for (int bi = 0; bi < 6; bi++) free(all_runs[bi]);
    return 0;
}
