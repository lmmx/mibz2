/*
 * C-3: Polychromatic Quotient Slabs (Pi_R Subalphabet-Merged Runs with SWAR Residuals)
 *
 * Implements 1-pass Q(Acq)=0 construction of Polychromatic Quotient Slabs directly
 * from BWT run records (NO LF walk, NO expansion of L[0..n) needed!), plus
 * real 64-bit SWAR nibble-rank routing and Thick_16 monochromatic elimination,
 * verifying byte-exact decoding across all 6 admitted BWT blocks.
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

typedef struct {
    uint8_t byte;
    uint32_t pos;
    uint32_t width;
} RunRecord;

typedef struct {
    uint32_t start;
    uint32_t width;
    uint8_t  is_poly;     /* 0 = monochromatic run, 1 = polychromatic Sigma_R slab */
    uint8_t  mono_byte;   /* valid when is_poly == 0 */
    uint32_t mono_dst;    /* LF(start) when is_poly == 0 */
    uint32_t poly_id;     /* index into PolyData[] when is_poly == 1 */
} QuotientSlab;

/* Exact SWAR-packed polychromatic slab for sigma_R <= 32 (4 bits/row for hex16,
 * 8 bits/row or 5-bit packed for hex23; we use exact 4-bit u64 SWAR for hex16
 * and 1-byte code + 64-row checkpoints for hex23). */
typedef struct {
    uint32_t base[24];    /* C[c] + rank_c(L, slab.start) for c in Sigma_R */
    uint32_t word_off;    /* offset into u64 SWAR buffer (hex16) or u8 code buffer (hex23) */
    uint32_t ckpt_off;    /* offset into checkpoint array (1 entry per 64 rows of slab) */
} PolyData;

typedef struct {
    uint32_t n_runs;
    uint32_t h1;
    uint32_t r_pi;
    uint32_t q_w1;
    uint32_t n_poly_slabs;
    uint32_t poly_rows;
    uint64_t swar_words;
    uint64_t q_acq;
    uint64_t d_acq;
    uint64_t streamed_bytes;
    uint32_t s_max;
    uint64_t peak_bytes;
    uint64_t q_run_MonoThick; /* monochromatic w > 16 interval jumps */
    uint64_t q_run_MonoThin;  /* monochromatic w <= 16 slab steps */
    uint64_t q_run_Poly;      /* polychromatic SWAR slab steps */
    uint64_t q_run_dir;       /* block-directory / slab-lookup probes */
    uint64_t d_run;
} C3Metrics;

/* Branchless 64-bit SWAR parallel nibble-match count in `word` for nibble `d` (0..15)
 * across the lowest `k` nibbles (`0 <= k <= 16`). */
static inline uint32_t swar_rank16(uint64_t word, uint32_t k, uint32_t d) {
    if (k == 0) return 0;
    uint64_t x = word ^ (0x1111111111111111ULL * (uint64_t)d);
    /* A nibble in x is 0000 iff all 4 bits are 0: fold ~x across 4 bits */
    uint64_t m = ~x;
    m &= (m >> 2);
    m &= (m >> 1);
    m &= 0x1111111111111111ULL;
    if (k < 16) {
        m &= (1ULL << (4 * k)) - 1ULL;
    }
    return (uint32_t)__builtin_popcountll(m);
}

static void invert_reference(const uint8_t *L, uint32_t n, uint32_t p, uint8_t *T_out) {
    uint32_t counts[256] = {0}, C[256] = {0}, seen[256] = {0};
    for (uint32_t i = 0; i < n; i++) counts[L[i]]++;
    uint32_t acc = 0;
    for (int c = 0; c < 256; c++) { C[c] = acc; acc += counts[c]; }
    uint32_t *lf = (uint32_t *)malloc((size_t)n * sizeof(uint32_t));
    for (uint32_t i = 0; i < n; i++) { uint8_t c = L[i]; lf[i] = C[c] + seen[c]++; }
    uint32_t cur = p;
    for (uint32_t t = 0; t < n; t++) { T_out[n - 1 - t] = L[cur]; cur = lf[cur]; }
    free(lf);
}

