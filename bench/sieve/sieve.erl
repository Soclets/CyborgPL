#!/usr/bin/env escript
%%! +S 1
main(_) ->
    N = case catch list_to_integer(os:getenv("N", "")) of
        V when is_integer(V), V >= 0, V =< 200000000 -> V;
        _ -> halt(2)
    end,
    Comp = atomics:new(N + 1, [{signed, false}]),
    halt(sieve(2, N, Comp, 0) rem 256).
sieve(P, N, _, C) when P > N -> C;
sieve(P, N, Comp, C) ->
    case atomics:get(Comp, P + 1) of
        0 -> mark(P * P, P, N, Comp), sieve(P + 1, N, Comp, C + 1);
        _ -> sieve(P + 1, N, Comp, C)
    end.
mark(M, _, N, _) when M > N -> ok;
mark(M, P, N, Comp) -> atomics:put(Comp, M + 1, 1), mark(M + P, P, N, Comp).
