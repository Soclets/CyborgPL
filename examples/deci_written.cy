# Text into a deci goes by the value: "1234567.0" is 1234567, seven digits, and a d32 holds seven, as
# `to.i32(2.0)` is 2. `.written` counts the digits as written instead, trailing zeros included
# (owner, 2026-10-09: "I think there should be a way to say what you want", then "yea, `.written`").
MAIN:
    let $a := to.%d32("1234567.0")-
    otherwise:
        case not_a_number:
            is exactly 0.0;
        ;case not_a_number
        case too_many_digits:
            is exactly 0.0;
        ;case too_many_digits
        case out_of_range:
            is exactly 0.0;
        ;case out_of_range
    ;otherwise;
    let $b := to.%d32.written("1234567.0")-
    otherwise:
        case not_a_number:
            is exactly 0.0;
        ;case not_a_number
        case too_many_digits:
            is exactly 9.0;
        ;case too_many_digits
        case out_of_range:
            is exactly 0.0;
        ;case out_of_range
    ;otherwise;
    let $c := to.%d32.written("1234567")-
    otherwise:
        case not_a_number:
            is exactly 0.0;
        ;case not_a_number
        case too_many_digits:
            is exactly 9.0;
        ;case too_many_digits
        case out_of_range:
            is exactly 0.0;
        ;case out_of_range
    ;otherwise;
    std::println($a, " ", $b, " ", $c);
;MAIN
