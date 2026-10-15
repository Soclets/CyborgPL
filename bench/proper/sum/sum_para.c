// sum, in parallel the proper C way: split the range across every core with pthreads.
#include <pthread.h>
#include <stdlib.h>
#include <unistd.h>

typedef struct { long long lo, hi, acc; } Part;

static void *work(void *arg) {
    Part *p = arg;
    long long acc = 0;
    for (long long i = p->lo; i <= p->hi; i++) acc += (i * i) % 7;
    p->acc = acc;
    return NULL;
}

int main(void) {
    const char *s = getenv("N");
    long long n = s ? atoll(s) : -1;
    if (n < 0 || n > 1000000000) return 2;
    long k = sysconf(_SC_NPROCESSORS_ONLN);
    if (k < 1) k = 1;
    if (k > 256) k = 256;
    pthread_t th[256];
    Part parts[256];
    for (long t = 0; t < k; t++) {
        parts[t] = (Part){ 1 + n * t / k, n * (t + 1) / k, 0 };
        pthread_create(&th[t], NULL, work, &parts[t]);
    }
    long long acc = 0;
    for (long t = 0; t < k; t++) { pthread_join(th[t], NULL); acc += parts[t].acc; }
    return (int)(acc % 1000000007 % 256);
}
