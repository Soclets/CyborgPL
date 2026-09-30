using System;
class Program {
    static long Fib(long n) => n < 2 ? n : Fib(n - 1) + Fib(n - 2);
    static int Main() {
        long n = long.TryParse(Environment.GetEnvironmentVariable("N"), out var v) ? v : -1;
        if (n < 0 || n > 92) return 2;
        return (int)(Fib(n) % 256);
    }
}
