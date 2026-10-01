// sieve, the proper C way: one byte per number from calloc, crossing off from p*p.
#include <stdlib.h>

int main(void) {
    const char *s = getenv("N");
    long long n = s ? atoll(s) : -1;
    if (n < 0 || n > 200000000) return 2;
    unsigned char *comp = calloc((size_t)n + 1, 1);
    if (!comp) return 3;
    for (long long p = 2; p * p <= n; p++)
        if (!comp[p])
            for (long long m = p * p; m <= n; m += p) comp[m] = 1;
    long long count = 0;
    for (long long i = 2; i <= n; i++) count += !comp[i];
    free(comp);
    return (int)(count % 256);
}
