#include <stdlib.h>
#include <stdbool.h>
int main(void) {
    long n = atoi(getenv("N")), count = 0;
    bool *comp = calloc(n + 1, 1);
    for (long p = 2; p <= n; p++) if (!comp[p]) { count++; for (long m = p * p; m <= n; m += p) comp[m] = true; }
    free(comp);
    return (int)(count % 256);
}
