#!/usr/bin/env escript
%%! +S 1
main(_) ->
    N = case catch list_to_integer(os:getenv("N", "")) of
        V when is_integer(V), V >= 0, V =< 1000000000 -> V;
        _ -> halt(2)
    end,
    halt(loop(1, N, 0) rem 256).
loop(I, N, Acc) when I > N -> Acc;
loop(I, N, Acc) -> loop(I + 1, N, (Acc + (I * I) rem 7) rem 1000000007).
