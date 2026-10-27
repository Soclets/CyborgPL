# `!` is free to declare as a custom operator, as `^` and `|` are (`operators.md`): between two values,
# declared, it is an operator like any other. Its old meaning, `! $x` for `not $x`, stays refused,
# naming `not`. (`&` was free too until 2026-10-14, when it became the language's own `and`.)

# a over b: how many times b fits in a
op ! [$a: int.sign.32, $b: int.sign.32] -> int.sign.32:
    ret $a div.floor $b;
end of op not

!MAIN:
    std::print.shortest((17 ! 5), newln);
    std::print.shortest(not true, newln);
!end of MAIN
