/*
 * Profile breakdown of T_con and T_exp on block-0.L (and sum of 6 blocks)
 * so we separate libc qsort overhead from irreducible binary-search / SLP-expand costs!
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/* Include the exact bench_btm_native logic with fine-grained timers inside resolve_subset_c */
typedef struct {
    int32_t lo, hi, delta;
    uint32_t slp_id;
    int32_t mark_off;
    uint8_t tsym, is_int;
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

typedef struct { uint32_t a, b, len; } SlpNode;
typedef struct { SlpNode *nodes; uint32_t len, cap; } SlpStore;
static void slp_init(SlpStore *s, uint32_t cap) {
    s->cap = cap; s->nodes = (SlpNode *)malloc((size_t)cap * sizeof(SlpNode));
    s->len = 256;
    for (uint32_t c = 0; c < 256; c++) s->nodes[c] = (SlpNode){c, 0, 1};
}
static inline uint32_t slp_concat(SlpStore *s, uint32_t u, uint32_t v) {
    if (s->len == s->cap) { s->cap *= 2; s->nodes = (SlpNode *)realloc(s->nodes, (size_t)s->cap * sizeof(SlpNode)); }
    uint32_t id = s->len++;
    s->nodes[id] = (SlpNode){u, v, s->nodes[u].len + s->nodes[v].len};
    return id;
}
static inline int32_t combine_mark(int32_t m1, uint32_t len1, int32_t m2) {
    return (m1 >= 0) ? m1 : ((m2 >= 0) ? (int32_t)len1 + m2 : -1);
}
typedef struct { int32_t a, b; } Range;
static int cmp_range(const void *pa, const void *pb) {
    int32_t a = ((const Range *)pa)->a, b = ((const Range *)pb)->a;
    return (a > b) - (a < b);
}
static int cmp_piece_lo(const void *pa, const void *pb) {
    int32_t a = ((const Piece *)pa)->lo, b = ((const Piece *)pb)->lo;
    return (a > b) - (a < b);
}
static double now_ms(void) {
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1e3 + (double)ts.tv_nsec * 1e-6;
}

typedef struct { uint64_t bits[4]; } SymSet;
static inline void ss_clear(SymSet *s) { memset(s, 0, sizeof(*s)); }
static inline void ss_add(SymSet *s, uint8_t c) { s->bits[c >> 6] |= (1ULL << (c & 63)); }
static inline int ss_has(const SymSet *s, uint8_t c) { return (int)((s->bits[c >> 6] >> (c & 63)) & 1ULL); }
static inline SymSet ss_union(SymSet a, SymSet b) {
    SymSet r; for (int i = 0; i < 4; i++) r.bits[i] = a.bits[i] | b.bits[i]; return r;
}

typedef struct {
    PieceVec collapsed, stack;
    Range *int_targets, *merged_t;
    int32_t cap_t;
    int32_t sym_off[257];
    double t_merge, t_sort_t, t_in_scan, t_bounce, t_sort_out;
    int64_t q_rand, bounces;
} ProfScratch;

static inline int32_t find_slice(const Piece *arr, int32_t l, int32_t r, int32_t target) {
    while (l + 1 < r) {
        int32_t m = l + ((r - l) >> 1);
        if (arr[m].lo <= target) l = m; else r = m;
    }
    return l;
}

static void resolve_prof(
    const SymSet *sset, const PieceVec *p1, const PieceVec *p2,
    const int32_t *C, SlpStore *slp, ProfScratch *sc, PieceVec *out,
    int *has_pcyc, uint32_t *pcyc_slp, int32_t *pcyc_mark
) {
    double t0 = now_ms();
    sc->collapsed.len = 0;
    int32_t i = 0, j = 0, n1 = p1->len, n2 = p2 ? p2->len : 0;
    while (i < n1 || j < n2) {
        Piece p = (j >= n2 || (i < n1 && p1->data[i].lo < p2->data[j].lo)) ? p1->data[i++] : p2->data[j++];
        p.is_int = (uint8_t)ss_has(sset, p.tsym);
        if (p.is_int && p.delta == 0) {
            if (p.mark_off >= 0) { *has_pcyc = 1; *pcyc_slp = p.slp_id; *pcyc_mark = p.mark_off; }
            continue;
        }
        pv_push(&sc->collapsed, p);
    }
    int32_t nc = sc->collapsed.len;
    const Piece *col = sc->collapsed.data;
    int32_t cur_c = 0;
    for (int32_t idx = 0; idx < nc; idx++) {
        int32_t lo = col[idx].lo;
        while (cur_c < 256 && C[cur_c + 1] <= lo) { sc->sym_off[cur_c + 1] = idx; cur_c++; }
    }
    while (cur_c < 256) { sc->sym_off[cur_c + 1] = nc; cur_c++; }
    sc->sym_off[0] = 0;

    if (nc > sc->cap_t) {
        sc->cap_t = nc * 2;
        sc->int_targets = (Range *)realloc(sc->int_targets, (size_t)sc->cap_t * sizeof(Range));
        sc->merged_t = (Range *)realloc(sc->merged_t, (size_t)sc->cap_t * sizeof(Range));
    }
    int32_t nt = 0, primary_pos = -1;
    for (int32_t k = 0; k < nc; k++) {
        if (col[k].mark_off >= 0) primary_pos = col[k].lo;
        if (col[k].is_int) {
            sc->int_targets[nt].a = col[k].lo + col[k].delta;
            sc->int_targets[nt].b = col[k].hi + col[k].delta;
            nt++;
        }
    }
    double t1 = now_ms();
    sc->t_merge += (t1 - t0);

    if (nt == 0) {
        if (nc > out->cap) { out->cap = nc; out->data = (Piece *)realloc(out->data, (size_t)nc * sizeof(Piece)); }
        memcpy(out->data, col, (size_t)nc * sizeof(Piece));
        out->len = nc;
        return;
    }

    qsort(sc->int_targets, (size_t)nt, sizeof(Range), cmp_range);
    double t2 = now_ms();
    sc->t_sort_t += (t2 - t1);

    int32_t nm = 0;
    for (int32_t k = 0; k < nt; k++) {
        if (nm > 0 && sc->merged_t[nm - 1].b == sc->int_targets[k].a) sc->merged_t[nm - 1].b = sc->int_targets[k].b;
        else sc->merged_t[nm++] = sc->int_targets[k];
    }
    sc->stack.len = 0; out->len = 0;
    int32_t mt_idx = 0; int saw_p = 0;
    for (int32_t k = 0; k < nc; k++) {
        Piece p = col[k]; int32_t pos = p.lo;
        while (pos < p.hi) {
            while (mt_idx < nm && sc->merged_t[mt_idx].b <= pos) mt_idx++;
            if (mt_idx >= nm || sc->merged_t[mt_idx].a >= p.hi) {
                Piece seg = p; seg.lo = pos; seg.hi = p.hi;
                if (!seg.is_int) { pv_push(out, seg); if (seg.mark_off >= 0) saw_p = 1; }
                else pv_push(&sc->stack, seg);
                pos = p.hi;
            } else {
                int32_t ta = sc->merged_t[mt_idx].a, tb = sc->merged_t[mt_idx].b;
                if (pos < ta) {
                    Piece seg = p; seg.lo = pos; seg.hi = ta;
                    if (!seg.is_int) { pv_push(out, seg); if (seg.mark_off >= 0) saw_p = 1; }
                    else pv_push(&sc->stack, seg);
                    pos = ta;
                }
                pos = (p.hi < tb) ? p.hi : tb;
            }
        }
    }
    double t3 = now_ms();
    sc->t_in_scan += (t3 - t2);

    while (sc->stack.len > 0) {
        Piece cur = sc->stack.data[--sc->stack.len];
        sc->bounces++;
        int32_t t_lo = cur.lo + cur.delta, t_hi = cur.hi + cur.delta;
        uint8_t ts = cur.tsym;
        int32_t idx = find_slice(col, sc->sym_off[ts], sc->sym_off[ts + 1], t_lo);
        int32_t pos = t_lo;
        uint32_t w_len = slp->nodes[cur.slp_id].len;
        while (pos < t_hi) {
            sc->q_rand++;
            const Piece *q = &col[idx];
            int32_t ov_hi = (t_hi < q->hi) ? t_hi : q->hi;
            Piece nxt = { pos - cur.delta, ov_hi - cur.delta, cur.delta + q->delta,
                          slp_concat(slp, cur.slp_id, q->slp_id),
                          combine_mark(cur.mark_off, w_len, q->mark_off), q->tsym, q->is_int };
            if (!nxt.is_int) { pv_push(out, nxt); if (nxt.mark_off >= 0) saw_p = 1; }
            else pv_push(&sc->stack, nxt);
            pos = ov_hi; idx++;
        }
    }
    if (primary_pos >= 0 && !saw_p && !(*has_pcyc)) {
        int32_t idx = find_slice(col, 0, nc, primary_pos);
        sc->q_rand++;
        const Piece *p0 = &col[idx];
        uint32_t w_acc = p0->slp_id; int32_t m_acc = p0->mark_off;
        int32_t x = primary_pos + p0->delta; uint8_t ts = p0->tsym;
        while (x != primary_pos) {
            sc->q_rand++;
            idx = find_slice(col, sc->sym_off[ts], sc->sym_off[ts + 1], x);
            const Piece *q = &col[idx];
            w_acc = slp_concat(slp, w_acc, q->slp_id);
            x += q->delta; ts = q->tsym;
        }
        *has_pcyc = 1; *pcyc_slp = w_acc; *pcyc_mark = m_acc;
    }
    double t4 = now_ms();
    sc->t_bounce += (t4 - t3);

    if (out->len > 1) qsort(out->data, (size_t)out->len, sizeof(Piece), cmp_piece_lo);
    double t5 = now_ms();
    sc->t_sort_out += (t5 - t4);
}

typedef struct { int left, right, sym; } TreeNode;
static int build_tree(const int *ps, int ns, const int64_t rw0[256][256], const int32_t pw0[256][256], const int32_t lsz[256], TreeNode *out) {
    int active[512], n_act = ns;
    static int64_t rw[512][512]; static int32_t pw[512][512]; int32_t sz[512];
    for (int i = 0; i < ns; i++) { out[i] = (TreeNode){-1,-1,ps[i]}; active[i] = i; sz[i] = lsz[ps[i]]; }
    for (int i = 0; i < ns; i++) for (int j = 0; j < ns; j++) {
        rw[i][j] = (i==j)?0:(rw0[ps[i]][ps[j]] + rw0[ps[j]][ps[i]]);
        pw[i][j] = (i==j)?0:(pw0[ps[i]][ps[j]] + pw0[ps[j]][ps[i]]);
    }
    int rp = 0; int64_t bd = 1LL<<60;
    for (int i = 0; i < n_act; i++) {
        int64_t d = 0; for (int j = 0; j < n_act; j++) d += rw[active[i]][active[j]];
        if (d < bd) { bd = d; rp = i; }
    }
    int rid = active[rp]; active[rp] = active[--n_act];
    int nid = ns;
    while (n_act > 1) {
        int bi=0, bj=1; double bsc=-1.0;
        for (int i=0;i<n_act;i++) for (int j=i+1;j<n_act;j++) {
            int u=active[i], v=active[j];
            if (rw[u][v] <= 0) continue;
            double sc = ((double)rw[u][v]*(double)rw[u][v])/(((double)pw[u][v]+1.0)*((double)(sz[u]+sz[v])+1.0));
            if (sc > bsc) { bsc=sc; bi=i; bj=j; }
        }
        int u=active[bi], v=active[bj], cid=nid++;
        out[cid]=(TreeNode){u,v,-1}; sz[cid]=sz[u]+sz[v];
        active[bj]=active[--n_act]; active[bi]=active[--n_act];
        for (int i=0;i<n_act;i++) {
            int nb=active[i];
            rw[cid][nb]=rw[nb][cid]=rw[u][nb]+rw[v][nb];
            pw[cid][nb]=pw[nb][cid]=pw[u][nb]+pw[v][nb];
        }
        rw[cid][cid]=0; pw[cid][cid]=0; active[n_act++]=cid;
    }
    int root = nid++; out[root]=(TreeNode){active[0], rid, -1}; return root;
}

static void eval_rec(int id, const TreeNode *tr, PieceVec *leaf, const int32_t *C, SlpStore *slp, ProfScratch *sc,
                     SymSet *oss, PieceVec *opv, int *hp, uint32_t *ps, int32_t *pm) {
    if (tr[id].left < 0) {
        int c = tr[id].sym; ss_clear(oss); ss_add(oss, (uint8_t)c); *opv = leaf[c]; leaf[c].data = NULL; return;
    }
    SymSet sl, sr; PieceVec pl, pr;
    eval_rec(tr[id].left, tr, leaf, C, slp, sc, &sl, &pl, hp, ps, pm);
    eval_rec(tr[id].right, tr, leaf, C, slp, sc, &sr, &pr, hp, ps, pm);
    *oss = ss_union(sl, sr); pv_init(opv, pl.len + pr.len + 16);
    resolve_prof(oss, &pl, &pr, C, slp, sc, opv, hp, ps, pm);
    pv_free(&pl); pv_free(&pr);
}

int main(void) {
    const char *files[6] = {"block-0.L","block-2.L","block-85.L","block-143.L","block-210.L","block-207.L"};
    const int32_t ns[6] = {899931,900000,899935,899869,899959,899998};
    const int32_t ps_arr[6] = {264045,320807,325088,802868,791429,205180};
    printf("block       |  merge  sort_T  in_scan  BOUNCE  sort_out |  T_con  | ns/Q_bounce\n");
    printf("------------+-------------------------------------------+---------+------------\n");
    double sm=0, st=0, si=0, sb=0, so=0; int64_t sq=0;
    for (int bi = 0; bi < 6; bi++) {
        char path[256]; snprintf(path, sizeof(path), "/tmp/mibz2-blocks/%s", files[bi]);
        FILE *fp = fopen(path, "rb"); int32_t n = ns[bi], p = ps_arr[bi];
        uint8_t *L = (uint8_t *)malloc((size_t)n); fread(L, 1, (size_t)n, fp); fclose(fp);
        int32_t counts[256]={0};
        for (int32_t i=0;i<n;i++) counts[L[i]]++;
        int32_t C[257], acc=0, psyms[256], nsyms=0;
        for (int c=0;c<256;c++){ C[c]=acc; if(counts[c]>0) psyms[nsyms++]=c; acc+=counts[c]; } C[256]=acc;
        int32_t cur_t[256]; memcpy(cur_t, C, sizeof(cur_t));
        PieceVec raw[256]; memset(raw, 0, sizeof(raw));
        for (int i=0;i<nsyms;i++) pv_init(&raw[psyms[i]], 64);
        static int64_t rw[256][256]; static int32_t pw[256][256];
        memset(rw,0,sizeof(rw)); memset(pw,0,sizeof(pw));
        int b_idx=0; int32_t r0=0;
        for (int32_t i=0; i<n; ) {
            uint8_t c=L[i]; int32_t j=i+1; while(j<n && L[j]==c) j++;
            int32_t ln=j-i, delta=cur_t[c]-i; cur_t[c]+=ln; r0++;
            int32_t pos=i;
            while (pos<j) {
                while (b_idx<nsyms && C[psyms[b_idx]+1]<=pos) b_idx++;
                uint8_t bc=(uint8_t)psyms[b_idx]; int32_t sh=(j<C[bc+1])?j:C[bc+1];
                rw[bc][c]+=sh-pos; pw[bc][c]++;
                if (pos<=p && p<sh) {
                    if (pos<p) pv_push(&raw[bc],(Piece){pos,p,delta,c,-1,c,0});
                    pv_push(&raw[bc],(Piece){p,p+1,delta,c,0,c,0});
                    if (p+1<sh) pv_push(&raw[bc],(Piece){p+1,sh,delta,c,-1,c,0});
                } else pv_push(&raw[bc],(Piece){pos,sh,delta,c,-1,c,0});
                pos=sh;
            }
            i=j;
        }
        free(L);
        int32_t lsz[256]={0}; for (int i=0;i<nsyms;i++) lsz[psyms[i]]=raw[psyms[i]].len;
        TreeNode tr[512]; int root = build_tree(psyms, nsyms, rw, pw, lsz, tr);
        SlpStore slp; slp_init(&slp, (uint32_t)(r0*4+1024));
        ProfScratch sc; memset(&sc, 0, sizeof(sc));
        pv_init(&sc.collapsed, 1<<17); pv_init(&sc.stack, 1<<17);
        sc.cap_t = 1<<17; sc.int_targets=(Range*)malloc((size_t)sc.cap_t*sizeof(Range));
        sc.merged_t=(Range*)malloc((size_t)sc.cap_t*sizeof(Range));
        int hp=0; uint32_t ps_id=0; int32_t pm_off=-1;
        PieceVec leaf[256]; memset(leaf, 0, sizeof(leaf));
        for (int i=0;i<nsyms;i++) {
            uint8_t c=(uint8_t)psyms[i]; SymSet ss; ss_clear(&ss); ss_add(&ss,c);
            pv_init(&leaf[c], raw[c].len+8);
            resolve_prof(&ss, &raw[c], NULL, C, &slp, &sc, &leaf[c], &hp, &ps_id, &pm_off);
            pv_free(&raw[c]);
        }
        SymSet rss; PieceVec rpv;
        eval_rec(root, tr, leaf, C, &slp, &sc, &rss, &rpv, &hp, &ps_id, &pm_off);
        pv_free(&rpv);
        double tot = sc.t_merge + sc.t_sort_t + sc.t_in_scan + sc.t_bounce + sc.t_sort_out;
        printf("%-11s | %6.2f  %6.2f  %7.2f  %6.2f  %8.2f | %6.2f  | %6.1f ns\n",
               files[bi], sc.t_merge, sc.t_sort_t, sc.t_in_scan, sc.t_bounce, sc.t_sort_out,
               tot, (sc.t_bounce * 1e6) / (double)sc.q_rand);
        sm+=sc.t_merge; st+=sc.t_sort_t; si+=sc.t_in_scan; sb+=sc.t_bounce; so+=sc.t_sort_out; sq+=sc.q_rand;
        pv_free(&sc.collapsed); pv_free(&sc.stack); free(sc.int_targets); free(sc.merged_t); free(slp.nodes);
    }
    printf("------------+-------------------------------------------+---------+------------\n");
    printf("SUM 6 BLKS  | %6.2f  %6.2f  %7.2f  %6.2f  %8.2f | %6.2f  | %6.1f ns\n",
           sm, st, si, sb, so, sm+st+si+sb+so, (sb*1e6)/(double)sq);
    return 0;
}
