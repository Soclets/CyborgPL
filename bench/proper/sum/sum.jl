# sum, the proper Julia way: the range in one chunk per thread, each a Threads.@spawn task (run with -t auto).
function part(lo::Int, hi::Int)
    acc = 0
    for i in lo:hi
        acc += (i * i) % 7
    end
    return acc
end

function main()
    n = tryparse(Int, get(ENV, "N", ""))
    (n === nothing || n < 0 || n > 1_000_000_000) && return 2
    k = Threads.nthreads()
    tasks = [Threads.@spawn part(1 + div(n * t, k), div(n * (t + 1), k)) for t in 0:k-1]
    acc = sum(fetch, tasks)
    return Int(acc % 1_000_000_007 % 256)
end

exit(main())
