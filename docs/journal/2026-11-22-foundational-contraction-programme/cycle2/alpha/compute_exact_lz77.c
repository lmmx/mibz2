/*
 * Compute exact LZ77 factor count z(T) and non-overlapping LZ77 z_no(T)
 * for the exact decoded texts of all 6 authenticated Wikidata blocks
 * using exact O(n log n) suffix array + Kasai LCP + nearest-smaller-previous-pos (PSV/NSV)!
 *
 * Remember Theorem (Universal Grammar/SLP Lower Bound):
 *   Every exact RLSLP of size |G| generating T satisfies |G| >= z_no(T) >= z(T) (modulo O(|Sigma|) literals)!
 *   (Exact lemma: every non-overlapping LZ77 phrase starts at a leftmost occurrence of some
 *    substring or a phrase boundary, and by Charikar et al. 2005 / Rytter 2003,
 *    g(T) >= z_no(T) for any SLP, and g_rl(T) >= z_no_rl(T) / 2 or even tighter:
 *    for RLSLP with power nodes, each power node a^k is self-overlapping LZ77 of 1 phrase!
 *    So g_rl(T) >= z(T) (self-overlapping LZ77!) ALWAYS holds!)
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

/* Exact O(n log n) prefix-doubling Suffix Array with counting sort */
static void build_sa(const uint8_t *s, int32_t n, int32_t *sa) {
    int32_t *rk = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    int32_t *tmp = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    int32_t *cnt = (int32_t *)malloc((size_t)(n > 256 ? n + 1 : 257) * sizeof(int32_t));

    int32_t alpha = 256;
    memset(cnt, 0, (size_t)alpha * sizeof(int32_t));
    for (int32_t i = 0; i < n; i++) cnt[s[i]]++;
    for (int32_t i = 1; i < alpha; i++) cnt[i] += cnt[i - 1];
    for (int32_t i = n - 1; i >= 0; i--) sa[--cnt[s[i]]] = i;
    rk[sa[0]] = 0;
    int32_t classes = 1;
    for (int32_t i = 1; i < n; i++) {
        if (s[sa[i]] != s[sa[i - 1]]) classes++;
        rk[sa[i]] = classes - 1;
    }

    int32_t *sa2 = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    for (int32_t h = 1; h < n && classes < n; h <<= 1) {
        int32_t p = 0;
        for (int32_t i = n - h; i < n; i++) sa2[p++] = i;
        for (int32_t i = 0; i < n; i++) if (sa[i] >= h) sa2[p++] = sa[i] - h;

        memset(cnt, 0, (size_t)classes * sizeof(int32_t));
        for (int32_t i = 0; i < n; i++) cnt[rk[sa2[i]]]++;
        for (int32_t i = 1; i < classes; i++) cnt[i] += cnt[i - 1];
        for (int32_t i = n - 1; i >= 0; i--) sa[--cnt[rk[sa2[i]]]] = sa2[i];

        tmp[sa[0]] = 0;
        int32_t nc = 1;
        for (int32_t i = 1; i < n; i++) {
            int32_t u = sa[i], v = sa[i - 1];
            int32_t ru2 = (u + h < n) ? rk[u + h] : -1;
            int32_t rv2 = (v + h < n) ? rk[v + h] : -1;
            if (rk[u] != rk[v] || ru2 != rv2) nc++;
            tmp[u] = nc - 1;
        }
        memcpy(rk, tmp, (size_t)n * sizeof(int32_t));
        classes = nc;
    }
    free(rk); free(tmp); free(cnt); free(sa2);
}

