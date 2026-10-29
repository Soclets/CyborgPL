# A step past the counter's last value is never taken (control-flow.md,
# owner 2026-09-23), and a negative step counts a u64 down.
%extern.C proc exit [let $code: i32];
MAIN:
    var $t: i32 = 0;
    for $i: u8 [from: 1, to: 0, by: -2]:
        change $t to $t + 1;
    ;for
    for $i: u8 [from: 250, to: 255, by: 3]:
        change $t to $t + 1;
    ;for
    for $i: i8 [from: 120, to: 127, by: 5]:
        change $t to $t + 1;
    ;for
    for $i: i8 [from: -120, to: -128, by: -5]:
        change $t to $t + 1;
    ;for
    for $i: u64 [from: 3, to: 1, by: -1]:
        change $t to $t + 1;
    ;for
    for $i: i64 [from: 9223372036854775806, to: 9223372036854775807, by: 2]:
        change $t to $t + 1;
    ;for
    no_check my.exit($t);
;MAIN
