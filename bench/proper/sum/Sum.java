// sum, the proper Java way: a parallel LongStream.
import java.util.stream.LongStream;

public class Sum {
    public static void main(String[] args) {
        long n;
        try { n = Long.parseLong(System.getenv("N")); } catch (Exception e) { n = -1; }
        if (n < 0 || n > 1_000_000_000L) System.exit(2);
        long acc = LongStream.rangeClosed(1, n).parallel().map(i -> i * i % 7).sum();
        System.exit((int) (acc % 1_000_000_007L % 256));
    }
}
