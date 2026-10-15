// lists, in parallel the proper C way: one malloc, then each pthread fills its own part and sums it
// ten times.
#include <pthread.h>
#include <stdlib.h>
#include <unistd.h>

typedef struct { long long *xs, lo, hi, acc; } Part;
static void *work(void *arg) {
    Part *p = arg;
    for (long long i = p->lo; i < p->hi; i++) p->xs[i] = (i + 1) % 1000;
    long long acc = 0;
    for (int r = 0; r < 10; r++)
        for (long long i = p->lo; i < p->hi; i++) acc += p->xs[i];
    p->acc = acc;
    return NULL;
}

int main(void) {
    const char *s = getenv("N");
    long long n = s ? atoll(s) : -1;
    if (n < 0 || n > 100000000) return 2;
    long long *xs = malloc((size_t)(n ? n : 1) * sizeof *xs);
    if (!xs) return 3;
    long k = sysconf(_SC_NPROCESSORS_ONLN);
    if (k < 1) k = 1;
    if (k > 256) k = 256;
    pthread_t th[256]; Part parts[256];
    for (long t = 0; t < k; t++) { parts[t] = (Part){ xs, n * t / k, n * (t + 1) / k, 0 }; pthread_create(&th[t], NULL, work, &parts[t]); }
    long long acc = 0;
    for (long t = 0; t < k; t++) { pthread_join(th[t], NULL); acc += parts[t].acc; }
    free(xs);
    return (int)(acc % 256);
}
