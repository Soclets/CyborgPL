#include <cstdlib>
static long long fib(long long n) { return n < 2 ? n : fib(n - 1) + fib(n - 2); }
int main() {
    const char *s = std::getenv("N");
    long long n = s ? std::atoll(s) : -1;
    if (n < 0 || n > 92) return 2;
    return (int)(fib(n) % 256);
}
