# C's arguments (owner, 2026-09-29): run as `cca run words.cpls -- apples 3
# "hello world"`. argc is 4 (the program's name first); the first word's
# first byte is `a`, 97; the last word is 11 bytes long; argv[argc] is
# null. 4 + 97 + 11 + 100 = 212.
%extern.C proc exit [let $code: i32];
MAIN [let $argc: i32, let $argv: ptr[ptr[u8]]]:
    no_check let $first := ptr.read(ptr.at($argv, by: 1));
    no_check let $a := ptr.read($first);
    no_check let $last := ptr.read(ptr.at($argv, by: $argc - 1));
    var $n: i32 = 0;
    loop:
        no_check let $b := ptr.read(ptr.at($last, by: $n));
        if $b == 0:
            break;
        ;if
        change $n to $n + 1;
    ;loop
    no_check let $end := ptr.read(ptr.at($argv, by: $argc));
    var $null: i32 = 0;
    if ptr.is_null($end):
        change $null to 100;
    ;if
    no_check my.exit($argc + to.i32($a) + $n + $null);
;MAIN
