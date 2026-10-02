# fib: naive recursion, as the program asks.
fib(n::Int) = n < 2 ? n : fib(n - 1) + fib(n - 2)

function main()
    n = tryparse(Int, get(ENV, "N", ""))
    (n === nothing || n < 0 || n > 92) && return 2
    return Int(fib(n) % 256)
end

exit(main())
