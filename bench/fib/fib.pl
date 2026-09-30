use strict; use warnings; no warnings 'recursion';
my $n = $ENV{N} // "";
exit 2 unless $n =~ /^\d+$/ && $n <= 92;
sub fib { my $k = shift; return $k < 2 ? $k : fib($k - 1) + fib($k - 2); }
exit(fib($n) % 256);
