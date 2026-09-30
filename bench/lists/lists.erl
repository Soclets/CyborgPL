#!/usr/bin/env escript
%%! +S 1
main(_) ->
    N = case catch list_to_integer(os:getenv("N", "")) of
        V when is_integer(V), V >= 0, V =< 100000000 -> V;
        _ -> halt(2)
    end,
    Xs = array:from_list([I rem 1000 || I <- lists:seq(1, N)]),
    Len = array:size(Xs),
    halt(passes(10, Xs, Len, 0) rem 256).
passes(0, _, _, Acc) -> Acc;
passes(R, Xs, Len, Acc) -> passes(R - 1, Xs, Len, add(0, Len, Xs, Acc)).
add(J, Len, _, Acc) when J >= Len -> Acc;
add(J, Len, Xs, Acc) -> add(J + 1, Len, Xs, Acc + array:get(J, Xs)).
