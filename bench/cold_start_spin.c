/* W1's spinners (docs/journal/2026-11-10-cold-start.md).
 *
 *   cold_start_spin burst MS -- COMMAND [ARG...]
 *       one thread pinned to each CPU the process may run on spins for MS ms; when every one
 *       has finished, COMMAND is exec'd (same pid): the spin ends as the child launches.
 *   cold_start_spin hold CPU
 *       one thread pinned to CPU spins until the process is killed.
 */
#define _GNU_SOURCE
#include <pthread.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

static double now(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return t.tv_sec + t.tv_nsec / 1e9;
}

static double until;

static void *spin(void *arg) {
    cpu_set_t s;
    CPU_ZERO(&s);
    CPU_SET((int)(long)arg, &s);
    pthread_setaffinity_np(pthread_self(), sizeof s, &s);
    volatile unsigned long x = 0;
    while (until < 0 || now() < until)
        x++;
    return NULL;
}

int main(int argc, char **argv) {
    if (argc >= 5 && strcmp(argv[1], "burst") == 0 && strcmp(argv[3], "--") == 0) {
        cpu_set_t mask;
        sched_getaffinity(0, sizeof mask, &mask);
        pthread_t th[CPU_SETSIZE];
        int n = 0;
        until = now() + atof(argv[2]) / 1e3;
        for (int c = 0; c < CPU_SETSIZE; c++)
            if (CPU_ISSET(c, &mask) && pthread_create(&th[n], NULL, spin, (void *)(long)c) == 0)
                n++;
        for (int i = 0; i < n; i++)
            pthread_join(th[i], NULL);
        execvp(argv[4], argv + 4);
        perror("cold_start_spin: exec");
        return 127;
    }
    if (argc == 3 && strcmp(argv[1], "hold") == 0) {
        until = -1;
        spin((void *)(long)atoi(argv[2]));
        return 0;
    }
    fprintf(stderr, "usage: cold_start_spin burst MS -- COMMAND [ARG...] | hold CPU\n");
    return 2;
}
