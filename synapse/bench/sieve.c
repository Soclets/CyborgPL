#include <stdlib.h>
#include <stdbool.h>
int main(void) {
    const char *s = getenv("N");
    long long n = s ? atoll(s) : -1;
    if (n < 0 || n > 200000000) return 2;
    long count = 0;
    bool *comp = calloc(n + 1, 1);
    for (long p = 2; p <= n; p++) if (!comp[p]) { count++; for (long m = p * p; m <= n; m += p) comp[m] = true; }
    free(comp);
    return (int)(count % 256);
}
