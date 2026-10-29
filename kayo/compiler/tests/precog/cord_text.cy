# cord's text shapes (cord claude, 2026-10-14), on bytes read from the command line, so Precog
# cannot run them: `sits_at`'s two positions in `for 'k' from 'pos' to …` (the counter is never
# below 'pos'), and `count_pieces`' `'text'[pos 'pos']` (the landing of `'pos' plus 'm'` in a
# `u64[min 1]` passed, so 'pos' never wrapped round to 0). All three are proven.
proc word [let $argv: ptr[ptr[u8]], let $i: i32] -> std::ls[u8]:
    no_check let $w := ptr.read(ptr.at($argv, by: $i));
    var $k: i32 = 0;
    var $bytes: std::ls[u8] = [];
    loop:
        no_check let $b := ptr.read(ptr.at($w, by: $k));
        if $b == 0:
            break;
        ;if
        std::ls.append(lend.mut $bytes, value: $b);
        change $k to $k + 1;
    ;loop
    ret give $bytes;
;proc word

fn sits_at [lent $text: std::ls[u8], lent $part: std::ls[u8], let $pos: u64[min: 1]] -> bool:
    let $n := std::ls.len(lend $text);
    let $m := std::ls.len(lend $part);
    if $m == 0:
        ret $pos <= $n + 1;
    ;if
    if $m > $n:
        ret false;
    ;if
    if $pos > $n - $m + 1:
        ret false;
    ;if
    for $k: u64 [from: $pos, to: ($pos + $m - 1), by: 1]:
        if $text[pos: $k] != $part[pos: $k - $pos + 1]:
            ret false;
        ;if
    ;for
    ret true;
;fn sits_at

fn count_pieces [lent $text: std::ls[u8], lent $sep: std::ls[u8]] -> u64:
    let $n := std::ls.len(lend $text);
    let $m := std::ls.len(lend $sep);
    var $pieces: u64 = 1;
    var $piece: std::ls[u8] = [];
    var $pos: u64[min: 1] = 1;
    while $pos <= $n:
        if sits_at(lend $text, lend $sep, $pos):
            change $pieces to $pieces + 1;
            change $piece to std::ls.empty;
            change $pos to $pos + $m;
        else:
            std::ls.append(lend.mut $piece, value: $text[pos: $pos]);
            change $pos to $pos + 1;
        ;if
    ;while
    ret $pieces;
;fn count_pieces

MAIN [let $argc: i32, let $argv: ptr[ptr[u8]]]:
    let $text := word($argv, 1);
    let $sep := word($argv, 2);
    std::print(sits_at(lend $text, lend $sep, 1), " ", count_pieces(lend $text, lend $sep), newln);
;MAIN
