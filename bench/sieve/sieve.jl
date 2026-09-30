function main()
    n = something(tryparse(Int, get(ENV, "N", "")), -1)
    (n < 0 || n > 200_000_000) && exit(2)
    comp = falses(n + 1)
    count = 0
    for p in 2:n
        if !comp[p + 1]
            count += 1
            m = p * p
            while m <= n
                comp[m + 1] = true
                m += p
            end
        end
    end
    exit(count % 256)
end
main()
