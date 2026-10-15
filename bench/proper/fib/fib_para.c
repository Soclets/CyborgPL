// fib, in parallel the proper C way: the top of the recursion tree split into tasks, one pthread each;
// below that, naive recursion as the program asks.
#include <pthread.h>
#include <stdlib.h>

static long long fib(long long n) { return n < 2 ? n : fib(n - 1) + fib(n - 2); }

typedef struct { long long n, r; } Task;
static void *work(void *arg) { Task *t = arg; t->r = fib(t->n); return NULL; }

int main(void) {
    const char *s = getenv("N");
    long long n = s ? atoll(s) : -1;
    if (n < 0 || n > 92) return 2;
    // fib(n) = sum of fib over the tree's nodes 4 levels down (16 of them), or the leaves above that
    long long args[64]; int k = 0; long long done = 0;
    long long stack[64]; int depth[64]; int sp = 0;
    stack[sp] = n; depth[sp++] = 0;
    while (sp) {
        long long m = stack[--sp]; int d = depth[sp];
        if (m < 2) { done += m; continue; }
        if (d == 4) { args[k++] = m; continue; }
        stack[sp] = m - 1; depth[sp++] = d + 1;
        stack[sp] = m - 2; depth[sp++] = d + 1;
    }
    pthread_t th[64]; Task ts[64];
    for (int i = 0; i < k; i++) { ts[i] = (Task){ args[i], 0 }; pthread_create(&th[i], NULL, work, &ts[i]); }
    for (int i = 0; i < k; i++) { pthread_join(th[i], NULL); done += ts[i].r; }
    return (int)(done % 256);
}
