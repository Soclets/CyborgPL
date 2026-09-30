#include <cstdlib>
#include <vector>
int main() {
    const char *s = std::getenv("N");
    long long n = s ? std::atoll(s) : -1;
    if (n < 0 || n > 200000000) return 2;
    std::vector<char> comp(n + 1, 0);
    long long count = 0;
    for (long long p = 2; p <= n; p++) if (!comp[p]) { count++; for (long long m = p * p; m <= n; m += p) comp[m] = 1; }
    return (int)(count % 256);
}
