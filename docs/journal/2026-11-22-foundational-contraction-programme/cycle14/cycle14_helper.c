/*
 * cycle14_helper.c — Fast exact O(n log n) C routines for Cycle 14:
 *   1. Exact circular BWT + single-cycle LF check + run count r_0
 *   2. Exact standard LZ77 (self-overlapping, z_77) & non-overlapping (z_no)
 *      via exact Suffix Array (with unique sentinel) + Kasai LCP + PSV/NSV
 *   3. Exact Jez0 RLSLP (RLE + greedy 2-partition pair compression) rule count g_rl
 */
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/* ---- 1. Circular Suffix Array & BWT (doubling, exact for cyclic shifts) ---- */
static void build_circular_sa(const uint8_t *s, int n, int *sa) {
    int *rk = (int *)malloc((size_t)n * sizeof(int));
    int *tmp = (int *)malloc((size_t)n * sizeof(int));
    int *sa2 = (int *)malloc((size_t)n * sizeof(int));
    int m = 256;
    if (n > m) m = n;
    int *cnt = (int *)malloc((size_t)(m + 2) * sizeof(int));

    for (int i = 0; i < 256; i++) cnt[i] = 0;
    for (int i = 0; i < n; i++) { rk[i] = s[i]; cnt[rk[i]]++; }
    for (int i = 1; i < 256; i++) cnt[i] += cnt[i - 1];
    for (int i = n - 1; i >= 0; i--) sa[--cnt[rk[i]]] = i;

    int classes = 1;
    tmp[sa[0]] = 0;
    for (int i = 1; i < n; i++) {
        if (rk[sa[i]] != rk[sa[i - 1]]) classes++;
        tmp[sa[i]] = classes - 1;
    }
    memcpy(rk, tmp, (size_t)n * sizeof(int));

    for (int k = 1; k < n && classes < n; k <<= 1) {
        for (int i = 0; i < n; i++) {
            int p = sa[i] - k;
            if (p < 0) p += n;
            sa2[i] = p;
        }
        for (int i = 0; i < classes; i++) cnt[i] = 0;
        for (int i = 0; i < n; i++) cnt[rk[sa2[i]]]++;
        for (int i = 1; i < classes; i++) cnt[i] += cnt[i - 1];
        for (int i = n - 1; i >= 0; i--) {
            int p = sa2[i];
            sa[--cnt[rk[p]]] = p;
        }
        tmp[sa[0]] = 0;
        classes = 1;
        for (int i = 1; i < n; i++) {
            int cur = sa[i], prev = sa[i - 1];
            int cur2 = cur + k; if (cur2 >= n) cur2 -= n;
            int prev2 = prev + k; if (prev2 >= n) prev2 -= n;
            if (rk[cur] != rk[prev] || rk[cur2] != rk[prev2]) classes++;
            tmp[cur] = classes - 1;
        }
        memcpy(rk, tmp, (size_t)n * sizeof(int));
    }

    free(rk); free(tmp); free(sa2); free(cnt);
}

/* Returns r_0 (circular BWT runs), writes L_out (if non-NULL), sets *is_single_cycle (1 iff primitive/single LF cycle). */
int exact_circular_bwt(const uint8_t *s, int n, uint8_t *L_out, int *is_single_cycle, int *primary_idx) {
    if (n <= 0) { if (is_single_cycle) *is_single_cycle = 0; return 0; }
    int *sa = (int *)malloc((size_t)n * sizeof(int));
    build_circular_sa(s, n, sa);

    uint8_t *L = L_out ? L_out : (uint8_t *)malloc((size_t)n);
    int p0 = -1;
    for (int i = 0; i < n; i++) {
        int pos = sa[i];
        if (pos == 0) p0 = i;
        L[i] = s[(pos == 0) ? (n - 1) : (pos - 1)];
    }
    if (primary_idx) *primary_idx = p0;

    int runs = 1;
    for (int i = 1; i < n; i++) {
        if (L[i] != L[i - 1]) runs++;
    }

    /* Check single-cycle LF */
    int counts[256] = {0};
    for (int i = 0; i < n; i++) counts[L[i]]++;
    int C[256];
    int acc = 0;
    for (int b = 0; b < 256; b++) { C[b] = acc; acc += counts[b]; }
    int *lf = (int *)malloc((size_t)n * sizeof(int));
    int occ[256] = {0};
    for (int i = 0; i < n; i++) {
        uint8_t b = L[i];
        lf[i] = C[b] + occ[b]++;
    }
    int cur = 0, vis = 0;
    do {
        cur = lf[cur];
        vis++;
    } while (cur != 0 && vis <= n);
    if (is_single_cycle) *is_single_cycle = (vis == n) ? 1 : 0;

    free(lf);
    if (!L_out) free(L);
    free(sa);
    return runs;
}

