use strict; use warnings;
my $n = $ENV{N} // "";
exit 2 unless $n =~ /^\d+$/ && $n <= 100000000;
my @xs;
for my $i (1 .. $n) { push @xs, $i % 1000; }
my $acc = 0;
for my $r (1 .. 10) { for my $j (0 .. $#xs) { $acc += $xs[$j]; } }
exit($acc % 256);
