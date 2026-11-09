#define _GNU_SOURCE
#include <assert.h>
#include <dirent.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

/* Linux pthread/TLS stand-in only: not an execution of mibz2 or Rust. */
static pthread_key_t key;
static atomic_int started, release_worker, published;
static void publish(void *p) { if (p) atomic_fetch_add(&published, 1); }
static void *worker(void *p) {
    (void)p;
    assert(!pthread_setname_np(pthread_self(), "mibz2-late"));
    assert(!pthread_setspecific(key, (void *)1));
    atomic_store(&started, 1);
    while (!atomic_load(&release_worker)) usleep(100);
    return NULL;
}
static int decoder_threads(void) {
    DIR *d=opendir("/proc/self/task"); if (!d) { perror("opendir /proc/self/task"); exit(77); }
    struct dirent *e; int count=0; char path[512],name[128];
    while ((e=readdir(d))) {
        snprintf(path,sizeof(path),"/proc/self/task/%s/comm",e->d_name);
        FILE *f=fopen(path,"r"); if (!f) continue;
        if (fgets(name,sizeof(name),f) && !strncmp(name,"mibz2",5)) count++;
        fclose(f);
    }
    closedir(d); return count;
}
static void fence(void) {
    for (int i=0;i<5000;i++) {
        if (!decoder_threads()) return;
        usleep(1000);
    }
    assert(!"bounded fence timeout");
}
static void *release_later(void *p) {
    (void)p; usleep(20000); atomic_store(&release_worker,1); return NULL;
}
int main(void) {
    assert(!pthread_key_create(&key,publish));
    for(int fenced=0;fenced<=1;fenced++) {
        atomic_store(&started,0);atomic_store(&release_worker,0);atomic_store(&published,0);
        pthread_t w,r;assert(!pthread_create(&w,NULL,worker,NULL));
        while(!atomic_load(&started)) usleep(100);
        assert(decoder_threads()>0);
        if(fenced) {assert(!pthread_create(&r,NULL,release_later,NULL));fence();assert(!pthread_join(r,NULL));}
        int own=atomic_exchange(&published,0);
        /* The next case resets the common completed-trace list here. */
        atomic_store(&release_worker,1);assert(!pthread_join(w,NULL));fence();
        int next=atomic_exchange(&published,0);
        assert(own==fenced && next==!fenced);
        printf("%s own=%d next=%d\n",fenced?"fenced":"unfenced",own,next);
    }
    assert(!pthread_key_delete(key));return 0;
}
