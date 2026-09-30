function main()
    n = something(tryparse(Int, get(ENV, "N", "")), -1)
    (n < 0 || n > 1_000_000_000) && exit(2)
    acc = 0
    for i in 1:n
        acc = (acc + (i * i) % 7) % 1_000_000_007
    end
    exit(acc % 256)
end
main()
