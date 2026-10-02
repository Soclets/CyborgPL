# lists, the proper Julia way: a comprehension builds the array at its known size; `sum` adds it.
function main()
    n = tryparse(Int, get(ENV, "N", ""))
    (n === nothing || n < 0 || n > 100_000_000) && return 2
    xs = [i % 1000 for i in 1:n]
    acc = 0
    for _ in 1:10
        acc += sum(xs)
    end
    return Int(acc % 256)
end

exit(main())
