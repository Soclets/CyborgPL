#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
static long n; static int T; static const char *mode; static int *xs; static long part[64];
static void *work(void *a) {
    long t = (long)a, acc = 0;
    if (!strcmp(mode, "sum")) { for (long i = 1 + t; i <= n; i += T) acc = (acc + (i * i) % 7); }
    else if (!strcmp(mode, "map")) { long lo = n * t / T, hi = n * (t + 1) / T; for (long i = lo; i < hi; i++) xs[i] = xs[i] * 3 + 1; }
    else { long w = n; for (long y = t; y < w; y += T) for (long x = 0; x < w; x++) {
        float cr = -2.0f + 3.0f * x / w, ci = -1.5f + 3.0f * y / w, zr = 0, zi = 0; int k = 0;
        for (; k < 256 && zr * zr + zi * zi <= 4.0f; k++) { float u = zr * zr - zi * zi + cr; zi = 2 * zr * zi + ci; zr = u; }
        if (k == 256) acc++; } }
    part[t] = acc; return 0;
}
int main(int argc, char **argv) {
    mode = argv[1]; T = atoi(argv[2]); n = atol(getenv("N"));
    if (!strcmp(mode, "map")) { xs = malloc(n * 4); for (long i = 0; i < n; i++) xs[i] = (int)(i % 1000); }
    pthread_t th[64]; for (long t = 0; t < T; t++) pthread_create(&th[t], 0, work, (void *)t);
    long r = 0; for (int t = 0; t < T; t++) { pthread_join(th[t], 0); r += part[t]; }
    if (!strcmp(mode, "sum")) r %= 1000000007;
    if (!strcmp(mode, "map")) for (long i = 0; i < n; i++) r += xs[i];
    printf("%ld\n", r); return 0;
}
