// sieve, in parallel the proper C way: the primes up to √N first, then each pthread crosses off and
// counts its own part of one calloc'd byte array.
#include <pthread.h>
#include <stdlib.h>
#include <unistd.h>

typedef struct { unsigned char *comp; long long *base; long long nb, lo, hi, count; } Part;
static void *work(void *arg) {
    Part *p = arg;
    for (long long b = 0; b < p->nb; b++) {
        long long q = p->base[b];
        long long m = (p->lo + q - 1) / q * q;
        if (m < q * q) m = q * q;
        for (; m <= p->hi; m += q) p->comp[m] = 1;
    }
    long long count = 0;
    for (long long i = p->lo; i <= p->hi; i++) count += !p->comp[i];
    p->count = count;
    return NULL;
}

int main(void) {
    const char *s = getenv("N");
    long long n = s ? atoll(s) : -1;
    if (n < 0 || n > 200000000) return 2;
    if (n < 2) return 0;
    unsigned char *comp = calloc((size_t)n + 1, 1);
    if (!comp) return 3;
    long long r = 1; while ((r + 1) * (r + 1) <= n) r++;
    unsigned char *small = calloc((size_t)r + 1, 1);
    long long *base = malloc(sizeof *base * (size_t)(r + 1)); long long nb = 0;
    for (long long p = 2; p <= r; p++) if (!small[p]) { base[nb++] = p; for (long long m = p * p; m <= r; m += p) small[m] = 1; }
    long k = sysconf(_SC_NPROCESSORS_ONLN);
    if (k < 1) k = 1;
    if (k > 256) k = 256;
    pthread_t th[256]; Part parts[256];
    for (long t = 0; t < k; t++) {
        parts[t] = (Part){ comp, base, nb, 2 + (n - 1) * t / k, 1 + (n - 1) * (t + 1) / k, 0 };
        pthread_create(&th[t], NULL, work, &parts[t]);
    }
    long long count = 0;
    for (long t = 0; t < k; t++) { pthread_join(th[t], NULL); count += parts[t].count; }
    return (int)(count % 256);
}
