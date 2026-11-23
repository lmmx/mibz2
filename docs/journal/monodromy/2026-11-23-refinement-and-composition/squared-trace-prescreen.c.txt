// Pre-screen: two-chain chase on T vs gather T^2 then two-chain chase on T^2, random m-cycle.
// Each step emits its word id(s) to out[] at the chain's running offset (the emit's dependent part).
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <time.h>
typedef struct { uint32_t nx, w; } E1;
typedef struct { uint32_t nx, w1, w2, pad; } E2;
static double now(){ struct timespec t; clock_gettime(CLOCK_MONOTONIC,&t); return t.tv_sec*1e9+t.tv_nsec; }
static uint64_t s=88172645463325252ull; static uint64_t rnd(){ s^=s<<13; s^=s>>7; s^=s<<17; return s; }
int main(int argc,char**argv){
  uint32_t m=atoi(argv[1]); int reps=atoi(argv[2]);
  uint32_t *perm=malloc(m*4); for(uint32_t i=0;i<m;i++) perm[i]=i;
  for(uint32_t i=m-1;i>0;i--){ uint32_t j=rnd()%(i+1); uint32_t t=perm[i]; perm[i]=perm[j]; perm[j]=t; }
  E1 *T=malloc(m*sizeof(E1)); for(uint32_t i=0;i<m;i++){ T[perm[i]].nx=perm[(i+1)%m]; T[perm[i]].w=rnd()&0xffff; }
  E2 *T2=malloc(m*sizeof(E2)); uint32_t *N2=malloc(m*4); uint32_t *out=malloc((m+2)*4);
  uint32_t a0=perm[0], b0=perm[m/2];            // two chains, half the cycle each (two ends stand-in)
  double best1=1e30,best2=1e30,bestg=1e30; uint64_t chk=0;
  for(int r=0;r<reps;r++){
    double t0=now(); uint32_t a=a0,b=b0; uint32_t h=m/2;
    for(uint32_t i=0;i<h;i++){ E1 ea=T[a], eb=T[b]; out[i]=ea.w; out[h+i]=eb.w; a=ea.nx; b=eb.nx; }
    double t1=now(); if(t1-t0<best1) best1=t1-t0; chk+=out[m/3];
    t0=now();
    for(uint32_t i=0;i<m;i++){ N2[i]=T[T[i].nx].nx; }
    double tg=now();
    a=a0;b=b0; uint32_t q=h/2;
    for(uint32_t i=0;i<q;i++){ uint32_t na=N2[a], nb=N2[b]; E1 ea=T[a], eb=T[b]; out[2*i]=ea.w; out[2*i+1]=T[ea.nx].w; out[h+2*i]=eb.w; out[h+2*i+1]=T[eb.nx].w; a=na; b=nb; }
    t1=now(); if(tg-t0<bestg) bestg=tg-t0; if(t1-tg<best2) best2=t1-tg; chk+=out[m/5];
  }
  printf("m %u: chase T %.2f ns/row | gather T^2 %.2f ns/row + chase T^2 %.2f ns/row = %.2f ns/row | ratio %.3f (chk %llu)\n",
    m,best1/m,bestg/m,best2/m,(bestg+best2)/m,(bestg+best2)/best1,(unsigned long long)chk);
}
