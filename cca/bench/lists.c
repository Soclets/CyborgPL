#include <stdlib.h>
int main(void) {
    const char *s = getenv("N");
    long long n = s ? atoll(s) : -1;
    if (n < 0 || n > 100000000) return 2;
    long len = 0, cap = 0, acc = 0;
    long *xs = NULL;
    for (long i = 1; i <= n; i++) {
        if (len == cap) { cap = cap ? cap * 2 : 4; xs = realloc(xs, cap * sizeof *xs); }
        xs[len++] = i % 1000;
    }
    for (int r = 0; r < 10; r++) for (long j = 0; j < len; j++) acc += xs[j];
    free(xs);
    return (int)(acc % 256);
}
