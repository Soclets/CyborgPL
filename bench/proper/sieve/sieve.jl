# sieve, the proper Julia way: a Vector{Bool} (one byte per number), crossing off from p*p by ranges.
function main()
    n = tryparse(Int, get(ENV, "N", ""))
    (n === nothing || n < 0 || n > 200_000_000) && return 2
    comp = zeros(Bool, max(n, 1))
    p = 2
    while p * p <= n
        if !comp[p]
            comp[p*p:p:n] .= true
        end
        p += 1
    end
    count = n < 2 ? 0 : Base.count(!, @view comp[2:n])
    return Int(count % 256)
end

exit(main())
