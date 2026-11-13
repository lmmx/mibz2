/* Untimed touched-memory control for Linux native getrusage/VmHWM/wait4. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <unistd.h>
int main(int argc, char **argv) {
    if (argc != 2) return 2;
    long mib = strtol(argv[1], NULL, 10);
    if (mib != 1 && mib != 64) return 2;
    size_t n = (size_t)mib << 20;
    volatile unsigned char *bytes = malloc(n);
    if (!bytes) return 3;
    for (size_t i = 0; i < n; ++i) bytes[i] = (unsigned char)(i + 1);
    struct rusage usage;
    if (getrusage(RUSAGE_SELF, &usage)) return 4;
    FILE *status = fopen("/proc/self/status", "r");
    if (!status) return 5;
    char line[256]; long hwm = 0;
    while (fgets(line, sizeof line, status))
        if (!strncmp(line, "VmHWM:", 6)) hwm = strtol(line + 6, NULL, 10);
    fclose(status);
    fprintf(stderr, "{\"pid\":%ld,\"mib\":%ld,\"self_hwm_kib\":%ld,\"proc_hwm_kib\":%ld}\n",
            (long)getpid(), mib, usage.ru_maxrss, hwm);
    free((void *)bytes);
    return hwm > 0 ? 0 : 6;
}