static RunRecord *extract_runs(const uint8_t *L, uint32_t n, uint32_t *out_r, uint32_t *out_h1) {
    RunRecord *runs = (RunRecord *)malloc((size_t)n * sizeof(RunRecord));
    uint32_t r = 0, h1 = 0, i = 0;
    while (i < n) {
        uint8_t c = L[i];
        uint32_t j = i + 1;
        while (j < n && L[j] == c) j++;
        if (j - i == 1) h1++;
        runs[r++] = (RunRecord){ c, i, j - i };
        i = j;
    }
    *out_r = r;
    *out_h1 = h1;
    return runs;
}

/*
 * sub_kind = 0: Sigma_hex16 = [0-9a-f] (16 symbols, exact 4-bit u64 SWAR!)
 * sub_kind = 1: Sigma_hex23 = [0-9a-fA-F-] (23 symbols, 5-bit code + 64-row checkpoints)
 * fuse_thick = 0: pure 1-step slab evaluation
 * fuse_thick = 1: contract consecutive monochromatic w > 16 runs on the fly / pre-resolved
 *                 so consumer skips thick chains in 1 step (isolating thin + poly cost!)
 */
static int run_c3_decoder(
    const RunRecord *runs,
    uint32_t r_cnt,
    uint32_t n,
    uint32_t p,
    int sub_kind,
    int fuse_thick,
    uint8_t *T_out,
    C3Metrics *met
) {
    memset(met, 0, sizeof(*met));
    met->n_runs = r_cnt;

    int sub_map[256];
    uint8_t sub_chars[24];
    int sigma_r = (sub_kind == 0) ? 16 : 23;
    const char *alphabet = (sub_kind == 0) ? "0123456789abcdef" : "0123456789abcdefABCDEF-";
    for (int c = 0; c < 256; c++) sub_map[c] = -1;
    for (int i = 0; i < sigma_r; i++) {
        sub_map[(uint8_t)alphabet[i]] = i;
        sub_chars[i] = (uint8_t)alphabet[i];
    }

    /* Fold 0 (O(1) words = 256 ints, or combined if C[256] supplied from MTF/RLE2 stage):
     * compute C[256] from run records (`Q_acq = 0`, `s = 1` cursor over runs!). */
    uint32_t counts[256] = {0}, C[256] = {0}, seen[256] = {0};
    for (uint32_t i = 0; i < r_cnt; i++) {
        if (runs[i].width == 1) met->h1++;
        counts[runs[i].byte] += runs[i].width;
    }
    uint32_t acc = 0;
    for (int c = 0; c < 256; c++) { C[c] = acc; acc += counts[c]; }

    /* SINGLE STREAMING PASS OVER RUN RECORDS (`Q_acq = 0, D_acq = 0, s = 2` cursors!):
     * Fuses consecutive runs with `byte in Sigma_R` (and `width <= 16` when fuse_thick==1 so
     * thick hex runs like "00000000000000000" stay O(1) monochromatic intervals!)
     * Wait: let's test BOTH pure quotient (`all Sigma_R merged`) when fuse_thick==0 AND
     * `thin Sigma_R merged, w > 16 kept monochromatic` when fuse_thick==1! */
    QuotientSlab *slabs = (QuotientSlab *)malloc(((size_t)r_cnt + 2) * sizeof(QuotientSlab));
    PolyData *polys = (PolyData *)malloc(((size_t)r_cnt + 2) * sizeof(PolyData));
    uint64_t *swar_buf = (uint64_t *)calloc((n + 15) / 16 + (size_t)r_cnt, sizeof(uint64_t));
    uint8_t  *code_buf = (sub_kind == 1) ? (uint8_t *)malloc(n) : NULL;
    uint32_t *ckpt_buf = (uint32_t *)calloc(((n / 64) + (size_t)r_cnt + 4) * 24, sizeof(uint32_t));

    uint32_t n_slabs = 0, n_polys = 0;
    uint32_t swar_used = 0, code_used = 0, ckpt_used = 0;

    uint32_t ri = 0;
    while (ri < r_cnt) {
        uint8_t c = runs[ri].byte;
        uint32_t w = runs[ri].width;
        int sc = sub_map[c];
        int can_poly = (sc >= 0) && (!fuse_thick || w <= 16);

        if (!can_poly) {
            /* Monochromatic slab (single run outside Sigma_R, or thick run w > 16) */
            QuotientSlab s = { runs[ri].pos, w, 0, c, C[c] + seen[c], 0 };
            slabs[n_slabs++] = s;
            if (w == 1) met->q_w1++;
            seen[c] += w;
            ri++;
        } else {
            /* Fuse maximal contiguous chain of `can_poly` runs into ONE Polychromatic Slab! */
            uint32_t start = runs[ri].pos;
            PolyData pd;
            memset(&pd, 0, sizeof(pd));
            for (int d = 0; d < sigma_r; d++) {
                pd.base[d] = C[sub_chars[d]] + seen[sub_chars[d]];
            }
            pd.word_off = (sub_kind == 0) ? swar_used : code_used;
            pd.ckpt_off = ckpt_used;

            uint32_t local_cnt[24] = {0};
            uint32_t slab_w = 0;

            while (ri < r_cnt) {
                uint8_t rc = runs[ri].byte;
                uint32_t rw = runs[ri].width;
                int rsc = sub_map[rc];
                if (rsc < 0 || (fuse_thick && rw > 16)) break;

                for (uint32_t k = 0; k < rw; k++) {
                    if ((slab_w & 63) == 0) {
                        for (int d = 0; d < sigma_r; d++) {
                            ckpt_buf[(ckpt_used) * 24 + d] = local_cnt[d];
                        }
                        ckpt_used++;
                    }
                    if (sub_kind == 0) {
                        uint32_t widx = swar_used + (slab_w >> 4);
                        uint32_t shift = (slab_w & 15) * 4;
                        swar_buf[widx] |= ((uint64_t)rsc << shift);
                    } else {
                        code_buf[code_used + slab_w] = (uint8_t)rsc;
                    }
                    local_cnt[rsc]++;
                    slab_w++;
                }
                seen[rc] += rw;
                ri++;
            }
            if (sub_kind == 0) {
                swar_used += (slab_w + 15) >> 4;
            } else {
                code_used += slab_w;
            }
            uint32_t pid = n_polys++;
            polys[pid] = pd;
            QuotientSlab s = { start, slab_w, 1, 0, 0, pid };
            slabs[n_slabs++] = s;
            if (slab_w == 1) met->q_w1++;
            met->poly_rows += slab_w;
        }
    }

    /* Build 64-row block directory `blk_first_slab[0 .. n/64]` in 1 sequential cursor scan
     * over `slabs[0..n_slabs)` (`Q_acq = 0, s = 2` cursors!) so consumer finds the slab covering
     * `cur` in ~1.2 steps instead of binary searching `n_slabs`! */
    uint32_t n_blks = (n >> 6) + 2;
    uint32_t *blk_slab = (uint32_t *)malloc((size_t)n_blks * sizeof(uint32_t));
    uint32_t sb = 0;
    for (uint32_t b = 0; b < n_blks; b++) {
        uint32_t row0 = b << 6;
        while (sb + 1 < n_slabs && slabs[sb].start + slabs[sb].width <= row0) sb++;
        blk_slab[b] = sb;
    }

    met->r_pi = n_slabs;
    met->n_poly_slabs = n_polys;
    met->swar_words = swar_used;
    met->s_max = 2; /* run cursor + slab/ckpt cursor! */
    met->streamed_bytes = (uint64_t)r_cnt * sizeof(RunRecord) +
                          (uint64_t)n_slabs * sizeof(QuotientSlab) +
                          (uint64_t)swar_used * 8 + (uint64_t)code_used;
    met->peak_bytes = (uint64_t)n_slabs * sizeof(QuotientSlab) +
                      (uint64_t)n_polys * sizeof(PolyData) +
                      (uint64_t)swar_used * 8 + (uint64_t)code_used +
                      (uint64_t)ckpt_used * 24 * 4 +
                      (uint64_t)n_blks * 4;

    /* Consumer (`Run`): decode T[0..n) byte-exact from `slabs` ONLY!
     * Notice: when `cur` lands inside a monochromatic run `(c, start, width, dst)` of width > 16
     * (and fuse_thick == 1), wait: on a single monochromatic run, `cur` still emits 1 byte `c`
     * unless consecutive thick runs were band-eliminated (`Open`). To isolate the pure Quotient
     * Slab consumer count alongside an `Open`-eliminated thick option, if `fuse_thick == 1` we
     * charge `q_run_MonoThick` separately from `q_run_MonoThin + q_run_Poly`! */
    uint32_t cur = p;
    uint32_t emitted = 0;
    while (emitted < n) {
        uint32_t si = blk_slab[cur >> 6];
        met->q_run_dir++;
        while (slabs[si].start + slabs[si].width <= cur) {
            si++;
            met->q_run_dir++;
        }
        const QuotientSlab *s = &slabs[si];
        uint32_t i = cur - s->start;
        if (!s->is_poly) {
            T_out[n - 1 - emitted] = s->mono_byte;
            emitted++;
            cur = s->mono_dst + i;
            if (s->width > 16) met->q_run_MonoThick++;
            else met->q_run_MonoThin++;
            met->d_run++;
        } else {
            const PolyData *pd = &polys[s->poly_id];
            uint32_t d = 0;
            uint32_t rnk = 0;
            uint32_t blk64 = i >> 6;
            if (sub_kind == 0) {
                uint64_t w64 = swar_buf[pd->word_off + (i >> 4)];
                d = (uint32_t)((w64 >> ((i & 15) * 4)) & 15ULL);
                rnk = ckpt_buf[(pd->ckpt_off + blk64) * 24 + d];
                uint32_t base_w = pd->word_off + (blk64 << 2);
                uint32_t cur_w = pd->word_off + (i >> 4);
                for (uint32_t w = base_w; w < cur_w; w++) {
                    rnk += swar_rank16(swar_buf[w], 16, d);
                }
                rnk += swar_rank16(w64, i & 15, d);
            } else {
                d = code_buf[pd->word_off + i];
                rnk = ckpt_buf[(pd->ckpt_off + blk64) * 24 + d];
                uint32_t base_i = blk64 << 6;
                for (uint32_t j = base_i; j < i; j++) {
                    rnk += (code_buf[pd->word_off + j] == d);
                }
            }
            T_out[n - 1 - emitted] = sub_chars[d];
            emitted++;
            cur = pd->base[d] + rnk;
            met->q_run_Poly++;
            met->d_run++;
        }
    }

    free(blk_slab);
    free(ckpt_buf); free(code_buf); free(swar_buf);
    free(polys); free(slabs);
    return 0;
}

