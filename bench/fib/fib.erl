#!/usr/bin/env escript
%%! +S 1
main(_) ->
    N = case catch list_to_integer(os:getenv("N", "")) of
        V when is_integer(V), V >= 0, V =< 92 -> V;
        _ -> halt(2)
    end,
    halt(fib(N) rem 256).
fib(K) when K < 2 -> K;
fib(K) -> fib(K - 1) + fib(K - 2).
