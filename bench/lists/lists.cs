using System;
using System.Collections.Generic;
class Program {
    static int Main() {
        long n = long.TryParse(Environment.GetEnvironmentVariable("N"), out var v) ? v : -1;
        if (n < 0 || n > 100000000) return 2;
        var xs = new List<long>();
        for (long i = 1; i <= n; i++) xs.Add(i % 1000);
        long acc = 0;
        for (int r = 0; r < 10; r++) for (int j = 0; j < xs.Count; j++) acc += xs[j];
        return (int)(acc % 256);
    }
}
