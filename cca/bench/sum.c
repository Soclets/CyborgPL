#include <stdlib.h>
int main(void) {
    long n = atoi(getenv("N")), acc = 0;
    for (long i = 1; i <= n; i++) acc = (acc + (i * i) % 7) % 1000000007;
    return (int)(acc % 256);
}
