n = Integer(ENV["N"] || "", exception: false)
exit(2) if n.nil? || n < 0 || n > 92
def fib(k) = k < 2 ? k : fib(k - 1) + fib(k - 2)
exit(fib(n) % 256)
