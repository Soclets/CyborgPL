fib(n) = n < 2 ? n : fib(n - 1) + fib(n - 2)
function main()
    n = something(tryparse(Int, get(ENV, "N", "")), -1)
    (n < 0 || n > 92) && exit(2)
    exit(fib(n) % 256)
end
main()
