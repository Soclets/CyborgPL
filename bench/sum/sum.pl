use strict; use warnings;
my $n = $ENV{N} // "";
exit 2 unless $n =~ /^\d+$/ && $n <= 1000000000;
my $acc = 0;
for my $i (1 .. $n) { $acc = ($acc + ($i * $i) % 7) % 1000000007; }
exit($acc % 256);