/* ---- 2. Exact Linear/O(n log n) Suffix Array (non-circular, unique sentinel -1 at s[n]) ---- */
static void build_linear_sa_lcp(const uint8_t *s, int n, int *sa, int *rk, int *lcp) {
    int N = n + 1;
    int *tmp = (int *)malloc((size_t)N * sizeof(int));
    int *sa2 = (int *)malloc((size_t)N * sizeof(int));
    int m = (N > 257) ? N : 257;
    int *cnt = (int *)malloc((size_t)(m + 2) * sizeof(int));

    for (int i = 0; i < 257; i++) cnt[i] = 0;
    for (int i = 0; i < n; i++) { rk[i] = (int)s[i] + 1; cnt[rk[i]]++; }
    rk[n] = 0; cnt[0]++;
    for (int i = 1; i < 257; i++) cnt[i] += cnt[i - 1];
    for (int i = N - 1; i >= 0; i--) sa[--cnt[rk[i]]] = i;

    int classes = 1;
    tmp[sa[0]] = 0;
    for (int i = 1; i < N; i++) {
        if (rk[sa[i]] != rk[sa[i - 1]]) classes++;
        tmp[sa[i]] = classes - 1;
    }
    memcpy(rk, tmp, (size_t)N * sizeof(int));

    for (int k = 1; k < N && classes < N; k <<= 1) {
        for (int i = 0; i < N; i++) {
            int p = sa[i] - k;
            if (p < 0) p += N;
            sa2[i] = p;
        }
        for (int i = 0; i < classes; i++) cnt[i] = 0;
        for (int i = 0; i < N; i++) cnt[rk[sa2[i]]]++;
        for (int i = 1; i < classes; i++) cnt[i] += cnt[i - 1];
        for (int i = N - 1; i >= 0; i--) {
            int p = sa2[i];
            sa[--cnt[rk[p]]] = p;
        }
        tmp[sa[0]] = 0;
        classes = 1;
        for (int i = 1; i < N; i++) {
            int cur = sa[i], prev = sa[i - 1];
            int cur2 = cur + k; if (cur2 >= N) cur2 -= N;
            int prev2 = prev + k; if (prev2 >= N) prev2 -= N;
            if (rk[cur] != rk[prev] || rk[cur2] != rk[prev2]) classes++;
            tmp[cur] = classes - 1;
        }
        memcpy(rk, tmp, (size_t)N * sizeof(int));
    }

    /* Kasai LCP: lcp[i] = LCP(sa[i], sa[i-1]) for i = 1..N-1 */
    for (int i = 0; i < N; i++) rk[sa[i]] = i;
    int h = 0;
    lcp[0] = 0;
    for (int i = 0; i < N; i++) {
        int r = rk[i];
        if (r == 0) { h = 0; continue; }
        int j = sa[r - 1];
        while (i + h < n && j + h < n && s[i + h] == s[j + h]) h++;
        lcp[r] = h;
        if (h > 0) h--;
    }

    free(tmp); free(sa2); free(cnt);
}

/*
 * Exact LZ77 parsing:
 *   z_std: standard self-overlapping LZ77 (each phrase is longest match starting at j < i,
 *          or single literal if LPF[i] == 0; notice NO extra trailing literal per phrase,
 *          standard stringology z(T) where phrase length is max(1, LPF[i])).
 *   z_lit: LZ77 with explicit trailing literal (phrase length is LPF[i] + 1).
 *   z_no:  non-overlapping LZ77 (source j + len <= i).
 * Using exact LPF via PSV/NSV over SA + RMQ (stack-based O(n) PSV/NSV!).
 */
