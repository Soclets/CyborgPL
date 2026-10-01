// lists, the proper C way: the size is known, so allocate once and fill.
#include <stdlib.h>

int main(void) {
    const char *s = getenv("N");
    long long n = s ? atoll(s) : -1;
    if (n < 0 || n > 100000000) return 2;
    long long *xs = malloc((size_t)(n ? n : 1) * sizeof *xs);
    if (!xs) return 3;
    for (long long i = 0; i < n; i++) xs[i] = (i + 1) % 1000;
    long long acc = 0;
    for (int r = 0; r < 10; r++)
        for (long long i = 0; i < n; i++) acc += xs[i];
    free(xs);
    return (int)(acc % 256);
}
