public class Sum {
    public static void main(String[] args) {
        long n;
        try { n = Long.parseLong(System.getenv("N")); } catch (Exception e) { n = -1; }
        if (n < 0 || n > 1000000000L) System.exit(2);
        long acc = 0;
        for (long i = 1; i <= n; i++) acc += (i * i) % 7;
        System.exit((int) (acc % 1000000007L % 256));
    }
}
