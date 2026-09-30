function main()
    n = something(tryparse(Int, get(ENV, "N", "")), -1)
    (n < 0 || n > 1_000_000_000) && exit(2)
    acc = 0
    for i in 1:n
        acc += (i * i) % 7
    end
    exit(acc % 1_000_000_007 % 256)
end
main()
