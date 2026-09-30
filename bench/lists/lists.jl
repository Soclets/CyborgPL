function main()
    n = something(tryparse(Int, get(ENV, "N", "")), -1)
    (n < 0 || n > 100_000_000) && exit(2)
    xs = Int[]
    for i in 1:n
        push!(xs, i % 1000)
    end
    acc = 0
    for r in 1:10, j in eachindex(xs)
        acc += xs[j]
    end
    exit(acc % 256)
end
main()
