public class Fib {
    static long fib(long n) { return n < 2 ? n : fib(n - 1) + fib(n - 2); }
    public static void main(String[] args) {
        long n;
        try { n = Long.parseLong(System.getenv("N")); } catch (Exception e) { n = -1; }
        if (n < 0 || n > 92) System.exit(2);
        System.exit((int) (fib(n) % 256));
    }
}
