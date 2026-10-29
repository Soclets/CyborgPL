# Each word's bytes collected and read as UTF-8 once (owner, 2026-09-30),
# so a word past ASCII arrives whole: run with "héllo", "ab" and the one
# byte 0xFF. Points 5 + 2 = 7, bytes 6 + 2 = 8, and 0xFF is not UTF-8
# (100): 7 x 10 + 8 + 100 = 178.
%extern.C proc exit [$code: i32];
MAIN [$argc: i32, $argv: ptr[ptr[u8]]]:
    var $points: u64 = 0;
    var $bytes: u64 = 0;
    var $bad: i32 = 0;
    var $i: i32 = 1;
    while $i < $argc:
        no_check let $word := ptr.read(ptr.at($argv, by: $i));
        var $k: i32 = 0;
        var $raw: std::ls[u8] = [];
        loop:
            no_check let $b := ptr.read(ptr.at($word, by: $k));
            if $b == 0:
                break;
            ;if
            std::ls.append(lend.mut $raw, value: $b);
            change $k to $k + 1;
        ;loop
        let $text := to.str.utf8(lend $raw) ?? str.empty;
        if std::str.bytes(lend $text) == 0:
            change $bad to $bad + 100;
        ;if
        change $points to $points + std::str.points(lend $text);
        change $bytes to $bytes + std::str.bytes(lend $text);
        change $i to $i + 1;
    ;while
    no_check my.exit(to.i32($points x 10 + $bytes) + $bad);
;MAIN
