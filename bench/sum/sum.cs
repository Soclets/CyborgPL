using System;
class Program {
    static int Main() {
        long n = long.TryParse(Environment.GetEnvironmentVariable("N"), out var v) ? v : -1;
        if (n < 0 || n > 1000000000L) return 2;
        long acc = 0;
        for (long i = 1; i <= n; i++) acc += (i * i) % 7;
        return (int)(acc % 1000000007L % 256);
    }
}
