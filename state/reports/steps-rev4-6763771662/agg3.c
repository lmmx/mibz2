// agg3: candidate false store-to-load dependences, counted INDEPENDENTLY per class.
// Reads valgrind lackey --trace-mem=yes output (or a synthetic trace in the same format) on
// stdin. argv: name:start:end ... (absolute hex PC ranges of the functions whose loads count),
// optionally "--image=lo:hi" (the binary's static image range; default 0x100000:0x1000000)
// and any number of "--named=SPC:LPC[,SPC:LPC...]" (absolute hex store PC, load PC pairs).
//
// For each load (L or M) by a listed function, and for each modulus m in {4096, 1024}: for
// each region class of the store (S stack >= 0x1000000000; D the static image; H anything
// else: heap and mmaps) it records whether ANY of the last NST stores (any function) of that
// class overlaps the load's bytes modulo m at a different real address. A load can count in
// several classes at once; no class consumes another (the defect of agg.c, which took the
// first match in ring order and recorded only its class: D086).
//
// COUNT UNITS (frozen; printed in the UNITS line):
// - class count (the "mod 4K"/"mod 1K" lines): LOADS. A load counts once in a class however
//   many ring stores of that class match it. This unit is unchanged from the original agg3.
// - named / outside / both (NAMED lines): LOADS of that class, counted directly per load:
//   named   = loads with >= 1 matching store whose (store PC, load PC) is a named pair;
//   outside = loads with >= 1 matching store whose pair is NOT named;
//   both    = loads with both. class = named + outside - both, checked; neither population is
//   ever obtained by subtraction.
// - pair loads (PAIR / NAMEDPAIR "loads="): LOADS in which that (store PC, load PC) pair
//   matched at least once. Pairs sharing a load each count it, so pair loads do not sum to a
//   class count.
// - pair weight ("weight=", and the legacy PAIRS "n="): MATCHING STORED ACCESSES, i.e. one per
//   matching store in the ring, so two stores at one PC matching one load give weight 2.
//   NAMED's "weight named= outside=" are sums of pair weight over named and non-named pairs.
//   (The original comment here said pairs were "counted once per load"; that was false: the
//   legacy n= was, and is, this stored-access weight.)
// Output is complete: every pair of every class (PAIR lines), every requested named pair
// with explicit zeros (NAMEDPAIR), then UNITS and a final "END agg3 complete" line. The legacy
// PAIRS line (five heaviest pairs per class) is a summary only, never a count source.
// These are trace candidates, not stalls: a store among the last NST observed need not be
// outstanding when a native load executes.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define NST 48
#define NF 16
#define NNAMED 256
#define STACK_LO 0x1000000000ULL
typedef unsigned long long u64;
static u64 IMG_LO=0x100000, IMG_HI=0x1000000;
static char *nm[NF]; static u64 lo[NF], hi[NF]; static int nf;
static u64 sa[NST], sip[NST]; static int ssz[NST], sp, filled;
static u64 loads[NF][3];            // [f][load region]
static u64 cnt[NF][3][3][2];        // [f][load region][store region][m: 0=4K 1=1K]
static u64 anyc[NF][2];
static u64 cn[NF][3][3][2], co[NF][3][3][2], cb[NF][3][3][2];  // named, outside, both (loads)
static u64 wn[NF][3][3][2], wo[NF][3][3][2];                   // named, outside (weight)
static u64 nspc[NNAMED], nlpc[NNAMED]; static int nn;
// pair table
#define HP (1<<20)
static struct { u64 key1, key2, w, l; int used; } T[HP];
static int reg(u64 a){ return a>=STACK_LO?0: (a>=IMG_LO&&a<IMG_HI)?1:2; } // 0 S,1 D,2 H
static const char *RN="SDH";
static int fn_of(u64 ia){ for(int i=0;i<nf;i++) if(ia>=lo[i]&&ia<hi[i]) return i; return -1; }
static int overl(u64 a,int as,u64 b,int bs,u64 m){ for(int i=0;i<as;i++){ u64 x=(a+i)&(m-1); for(int j=0;j<bs;j++) if(((b+j)&(m-1))==x) return 1; } return 0; }
static int named(u64 spc,u64 lpc){ for(int i=0;i<nn;i++) if(nspc[i]==spc&&nlpc[i]==lpc) return 1; return 0; }
static u64 kkey(u64 spc,u64 lpc){ return (spc<<32)|(lpc&0xffffffffULL); } // PCs < 2^32 under valgrind
static long pair_find(u64 k1,u64 kk,int create){
  u64 k2=(kk>>32)*1000003ULL^(kk&0xffffffffULL);   // the original agg3's hash, so the legacy
  u64 h=(k1*0x9e3779b97f4a7c15ULL^k2*0xc2b2ae3d27d4eb4fULL)&(HP-1);  // PAIRS tie order is unchanged
  for(;;){ if(!T[h].used){ if(!create) return -1; T[h].used=1; T[h].key1=k1; T[h].key2=kk; return (long)h; }
           if(T[h].key1==k1 && T[h].key2==kk) return (long)h;
           h=(h+1)&(HP-1); }
}
static u64 K1(int f,int lr,int sr,int mi){ return ((u64)f<<8)|(lr<<4)|(sr<<2)|mi; }
static int cmpkk(const void*a,const void*b){ u64 x=T[*(const long*)a].key2, y=T[*(const long*)b].key2; return x<y?-1:x>y; }
static void addnamed(char *s){
  for(char *t=strtok(s,","); t; t=strtok(0,",")){
    char *e; u64 a=strtoull(t,&e,16); if(*e!=':'){ fprintf(stderr,"agg3: bad --named %s\n",t); exit(2); }
    if(nn>=NNAMED){ fprintf(stderr,"agg3: too many named pairs\n"); exit(2); }
    nspc[nn]=a; nlpc[nn]=strtoull(e+1,0,16); nn++; }
}
int main(int argc,char**argv){
  for(int i=1;i<argc;i++){
    if(!strncmp(argv[i],"--image=",8)){ char*s=argv[i]+8; IMG_LO=strtoull(s,&s,16); IMG_HI=strtoull(s+1,0,16); continue; }
    if(!strncmp(argv[i],"--named=",8)){ addnamed(strdup(argv[i]+8)); continue; }
    if(nf>=NF){ fprintf(stderr,"agg3: too many functions\n"); return 2; }
    char *s=strdup(argv[i]); char *a=strtok(s,":"), *b=strtok(0,":"), *c=strtok(0,":");
    if(!a||!b||!c){ fprintf(stderr,"agg3: bad function %s\n",argv[i]); return 2; }
    nm[nf]=a; lo[nf]=strtoull(b,0,16); hi[nf]=strtoull(c,0,16); nf++; }
  char line[256]; int cur=-1; u64 ia=0, nlines=0;
  while(fgets(line,sizeof line,stdin)){
    char k; u64 a; int sz; nlines++;
    if(line[0]=='I'){ if(sscanf(line+1," %llx,%d",&ia,&sz)==2) cur=fn_of(ia); continue; }
    if(line[0]!=' ' || sscanf(line," %c %llx,%d",&k,&a,&sz)!=3) continue;
    if((k=='L'||k=='M') && cur>=0){
      int lr=reg(a); loads[cur][lr]++;
      int hit[3][2]={{0}}; int any[2]={0,0};
      int nh[3][2]={{0}}, oh[3][2]={{0}};
      struct { int sr, mi; u64 spc; } mt[2*NST]; int nmt=0;
      for(int i=0;i<(filled<NST?filled:NST);i++){ u64 b=sa[i]; int z=ssz[i];
        if(a<b+z && b<a+sz) continue;                    // a real overlap: a true dependence
        int sr=reg(b);
        for(int mi=0;mi<2;mi++){ u64 m=mi?1024:4096;
          if(overl(a,sz,b,z,m)){ hit[sr][mi]=1; any[mi]=1;
            mt[nmt].sr=sr; mt[nmt].mi=mi; mt[nmt].spc=sip[i]; nmt++;
            if(named(sip[i],ia)){ nh[sr][mi]=1; wn[cur][lr][sr][mi]++; } else { oh[sr][mi]=1; wo[cur][lr][sr][mi]++; } } } }
      // pairs: weight += every matching stored access; loads += 1 per distinct pair per load
      for(int j=0;j<nmt;j++){
        long h=pair_find(K1(cur,lr,mt[j].sr,mt[j].mi),kkey(mt[j].spc,ia),1); T[h].w++;
        int seen=0; for(int q=0;q<j;q++) if(mt[q].sr==mt[j].sr&&mt[q].mi==mt[j].mi&&mt[q].spc==mt[j].spc){ seen=1; break; }
        if(!seen) T[h].l++; }
      for(int sr=0;sr<3;sr++) for(int mi=0;mi<2;mi++){
        cnt[cur][lr][sr][mi]+=hit[sr][mi];
        cn[cur][lr][sr][mi]+=nh[sr][mi]; co[cur][lr][sr][mi]+=oh[sr][mi]; cb[cur][lr][sr][mi]+=nh[sr][mi]&&oh[sr][mi]; }
      for(int mi=0;mi<2;mi++) anyc[cur][mi]+=any[mi];
    }
    if(k=='S'||k=='M'){ sa[sp]=a; sip[sp]=ia; ssz[sp]=sz>64?64:sz; sp=(sp+1)%NST; if(filled<NST) filled++; }
  }
  for(int f=0;f<nf;f++){
    printf("%s loads S=%llu D=%llu H=%llu | any4K=%llu any1K=%llu\n",nm[f],loads[f][0],loads[f][1],loads[f][2],anyc[f][0],anyc[f][1]);
    for(int mi=0;mi<2;mi++){ printf("  mod %s (load<-store):",mi?"1K":"4K");
      for(int lr=0;lr<3;lr++) for(int sr=0;sr<3;sr++) if(cnt[f][lr][sr][mi]) printf(" %c<-%c=%llu",RN[lr],RN[sr],cnt[f][lr][sr][mi]); printf("\n"); }
  }
  // legacy summary: the five heaviest pairs (by weight) per (f, lr, sr, m); first found wins ties
  static unsigned char taken[HP];
  for(int f=0;f<nf;f++) for(int mi=0;mi<2;mi++) for(int lr=0;lr<3;lr++) for(int sr=0;sr<3;sr++){
    u64 k1=K1(f,lr,sr,mi); int any=0;
    for(int r=0;r<5;r++){ long best=-1; for(long h=0;h<HP;h++) if(T[h].used && !taken[h] && T[h].key1==k1 && (best<0||T[h].w>T[best].w)) best=h;
      if(best<0) break; if(!any){ printf("PAIRS %s mod%s %c<-%c:",nm[f],mi?"1K":"4K",RN[lr],RN[sr]); any=1; }
      printf(" [store %llx load %llx n=%llu]",T[best].key2>>32,T[best].key2&0xffffffffULL,T[best].w); taken[best]=1; }
    if(any) printf("\n"); }
  // complete pair listing, every pair of every class, ordered by (store PC, load PC)
  long *ix=malloc(sizeof(long)*HP); u64 npairs=0;
  for(int f=0;f<nf;f++) for(int mi=0;mi<2;mi++) for(int lr=0;lr<3;lr++) for(int sr=0;sr<3;sr++){
    u64 k1=K1(f,lr,sr,mi); long n=0;
    for(long h=0;h<HP;h++) if(T[h].used && T[h].key1==k1) ix[n++]=h;
    qsort(ix,n,sizeof(long),cmpkk);
    for(long j=0;j<n;j++){ long h=ix[j]; npairs++;
      printf("PAIR %s mod%s %c<-%c: store %llx load %llx loads=%llu weight=%llu named=%d\n",nm[f],mi?"1K":"4K",RN[lr],RN[sr],
             T[h].key2>>32,T[h].key2&0xffffffffULL,T[h].l,T[h].w,named(T[h].key2>>32,T[h].key2&0xffffffffULL)); } }
  // named summaries: for every function holding a named load PC, every class, explicit zeros
  int bad=0;
  if(nn) for(int f=0;f<nf;f++){
    int has=0; for(int i=0;i<nn;i++) if(nlpc[i]>=lo[f]&&nlpc[i]<hi[f]) has=1;
    if(!has) continue;
    for(int mi=0;mi<2;mi++) for(int lr=0;lr<3;lr++) for(int sr=0;sr<3;sr++){
      u64 c=cnt[f][lr][sr][mi], n=cn[f][lr][sr][mi], o=co[f][lr][sr][mi], b=cb[f][lr][sr][mi];
      if(c!=n+o-b) bad=1;
      printf("NAMED %s mod%s %c<-%c: class=%llu named=%llu outside=%llu both=%llu | weight named=%llu outside=%llu\n",
             nm[f],mi?"1K":"4K",RN[lr],RN[sr],c,n,o,b,wn[f][lr][sr][mi],wo[f][lr][sr][mi]);
      for(int i=0;i<nn;i++){ if(!(nlpc[i]>=lo[f]&&nlpc[i]<hi[f])) continue;
        long h=pair_find(K1(f,lr,sr,mi),kkey(nspc[i],nlpc[i]),0);
        printf("NAMEDPAIR %s mod%s %c<-%c: store %llx load %llx loads=%llu weight=%llu\n",nm[f],mi?"1K":"4K",RN[lr],RN[sr],
               nspc[i],nlpc[i],h<0?0ULL:T[h].l,h<0?0ULL:T[h].w); } }
    }
  printf("UNITS class,named,outside,both,pair-loads=loads; weight,n=matching stored accesses (ring %d)\n",NST);
  if(bad){ printf("ERROR class != named + outside - both\n"); return 3; }
  printf("END agg3 complete lines=%llu pairs=%llu named=%d\n",nlines,npairs,nn);
  return 0;
}
