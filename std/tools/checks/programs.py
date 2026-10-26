# programs.py NAME: prints one of std's fixed test programs, kept here and not as .cy files, since every .cy under
# std/ is a module of the kit. argv, argv_round and argv_rdeci read their text from the command line, so Precog
# proves std's sites for any input; emptyat must be refused when building.
import sys

PROGRAMS = {}

PROGRAMS["argv"] = r'''use std::list as list;
# Each argument, read as text: the first is worked on, the second is the part.
fn arg [$argv: ptr[ptr[u8]], $i: i32] -> str.utf8 {
    no_check let $word := ptr.read(ptr.at($argv, by: $i));
    let mut $k: i32 = 0;
    let mut $bytes: list[u8] = [];
    loop {
        no_check let $b := ptr.read(ptr.at($word, by: $k));
        if $b equal_to 0 { break; }
        list.append(lend.mut $bytes, value: $b);
        change $k = $k + 1;
    }
    let $text := to.str.utf8($bytes) otherwise str.empty;
    return give $text;
}

MAIN [$argc: i32, $argv: ptr[ptr[u8]]] {
    if $argc < 3 { std::exit(code: 2); }
    let $s := arg($argv, 1);
    let $p := arg($argv, 2);
    std::print.useful(std::str.contains(lend $s, part: lend $p), " ", std::str.starts_with(lend $s, part: lend $p), " ", std::str.ends_with(lend $s, part: lend $p), newln);
    let $b2 := std::str.at.byte(lend $s, pos: 2) otherwise 0;
    let $p2 := std::str.at.point(lend $s, pos: 2) otherwise `?`;
    std::print.useful($b2, " ", $p2, newln);
    let $g2 := std::str.at.grapheme(lend $s, pos: 2) otherwise `?`;
    let $sg := std::str.slice.grapheme(lend $s, from: pos 1, to: pos 2) otherwise str.empty;
    let $sp := std::str.slice.point(lend $s, from: pos 1, to: pos 2) otherwise str.empty;
    std::print.useful($g2, " [", $sg, "] [", $sp, "]", newln);
    let $sb := std::str.slice.byte(lend $s, from: pos 1, before: pos 3) otherwise [];
    std::print.useful(list.length(lend $sb), newln);
    std::print.useful(std::str.upper.ascii(lend $s), " ", std::str.lower.ascii(lend $s), newln);
    std::print.useful(std::str.upper.simple(lend $s), " ", std::str.lower.simple(lend $s), newln);
    std::print.useful(std::str.upper.full(lend $s), " ", std::str.lower.full(lend $s), newln);
    std::print.useful(std::str.upper.full(lend $s, lang: std::casing.tr), " ", std::str.lower.full(lend $s, lang: std::casing.tr), newln);
    std::print.useful(std::str.upper.full(lend $s, lang: std::casing.lt), " ", std::str.lower.full(lend $s, lang: std::casing.lt), newln);
    if std::str.bytes(lend $p) at_least 1 {
        let $parts := std::str.split(lend $s, at: lend $p);
        std::print.useful(list.length(lend $parts), " ", std::str.join(lend $parts, between: "|"), newln);
        std::print.useful(std::str.replace(lend $s, from: lend $p, to: "<>"), newln);
    }
}
'''

PROGRAMS["argv_round"] = r'''use std::list as list;
# Rounding a bin read from the command line, so Precog proves std's round from std's own code.
fn arg [$argv: ptr[ptr[u8]], $i: i32] -> str.utf8 {
    no_check let $word := ptr.read(ptr.at($argv, by: $i));
    let mut $k: i32 = 0;
    let mut $bytes: list[u8] = [];
    loop {
        no_check let $b := ptr.read(ptr.at($word, by: $k));
        if $b equal_to 0 { break; }
        list.append(lend.mut $bytes, value: $b);
        change $k = $k + 1;
    }
    let $text := to.str.utf8($bytes) otherwise str.empty;
    return give $text;
}

MAIN [$argc: i32, $argv: ptr[ptr[u8]]] {
    if $argc < 3 { std::exit(code: 2); }
    let $xt := arg($argv, 1);
    let $nt := arg($argv, 2);
    let $x: f64 = to.f64($xt) otherwise 0.0;
    let $n: i32[min: 0] = to.i32[min: 0]($nt) otherwise 0;
    let $h: f16 = to.f16($xt) otherwise 0.0;
    std::print.stored(std::round.near($x), " ", std::round.half_up($x), " ", std::round.floor($x), " ", std::round.ceil($x), " ", std::round.trunc($x), newln);
    std::print.stored(std::round.near($x, places: $n), " ", std::round.half_up($x, places: $n), " ", std::round.floor($x, places: $n), " ", std::round.ceil($x, places: $n), " ", std::round.trunc($x, places: $n), newln);
    std::print.stored(std::round.near($h), " ", std::round.near($h, places: $n), newln);
}
'''

PROGRAMS["argv_rdeci"] = r'''use std::list as list;
# Rounding a deci read from the command line, so Precog proves std's round of the decis from std's own code.
fn arg [$argv: ptr[ptr[u8]], $i: i32] -> str.utf8 {
    no_check let $word := ptr.read(ptr.at($argv, by: $i));
    let mut $k: i32 = 0;
    let mut $bytes: list[u8] = [];
    loop {
        no_check let $b := ptr.read(ptr.at($word, by: $k));
        if $b equal_to 0 { break; }
        list.append(lend.mut $bytes, value: $b);
        change $k = $k + 1;
    }
    let $text := to.str.utf8($bytes) otherwise str.empty;
    return give $text;
}


MAIN [$argc: i32, $argv: ptr[ptr[u8]]] {
    if $argc < 3 { std::exit(code: 2); }
    let $xt := arg($argv, 1);
    let $nt := arg($argv, 2);
    let $zero: %d64 = exactly 0;
    let $x: %d64 = to.%d64($xt) otherwise $zero;
    let $n: i32[min: 0] = to.i32[min: 0]($nt) otherwise 0;
    let $zero32: %d32 = exactly 0;
    let $h: %d32 = to.%d32($xt) otherwise $zero32;
    std::print.stored(std::round.near($x), " ", std::round.half_up($x), " ", std::round.floor($x), " ", std::round.ceil($x), " ", std::round.trunc($x), newln);
    std::print.stored(std::round.near($x, places: $n), " ", std::round.half_up($x, places: $n), " ", std::round.floor($x, places: $n), " ", std::round.ceil($x, places: $n), " ", std::round.trunc($x, places: $n), newln);
    std::print.stored(std::round.near($h), " ", std::round.near($h, places: $n), newln);
}
'''

PROGRAMS["emptyat"] = r'''# A written empty `at` is refused when building (owner, 2026-10-14: `at` is `str.utf8[min: 1 bytes]`).
use std::list as list;
MAIN {
    let $s := "a,b";
    let $p := std::str.split(lend $s, at: str.empty);
    std::print.useful(list.length(lend $p), newln);
}
'''

sys.stdout.write(PROGRAMS[sys.argv[1]])