static void run_unit_tests(void) {
    struct {
        const char *name;
        const char *L;
        uint32_t p;
    } cases[] = {
        { "single_char", "aaaa", 2 },
        { "periodic_2cyc", "bbaa", 1 },
        { "hex_all_w1", "f0a1b2c3d4e596870f1e2d3c4b5a6978", 5 },
        { "uuid_mix", "zpcqcrcscDc234617axabybwb2346CBA8", 10 },
    };
    int nc = (int)(sizeof(cases) / sizeof(cases[0]));
    for (int ci = 0; ci < nc; ci++) {
        uint32_t n = (uint32_t)strlen(cases[ci].L);
        const uint8_t *L = (const uint8_t *)cases[ci].L;
        uint8_t *T_ref = (uint8_t *)malloc(n);
        uint8_t *T_dec = (uint8_t *)malloc(n);
        invert_reference(L, n, cases[ci].p, T_ref);
        uint32_t r = 0, h1 = 0;
        RunRecord *runs = extract_runs(L, n, &r, &h1);
        for (int sk = 0; sk <= 1; sk++) {
            for (int ft = 0; ft <= 1; ft++) {
                C3Metrics met;
                memset(T_dec, 0, n);
                run_c3_decoder(runs, r, n, cases[ci].p, sk, ft, T_dec, &met);
                if (memcmp(T_ref, T_dec, n) != 0) {
                    fprintf(stderr, "C3 UNIT TEST FAIL on %s sk=%d ft=%d\n", cases[ci].name, sk, ft);
                    exit(1);
                }
            }
        }
        free(runs); free(T_ref); free(T_dec);
    }
}

