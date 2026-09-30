#include <cstdlib>
int main() {
    const char *s = std::getenv("N");
    long long n = s ? std::atoll(s) : -1;
    if (n < 0 || n > 1000000000) return 2;
    long long acc = 0;
    for (long long i = 1; i <= n; i++) acc += (i * i) % 7;
    return (int)(acc % 1000000007 % 256);
}
