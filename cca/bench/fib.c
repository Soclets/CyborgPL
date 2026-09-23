#include <stdlib.h>
static long fib(long n) { return n < 2 ? n : fib(n - 1) + fib(n - 2); }
int main(void) { return (int)(fib(atoi(getenv("N"))) % 256); }