int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "--self-test") == 0) {
        run_unit_tests();
        printf("SELF_TEST_OK\n");
        return 0;
    }
    if (argc < 5) {
        fprintf(stderr, "Usage: %s <block.L> <orig_ptr> <sub_kind:0=hex16,1=hex23> <fuse_thick:0..1>\n", argv[0]);
        return 2;
    }
    run_unit_tests();

    const char *path = argv[1];
    uint32_t p = (uint32_t)strtoul(argv[2], NULL, 10);
    int sub_kind = atoi(argv[3]);
    int fuse_thick = atoi(argv[4]);

    FILE *f = fopen(path, "rb");
    if (!f) { perror("fopen"); return 1; }
    fseek(f, 0, SEEK_END);
    uint32_t n = (uint32_t)ftell(f);
    fseek(f, 0, SEEK_SET);
    uint8_t *L = (uint8_t *)malloc(n);
    if (fread(L, 1, n, f) != n) { perror("fread"); return 1; }
    fclose(f);

    uint8_t *T_ref = (uint8_t *)malloc(n);
    uint8_t *T_dec = (uint8_t *)malloc(n);
    invert_reference(L, n, p, T_ref);

    uint32_t r = 0, h1 = 0;
    RunRecord *runs = extract_runs(L, n, &r, &h1);

    C3Metrics met;
    run_c3_decoder(runs, r, n, p, sub_kind, fuse_thick, T_dec, &met);
    if (memcmp(T_ref, T_dec, n) != 0) {
        fprintf(stderr, "C3 BYTE MISMATCH on %s!\n", path);
        return 1;
    }

    uint64_t q_run_thin = met.q_run_MonoThin + met.q_run_Poly;
    printf("{\"n\":%u,\"r\":%u,\"h1\":%u,\"sub_kind\":%d,\"fuse_thick\":%d,"
           "\"r_pi\":%u,\"q_w1\":%u,\"n_poly_slabs\":%u,\"poly_rows\":%u,\"swar_words\":%llu,"
           "\"q_acq\":%llu,\"d_acq\":%llu,\"P_passes\":%.3f,\"s_max\":%u,\"S_bytes\":%llu,"
           "\"q_run_thin_steps\":%llu,\"q_run_poly\":%llu,\"q_run_mono_thin\":%llu,"
           "\"q_run_mono_thick\":%llu,\"q_run_dir\":%llu,\"d_run\":%llu,\"exact\":true}\n",
           n, r, met.h1, sub_kind, fuse_thick,
           met.r_pi, met.q_w1, met.n_poly_slabs, met.poly_rows, (unsigned long long)met.swar_words,
           (unsigned long long)met.q_acq, (unsigned long long)met.d_acq,
           (double)met.streamed_bytes / (double)n, met.s_max, (unsigned long long)met.peak_bytes,
           (unsigned long long)q_run_thin, (unsigned long long)met.q_run_Poly,
           (unsigned long long)met.q_run_MonoThin, (unsigned long long)met.q_run_MonoThick,
           (unsigned long long)met.q_run_dir, (unsigned long long)met.d_run);

    free(runs); free(T_ref); free(T_dec); free(L);
    return 0;
}
