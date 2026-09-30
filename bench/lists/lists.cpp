#include <cstdlib>
#include <vector>
int main() {
    const char *s = std::getenv("N");
    long long n = s ? std::atoll(s) : -1;
    if (n < 0 || n > 100000000) return 2;
    std::vector<long long> xs;
    for (long long i = 1; i <= n; i++) xs.push_back(i % 1000);
    long long acc = 0;
    for (int r = 0; r < 10; r++) for (size_t j = 0; j < xs.size(); j++) acc += xs[j];
    return (int)(acc % 256);
}