void exact_lz77_counts(const uint8_t *s, int n, int *z_std_out, int *z_lit_out, int *z_no_out) {
    int N = n + 1;
    int *sa = (int *)malloc((size_t)N * sizeof(int));
    int *rk = (int *)malloc((size_t)N * sizeof(int));
    int *lcp = (int *)malloc((size_t)N * sizeof(int));
    build_linear_sa_lcp(s, n, sa, rk, lcp);

    /* Compute exact LPF[pos] in O(n) via monotone stack on SA (Crochemore-Ilie-Smyth) */
    int *lpf = (int *)calloc((size_t)n, sizeof(int));
    int *stk_pos = (int *)malloc((size_t)(N + 2) * sizeof(int));
    int *stk_lcp = (int *)malloc((size_t)(N + 2) * sizeof(int));
    int top = 0;
    /* Pass 1: PSV (closest preceding SA entry with sa[k] < sa[i]) */
    for (int i = 0; i < N; i++) {
        int pos = sa[i];
        int cur_lcp = (i > 0) ? lcp[i] : 0;
        while (top > 0 && stk_pos[top - 1] > pos) {
            if (stk_lcp[top - 1] < cur_lcp) cur_lcp = stk_lcp[top - 1];
            top--;
        }
        if (pos < n && top > 0) {
            if (cur_lcp > lpf[pos]) lpf[pos] = cur_lcp;
        }
        stk_pos[top] = pos;
        stk_lcp[top] = cur_lcp;
        top++;
    }
    /* Pass 2: NSV (closest following SA entry with sa[k] < sa[i]) */
    top = 0;
    for (int i = N - 1; i >= 0; i--) {
        int pos = sa[i];
        int cur_lcp = (i + 1 < N) ? lcp[i + 1] : 0;
        while (top > 0 && stk_pos[top - 1] > pos) {
            if (stk_lcp[top - 1] < cur_lcp) cur_lcp = stk_lcp[top - 1];
            top--;
        }
        if (pos < n && top > 0) {
            if (cur_lcp > lpf[pos]) lpf[pos] = cur_lcp;
        }
        stk_pos[top] = pos;
        stk_lcp[top] = cur_lcp;
        top++;
    }

    int z_std = 0, i = 0;
    while (i < n) {
        int len = lpf[i];
        i += (len > 0) ? len : 1;
        z_std++;
    }
    int z_lit = 0;
    i = 0;
    while (i < n) {
        i += lpf[i] + 1;
        z_lit++;
    }

    /* Non-overlapping LZ77 (if requested): exact binary-lifted / direct scan from SA neighbors */
    int z_no = 0;
    if (z_no_out) {
        i = 0;
        while (i < n) {
            int max_len = lpf[i];
            if (max_len == 0) { i += 1; z_no++; continue; }
            /* Find longest match starting at j with j + L <= i by walking left/right in SA from rk[i] */
            int r = rk[i];
            int best = 0;
            int mn = n;
            for (int k = r - 1; k >= 0; k--) {
                if (lcp[k + 1] < mn) mn = lcp[k + 1];
                if (mn <= best) break;
                int j = sa[k];
                if (j < i) {
                    int cap = i - j;
                    int cand = (mn < cap) ? mn : cap;
                    if (cand > best) best = cand;
                }
            }
            mn = n;
            for (int k = r + 1; k < N; k++) {
                if (lcp[k] < mn) mn = lcp[k];
                if (mn <= best) break;
                int j = sa[k];
                if (j < i) {
                    int cap = i - j;
                    int cand = (mn < cap) ? mn : cap;
                    if (cand > best) best = cand;
                }
            }
            i += (best > 0) ? best : 1;
            z_no++;
        }
        *z_no_out = z_no;
    }

    if (z_std_out) *z_std_out = z_std;
    if (z_lit_out) *z_lit_out = z_lit;

    free(sa); free(rk); free(lcp); free(lpf); free(stk_pos); free(stk_lcp);
}

/* ---- 3. Exact Jez0 RLSLP (alternating RLE + greedy bipartite pair compression) ---- */
typedef struct { uint64_t key; int val; } KV;

static uint64_t hash64(uint64_t x) {
    x ^= x >> 33; x *= 0xff51afd7ed558ccdULL;
    x ^= x >> 33; x *= 0xc4ceb9fe1a85ec53ULL;
    x ^= x >> 33; return x;
}

typedef struct { KV *tab; size_t mask; size_t used; } HashMap;

static void hm_init(HashMap *hm, size_t cap_pow2) {
    hm->tab = (KV *)calloc(cap_pow2, sizeof(KV));
    hm->mask = cap_pow2 - 1;
    hm->used = 0;
}
static void hm_free(HashMap *hm) { free(hm->tab); }
static void hm_grow(HashMap *hm) {
    size_t old_cap = hm->mask + 1;
    KV *old_tab = hm->tab;
    hm_init(hm, old_cap * 2);
    for (size_t i = 0; i < old_cap; i++) {
        if (old_tab[i].val != 0) {
            size_t idx = hash64(old_tab[i].key) & hm->mask;
            while (hm->tab[idx].val != 0) idx = (idx + 1) & hm->mask;
            hm->tab[idx] = old_tab[i];
            hm->used++;
        }
    }
    free(old_tab);
}
static int hm_get_or_insert(HashMap *hm, uint64_t key, int *next_id, int *is_new) {
    if ((hm->used + 1) * 2 >= hm->mask + 1) hm_grow(hm);
    size_t idx = hash64(key) & hm->mask;
    while (hm->tab[idx].val != 0) {
        if (hm->tab[idx].key == key) { *is_new = 0; return hm->tab[idx].val; }
        idx = (idx + 1) & hm->mask;
    }
    int id = (*next_id)++;
    hm->tab[idx].key = key;
    hm->tab[idx].val = id;
    hm->used++;
    *is_new = 1;
    return id;
}

