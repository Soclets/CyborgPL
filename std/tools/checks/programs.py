# programs.py NAME: prints one of std's fixed test programs, kept here and not as .cy files, since every .cy under
# std/ is a module of the kit. argv, argv_round and argv_rdeci read their text from the command line, so Precog
# proves std's sites for any input; emptyat must be refused when building.
import sys

PROGRAMS = {}

PROGRAMS["argv"] = r'''use std::ls as ls;
# Each argument, read as text: the first is worked on, the second is the part.
proc arg [let $argv: ptr[ptr[u8]], let $i: i32] -> str.utf8:
    no_check let $word := ptr.read(ptr.at($argv, by: $i));
    var $k: i32 = 0;
    var $bytes: ls[u8] = [];
    loop:
        no_check let $b := ptr.read(ptr.at($word, by: $k));
        if $b == 0:
            break;
        ;if
        ls.append(lend.mut $bytes, value: $b);
        change $k to $k + 1;
    ;loop
    let $text := to.str.utf8(lend $bytes) ?? str.empty;
    ret give $text;
;proc arg

MAIN [let $argc: i32, let $argv: ptr[ptr[u8]]]:
    if $argc < 3:
        std::exit(code: 2);
    ;if
    let $s := arg($argv, 1);
    let $p := arg($argv, 2);
    std::print(std::str.contains(lend $s, part: lend $p), " ", std::str.starts_with(lend $s, part: lend $p), " ", std::str.ends_with(lend $s, part: lend $p), newln);
    let $b2 := std::str.at.byte(lend $s, pos: 2) ?? 0;
    let $p2 := std::str.at.point(lend $s, pos: 2) ?? `?`;
    std::print($b2, " ", $p2, newln);
    let $g2 := std::str.at.cluster(lend $s, pos: 2) ?? `?`;
    let $sg := std::str.slice.cluster(lend $s, from: pos 1, to: pos 2) ?? str.empty;
    let $sp := std::str.slice.point(lend $s, from: pos 1, to: pos 2) ?? str.empty;
    std::print($g2, " [", $sg, "] [", $sp, "]", newln);
    let $sb := std::str.slice.byte(lend $s, from: pos 1, before: pos 3) ?? [];
    std::print(ls.len(lend $sb), newln);
    std::print(std::str.upper.ascii(lend $s), " ", std::str.lower.ascii(lend $s), newln);
    std::print(std::str.upper.simple(lend $s), " ", std::str.lower.simple(lend $s), newln);
    std::print(std::str.upper.full(lend $s), " ", std::str.lower.full(lend $s), newln);
    std::print(std::str.upper.full(lend $s, lang: std::casing.tr), " ", std::str.lower.full(lend $s, lang: std::casing.tr), newln);
    std::print(std::str.upper.full(lend $s, lang: std::casing.lt), " ", std::str.lower.full(lend $s, lang: std::casing.lt), newln);
    if std::str.bytes(lend $p) >= 1:
        let $parts := std::str.split(lend $s, at: lend $p);
        std::print(ls.len(lend $parts), " ", std::str.join(lend $parts, between: "|"), newln);
        std::print(std::str.replace(lend $s, from: lend $p, to: "<>"), newln);
    ;if
;MAIN
'''

PROGRAMS["argv_round"] = r'''use std::ls as ls;
# Rounding a bin read from the command line, so Precog proves std's round from std's own code.
proc arg [let $argv: ptr[ptr[u8]], let $i: i32] -> str.utf8:
    no_check let $word := ptr.read(ptr.at($argv, by: $i));
    var $k: i32 = 0;
    var $bytes: ls[u8] = [];
    loop:
        no_check let $b := ptr.read(ptr.at($word, by: $k));
        if $b == 0:
            break;
        ;if
        ls.append(lend.mut $bytes, value: $b);
        change $k to $k + 1;
    ;loop
    let $text := to.str.utf8(lend $bytes) ?? str.empty;
    ret give $text;
;proc arg

MAIN [let $argc: i32, let $argv: ptr[ptr[u8]]]:
    if $argc < 3:
        std::exit(code: 2);
    ;if
    let $xt := arg($argv, 1);
    let $nt := arg($argv, 2);
    let $x: f64 := to.f64(lend $xt) ?? 0.0;
    let $n: i32[min: 0] := to.i32[min: 0](lend $nt) ?? 0;
    let $h: f16 := to.f16(lend $xt) ?? 0.0;
    std::print.stored(std::round.near($x), " ", std::round.half_up($x), " ", std::round.floor($x), " ", std::round.ceil($x), " ", std::round.trunc($x), newln);
    std::print.stored(std::round.near($x, places: $n), " ", std::round.half_up($x, places: $n), " ", std::round.floor($x, places: $n), " ", std::round.ceil($x, places: $n), " ", std::round.trunc($x, places: $n), newln);
    std::print.stored(std::round.near($h), " ", std::round.near($h, places: $n), newln);
;MAIN
'''

PROGRAMS["argv_rdeci"] = r'''use std::ls as ls;
# Rounding a deci read from the command line, so Precog proves std's round of the decis from std's own code.
proc arg [let $argv: ptr[ptr[u8]], let $i: i32] -> str.utf8:
    no_check let $word := ptr.read(ptr.at($argv, by: $i));
    var $k: i32 = 0;
    var $bytes: ls[u8] = [];
    loop:
        no_check let $b := ptr.read(ptr.at($word, by: $k));
        if $b == 0:
            break;
        ;if
        ls.append(lend.mut $bytes, value: $b);
        change $k to $k + 1;
    ;loop
    let $text := to.str.utf8(lend $bytes) ?? str.empty;
    ret give $text;
;proc arg


MAIN [let $argc: i32, let $argv: ptr[ptr[u8]]]:
    if $argc < 3:
        std::exit(code: 2);
    ;if
    let $xt := arg($argv, 1);
    let $nt := arg($argv, 2);
    let $zero: %d64 := exactly 0;
    let $x: %d64 := to.%d64(lend $xt) ?? $zero;
    let $n: i32[min: 0] := to.i32[min: 0](lend $nt) ?? 0;
    let $zero32: %d32 := exactly 0;
    let $h: %d32 := to.%d32(lend $xt) ?? $zero32;
    std::print.stored(std::round.near($x), " ", std::round.half_up($x), " ", std::round.floor($x), " ", std::round.ceil($x), " ", std::round.trunc($x), newln);
    std::print.stored(std::round.near($x, places: $n), " ", std::round.half_up($x, places: $n), " ", std::round.floor($x, places: $n), " ", std::round.ceil($x, places: $n), " ", std::round.trunc($x, places: $n), newln);
    std::print.stored(std::round.near($h), " ", std::round.near($h, places: $n), newln);
;MAIN
'''

PROGRAMS["emptyat"] = r'''# A written empty `at` is refused when building (owner, 2026-10-14: `at` is `str.utf8[min: 1 bytes]`).
use std::ls as ls;
MAIN:
    let $s := "a,b";
    let $p := std::str.split(lend $s, at: str.empty);
    std::print(ls.len(lend $p), newln);
;MAIN
'''

sys.stdout.write(PROGRAMS[sys.argv[1]])
