// lists, the proper Java way: a primitive long[] built by a LongStream (no boxing), summed by a stream.
import java.util.Arrays;
import java.util.stream.LongStream;

public class Lists {
    public static void main(String[] args) {
        long n;
        try { n = Long.parseLong(System.getenv("N")); } catch (Exception e) { n = -1; }
        if (n < 0 || n > 100_000_000L) System.exit(2);
        long[] xs = LongStream.rangeClosed(1, n).map(i -> i % 1000).toArray();
        long acc = 0;
        for (int r = 0; r < 10; r++) acc += Arrays.stream(xs).sum();
        System.exit((int) (acc % 256));
    }
}
