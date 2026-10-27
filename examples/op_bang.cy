# `!` is free to declare as a custom operator, as `^` and `|` are (`operators.md`): between two values,
# declared, it is an operator like any other. Its old meaning, `! $x` for `not $x`, stays refused,
# naming `not`. (`&` was free too until 2026-10-14, when it became the language's own `and`.)

# a over b: how many times b fits in a
operator ! [$a: integer.signed.32_bit, $b: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a divide.floor $b;
end of operator not

!MAIN:
    std::print.shortest((17 ! 5), newline);
    std::print.shortest(not true, newline);
!end of MAIN
