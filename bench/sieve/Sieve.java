public class Sieve {
    public static void main(String[] args) {
        long n;
        try { n = Long.parseLong(System.getenv("N")); } catch (Exception e) { n = -1; }
        if (n < 0 || n > 200000000) System.exit(2);
        int un = (int) n;
        boolean[] comp = new boolean[un + 1];
        long count = 0;
        for (long p = 2; p <= n; p++) if (!comp[(int) p]) { count++; for (long m = p * p; m <= n; m += p) comp[(int) m] = true; }
        System.exit((int) (count % 256));
    }
}