/* Exact LPF (Longest Previous Factor) using SA + Kasai LCP + monotone stack (Crochemore-Ilie) */
static void compute_lz77_exact(const uint8_t *s, int32_t n, int32_t *z_self, int32_t *z_no) {
    int32_t *sa = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    build_sa(s, n, sa);

    int32_t *isa = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    for (int32_t i = 0; i < n; i++) isa[sa[i]] = i;

    int32_t *lcp = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    int32_t h = 0;
    for (int32_t i = 0; i < n; i++) {
        int32_t r = isa[i];
        if (r == 0) { lcp[0] = 0; h = 0; continue; }
        int32_t j = sa[r - 1];
        while (i + h < n && j + h < n && s[i + h] == s[j + h]) h++;
        lcp[r] = h;
        if (h > 0) h--;
    }

    /* Compute LPF[i] = max_{j < i} LCP(s[j..], s[i..]) via PSV and NSV in SA order! */
    int32_t *lpf = (int32_t *)calloc((size_t)n, sizeof(int32_t));
    int32_t *st_pos = (int32_t *)malloc((size_t)(n + 1) * sizeof(int32_t));
    int32_t *st_lcp = (int32_t *)malloc((size_t)(n + 1) * sizeof(int32_t));

    /* Left-to-right pass (PSV in SA): maintains stack of strictly increasing SA positions */
    int32_t top = 0;
    for (int32_t r = 0; r < n; r++) {
        int32_t pos = sa[r];
        int32_t cur_lcp = lcp[r];
        while (top > 0 && st_pos[top - 1] > pos) {
            if (st_lcp[top - 1] < cur_lcp) cur_lcp = st_lcp[top - 1];
            top--;
        }
        if (top > 0) {
            if (cur_lcp > lpf[pos]) lpf[pos] = cur_lcp;
        }
        st_pos[top] = pos;
        st_lcp[top] = cur_lcp;
        top++;
    }

    /* Right-to-left pass (NSV in SA) */
    top = 0;
    for (int32_t r = n - 1; r >= 0; r--) {
        int32_t pos = sa[r];
        int32_t cur_lcp = (r + 1 < n) ? lcp[r + 1] : 0;
        while (top > 0 && st_pos[top - 1] > pos) {
            if (st_lcp[top - 1] < cur_lcp) cur_lcp = st_lcp[top - 1];
            top--;
        }
        if (top > 0) {
            if (cur_lcp > lpf[pos]) lpf[pos] = cur_lcp;
        }
        st_pos[top] = pos;
        st_lcp[top] = cur_lcp;
        top++;
    }

    /* Exact self-overlapping LZ77 (without trailing char, i.e., standard LZ77):
     * if lpf[i] == 0: 1 literal phrase; else phrase of length lpf[i]! */
    int32_t zs = 0;
    for (int32_t i = 0; i < n; ) {
        int32_t len = lpf[i];
        i += (len > 0 ? len : 1);
        zs++;
    }
    *z_self = zs;

    /* Exact non-overlapping LZ77 (LPnF via SA neighborhood capped at pos - src):
     * Quick exact computation by checking nearest smaller SA neighbor on left/right with src + len <= pos!
     * For upper/exact non-overlapping: note lpnf[i] <= lpf[i], and on Wikidata RLE1 removed long runs
     * so self-overlap is rare; compute exact LPnF by scanning SA locally around isa[i] only at phrase starts! */
    int32_t zn = 0;
    for (int32_t i = 0; i < n; ) {
        int32_t r = isa[i];
        int32_t best = 0;
        /* Scan left in SA while min_lcp > best */
        int32_t ml = lcp[r];
        for (int32_t k = r - 1; k >= 0 && ml > best; k--) {
            int32_t src = sa[k];
            if (src < i) {
                int32_t cap = i - src;
                int32_t eff = (ml < cap) ? ml : cap;
                if (eff > best) best = eff;
            }
            if (k > 0 && lcp[k] < ml) ml = lcp[k];
        }
        /* Scan right in SA while min_lcp > best */
        if (r + 1 < n) {
            ml = lcp[r + 1];
            for (int32_t k = r + 1; k < n && ml > best; k++) {
                int32_t src = sa[k];
                if (src < i) {
                    int32_t cap = i - src;
                    int32_t eff = (ml < cap) ? ml : cap;
                    if (eff > best) best = eff;
                }
                if (k + 1 < n && lcp[k + 1] < ml) ml = lcp[k + 1];
            }
        }
        i += (best > 0 ? best : 1);
        zn++;
    }
    *z_no = zn;

    free(sa); free(isa); free(lcp); free(lpf); free(st_pos); free(st_lcp);
}

int main(void) {
    const char *files[6] = {"block-0.L","block-2.L","block-85.L","block-143.L","block-210.L","block-207.L"};
    const int32_t ns[6] = {899931,900000,899935,899869,899959,899998};
    const int32_t ps[6] = {264045,320807,325088,802868,791429,205180};
    const int32_t r0s[6] = {51184,54532,79375,108335,128732,87111};
    const int32_t m16s[6] = {89874,112070,156404,196841,204281,135921};

    printf("block       |   r0   |  m16   |  z(T)  (z/r0, z/m16) | z_no(T) (zno/r0) | Q_wtr (Q/z_no)\n");
    printf("------------+--------+--------+----------------------+------------------+---------------\n");
    const int32_t qwtr[6] = {106144, 120489, 158378, 217323, 238661, 177937};
    int64_t sr=0, sm=0, sz=0, szn=0, sq=0;
    for (int bi = 0; bi < 6; bi++) {
        char path[256]; snprintf(path, sizeof(path), "/tmp/mibz2-blocks/%s", files[bi]);
        FILE *fp = fopen(path, "rb"); int32_t n = ns[bi], p = ps[bi];
        uint8_t *L = (uint8_t *)malloc((size_t)n);
        fread(L, 1, (size_t)n, fp); fclose(fp);
        uint8_t *T = (uint8_t *)malloc((size_t)n);
        ref_ibwt(L, n, p, T); free(L);

        int32_t zs = 0, zn = 0;
        compute_lz77_exact(T, n, &zs, &zn);
        free(T);

        printf("%-11s | %6d | %6d | %6d (%5.3fr, %5.3fm) | %7d (%5.3fr) | %6d (%4.2fx)\n",
               files[bi], r0s[bi], m16s[bi], zs, (double)zs/r0s[bi], (double)zs/m16s[bi],
               zn, (double)zn/r0s[bi], qwtr[bi], (double)qwtr[bi]/zn);
        sr += r0s[bi]; sm += m16s[bi]; sz += zs; szn += zn; sq += qwtr[bi];
    }
    printf("------------+--------+--------+----------------------+------------------+---------------\n");
    printf("SUM 6 BLKS  | %6ld | %6ld | %6ld (%5.3fr, %5.3fm) | %7ld (%5.3fr) | %6ld (%4.2fx)\n",
           (long)sr, (long)sm, (long)sz, (double)sz/sr, (double)sz/sm,
           (long)szn, (double)szn/sr, (long)sq, (double)sq/szn);
    return 0;
}
