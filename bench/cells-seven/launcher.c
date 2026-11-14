/* Diagnostic process launcher, never linked to the decoder. No timed estimator.
 * The child calls only async-signal-safe functions between fork and exec.
 * wait4 consumes exactly its PID once; a deadline kills its process group and
 * still reaps it, preserving status and resource evidence on every exit path.
 */
#define _DEFAULT_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/resource.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

static volatile sig_atomic_t expired;
static void deadline(int signal_number) { (void)signal_number; expired = 1; }

int main(int argc, char **argv) {
    if (argc < 3) return 125;
    FILE *record = fopen(argv[1], "wx");
    if (!record) return 125;
    if (fcntl(fileno(record), F_SETFD, FD_CLOEXEC)) return 125;
    struct sigaction action = {0};
    action.sa_handler = deadline;
    sigemptyset(&action.sa_mask);
    if (sigaction(SIGALRM, &action, NULL)) return 125;
    pid_t child = fork();
    if (child < 0) return 125;
    if (!child) {
        if (setpgid(0, 0)) _exit(125);
        execvp(argv[2], argv + 2);
        _exit(127);
    }
    /* Either parent or child establishes the group before a possible deadline. */
    if (setpgid(child, child) && errno != EACCES && errno != ESRCH) {
        kill(child, SIGKILL);
    }
    alarm(30);
    struct rusage usage = {0};
    int status = 0;
    pid_t waited;
    do {
        if (expired) kill(-child, SIGKILL);
        waited = wait4(child, &status, 0, &usage);
    } while (waited < 0 && errno == EINTR);
    alarm(0);
    if (waited != child) return 125;
    int code = WIFEXITED(status) ? WEXITSTATUS(status) : 128 + WTERMSIG(status);
    fprintf(record, "{\"pid\":%ld,\"waited_pid\":%ld,\"exit\":%d,\"timeout\":%s,\"wait4_hwm_kib\":%ld}\n",
            (long)child, (long)waited, code, expired ? "true" : "false", usage.ru_maxrss);
    if (fclose(record)) return 125;
    return expired ? 124 : code;
}
