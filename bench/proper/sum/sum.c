// sum, the proper C way, one thread: a plain loop.
#include <stdlib.h>

int main(void) {
    const char *s = getenv("N");
    long long n = s ? atoll(s) : -1;
    if (n < 0 || n > 1000000000) return 2;
    long long acc = 0;
    for (long long i = 1; i <= n; i++) acc += (i * i) % 7;
    return (int)(acc % 1000000007 % 256);
}
