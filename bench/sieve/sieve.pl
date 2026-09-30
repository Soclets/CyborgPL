use strict; use warnings;
my $n = $ENV{N} // "";
exit 2 unless $n =~ /^\d+$/ && $n <= 200000000;
my $comp = "\0" x ($n + 1);
my $count = 0;
for my $p (2 .. $n) {
    next if vec($comp, $p, 8);
    $count++;
    for (my $m = $p * $p; $m <= $n; $m += $p) { vec($comp, $m, 8) = 1; }
}
exit($count % 256);
