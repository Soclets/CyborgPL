using System;
class Program {
    static int Main() {
        long n = long.TryParse(Environment.GetEnvironmentVariable("N"), out var v) ? v : -1;
        if (n < 0 || n > 200000000) return 2;
        var comp = new bool[n + 1];
        long count = 0;
        for (long p = 2; p <= n; p++) if (!comp[p]) { count++; for (long m = p * p; m <= n; m += p) comp[m] = true; }
        return (int)(count % 256);
    }
}