/* Returns total distinct non-terminal rules g_rl (plus distinct terminals counted separately if desired) */
int exact_jez0_rlslp(const uint8_t *s, int n, int *rounds_out, int *terminals_out) {
    if (n <= 0) return 0;
    int *arr = (int *)malloc((size_t)n * sizeof(int));
    int *nxt = (int *)malloc((size_t)n * sizeof(int));
    int seen_term[256] = {0};
    int terms = 0;
    for (int i = 0; i < n; i++) {
        arr[i] = (int)s[i] + 1; /* 1..256 */
        if (!seen_term[s[i]]) { seen_term[s[i]] = 1; terms++; }
    }
    if (terminals_out) *terminals_out = terms;

    HashMap rle_map, pair_map;
    hm_init(&rle_map, 1024);
    hm_init(&pair_map, 1024);
    int next_id = 257;
    int nonterms = 0;
    int len = n;
    int rounds = 0;

    while (len > 1) {
        rounds++;
        /* Step A: RLE collapse all maximal runs of length >= 2 */
        int out = 0;
        int i = 0;
        while (i < len) {
            int j = i + 1;
            while (j < len && arr[j] == arr[i]) j++;
            int run_len = j - i;
            if (run_len >= 2) {
                uint64_t key = ((uint64_t)(uint32_t)arr[i] << 32) | (uint64_t)(uint32_t)run_len;
                int is_new = 0;
                int id = hm_get_or_insert(&rle_map, key, &next_id, &is_new);
                if (is_new) nonterms++;
                nxt[out++] = id;
            } else {
                nxt[out++] = arr[i];
            }
            i = j;
        }
        int *sw = arr; arr = nxt; nxt = sw;
        len = out;
        if (len <= 1) break;

        /* Step B: Greedy deterministic 2-partition (L/R) maximizing cross-pair count */
        int max_sym = next_id;
        int *side = (int *)calloc((size_t)max_sym, sizeof(int)); /* 0=unassigned, 1=L, 2=R */
        /* Assign symbols in order of appearance, placing each symbol on L or R to maximize already-assigned cross-edges */
        int *degL = (int *)calloc((size_t)max_sym, sizeof(int));
        int *degR = (int *)calloc((size_t)max_sym, sizeof(int));
        /* Simple deterministic greedy pass over adjacent pairs (no adjacent equals after RLE!) */
        for (int k = 0; k + 1 < len; k++) {
            int u = arr[k], v = arr[k + 1];
            if (side[u] == 0) {
                side[u] = (degR[u] >= degL[u]) ? 1 : 2;
            }
            if (side[u] == 1) degL[v]++; else degR[v]++;
            if (side[v] == 0) {
                side[v] = (degL[v] >= degR[v]) ? 2 : 1;
            }
        }
        /* Compress every L -> R pair */
        out = 0;
        i = 0;
        int merged = 0;
        while (i < len) {
            if (i + 1 < len && side[arr[i]] == 1 && side[arr[i + 1]] == 2) {
                uint64_t key = ((uint64_t)(uint32_t)arr[i] << 32) | (uint64_t)(uint32_t)arr[i + 1];
                int is_new = 0;
                int id = hm_get_or_insert(&pair_map, key, &next_id, &is_new);
                if (is_new) nonterms++;
                nxt[out++] = id;
                i += 2;
                merged = 1;
            } else {
                nxt[out++] = arr[i];
                i += 1;
            }
        }
        /* Fallback if somehow 0 merges (shouldn't happen, but safeguard): merge (arr[0], arr[1]) */
        if (!merged && len >= 2) {
            out = 0;
            uint64_t key = ((uint64_t)(uint32_t)arr[0] << 32) | (uint64_t)(uint32_t)arr[1];
            int is_new = 0;
            int id = hm_get_or_insert(&pair_map, key, &next_id, &is_new);
            if (is_new) nonterms++;
            nxt[out++] = id;
            for (int k = 2; k < len; k++) nxt[out++] = arr[k];
        }
        free(side); free(degL); free(degR);
        sw = arr; arr = nxt; nxt = sw;
        len = out;
    }

    if (rounds_out) *rounds_out = rounds;
    hm_free(&rle_map);
    hm_free(&pair_map);
    free(arr); free(nxt);
    return nonterms + terms; /* g_rl = distinct non-terminals + distinct terminals */
}
