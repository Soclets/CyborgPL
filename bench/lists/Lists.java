import java.util.ArrayList;
public class Lists {
    public static void main(String[] args) {
        long n;
        try { n = Long.parseLong(System.getenv("N")); } catch (Exception e) { n = -1; }
        if (n < 0 || n > 100000000) System.exit(2);
        ArrayList<Long> xs = new ArrayList<>();
        for (long i = 1; i <= n; i++) xs.add(i % 1000);
        long acc = 0;
        for (int r = 0; r < 10; r++) for (int j = 0; j < xs.size(); j++) acc += xs.get(j);
        System.exit((int) (acc % 256));
    }
}
