// sieve, the proper Java way: a boolean[], crossing off from p*p.
public class Sieve {
    public static void main(String[] args) {
        int n;
        try { n = Integer.parseInt(System.getenv("N")); } catch (Exception e) { n = -1; }
        if (n < 0 || n > 200_000_000) System.exit(2);
        boolean[] comp = new boolean[n + 1];
        for (long p = 2; p * p <= n; p++)
            if (!comp[(int) p])
                for (long m = p * p; m <= n; m += p) comp[(int) m] = true;
        int count = 0;
        for (int i = 2; i <= n; i++) if (!comp[i]) count++;
        System.exit(count % 256);
    }
}
